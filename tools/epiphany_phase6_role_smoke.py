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
from epiphany_phase6_reorient_launch_smoke import GRAPH_NODE_ID
from epiphany_phase6_reorient_launch_smoke import SUBGOAL_ID
from epiphany_phase6_reorient_launch_smoke import job_by_id
from epiphany_phase6_reorient_launch_smoke import locate_state_db
from epiphany_phase6_reorient_smoke import WATCHED_RELATIVE_PATH
from epiphany_phase6_reorient_smoke import prepare_workspace
from epiphany_phase6_reorient_smoke import reorient_patch


DEFAULT_CODEX_HOME = ROOT / ".epiphany-smoke" / "phase6-role-codex-home"
DEFAULT_WORKSPACE = ROOT / ".epiphany-smoke" / "phase6-role-workspace"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "phase6-role-smoke-result.json"
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-smoke" / "phase6-role-smoke-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-smoke" / "phase6-role-smoke-server.stderr.log"

MODELING_BINDING_ID = "modeling-checkpoint-worker"
VERIFICATION_BINDING_ID = "verification-review-worker"


def complete_role_backend_job(
    codex_home: Path,
    backend_job_id: str,
    *,
    binding_id: str,
    role_id: str,
    verdict: str,
    evidence_ids: list[str] | None = None,
) -> dict[str, Any]:
    result = {
        "roleId": role_id,
        "verdict": verdict,
        "summary": f"{role_id} role result was read back from agent_jobs.",
        "nextSafeMove": f"Review the {role_id} finding before mutating Epiphany state.",
        "checkpointSummary": "Checkpoint remains bounded to the smoke slice.",
        "scratchSummary": "Scratch remains review-only until explicitly updated.",
        "filesInspected": [WATCHED_RELATIVE_PATH.as_posix()],
        "frontierNodeIds": [GRAPH_NODE_ID],
        "evidenceIds": evidence_ids or ["ev-checkpoint"],
    }
    if role_id == "modeling":
        patch = reorient_patch()
        graphs = patch["graphs"]
        graphs["architecture"]["nodes"].append(
            {
                "id": "accepted-modeling-node",
                "title": "Accepted modeling node",
                "purpose": "Proves a modeling role finding can grow the durable graph after review.",
                "code_refs": [
                    {
                        "path": WATCHED_RELATIVE_PATH.as_posix(),
                        "start_line": 1,
                        "end_line": 3,
                        "symbol": "reorient_target",
                    }
                ],
            }
        )
        result["statePatch"] = {
            "graphs": graphs,
            "graphFrontier": {
                "active_node_ids": [GRAPH_NODE_ID, "accepted-modeling-node"],
                "dirty_paths": [],
            },
            "graphCheckpoint": {
                "checkpoint_id": "ck-modeling-accepted",
                "graph_revision": 2,
                "summary": "Modeling role accepted a graph-growth checkpoint.",
                "frontier_node_ids": [GRAPH_NODE_ID, "accepted-modeling-node"],
            },
            "scratch": {
                "summary": "Modeling role found a graph growth candidate.",
                "current_focus": "Review-gated modeling acceptance smoke.",
                "next_action": "Verify the accepted graph node is visible in durable Epiphany state.",
            },
        }
    now = int(time.time())
    db_path = locate_state_db(codex_home)
    connection = sqlite3.connect(db_path)
    try:
        with connection:
            connection.execute(
                """
                UPDATE agent_jobs
                SET status = 'completed', updated_at = ?, completed_at = ?, last_error = NULL
                WHERE id = ?
                """,
                (now, now, backend_job_id),
            )
            connection.execute(
                """
                UPDATE agent_job_items
                SET status = 'completed',
                    result_json = ?,
                    reported_at = ?,
                    completed_at = ?,
                    updated_at = ?,
                    last_error = NULL,
                    assigned_thread_id = NULL
                WHERE job_id = ? AND item_id = ?
                """,
                (json.dumps(result), now, now, now, backend_job_id, binding_id),
            )
    finally:
        connection.close()
    return result


def assert_role_job(
    job: dict[str, Any],
    *,
    binding_id: str,
    owner_role: str,
    authority_scope: str,
    scope: str,
    expected_graph_node_ids: list[str] | None = None,
) -> None:
    require(job["id"] == binding_id, f"{binding_id} should use the fixed binding id")
    require(job["kind"] == "specialist", f"{binding_id} should use a specialist job")
    require(job["ownerRole"] == owner_role, f"{binding_id} should use the fixed owner role")
    require(job["authorityScope"] == authority_scope, f"{binding_id} should expose authority")
    require(job["scope"] == scope, f"{binding_id} should expose the fixed scope")
    require(job["backendKind"] == "agentJobs", f"{binding_id} should use agent_jobs")
    require(
        isinstance(job.get("backendJobId"), str) and job["backendJobId"],
        f"{binding_id} should expose a backend job id",
    )
    require(
        job["runtimeAgentJobId"] == job["backendJobId"],
        f"{binding_id} should align runtime and backend ids",
    )
    require(job["linkedSubgoalIds"] == [SUBGOAL_ID], f"{binding_id} should bind subgoal")
    expected_graph_node_ids = expected_graph_node_ids or [GRAPH_NODE_ID]
    require(
        job["linkedGraphNodeIds"] == expected_graph_node_ids,
        f"{binding_id} should bind frontier",
    )


def launch_role(
    client: AppServerClient,
    thread_id: str,
    *,
    role_id: str,
    expected_revision: int,
) -> dict[str, Any]:
    notification_start = len(client.notifications)
    launch = client.send(
        "thread/epiphany/roleLaunch",
        {
            "threadId": thread_id,
            "roleId": role_id,
            "expectedRevision": expected_revision,
            "maxRuntimeSeconds": 90,
        },
    )
    assert launch is not None
    require(launch["roleId"] == role_id, f"{role_id} launch should echo role id")
    require(
        launch["changedFields"] == ["jobBindings"],
        f"{role_id} launch should only mutate job bindings",
    )
    notification = client.wait_for_notification(
        "thread/epiphany/stateUpdated",
        start_index=notification_start,
        timeout=15.0,
    )
    require(
        notification["params"]["source"] == "jobLaunch",
        f"{role_id} launch should emit jobLaunch state update source",
    )
    require(
        notification["params"]["revision"] == launch["revision"],
        f"{role_id} launch notification should expose the launch revision",
    )
    return launch


def assert_role_result(
    result: dict[str, Any],
    *,
    role_id: str,
    binding_id: str,
    payload: dict[str, Any],
    expected_revision: int,
) -> None:
    require(result["source"] == "live", f"{role_id} result should read live state")
    require(result["roleId"] == role_id, f"{role_id} result should echo role id")
    require(result["stateStatus"] == "ready", f"{role_id} result should see ready state")
    require(
        result["stateRevision"] == expected_revision,
        f"{role_id} result should preserve the read revision",
    )
    require(result["bindingId"] == binding_id, f"{role_id} result should use fixed binding")
    require(result["status"] == "completed", f"{role_id} result should be completed")
    require(
        result["finding"]["rawResult"] == payload,
        f"{role_id} result should expose raw worker output",
    )
    require(
        result["finding"]["nextSafeMove"]
        == f"Review the {role_id} finding before mutating Epiphany state.",
        f"{role_id} result should project next safe move",
    )


def run_smoke(args: argparse.Namespace) -> dict[str, Any]:
    app_server = args.app_server.resolve()
    if not app_server.exists():
        raise FileNotFoundError(f"codex app-server binary not found: {app_server}")

    codex_home = args.codex_home.resolve()
    workspace = args.workspace.resolve()
    prepare_workspace(workspace)
    result_path = args.result.resolve()
    transcript_path = args.transcript.resolve()
    stderr_path = args.stderr.resolve()
    reset_smoke_paths(codex_home, result_path, transcript_path, stderr_path)

    with AppServerClient(app_server, codex_home, transcript_path, stderr_path) as client:
        client.send(
            "initialize",
            {
                "clientInfo": {
                    "name": "epiphany-phase6-role-smoke",
                    "title": "Epiphany Phase 6 Role Smoke",
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
        require(update["revision"] == 1, "role smoke patch should advance revision to 1")
        client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            start_index=update_notification_start,
        )

        modeling_launch = launch_role(
            client,
            thread_id,
            role_id="modeling",
            expected_revision=1,
        )
        require(modeling_launch["revision"] == 2, "modeling launch should advance revision")
        assert_role_job(
            modeling_launch["job"],
            binding_id=MODELING_BINDING_ID,
            owner_role="epiphany-modeler",
            authority_scope="epiphany.role.modeling",
            scope="role-scoped modeling/checkpoint maintenance",
        )
        modeling_payload = complete_role_backend_job(
            codex_home,
            modeling_launch["job"]["backendJobId"],
            binding_id=MODELING_BINDING_ID,
            role_id="modeling",
            verdict="checkpoint-ready",
        )
        modeling_result_start = len(client.notifications)
        modeling_result = client.send(
            "thread/epiphany/roleResult",
            {"threadId": thread_id, "roleId": "modeling"},
        )
        assert modeling_result is not None
        assert_role_result(
            modeling_result,
            role_id="modeling",
            binding_id=MODELING_BINDING_ID,
            payload=modeling_payload,
            expected_revision=2,
        )
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=modeling_result_start,
        )

        modeling_roles = client.send("thread/epiphany/roles", {"threadId": thread_id})
        assert modeling_roles is not None
        modeling_lane = next(
            role for role in modeling_roles["roles"] if role["id"] == "modeling"
        )
        require(
            modeling_lane["recommendedAction"] == "roleResult",
            "roles should point modeling at roleResult while the binding exists",
        )
        assert_role_job(
            job_by_id(modeling_lane["jobs"], MODELING_BINDING_ID),
            binding_id=MODELING_BINDING_ID,
            owner_role="epiphany-modeler",
            authority_scope="epiphany.role.modeling",
            scope="role-scoped modeling/checkpoint maintenance",
        )

        accept_start = len(client.notifications)
        modeling_accept = client.send(
            "thread/epiphany/roleAccept",
            {
                "threadId": thread_id,
                "roleId": "modeling",
                "expectedRevision": 2,
            },
        )
        assert modeling_accept is not None
        require(
            modeling_accept["changedFields"]
            == [
                "graphs",
                "graphFrontier",
                "graphCheckpoint",
                "scratch",
                "observations",
                "evidence",
            ],
            "modeling accept should apply only modeling patch fields plus audit records",
        )
        require(
            modeling_accept["appliedPatch"]["graphs"]["architecture"]["nodes"][-1]["id"]
            == "accepted-modeling-node",
            "modeling accept should apply the reviewed graph patch",
        )
        accept_notification = client.wait_for_notification(
            "thread/epiphany/stateUpdated",
            start_index=accept_start,
            timeout=15.0,
        )
        require(
            accept_notification["params"]["source"] == "roleAccept",
            "modeling accept should emit roleAccept state update source",
        )
        require(
            accept_notification["params"]["revision"] == 3,
            "modeling accept should advance durable revision",
        )

        verification_launch = launch_role(
            client,
            thread_id,
            role_id="verification",
            expected_revision=3,
        )
        require(
            verification_launch["revision"] == 4,
            "verification launch should advance revision",
        )
        assert_role_job(
            verification_launch["job"],
            binding_id=VERIFICATION_BINDING_ID,
            owner_role="epiphany-verifier",
            authority_scope="epiphany.role.verification",
            scope="role-scoped verification/review",
            expected_graph_node_ids=[GRAPH_NODE_ID, "accepted-modeling-node"],
        )
        verification_payload = complete_role_backend_job(
            codex_home,
            verification_launch["job"]["backendJobId"],
            binding_id=VERIFICATION_BINDING_ID,
            role_id="verification",
            verdict="pass",
        )
        verification_result_start = len(client.notifications)
        verification_result = client.send(
            "thread/epiphany/roleResult",
            {"threadId": thread_id, "roleId": "verification"},
        )
        assert verification_result is not None
        assert_role_result(
            verification_result,
            role_id="verification",
            binding_id=VERIFICATION_BINDING_ID,
            payload=verification_payload,
            expected_revision=4,
        )
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=verification_result_start,
        )

        final_read = client.send("thread/read", {"threadId": thread_id, "includeTurns": False})
        assert final_read is not None
        require(
            final_read["thread"]["epiphanyState"]["revision"] == 4,
            "role result read-back should not mutate durable state",
        )
        graph_ids = [
            node["id"]
            for node in final_read["thread"]["epiphanyState"]["graphs"]["architecture"]["nodes"]
        ]
        require(
            "accepted-modeling-node" in graph_ids,
            "accepted modeling result should grow the durable graph",
        )

        result = {
            "threadId": thread_id,
            "codexHome": str(codex_home),
            "workspace": str(workspace),
            "modelingRevision": modeling_launch["revision"],
            "modelingResultStatus": modeling_result["status"],
            "modelingNextSafeMove": modeling_result["finding"]["nextSafeMove"],
            "modelingAcceptRevision": modeling_accept["revision"],
            "modelingAcceptedNode": "accepted-modeling-node",
            "verificationRevision": verification_launch["revision"],
            "verificationResultStatus": verification_result["status"],
            "verificationVerdict": verification_result["finding"]["verdict"],
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
        description="Live-smoke explicit Phase 6 modeling/verification role launch and read-back."
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
