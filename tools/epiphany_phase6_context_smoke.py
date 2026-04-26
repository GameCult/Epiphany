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


DEFAULT_CODEX_HOME = ROOT / ".epiphany-smoke" / "phase6-context-codex-home"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "phase6-context-smoke-result.json"
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-smoke" / "phase6-context-smoke-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-smoke" / "phase6-context-smoke-server.stderr.log"

CONTEXT_CODE_REF = {
    "path": "app-server/src/codex_message_processor.rs",
    "start_line": 10791,
    "end_line": 10989,
    "symbol": "map_epiphany_context",
}


def context_patch() -> dict[str, Any]:
    return {
        "objective": "Expose a read-only Epiphany context shard without returning the full state.",
        "activeSubgoalId": "phase6-context-smoke",
        "subgoals": [
            {
                "id": "phase6-context-smoke",
                "title": "Live-smoke context shard reflection",
                "status": "active",
                "summary": "The app-server context surface should return targeted graph/evidence state.",
            }
        ],
        "graphs": {
            "architecture": {
                "nodes": [
                    {
                        "id": "context-surface",
                        "title": "Context shard surface",
                        "purpose": "Return bounded state context for clients without becoming a writer.",
                        "code_refs": [CONTEXT_CODE_REF],
                    }
                ],
                "edges": [
                    {
                        "id": "context-edge",
                        "source_id": "context-surface",
                        "target_id": "context-surface",
                        "kind": "reflects",
                        "code_refs": [CONTEXT_CODE_REF],
                    }
                ],
            },
            "dataflow": {
                "nodes": [
                    {
                        "id": "typed-state",
                        "title": "Typed Epiphany state",
                        "purpose": "Remain authoritative while context shards reflect a slice.",
                    }
                ]
            },
            "links": [
                {
                    "dataflow_node_id": "typed-state",
                    "architecture_node_id": "context-surface",
                    "relationship": "bounded-reflection",
                }
            ],
        },
        "graphFrontier": {
            "active_node_ids": ["context-surface"],
            "active_edge_ids": ["context-edge"],
        },
        "graphCheckpoint": {
            "checkpoint_id": "phase6-context-smoke",
            "graph_revision": 1,
            "summary": "Context shard reflection is the active Phase 6 smoke target.",
            "frontier_node_ids": ["context-surface"],
        },
        "evidence": [
            {
                "id": "ev-context-linked",
                "kind": "smoke-test",
                "status": "ok",
                "summary": "Context smoke linked evidence should come along with the observation.",
                "code_refs": [CONTEXT_CODE_REF],
            },
            {
                "id": "ev-context-extra",
                "kind": "review",
                "status": "ok",
                "summary": "Context smoke direct evidence should be returned when requested.",
                "code_refs": [CONTEXT_CODE_REF],
            },
        ],
        "observations": [
            {
                "id": "obs-context",
                "summary": "Context smoke observation should be selected by id.",
                "source_kind": "smoke",
                "status": "ok",
                "code_refs": [CONTEXT_CODE_REF],
                "evidence_ids": ["ev-context-linked"],
            }
        ],
    }


def assert_missing_context(response: dict[str, Any]) -> None:
    require(response["source"] == "live", "missing context should report live source")
    require(response["stateStatus"] == "missing", "missing context should report missing state")
    require("stateRevision" not in response, "missing context should not invent a revision")
    require(
        response["context"]["graph"] == {},
        "missing context should not invent graph records",
    )
    require(
        response["missing"]["graphNodeIds"] == ["context-surface"],
        "missing context should echo requested missing node ids",
    )


def assert_ready_context(response: dict[str, Any]) -> None:
    require(response["source"] == "live", "ready context should report live source")
    require(response["stateStatus"] == "ready", "ready context should report ready state")
    require(response["stateRevision"] == 1, "ready context should preserve state revision identity")
    require(
        [node["id"] for node in response["context"]["graph"]["architectureNodes"]]
        == ["context-surface"],
        "context should include active frontier architecture node",
    )
    require(
        [edge["id"] for edge in response["context"]["graph"]["architectureEdges"]]
        == ["context-edge"],
        "context should include active frontier architecture edge",
    )
    require(
        response["context"]["graph"]["links"][0]["architecture_node_id"] == "context-surface",
        "context should include links touching selected graph nodes",
    )
    require(
        response["context"]["frontier"]["active_node_ids"] == ["context-surface"],
        "context should include frontier when active frontier is requested by default",
    )
    require(
        response["context"]["checkpoint"]["checkpoint_id"] == "phase6-context-smoke",
        "context should expose the current graph checkpoint",
    )
    require(
        [record["id"] for record in response["context"]["observations"]] == ["obs-context"],
        "context should include requested observation",
    )
    require(
        [record["id"] for record in response["context"]["evidence"]]
        == ["ev-context-linked", "ev-context-extra"],
        "context should include linked and directly requested evidence",
    )
    require(
        response["missing"] == {
            "graphNodeIds": ["missing-node"],
            "graphEdgeIds": ["missing-edge"],
            "observationIds": ["missing-observation"],
            "evidenceIds": ["missing-evidence"],
        },
        "context should report unresolved requested ids",
    )


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
                    "name": "epiphany-phase6-context-smoke",
                    "title": "Epiphany Phase 6 Context Smoke",
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
        missing_response = client.send(
            "thread/epiphany/context",
            {"threadId": thread_id, "graphNodeIds": ["context-surface"]},
        )
        assert missing_response is not None
        require(
            missing_response["threadId"] == thread_id,
            "context response should echo thread id",
        )
        assert_missing_context(missing_response)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=missing_notification_start,
        )

        update_notification_start = len(client.notifications)
        update = client.send(
            "thread/epiphany/update",
            {"threadId": thread_id, "expectedRevision": 0, "patch": context_patch()},
        )
        assert update is not None
        require(update["revision"] == 1, "context smoke patch should advance revision to 1")
        client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            start_index=update_notification_start,
        )

        context_notification_start = len(client.notifications)
        ready_response = client.send(
            "thread/epiphany/context",
            {
                "threadId": thread_id,
                "graphNodeIds": ["missing-node"],
                "graphEdgeIds": ["missing-edge"],
                "observationIds": ["obs-context", "missing-observation"],
                "evidenceIds": ["ev-context-extra", "missing-evidence"],
            },
        )
        assert ready_response is not None
        assert_ready_context(ready_response)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=context_notification_start,
        )

        final_read = client.send("thread/read", {"threadId": thread_id, "includeTurns": False})
        assert final_read is not None
        require(
            final_read["thread"]["epiphanyState"]["revision"] == 1,
            "context reflection should not mutate state revision",
        )

        result = {
            "threadId": thread_id,
            "codexHome": str(codex_home),
            "missingStateStatus": missing_response["stateStatus"],
            "readyStateStatus": ready_response["stateStatus"],
            "readyRevision": ready_response["stateRevision"],
            "architectureNodeIds": [
                node["id"] for node in ready_response["context"]["graph"]["architectureNodes"]
            ],
            "evidenceIds": [record["id"] for record in ready_response["context"]["evidence"]],
            "contextNotificationCount": client.count_notifications(
                "thread/epiphany/stateUpdated",
                start_index=context_notification_start,
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
        description="Live-smoke the Phase 6 Epiphany context shard surface."
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
