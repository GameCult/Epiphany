from __future__ import annotations

import argparse
import json
import shutil
import time
from pathlib import Path
from typing import Any

from epiphany_phase5_smoke import AppServerClient
from epiphany_phase5_smoke import DEFAULT_APP_SERVER
from epiphany_phase5_smoke import ROOT
from epiphany_phase5_smoke import require
from epiphany_phase5_smoke import reset_smoke_paths


DEFAULT_CODEX_HOME = ROOT / ".epiphany-smoke" / "phase6-reorient-codex-home"
DEFAULT_WORKSPACE = ROOT / ".epiphany-smoke" / "phase6-reorient-workspace"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "phase6-reorient-smoke-result.json"
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-smoke" / "phase6-reorient-smoke-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-smoke" / "phase6-reorient-smoke-server.stderr.log"

WATCHED_RELATIVE_PATH = Path("src") / "reorient_target.rs"


def prepare_workspace(workspace: Path) -> Path:
    if workspace.exists():
        shutil.rmtree(workspace)
    watched_file = workspace / WATCHED_RELATIVE_PATH
    watched_file.parent.mkdir(parents=True, exist_ok=True)
    watched_file.write_text(
        "pub fn reorient_target() -> &'static str {\n    \"before\"\n}\n",
        encoding="utf-8",
    )
    return watched_file


def reorient_patch() -> dict[str, Any]:
    return {
        "objective": "Decide whether a durable checkpoint still deserves to be resumed after rehydrate.",
        "activeSubgoalId": "phase6-reorient-smoke",
        "subgoals": [
            {
                "id": "phase6-reorient-smoke",
                "title": "Live-smoke CRRC reorientation policy",
                "status": "active",
                "summary": "Resume when the checkpoint is still aligned; regather when the touched file proves it isn't.",
            }
        ],
        "graphs": {
            "architecture": {
                "nodes": [
                    {
                        "id": "reorient-target",
                        "title": "Reorient target",
                        "purpose": "Map the file the watcher will touch so reorientation can notice drift.",
                        "code_refs": [
                            {
                                "path": WATCHED_RELATIVE_PATH.as_posix(),
                                "start_line": 1,
                                "end_line": 3,
                                "symbol": "reorient_target",
                            }
                        ],
                    }
                ]
            },
            "dataflow": {"nodes": []},
            "links": [],
        },
        "graphFrontier": {
            "active_node_ids": ["reorient-target"],
            "dirty_paths": [],
        },
        "graphCheckpoint": {
            "checkpoint_id": "ck-reorient-1",
            "graph_revision": 1,
            "summary": "Reorientation smoke graph checkpoint",
            "frontier_node_ids": ["reorient-target"],
        },
        "investigationCheckpoint": {
            "checkpoint_id": "ix-reorient-1",
            "kind": "source_gathering",
            "disposition": "resume_ready",
            "focus": "Verify the touched file before broad edits.",
            "summary": "This checkpoint should remain resumable until the watched source moves.",
            "next_action": "Resume the bounded slice if the watched source still matches the checkpoint.",
            "captured_at_turn_id": "turn-phase6-reorient",
            "code_refs": [
                {
                    "path": WATCHED_RELATIVE_PATH.as_posix(),
                    "start_line": 1,
                    "end_line": 3,
                    "symbol": "reorient_target",
                }
            ],
        },
        "churn": {
            "understanding_status": "ready",
            "diff_pressure": "low",
            "graph_freshness": "fresh",
            "unexplained_writes": 0,
        },
    }


def assert_missing_reorient(response: dict[str, Any]) -> None:
    require(response["source"] == "live", "missing reorient response should report live source")
    require(response["stateStatus"] == "missing", "missing reorient response should report missing state")
    decision = response["decision"]
    require(decision["action"] == "regather", "missing reorient response should regather")
    require(
        decision["checkpointStatus"] == "missing",
        "missing reorient response should report missing checkpoint",
    )
    require(
        decision["reasons"] == ["missingState", "missingCheckpoint"],
        "missing reorient response should explain missing state and checkpoint",
    )


def assert_ready_reorient(response: dict[str, Any]) -> None:
    require(response["source"] == "live", "ready reorient response should report live source")
    require(response["stateStatus"] == "ready", "ready reorient response should report ready state")
    require(response["stateRevision"] == 1, "ready reorient response should preserve revision identity")
    decision = response["decision"]
    require(decision["action"] == "resume", "clean checkpoint should remain resumable")
    require(
        decision["checkpointStatus"] == "resumeReady",
        "ready reorient response should report resume-ready checkpoint status",
    )
    require(
        decision["reasons"] == ["checkpointReady"],
        "ready reorient response should explain that the checkpoint is still aligned",
    )
    require(
        decision["checkpointId"] == "ix-reorient-1",
        "ready reorient response should expose the checkpoint id",
    )
    require(
        decision["nextAction"]
        == "Resume the bounded slice if the watched source still matches the checkpoint.",
        "ready reorient response should preserve checkpoint next action",
    )
    require(
        decision["watcherStatus"] == "clean",
        "ready reorient response should report a clean watcher before drift",
    )


def wait_for_regather_reorient(
    client: AppServerClient,
    thread_id: str,
    *,
    timeout: float = 10.0,
) -> dict[str, Any]:
    deadline = time.time() + timeout
    last_response: dict[str, Any] | None = None
    while time.time() < deadline:
        response = client.send("thread/epiphany/reorient", {"threadId": thread_id})
        assert response is not None
        last_response = response
        if response["decision"]["action"] == "regather":
            return response
        time.sleep(0.2)
    raise AssertionError(
        f"reorientation policy did not switch to regather before timeout; last response: {last_response!r}"
    )


def assert_regather_reorient(response: dict[str, Any]) -> None:
    decision = response["decision"]
    normalized_changed_paths = [
        path.replace("\\", "/") for path in decision["checkpointChangedPaths"]
    ]
    require(decision["action"] == "regather", "touched checkpoint path should force regather")
    require(
        decision["checkpointStatus"] == "resumeReady",
        "regather response should preserve the underlying checkpoint disposition",
    )
    require(
        decision["reasons"] == ["checkpointPathsChanged", "frontierChanged"],
        "regather response should explain watcher path and frontier drift",
    )
    require(
        WATCHED_RELATIVE_PATH.as_posix() in normalized_changed_paths,
        "regather response should report the changed checkpoint path",
    )
    require(
        decision["activeFrontierNodeIds"] == ["reorient-target"],
        "regather response should report touched frontier nodes",
    )
    require(
        decision["note"].startswith("Re-gather before editing:"),
        "regather response should explain why the checkpoint is no longer safe to resume",
    )


def run_smoke(args: argparse.Namespace) -> dict[str, Any]:
    app_server = args.app_server.resolve()
    if not app_server.exists():
        raise FileNotFoundError(f"codex app-server binary not found: {app_server}")

    codex_home = args.codex_home.resolve()
    workspace = args.workspace.resolve()
    watched_file = prepare_workspace(workspace)
    result_path = args.result.resolve()
    transcript_path = args.transcript.resolve()
    stderr_path = args.stderr.resolve()
    reset_smoke_paths(codex_home, result_path, transcript_path, stderr_path)

    with AppServerClient(app_server, codex_home, transcript_path, stderr_path) as client:
        client.send(
            "initialize",
            {
                "clientInfo": {
                    "name": "epiphany-phase6-reorient-smoke",
                    "title": "Epiphany Phase 6 Reorient Smoke",
                    "version": "0.1.0",
                },
                "capabilities": {"experimentalApi": True},
            },
        )
        client.send("initialized", expect_response=False)
        started = client.send("thread/start", {"cwd": str(workspace), "ephemeral": True})
        assert started is not None
        thread_id = started["thread"]["id"]

        missing_notification_start = len(client.notifications)
        missing_response = client.send("thread/epiphany/reorient", {"threadId": thread_id})
        assert missing_response is not None
        assert_missing_reorient(missing_response)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=missing_notification_start,
        )

        update_notification_start = len(client.notifications)
        update = client.send(
            "thread/epiphany/update",
            {"threadId": thread_id, "expectedRevision": 0, "patch": reorient_patch()},
        )
        assert update is not None
        require(update["revision"] == 1, "reorient smoke patch should advance revision to 1")
        client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            start_index=update_notification_start,
        )

        ready_notification_start = len(client.notifications)
        ready_response = client.send("thread/epiphany/reorient", {"threadId": thread_id})
        assert ready_response is not None
        assert_ready_reorient(ready_response)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=ready_notification_start,
        )

        watched_file.write_text(
            "pub fn reorient_target() -> &'static str {\n    \"after\"\n}\n",
            encoding="utf-8",
        )

        regather_notification_start = len(client.notifications)
        regather_response = wait_for_regather_reorient(client, thread_id)
        assert_regather_reorient(regather_response)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=regather_notification_start,
        )

        final_read = client.send("thread/read", {"threadId": thread_id, "includeTurns": False})
        assert final_read is not None
        require(
            final_read["thread"]["epiphanyState"]["revision"] == 1,
            "reorient reflection should not mutate durable state",
        )

        result = {
            "threadId": thread_id,
            "codexHome": str(codex_home),
            "workspace": str(workspace),
            "missingAction": missing_response["decision"]["action"],
            "readyAction": ready_response["decision"]["action"],
            "regatherAction": regather_response["decision"]["action"],
            "regatherReasons": regather_response["decision"]["reasons"],
            "checkpointChangedPaths": regather_response["decision"]["checkpointChangedPaths"],
            "activeFrontierNodeIds": regather_response["decision"]["activeFrontierNodeIds"],
            "stateUpdatedNotificationCount": client.count_notifications(
                "thread/epiphany/stateUpdated",
                start_index=regather_notification_start,
            ),
            "finalReadRevision": final_read["thread"]["epiphanyState"]["revision"],
        }

    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(
        json.dumps(result, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    return result


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Live-smoke the Phase 6 Epiphany reorientation policy surface."
    )
    parser.add_argument("--app-server", type=Path, default=DEFAULT_APP_SERVER)
    parser.add_argument("--codex-home", type=Path, default=DEFAULT_CODEX_HOME)
    parser.add_argument("--workspace", type=Path, default=DEFAULT_WORKSPACE)
    parser.add_argument("--result", type=Path, default=DEFAULT_RESULT)
    parser.add_argument("--transcript", type=Path, default=DEFAULT_TRANSCRIPT)
    parser.add_argument("--stderr", type=Path, default=DEFAULT_STDERR)
    args = parser.parse_args()
    result = run_smoke(args)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
