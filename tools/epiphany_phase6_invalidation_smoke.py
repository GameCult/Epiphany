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


DEFAULT_CODEX_HOME = ROOT / ".epiphany-smoke" / "phase6-invalidation-codex-home"
DEFAULT_WORKSPACE = ROOT / ".epiphany-smoke" / "phase6-invalidation-workspace"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "phase6-invalidation-smoke-result.json"
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-smoke" / "phase6-invalidation-smoke-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-smoke" / "phase6-invalidation-smoke-server.stderr.log"

WATCHED_RELATIVE_PATH = Path("src") / "watcher_target.rs"


def prepare_workspace(workspace: Path) -> Path:
    if workspace.exists():
        shutil.rmtree(workspace)
    watched_file = workspace / WATCHED_RELATIVE_PATH
    watched_file.parent.mkdir(parents=True, exist_ok=True)
    watched_file.write_text(
        "pub fn watcher_target() -> &'static str {\n    \"before\"\n}\n",
        encoding="utf-8",
    )
    return watched_file


def invalidation_patch() -> dict[str, Any]:
    return {
        "objective": "Expose watcher-backed invalidation inputs through the freshness lens without mutating Epiphany state.",
        "activeSubgoalId": "phase6-invalidation-smoke",
        "subgoals": [
            {
                "id": "phase6-invalidation-smoke",
                "title": "Watch live file changes",
                "status": "active",
                "summary": "Freshness should reflect watcher input as a read-only invalidation hint.",
            }
        ],
        "graphs": {
            "architecture": {
                "nodes": [
                    {
                        "id": "watcher-target",
                        "title": "Watcher target",
                        "purpose": "Expose a mapped file that the workspace watcher can touch.",
                        "code_refs": [
                            {
                                "path": WATCHED_RELATIVE_PATH.as_posix(),
                                "start_line": 1,
                                "end_line": 3,
                                "symbol": "watcher_target",
                            }
                        ],
                    }
                ]
            },
            "dataflow": {"nodes": []},
            "links": [],
        },
        "graphFrontier": {
            "active_node_ids": ["watcher-target"],
            "dirty_paths": [],
        },
        "graphCheckpoint": {
            "checkpoint_id": "ck-invalidation-1",
            "graph_revision": 1,
            "summary": "Watcher invalidation smoke checkpoint",
            "frontier_node_ids": ["watcher-target"],
        },
        "churn": {
            "understanding_status": "ready",
            "diff_pressure": "low",
            "graph_freshness": "fresh",
            "unexplained_writes": 0,
        },
    }


def assert_clean_watcher(response: dict[str, Any]) -> None:
    watcher = response["watcher"]
    require(response["source"] == "live", "invalidation freshness should report live source")
    require(watcher["status"] == "clean", "watcher should start clean before file changes")
    require(
        watcher["watchedRoot"].endswith("phase6-invalidation-workspace"),
        "watcher should report the live workspace root",
    )
    require(watcher["changedPathCount"] == 0, "clean watcher should not report changed paths")


def wait_for_changed_watcher(
    client: AppServerClient,
    thread_id: str,
    *,
    timeout: float = 10.0,
) -> dict[str, Any]:
    deadline = time.time() + timeout
    last_response: dict[str, Any] | None = None
    while time.time() < deadline:
        response = client.send("thread/epiphany/freshness", {"threadId": thread_id})
        assert response is not None
        last_response = response
        if response["watcher"]["status"] == "changed":
            return response
        time.sleep(0.2)
    raise AssertionError(
        f"watcher-backed invalidation did not surface before timeout; last response: {last_response!r}"
    )


def assert_changed_watcher(response: dict[str, Any]) -> None:
    watcher = response["watcher"]
    normalized_changed_paths = [path.replace("\\", "/") for path in watcher["changedPaths"]]
    require(watcher["status"] == "changed", "watcher should report changed after file write")
    require(watcher["changedPathCount"] >= 1, "changed watcher should report path count")
    require(
        WATCHED_RELATIVE_PATH.as_posix() in normalized_changed_paths,
        "changed watcher should report the touched relative path",
    )
    require(
        watcher["graphNodeIds"] == ["watcher-target"],
        "changed watcher should identify the mapped graph node",
    )
    require(
        watcher["activeFrontierNodeIds"] == ["watcher-target"],
        "changed watcher should identify active frontier nodes touched by the change",
    )
    require(
        isinstance(watcher.get("observedAtUnixSeconds"), int),
        "changed watcher should record when the change was observed",
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
                    "name": "epiphany-phase6-invalidation-smoke",
                    "title": "Epiphany Phase 6 Invalidation Smoke",
                    "version": "0.1.0",
                },
                "capabilities": {"experimentalApi": True},
            },
        )
        client.send("initialized", expect_response=False)
        started = client.send("thread/start", {"cwd": str(workspace), "ephemeral": True})
        assert started is not None
        thread_id = started["thread"]["id"]

        clean_notification_start = len(client.notifications)
        clean_response = client.send("thread/epiphany/freshness", {"threadId": thread_id})
        assert clean_response is not None
        assert_clean_watcher(clean_response)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=clean_notification_start,
        )

        update_notification_start = len(client.notifications)
        update = client.send(
            "thread/epiphany/update",
            {"threadId": thread_id, "expectedRevision": 0, "patch": invalidation_patch()},
        )
        assert update is not None
        require(update["revision"] == 1, "invalidation smoke patch should advance revision to 1")
        client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            start_index=update_notification_start,
        )

        ready_notification_start = len(client.notifications)
        ready_response = client.send("thread/epiphany/freshness", {"threadId": thread_id})
        assert ready_response is not None
        assert_clean_watcher(ready_response)
        require(
            ready_response["stateRevision"] == 1,
            "watcher-backed freshness should preserve state revision identity",
        )
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=ready_notification_start,
        )

        watched_file.write_text(
            "pub fn watcher_target() -> &'static str {\n    \"after\"\n}\n",
            encoding="utf-8",
        )

        changed_notification_start = len(client.notifications)
        changed_response = wait_for_changed_watcher(client, thread_id)
        assert_changed_watcher(changed_response)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=changed_notification_start,
        )

        final_read = client.send("thread/read", {"threadId": thread_id, "includeTurns": False})
        assert final_read is not None
        require(
            final_read["thread"]["epiphanyState"]["revision"] == 1,
            "watcher-backed freshness should not mutate durable state",
        )

        result = {
            "threadId": thread_id,
            "codexHome": str(codex_home),
            "workspace": str(workspace),
            "initialWatcherStatus": clean_response["watcher"]["status"],
            "readyWatcherStatus": ready_response["watcher"]["status"],
            "changedWatcherStatus": changed_response["watcher"]["status"],
            "changedPaths": changed_response["watcher"]["changedPaths"],
            "graphNodeIds": changed_response["watcher"]["graphNodeIds"],
            "activeFrontierNodeIds": changed_response["watcher"]["activeFrontierNodeIds"],
            "changedWatcherNotificationCount": client.count_notifications(
                "thread/epiphany/stateUpdated",
                start_index=changed_notification_start,
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
        description="Live-smoke watcher-backed Epiphany invalidation inputs."
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
