from __future__ import annotations

import argparse
import json
import os
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
STATE_DIR = ROOT / "state"
MAP_PATH = STATE_DIR / "map.yaml"
BRANCHES_PATH = STATE_DIR / "branches.json"
EVIDENCE_PATH = STATE_DIR / "evidence.jsonl"
LEDGER_STORE_PATH = STATE_DIR / "ledgers.msgpack"
STATE_LEDGER_BIN = "epiphany-state-ledger-store"
STATE_LEDGER_EXE = (
    Path(os.environ.get("CARGO_TARGET_DIR", r"C:\Users\Meta\.cargo-target-codex"))
    / "debug"
    / "epiphany-state-ledger-store.exe"
)


def native_ledger_command(*args: str) -> dict[str, Any]:
    if STATE_LEDGER_EXE.exists():
        command = [str(STATE_LEDGER_EXE), *args]
    else:
        command = [
            "cargo",
            "run",
            "--quiet",
            "--manifest-path",
            str(ROOT / "epiphany-core" / "Cargo.toml"),
            "--bin",
            STATE_LEDGER_BIN,
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
        message = completed.stderr.strip() or completed.stdout.strip() or f"{STATE_LEDGER_BIN} failed"
        raise ValueError(message)
    return json.loads(completed.stdout)


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def utc_stamp() -> str:
    return datetime.now(timezone.utc).replace(microsecond=0).isoformat()


def extract_map_field(name: str) -> str | None:
    prefix = f"  {name}:"
    for line in read_text(MAP_PATH).splitlines():
        if line.startswith(prefix):
            return line.split(":", 1)[1].strip()
    return None


def extract_active_subgoals() -> list[str]:
    lines = read_text(MAP_PATH).splitlines()
    results: list[str] = []
    in_section = False
    for line in lines:
        if not in_section:
            if line.strip() == "active_subgoals:":
                in_section = True
            continue
        if line.startswith("  - "):
            results.append(line[4:].strip())
            continue
        if line and not line.startswith(" "):
            break
    return results


def cmd_migrate_json(args: argparse.Namespace) -> int:
    print(
        json.dumps(
            native_ledger_command(
                "migrate-json",
                "--branches",
                str(args.branches),
                "--evidence",
                str(args.evidence),
                "--store",
                str(args.store),
            ),
            indent=2,
        )
    )
    return 0


def cmd_status(_: argparse.Namespace) -> int:
    ledger = native_ledger_command("status", "--store", str(LEDGER_STORE_PATH))
    summary = extract_map_field("summary") or "(missing)"
    next_action = extract_map_field("next_action") or "(missing)"
    subgoals = extract_active_subgoals()

    print(f"Workspace: {ROOT}")
    print(f"Summary: {summary}")
    print(f"Next action: {next_action}")
    print(f"Active branches: {ledger.get('activeBranches', 0)} / {ledger.get('branches', 0)}")
    print(f"Evidence records: {ledger.get('evidence', 0)}")
    if subgoals:
        print("Active subgoals:")
        for item in subgoals:
            print(f"- {item}")
    return 0


def cmd_add_evidence(args: argparse.Namespace) -> int:
    command = [
        "add-evidence",
        "--store",
        str(LEDGER_STORE_PATH),
        "--ts",
        utc_stamp(),
        "--type",
        args.type,
        "--status",
        args.status,
        "--note",
        args.note,
    ]
    if args.branch:
        command.extend(["--branch", args.branch])
    native_ledger_command(*command)
    print("Appended evidence record.")
    return 0


def cmd_add_branch(args: argparse.Namespace) -> int:
    command = [
        "add-branch",
        "--store",
        str(LEDGER_STORE_PATH),
        "--id",
        args.id,
        "--hypothesis",
        args.hypothesis,
    ]
    for artifact in args.artifact or []:
        command.extend(["--artifact", artifact])
    if args.note:
        command.extend(["--note", args.note])
    native_ledger_command(*command)
    print(f"Added branch '{args.id}'.")
    return 0


def cmd_close_branch(args: argparse.Namespace) -> int:
    command = [
        "close-branch",
        "--store",
        str(LEDGER_STORE_PATH),
        "--id",
        args.id,
        "--status",
        args.status,
    ]
    if args.note:
        command.extend(["--note", args.note])
    native_ledger_command(*command)
    print(f"Updated branch '{args.id}' to status '{args.status}'.")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Inspect and update EpiphanyAgent typed state ledgers."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    migrate_parser = subparsers.add_parser("migrate-json", help="One-shot migration from legacy JSON ledgers.")
    migrate_parser.add_argument("--branches", type=Path, default=BRANCHES_PATH)
    migrate_parser.add_argument("--evidence", type=Path, default=EVIDENCE_PATH)
    migrate_parser.add_argument("--store", type=Path, default=LEDGER_STORE_PATH)
    migrate_parser.set_defaults(func=cmd_migrate_json)

    status_parser = subparsers.add_parser("status", help="Show a compact state summary.")
    status_parser.set_defaults(func=cmd_status)

    evidence_parser = subparsers.add_parser(
        "add-evidence", help="Append one distilled evidence record."
    )
    evidence_parser.add_argument("--type", required=True)
    evidence_parser.add_argument("--status", required=True)
    evidence_parser.add_argument("--note", required=True)
    evidence_parser.add_argument("--branch")
    evidence_parser.set_defaults(func=cmd_add_evidence)

    add_branch_parser = subparsers.add_parser(
        "add-branch", help="Create a new active branch entry."
    )
    add_branch_parser.add_argument("--id", required=True)
    add_branch_parser.add_argument("--hypothesis", required=True)
    add_branch_parser.add_argument("--artifact", action="append")
    add_branch_parser.add_argument("--note")
    add_branch_parser.set_defaults(func=cmd_add_branch)

    close_branch_parser = subparsers.add_parser(
        "close-branch", help="Close an existing branch."
    )
    close_branch_parser.add_argument("--id", required=True)
    close_branch_parser.add_argument(
        "--status", required=True, choices=["accepted", "rejected", "archived"]
    )
    close_branch_parser.add_argument("--note")
    close_branch_parser.set_defaults(func=cmd_close_branch)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        return args.func(args)
    except ValueError as exc:
        print(json.dumps({"ok": False, "error": str(exc)}, indent=2))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
