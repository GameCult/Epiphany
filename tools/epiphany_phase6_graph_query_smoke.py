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


DEFAULT_CODEX_HOME = ROOT / ".epiphany-smoke" / "phase6-graph-query-codex-home"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "phase6-graph-query-smoke-result.json"
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-smoke" / "phase6-graph-query-smoke-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-smoke" / "phase6-graph-query-smoke-server.stderr.log"

GRAPH_QUERY_CODE_REF = {
    "path": "app-server/src/codex_message_processor.rs",
    "start_line": 15201,
    "end_line": 15534,
    "symbol": "map_epiphany_graph_query",
}


def graph_query_patch() -> dict[str, Any]:
    return {
        "objective": "Expose read-only graph traversal for implementation and verifier agents.",
        "activeSubgoalId": "phase6-graph-query-smoke",
        "subgoals": [
            {
                "id": "phase6-graph-query-smoke",
                "title": "Live-smoke graph query traversal",
                "status": "active",
                "summary": "The graph query surface should traverse typed graph state without mutating it.",
            }
        ],
        "graphs": {
            "architecture": {
                "nodes": [
                    {
                        "id": "graph-query-surface",
                        "title": "Graph query surface",
                        "purpose": "Return bounded graph neighborhoods and path matches for agents.",
                        "code_refs": [GRAPH_QUERY_CODE_REF],
                    },
                    {
                        "id": "implementation-agent",
                        "title": "Implementation agent",
                        "purpose": "Uses graph neighborhoods to decide where to edit.",
                    },
                    {
                        "id": "verifier-agent",
                        "title": "Verifier agent",
                        "purpose": "Uses graph neighborhoods to inspect blast radius.",
                    },
                ],
                "edges": [
                    {
                        "id": "edge-query-implementation",
                        "source_id": "graph-query-surface",
                        "target_id": "implementation-agent",
                        "kind": "guides",
                        "code_refs": [GRAPH_QUERY_CODE_REF],
                    },
                    {
                        "id": "edge-query-verifier",
                        "source_id": "graph-query-surface",
                        "target_id": "verifier-agent",
                        "kind": "guides",
                    },
                ],
            },
            "dataflow": {
                "nodes": [
                    {
                        "id": "typed-graph-state",
                        "title": "Typed graph state",
                        "purpose": "Remain the authoritative model behind graph traversal.",
                        "code_refs": [GRAPH_QUERY_CODE_REF],
                    }
                ]
            },
            "links": [
                {
                    "dataflow_node_id": "typed-graph-state",
                    "architecture_node_id": "graph-query-surface",
                    "relationship": "authoritative-query-source",
                    "code_refs": [GRAPH_QUERY_CODE_REF],
                }
            ],
        },
        "graphFrontier": {
            "active_node_ids": ["graph-query-surface"],
            "active_edge_ids": ["edge-query-implementation"],
        },
        "graphCheckpoint": {
            "checkpoint_id": "phase6-graph-query-smoke",
            "graph_revision": 1,
            "summary": "Graph query traversal is the active Phase 6 smoke target.",
            "frontier_node_ids": ["graph-query-surface"],
        },
    }


def assert_missing_graph_query(response: dict[str, Any], thread_id: str) -> None:
    require(response["threadId"] == thread_id, "graph query should echo thread id")
    require(response["source"] == "live", "missing graph query should report live source")
    require(response["stateStatus"] == "missing", "missing graph query should report missing state")
    require("stateRevision" not in response, "missing graph query should not invent a revision")
    require(response["graph"] == {}, "missing graph query should not invent graph records")
    require(
        response["missing"]["nodeIds"] == ["graph-query-surface"],
        "missing graph query should echo unresolved explicit node ids",
    )


def assert_frontier_graph_query(response: dict[str, Any]) -> None:
    require(response["stateStatus"] == "ready", "frontier graph query should report ready state")
    require(response["stateRevision"] == 1, "frontier graph query should preserve revision")
    require(
        [node["id"] for node in response["graph"]["architectureNodes"]]
        == ["graph-query-surface", "implementation-agent", "verifier-agent"],
        "frontier graph query should return one-hop architecture neighbors",
    )
    require(
        [node["id"] for node in response["graph"]["dataflowNodes"]] == ["typed-graph-state"],
        "frontier graph query should preserve linked dataflow node",
    )
    require(
        [edge["id"] for edge in response["graph"]["architectureEdges"]]
        == ["edge-query-implementation", "edge-query-verifier"],
        "frontier graph query should return active and incident architecture edges",
    )
    require(
        response["graph"]["links"][0]["dataflow_node_id"] == "typed-graph-state",
        "frontier graph query should return architecture/dataflow link",
    )
    require(
        response["frontier"]["active_node_ids"] == ["graph-query-surface"],
        "frontier graph query should include current frontier",
    )
    require(
        response["checkpoint"]["checkpoint_id"] == "phase6-graph-query-smoke",
        "frontier graph query should include graph checkpoint",
    )
    require(response["missing"] == {}, "frontier graph query should have no missing records")


def assert_path_graph_query(response: dict[str, Any]) -> None:
    require(response["stateStatus"] == "ready", "path graph query should report ready state")
    require(
        response["matched"]["paths"] == ["app-server/src/codex_message_processor.rs"],
        "path graph query should report matched code ref path",
    )
    require(
        response["matched"]["symbols"] == ["map_epiphany_graph_query"],
        "path graph query should report matched symbol",
    )
    require(
        "graph-query-surface" in response["matched"]["nodeIds"],
        "path graph query should include graph query node in matches",
    )
    require(
        "typed-graph-state" in response["matched"]["nodeIds"],
        "path graph query should include linked dataflow node in matches",
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
                    "name": "epiphany-phase6-graph-query-smoke",
                    "title": "Epiphany Phase 6 Graph Query Smoke",
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
            "thread/epiphany/graphQuery",
            {
                "threadId": thread_id,
                "query": {"kind": "node", "nodeIds": ["graph-query-surface"]},
            },
        )
        assert missing_response is not None
        assert_missing_graph_query(missing_response, thread_id)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=missing_notification_start,
        )

        update_notification_start = len(client.notifications)
        update = client.send(
            "thread/epiphany/update",
            {"threadId": thread_id, "expectedRevision": 0, "patch": graph_query_patch()},
        )
        assert update is not None
        require(update["revision"] == 1, "graph query smoke patch should advance revision to 1")
        client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            start_index=update_notification_start,
        )

        frontier_notification_start = len(client.notifications)
        frontier_response = client.send(
            "thread/epiphany/graphQuery",
            {
                "threadId": thread_id,
                "query": {
                    "kind": "frontierNeighborhood",
                    "direction": "outgoing",
                    "depth": 1,
                },
            },
        )
        assert frontier_response is not None
        assert_frontier_graph_query(frontier_response)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=frontier_notification_start,
        )

        path_notification_start = len(client.notifications)
        path_response = client.send(
            "thread/epiphany/graphQuery",
            {
                "threadId": thread_id,
                "query": {
                    "kind": "path",
                    "paths": ["app-server/src/codex_message_processor.rs"],
                    "symbols": ["map_epiphany_graph_query"],
                },
            },
        )
        assert path_response is not None
        assert_path_graph_query(path_response)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=path_notification_start,
        )

        final_read = client.send("thread/read", {"threadId": thread_id, "includeTurns": False})
        assert final_read is not None
        require(
            final_read["thread"]["epiphanyState"]["revision"] == 1,
            "graph query reflection should not mutate state revision",
        )

        result = {
            "threadId": thread_id,
            "codexHome": str(codex_home),
            "missingStateStatus": missing_response["stateStatus"],
            "frontierStateStatus": frontier_response["stateStatus"],
            "pathStateStatus": path_response["stateStatus"],
            "readyRevision": frontier_response["stateRevision"],
            "frontierArchitectureNodeIds": [
                node["id"] for node in frontier_response["graph"]["architectureNodes"]
            ],
            "frontierDataflowNodeIds": [
                node["id"] for node in frontier_response["graph"]["dataflowNodes"]
            ],
            "pathMatchedSymbols": path_response["matched"]["symbols"],
            "frontierNotificationCount": client.count_notifications(
                "thread/epiphany/stateUpdated",
                start_index=frontier_notification_start,
            ),
            "pathNotificationCount": client.count_notifications(
                "thread/epiphany/stateUpdated",
                start_index=path_notification_start,
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
        description="Live-smoke the Phase 6 Epiphany graph query traversal surface."
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
