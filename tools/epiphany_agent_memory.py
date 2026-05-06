from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
from tempfile import TemporaryDirectory
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_AGENT_STORE = ROOT / "state" / "agents.msgpack"
LEGACY_AGENT_DIR = ROOT / "state" / "agents"
DEFAULT_AGENT_DIR = DEFAULT_AGENT_STORE
AGENT_MEMORY_STORE_BIN = "epiphany-agent-memory-store"
AGENT_MEMORY_STORE_EXE = (
    Path(os.environ.get("CARGO_TARGET_DIR", r"C:\Users\Meta\.cargo-target-codex"))
    / "debug"
    / "epiphany-agent-memory-store.exe"
)

ROLE_TARGETS = {
    "imagination": ("epiphany.imagination", "imagination.agent-state.json"),
    "modeling": ("epiphany.body", "body.agent-state.json"),
    "verification": ("epiphany.soul", "soul.agent-state.json"),
    "implementation": ("epiphany.hands", "hands.agent-state.json"),
    "reorientation": ("epiphany.life", "life.agent-state.json"),
    "research": ("epiphany.eyes", "eyes.agent-state.json"),
    "face": ("epiphany.face", "face.agent-state.json"),
    "coordinator": ("epiphany.self", "self.agent-state.json"),
}


def resolve_store_path(path: Path | None = None) -> Path:
    if path is None:
        return DEFAULT_AGENT_STORE
    if path.suffix == ".msgpack":
        return path
    candidate = path / "agents.msgpack"
    if candidate.exists():
        return candidate
    if path == DEFAULT_AGENT_DIR and DEFAULT_AGENT_STORE.exists():
        return DEFAULT_AGENT_STORE
    return candidate


def native_agent_memory_command(*args: str) -> dict[str, Any]:
    if AGENT_MEMORY_STORE_EXE.exists():
        command = [str(AGENT_MEMORY_STORE_EXE), *args]
    else:
        command = [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            str(ROOT / "epiphany-core" / "Cargo.toml"),
            "--bin",
            AGENT_MEMORY_STORE_BIN,
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
        message = completed.stderr.strip() or completed.stdout.strip() or f"{AGENT_MEMORY_STORE_BIN} failed"
        raise ValueError(message)
    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as exc:
        raise ValueError(f"{AGENT_MEMORY_STORE_BIN} returned non-JSON output: {completed.stdout}") from exc


def load_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def parse_patch_arg(value: str) -> Any:
    path = Path(value)
    if path.exists():
        return load_json(path)
    return json.loads(value)


def role_target(role_id: str) -> tuple[str, Path]:
    try:
        agent_id, filename = ROLE_TARGETS[role_id]
    except KeyError as exc:
        raise ValueError(f"unknown role id: {role_id}") from exc
    return agent_id, DEFAULT_AGENT_STORE


def migrate_json_dir(agent_dir: Path = LEGACY_AGENT_DIR, store: Path = DEFAULT_AGENT_STORE) -> dict[str, Any]:
    return native_agent_memory_command(
        "migrate-json-dir",
        "--agent-dir",
        str(agent_dir),
        "--store",
        str(store),
    )


def validate_all(agent_dir: Path = DEFAULT_AGENT_DIR) -> list[str]:
    store = resolve_store_path(agent_dir)
    result = native_agent_memory_command("validate", "--store", str(store))
    return result.get("errors", [])


def review_patch(role_id: str, patch: Any, *, store: Path = DEFAULT_AGENT_STORE) -> dict[str, Any]:
    return native_agent_memory_command(
        "review-patch",
        "--store",
        str(resolve_store_path(store)),
        "--role-id",
        role_id,
        "--patch",
        json.dumps(patch),
    )


def apply_self_patch(
    role_id: str,
    patch: dict[str, Any],
    *,
    agent_dir: Path = DEFAULT_AGENT_DIR,
) -> dict[str, Any]:
    return native_agent_memory_command(
        "apply-patch",
        "--store",
        str(resolve_store_path(agent_dir)),
        "--role-id",
        role_id,
        "--patch",
        json.dumps(patch),
    )


def status(store: Path = DEFAULT_AGENT_STORE) -> dict[str, Any]:
    return native_agent_memory_command("status", "--store", str(resolve_store_path(store)))


def project_json_dir(store: Path = DEFAULT_AGENT_STORE, output_dir: Path = DEFAULT_AGENT_DIR) -> dict[str, Any]:
    return native_agent_memory_command(
        "project-json-dir",
        "--store",
        str(resolve_store_path(store)),
        "--output-dir",
        str(output_dir),
    )


def run_smoke(store: Path = DEFAULT_AGENT_STORE) -> dict[str, Any]:
    errors = validate_all(store)
    if errors:
        return {"ok": False, "phase": "validate", "errors": errors}
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
    accepted = review_patch("modeling", accepted_patch, store=store)
    wrong_role = review_patch("verification", accepted_patch, store=store)
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
        store=store,
    )
    with TemporaryDirectory() as tmp:
        tmp_store = Path(tmp) / "agents.msgpack"
        shutil.copy2(resolve_store_path(store), tmp_store)
        applied = apply_self_patch("modeling", accepted_patch, agent_dir=tmp_store)
        temp_errors = validate_all(tmp_store)
    ok = (
        accepted["status"] == "accepted"
        and wrong_role["status"] == "rejected"
        and forbidden["status"] == "rejected"
        and applied["status"] == "accepted"
        and not temp_errors
    )
    return {
        "ok": ok,
        "accepted": accepted,
        "wrongRole": wrong_role,
        "forbidden": forbidden,
        "applied": applied,
        "tempValidationErrors": temp_errors,
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Review and apply Epiphany specialist self-memory patches.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    migrate_parser = subparsers.add_parser("migrate-json-dir")
    migrate_parser.add_argument("--agent-dir", type=Path, default=LEGACY_AGENT_DIR)
    migrate_parser.add_argument("--store", type=Path, default=DEFAULT_AGENT_STORE)

    project_parser = subparsers.add_parser("project-json-dir")
    project_parser.add_argument("--store", type=Path, default=DEFAULT_AGENT_STORE)
    project_parser.add_argument("--output-dir", type=Path, default=DEFAULT_AGENT_DIR)

    status_parser = subparsers.add_parser("status")
    status_parser.add_argument("--store", type=Path, default=DEFAULT_AGENT_STORE)

    validate_parser = subparsers.add_parser("validate")
    validate_parser.add_argument("--agent-dir", type=Path, default=DEFAULT_AGENT_STORE)

    review_parser = subparsers.add_parser("review-patch")
    review_parser.add_argument("--role-id", required=True, choices=sorted(ROLE_TARGETS))
    review_parser.add_argument("--patch", required=True, help="JSON string or path to a JSON patch")
    review_parser.add_argument("--store", type=Path, default=DEFAULT_AGENT_STORE)

    apply_parser = subparsers.add_parser("apply-patch")
    apply_parser.add_argument("--role-id", required=True, choices=sorted(ROLE_TARGETS))
    apply_parser.add_argument("--patch", required=True, help="JSON string or path to a JSON patch")
    apply_parser.add_argument("--agent-dir", type=Path, default=DEFAULT_AGENT_STORE)

    smoke_parser = subparsers.add_parser("smoke")
    smoke_parser.add_argument("--store", type=Path, default=DEFAULT_AGENT_STORE)

    args = parser.parse_args(argv)
    try:
        if args.command == "migrate-json-dir":
            print(json.dumps(migrate_json_dir(args.agent_dir, args.store), indent=2))
            return 0
        if args.command == "project-json-dir":
            print(json.dumps(project_json_dir(args.store, args.output_dir), indent=2))
            return 0
        if args.command == "status":
            print(json.dumps(status(args.store), indent=2))
            return 0
        if args.command == "validate":
            errors = validate_all(args.agent_dir)
            print(json.dumps({"ok": not errors, "errors": errors}, indent=2))
            return 0 if not errors else 1
        if args.command == "review-patch":
            print(json.dumps(review_patch(args.role_id, parse_patch_arg(args.patch), store=args.store), indent=2))
            return 0
        if args.command == "apply-patch":
            patch = parse_patch_arg(args.patch)
            result = apply_self_patch(args.role_id, patch, agent_dir=args.agent_dir)
            print(json.dumps(result, indent=2))
            return 0 if result["status"] == "accepted" else 1
        if args.command == "smoke":
            result = run_smoke(args.store)
            print(json.dumps(result, indent=2))
            return 0 if result["ok"] else 1
    except ValueError as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, indent=2))
        return 1
    raise AssertionError(args.command)


if __name__ == "__main__":
    sys.exit(main())
