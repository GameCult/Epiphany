from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path
from typing import Any

from epiphany_mvp_status import DEFAULT_APP_SERVER
from epiphany_mvp_status import collect_status
from epiphany_mvp_status import render_status
from epiphany_mvp_status import sanitize_for_operator
from epiphany_mvp_status import write_transcript_telemetry
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
DEFAULT_VANILLA_CODEX_HOME = Path.home() / ".codex"


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


def latest_agent_message(thread_read: dict[str, Any]) -> str:
    turns = thread_read.get("thread", {}).get("turns", [])
    if not isinstance(turns, list):
        return ""
    for turn in reversed(turns):
        items = turn.get("items", []) if isinstance(turn, dict) else []
        if not isinstance(items, list):
            continue
        for item in reversed(items):
            if isinstance(item, dict) and item.get("type") == "agentMessage":
                text = item.get("text")
                if isinstance(text, str):
                    return text
    return ""


def vanilla_reference_prompt(workspace: Path) -> str:
    return f"""You are a vanilla Codex reference agent with no Epiphany typed-state surfaces.

Inspect the bounded smoke workspace at:
{workspace}

Task:
- Decide whether it is safe to resume from this checkpoint:
  - checkpoint id: ix-reorient-1
  - focus: Verify src/reorient_target.rs before broad edits.
  - expected symbol: reorient_target at lines 1-3.
- Return a concise review with:
  - files inspected
  - whether the checkpoint still matches the source
  - the next safe move

Do not edit files.
"""


def write_reference_not_run(artifact_dir: Path, reason: str, prompt: str) -> dict[str, Any]:
    write_text(artifact_dir / "vanilla-reference-prompt.md", prompt)
    write_text(
        artifact_dir / "vanilla-reference.md",
        f"Vanilla reference was not executed.\n\nReason: {reason}\n",
    )
    comparison = (
        "# Dogfood Comparison\n\n"
        "Epiphany artifacts were produced for the MVP loop.\n\n"
        f"Vanilla reference: not executed. {reason}\n"
    )
    write_text(artifact_dir / "comparison.md", comparison)
    return {
        "status": "notRun",
        "reason": reason,
        "promptPath": "vanilla-reference-prompt.md",
        "responsePath": "vanilla-reference.md",
        "comparisonPath": "comparison.md",
    }


def run_vanilla_reference(args: argparse.Namespace, artifact_dir: Path, workspace: Path) -> dict[str, Any]:
    prompt = vanilla_reference_prompt(workspace)
    write_text(artifact_dir / "vanilla-reference-prompt.md", prompt)

    if not args.run_vanilla_reference:
        return write_reference_not_run(
            artifact_dir,
            "Pass --run-vanilla-reference to spend a real vanilla Codex turn.",
            prompt,
        )

    codex_home = args.vanilla_codex_home.resolve()
    transcript_path = artifact_dir / "vanilla-reference-transcript.jsonl"
    stderr_path = artifact_dir / "vanilla-reference.stderr.log"
    try:
        with AppServerClient(args.app_server.resolve(), codex_home, transcript_path, stderr_path) as client:
            client.send(
                "initialize",
                {
                    "clientInfo": {
                        "name": "epiphany-mvp-vanilla-reference",
                        "title": "Epiphany MVP Vanilla Reference",
                        "version": "0.1.0",
                    },
                    "capabilities": {"experimentalApi": True},
                },
            )
            client.send("initialized", expect_response=False)
            started = client.send("thread/start", {"cwd": str(workspace), "ephemeral": False})
            assert started is not None
            thread_id = started["thread"]["id"]
            turn = client.send(
                "turn/start",
                {
                    "threadId": thread_id,
                    "input": [{"type": "text", "text": prompt, "textElements": []}],
                },
            )
            assert turn is not None
            client.wait_for_notification("turn/completed", timeout=args.vanilla_timeout_seconds)
            read = client.send("thread/read", {"threadId": thread_id, "includeTurns": True})
            assert read is not None
            response = latest_agent_message(read)
    except Exception as exc:
        write_text(artifact_dir / "vanilla-reference.md", f"Vanilla reference failed: {exc}\n")
        write_text(
            artifact_dir / "comparison.md",
            "# Dogfood Comparison\n\n"
            "Epiphany artifacts were produced for the MVP loop.\n\n"
            f"Vanilla reference failed: {exc}\n",
        )
        return {
            "status": "failed",
            "error": str(exc),
            "promptPath": "vanilla-reference-prompt.md",
            "responsePath": "vanilla-reference.md",
            "comparisonPath": "comparison.md",
            "transcriptPath": "vanilla-reference-transcript.jsonl",
            "stderrPath": "vanilla-reference.stderr.log",
        }

    write_text(artifact_dir / "vanilla-reference.md", response)
    write_text(
        artifact_dir / "comparison.md",
        "# Dogfood Comparison\n\n"
        "Epiphany run produced typed state, role-lane findings, CRRC drift detection, reorient acceptance, rendered snapshots, and sealed JSON-RPC transcript artifacts.\n\n"
        "Vanilla reference produced a single untyped review turn over the same bounded workspace.\n\n"
        "Primary product signal: Epiphany makes lane ownership, checkpoint state, drift, and review gates inspectable as structured artifacts. Vanilla Codex can still solve the tiny source question, but it does not produce durable typed role/CRRC state unless the operator asks for it manually.\n",
    )
    return {
        "status": "completed",
        "threadId": thread_id,
        "turnId": turn.get("turnId") or turn.get("turn_id"),
        "promptPath": "vanilla-reference-prompt.md",
        "responsePath": "vanilla-reference.md",
        "comparisonPath": "comparison.md",
        "transcriptPath": "vanilla-reference-transcript.jsonl",
        "stderrPath": "vanilla-reference.stderr.log",
    }


def snapshot(
    client: AppServerClient,
    *,
    thread_id: str,
    cwd: Path,
    label: str,
) -> dict[str, Any]:
    status = collect_status(client, thread_id=thread_id, cwd=cwd, ephemeral=True)
    operator_status = sanitize_for_operator(status)
    return {"label": label, "status": operator_status, "rendered": render_status(operator_status)}


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
    telemetry_path = artifact_dir / "agent-function-telemetry.json"
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
        operator_final_status = sanitize_for_operator(final_status)
        final_rendered = render_status(operator_final_status)
        final_read = client.send("thread/read", {"threadId": thread_id, "includeTurns": False})
        assert final_read is not None

    reference = run_vanilla_reference(args, artifact_dir, workspace)

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
            "payload": sanitize_for_operator(modeling_payload),
        },
        "verification": {
            "revision": verification_launch["revision"],
            "bindingId": VERIFICATION_BINDING_ID,
            "backendJobId": verification_launch["job"]["backendJobId"],
            "resultStatus": verification_result["status"],
            "payload": sanitize_for_operator(verification_payload),
        },
        "crrc": {
            "driftDecision": regather["decision"],
            "launchRevision": reorient_launch["revision"],
            "resultStatus": reorient_result["status"],
            "acceptedRevision": accepted["revision"],
            "payload": sanitize_for_operator(reorient_payload),
        },
        "finalRecommendation": operator_final_status["crrc"]["recommendation"],
        "vanillaReference": reference,
        "artifactManifest": [
            "epiphany-dogfood-summary.json",
            "epiphany-final-status.json",
            "epiphany-final-status.txt",
            "epiphany-snapshots.json",
            "agent-function-telemetry.json",
            "vanilla-reference-prompt.md",
            "vanilla-reference.md",
            "comparison.md",
        ],
        "sealedArtifactManifest": [
            {
                "path": "epiphany-transcript.jsonl",
                "reason": "sealed JSON-RPC audit trail; do not read during normal supervision",
            },
            {
                "path": "epiphany-server.stderr.log",
                "reason": "sealed app-server diagnostics; inspect only for explicit debugging",
            },
        ],
    }
    if reference.get("transcriptPath"):
        summary["sealedArtifactManifest"].append(
            {
                "path": str(reference["transcriptPath"]),
                "reason": "sealed vanilla-reference transcript; inspect only for explicit comparison debugging",
            }
        )
    if reference.get("stderrPath"):
        summary["sealedArtifactManifest"].append(
            {
                "path": str(reference["stderrPath"]),
                "reason": "sealed vanilla-reference diagnostics; inspect only for explicit debugging",
            }
        )

    write_json(artifact_dir / "epiphany-snapshots.json", snapshots)
    write_json(artifact_dir / "epiphany-final-status.json", operator_final_status)
    write_text(artifact_dir / "epiphany-final-status.txt", final_rendered)
    write_json(artifact_dir / "epiphany-dogfood-summary.json", summary)
    write_transcript_telemetry(transcript_path, telemetry_path)
    write_json(
        artifact_dir / "artifact-manifest.json",
        {
            "artifactDir": str(artifact_dir),
            "files": summary["artifactManifest"],
            "sealedFiles": summary["sealedArtifactManifest"],
            "notes": [
                "sealed transcripts contain JSON-RPC request/response audit trails",
                "sealed stderr files capture app-server diagnostics",
                "snapshots preserve rendered and operator-safe status at each dogfood checkpoint",
                "comparison.md states whether the optional vanilla reference actually ran",
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
    parser.add_argument("--run-vanilla-reference", action="store_true")
    parser.add_argument("--vanilla-codex-home", type=Path, default=DEFAULT_VANILLA_CODEX_HOME)
    parser.add_argument("--vanilla-timeout-seconds", type=float, default=240.0)
    args = parser.parse_args()
    result = run_dogfood(args)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
