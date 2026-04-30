from __future__ import annotations

import argparse
from datetime import datetime
from datetime import timezone
import json
import os
import time
from pathlib import Path
from typing import Any

from epiphany_mvp_status import DEFAULT_APP_SERVER
from epiphany_mvp_status import collect_status
from epiphany_mvp_status import render_status
from epiphany_mvp_status import sanitize_for_operator
from epiphany_phase5_smoke import AppServerClient
from epiphany_phase5_smoke import ROOT
from epiphany_phase6_reorient_launch_smoke import BINDING_ID as REORIENT_BINDING_ID


DEFAULT_CODEX_HOME = ROOT / ".epiphany-gui" / "codex-home"
DEFAULT_ARTIFACT_ROOT = ROOT / ".epiphany-gui" / "actions"


def checkpoint_patch(cwd: Path) -> dict[str, Any]:
    generated_at = datetime.now(timezone.utc).isoformat()
    return {
        "objective": "Reach a testable Epiphany MVP through the local GUI/operator workflow.",
        "activeSubgoalId": "epiphany-gui-mvp",
        "subgoals": [
            {
                "id": "epiphany-gui-mvp",
                "title": "Make the Epiphany GUI usable for local operator testing",
                "status": "active",
                "summary": "The operator can prepare durable state, inspect coordinator guidance, and launch/read fixed specialist lanes without terminal work.",
            }
        ],
        "graphs": {
            "architecture": {
                "nodes": [
                    {
                        "id": "gui-operator-console",
                        "title": "Tauri/React operator console",
                        "purpose": "Human-facing surface for status, artifacts, and bounded lane actions.",
                        "code_refs": [
                            {
                                "path": "apps/epiphany-gui/src/App.tsx",
                                "start_line": 1,
                                "end_line": 1,
                                "symbol": "App",
                            }
                        ],
                    },
                    {
                        "id": "gui-action-bridge",
                        "title": "GUI action bridge",
                        "purpose": "Python bridge that invokes fixed app-server Epiphany lane APIs and writes auditable artifacts.",
                        "code_refs": [
                            {
                                "path": "tools/epiphany_gui_action.py",
                                "start_line": 1,
                                "end_line": 1,
                                "symbol": "run_action",
                            }
                        ],
                    },
                    {
                        "id": "app-server-epiphany-control-plane",
                        "title": "App-server Epiphany control plane",
                        "purpose": "Typed state, coordinator, CRRC, role, reorient, and job surfaces used by the GUI.",
                        "code_refs": [
                            {
                                "path": "vendor/codex/codex-rs/app-server/src/codex_message_processor.rs",
                                "start_line": 1,
                                "end_line": 1,
                                "symbol": "thread/epiphany",
                            }
                        ],
                    },
                ]
            },
            "dataflow": {
                "nodes": [
                    {
                        "id": "operator-action-flow",
                        "title": "Operator action flow",
                        "purpose": "GUI button -> Tauri command -> Python bridge -> app-server API -> artifact bundle -> refreshed GUI status.",
                    }
                ]
            },
            "links": [
                {
                    "dataflow_node_id": "operator-action-flow",
                    "architecture_node_id": "gui-operator-console",
                    "relationship": "operator-surface",
                },
                {
                    "dataflow_node_id": "operator-action-flow",
                    "architecture_node_id": "gui-action-bridge",
                    "relationship": "bridge",
                },
                {
                    "dataflow_node_id": "operator-action-flow",
                    "architecture_node_id": "app-server-epiphany-control-plane",
                    "relationship": "control-plane",
                },
            ],
        },
        "graphFrontier": {
            "active_node_ids": [
                "gui-operator-console",
                "gui-action-bridge",
                "app-server-epiphany-control-plane",
            ],
            "dirty_paths": [],
        },
        "graphCheckpoint": {
            "checkpoint_id": f"gui-operator-{int(time.time())}",
            "graph_revision": 1,
            "summary": "GUI-prepared checkpoint for local Epiphany MVP operator testing.",
            "frontier_node_ids": [
                "gui-operator-console",
                "gui-action-bridge",
                "app-server-epiphany-control-plane",
            ],
        },
        "investigationCheckpoint": {
            "checkpoint_id": f"ix-gui-operator-{int(time.time())}",
            "kind": "source_gathering",
            "disposition": "resume_ready",
            "focus": "Validate the GUI operator loop against the fixed Epiphany lane APIs.",
            "summary": "The GUI is the operator surface; app-server typed state remains the source of truth.",
            "next_action": "Use the GUI to run coordinator plan, launch/read specialist lanes, and inspect written artifacts.",
            "captured_at_turn_id": f"gui-prepare-{int(time.time())}",
            "code_refs": [
                {
                    "path": "apps/epiphany-gui/src/App.tsx",
                    "start_line": 1,
                    "end_line": 1,
                    "symbol": "App",
                },
                {
                    "path": "tools/epiphany_gui_action.py",
                    "start_line": 1,
                    "end_line": 1,
                    "symbol": "run_action",
                },
            ],
        },
        "scratch": {
            "summary": "Prepared from the GUI so the operator has a durable thread before launching fixed lanes.",
            "current_focus": "Test the local GUI MVP workflow end to end.",
            "next_action": "Run coordinator guidance, then launch/read modeling, verification, or reorient lanes as recommended.",
            "updated_at": generated_at,
            "cwd": str(cwd),
        },
        "churn": {
            "understanding_status": "ready",
            "diff_pressure": "low",
            "graph_freshness": "fresh",
            "unexplained_writes": 0,
        },
    }


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def write_text(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value, encoding="utf-8")


def state_revision(status: dict[str, Any]) -> int | None:
    state = status.get("read", {}).get("thread", {}).get("epiphanyState")
    if isinstance(state, dict):
        revision = state.get("revision")
        if isinstance(revision, int):
            return revision
    scene_revision = status.get("scene", {}).get("scene", {}).get("revision")
    return scene_revision if isinstance(scene_revision, int) else None


def require_thread_id(thread_id: str | None, action: str) -> str:
    if not thread_id:
        raise ValueError(f"{action} requires a persistent thread id")
    return thread_id


def is_unresumable_empty_thread_error(error: RuntimeError) -> bool:
    message = str(error)
    return "no rollout found for thread id" in message or "thread not loaded" in message


def run_action(args: argparse.Namespace) -> dict[str, Any]:
    thread_id = args.thread_id or ""
    if args.action != "prepareCheckpoint":
        thread_id = require_thread_id(args.thread_id, args.action)
    codex_home = args.codex_home.resolve()
    artifact_dir = args.artifact_root.resolve() / f"{args.action}-{time.time_ns()}-{os.getpid()}"
    transcript_path = artifact_dir / "transcript.jsonl"
    stderr_path = artifact_dir / "server.stderr.log"
    cwd = args.cwd.resolve()
    codex_home.mkdir(parents=True, exist_ok=True)
    artifact_dir.mkdir(parents=True, exist_ok=True)

    with AppServerClient(args.app_server.resolve(), codex_home, transcript_path, stderr_path) as client:
        client.send(
            "initialize",
            {
                "clientInfo": {
                    "name": "epiphany-gui-action",
                    "title": "Epiphany GUI Action",
                    "version": "0.1.0",
                },
                "capabilities": {"experimentalApi": True},
            },
        )
        client.send("initialized", expect_response=False)
        if args.action == "prepareCheckpoint":
            if args.thread_id:
                try:
                    client.send("thread/resume", {"threadId": args.thread_id})
                except RuntimeError as error:
                    if not is_unresumable_empty_thread_error(error):
                        raise
                    thread_id = ""
            if not thread_id:
                started = client.send("thread/start", {"cwd": str(cwd), "ephemeral": False})
                if started is None:
                    raise RuntimeError("thread/start returned no response")
                thread_id = started["thread"]["id"]
            before = collect_status(client, thread_id=thread_id, cwd=cwd, ephemeral=True)
            revision = state_revision(before)
            response = client.send(
                "thread/epiphany/update",
                {
                    "threadId": thread_id,
                    "expectedRevision": revision if revision is not None else 0,
                    "patch": checkpoint_patch(cwd),
                },
            )
            summary = "Prepared durable Epiphany checkpoint."
        else:
            before = collect_status(client, thread_id=thread_id, cwd=cwd, ephemeral=False)
            revision = state_revision(before)

            if args.action in {"launchModeling", "launchVerification"}:
                role_id = "modeling" if args.action == "launchModeling" else "verification"
                payload: dict[str, Any] = {
                    "threadId": thread_id,
                    "roleId": role_id,
                    "maxRuntimeSeconds": args.max_runtime_seconds,
                }
                if revision is not None:
                    payload["expectedRevision"] = revision
                response = client.send("thread/epiphany/roleLaunch", payload)
                summary = f"Launched {role_id} role worker."
            elif args.action in {"readModelingResult", "readVerificationResult"}:
                role_id = "modeling" if args.action == "readModelingResult" else "verification"
                response = client.send(
                    "thread/epiphany/roleResult", {"threadId": thread_id, "roleId": role_id}
                )
                summary = f"Read {role_id} role result."
            elif args.action == "acceptModeling":
                if revision is None:
                    raise ValueError("acceptModeling requires ready Epiphany state with a revision")
                response = client.send(
                    "thread/epiphany/roleAccept",
                    {
                        "threadId": thread_id,
                        "roleId": "modeling",
                        "expectedRevision": revision,
                    },
                )
                summary = "Accepted reviewed modeling graph/checkpoint patch."
            elif args.action == "launchReorient":
                payload = {"threadId": thread_id, "maxRuntimeSeconds": args.max_runtime_seconds}
                if revision is not None:
                    payload["expectedRevision"] = revision
                response = client.send("thread/epiphany/reorientLaunch", payload)
                summary = "Launched fixed reorient-worker."
            elif args.action == "readReorientResult":
                response = client.send(
                    "thread/epiphany/reorientResult",
                    {"threadId": thread_id, "bindingId": REORIENT_BINDING_ID},
                )
                summary = "Read reorient-worker result."
            elif args.action == "acceptReorient":
                if revision is None:
                    raise ValueError("acceptReorient requires ready Epiphany state with a revision")
                response = client.send(
                    "thread/epiphany/reorientAccept",
                    {
                        "threadId": thread_id,
                        "expectedRevision": revision,
                        "bindingId": REORIENT_BINDING_ID,
                        "updateScratch": True,
                        "updateInvestigationCheckpoint": True,
                    },
                )
                summary = "Accepted reviewed reorientation finding."
            else:
                raise ValueError(f"unsupported GUI action: {args.action}")

        assert response is not None
        after = collect_status(client, thread_id=thread_id, cwd=cwd, ephemeral=True)

    operator_before = sanitize_for_operator(before)
    operator_response = sanitize_for_operator(response)
    operator_after = sanitize_for_operator(after)
    write_json(artifact_dir / "before-status.json", operator_before)
    write_json(artifact_dir / "action-response.json", operator_response)
    write_json(artifact_dir / "after-status.json", operator_after)
    write_text(artifact_dir / "after-status.txt", render_status(operator_after))
    result = {
        "action": args.action,
        "artifactPath": str(artifact_dir),
        "summary": summary,
        "threadId": thread_id,
        "response": operator_response,
        "sealedArtifactManifest": [
            {
                "path": "transcript.jsonl",
                "reason": "sealed JSON-RPC audit trail; do not read during normal supervision",
            },
            {
                "path": "server.stderr.log",
                "reason": "sealed app-server diagnostics; inspect only for explicit debugging",
            },
        ],
    }
    write_json(artifact_dir / "gui-action-summary.json", result)
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description="Run one bounded Epiphany GUI action.")
    parser.add_argument("--app-server", type=Path, default=DEFAULT_APP_SERVER)
    parser.add_argument("--codex-home", type=Path, default=DEFAULT_CODEX_HOME)
    parser.add_argument("--cwd", type=Path, default=ROOT)
    parser.add_argument("--thread-id")
    parser.add_argument("--artifact-root", type=Path, default=DEFAULT_ARTIFACT_ROOT)
    parser.add_argument("--max-runtime-seconds", type=int, default=180)
    parser.add_argument(
        "--action",
        required=True,
        choices=[
            "launchModeling",
            "readModelingResult",
            "acceptModeling",
            "launchVerification",
            "readVerificationResult",
            "launchReorient",
            "readReorientResult",
            "acceptReorient",
            "prepareCheckpoint",
        ],
    )
    args = parser.parse_args()
    print(json.dumps(run_action(args), indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
