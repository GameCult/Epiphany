from __future__ import annotations

import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import sys
from tempfile import TemporaryDirectory
from typing import Any

from epiphany_agent_memory import DEFAULT_AGENT_DIR
from epiphany_agent_memory import ROLE_TARGETS
from epiphany_agent_memory import apply_self_patch
from epiphany_agent_memory import validate_all


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_HEARTBEAT_STATE = ROOT / "state" / "agent-heartbeats.json"
DEFAULT_ARTIFACT_DIR = ROOT / ".epiphany-heartbeats"
SCHEMA_VERSION = "epiphany.agent_heartbeat.v0"
INITIATIVE_SCHEMA_VERSION = "ghostlight.initiative_schedule.v0"
GHOSTLIGHT_ACTION_TYPES = {
    "speak",
    "silence",
    "move",
    "gesture",
    "touch_object",
    "block_object",
    "use_object",
    "show_object",
    "withhold_object",
    "transfer_object",
    "spend_resource",
    "attack",
    "wait",
    "mixed",
}
GHOSTLIGHT_ACTION_SCALES = {"micro", "short", "standard", "major", "committed"}
GHOSTLIGHT_PARTICIPANT_STATUSES = {"active", "blocked", "withdrawn", "incapacitated", "offscreen"}

ROLE_ORDER = [
    "coordinator",
    "imagination",
    "research",
    "modeling",
    "implementation",
    "verification",
    "reorientation",
]

DISPLAY_NAMES = {
    "coordinator": "Self",
    "imagination": "Imagination",
    "research": "Eyes",
    "modeling": "Body",
    "implementation": "Hands",
    "verification": "Soul",
    "reorientation": "Life",
}

INITIATIVE_SPEEDS = {
    "coordinator": 1.28,
    "imagination": 0.82,
    "research": 0.78,
    "modeling": 0.92,
    "implementation": 0.74,
    "verification": 0.88,
    "reorientation": 1.04,
}

REACTION_BIAS = {
    "coordinator": 0.88,
    "imagination": 0.54,
    "research": 0.62,
    "modeling": 0.74,
    "implementation": 0.58,
    "verification": 0.82,
    "reorientation": 0.86,
}

INTERRUPT_THRESHOLD = {
    "coordinator": 0.42,
    "imagination": 0.64,
    "research": 0.58,
    "modeling": 0.5,
    "implementation": 0.5,
    "verification": 0.48,
    "reorientation": 0.44,
}

WORK_ACTIONS = {
    "prepareCheckpoint": "coordinator",
    "continueImplementation": "implementation",
    "launchImagination": "imagination",
    "readImaginationResult": "imagination",
    "reviewImaginationResult": "imagination",
    "launchModeling": "modeling",
    "readModelingResult": "modeling",
    "reviewModelingResult": "modeling",
    "launchVerification": "verification",
    "readVerificationResult": "verification",
    "reviewVerificationResult": "verification",
    "launchReorientWorker": "reorientation",
    "readReorientResult": "reorientation",
    "acceptReorientResult": "reorientation",
    "compactRehydrateReorient": "reorientation",
    "regatherManually": "reorientation",
}


def now_iso() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def default_state(*, target_heartbeat_rate: float = 1.0) -> dict[str, Any]:
    return {
        "schema_version": SCHEMA_VERSION,
        "target_heartbeat_rate": target_heartbeat_rate,
        "scene_clock": 0.0,
        "selection_policy": {
            "mode": "readiness_queue",
            "reaction_precedence": True,
            "minimum_speed": 0.2,
            "tie_breakers": [
                "reaction_readiness_desc",
                "next_ready_at_asc",
                "initiative_speed_desc",
                "stable_actor_id_asc",
            ],
        },
        "participants": [
            {
                "agent_id": agent_id_for_role(role_id),
                "role_id": role_id,
                "display_name": DISPLAY_NAMES[role_id],
                "initiative_speed": INITIATIVE_SPEEDS[role_id],
                "next_ready_at": 0.0,
                "reaction_bias": REACTION_BIAS[role_id],
                "interrupt_threshold": INTERRUPT_THRESHOLD[role_id],
                "current_load": 0.0,
                "status": "active",
                "constraints": participant_constraints(role_id),
                "last_action_id": None,
                "last_woke_at": None,
            }
            for role_id in ROLE_ORDER
        ],
        "history": [],
    }


def agent_id_for_role(role_id: str) -> str:
    return ROLE_TARGETS[role_id][0]


def participant_constraints(role_id: str) -> list[str]:
    base = [
        "Runs Ghostlight-shaped persistent role memory.",
        "May improve lane memory when awake and idle.",
        "Project truth belongs in EpiphanyThreadState, not role memory.",
    ]
    role_specific = {
        "coordinator": "Routes and reviews; must not implement, verify, or accept its own comfort.",
        "imagination": "Synthesizes futures; must not adopt objectives.",
        "research": "Scouts known work; must not turn research into procrastination.",
        "modeling": "Grows source-grounded maps and checkpoints; must not edit implementation code.",
        "implementation": "Touches source only with accepted guidance and verifier-readable evidence.",
        "verification": "Falsifies promises; must not bless theater.",
        "reorientation": "Protects continuity; must not fake survived context.",
    }
    return [*base, role_specific[role_id]]


def load_state(path: Path, *, target_heartbeat_rate: float) -> dict[str, Any]:
    if not path.exists():
        state = default_state(target_heartbeat_rate=target_heartbeat_rate)
        write_json(path, state)
        return state
    state = load_json(path)
    if state.get("schema_version") != SCHEMA_VERSION:
        raise ValueError(f"{path} has wrong schema_version")
    if target_heartbeat_rate > 0:
        state["target_heartbeat_rate"] = target_heartbeat_rate
    present = {item.get("role_id") for item in state.get("participants", []) if isinstance(item, dict)}
    for role_id in ROLE_ORDER:
        if role_id not in present:
            state.setdefault("participants", []).append(default_state()["participants"][ROLE_ORDER.index(role_id)])
    return state


def work_role_for_action(action: str | None, target_role: str | None) -> str | None:
    if target_role in ROLE_TARGETS:
        return target_role
    if action in WORK_ACTIONS:
        return WORK_ACTIONS[action]
    return None


def participant_by_role(state: dict[str, Any], role_id: str) -> dict[str, Any]:
    for participant in state["participants"]:
        if participant.get("role_id") == role_id:
            return participant
    raise KeyError(role_id)


def active_participants(state: dict[str, Any]) -> list[dict[str, Any]]:
    return [item for item in state["participants"] if item.get("status") == "active"]


def readiness_snapshot(
    participants: list[dict[str, Any]],
    *,
    work_role: str | None,
    urgency: float,
) -> list[dict[str, Any]]:
    snapshot: list[dict[str, Any]] = []
    for participant in participants:
        eligible = participant.get("role_id") == work_role and work_role is not None
        reaction_readiness = None
        if eligible:
            reaction_readiness = round(
                float(participant["reaction_bias"]) * urgency - float(participant.get("current_load", 0.0)),
                6,
            )
        snapshot.append(
            {
                "agent_id": participant["agent_id"],
                "next_ready_at": participant["next_ready_at"],
                "reaction_readiness": reaction_readiness,
                "eligible_for_reaction": eligible,
            }
        )
    return snapshot


def select_participant(
    state: dict[str, Any],
    *,
    work_role: str | None,
    urgency: float,
) -> tuple[dict[str, Any], str, str | None]:
    participants = active_participants(state)
    if not participants:
        raise ValueError("heartbeat has no active participants")
    if work_role:
        candidate = participant_by_role(state, work_role)
        reaction_readiness = float(candidate["reaction_bias"]) * urgency - float(candidate.get("current_load", 0.0))
        if (
            candidate.get("status") == "active"
            and reaction_readiness >= float(candidate["interrupt_threshold"])
        ):
            return candidate, "reaction_interrupt", (
                f"{candidate['display_name']} won a heartbeat reaction window for pending {work_role} work."
            )
    selected = min(
        participants,
        key=lambda item: (
            float(item["next_ready_at"]),
            -float(item["initiative_speed"]),
            str(item["agent_id"]),
        ),
    )
    return selected, "scheduled_turn", (
        "No pending work cleared a reaction threshold; earliest ready active lane won the heartbeat slot."
    )


def action_for_selection(
    selected: dict[str, Any],
    *,
    work_role: str | None,
    coordinator_action: str | None,
) -> tuple[str, str, float, float, float, str]:
    role_id = selected["role_id"]
    heartbeat_rate = 1.0
    if role_id == work_role:
        action_id = f"heartbeat.{role_id}.work"
        return (
            action_id,
            "mixed",
            6.0 / heartbeat_rate,
            4.0,
            0.45,
            f"Wake {selected['display_name']} for coordinator action {coordinator_action or 'pending work'}.",
        )
    action_id = f"heartbeat.{role_id}.ruminate"
    return (
        action_id,
        "wait",
        2.0 / heartbeat_rate,
        1.0,
        0.9,
        f"{selected['display_name']} has no actionable lane work; ruminate and distill role memory.",
    )


def rumination_patch(role_id: str, action_id: str) -> dict[str, Any]:
    agent_id = agent_id_for_role(role_id)
    display_name = DISPLAY_NAMES[role_id]
    memory_id = f"mem-{role_id}-heartbeat-rumination"
    goal_id = f"goal-{role_id}-heartbeat-self-distill"
    return {
        "agentId": agent_id,
        "reason": (
            f"{display_name} won an idle heartbeat slot and should preserve the habit of using idle wakeups "
            "to distill role memory instead of manufacturing project work."
        ),
        "semanticMemories": [
            {
                "memoryId": memory_id,
                "summary": (
                    "When a heartbeat wakes this lane and no coordinator-approved work is available, "
                    "the correct move is to ruminate on role quality, cut stale memory, and return a bounded "
                    "self-memory improvement rather than inventing project authority."
                ),
                "salience": 0.78,
                "confidence": 0.88,
            }
        ],
        "goals": [
            {
                "goalId": goal_id,
                "description": "Use idle heartbeat slots to become sharper at this lane's own work before touching project state.",
                "scope": "life",
                "priority": 0.82,
                "emotionalStake": "An idle organ that invents work becomes noise in the bloodstream.",
                "status": "active",
            }
        ],
        "privateNotes": [
            f"Last idle heartbeat action `{action_id}` chose rumination over fake urgency.",
        ],
    }


def tick_once(
    state: dict[str, Any],
    *,
    coordinator_action: str | None,
    target_role: str | None,
    urgency: float,
    apply_rumination: bool,
    agent_dir: Path,
    schedule_id: str,
    source_scene_ref: str,
) -> dict[str, Any]:
    rate = max(float(state.get("target_heartbeat_rate", 1.0)), 0.001)
    work_role = work_role_for_action(coordinator_action, target_role)
    selected, selection_kind, selection_reason = select_participant(
        state,
        work_role=work_role,
        urgency=urgency,
    )
    action_id, action_type, base_recovery, initiative_cost, interruptibility, action_reason = (
        action_for_selection(selected, work_role=work_role, coordinator_action=coordinator_action)
    )
    base_recovery = base_recovery / rate
    scene_clock = max(float(state["scene_clock"]), float(selected["next_ready_at"]))
    recovery = base_recovery / max(float(selected["initiative_speed"]), float(state["selection_policy"]["minimum_speed"]))
    selected["next_ready_at"] = round(scene_clock + recovery, 6)
    selected["last_action_id"] = action_id
    selected["last_woke_at"] = now_iso()
    selected["current_load"] = round(min(1.0, max(0.0, float(selected.get("current_load", 0.0)) * 0.75)), 6)
    state["scene_clock"] = round(scene_clock, 6)

    snapshot = readiness_snapshot(active_participants(state), work_role=work_role, urgency=urgency)
    schedule = {
        "schema_version": INITIATIVE_SCHEMA_VERSION,
        "schedule_id": schedule_id,
        "source_scene_ref": source_scene_ref,
        "scene_clock": state["scene_clock"],
        "participants": [
            {
                "agent_id": item["agent_id"],
                "display_name": item["display_name"],
                "initiative_speed": item["initiative_speed"],
                "next_ready_at": item["next_ready_at"],
                "reaction_bias": item["reaction_bias"],
                "interrupt_threshold": item["interrupt_threshold"],
                "current_load": item["current_load"],
                "status": item["status"],
                "constraints": item["constraints"],
            }
            for item in state["participants"]
        ],
        "action_catalog": [
            {
                "action_id": action_id,
                "actor_id": selected["agent_id"],
                "action_type": action_type,
                "action_scale": "short" if action_id.endswith(".ruminate") else "standard",
                "base_recovery": base_recovery,
                "initiative_cost": initiative_cost,
                "interruptibility": interruptibility,
                "commitment": 0.25 if action_id.endswith(".ruminate") else 0.65,
                "local_affordance_basis": [
                    action_reason,
                    "Heartbeat slots control opportunity, not project authority.",
                ],
            }
        ],
        "reaction_windows": [
            {
                "window_id": f"{schedule_id}.pending-work",
                "trigger_event_ref": source_scene_ref,
                "urgency": urgency,
                "eligible_actor_ids": [agent_id_for_role(work_role)] if work_role else [],
                "allowed_action_scales": ["short", "standard"],
                "expires_at": round(state["scene_clock"] + 1.0, 6),
                "notes": "Pending coordinator work can pull its owning lane forward only if readiness clears threshold.",
            }
        ]
        if work_role
        else [],
        "selection_policy": state["selection_policy"],
        "next_actor_selection": {
            "selection_kind": selection_kind,
            "selected_actor_id": selected["agent_id"],
            "selected_action_ids": [action_id],
            "scene_clock_after_selection": state["scene_clock"],
            "selection_reason": selection_reason,
            "override_reason": None,
            "readiness_snapshot": snapshot,
        },
        "review_notes": [
            "Epiphany heartbeat uses Ghostlight initiative timing as a harness scheduling receipt.",
            "A selected idle lane may ruminate and request bounded self-memory mutation; it may not invent project work.",
        ],
    }

    rumination = None
    if action_id.endswith(".ruminate"):
        patch = rumination_patch(selected["role_id"], action_id)
        rumination = {
            "roleId": selected["role_id"],
            "selfPatch": patch,
            "result": apply_self_patch(selected["role_id"], patch, agent_dir=agent_dir)
            if apply_rumination
            else None,
            "applied": apply_rumination,
        }

    event = {
        "ts": now_iso(),
        "scheduleId": schedule_id,
        "selectedRole": selected["role_id"],
        "selectedAgentId": selected["agent_id"],
        "actionId": action_id,
        "actionType": "ruminate_memory" if action_id.endswith(".ruminate") else "role_work",
        "coordinatorAction": coordinator_action,
        "targetRole": target_role,
        "workRole": work_role,
        "sceneClock": state["scene_clock"],
        "nextReadyAt": selected["next_ready_at"],
    }
    state.setdefault("history", []).append(event)
    state["history"] = state["history"][-128:]
    return {"event": event, "schedule": schedule, "rumination": rumination}


def run_tick(args: argparse.Namespace) -> dict[str, Any]:
    state = load_state(args.state_file, target_heartbeat_rate=args.target_heartbeat_rate)
    errors = validate_all(args.agent_dir)
    if errors:
        raise ValueError("agent memory validation failed: " + "; ".join(errors))
    result = tick_once(
        state,
        coordinator_action=args.coordinator_action,
        target_role=args.target_role,
        urgency=args.urgency,
        apply_rumination=args.apply_rumination,
        agent_dir=args.agent_dir,
        schedule_id=args.schedule_id,
        source_scene_ref=args.source_scene_ref,
    )
    write_json(args.state_file, state)
    args.artifact_dir.mkdir(parents=True, exist_ok=True)
    write_json(args.artifact_dir / f"{args.schedule_id}.initiative.json", result["schedule"])
    write_json(args.artifact_dir / f"{args.schedule_id}.event.json", result["event"])
    if result["rumination"] is not None:
        write_json(args.artifact_dir / f"{args.schedule_id}.rumination.json", result["rumination"])
    return {
        "ok": True,
        "stateFile": str(args.state_file),
        "artifactDir": str(args.artifact_dir),
        **result,
    }


def run_smoke(args: argparse.Namespace) -> dict[str, Any]:
    with TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        agent_dir = tmp_dir / "agents"
        state_file = tmp_dir / "heartbeats.json"
        artifact_dir = tmp_dir / "artifacts"
        import shutil

        shutil.copytree(args.agent_dir, agent_dir)
        first_args = argparse.Namespace(
            state_file=state_file,
            artifact_dir=artifact_dir,
            agent_dir=agent_dir,
            target_heartbeat_rate=1.0,
            coordinator_action="continueImplementation",
            target_role=None,
            urgency=0.95,
            apply_rumination=True,
            schedule_id="smoke-work",
            source_scene_ref="smoke/coordinator",
        )
        work = run_tick(first_args)
        second_args = argparse.Namespace(
            state_file=state_file,
            artifact_dir=artifact_dir,
            agent_dir=agent_dir,
            target_heartbeat_rate=1.0,
            coordinator_action=None,
            target_role=None,
            urgency=0.0,
            apply_rumination=True,
            schedule_id="smoke-idle",
            source_scene_ref="smoke/idle",
        )
        idle = run_tick(second_args)
        validation_errors = validate_all(agent_dir)
        initiative_errors = [
            *validate_initiative_schedule_shape(work["schedule"]),
            *validate_initiative_schedule_shape(idle["schedule"]),
        ]
        state = load_json(state_file)
        ok = (
            work["event"]["selectedRole"] == "implementation"
            and idle["event"]["actionType"] == "ruminate_memory"
            and idle["rumination"]["result"]["status"] == "accepted"
            and not validation_errors
            and not initiative_errors
            and (artifact_dir / "smoke-work.initiative.json").exists()
            and (artifact_dir / "smoke-idle.rumination.json").exists()
            and len(state.get("participants", [])) == len(ROLE_ORDER)
        )
        return {
            "ok": ok,
            "workEvent": work["event"],
            "idleEvent": idle["event"],
            "idleRumination": idle["rumination"],
            "validationErrors": validation_errors,
            "initiativeErrors": initiative_errors,
        }


def validate_initiative_schedule_shape(schedule: dict[str, Any]) -> list[str]:
    errors: list[str] = []
    required = [
        "schema_version",
        "schedule_id",
        "source_scene_ref",
        "scene_clock",
        "participants",
        "action_catalog",
        "reaction_windows",
        "selection_policy",
        "next_actor_selection",
        "review_notes",
    ]
    for key in required:
        if key not in schedule:
            errors.append(f"initiative schedule missing {key}")
    if schedule.get("schema_version") != INITIATIVE_SCHEMA_VERSION:
        errors.append("initiative schedule has wrong schema_version")
    if not isinstance(schedule.get("scene_clock"), (int, float)) or schedule.get("scene_clock", -1) < 0:
        errors.append("initiative schedule scene_clock must be non-negative number")
    for participant in schedule.get("participants", []):
        if participant.get("status") not in GHOSTLIGHT_PARTICIPANT_STATUSES:
            errors.append(f"participant {participant.get('agent_id')} has invalid status")
        for key in ("initiative_speed", "next_ready_at", "reaction_bias", "interrupt_threshold", "current_load"):
            if not isinstance(participant.get(key), (int, float)):
                errors.append(f"participant {participant.get('agent_id')} {key} must be numeric")
    for action in schedule.get("action_catalog", []):
        if action.get("action_type") not in GHOSTLIGHT_ACTION_TYPES:
            errors.append(f"action {action.get('action_id')} has invalid action_type")
        if action.get("action_scale") not in GHOSTLIGHT_ACTION_SCALES:
            errors.append(f"action {action.get('action_id')} has invalid action_scale")
    selection = schedule.get("next_actor_selection", {})
    if selection.get("selection_kind") not in {"scheduled_turn", "reaction_interrupt", "coordinator_override"}:
        errors.append("next_actor_selection has invalid selection_kind")
    for snapshot in selection.get("readiness_snapshot", []):
        extra_keys = set(snapshot) - {
            "agent_id",
            "next_ready_at",
            "reaction_readiness",
            "eligible_for_reaction",
        }
        if extra_keys:
            errors.append(f"readiness snapshot has Ghostlight-incompatible keys: {sorted(extra_keys)}")
    return errors


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run Epiphany Ghostlight-style agent heartbeat scheduling.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    tick = subparsers.add_parser("tick")
    tick.add_argument("--state-file", type=Path, default=DEFAULT_HEARTBEAT_STATE)
    tick.add_argument("--artifact-dir", type=Path, default=DEFAULT_ARTIFACT_DIR)
    tick.add_argument("--agent-dir", type=Path, default=DEFAULT_AGENT_DIR)
    tick.add_argument("--target-heartbeat-rate", type=float, default=1.0)
    tick.add_argument("--coordinator-action")
    tick.add_argument("--target-role", choices=sorted(ROLE_TARGETS))
    tick.add_argument("--urgency", type=float, default=0.75)
    tick.add_argument("--apply-rumination", action="store_true")
    tick.add_argument("--schedule-id", default="epiphany-heartbeat")
    tick.add_argument("--source-scene-ref", default="epiphany/coordinator")

    init = subparsers.add_parser("init")
    init.add_argument("--state-file", type=Path, default=DEFAULT_HEARTBEAT_STATE)
    init.add_argument("--target-heartbeat-rate", type=float, default=1.0)

    smoke = subparsers.add_parser("smoke")
    smoke.add_argument("--agent-dir", type=Path, default=DEFAULT_AGENT_DIR)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        if args.command == "init":
            state = default_state(target_heartbeat_rate=args.target_heartbeat_rate)
            write_json(args.state_file, state)
            print(json.dumps({"ok": True, "stateFile": str(args.state_file)}, indent=2))
            return 0
        if args.command == "tick":
            print(json.dumps(run_tick(args), indent=2))
            return 0
        if args.command == "smoke":
            result = run_smoke(args)
            print(json.dumps(result, indent=2))
            return 0 if result["ok"] else 1
    except Exception as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, indent=2), file=sys.stderr)
        return 1
    raise AssertionError(args.command)


if __name__ == "__main__":
    sys.exit(main())
