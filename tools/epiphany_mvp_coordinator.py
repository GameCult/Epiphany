from __future__ import annotations

import argparse
import json
import os
import shutil
import time
from pathlib import Path
from typing import Any

from epiphany_agent_telemetry import write_transcript_telemetry
from epiphany_mvp_status import DEFAULT_APP_SERVER
from epiphany_mvp_status import collect_status
from epiphany_mvp_status import render_status
from epiphany_mvp_status import sanitize_for_operator
from epiphany_phase5_smoke import AppServerClient
from epiphany_phase5_smoke import ROOT
from epiphany_phase5_smoke import require
from epiphany_phase6_reorient_launch_smoke import BINDING_ID as REORIENT_BINDING_ID
from epiphany_phase6_reorient_launch_smoke import complete_reorient_backend_job
from epiphany_phase6_reorient_smoke import prepare_workspace
from epiphany_phase6_reorient_smoke import reorient_patch
from epiphany_phase6_role_smoke import MODELING_BINDING_ID
from epiphany_phase6_role_smoke import VERIFICATION_BINDING_ID
from epiphany_phase6_role_smoke import complete_role_backend_job


DEFAULT_ARTIFACT_DIR = ROOT / ".epiphany-dogfood" / "coordinator"
DEFAULT_CODEX_HOME = Path(os.environ.get("CODEX_HOME", Path.home() / ".codex"))
TERMINAL_ROLE_STATUSES = {"completed", "failed", "cancelled"}
TERMINAL_REORIENT_STATUSES = {"completed", "failed", "cancelled"}
STOP_ACTIONS = {
    "prepareCheckpoint",
    "reviewReorientResult",
    "regatherManually",
    "reviewModelingResult",
    "reviewVerificationResult",
    "continueImplementation",
}


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


def append_jsonl(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as handle:
        handle.write(json.dumps(value, ensure_ascii=False) + "\n")


def thread_lifecycle_event(kind: str, response: dict[str, Any]) -> dict[str, Any]:
    thread = response.get("thread")
    if not isinstance(thread, dict):
        return {"type": kind}
    return {
        "type": kind,
        "threadId": thread.get("id"),
        "status": thread.get("status"),
        "cwd": thread.get("cwd"),
        "ephemeral": thread.get("ephemeral"),
    }


def state_revision(status: dict[str, Any]) -> int | None:
    state = status.get("read", {}).get("thread", {}).get("epiphanyState")
    if isinstance(state, dict):
        revision = state.get("revision")
        if isinstance(revision, int):
            return revision
    scene_revision = status.get("scene", {}).get("scene", {}).get("revision")
    return scene_revision if isinstance(scene_revision, int) else None


def coordinator_from_status(status: dict[str, Any]) -> dict[str, Any]:
    coordinator = status.get("coordinator")
    if isinstance(coordinator, dict):
        return coordinator
    recommendation = status["crrc"]["recommendation"]
    return {
        "action": recommendation["action"],
        "targetRole": None,
        "recommendedSceneAction": recommendation.get("recommendedSceneAction"),
        "requiresReview": recommendation["action"] in {"acceptReorientResult", "reviewReorientResult"},
        "canAutoRun": recommendation["action"] == "launchReorientWorker",
        "reason": recommendation["reason"],
        "sourceSignals": {},
        "roles": status["roles"]["roles"],
        "note": "local fallback from CRRC recommendation",
    }


def collect_coordinator_status(
    client: AppServerClient,
    *,
    thread_id: str,
    cwd: Path,
    ephemeral: bool,
) -> dict[str, Any]:
    status = collect_status(client, thread_id=thread_id, cwd=cwd, ephemeral=ephemeral)
    status["coordinator"] = client.send(
        "thread/epiphany/coordinator", {"threadId": thread_id}
    )
    return status


def wait_for_role_result(
    client: AppServerClient,
    *,
    thread_id: str,
    role_id: str,
    timeout_seconds: int,
    poll_seconds: float,
) -> dict[str, Any]:
    deadline = time.time() + timeout_seconds
    latest: dict[str, Any] | None = None
    while time.time() < deadline:
        latest = client.send(
            "thread/epiphany/roleResult",
            {"threadId": thread_id, "roleId": role_id},
        )
        assert latest is not None
        if latest.get("status") in TERMINAL_ROLE_STATUSES:
            return latest
        time.sleep(poll_seconds)
    assert latest is not None
    return latest


def wait_for_reorient_result(
    client: AppServerClient,
    *,
    thread_id: str,
    timeout_seconds: int,
    poll_seconds: float,
) -> dict[str, Any]:
    deadline = time.time() + timeout_seconds
    latest: dict[str, Any] | None = None
    while time.time() < deadline:
        latest = client.send(
            "thread/epiphany/reorientResult",
            {"threadId": thread_id, "bindingId": REORIENT_BINDING_ID},
        )
        assert latest is not None
        if latest.get("status") in TERMINAL_REORIENT_STATUSES:
            return latest
        time.sleep(poll_seconds)
    assert latest is not None
    return latest


def wait_for_any_notification(
    client: AppServerClient,
    methods: set[str],
    *,
    start_index: int,
    timeout: float,
) -> dict[str, Any]:
    deadline = time.time() + timeout
    while time.time() < deadline:
        assert client.proc is not None
        if client.proc.poll() is not None:
            ordered = ", ".join(sorted(methods))
            raise RuntimeError(
                f"app-server exited with {client.proc.returncode} before notification {ordered}"
            )
        for msg in client.notifications[start_index:]:
            if msg.get("method") in methods:
                return msg
        time.sleep(0.1)
    ordered = ", ".join(sorted(methods))
    raise TimeoutError(f"timed out waiting for notification {ordered}")


def launch_role(
    client: AppServerClient,
    *,
    thread_id: str,
    role_id: str,
    expected_revision: int | None,
    max_runtime_seconds: int,
) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "threadId": thread_id,
        "roleId": role_id,
        "maxRuntimeSeconds": max_runtime_seconds,
    }
    if expected_revision is not None:
        payload["expectedRevision"] = expected_revision
    launch = client.send("thread/epiphany/roleLaunch", payload)
    assert launch is not None
    return launch


def launch_reorient(
    client: AppServerClient,
    *,
    thread_id: str,
    expected_revision: int | None,
    max_runtime_seconds: int,
) -> dict[str, Any]:
    payload: dict[str, Any] = {
        "threadId": thread_id,
        "maxRuntimeSeconds": max_runtime_seconds,
    }
    if expected_revision is not None:
        payload["expectedRevision"] = expected_revision
    launch = client.send("thread/epiphany/reorientLaunch", payload)
    assert launch is not None
    return launch


def maybe_complete_role_backend(
    args: argparse.Namespace,
    launch: dict[str, Any],
    *,
    role_id: str,
) -> dict[str, Any] | None:
    if not args.test_complete_backend:
        return None
    backend_job_id = launch.get("job", {}).get("backendJobId")
    if not backend_job_id:
        return None
    binding_id = MODELING_BINDING_ID if role_id == "modeling" else VERIFICATION_BINDING_ID
    verdict = "checkpoint-ready" if role_id == "modeling" else "pass"
    return complete_role_backend_job(
        args.codex_home.resolve(),
        backend_job_id,
        binding_id=binding_id,
        role_id=role_id,
        verdict=verdict,
    )


def maybe_complete_reorient_backend(
    args: argparse.Namespace,
    launch: dict[str, Any],
) -> dict[str, Any] | None:
    if not args.test_complete_backend:
        return None
    backend_job_id = launch.get("job", {}).get("backendJobId")
    if not backend_job_id:
        return None
    return complete_reorient_backend_job(
        args.codex_home.resolve(),
        backend_job_id,
        mode="regather",
    )


def run_coordinator(args: argparse.Namespace) -> dict[str, Any]:
    app_server = args.app_server.resolve()
    if not app_server.exists():
        raise FileNotFoundError(f"codex app-server binary not found: {app_server}")

    artifact_dir = args.artifact_dir.resolve()
    reset_artifact_dir(artifact_dir)
    transcript_path = artifact_dir / "epiphany-transcript.jsonl"
    stderr_path = artifact_dir / "epiphany-server.stderr.log"
    telemetry_path = artifact_dir / "agent-function-telemetry.json"
    steps_path = artifact_dir / "coordinator-steps.jsonl"
    cwd = args.cwd.resolve()
    codex_home = args.codex_home.resolve()
    codex_home.mkdir(parents=True, exist_ok=True)

    if args.bootstrap_smoke_state:
        cwd = artifact_dir / "workspace"
        prepare_workspace(cwd)

    steps: list[dict[str, Any]] = []
    startup_events: list[dict[str, Any]] = []
    snapshots: list[str] = []
    final_status: dict[str, Any] | None = None
    final_action: dict[str, Any] | None = None

    with AppServerClient(app_server, codex_home, transcript_path, stderr_path) as client:
        client.send(
            "initialize",
            {
                "clientInfo": {
                    "name": "epiphany-mvp-coordinator",
                    "title": "Epiphany MVP Coordinator",
                    "version": "0.1.0",
                },
                "capabilities": {"experimentalApi": True},
            },
        )
        client.send("initialized", expect_response=False)
        if args.thread_id is None:
            started = client.send(
                "thread/start",
                {"cwd": str(cwd), "ephemeral": args.ephemeral},
            )
            assert started is not None
            thread_id = started["thread"]["id"]
            startup_events.append(thread_lifecycle_event("threadStart", started))
        else:
            thread_id = args.thread_id
            resumed = client.send("thread/resume", {"threadId": thread_id})
            assert resumed is not None
            startup_events.append(thread_lifecycle_event("threadResume", resumed))

        if args.bootstrap_smoke_state:
            update = client.send(
                "thread/epiphany/update",
                {"threadId": thread_id, "expectedRevision": 0, "patch": reorient_patch()},
            )
            assert update is not None
            if args.simulate_source_drift:
                client.send("thread/epiphany/freshness", {"threadId": thread_id})
                watched = cwd / "src" / "reorient_target.rs"
                watched.write_text(
                    "pub fn reorient_target() -> &'static str {\n    \"after\"\n}\n",
                    encoding="utf-8",
                )
                time.sleep(0.5)

        for index in range(args.max_steps):
            status = collect_coordinator_status(
                client,
                thread_id=thread_id,
                cwd=cwd,
                ephemeral=args.ephemeral,
            )
            coordinator = coordinator_from_status(status)
            action = coordinator["action"]
            if args.simulate_high_pressure and index == 0:
                action = "compactRehydrateReorient"
                coordinator = {
                    **coordinator,
                    "action": action,
                    "canAutoRun": True,
                    "requiresReview": False,
                    "reason": "Simulated high pressure requested by smoke test.",
                }

            snapshot_name = f"step-{index:02d}-{action}.txt"
            write_text(artifact_dir / snapshot_name, render_status(sanitize_for_operator(status)))
            snapshots.append(snapshot_name)

            step: dict[str, Any] = {
                "index": index,
                "action": action,
                "coordinator": coordinator,
                "stateRevision": state_revision(status),
                "events": [],
            }
            steps.append(step)
            final_status = status
            final_action = coordinator

            if args.mode == "plan":
                append_jsonl(steps_path, step)
                break
            if action in STOP_ACTIONS and not args.auto_review:
                append_jsonl(steps_path, step)
                break

            revision = state_revision(status)
            if action == "reviewModelingResult":
                result = client.send(
                    "thread/epiphany/roleResult",
                    {"threadId": thread_id, "roleId": "modeling"},
                )
                step["events"].append(
                    {
                        "type": "roleResult",
                        "roleId": "modeling",
                        "result": sanitize_for_operator(result),
                    }
                )
                finding = result.get("finding") if isinstance(result, dict) else None
                if not (
                    args.auto_review
                    and isinstance(finding, dict)
                    and isinstance(finding.get("statePatch"), dict)
                    and revision is not None
                ):
                    final_action = {
                        "action": "reviewModelingResult",
                        "reason": result.get("note") if isinstance(result, dict) else coordinator.get("reason"),
                    }
                    append_jsonl(steps_path, step)
                    break
                accepted = client.send(
                    "thread/epiphany/roleAccept",
                    {
                        "threadId": thread_id,
                        "roleId": "modeling",
                        "expectedRevision": revision,
                    },
                )
                step["events"].append(
                    {
                        "type": "roleAccept",
                        "roleId": "modeling",
                        "accepted": sanitize_for_operator(accepted),
                    }
                )
                final_status = collect_coordinator_status(
                    client,
                    thread_id=thread_id,
                    cwd=cwd,
                    ephemeral=args.ephemeral,
                )
                append_jsonl(steps_path, step)
                continue

            if action == "launchModeling":
                launch = launch_role(
                    client,
                    thread_id=thread_id,
                    role_id="modeling",
                    expected_revision=revision,
                    max_runtime_seconds=args.max_runtime_seconds,
                )
                step["events"].append(
                    {
                        "type": "roleLaunch",
                        "roleId": "modeling",
                        "launch": sanitize_for_operator(launch),
                    }
                )
                completed = maybe_complete_role_backend(args, launch, role_id="modeling")
                if completed is not None:
                    step["events"].append(
                        {"type": "testCompleteBackend", "payload": sanitize_for_operator(completed)}
                    )
                result = wait_for_role_result(
                    client,
                    thread_id=thread_id,
                    role_id="modeling",
                    timeout_seconds=args.timeout_seconds,
                    poll_seconds=args.poll_seconds,
                )
                step["events"].append(
                    {
                        "type": "roleResult",
                        "roleId": "modeling",
                        "result": sanitize_for_operator(result),
                    }
                )
                final_status = collect_coordinator_status(
                    client,
                    thread_id=thread_id,
                    cwd=cwd,
                    ephemeral=args.ephemeral,
                )
                if not args.auto_review:
                    final_action = {"action": "reviewModelingResult", "reason": result.get("note")}
                    append_jsonl(steps_path, step)
                    break
                append_jsonl(steps_path, step)
                continue

            if action == "launchVerification":
                launch = launch_role(
                    client,
                    thread_id=thread_id,
                    role_id="verification",
                    expected_revision=revision,
                    max_runtime_seconds=args.max_runtime_seconds,
                )
                step["events"].append(
                    {
                        "type": "roleLaunch",
                        "roleId": "verification",
                        "launch": sanitize_for_operator(launch),
                    }
                )
                completed = maybe_complete_role_backend(args, launch, role_id="verification")
                if completed is not None:
                    step["events"].append(
                        {"type": "testCompleteBackend", "payload": sanitize_for_operator(completed)}
                    )
                result = wait_for_role_result(
                    client,
                    thread_id=thread_id,
                    role_id="verification",
                    timeout_seconds=args.timeout_seconds,
                    poll_seconds=args.poll_seconds,
                )
                step["events"].append(
                    {
                        "type": "roleResult",
                        "roleId": "verification",
                        "result": sanitize_for_operator(result),
                    }
                )
                final_status = collect_coordinator_status(
                    client,
                    thread_id=thread_id,
                    cwd=cwd,
                    ephemeral=args.ephemeral,
                )
                if not args.auto_review:
                    final_action = {"action": "reviewVerificationResult", "reason": result.get("note")}
                    append_jsonl(steps_path, step)
                    break
                append_jsonl(steps_path, step)
                continue

            if action == "launchReorientWorker":
                launch = launch_reorient(
                    client,
                    thread_id=thread_id,
                    expected_revision=revision,
                    max_runtime_seconds=args.max_runtime_seconds,
                )
                step["events"].append({"type": "reorientLaunch", "launch": sanitize_for_operator(launch)})
                completed = maybe_complete_reorient_backend(args, launch)
                if completed is not None:
                    step["events"].append(
                        {"type": "testCompleteBackend", "payload": sanitize_for_operator(completed)}
                    )
                result = wait_for_reorient_result(
                    client,
                    thread_id=thread_id,
                    timeout_seconds=args.timeout_seconds,
                    poll_seconds=args.poll_seconds,
                )
                step["events"].append({"type": "reorientResult", "result": sanitize_for_operator(result)})
                final_status = collect_coordinator_status(
                    client,
                    thread_id=thread_id,
                    cwd=cwd,
                    ephemeral=args.ephemeral,
                )
                if not args.auto_review:
                    final_action = {"action": "reviewReorientResult", "reason": result.get("note")}
                    append_jsonl(steps_path, step)
                    break
                append_jsonl(steps_path, step)
                continue

            if action == "compactRehydrateReorient":
                if args.dry_compact:
                    step["events"].append({"type": "dryCompact", "threadId": thread_id})
                    append_jsonl(steps_path, step)
                    continue
                notification_start_index = len(client.notifications)
                compact = client.send("thread/compact/start", {"threadId": thread_id})
                step["events"].append({"type": "compactStart", "response": sanitize_for_operator(compact)})
                try:
                    notification = wait_for_any_notification(
                        client,
                        {"thread/compacted", "turn/completed"},
                        start_index=notification_start_index,
                        timeout=args.timeout_seconds,
                    )
                    step["events"].append(
                        {"type": "compacted", "notification": sanitize_for_operator(notification)}
                    )
                except TimeoutError as exc:
                    step["events"].append({"type": "compactWaitTimeout", "error": str(exc)})
                    final_action = {
                        "action": "reviewReorientResult",
                        "reason": "Compaction did not finish before timeout.",
                    }
                    append_jsonl(steps_path, step)
                    break
                resumed = client.send("thread/resume", {"threadId": thread_id})
                step["events"].append({"type": "resume", "response": sanitize_for_operator(resumed)})
                post_compact_status = collect_coordinator_status(
                    client,
                    thread_id=thread_id,
                    cwd=cwd,
                    ephemeral=args.ephemeral,
                )
                post_compact_crrc = post_compact_status.get("crrc", {}).get("recommendation", {})
                step["events"].append(
                    {"type": "postCompactCrrc", "recommendation": sanitize_for_operator(post_compact_crrc)}
                )
                final_status = post_compact_status
                if post_compact_crrc.get("action") == "launchReorientWorker":
                    launch = launch_reorient(
                        client,
                        thread_id=thread_id,
                        expected_revision=state_revision(post_compact_status),
                        max_runtime_seconds=args.max_runtime_seconds,
                    )
                    step["events"].append(
                        {"type": "reorientLaunch", "launch": sanitize_for_operator(launch)}
                    )
                    completed = maybe_complete_reorient_backend(args, launch)
                    if completed is not None:
                        step["events"].append(
                            {
                                "type": "testCompleteBackend",
                                "payload": sanitize_for_operator(completed),
                            }
                        )
                    result = wait_for_reorient_result(
                        client,
                        thread_id=thread_id,
                        timeout_seconds=args.timeout_seconds,
                        poll_seconds=args.poll_seconds,
                    )
                    step["events"].append(
                        {"type": "reorientResult", "result": sanitize_for_operator(result)}
                    )
                    final_status = collect_coordinator_status(
                        client,
                        thread_id=thread_id,
                        cwd=cwd,
                        ephemeral=args.ephemeral,
                    )
                    if not args.auto_review:
                        final_action = {
                            "action": "reviewReorientResult",
                            "reason": result.get("note"),
                        }
                        append_jsonl(steps_path, step)
                        break
                append_jsonl(steps_path, step)
                continue

            if action in STOP_ACTIONS:
                append_jsonl(steps_path, step)
                break
            append_jsonl(steps_path, step)

        if final_status is None:
            final_status = collect_coordinator_status(
                client, thread_id=thread_id, cwd=cwd, ephemeral=args.ephemeral
            )
        operator_final_status = sanitize_for_operator(final_status)
        final_rendered = render_status(operator_final_status)

    operator_steps = sanitize_for_operator(steps)
    operator_final_action = sanitize_for_operator(final_action)
    summary = {
        "objective": "Coordinate the Epiphany MVP lanes over existing app-server APIs.",
        "artifactDir": str(artifact_dir),
        "codexHome": str(codex_home),
        "workspace": str(cwd),
        "threadId": final_status["threadId"] if final_status else args.thread_id,
        "mode": args.mode,
        "startupEvents": sanitize_for_operator(startup_events),
        "steps": operator_steps,
        "snapshots": snapshots,
        "finalAction": operator_final_action,
        "finalStatus": operator_final_status,
        "artifactManifest": [
            "coordinator-summary.json",
            "coordinator-steps.jsonl",
            "coordinator-final-status.json",
            "coordinator-final-status.txt",
            "coordinator-final-action.txt",
            "agent-function-telemetry.json",
            *snapshots,
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
    write_json(artifact_dir / "coordinator-summary.json", summary)
    write_json(artifact_dir / "coordinator-final-status.json", operator_final_status)
    write_text(artifact_dir / "coordinator-final-status.txt", final_rendered)
    write_text(
        artifact_dir / "coordinator-final-action.txt",
        json.dumps(operator_final_action, indent=2, ensure_ascii=False) + "\n",
    )
    write_transcript_telemetry(transcript_path, telemetry_path)
    return summary


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run an auditable fixed-lane Epiphany MVP coordinator."
    )
    parser.add_argument("--app-server", type=Path, default=DEFAULT_APP_SERVER)
    parser.add_argument("--thread-id")
    parser.add_argument("--cwd", type=Path, default=ROOT)
    parser.add_argument("--codex-home", type=Path, default=DEFAULT_CODEX_HOME)
    parser.add_argument("--artifact-dir", type=Path, default=DEFAULT_ARTIFACT_DIR)
    parser.add_argument("--mode", choices=["plan", "run"], default="plan")
    parser.add_argument("--max-steps", type=int, default=4)
    parser.add_argument("--poll-seconds", type=float, default=5.0)
    parser.add_argument("--timeout-seconds", type=int, default=240)
    parser.add_argument("--max-runtime-seconds", type=int, default=180)
    parser.add_argument("--ephemeral", action=argparse.BooleanOptionalAction, default=True)
    parser.add_argument("--auto-review", action="store_true")
    parser.add_argument("--test-complete-backend", action="store_true")
    parser.add_argument("--bootstrap-smoke-state", action="store_true")
    parser.add_argument("--simulate-high-pressure", action="store_true")
    parser.add_argument("--simulate-source-drift", action="store_true")
    parser.add_argument("--dry-compact", action="store_true")
    args = parser.parse_args()

    result = run_coordinator(args)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
