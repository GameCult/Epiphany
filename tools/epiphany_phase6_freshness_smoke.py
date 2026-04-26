from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Any

from epiphany_phase5_smoke import AppServerClient
from epiphany_phase5_smoke import DEFAULT_APP_SERVER
from epiphany_phase5_smoke import ROOT
from epiphany_phase5_smoke import require
from epiphany_phase5_smoke import reset_smoke_paths


DEFAULT_CODEX_HOME = ROOT / ".epiphany-smoke" / "phase6-freshness-codex-home"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "phase6-freshness-smoke-result.json"
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-smoke" / "phase6-freshness-smoke-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-smoke" / "phase6-freshness-smoke-server.stderr.log"

FRESHNESS_CODE_REF = {
    "path": "app-server/src/codex_message_processor.rs",
    "start_line": 4200,
    "end_line": 4340,
    "symbol": "thread_epiphany_freshness",
}


def freshness_patch() -> dict[str, Any]:
    return {
        "objective": "Expose read-only Epiphany freshness reflection without inventing watcher-driven invalidation.",
        "activeSubgoalId": "phase6-freshness-smoke",
        "subgoals": [
            {
                "id": "phase6-freshness-smoke",
                "title": "Live-smoke freshness reflection",
                "status": "active",
                "summary": "The freshness surface should reflect graph staleness and live retrieval state without mutation.",
            }
        ],
        "graphs": {
            "architecture": {
                "nodes": [
                    {
                        "id": "freshness-surface",
                        "title": "Freshness reflection surface",
                        "purpose": "Expose exact retrieval and graph freshness pressure without becoming a scheduler.",
                        "code_refs": [FRESHNESS_CODE_REF],
                    }
                ]
            },
            "dataflow": {"nodes": []},
            "links": [],
        },
        "graphFrontier": {
            "active_node_ids": ["freshness-surface"],
            "dirty_paths": ["app-server/src/codex_message_processor.rs"],
            "open_question_ids": ["q-freshness-gap"],
        },
        "graphCheckpoint": {
            "checkpoint_id": "ck-freshness-1",
            "graph_revision": 1,
            "summary": "Freshness smoke checkpoint",
            "frontier_node_ids": ["freshness-surface"],
            "open_question_ids": ["q-freshness-gap"],
        },
        "churn": {
            "understanding_status": "ready",
            "diff_pressure": "low",
            "graph_freshness": "stale",
            "unexplained_writes": 0,
        },
    }


def assert_missing_state_freshness(response: dict[str, Any]) -> None:
    require(
        response["source"] == "live",
        "freshness should report live source for a loaded thread",
    )
    require(
        "stateRevision" not in response,
        "missing-state freshness should not invent a revision",
    )
    retrieval = response["retrieval"]
    require(retrieval["status"] == "ready", "live retrieval freshness should be available")
    require(
        retrieval["note"] == "Retrieval catalog is ready.",
        "fresh live retrieval should report ready note",
    )
    graph = response["graph"]
    require(graph["status"] == "missing", "missing Epiphany state should block graph freshness")
    require(
        graph["note"] == "Epiphany state is missing, so graph freshness cannot be assessed.",
        "missing graph freshness should explain itself",
    )


def assert_ready_freshness(response: dict[str, Any]) -> None:
    require(response["source"] == "live", "ready freshness should report live source")
    require(response["stateRevision"] == 1, "freshness should preserve state revision identity")

    retrieval = response["retrieval"]
    require(
        retrieval["status"] == "ready",
        "live retrieval freshness should stay ready for the smoke workspace",
    )
    require(
        retrieval["semanticAvailable"] is True,
        "live retrieval freshness should report semantic availability",
    )

    graph = response["graph"]
    require(graph["status"] == "stale", "dirty graph frontier should report stale freshness")
    require(
        graph["graphFreshness"] == "stale",
        "graph freshness should expose the churn hint",
    )
    require(
        graph["checkpointId"] == "ck-freshness-1",
        "graph freshness should expose the checkpoint id",
    )
    require(graph["dirtyPathCount"] == 1, "graph freshness should count dirty paths")
    require(graph["dirtyPaths"] == ["app-server/src/codex_message_processor.rs"], "graph freshness should expose dirty paths")
    require(graph["openQuestionCount"] == 1, "graph freshness should count open questions")
    require(graph["openGapCount"] == 0, "graph freshness should count open gaps")


def run_smoke(args: argparse.Namespace) -> dict[str, Any]:
    app_server = args.app_server.resolve()
    if not app_server.exists():
        raise FileNotFoundError(f"codex app-server binary not found: {app_server}")

    codex_home = args.codex_home.resolve()
    result_path = args.result.resolve()
    transcript_path = args.transcript.resolve()
    stderr_path = args.stderr.resolve()
    reset_smoke_paths(codex_home, result_path, transcript_path, stderr_path)

    with AppServerClient(app_server, codex_home, transcript_path, stderr_path) as client:
        client.send(
            "initialize",
            {
                "clientInfo": {
                    "name": "epiphany-phase6-freshness-smoke",
                    "title": "Epiphany Phase 6 Freshness Smoke",
                    "version": "0.1.0",
                },
                "capabilities": {"experimentalApi": True},
            },
        )
        client.send("initialized", expect_response=False)
        started = client.send(
            "thread/start",
            {"cwd": str(ROOT / "epiphany-core"), "ephemeral": True},
        )
        assert started is not None
        thread_id = started["thread"]["id"]

        missing_notification_start = len(client.notifications)
        missing_response = client.send("thread/epiphany/freshness", {"threadId": thread_id})
        assert missing_response is not None
        require(
            missing_response["threadId"] == thread_id,
            "freshness response should echo thread id",
        )
        assert_missing_state_freshness(missing_response)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=missing_notification_start,
        )

        update_notification_start = len(client.notifications)
        update = client.send(
            "thread/epiphany/update",
            {"threadId": thread_id, "expectedRevision": 0, "patch": freshness_patch()},
        )
        assert update is not None
        require(update["revision"] == 1, "freshness smoke patch should advance revision to 1")
        client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            start_index=update_notification_start,
        )

        ready_notification_start = len(client.notifications)
        ready_response = client.send("thread/epiphany/freshness", {"threadId": thread_id})
        assert ready_response is not None
        assert_ready_freshness(ready_response)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=ready_notification_start,
        )

        final_read = client.send("thread/read", {"threadId": thread_id, "includeTurns": False})
        assert final_read is not None
        require(
            final_read["thread"]["epiphanyState"]["revision"] == 1,
            "freshness reflection should not mutate state revision",
        )

        result = {
            "threadId": thread_id,
            "codexHome": str(codex_home),
            "missingRetrievalStatus": missing_response["retrieval"]["status"],
            "missingGraphStatus": missing_response["graph"]["status"],
            "readyRevision": ready_response["stateRevision"],
            "readyRetrievalStatus": ready_response["retrieval"]["status"],
            "readyGraphStatus": ready_response["graph"]["status"],
            "readyDirtyPathCount": ready_response["graph"]["dirtyPathCount"],
            "freshnessNotificationCount": client.count_notifications(
                "thread/epiphany/stateUpdated",
                start_index=ready_notification_start,
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
        description="Live-smoke the Phase 6 Epiphany freshness reflection surface."
    )
    parser.add_argument("--app-server", type=Path, default=DEFAULT_APP_SERVER)
    parser.add_argument("--codex-home", type=Path, default=DEFAULT_CODEX_HOME)
    parser.add_argument("--result", type=Path, default=DEFAULT_RESULT)
    parser.add_argument("--transcript", type=Path, default=DEFAULT_TRANSCRIPT)
    parser.add_argument("--stderr", type=Path, default=DEFAULT_STDERR)
    args = parser.parse_args()
    result = run_smoke(args)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
