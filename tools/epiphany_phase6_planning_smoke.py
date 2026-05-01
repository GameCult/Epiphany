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


DEFAULT_CODEX_HOME = ROOT / ".epiphany-smoke" / "phase6-planning-codex-home"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "phase6-planning-smoke-result.json"
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-smoke" / "phase6-planning-smoke-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-smoke" / "phase6-planning-smoke-server.stderr.log"


def planning_patch() -> dict[str, Any]:
    github_source = {
        "kind": "github_issue",
        "provider": "github",
        "repo": "GameCult/EpiphanyAgent",
        "issue_number": 42,
        "url": "https://github.com/GameCult/EpiphanyAgent/issues/42",
        "state": "open",
        "labels": ["planning", "mvp"],
        "assignees": ["Meta"],
        "author": "Meta",
        "created_at": "2026-05-01T10:00:00Z",
        "updated_at": "2026-05-01T10:05:00Z",
        "imported_at": "2026-05-01T10:10:00Z",
    }
    chat_source = {
        "kind": "chat",
        "uri": "codex://threads/planning-smoke",
        "external_id": "turn-planning-smoke",
    }
    return {
        "objective": "Keep planning state reviewable before adopting an implementation objective.",
        "planning": {
            "workspace_root": str(ROOT),
            "captures": [
                {
                    "id": "capture-github-planning",
                    "title": "Import GitHub issue into planning inbox",
                    "body": "Issues should become planning captures before becoming objectives.",
                    "confidence": "observed",
                    "status": "inbox",
                    "speaker": "human",
                    "tags": ["github", "backlog"],
                    "source": github_source,
                    "created_at": "2026-05-01T10:10:00Z",
                    "updated_at": "2026-05-01T10:10:00Z",
                },
                {
                    "id": "capture-chat-planning",
                    "title": "Draft objectives from chat only after bounding them",
                    "body": "User discussion remains planning material until it becomes a firm artifact.",
                    "confidence": "medium",
                    "status": "triaged",
                    "speaker": "human",
                    "tags": ["chat", "objective"],
                    "source": chat_source,
                    "created_at": "2026-05-01T10:12:00Z",
                    "updated_at": "2026-05-01T10:12:00Z",
                },
            ],
            "backlog_items": [
                {
                    "id": "backlog-planning-dashboard",
                    "title": "Expose planning state in the Epiphany dashboard",
                    "kind": "feature",
                    "summary": "Show captures, backlog, roadmap streams, and objective drafts.",
                    "status": "ready",
                    "horizon": "near",
                    "priority": {
                        "value": "high",
                        "rationale": "Planning needs a visible user review loop before automation.",
                        "impact": "user-steering",
                        "urgency": "soon",
                        "confidence": "medium",
                        "effort": "small",
                        "unblocks": ["objective-adoption"],
                    },
                    "confidence": "medium",
                    "product_area": "epiphany-gui",
                    "lane_hints": ["imagination", "hands", "soul"],
                    "acceptance_sketch": ["Dashboard can render typed planning records."],
                    "source_refs": [github_source, chat_source],
                    "updated_at": "2026-05-01T10:15:00Z",
                }
            ],
            "roadmap_streams": [
                {
                    "id": "stream-user-steering",
                    "title": "Human-steered planning",
                    "purpose": "Keep future work visible and bounded before implementation starts.",
                    "status": "active",
                    "item_ids": ["backlog-planning-dashboard"],
                    "near_term_focus": "backlog-planning-dashboard",
                    "review_cadence": "per objective",
                }
            ],
            "objective_drafts": [
                {
                    "id": "draft-planning-dashboard",
                    "title": "Build the planning dashboard slice",
                    "summary": "Render planning records and let the user adopt one bounded objective.",
                    "source_item_ids": ["backlog-planning-dashboard"],
                    "scope": {
                        "includes": ["read-only planning projection", "objective draft review"],
                        "excludes": ["automatic objective adoption"],
                    },
                    "acceptance_criteria": [
                        "Planning records render without changing thread state.",
                        "A draft objective remains review-gated.",
                    ],
                    "evidence_required": ["live smoke", "GUI screenshot"],
                    "lane_plan": {
                        "imagination": "organize planning records into a bounded objective candidate",
                        "eyes": "check prior art and GitHub issue metadata shape",
                        "hands": "wire dashboard controls after the read-only surface is stable",
                        "soul": "verify no planning record silently becomes active objective",
                    },
                    "risks": ["accidental objective adoption"],
                    "review_gates": ["human adoption"],
                    "status": "draft",
                }
            ],
        },
    }


def assert_missing_planning(response: dict[str, Any]) -> None:
    require(response["source"] == "live", "missing planning should report live source")
    require(response["stateStatus"] == "missing", "missing planning should report missing state")
    require("stateRevision" not in response, "missing planning should not invent a revision")
    require(response["planning"] == {}, "missing planning should return empty planning")
    summary = response["summary"]
    require(summary["captureCount"] == 0, "missing planning should not invent captures")
    require(summary["backlogItemCount"] == 0, "missing planning should not invent backlog")
    require("activeObjective" not in summary, "missing planning should not invent objective")


def assert_ready_planning(response: dict[str, Any]) -> None:
    require(response["source"] == "live", "ready planning should report live source")
    require(response["stateStatus"] == "ready", "ready planning should report ready state")
    require(response["stateRevision"] == 1, "ready planning should expose current revision")
    planning = response["planning"]
    require(
        [capture["id"] for capture in planning["captures"]]
        == ["capture-github-planning", "capture-chat-planning"],
        "planning should preserve captures",
    )
    require(
        planning["captures"][0]["source"]["kind"] == "github_issue",
        "planning should preserve GitHub issue source kind",
    )
    require(
        planning["captures"][0]["source"]["issue_number"] == 42,
        "planning should preserve GitHub issue number",
    )
    require(
        [item["id"] for item in planning["backlog_items"]] == ["backlog-planning-dashboard"],
        "planning should preserve backlog items",
    )
    require(
        planning["roadmap_streams"][0]["near_term_focus"] == "backlog-planning-dashboard",
        "planning should preserve roadmap near-term focus",
    )
    require(
        planning["objective_drafts"][0]["status"] == "draft",
        "planning should preserve draft objective state",
    )
    summary = response["summary"]
    require(summary["captureCount"] == 2, "planning summary should count captures")
    require(summary["pendingCaptureCount"] == 1, "planning summary should count inbox captures")
    require(
        summary["githubIssueCaptureCount"] == 1,
        "planning summary should count GitHub captures",
    )
    require(summary["backlogItemCount"] == 1, "planning summary should count backlog")
    require(summary["readyBacklogItemCount"] == 1, "planning summary should count ready backlog")
    require(summary["roadmapStreamCount"] == 1, "planning summary should count roadmap streams")
    require(summary["objectiveDraftCount"] == 1, "planning summary should count drafts")
    require(summary["draftObjectiveCount"] == 1, "planning summary should count draft status")
    require(
        summary["activeObjective"].startswith("Keep planning state reviewable"),
        "planning summary should expose the active thread objective separately",
    )
    require(
        "human explicitly adopts an objective" in summary["note"],
        "planning summary should remind clients that adoption is explicit",
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
                    "name": "epiphany-phase6-planning-smoke",
                    "title": "Epiphany Phase 6 Planning Smoke",
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
        missing_response = client.send("thread/epiphany/planning", {"threadId": thread_id})
        assert missing_response is not None
        require(missing_response["threadId"] == thread_id, "planning response should echo thread id")
        assert_missing_planning(missing_response)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=missing_notification_start,
        )

        update_notification_start = len(client.notifications)
        update = client.send(
            "thread/epiphany/update",
            {"threadId": thread_id, "expectedRevision": 0, "patch": planning_patch()},
        )
        assert update is not None
        require(update["revision"] == 1, "planning patch should advance revision to 1")
        update_notification = client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            start_index=update_notification_start,
        )
        require(
            "planning" in update_notification["params"]["changedFields"],
            "planning update should report the planning changed field",
        )

        planning_notification_start = len(client.notifications)
        ready_response = client.send("thread/epiphany/planning", {"threadId": thread_id})
        assert ready_response is not None
        assert_ready_planning(ready_response)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=planning_notification_start,
        )

        final_read = client.send("thread/read", {"threadId": thread_id, "includeTurns": False})
        assert final_read is not None
        require(
            final_read["thread"]["epiphanyState"]["revision"] == 1,
            "planning reflection should not mutate state revision",
        )

        result = {
            "threadId": thread_id,
            "codexHome": str(codex_home),
            "missingStateStatus": missing_response["stateStatus"],
            "readyStateStatus": ready_response["stateStatus"],
            "readyRevision": ready_response["stateRevision"],
            "captureCount": ready_response["summary"]["captureCount"],
            "githubIssueCaptureCount": ready_response["summary"]["githubIssueCaptureCount"],
            "backlogItemCount": ready_response["summary"]["backlogItemCount"],
            "objectiveDraftCount": ready_response["summary"]["objectiveDraftCount"],
            "planningNotificationCount": client.count_notifications(
                "thread/epiphany/stateUpdated",
                start_index=planning_notification_start,
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
        description="Live-smoke the Phase 6 Epiphany planning substrate surface."
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
