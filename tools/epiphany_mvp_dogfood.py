from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path
from typing import Any

from epiphany_mvp_status import DEFAULT_APP_SERVER
from epiphany_mvp_status import collect_status
from epiphany_mvp_status import render_status
from epiphany_phase5_smoke import AppServerClient
from epiphany_phase5_smoke import ROOT
from epiphany_phase5_smoke import require
from epiphany_phase6_reorient_launch_smoke import BINDING_ID as REORIENT_BINDING_ID
from epiphany_phase6_reorient_launch_smoke import complete_reorient_backend_job
from epiphany_phase6_reorient_smoke import WATCHED_RELATIVE_PATH
from epiphany_phase6_reorient_smoke import prepare_workspace
from epiphany_phase6_reorient_smoke import reorient_patch
from epiphany_phase6_reorient_smoke import wait_for_regather_reorient
from epiphany_phase6_role_smoke import MODELING_BINDING_ID
from epiphany_phase6_role_smoke import VERIFICATION_BINDING_ID
from epiphany_phase6_role_smoke import complete_role_backend_job


DEFAULT_ARTIFACT_DIR = ROOT / ".epiphany-dogfood" / "mvp-loop"


def reset_artifact_dir(path: Path) -> None:
    root = (ROOT / ".epiphany-dogfood").resolve()
    resolved = path.resolve()
    if resolved == root or root not in resolved.parents:
        raise ValueError(f"refusing to delete non-dogfood artifact dir: {path}")
    if path.exists():
        shutil.rmtree(path)
    path.mkdir(parents=True, exist_ok=True)


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def write_text(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value, encoding="utf-8")


def snapshot(
    client: AppServerClient,
    *,
    thread_id: str,
    cwd: Path,
    label: str,
) -> dict[str, Any]:
    status = collect_status(client, thread_id=thread_id, cwd=cwd, ephemeral=True)
    return {"label": label, "status": status, "rendered": render_status(status)}


def launch_role(
    client: AppServerClient,
    thread_id: str,
    *,
    role_id: str,
    expected_revision: int,
) -> dict[str, Any]:
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
        f"{role_id} launch should mutate only job bindings",
    )
    return launch


def run_dogfood(args: argparse.Namespace) -> dict[str, Any]:
    app_server = args.app_server.resolve()
    if not app_server.exists():
        raise FileNotFoundError(f"codex app-server binary not found: {app_server}")

    artifact_dir = args.artifact_dir.resolve()
    reset_artifact_dir(artifact_dir)
    codex_home = artifact_dir / "codex-home"
    workspace = artifact_dir / "workspace"
    transcript_path = artifact_dir / "epiphany-transcript.jsonl"
    stderr_path = artifact_dir / "epiphany-server.stderr.log"
    codex_home.mkdir(parents=True, exist_ok=True)
    watched_file = prepare_workspace(workspace)

    snapshots: list[dict[str, Any]] = []

    with AppServerClient(app_server, codex_home, transcript_path, stderr_path) as client:
        client.send(
            "initialize",
            {
                "clientInfo": {
                    "name": "epiphany-mvp-dogfood",
                    "title": "Epiphany MVP Dogfood",
                    "version": "0.1.0",
                },
                "capabilities": {"experimentalApi": True},
            },
        )
        client.send("initialized", expect_response=False)
        started = client.send("thread/start", {"cwd": str(workspace), "ephemeral": True})
        assert started is not None
        thread_id = started["thread"]["id"]

        snapshots.append(snapshot(client, thread_id=thread_id, cwd=workspace, label="cold-start"))

        update = client.send(
            "thread/epiphany/update",
            {"threadId": thread_id, "expectedRevision": 0, "patch": reorient_patch()},
        )
        assert update is not None
        require(update["revision"] == 1, "checkpoint update should advance revision to 1")
        snapshots.append(snapshot(client, thread_id=thread_id, cwd=workspace, label="checkpoint-ready"))

        modeling_launch = launch_role(
            client,
            thread_id,
            role_id="modeling",
            expected_revision=1,
        )
        modeling_payload = complete_role_backend_job(
            codex_home,
            modeling_launch["job"]["backendJobId"],
            binding_id=MODELING_BINDING_ID,
            role_id="modeling",
            verdict="checkpoint-ready",
        )
        modeling_result = client.send(
            "thread/epiphany/roleResult", {"threadId": thread_id, "roleId": "modeling"}
        )
        assert modeling_result is not None
        require(modeling_result["status"] == "completed", "modeling result should complete")
        snapshots.append(snapshot(client, thread_id=thread_id, cwd=workspace, label="modeling-result"))

        verification_launch = launch_role(
            client,
            thread_id,
            role_id="verification",
            expected_revision=2,
        )
        verification_payload = complete_role_backend_job(
            codex_home,
            verification_launch["job"]["backendJobId"],
            binding_id=VERIFICATION_BINDING_ID,
            role_id="verification",
            verdict="pass",
        )
        verification_result = client.send(
            "thread/epiphany/roleResult",
            {"threadId": thread_id, "roleId": "verification"},
        )
        assert verification_result is not None
        require(verification_result["status"] == "completed", "verification result should complete")
        snapshots.append(snapshot(client, thread_id=thread_id, cwd=workspace, label="verification-result"))

        watched_file.write_text(
            "pub fn reorient_target() -> &'static str {\n    \"after\"\n}\n",
            encoding="utf-8",
        )
        regather = wait_for_regather_reorient(client, thread_id)
        require(regather["decision"]["action"] == "regather", "source drift should force regather")
        snapshots.append(snapshot(client, thread_id=thread_id, cwd=workspace, label="drift-detected"))

        reorient_launch = client.send(
            "thread/epiphany/reorientLaunch",
            {"threadId": thread_id, "expectedRevision": 3, "maxRuntimeSeconds": 90},
        )
        assert reorient_launch is not None
        require(reorient_launch["revision"] == 4, "reorient launch should advance revision to 4")
        reorient_payload = complete_reorient_backend_job(
            codex_home,
            reorient_launch["job"]["backendJobId"],
            mode="regather",
        )
        reorient_result = client.send(
            "thread/epiphany/reorientResult",
            {"threadId": thread_id, "bindingId": REORIENT_BINDING_ID},
        )
        assert reorient_result is not None
        require(reorient_result["status"] == "completed", "reorient result should complete")

        accepted = client.send(
            "thread/epiphany/reorientAccept",
            {
                "threadId": thread_id,
                "expectedRevision": 4,
                "bindingId": REORIENT_BINDING_ID,
                "updateScratch": True,
                "updateInvestigationCheckpoint": True,
            },
        )
        assert accepted is not None
        require(accepted["revision"] == 5, "accepting reorient result should advance revision")
        snapshots.append(snapshot(client, thread_id=thread_id, cwd=workspace, label="accepted-reorient"))

        final_status = collect_status(client, thread_id=thread_id, cwd=workspace, ephemeral=True)
        require(
            final_status["crrc"]["recommendation"]["action"] == "regatherManually",
            "accepted regather findings should not be recommended for acceptance again",
        )
        final_rendered = render_status(final_status)
        final_read = client.send("thread/read", {"threadId": thread_id, "includeTurns": False})
        assert final_read is not None

    summary = {
        "objective": "Dogfood the Epiphany MVP loop on a bounded continuity/role-separation task.",
        "artifactDir": str(artifact_dir),
        "threadId": thread_id,
        "workspace": str(workspace),
        "finalRevision": final_read["thread"]["epiphanyState"]["revision"],
        "snapshots": [item["label"] for item in snapshots],
        "modeling": {
            "revision": modeling_launch["revision"],
            "bindingId": MODELING_BINDING_ID,
            "backendJobId": modeling_launch["job"]["backendJobId"],
            "resultStatus": modeling_result["status"],
            "payload": modeling_payload,
        },
        "verification": {
            "revision": verification_launch["revision"],
            "bindingId": VERIFICATION_BINDING_ID,
            "backendJobId": verification_launch["job"]["backendJobId"],
            "resultStatus": verification_result["status"],
            "payload": verification_payload,
        },
        "crrc": {
            "driftDecision": regather["decision"],
            "launchRevision": reorient_launch["revision"],
            "resultStatus": reorient_result["status"],
            "acceptedRevision": accepted["revision"],
            "payload": reorient_payload,
        },
        "finalRecommendation": final_status["crrc"]["recommendation"],
        "artifactManifest": [
            "epiphany-dogfood-summary.json",
            "epiphany-final-status.json",
            "epiphany-final-status.txt",
            "epiphany-snapshots.json",
            "epiphany-transcript.jsonl",
            "epiphany-server.stderr.log",
            "vanilla-reference-prompt.md",
            "vanilla-reference.md",
            "vanilla-reference.stdout.log",
            "vanilla-reference.stderr.log",
            "comparison.md",
        ],
    }

    write_json(artifact_dir / "epiphany-snapshots.json", snapshots)
    write_json(artifact_dir / "epiphany-final-status.json", final_status)
    write_text(artifact_dir / "epiphany-final-status.txt", final_rendered)
    write_json(artifact_dir / "epiphany-dogfood-summary.json", summary)
    write_json(
        artifact_dir / "artifact-manifest.json",
        {
            "artifactDir": str(artifact_dir),
            "files": summary["artifactManifest"],
            "notes": [
                "transcript contains JSON-RPC request/response audit trail",
                "stderr captures app-server diagnostics",
                "snapshots preserve rendered and raw status at each dogfood checkpoint",
                "vanilla reference and comparison files are added after the control agent run",
            ],
        },
    )
    return summary


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run an auditable Epiphany MVP dogfood pass over state, roles, and CRRC."
    )
    parser.add_argument("--app-server", type=Path, default=DEFAULT_APP_SERVER)
    parser.add_argument("--artifact-dir", type=Path, default=DEFAULT_ARTIFACT_DIR)
    args = parser.parse_args()
    result = run_dogfood(args)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
