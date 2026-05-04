from __future__ import annotations

import argparse
import json
from pathlib import Path
import shutil
import sys
from tempfile import TemporaryDirectory
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_AGENT_DIR = ROOT / "state" / "agents"
SCHEMA_VERSION = "ghostlight.agent_state.v0"


ROLE_TARGETS = {
    "imagination": ("epiphany.imagination", "imagination.agent-state.json"),
    "modeling": ("epiphany.body", "body.agent-state.json"),
    "verification": ("epiphany.soul", "soul.agent-state.json"),
    "implementation": ("epiphany.hands", "hands.agent-state.json"),
    "reorientation": ("epiphany.life", "life.agent-state.json"),
    "research": ("epiphany.eyes", "eyes.agent-state.json"),
    "coordinator": ("epiphany.self", "self.agent-state.json"),
}

ALLOWED_PATCH_FIELDS = {
    "agentId",
    "reason",
    "evidenceIds",
    "semanticMemories",
    "episodicMemories",
    "relationshipMemories",
    "goals",
    "values",
    "privateNotes",
}

FORBIDDEN_PATCH_FIELDS = {
    "statePatch",
    "objective",
    "activeSubgoalId",
    "subgoals",
    "invariants",
    "graphs",
    "graphFrontier",
    "graphCheckpoint",
    "scratch",
    "investigationCheckpoint",
    "jobBindings",
    "planning",
    "churn",
    "mode",
    "codeEdits",
    "files",
    "authorityScope",
    "backendJobId",
    "rawResult",
}


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, sort_keys=False) + "\n", encoding="utf-8")


def role_target(role_id: str) -> tuple[str, Path]:
    try:
        agent_id, filename = ROLE_TARGETS[role_id]
    except KeyError as exc:
        raise ValueError(f"unknown role id: {role_id}") from exc
    return agent_id, DEFAULT_AGENT_DIR / filename


def require_object(value: Any, path: str, errors: list[str]) -> dict[str, Any]:
    if isinstance(value, dict):
        return value
    errors.append(f"{path} must be an object")
    return {}


def require_array(value: Any, path: str, errors: list[str]) -> list[Any]:
    if isinstance(value, list):
        return value
    errors.append(f"{path} must be an array")
    return []


def check_unit(value: Any, path: str, errors: list[str]) -> None:
    if not isinstance(value, (int, float)) or isinstance(value, bool) or not 0 <= float(value) <= 1:
        errors.append(f"{path} must be a number between 0 and 1")


def check_string(value: Any, path: str, errors: list[str], *, max_len: int = 800) -> None:
    if not isinstance(value, str) or not value.strip() or len(value) > max_len:
        errors.append(f"{path} must be non-empty text under {max_len} characters")


def validate_memory(record: Any, path: str, errors: list[str]) -> None:
    item = require_object(record, path, errors)
    if not item:
        return
    check_string(item.get("memory_id"), f"{path}.memory_id", errors, max_len=120)
    check_string(item.get("summary"), f"{path}.summary", errors, max_len=800)
    check_unit(item.get("salience"), f"{path}.salience", errors)
    check_unit(item.get("confidence"), f"{path}.confidence", errors)


def validate_agent_file(path: Path) -> list[str]:
    errors: list[str] = []
    try:
        document = load_json(path)
    except Exception as exc:
        return [f"{path}: failed to parse JSON: {exc}"]

    root = require_object(document, str(path), errors)
    if root.get("schema_version") != SCHEMA_VERSION:
        errors.append(f"{path}: schema_version must be {SCHEMA_VERSION!r}")
    world = require_object(root.get("world"), f"{path}.world", errors)
    for key in ("world_id", "setting", "canon_context"):
        if key == "canon_context":
            require_array(world.get(key), f"{path}.world.{key}", errors)
        else:
            check_string(world.get(key), f"{path}.world.{key}", errors)
    time = require_object(world.get("time"), f"{path}.world.time", errors)
    check_string(time.get("label"), f"{path}.world.time.label", errors, max_len=200)

    agents = require_array(root.get("agents"), f"{path}.agents", errors)
    if len(agents) != 1:
        errors.append(f"{path}.agents must contain exactly one agent")
    for index, raw_agent in enumerate(agents):
        agent = require_object(raw_agent, f"{path}.agents[{index}]", errors)
        check_string(agent.get("agent_id"), f"{path}.agents[{index}].agent_id", errors, max_len=120)
        identity = require_object(agent.get("identity"), f"{path}.agents[{index}].identity", errors)
        for key in ("name", "origin", "public_description"):
            check_string(identity.get(key), f"{path}.agents[{index}].identity.{key}", errors)
        require_array(identity.get("roles"), f"{path}.agents[{index}].identity.roles", errors)

        canonical = require_object(
            agent.get("canonical_state"), f"{path}.agents[{index}].canonical_state", errors
        )
        for key in (
            "underlying_organization",
            "stable_dispositions",
            "behavioral_dimensions",
            "presentation_strategy",
            "voice_style",
            "situational_state",
        ):
            require_object(canonical.get(key), f"{path}.agents[{index}].canonical_state.{key}", errors)
        values = require_array(
            canonical.get("values"), f"{path}.agents[{index}].canonical_state.values", errors
        )
        for value_index, raw_value in enumerate(values):
            value = require_object(
                raw_value,
                f"{path}.agents[{index}].canonical_state.values[{value_index}]",
                errors,
            )
            check_string(value.get("value_id"), f"{path}.values[{value_index}].value_id", errors)
            check_string(value.get("label"), f"{path}.values[{value_index}].label", errors)
            check_unit(value.get("priority"), f"{path}.values[{value_index}].priority", errors)
            if not isinstance(value.get("unforgivable_if_betrayed"), bool):
                errors.append(f"{path}.values[{value_index}].unforgivable_if_betrayed must be boolean")

        goals = require_array(agent.get("goals"), f"{path}.agents[{index}].goals", errors)
        for goal_index, raw_goal in enumerate(goals):
            goal = require_object(raw_goal, f"{path}.agents[{index}].goals[{goal_index}]", errors)
            check_string(goal.get("goal_id"), f"{path}.goals[{goal_index}].goal_id", errors)
            check_string(goal.get("description"), f"{path}.goals[{goal_index}].description", errors)
            if goal.get("scope") not in {"immediate", "scene", "case", "arc", "life"}:
                errors.append(f"{path}.goals[{goal_index}].scope is not a Ghostlight scope")
            check_unit(goal.get("priority"), f"{path}.goals[{goal_index}].priority", errors)
            check_string(goal.get("emotional_stake"), f"{path}.goals[{goal_index}].emotional_stake", errors)
            if goal.get("status") not in {"active", "blocked", "dormant", "resolved", "abandoned"}:
                errors.append(f"{path}.goals[{goal_index}].status is not a Ghostlight status")

        memories = require_object(agent.get("memories"), f"{path}.agents[{index}].memories", errors)
        for bundle_name in ("episodic", "semantic", "relationship_summaries"):
            for memory_index, raw_memory in enumerate(
                require_array(memories.get(bundle_name), f"{path}.memories.{bundle_name}", errors)
            ):
                validate_memory(raw_memory, f"{path}.memories.{bundle_name}[{memory_index}]", errors)

        require_array(
            agent.get("perceived_state_overlays"),
            f"{path}.agents[{index}].perceived_state_overlays",
            errors,
        )

    for key in ("relationships", "events", "scenes"):
        require_array(root.get(key), f"{path}.{key}", errors)
    return errors


def valid_identifier(value: Any, prefix: str) -> bool:
    return (
        isinstance(value, str)
        and value.startswith(prefix)
        and len(value) <= 120
        and all(ch.isascii() and (ch.isalnum() or ch in "-_.") for ch in value)
    )


def review_patch(role_id: str, patch: Any) -> dict[str, Any]:
    agent_id, target_path = role_target(role_id)
    reasons: list[str] = []
    if not isinstance(patch, dict):
        reasons.append("selfPatch must be a JSON object")
    else:
        actual_agent = patch.get("agentId")
        if actual_agent != agent_id:
            reasons.append(f"selfPatch agentId {actual_agent!r} does not match this lane; expected {agent_id!r}")
        reason = patch.get("reason")
        if not isinstance(reason, str) or len(reason.strip()) < 16 or len(reason) > 800:
            reasons.append("selfPatch reason must be a bounded explanation of at least 16 characters")
        for key in patch:
            if key in FORBIDDEN_PATCH_FIELDS:
                reasons.append(
                    f"selfPatch field {key!r} is project truth or authority; use the proper Epiphany control surface instead"
                )
            elif key not in ALLOWED_PATCH_FIELDS:
                reasons.append(f"selfPatch field {key!r} is not part of the bounded memory mutation contract")

        mutation_count = 0
        for field in ("semanticMemories", "episodicMemories", "relationshipMemories"):
            mutation_count += review_memory_patch_array(patch, field, reasons)
        mutation_count += review_goal_patch_array(patch, reasons)
        mutation_count += review_value_patch_array(patch, reasons)
        mutation_count += review_private_notes(patch, reasons)
        review_string_array(patch, "evidenceIds", reasons, max_items=16, max_len=160)
        if mutation_count == 0:
            reasons.append(
                "selfPatch must contain at least one semantic memory, episodic memory, relationship memory, goal, value, or private note"
            )

    return {
        "status": "rejected" if reasons else "accepted",
        "targetAgentId": agent_id,
        "targetPath": str(target_path),
        "reasons": reasons,
    }


def review_memory_patch_array(patch: dict[str, Any], field: str, reasons: list[str]) -> int:
    if field not in patch:
        return 0
    value = patch[field]
    if not isinstance(value, list):
        reasons.append(f"selfPatch {field} must be an array")
        return 0
    if len(value) > 8:
        reasons.append(f"selfPatch {field} may contain at most 8 records")
    for index, item in enumerate(value):
        if not isinstance(item, dict):
            reasons.append(f"selfPatch {field}[{index}] must be an object")
            continue
        if not valid_identifier(item.get("memoryId"), "mem-"):
            reasons.append(f"selfPatch {field}[{index}].memoryId must start with 'mem-' and avoid whitespace")
        check_patch_text(item.get("summary"), f"selfPatch {field}[{index}].summary", reasons, 600)
        check_patch_unit(item.get("salience"), f"selfPatch {field}[{index}].salience", reasons)
        check_patch_unit(item.get("confidence"), f"selfPatch {field}[{index}].confidence", reasons)
    return len(value)


def review_goal_patch_array(patch: dict[str, Any], reasons: list[str]) -> int:
    if "goals" not in patch:
        return 0
    value = patch["goals"]
    if not isinstance(value, list):
        reasons.append("selfPatch goals must be an array")
        return 0
    if len(value) > 6:
        reasons.append("selfPatch goals may contain at most 6 records")
    for index, item in enumerate(value):
        if not isinstance(item, dict):
            reasons.append(f"selfPatch goals[{index}] must be an object")
            continue
        if not valid_identifier(item.get("goalId"), "goal-"):
            reasons.append(f"selfPatch goals[{index}].goalId must start with 'goal-' and avoid whitespace")
        check_patch_text(item.get("description"), f"selfPatch goals[{index}].description", reasons, 700)
        if item.get("scope") not in {"immediate", "scene", "case", "arc", "life"}:
            reasons.append(f"selfPatch goals[{index}].scope is not a Ghostlight scope")
        check_patch_unit(item.get("priority"), f"selfPatch goals[{index}].priority", reasons)
        check_patch_text(item.get("emotionalStake"), f"selfPatch goals[{index}].emotionalStake", reasons, 400)
        if item.get("status") not in {"active", "blocked", "dormant", "resolved", "abandoned"}:
            reasons.append(f"selfPatch goals[{index}].status is not a Ghostlight status")
    return len(value)


def review_value_patch_array(patch: dict[str, Any], reasons: list[str]) -> int:
    if "values" not in patch:
        return 0
    value = patch["values"]
    if not isinstance(value, list):
        reasons.append("selfPatch values must be an array")
        return 0
    if len(value) > 6:
        reasons.append("selfPatch values may contain at most 6 records")
    for index, item in enumerate(value):
        if not isinstance(item, dict):
            reasons.append(f"selfPatch values[{index}] must be an object")
            continue
        if not valid_identifier(item.get("valueId"), "value-"):
            reasons.append(f"selfPatch values[{index}].valueId must start with 'value-' and avoid whitespace")
        check_patch_text(item.get("label"), f"selfPatch values[{index}].label", reasons, 240)
        check_patch_unit(item.get("priority"), f"selfPatch values[{index}].priority", reasons)
        if not isinstance(item.get("unforgivableIfBetrayed"), bool):
            reasons.append(f"selfPatch values[{index}].unforgivableIfBetrayed must be boolean")
    return len(value)


def review_private_notes(patch: dict[str, Any], reasons: list[str]) -> int:
    if "privateNotes" not in patch:
        return 0
    value = patch["privateNotes"]
    if not isinstance(value, list):
        reasons.append("selfPatch privateNotes must be an array")
        return 0
    if len(value) > 6:
        reasons.append("selfPatch privateNotes may contain at most 6 records")
    for index, item in enumerate(value):
        check_patch_text(item, f"selfPatch privateNotes[{index}]", reasons, 600)
    return len(value)


def review_string_array(
    patch: dict[str, Any],
    field: str,
    reasons: list[str],
    *,
    max_items: int,
    max_len: int,
) -> None:
    if field not in patch:
        return
    value = patch[field]
    if not isinstance(value, list):
        reasons.append(f"selfPatch {field} must be an array")
        return
    if len(value) > max_items:
        reasons.append(f"selfPatch {field} may contain at most {max_items} records")
    for index, item in enumerate(value):
        check_patch_text(item, f"selfPatch {field}[{index}]", reasons, max_len)


def check_patch_text(value: Any, path: str, reasons: list[str], max_len: int) -> None:
    if not isinstance(value, str) or not value.strip() or len(value) > max_len:
        reasons.append(f"{path} must be non-empty text under {max_len} characters")


def check_patch_unit(value: Any, path: str, reasons: list[str]) -> None:
    if not isinstance(value, (int, float)) or isinstance(value, bool) or not 0 <= float(value) <= 1:
        reasons.append(f"{path} must be between 0 and 1")


def normalized_memory(item: dict[str, Any]) -> dict[str, Any]:
    record = {
        "memory_id": item["memoryId"],
        "summary": item["summary"],
        "salience": item["salience"],
        "confidence": item["confidence"],
    }
    if "linkedEventIds" in item:
        record["linked_event_ids"] = item["linkedEventIds"]
    if "linkedRelationshipId" in item:
        record["linked_relationship_id"] = item["linkedRelationshipId"]
    return record


def normalized_goal(item: dict[str, Any]) -> dict[str, Any]:
    return {
        "goal_id": item["goalId"],
        "description": item["description"],
        "scope": item["scope"],
        "priority": item["priority"],
        "emotional_stake": item["emotionalStake"],
        "blockers": item.get("blockers", []),
        "status": item["status"],
    }


def normalized_value(item: dict[str, Any]) -> dict[str, Any]:
    return {
        "value_id": item["valueId"],
        "label": item["label"],
        "priority": item["priority"],
        "unforgivable_if_betrayed": item["unforgivableIfBetrayed"],
    }


def upsert_by_id(records: list[dict[str, Any]], incoming: list[dict[str, Any]], id_field: str) -> list[dict[str, Any]]:
    index = {record.get(id_field): record for record in records if isinstance(record, dict)}
    for item in incoming:
        index[item[id_field]] = item
    return list(index.values())


def apply_self_patch(role_id: str, patch: dict[str, Any], *, agent_dir: Path = DEFAULT_AGENT_DIR) -> dict[str, Any]:
    review = review_patch(role_id, patch)
    if review["status"] != "accepted":
        return review
    _, filename = ROLE_TARGETS[role_id]
    path = agent_dir / filename
    document = load_json(path)
    agent = document["agents"][0]
    memories = agent["memories"]
    memories["semantic"] = upsert_by_id(
        memories["semantic"],
        [normalized_memory(item) for item in patch.get("semanticMemories", [])],
        "memory_id",
    )
    memories["episodic"] = upsert_by_id(
        memories["episodic"],
        [normalized_memory(item) for item in patch.get("episodicMemories", [])],
        "memory_id",
    )
    memories["relationship_summaries"] = upsert_by_id(
        memories["relationship_summaries"],
        [normalized_memory(item) for item in patch.get("relationshipMemories", [])],
        "memory_id",
    )
    agent["goals"] = upsert_by_id(
        agent["goals"],
        [normalized_goal(item) for item in patch.get("goals", [])],
        "goal_id",
    )
    canonical = agent["canonical_state"]
    canonical["values"] = upsert_by_id(
        canonical["values"],
        [normalized_value(item) for item in patch.get("values", [])],
        "value_id",
    )
    private_notes = agent["identity"].setdefault("private_notes", [])
    private_notes.extend(patch.get("privateNotes", []))
    agent["identity"]["private_notes"] = private_notes[-32:]
    write_json(path, document)
    review["applied"] = True
    return review


def validate_all(agent_dir: Path) -> list[str]:
    errors: list[str] = []
    for role_id, (_, filename) in ROLE_TARGETS.items():
        path = agent_dir / filename
        if not path.exists():
            errors.append(f"{role_id}: missing {path}")
            continue
        errors.extend(validate_agent_file(path))
    return errors


def parse_patch_arg(value: str) -> Any:
    path = Path(value)
    if path.exists():
        return load_json(path)
    return json.loads(value)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Review and apply Epiphany specialist self-memory patches.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    validate_parser = subparsers.add_parser("validate")
    validate_parser.add_argument("--agent-dir", type=Path, default=DEFAULT_AGENT_DIR)

    review_parser = subparsers.add_parser("review-patch")
    review_parser.add_argument("--role-id", required=True, choices=sorted(ROLE_TARGETS))
    review_parser.add_argument("--patch", required=True, help="JSON string or path to a JSON patch")

    apply_parser = subparsers.add_parser("apply-patch")
    apply_parser.add_argument("--role-id", required=True, choices=sorted(ROLE_TARGETS))
    apply_parser.add_argument("--patch", required=True, help="JSON string or path to a JSON patch")
    apply_parser.add_argument("--agent-dir", type=Path, default=DEFAULT_AGENT_DIR)

    smoke_parser = subparsers.add_parser("smoke")
    smoke_parser.add_argument("--agent-dir", type=Path, default=DEFAULT_AGENT_DIR)

    args = parser.parse_args(argv)
    if args.command == "validate":
        errors = validate_all(args.agent_dir)
        print(json.dumps({"ok": not errors, "errors": errors}, indent=2))
        return 0 if not errors else 1
    if args.command == "review-patch":
        print(json.dumps(review_patch(args.role_id, parse_patch_arg(args.patch)), indent=2))
        return 0
    if args.command == "apply-patch":
        patch = parse_patch_arg(args.patch)
        result = apply_self_patch(args.role_id, patch, agent_dir=args.agent_dir)
        print(json.dumps(result, indent=2))
        return 0 if result["status"] == "accepted" else 1
    if args.command == "smoke":
        errors = validate_all(args.agent_dir)
        if errors:
            print(json.dumps({"ok": False, "phase": "validate", "errors": errors}, indent=2))
            return 1
        accepted_patch = {
            "agentId": "epiphany.body",
            "reason": "The Body should remember accepted graph growth must be source-grounded and bounded.",
            "semanticMemories": [
                {
                    "memoryId": "mem-body-smoke-source-grounding",
                    "summary": "A modeling self-memory request is acceptable when it improves future graph/checkpoint judgment without smuggling project truth.",
                    "salience": 0.74,
                    "confidence": 0.86,
                }
            ],
        }
        accepted = review_patch("modeling", accepted_patch)
        wrong_role = review_patch("verification", accepted_patch)
        forbidden = review_patch(
            "modeling",
            {
                "agentId": "epiphany.body",
                "reason": "This tries to put project state in lane memory, which should be refused.",
                "graphs": {},
                "semanticMemories": [
                    {
                        "memoryId": "mem-body-bad-project-truth",
                        "summary": "Bad patch.",
                        "salience": 0.5,
                        "confidence": 0.5,
                    }
                ],
            },
        )
        with TemporaryDirectory() as tmp:
            tmp_dir = Path(tmp)
            shutil.copytree(args.agent_dir, tmp_dir / "agents")
            applied = apply_self_patch("modeling", accepted_patch, agent_dir=tmp_dir / "agents")
            temp_errors = validate_all(tmp_dir / "agents")
        ok = (
            accepted["status"] == "accepted"
            and wrong_role["status"] == "rejected"
            and forbidden["status"] == "rejected"
            and applied["status"] == "accepted"
            and not temp_errors
        )
        print(
            json.dumps(
                {
                    "ok": ok,
                    "accepted": accepted,
                    "wrongRole": wrong_role,
                    "forbidden": forbidden,
                    "applied": applied,
                    "tempValidationErrors": temp_errors,
                },
                indent=2,
            )
        )
        return 0 if ok else 1
    raise AssertionError(args.command)


if __name__ == "__main__":
    sys.exit(main())
