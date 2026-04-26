from __future__ import annotations

import argparse
import json
from pathlib import Path
import sqlite3
import time
from typing import Any

from epiphany_phase5_smoke import AppServerClient
from epiphany_phase5_smoke import DEFAULT_APP_SERVER
from epiphany_phase5_smoke import ROOT
from epiphany_phase5_smoke import require
from epiphany_phase5_smoke import reset_smoke_paths


DEFAULT_CODEX_HOME = ROOT / ".epiphany-smoke" / "phase6-jobs-codex-home"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "phase6-jobs-smoke-result.json"
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-smoke" / "phase6-jobs-smoke-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-smoke" / "phase6-jobs-smoke-server.stderr.log"

JOBS_CODE_REF = {
    "path": "app-server/src/codex_message_processor.rs",
    "start_line": 10734,
    "end_line": 10982,
    "symbol": "map_epiphany_jobs",
}


def locate_state_db(codex_home: Path) -> Path:
    candidates = sorted(codex_home.glob("state_*.sqlite"))
    if candidates:
        return candidates[-1]
    fallback = sorted(
        path
        for path in codex_home.glob("*.sqlite")
        if "logs" not in path.name.lower()
    )
    if fallback:
        return fallback[-1]
    raise FileNotFoundError(f"no state db found under {codex_home}")


def seed_agent_job(codex_home: Path, job_id: str, running_thread_id: str) -> None:
    db_path = locate_state_db(codex_home)
    now = int(time.time())
    connection = sqlite3.connect(db_path)
    try:
        with connection:
            connection.execute(
                """
                INSERT INTO agent_jobs (
                    id, name, status, instruction, output_schema_json, input_headers_json,
                    input_csv_path, output_csv_path, auto_export, created_at, updated_at,
                    started_at, completed_at, last_error
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                (
                    job_id,
                    "epiphany-specialist-smoke",
                    "running",
                    "Smoke-seeded runtime job for Epiphany jobs reflection.",
                    None,
                    "[]",
                    str(codex_home / "input.csv"),
                    str(codex_home / "output.csv"),
                    0,
                    now - 120,
                    now,
                    now - 90,
                    None,
                    None,
                ),
            )
            connection.executemany(
                """
                INSERT INTO agent_job_items (
                    job_id, item_id, row_index, source_id, row_json, status, assigned_thread_id,
                    attempt_count, result_json, last_error, created_at, updated_at, completed_at,
                    reported_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                """,
                [
                    (
                        job_id,
                        "item-completed",
                        0,
                        "row-0",
                        '{"row":0}',
                        "completed",
                        None,
                        1,
                        '{"accepted":true}',
                        None,
                        now - 120,
                        now - 30,
                        now - 30,
                        now - 30,
                    ),
                    (
                        job_id,
                        "item-running",
                        1,
                        "row-1",
                        '{"row":1}',
                        "running",
                        running_thread_id,
                        1,
                        None,
                        None,
                        now - 120,
                        now,
                        None,
                        None,
                    ),
                    (
                        job_id,
                        "item-pending",
                        2,
                        "row-2",
                        '{"row":2}',
                        "pending",
                        None,
                        0,
                        None,
                        None,
                        now - 120,
                        now - 10,
                        None,
                        None,
                    ),
                ],
            )
    finally:
        connection.close()


def job_surface_patch(runtime_agent_job_id: str) -> dict[str, Any]:
    return {
        "objective": "Expose read-only Epiphany job/progress reflection without creating a scheduler.",
        "activeSubgoalId": "phase6-jobs-smoke",
        "subgoals": [
            {
                "id": "phase6-jobs-smoke",
                "title": "Live-smoke job reflection",
                "status": "active",
                "summary": "The app-server jobs surface should reflect typed state and retrieval progress.",
            }
        ],
        "invariants": [
            {
                "id": "inv-jobs-read-only",
                "description": "thread/epiphany/jobs must not mutate Epiphany state.",
                "status": "ok",
            },
            {
                "id": "inv-jobs-needs-review",
                "description": "The first job surface should stay reflection-only until live scheduling lands.",
                "status": "needs_review",
            },
        ],
        "graphs": {
            "architecture": {
                "nodes": [
                    {
                        "id": "job-surface",
                        "title": "Job reflection surface",
                        "purpose": "Expose derived progress slots without becoming canonical job state.",
                        "code_refs": [JOBS_CODE_REF],
                    }
                ]
            },
            "dataflow": {"nodes": []},
            "links": [],
        },
        "graphFrontier": {
            "active_node_ids": ["job-surface"],
            "dirty_paths": ["app-server/src/codex_message_processor.rs"],
            "open_question_ids": ["q-live-progress"],
        },
        "jobBindings": [
            {
                "id": "specialist-work",
                "kind": "specialist",
                "scope": "role-scoped specialist work",
                "owner_role": "epiphany-harness",
                "launcher_job_id": "launcher-specialist-smoke",
                "authority_scope": "epiphany.specialist",
                "backend_kind": "agent_jobs",
                "backend_job_id": runtime_agent_job_id,
                "runtime_agent_job_id": runtime_agent_job_id,
                "linked_subgoal_ids": ["phase6-jobs-smoke"],
                "linked_graph_node_ids": ["job-surface"],
                "progress_note": "Bound to a real runtime specialist job.",
            }
        ],
        "churn": {
            "understanding_status": "ready",
            "diff_pressure": "low",
            "graph_freshness": "stale",
            "unexplained_writes": 0,
        },
    }


def job_by_id(jobs: list[dict[str, Any]], job_id: str) -> dict[str, Any]:
    for job in jobs:
        if job["id"] == job_id:
            return job
    raise AssertionError(f"missing job {job_id!r}: {jobs!r}")


def assert_missing_state_jobs(response: dict[str, Any]) -> None:
    require(response["source"] == "live", "jobs should report live source for loaded thread")
    require("stateRevision" not in response, "missing-state jobs should not invent a revision")
    require(len(response["jobs"]) == 4, "jobs surface should expose the four known slots")

    index = job_by_id(response["jobs"], "retrieval-index")
    require(index["kind"] == "indexing", "retrieval slot should be indexing kind")
    require(index["status"] in {"idle", "needed", "running", "unavailable"}, "index status should be typed")

    remap = job_by_id(response["jobs"], "graph-remap")
    require(remap["status"] == "blocked", "missing state should block graph remap")

    verification = job_by_id(response["jobs"], "verification")
    require(verification["status"] == "blocked", "missing state should block verification")

    specialist = job_by_id(response["jobs"], "specialist-work")
    require(specialist["status"] == "unavailable", "specialist slot should report not landed")


def assert_ready_jobs(response: dict[str, Any]) -> None:
    require(response["source"] == "live", "ready jobs should report live source")
    require(response["stateRevision"] == 1, "jobs should preserve state revision identity")
    require(len(response["jobs"]) == 4, "ready jobs should still expose the four known slots")

    index = job_by_id(response["jobs"], "retrieval-index")
    require(index["kind"] == "indexing", "retrieval slot should remain indexing kind")
    require(index["ownerRole"] == "epiphany-core", "retrieval job should name epiphany-core owner")
    require(index["linkedSubgoalIds"] == ["phase6-jobs-smoke"], "index job should link active subgoal")
    require(index["linkedGraphNodeIds"] == ["job-surface"], "index job should link active graph node")

    remap = job_by_id(response["jobs"], "graph-remap")
    require(remap["kind"] == "remap", "graph job should be remap kind")
    require(remap["status"] == "needed", "dirty/stale graph frontier should need remap")
    require(remap["linkedGraphNodeIds"] == ["job-surface"], "remap should expose active graph node")

    verification = job_by_id(response["jobs"], "verification")
    require(verification["kind"] == "verification", "verification slot should be typed")
    require(verification["status"] == "needed", "non-accepting invariant should need verification")
    require(verification["itemsProcessed"] == 1, "verification should count accepting invariants")
    require(verification["itemsTotal"] == 2, "verification should count total invariants")

    specialist = job_by_id(response["jobs"], "specialist-work")
    require(specialist["status"] == "running", "bound specialist job should report real runtime progress")
    require(
        specialist["launcherJobId"] == "launcher-specialist-smoke",
        "specialist launcher job id should surface through jobs reflection",
    )
    require(
        specialist["authorityScope"] == "epiphany.specialist",
        "specialist authority scope should surface through jobs reflection",
    )
    require(
        specialist["backendKind"] == "agentJobs",
        "specialist backend kind should surface through jobs reflection",
    )
    require(
        specialist["backendJobId"] == "job-specialist-smoke",
        "specialist backend job id should surface through jobs reflection",
    )
    require(
        specialist["runtimeAgentJobId"] == "job-specialist-smoke",
        "specialist runtime job id should surface through jobs reflection",
    )
    require(specialist["itemsProcessed"] == 1, "specialist job should count completed+failed items")
    require(specialist["itemsTotal"] == 3, "specialist job should count total runtime items")
    require(
        specialist["activeThreadIds"] == ["worker-thread-specialist"],
        "specialist job should expose active worker thread ids",
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
                    "name": "epiphany-phase6-jobs-smoke",
                    "title": "Epiphany Phase 6 Jobs Smoke",
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
        seed_agent_job(codex_home, "job-specialist-smoke", "worker-thread-specialist")

        missing_notification_start = len(client.notifications)
        missing_response = client.send("thread/epiphany/jobs", {"threadId": thread_id})
        assert missing_response is not None
        require(missing_response["threadId"] == thread_id, "jobs response should echo thread id")
        assert_missing_state_jobs(missing_response)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=missing_notification_start,
        )
        client.require_no_notification(
            "thread/epiphany/jobsUpdated",
            start_index=missing_notification_start,
        )

        update_notification_start = len(client.notifications)
        update = client.send(
            "thread/epiphany/update",
            {
                "threadId": thread_id,
                "expectedRevision": 0,
                "patch": job_surface_patch("job-specialist-smoke"),
            },
        )
        assert update is not None
        require(update["revision"] == 1, "job smoke patch should advance revision to 1")
        client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            start_index=update_notification_start,
        )

        jobs_notification_start = len(client.notifications)
        ready_response = client.send("thread/epiphany/jobs", {"threadId": thread_id})
        assert ready_response is not None
        assert_ready_jobs(ready_response)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=jobs_notification_start,
        )
        client.require_no_notification(
            "thread/epiphany/jobsUpdated",
            start_index=jobs_notification_start,
        )

        final_read = client.send("thread/read", {"threadId": thread_id, "includeTurns": False})
        assert final_read is not None
        require(
            final_read["thread"]["epiphanyState"]["revision"] == 1,
            "jobs reflection should not mutate state revision",
        )

        result = {
            "threadId": thread_id,
            "codexHome": str(codex_home),
            "missingJobStatuses": {
                job["id"]: job["status"] for job in missing_response["jobs"]
            },
            "readyRevision": ready_response["stateRevision"],
            "readyJobStatuses": {job["id"]: job["status"] for job in ready_response["jobs"]},
            "verificationItemsProcessed": job_by_id(
                ready_response["jobs"], "verification"
            )["itemsProcessed"],
            "verificationItemsTotal": job_by_id(
                ready_response["jobs"], "verification"
            )["itemsTotal"],
            "specialistStatus": job_by_id(ready_response["jobs"], "specialist-work")["status"],
            "specialistLauncherJobId": job_by_id(
                ready_response["jobs"], "specialist-work"
            )["launcherJobId"],
            "specialistAuthorityScope": job_by_id(
                ready_response["jobs"], "specialist-work"
            )["authorityScope"],
            "specialistBackendKind": job_by_id(
                ready_response["jobs"], "specialist-work"
            )["backendKind"],
            "specialistBackendJobId": job_by_id(
                ready_response["jobs"], "specialist-work"
            )["backendJobId"],
            "specialistRuntimeAgentJobId": job_by_id(
                ready_response["jobs"], "specialist-work"
            )["runtimeAgentJobId"],
            "specialistActiveThreadIds": job_by_id(
                ready_response["jobs"], "specialist-work"
            )["activeThreadIds"],
            "jobsNotificationCount": client.count_notifications(
                "thread/epiphany/stateUpdated",
                start_index=jobs_notification_start,
            ),
            "jobsUpdatedNotificationCount": client.count_notifications(
                "thread/epiphany/jobsUpdated",
                start_index=jobs_notification_start,
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
        description="Live-smoke the Phase 6 Epiphany job/progress reflection surface."
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
