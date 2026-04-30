from __future__ import annotations

import argparse
import json
import os
import shutil
import sys
import time
from pathlib import Path
from typing import Any

from epiphany_mvp_status import DEFAULT_APP_SERVER
from epiphany_mvp_status import collect_status
from epiphany_mvp_status import render_status
from epiphany_mvp_status import sanitize_for_operator
from epiphany_phase5_smoke import AppServerClient
from epiphany_phase5_smoke import ROOT
from epiphany_phase5_smoke import require
from epiphany_phase6_reorient_smoke import prepare_workspace
from epiphany_phase6_reorient_smoke import reorient_patch


DEFAULT_ARTIFACT_DIR = ROOT / ".epiphany-dogfood" / "live-specialist"
DEFAULT_CODEX_HOME = Path(os.environ.get("CODEX_HOME", Path.home() / ".codex"))
DEFAULT_ROLES = ("modeling",)
TERMINAL_STATUSES = {"completed", "failed", "cancelled"}


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


def launch_role(
    client: AppServerClient,
    *,
    thread_id: str,
    role_id: str,
    expected_revision: int,
    max_runtime_seconds: int,
) -> dict[str, Any]:
    launch = client.send(
        "thread/epiphany/roleLaunch",
        {
            "threadId": thread_id,
            "roleId": role_id,
            "expectedRevision": expected_revision,
            "maxRuntimeSeconds": max_runtime_seconds,
        },
    )
    assert launch is not None
    require(launch["roleId"] == role_id, f"{role_id} launch should echo role id")
    require(
        launch["changedFields"] == ["jobBindings"],
        f"{role_id} launch should only mutate job bindings",
    )
    return launch


def wait_for_role_result(
    client: AppServerClient,
    *,
    thread_id: str,
    role_id: str,
    timeout_seconds: int,
    poll_seconds: float,
) -> tuple[dict[str, Any], list[str]]:
    deadline = time.time() + timeout_seconds
    seen_statuses: list[str] = []
    result: dict[str, Any] | None = None
    while time.time() < deadline:
        response = client.send(
            "thread/epiphany/roleResult",
            {"threadId": thread_id, "roleId": role_id},
        )
        assert response is not None
        result = response
        status = response.get("status", "<missing>")
        seen_statuses.append(status)
        if status in TERMINAL_STATUSES:
            return response, seen_statuses
        time.sleep(poll_seconds)

    assert result is not None
    return result, seen_statuses


def run_live_specialist(args: argparse.Namespace) -> dict[str, Any]:
    app_server = args.app_server.resolve()
    if not app_server.exists():
        raise FileNotFoundError(f"codex app-server binary not found: {app_server}")

    codex_home = args.codex_home.resolve()
    if not codex_home.exists():
        raise FileNotFoundError(f"codex home not found: {codex_home}")

    artifact_dir = args.artifact_dir.resolve()
    reset_artifact_dir(artifact_dir)
    workspace = artifact_dir / "workspace"
    transcript_path = artifact_dir / "epiphany-transcript.jsonl"
    stderr_path = artifact_dir / "epiphany-server.stderr.log"
    prepare_workspace(workspace)

    role_results: list[dict[str, Any]] = []

    with AppServerClient(app_server, codex_home, transcript_path, stderr_path) as client:
        client.send(
            "initialize",
            {
                "clientInfo": {
                    "name": "epiphany-mvp-live-specialist",
                    "title": "Epiphany MVP Live Specialist",
                    "version": "0.1.0",
                },
                "capabilities": {"experimentalApi": True},
            },
        )
        client.send("initialized", expect_response=False)
        started = client.send("thread/start", {"cwd": str(workspace), "ephemeral": True})
        assert started is not None
        thread_id = started["thread"]["id"]

        update = client.send(
            "thread/epiphany/update",
            {"threadId": thread_id, "expectedRevision": 0, "patch": reorient_patch()},
        )
        assert update is not None
        revision = update["revision"]

        for role_id in args.roles:
            launch = launch_role(
                client,
                thread_id=thread_id,
                role_id=role_id,
                expected_revision=revision,
                max_runtime_seconds=args.max_runtime_seconds,
            )
            revision = launch["revision"]
            result, seen_statuses = wait_for_role_result(
                client,
                thread_id=thread_id,
                role_id=role_id,
                timeout_seconds=args.timeout_seconds,
                poll_seconds=args.poll_seconds,
            )
            role_results.append(
                {
                    "roleId": role_id,
                    "launch": sanitize_for_operator(launch),
                    "seenStatuses": seen_statuses,
                    "result": sanitize_for_operator(result),
                }
            )

        final_status = collect_status(client, thread_id=thread_id, cwd=workspace, ephemeral=True)
        operator_final_status = sanitize_for_operator(final_status)
        final_rendered = render_status(operator_final_status)

    summary = {
        "objective": "Run a real Epiphany role specialist through roleLaunch, agent_jobs, report_agent_job_result, and roleResult.",
        "artifactDir": str(artifact_dir),
        "codexHome": str(codex_home),
        "threadId": thread_id,
        "workspace": str(workspace),
        "roles": role_results,
        "finalStatus": operator_final_status,
        "artifactManifest": [
            "live-specialist-summary.json",
            "epiphany-final-status.json",
            "epiphany-final-status.txt",
        ],
        "sealedArtifactManifest": [
            {
                "path": "epiphany-transcript.jsonl",
                "reason": "sealed worker transcript; do not read during normal supervision",
            },
            {
                "path": "epiphany-server.stderr.log",
                "reason": "sealed app-server diagnostics; inspect only for explicit debugging",
            },
        ],
    }

    write_json(artifact_dir / "live-specialist-summary.json", summary)
    write_json(artifact_dir / "epiphany-final-status.json", operator_final_status)
    write_text(artifact_dir / "epiphany-final-status.txt", final_rendered)
    return summary


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run an auditable live Epiphany specialist through the real agent_jobs worker path."
    )
    parser.add_argument("--app-server", type=Path, default=DEFAULT_APP_SERVER)
    parser.add_argument("--artifact-dir", type=Path, default=DEFAULT_ARTIFACT_DIR)
    parser.add_argument("--codex-home", type=Path, default=DEFAULT_CODEX_HOME)
    parser.add_argument("--roles", nargs="+", choices=["modeling", "verification"], default=list(DEFAULT_ROLES))
    parser.add_argument("--max-runtime-seconds", type=int, default=180)
    parser.add_argument("--timeout-seconds", type=int, default=240)
    parser.add_argument("--poll-seconds", type=float, default=5.0)
    args = parser.parse_args()
    result = run_live_specialist(args)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
