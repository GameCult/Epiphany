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


DEFAULT_CODEX_HOME = ROOT / ".epiphany-smoke" / "phase6-job-control-codex-home"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "phase6-job-control-smoke-result.json"
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-smoke" / "phase6-job-control-smoke-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-smoke" / "phase6-job-control-smoke-server.stderr.log"

BINDING_ID = "specialist-work"
LINKED_SUBGOAL_ID = "phase6-job-control"
LINKED_GRAPH_NODE_ID = "job-control-surface"
OWNER_ROLE = "epiphany-harness"
AUTHORITY_SCOPE = "epiphany.specialist"
JOB_SCOPE = "role-scoped specialist work"


def job_by_id(jobs: list[dict[str, Any]], job_id: str) -> dict[str, Any]:
    for job in jobs:
        if job["id"] == job_id:
            return job
    raise AssertionError(f"missing job {job_id!r}: {jobs!r}")


def assert_bound_specialist_job(job: dict[str, Any]) -> None:
    require(job["id"] == BINDING_ID, "launched job should preserve binding id")
    require(job["kind"] == "specialist", "launched job should remain specialist kind")
    require(job["scope"] == JOB_SCOPE, "launched job should preserve scope")
    require(job["ownerRole"] == OWNER_ROLE, "launched job should preserve owner role")
    require(
        job["authorityScope"] == AUTHORITY_SCOPE,
        "launched job should surface authority scope",
    )
    require(
        isinstance(job.get("launcherJobId"), str)
        and job["launcherJobId"].startswith("epiphany-launch-"),
        "launched job should surface an Epiphany launcher id",
    )
    require(
        job["backendKind"] == "agentJobs",
        "launched job should resolve through the agent_jobs backend",
    )
    require(
        isinstance(job.get("backendJobId"), str) and job["backendJobId"],
        "launched job should expose a backend job id",
    )
    require(
        job["runtimeAgentJobId"] == job["backendJobId"],
        "launched job should keep runtime and backend job ids aligned",
    )
    require(
        job["linkedSubgoalIds"] == [LINKED_SUBGOAL_ID],
        "launched job should preserve linked subgoal ids",
    )
    require(
        job["linkedGraphNodeIds"] == [LINKED_GRAPH_NODE_ID],
        "launched job should preserve linked graph node ids",
    )
    require(
        job["status"] in {"pending", "running", "completed", "failed", "cancelled"},
        "launched job should bind to a real backend status",
    )


def assert_interrupted_specialist_job(job: dict[str, Any]) -> None:
    require(job["id"] == BINDING_ID, "interrupted job should preserve binding id")
    require(job["kind"] == "specialist", "interrupted job should remain specialist kind")
    require(job["scope"] == JOB_SCOPE, "interrupted job should preserve scope")
    require(job["ownerRole"] == OWNER_ROLE, "interrupted job should preserve owner role")
    require(
        job["authorityScope"] == AUTHORITY_SCOPE,
        "interrupted job should preserve authority scope",
    )
    require(job["status"] == "blocked", "interrupt should leave the slot explicitly blocked")
    require(
        job.get("launcherJobId") is None,
        "interrupt should clear launcher identity from the specialist slot",
    )
    require(
        job.get("backendKind") is None and job.get("backendJobId") is None,
        "interrupt should clear backend identity from the specialist slot",
    )
    require(
        job.get("runtimeAgentJobId") is None,
        "interrupt should clear runtime job identity from the specialist slot",
    )
    require(
        isinstance(job.get("blockingReason"), str)
        and "launch explicitly" in job["blockingReason"].lower(),
        "interrupt should leave a bounded relaunch reason behind",
    )


def read_backend_job_metadata(codex_home: Path, job_id: str) -> dict[str, Any]:
    db_paths = sorted(codex_home.glob("state_*.sqlite"))
    require(db_paths, f"state sqlite database should exist under {codex_home}")
    db_path = db_paths[-1]
    conn = sqlite3.connect(f"file:{db_path}?mode=ro", uri=True)
    conn.row_factory = sqlite3.Row
    try:
        job_row = conn.execute(
            """
            SELECT status, last_error IS NOT NULL AS has_error
            FROM agent_jobs
            WHERE id = ?
            """,
            (job_id,),
        ).fetchone()
        require(job_row is not None, f"backend job {job_id!r} should exist")
        item_row = conn.execute(
            """
            SELECT status,
                   assigned_thread_id IS NOT NULL AS has_thread,
                   last_error IS NOT NULL AS has_error,
                   completed_at IS NOT NULL AS has_completed_at
            FROM agent_job_items
            WHERE job_id = ?
            ORDER BY row_index ASC
            LIMIT 1
            """,
            (job_id,),
        ).fetchone()
        require(item_row is not None, f"backend item for job {job_id!r} should exist")
        return {
            "jobStatus": job_row["status"],
            "jobHasError": bool(job_row["has_error"]),
            "itemStatus": item_row["status"],
            "itemHasThread": bool(item_row["has_thread"]),
            "itemHasError": bool(item_row["has_error"]),
            "itemHasCompletedAt": bool(item_row["has_completed_at"]),
        }
    finally:
        conn.close()


def wait_for_jobs_surface(
    client: AppServerClient,
    thread_id: str,
    *,
    require_bound_backend: bool,
    timeout: float = 20.0,
) -> dict[str, Any]:
    deadline = time.time() + timeout
    while time.time() < deadline:
        response = client.send("thread/epiphany/jobs", {"threadId": thread_id})
        assert response is not None
        job = job_by_id(response["jobs"], BINDING_ID)
        if require_bound_backend and job.get("backendJobId"):
            return response
        if not require_bound_backend and job.get("backendJobId") is None:
            return response
        time.sleep(0.2)
    raise TimeoutError("timed out waiting for expected Epiphany jobs surface state")


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
                    "name": "epiphany-phase6-job-control-smoke",
                    "title": "Epiphany Phase 6 Job Control Smoke",
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

        launch_notification_start = len(client.notifications)
        launch = client.send(
            "thread/epiphany/jobLaunch",
            {
                "threadId": thread_id,
                "expectedRevision": 0,
                "bindingId": BINDING_ID,
                "kind": "specialist",
                "scope": JOB_SCOPE,
                "ownerRole": OWNER_ROLE,
                "authorityScope": AUTHORITY_SCOPE,
                "linkedSubgoalIds": [LINKED_SUBGOAL_ID],
                "linkedGraphNodeIds": [LINKED_GRAPH_NODE_ID],
                "instruction": (
                    "Acknowledge the smoke row and report a compact JSON object result for it."
                ),
                "inputJson": {
                    "task": "phase6-job-control-smoke",
                    "goal": "exercise explicit launch and interrupt authority",
                },
                "outputSchemaJson": {
                    "type": "object",
                    "properties": {
                        "ack": {"type": "boolean"},
                        "task": {"type": "string"},
                    },
                    "required": ["ack", "task"],
                    "additionalProperties": True,
                },
                "maxRuntimeSeconds": 60,
            },
        )
        assert launch is not None
        require(launch["revision"] == 1, "launch should create revision 1 from missing state")
        require(
            launch["changedFields"] == ["jobBindings"],
            "launch should only mutate job bindings",
        )
        require(
            launch["epiphanyState"]["revision"] == 1,
            "launch response should return the persisted state revision",
        )
        assert_bound_specialist_job(launch["job"])

        launch_notification = client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            start_index=launch_notification_start,
            timeout=15.0,
        )
        require(
            launch_notification["params"]["threadId"] == thread_id,
            "launch notification should identify the thread",
        )
        require(
            launch_notification["params"]["source"] == "jobLaunch",
            "launch notification should report the launch source",
        )
        require(
            launch_notification["params"]["revision"] == 1,
            "launch notification should expose revision 1",
        )
        require(
            launch_notification["params"]["changedFields"] == ["jobBindings"],
            "launch notification should identify the job binding mutation",
        )

        jobs_updated = client.wait_for_notification(
            "thread/epiphany/jobsUpdated",
            start_index=launch_notification_start,
            timeout=20.0,
        )
        require(
            jobs_updated["params"]["threadId"] == thread_id,
            "jobsUpdated should identify the thread",
        )
        require(
            jobs_updated["params"]["source"] == "runtimeProgress",
            "jobsUpdated should identify runtime progress",
        )
        require(
            jobs_updated["params"]["stateRevision"] == 1,
            "jobsUpdated should preserve the launched state revision",
        )
        updated_job = job_by_id(jobs_updated["params"]["jobs"], BINDING_ID)
        assert_bound_specialist_job(updated_job)

        ready_jobs = wait_for_jobs_surface(
            client,
            thread_id,
            require_bound_backend=True,
        )
        require(
            ready_jobs["source"] == "live",
            "jobs surface should resolve live state for the loaded thread",
        )
        require(
            ready_jobs["stateRevision"] == 1,
            "jobs surface should not invent a new revision after launch",
        )
        ready_job = job_by_id(ready_jobs["jobs"], BINDING_ID)
        assert_bound_specialist_job(ready_job)

        interrupt_notification_start = len(client.notifications)
        interrupt = client.send(
            "thread/epiphany/jobInterrupt",
            {
                "threadId": thread_id,
                "expectedRevision": 1,
                "bindingId": BINDING_ID,
                "reason": "Phase 6 job control smoke requested a clean interrupt.",
            },
        )
        assert interrupt is not None
        require(
            interrupt["revision"] == 2,
            "interrupt should advance the durable state revision",
        )
        require(
            interrupt["changedFields"] == ["jobBindings"],
            "interrupt should only mutate job bindings",
        )
        require(
            interrupt["epiphanyState"]["revision"] == 2,
            "interrupt response should return the new persisted state revision",
        )
        require(
            isinstance(interrupt["cancelRequested"], bool),
            "interrupt should report whether runtime cancellation was accepted",
        )
        require(
            isinstance(interrupt["interruptedThreadIds"], list),
            "interrupt should expose any interrupted worker thread ids",
        )
        assert_interrupted_specialist_job(interrupt["job"])

        interrupt_notification = client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            start_index=interrupt_notification_start,
            timeout=15.0,
        )
        require(
            interrupt_notification["params"]["threadId"] == thread_id,
            "interrupt notification should identify the thread",
        )
        require(
            interrupt_notification["params"]["source"] == "jobInterrupt",
            "interrupt notification should report the interrupt source",
        )
        require(
            interrupt_notification["params"]["revision"] == 2,
            "interrupt notification should expose revision 2",
        )
        require(
            interrupt_notification["params"]["changedFields"] == ["jobBindings"],
            "interrupt notification should identify the job binding mutation",
        )

        interrupted_jobs = wait_for_jobs_surface(
            client,
            thread_id,
            require_bound_backend=False,
        )
        require(
            interrupted_jobs["stateRevision"] == 2,
            "jobs surface should reflect the interrupted state revision",
        )
        interrupted_job = job_by_id(interrupted_jobs["jobs"], BINDING_ID)
        assert_interrupted_specialist_job(interrupted_job)

        backend_metadata = read_backend_job_metadata(
            codex_home, ready_job["backendJobId"]
        )
        require(
            backend_metadata["jobStatus"] == "cancelled",
            "interrupt should mark the backend job cancelled",
        )
        require(
            backend_metadata["itemStatus"] == "failed",
            "interrupt should close the running backend item instead of leaving it running",
        )
        require(
            not backend_metadata["itemHasThread"],
            "interrupt should clear the backend item thread assignment",
        )
        require(
            backend_metadata["itemHasError"],
            "interrupt should leave an item-level interruption reason",
        )
        require(
            backend_metadata["itemHasCompletedAt"],
            "interrupt should set completed_at on the closed backend item",
        )

        final_read = client.send("thread/read", {"threadId": thread_id, "includeTurns": False})
        assert final_read is not None
        require(
            final_read["thread"]["epiphanyState"]["revision"] == 2,
            "thread/read should reflect the interrupted revision",
        )

        result = {
            "threadId": thread_id,
            "codexHome": str(codex_home),
            "launchRevision": launch["revision"],
            "launchStatus": launch["job"]["status"],
            "launchBackendJobId": launch["job"]["backendJobId"],
            "jobsUpdatedStatus": updated_job["status"],
            "jobsUpdatedBackendJobId": updated_job["backendJobId"],
            "readyJobsStatus": ready_job["status"],
            "readyJobsBackendJobId": ready_job["backendJobId"],
            "interruptRevision": interrupt["revision"],
            "interruptCancelRequested": interrupt["cancelRequested"],
            "interruptInterruptedThreadIds": interrupt["interruptedThreadIds"],
            "interruptedStatus": interrupted_job["status"],
            "interruptedBlockingReason": interrupted_job["blockingReason"],
            "backendAfterInterrupt": backend_metadata,
            "jobsUpdatedNotificationCount": client.count_notifications(
                "thread/epiphany/jobsUpdated",
                start_index=launch_notification_start,
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
        description="Live-smoke the Phase 6 Epiphany job launch/interrupt control surface."
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
