from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
from typing import Any

from epiphany_agent_memory import DEFAULT_AGENT_DIR
from epiphany_agent_memory import resolve_store_path
from epiphany_agent_memory import ROLE_TARGETS
from epiphany_agent_memory import apply_self_patch
from epiphany_agent_memory import validate_all


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_HEARTBEAT_STATE = ROOT / "state" / "agent-heartbeats.json"
DEFAULT_HEARTBEAT_STORE = ROOT / "state" / "agent-heartbeats.msgpack"
DEFAULT_ARTIFACT_DIR = ROOT / ".epiphany-heartbeats"
HEARTBEAT_STORE_BIN = "epiphany-heartbeat-store"
HEARTBEAT_STORE_EXE = Path(os.environ.get("CARGO_TARGET_DIR", r"C:\Users\Meta\.cargo-target-codex")) / "debug" / "epiphany-heartbeat-store.exe"
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
    "face",
    "imagination",
    "research",
    "modeling",
    "implementation",
    "verification",
    "reorientation",
]

DISPLAY_NAMES = {
    "coordinator": "Self",
    "face": "Face",
    "imagination": "Imagination",
    "research": "Eyes",
    "modeling": "Body",
    "implementation": "Hands",
    "verification": "Soul",
    "reorientation": "Life",
}

def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def native_heartbeat_command(*args: str) -> dict[str, Any]:
    if HEARTBEAT_STORE_EXE.exists():
        command = [str(HEARTBEAT_STORE_EXE), *args]
    else:
        command = [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            str(ROOT / "epiphany-core" / "Cargo.toml"),
            "--bin",
            HEARTBEAT_STORE_BIN,
            "--",
            *args,
        ]
    completed = subprocess.run(
        command,
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        message = completed.stderr.strip() or completed.stdout.strip() or f"{HEARTBEAT_STORE_BIN} failed"
        raise ValueError(message)
    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as exc:
        raise ValueError(f"{HEARTBEAT_STORE_BIN} returned non-JSON output: {completed.stdout}") from exc


def heartbeat_status(
    *,
    state_file: Path = DEFAULT_HEARTBEAT_STATE,
    store_file: Path = DEFAULT_HEARTBEAT_STORE,
    artifact_dir: Path = DEFAULT_ARTIFACT_DIR,
    target_heartbeat_rate: float = 0.0,
    artifact_limit: int = 8,
) -> dict[str, Any]:
    return native_heartbeat_command(
        "status",
        "--store",
        str(store_file),
        "--artifact-dir",
        str(artifact_dir),
        "--target-heartbeat-rate",
        str(target_heartbeat_rate),
        "--limit",
        str(artifact_limit),
    )


def run_tick(args: argparse.Namespace) -> dict[str, Any]:
    errors = validate_all(args.agent_dir)
    if errors:
        raise ValueError("agent memory validation failed: " + "; ".join(errors))
    store_file = getattr(args, "store_file", DEFAULT_HEARTBEAT_STORE)
    command = [
        "tick",
        "--store",
        str(store_file),
        "--artifact-dir",
        str(args.artifact_dir),
        "--target-heartbeat-rate",
        str(args.target_heartbeat_rate),
        "--urgency",
        str(args.urgency),
        "--schedule-id",
        args.schedule_id,
        "--source-scene-ref",
        args.source_scene_ref,
    ]
    if args.coordinator_action:
        command.extend(["--coordinator-action", args.coordinator_action])
    if args.target_role:
        command.extend(["--target-role", args.target_role])
    if getattr(args, "defer_completion", False):
        command.append("--defer-completion")
    result = native_heartbeat_command(*command)
    rumination = result.get("rumination")
    if isinstance(rumination, dict) and rumination.get("selfPatch") is not None:
        rumination["result"] = (
            apply_self_patch(rumination.get("roleId"), rumination["selfPatch"], agent_dir=args.agent_dir)
            if args.apply_rumination
            else None
        )
        rumination["applied"] = bool(args.apply_rumination)
        write_json(args.artifact_dir / f"{args.schedule_id}.rumination.json", rumination)
        result["rumination"] = rumination
    result.setdefault("stateFile", None)
    result["storeFile"] = str(store_file)
    result["artifactDir"] = str(args.artifact_dir)
    return result


def run_complete(args: argparse.Namespace) -> dict[str, Any]:
    store_file = getattr(args, "store_file", DEFAULT_HEARTBEAT_STORE)
    command = [
        "complete",
        "--store",
        str(store_file),
        "--artifact-dir",
        str(args.artifact_dir),
        "--role",
        args.role,
    ]
    if args.action_id:
        command.extend(["--action-id", args.action_id])
    return native_heartbeat_command(*command)


def run_status(args: argparse.Namespace) -> dict[str, Any]:
    return heartbeat_status(
        store_file=args.store_file,
        artifact_dir=args.artifact_dir,
        target_heartbeat_rate=args.target_heartbeat_rate,
        artifact_limit=args.limit,
    )


def run_smoke(args: argparse.Namespace) -> dict[str, Any]:
    with TemporaryDirectory() as tmp:
        tmp_dir = Path(tmp)
        agent_store = tmp_dir / "agents.msgpack"
        store_file = tmp_dir / "heartbeats.msgpack"
        artifact_dir = tmp_dir / "artifacts"
        import shutil

        shutil.copy2(resolve_store_path(args.agent_dir), agent_store)
        native_heartbeat_command("init", "--store", str(store_file), "--target-heartbeat-rate", "1.0")
        first_args = argparse.Namespace(
            state_file=None,
            store_file=store_file,
            artifact_dir=artifact_dir,
            agent_dir=agent_store,
            target_heartbeat_rate=1.0,
            coordinator_action="continueImplementation",
            target_role=None,
            urgency=0.95,
            apply_rumination=True,
            schedule_id="smoke-work",
            source_scene_ref="smoke/coordinator",
            defer_completion=True,
        )
        work = run_tick(first_args)
        blocked_repeat = None
        try:
            run_tick(first_args)
        except ValueError as exc:
            blocked_repeat = str(exc)
        complete_args = argparse.Namespace(
            state_file=None,
            store_file=store_file,
            artifact_dir=artifact_dir,
            role="implementation",
            action_id=work["event"]["actionId"],
        )
        completed = run_complete(complete_args)
        second_args = argparse.Namespace(
            state_file=None,
            store_file=store_file,
            artifact_dir=artifact_dir,
            agent_dir=agent_store,
            target_heartbeat_rate=1.0,
            coordinator_action=None,
            target_role=None,
            urgency=0.0,
            apply_rumination=True,
            schedule_id="smoke-idle",
            source_scene_ref="smoke/idle",
            defer_completion=False,
        )
        idle = run_tick(second_args)
        validation_errors = validate_all(agent_store)
        initiative_errors = [
            *validate_initiative_schedule_shape(work["schedule"]),
            *validate_initiative_schedule_shape(idle["schedule"]),
        ]
        status = heartbeat_status(store_file=store_file, artifact_dir=artifact_dir)
        ok = (
            work["event"]["selectedRole"] == "implementation"
            and work["event"]["turnStatus"] == "running"
            and blocked_repeat is not None
            and "already has running heartbeat turn" in blocked_repeat
            and completed["event"]["turnStatus"] == "completed"
            and idle["event"]["actionType"] == "ruminate_memory"
            and idle["event"]["turnStatus"] == "completed"
            and idle["event"]["nextReadyAt"] > completed["event"]["nextReadyAt"]
            and idle["rumination"]["result"]["status"] == "accepted"
            and not validation_errors
            and not initiative_errors
            and (artifact_dir / "smoke-work.initiative.json").exists()
            and (artifact_dir / "smoke-work.completion.json").exists()
            and (artifact_dir / "smoke-idle.rumination.json").exists()
            and len(status.get("participants", [])) == len(ROLE_ORDER)
        )
        return {
            "ok": ok,
            "workEvent": work["event"],
            "blockedRepeat": blocked_repeat,
            "completionEvent": completed["event"],
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
    tick.add_argument("--store-file", type=Path, default=DEFAULT_HEARTBEAT_STORE)
    tick.add_argument("--artifact-dir", type=Path, default=DEFAULT_ARTIFACT_DIR)
    tick.add_argument("--agent-dir", type=Path, default=DEFAULT_AGENT_DIR)
    tick.add_argument("--target-heartbeat-rate", type=float, default=1.0)
    tick.add_argument("--coordinator-action")
    tick.add_argument("--target-role", choices=sorted(ROLE_TARGETS))
    tick.add_argument("--urgency", type=float, default=0.75)
    tick.add_argument("--apply-rumination", action="store_true")
    tick.add_argument("--schedule-id", default="epiphany-heartbeat")
    tick.add_argument("--source-scene-ref", default="epiphany/coordinator")
    tick.add_argument(
        "--defer-completion",
        action="store_true",
        help="Leave the selected lane in-flight; cooldown begins when the complete command is called.",
    )

    complete = subparsers.add_parser("complete")
    complete.add_argument("--state-file", type=Path, default=DEFAULT_HEARTBEAT_STATE)
    complete.add_argument("--store-file", type=Path, default=DEFAULT_HEARTBEAT_STORE)
    complete.add_argument("--artifact-dir", type=Path, default=DEFAULT_ARTIFACT_DIR)
    complete.add_argument("--role", choices=sorted(ROLE_TARGETS), required=True)
    complete.add_argument("--action-id")

    init = subparsers.add_parser("init")
    init.add_argument("--state-file", type=Path, default=DEFAULT_HEARTBEAT_STATE)
    init.add_argument("--store-file", type=Path, default=DEFAULT_HEARTBEAT_STORE)
    init.add_argument("--target-heartbeat-rate", type=float, default=1.0)

    status = subparsers.add_parser("status")
    status.add_argument("--state-file", type=Path, default=DEFAULT_HEARTBEAT_STATE)
    status.add_argument("--store-file", type=Path, default=DEFAULT_HEARTBEAT_STORE)
    status.add_argument("--artifact-dir", type=Path, default=DEFAULT_ARTIFACT_DIR)
    status.add_argument("--target-heartbeat-rate", type=float, default=0.0)
    status.add_argument("--limit", type=int, default=8)

    smoke = subparsers.add_parser("smoke")
    smoke.add_argument("--agent-dir", type=Path, default=DEFAULT_AGENT_DIR)

    return parser


def main(argv: list[str] | None = None) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    try:
        if args.command == "init":
            result = native_heartbeat_command(
                "init",
                "--store",
                str(args.store_file),
                "--target-heartbeat-rate",
                str(args.target_heartbeat_rate),
            )
            print(json.dumps(result, indent=2))
            return 0
        if args.command == "tick":
            print(json.dumps(run_tick(args), indent=2))
            return 0
        if args.command == "complete":
            print(json.dumps(run_complete(args), indent=2))
            return 0
        if args.command == "status":
            print(json.dumps(run_status(args), indent=2))
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
