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


DEFAULT_CODEX_HOME = ROOT / ".epiphany-smoke" / "phase6-scene-codex-home"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "phase6-scene-smoke-result.json"
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-smoke" / "phase6-scene-smoke-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-smoke" / "phase6-scene-smoke-server.stderr.log"

MAPPER_CODE_REF = {
    "path": "app-server/src/codex_message_processor.rs",
    "start_line": 10573,
    "end_line": 10689,
    "symbol": "map_epiphany_scene",
}


def evidence_record(index: int) -> dict[str, Any]:
    return {
        "id": f"ev-phase6-scene-{index}",
        "kind": "smoke-test",
        "status": "ok",
        "summary": f"Scene smoke evidence {index} should appear newest-first.",
        "code_refs": [MAPPER_CODE_REF],
    }


def observation_record(index: int) -> dict[str, Any]:
    return {
        "id": f"obs-phase6-scene-{index}",
        "summary": f"Scene smoke observation {index} should appear newest-first.",
        "source_kind": "smoke",
        "status": "ok",
        "code_refs": [MAPPER_CODE_REF],
        "evidence_ids": [f"ev-phase6-scene-{index}"],
    }


def initial_scene_patch() -> dict[str, Any]:
    return {
        "objective": "Expose live Epiphany scene reflection without creating a second source of truth.",
        "activeSubgoalId": "phase6-scene-smoke",
        "subgoals": [
            {
                "id": "phase6-scene-smoke",
                "title": "Live-smoke scene reflection",
                "status": "active",
                "summary": "The app-server scene surface should reflect the live typed state.",
            },
            {
                "id": "phase6-job-surface",
                "title": "Design job/progress reflection",
                "status": "queued",
                "summary": "The next larger organ after scene smoke.",
            },
        ],
        "invariants": [
            {
                "id": "inv-scene-read-only",
                "description": "thread/epiphany/scene must not mutate Epiphany state.",
                "status": "ok",
            },
            {
                "id": "inv-gui-not-source",
                "description": "Scene projection may reflect state but must not become canonical understanding.",
                "status": "ok",
            },
        ],
        "graphs": {
            "architecture": {
                "nodes": [
                    {
                        "id": "scene-projection",
                        "title": "Scene projection",
                        "purpose": "Compress authoritative Epiphany state into a client-readable reflection.",
                        "code_refs": [MAPPER_CODE_REF],
                    }
                ]
            },
            "dataflow": {
                "nodes": [
                    {
                        "id": "typed-state",
                        "title": "Typed Epiphany state",
                        "purpose": "Remain the authoritative source behind scene reflection.",
                    }
                ]
            },
            "links": [
                {
                    "dataflow_node_id": "typed-state",
                    "architecture_node_id": "scene-projection",
                    "relationship": "derived-reflection",
                }
            ],
        },
        "graphFrontier": {
            "active_node_ids": ["scene-projection"],
            "dirty_paths": ["app-server/src/codex_message_processor.rs"],
        },
        "graphCheckpoint": {
            "checkpoint_id": "phase6-scene-smoke",
            "graph_revision": 1,
            "summary": "Scene reflection is the active Phase 6 smoke target.",
            "frontier_node_ids": ["scene-projection"],
        },
        "churn": {
            "understanding_status": "ready",
            "diff_pressure": "low",
            "graph_freshness": "fresh",
            "unexplained_writes": 0,
        },
        "evidence": [evidence_record(1)],
        "observations": [observation_record(1)],
    }


def assert_missing_scene(scene: dict[str, Any]) -> None:
    require(scene["stateStatus"] == "missing", "initial scene should report missing state")
    require(scene["source"] == "live", "loaded missing scene should report live source")
    require(
        scene["availableActions"] == ["index", "retrieve", "distill", "jobs", "update"],
        "missing live scene should expose only bootstrap actions",
    )
    require(
        scene["observations"]["totalCount"] == 0,
        "missing scene should not report observations",
    )
    require(scene["evidence"]["totalCount"] == 0, "missing scene should not report evidence")


def assert_ready_scene(scene: dict[str, Any], expected_revision: int) -> None:
    require(scene["stateStatus"] == "ready", "scene should report ready state after update")
    require(scene["source"] == "live", "loaded scene should report live source")
    require(scene["revision"] == expected_revision, "scene should expose current revision")
    require(
        scene["objective"].startswith("Expose live Epiphany scene reflection"),
        "scene should expose objective",
    )
    require(
        scene["activeSubgoal"]["id"] == "phase6-scene-smoke",
        "scene should expose active subgoal",
    )
    require(
        scene["invariantStatusCounts"] == [{"status": "ok", "count": 2}],
        "scene should summarize invariant status counts",
    )
    require(scene["graph"]["architectureNodeCount"] == 1, "scene should count architecture nodes")
    require(scene["graph"]["dataflowNodeCount"] == 1, "scene should count dataflow nodes")
    require(scene["graph"]["linkCount"] == 1, "scene should count graph links")
    require(
        scene["graph"]["activeNodeIds"] == ["scene-projection"],
        "scene should expose graph frontier active nodes",
    )
    require(
        scene["graph"]["checkpointId"] == "phase6-scene-smoke",
        "scene should expose graph checkpoint",
    )
    require(
        scene["retrieval"]["workspaceRoot"].endswith("epiphany-core"),
        "scene should include live retrieval summary backfill",
    )
    require(scene["observations"]["totalCount"] == 6, "scene should count all observations")
    require(scene["evidence"]["totalCount"] == 6, "scene should count all evidence")
    require(
        [record["id"] for record in scene["observations"]["latest"]]
        == [
            "obs-phase6-scene-6",
            "obs-phase6-scene-5",
            "obs-phase6-scene-4",
            "obs-phase6-scene-3",
            "obs-phase6-scene-2",
        ],
        "scene latest observations should be newest-first and bounded",
    )
    require(
        [record["id"] for record in scene["evidence"]["latest"]]
        == [
            "ev-phase6-scene-6",
            "ev-phase6-scene-5",
            "ev-phase6-scene-4",
            "ev-phase6-scene-3",
            "ev-phase6-scene-2",
        ],
        "scene latest evidence should be newest-first and bounded",
    )
    require(
        scene["churn"]["diffPressure"] == "low",
        "scene should expose churn pressure",
    )
    require(
        scene["availableActions"]
        == ["index", "retrieve", "distill", "jobs", "update", "propose", "promote"],
        "ready live scene should expose full loaded-state actions",
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
                    "name": "epiphany-phase6-scene-smoke",
                    "title": "Epiphany Phase 6 Scene Smoke",
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

        missing_response = client.send("thread/epiphany/scene", {"threadId": thread_id})
        assert missing_response is not None
        assert_missing_scene(missing_response["scene"])

        update_notification_start = len(client.notifications)
        update = client.send(
            "thread/epiphany/update",
            {"threadId": thread_id, "expectedRevision": 0, "patch": initial_scene_patch()},
        )
        assert update is not None
        require(update["revision"] == 1, "initial scene patch should advance revision to 1")
        update_notification = client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            start_index=update_notification_start,
        )
        require(
            update_notification["params"]["revision"] == 1,
            "initial update notification should expose revision 1",
        )

        for index in range(2, 7):
            update_notification_start = len(client.notifications)
            update = client.send(
                "thread/epiphany/update",
                {
                    "threadId": thread_id,
                    "expectedRevision": index - 1,
                    "patch": {
                        "evidence": [evidence_record(index)],
                        "observations": [observation_record(index)],
                    },
                },
            )
            assert update is not None
            require(
                update["revision"] == index,
                f"record update {index} should advance revision to {index}",
            )
            update_notification = client.wait_for_notification(
                "thread/epiphany/stateUpdated",
                start_index=update_notification_start,
            )
            require(
                update_notification["params"]["revision"] == index,
                f"record update {index} notification should expose revision {index}",
            )

        scene_notification_start = len(client.notifications)
        ready_response = client.send("thread/epiphany/scene", {"threadId": thread_id})
        assert ready_response is not None
        require(ready_response["threadId"] == thread_id, "scene response should echo thread id")
        assert_ready_scene(ready_response["scene"], expected_revision=6)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=scene_notification_start,
        )

        final_read = client.send("thread/read", {"threadId": thread_id, "includeTurns": False})
        assert final_read is not None
        require(
            final_read["thread"]["epiphanyState"]["revision"] == 6,
            "scene should not mutate state revision",
        )

        result = {
            "threadId": thread_id,
            "codexHome": str(codex_home),
            "missingStateStatus": missing_response["scene"]["stateStatus"],
            "missingAvailableActions": missing_response["scene"]["availableActions"],
            "readyStateStatus": ready_response["scene"]["stateStatus"],
            "readySource": ready_response["scene"]["source"],
            "readyRevision": ready_response["scene"]["revision"],
            "readyAvailableActions": ready_response["scene"]["availableActions"],
            "latestObservationIds": [
                record["id"] for record in ready_response["scene"]["observations"]["latest"]
            ],
            "latestEvidenceIds": [
                record["id"] for record in ready_response["scene"]["evidence"]["latest"]
            ],
            "retrievalStatus": ready_response["scene"]["retrieval"]["status"],
            "sceneNotificationCount": client.count_notifications(
                "thread/epiphany/stateUpdated",
                start_index=scene_notification_start,
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
        description="Live-smoke the Phase 6 Epiphany scene reflection surface."
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
