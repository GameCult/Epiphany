from __future__ import annotations

import argparse
import json
from pathlib import Path
import time
from typing import Any

from epiphany_phase5_smoke import AppServerClient
from epiphany_phase5_smoke import DEFAULT_APP_SERVER
from epiphany_phase5_smoke import ROOT
from epiphany_phase5_smoke import require
from epiphany_phase5_smoke import reset_smoke_paths
from epiphany_phase6_reorient_smoke import WATCHED_RELATIVE_PATH
from epiphany_phase6_reorient_smoke import prepare_workspace
from epiphany_phase6_reorient_smoke import reorient_patch
from epiphany_phase6_reorient_smoke import wait_for_regather_reorient


DEFAULT_CODEX_HOME = ROOT / ".epiphany-smoke" / "phase6-reorient-launch-codex-home"
DEFAULT_WORKSPACE = ROOT / ".epiphany-smoke" / "phase6-reorient-launch-workspace"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "phase6-reorient-launch-smoke-result.json"
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-smoke" / "phase6-reorient-launch-smoke-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-smoke" / "phase6-reorient-launch-smoke-server.stderr.log"

BINDING_ID = "reorient-worker"
SUBGOAL_ID = "phase6-reorient-smoke"
GRAPH_NODE_ID = "reorient-target"


def job_by_id(jobs: list[dict[str, Any]], job_id: str) -> dict[str, Any]:
    for job in jobs:
        if job["id"] == job_id:
            return job
    raise AssertionError(f"missing job {job_id!r}: {jobs!r}")


def assert_reorient_job(
    job: dict[str, Any],
    *,
    action: str,
) -> None:
    require(job["id"] == BINDING_ID, "reorient launch should use the fixed binding id")
    require(job["kind"] == "specialist", "reorient launch should use a specialist job")
    require(job["ownerRole"] == "epiphany-reorient", "reorient launch should use the fixed owner role")
    require(
        job["authorityScope"] == f"epiphany.reorient.{action}",
        "reorient launch should expose the fixed authority scope",
    )
    require(
        job["scope"] == f"reorient-guided checkpoint {action}",
        "reorient launch should expose the reorientation scope",
    )
    require(
        isinstance(job.get("launcherJobId"), str)
        and job["launcherJobId"].startswith("epiphany-launch-"),
        "reorient launch should expose an Epiphany launcher id",
    )
    require(
        job["backendKind"] == "agentJobs",
        "reorient launch should resolve through the agent_jobs backend",
    )
    require(
        isinstance(job.get("backendJobId"), str) and job["backendJobId"],
        "reorient launch should expose a backend job id",
    )
    require(
        job["runtimeAgentJobId"] == job["backendJobId"],
        "reorient launch should keep runtime and backend job ids aligned",
    )
    require(
        job["linkedSubgoalIds"] == [SUBGOAL_ID],
        "reorient launch should bind the active subgoal",
    )
    require(
        job["linkedGraphNodeIds"] == [GRAPH_NODE_ID],
        "reorient launch should bind the active frontier node",
    )
    require(
        job["status"] in {"pending", "running", "completed", "failed", "cancelled"},
        "reorient launch should bind to a real backend status",
    )


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
                    "name": "epiphany-phase6-reorient-launch-smoke",
                    "title": "Epiphany Phase 6 Reorient Launch Smoke",
                    "version": "0.1.0",
                },
                "capabilities": {"experimentalApi": True},
            },
        )
        client.send("initialized", expect_response=False)
        started = client.send("thread/start", {"cwd": str(workspace), "ephemeral": True})
        assert started is not None
        thread_id = started["thread"]["id"]

        update_notification_start = len(client.notifications)
        update = client.send(
            "thread/epiphany/update",
            {"threadId": thread_id, "expectedRevision": 0, "patch": reorient_patch()},
        )
        assert update is not None
        require(update["revision"] == 1, "reorient launch smoke patch should advance revision to 1")
        client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            start_index=update_notification_start,
        )

        resume_launch_notification_start = len(client.notifications)
        resume_launch = client.send(
            "thread/epiphany/reorientLaunch",
            {
                "threadId": thread_id,
                "expectedRevision": 1,
                "maxRuntimeSeconds": 90,
            },
        )
        assert resume_launch is not None
        require(resume_launch["source"] == "live", "reorient launch should consume live state")
        require(resume_launch["stateStatus"] == "ready", "reorient launch should require ready state")
        require(
            resume_launch["stateRevision"] == 1,
            "reorient launch should report the consumed checkpoint revision",
        )
        require(
            resume_launch["decision"]["action"] == "resume",
            "clean checkpoint should launch a resume-guided specialist",
        )
        require(
            resume_launch["decision"]["reasons"] == ["checkpointReady"],
            "resume launch should preserve the checkpoint-ready verdict",
        )
        require(
            resume_launch["revision"] == 2,
            "resume launch should advance the durable revision",
        )
        require(
            resume_launch["changedFields"] == ["jobBindings"],
            "resume launch should only mutate job bindings",
        )
        require(
            resume_launch["epiphanyState"]["revision"] == 2,
            "resume launch should return the persisted revision",
        )
        assert_reorient_job(resume_launch["job"], action="resume")

        resume_launch_notification = client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            start_index=resume_launch_notification_start,
            timeout=15.0,
        )
        require(
            resume_launch_notification["params"]["source"] == "jobLaunch",
            "reorient launch should emit the bounded jobLaunch state update source",
        )
        require(
            resume_launch_notification["params"]["revision"] == 2,
            "resume launch notification should expose revision 2",
        )
        require(
            resume_launch_notification["params"]["changedFields"] == ["jobBindings"],
            "resume launch notification should identify the job-binding mutation",
        )

        resume_jobs = wait_for_jobs_surface(client, thread_id, require_bound_backend=True)
        require(
            resume_jobs["stateRevision"] == 2,
            "jobs surface should reflect the launched resume revision",
        )
        assert_reorient_job(job_by_id(resume_jobs["jobs"], BINDING_ID), action="resume")

        watched_file.write_text(
            "pub fn reorient_target() -> &'static str {\n    \"after\"\n}\n",
            encoding="utf-8",
        )
        regather_reorient = wait_for_regather_reorient(client, thread_id)
        require(
            regather_reorient["decision"]["action"] == "regather",
            "touched checkpoint path should force a regather verdict",
        )

        interrupt_notification_start = len(client.notifications)
        interrupt = client.send(
            "thread/epiphany/jobInterrupt",
            {
                "threadId": thread_id,
                "expectedRevision": 2,
                "bindingId": BINDING_ID,
                "reason": "Reorient launch smoke is clearing the resume binding before relaunching in regather mode.",
            },
        )
        assert interrupt is not None
        require(interrupt["revision"] == 3, "interrupt should advance the durable revision")
        client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            start_index=interrupt_notification_start,
            timeout=15.0,
        )
        interrupted_jobs = wait_for_jobs_surface(client, thread_id, require_bound_backend=False)
        require(
            interrupted_jobs["stateRevision"] == 3,
            "jobs surface should reflect the interrupted revision",
        )

        regather_launch_notification_start = len(client.notifications)
        regather_launch = client.send(
            "thread/epiphany/reorientLaunch",
            {
                "threadId": thread_id,
                "expectedRevision": 3,
                "maxRuntimeSeconds": 90,
            },
        )
        assert regather_launch is not None
        require(
            regather_launch["stateRevision"] == 3,
            "regather launch should report the consumed interrupted revision",
        )
        require(
            regather_launch["decision"]["action"] == "regather",
            "invalidated checkpoint should launch a regather-guided specialist",
        )
        require(
            regather_launch["decision"]["reasons"] == ["checkpointPathsChanged", "frontierChanged"],
            "regather launch should preserve the watcher invalidation verdict",
        )
        require(
            regather_launch["revision"] == 4,
            "regather launch should advance the durable revision again",
        )
        assert_reorient_job(regather_launch["job"], action="regather")
        normalized_changed_paths = [
            path.replace("\\", "/")
            for path in regather_launch["decision"]["checkpointChangedPaths"]
        ]
        require(
            WATCHED_RELATIVE_PATH.as_posix() in normalized_changed_paths,
            "regather launch should preserve the touched checkpoint path in its decision",
        )

        regather_launch_notification = client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            start_index=regather_launch_notification_start,
            timeout=15.0,
        )
        require(
            regather_launch_notification["params"]["source"] == "jobLaunch",
            "regather launch should emit the bounded jobLaunch state update source",
        )
        require(
            regather_launch_notification["params"]["revision"] == 4,
            "regather launch notification should expose revision 4",
        )

        regather_jobs = wait_for_jobs_surface(client, thread_id, require_bound_backend=True)
        require(
            regather_jobs["stateRevision"] == 4,
            "jobs surface should reflect the relaunched regather revision",
        )
        assert_reorient_job(job_by_id(regather_jobs["jobs"], BINDING_ID), action="regather")

        final_read = client.send("thread/read", {"threadId": thread_id, "includeTurns": False})
        assert final_read is not None
        require(
            final_read["thread"]["epiphanyState"]["revision"] == 4,
            "reorient launch smoke should leave the final revision at 4",
        )

        result = {
            "threadId": thread_id,
            "codexHome": str(codex_home),
            "workspace": str(workspace),
            "resumeAction": resume_launch["decision"]["action"],
            "resumeRevision": resume_launch["revision"],
            "resumeScope": resume_launch["job"]["scope"],
            "interruptRevision": interrupt["revision"],
            "regatherAction": regather_launch["decision"]["action"],
            "regatherRevision": regather_launch["revision"],
            "regatherScope": regather_launch["job"]["scope"],
            "checkpointChangedPaths": regather_launch["decision"]["checkpointChangedPaths"],
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
        description="Live-smoke the bounded Phase 6 reorient-guided worker launch surface."
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
