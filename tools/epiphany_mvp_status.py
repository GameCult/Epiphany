from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
from typing import Any

from epiphany_agent_telemetry import write_transcript_telemetry
from epiphany_phase5_smoke import AppServerClient
from epiphany_phase5_smoke import DEFAULT_APP_SERVER
from epiphany_phase5_smoke import ROOT


DEFAULT_CODEX_HOME = Path(os.environ.get("CODEX_HOME", Path.home() / ".codex"))
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-status" / "epiphany-mvp-status-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-status" / "epiphany-mvp-status-server.stderr.log"
SEALED_DIRECT_THOUGHT_KEYS = {
    "rawResult",
    "turns",
    "items",
    "inputTranscript",
    "activeTranscript",
}


def sealed_direct_thought(key: str, value: Any) -> dict[str, Any]:
    length: int | None = None
    if isinstance(value, (list, str, dict)):
        length = len(value)
    sealed: dict[str, Any] = {
        "sealed": True,
        "key": key,
        "reason": (
            "Operator-safe dogfood views use projected findings and audit "
            "receipts; direct agent transcript/thought payloads stay sealed "
            "unless the user explicitly requests forensic debugging."
        ),
    }
    if length is not None:
        sealed["size"] = length
    return sealed


def sanitize_for_operator(value: Any) -> Any:
    if isinstance(value, dict):
        sanitized: dict[str, Any] = {}
        for key, item in value.items():
            if key in SEALED_DIRECT_THOUGHT_KEYS:
                sanitized[key] = sealed_direct_thought(key, item)
            else:
                sanitized[key] = sanitize_for_operator(item)
        return sanitized
    if isinstance(value, list):
        return [sanitize_for_operator(item) for item in value]
    return value


def maybe(value: Any, fallback: str = "none") -> str:
    if value is None:
        return fallback
    if value == "":
        return fallback
    return str(value)


def list_text(values: list[Any] | None, fallback: str = "none") -> str:
    if not values:
        return fallback
    return ", ".join(str(value) for value in values)


def job_summary(job: dict[str, Any]) -> str:
    progress = ""
    processed = job.get("itemsProcessed")
    total = job.get("itemsTotal")
    if processed is not None or total is not None:
        progress = f" ({maybe(processed, '?')}/{maybe(total, '?')})"
    backend = job.get("backendJobId") or job.get("runtimeAgentJobId")
    backend_text = f", backend {backend}" if backend else ""
    return (
        f"- {job.get('id')}: {job.get('status')} {job.get('kind')}, "
        f"{job.get('ownerRole')} [{job.get('scope')}]{progress}{backend_text}"
    )


def job_by_id(jobs: list[dict[str, Any]], job_id: str) -> dict[str, Any] | None:
    for job in jobs:
        if job.get("id") == job_id:
            return job
    return None


def collect_status(
    client: AppServerClient,
    *,
    thread_id: str | None,
    cwd: Path,
    ephemeral: bool,
) -> dict[str, Any]:
    if thread_id is None:
        started = client.send(
            "thread/start",
            {"cwd": str(cwd), "ephemeral": ephemeral},
        )
        if started is None:
            raise RuntimeError("thread/start returned no response")
        thread_id = started["thread"]["id"]
    elif not ephemeral:
        client.send("thread/resume", {"threadId": thread_id})

    read = client.send("thread/read", {"threadId": thread_id, "includeTurns": False})
    scene = client.send("thread/epiphany/scene", {"threadId": thread_id})
    pressure = client.send("thread/epiphany/pressure", {"threadId": thread_id})
    reorient = client.send("thread/epiphany/reorient", {"threadId": thread_id})
    jobs = client.send("thread/epiphany/jobs", {"threadId": thread_id})
    roles = client.send("thread/epiphany/roles", {"threadId": thread_id})
    role_results = {
        "modeling": client.send(
            "thread/epiphany/roleResult", {"threadId": thread_id, "roleId": "modeling"}
        ),
        "verification": client.send(
            "thread/epiphany/roleResult", {"threadId": thread_id, "roleId": "verification"}
        ),
    }
    reorient_result = client.send("thread/epiphany/reorientResult", {"threadId": thread_id})
    crrc = client.send("thread/epiphany/crrc", {"threadId": thread_id})
    coordinator = client.send("thread/epiphany/coordinator", {"threadId": thread_id})

    status = {
        "threadId": thread_id,
        "read": read,
        "scene": scene,
        "pressure": pressure,
        "reorient": reorient,
        "jobs": jobs,
        "roles": roles,
        "roleResults": role_results,
        "reorientResult": reorient_result,
        "crrc": crrc,
        "coordinator": coordinator,
    }
    return status


def render_status(status: dict[str, Any]) -> str:
    thread_id = status["threadId"]
    scene = status["scene"]["scene"]
    pressure = status["pressure"]["pressure"]
    reorient = status["reorient"]["decision"]
    jobs = status["jobs"]["jobs"]
    result = status["reorientResult"]
    crrc = status["crrc"]
    roles = status["roles"]["roles"]
    role_results = status.get("roleResults") or {}
    recommendation = crrc["recommendation"]
    coordinator = status.get("coordinator") or {}
    checkpoint = scene.get("investigationCheckpoint") or {}

    lines = [
        "Epiphany MVP Status",
        f"Thread: {thread_id}",
        f"State: {scene['stateStatus']} rev {maybe(scene.get('revision'))} ({scene['source']})",
        "",
        "Recommendation",
        f"- action: {recommendation['action']}",
        f"- scene action: {maybe(recommendation.get('recommendedSceneAction'))}",
        f"- reason: {recommendation['reason']}",
        f"- coordinator: {maybe(coordinator.get('action'))} ({maybe(coordinator.get('targetRole'))})",
        "",
        "Continuity",
        (
            f"- pressure: {pressure['level']} ({pressure['status']}, "
            f"prepare={str(pressure['shouldPrepareCompaction']).lower()})"
        ),
        f"- reorient: {reorient['action']} via {list_text(reorient.get('reasons'))}",
        f"- next: {reorient['nextAction']}",
        f"- result: {result['status']} for {result['bindingId']}",
    ]

    finding = result.get("finding")
    if finding:
        lines.extend(
            [
                f"- finding mode: {maybe(finding.get('mode'))}",
                f"- finding next: {maybe(finding.get('nextSafeMove'))}",
            ]
        )

    lines.extend(
        [
            "",
            "Role Lanes",
        ]
    )
    for lane in roles:
        lines.append(
            f"- {lane['title']}: {lane['status']} ({lane['ownerRole']}) - {lane['note']}"
        )

    lines.extend(
        [
            "",
            "Role Findings",
        ]
    )
    for role_id in ("modeling", "verification"):
        role_result = role_results.get(role_id) or {}
        finding = role_result.get("finding")
        label = "Modeling / Checkpoint" if role_id == "modeling" else "Verification / Review"
        lines.append(
            f"- {label}: {maybe(role_result.get('status'))} for "
            f"{maybe(role_result.get('bindingId'))}"
        )
        if finding:
            lines.append(f"  verdict: {maybe(finding.get('verdict'))}")
            lines.append(f"  summary: {maybe(finding.get('summary'))}")
            lines.append(f"  next: {maybe(finding.get('nextSafeMove'))}")
            lines.append(
                "  state patch: "
                + ("available" if isinstance(finding.get("statePatch"), dict) else "none")
            )
            if finding.get("openQuestions"):
                lines.append(f"  open questions: {', '.join(map(str, finding['openQuestions']))}")
            if finding.get("evidenceGaps"):
                lines.append(f"  evidence gaps: {', '.join(map(str, finding['evidenceGaps']))}")
            if finding.get("risks"):
                lines.append(f"  risks: {', '.join(map(str, finding['risks']))}")

    lines.extend(
        [
            "",
            "Checkpoint",
            f"- id: {maybe(checkpoint.get('checkpointId'))}",
            f"- disposition: {maybe(checkpoint.get('disposition'))}",
            f"- focus: {maybe(checkpoint.get('focus'))}",
            f"- next: {maybe(checkpoint.get('nextAction'))}",
            "",
            "Jobs",
        ]
    )
    if jobs:
        lines.extend(job_summary(job) for job in jobs)
    else:
        lines.append("- none")

    lines.extend(
        [
            "",
            "Available Actions",
            f"- {list_text(scene.get('availableActions'))}",
        ]
    )
    return "\n".join(lines) + "\n"


def run(args: argparse.Namespace) -> dict[str, Any]:
    app_server = args.app_server.resolve()
    if not app_server.exists():
        raise FileNotFoundError(f"codex app-server binary not found: {app_server}")

    codex_home = args.codex_home.resolve()
    transcript_path = args.transcript.resolve()
    stderr_path = args.stderr.resolve()
    cwd = args.cwd.resolve()
    codex_home.mkdir(parents=True, exist_ok=True)

    with AppServerClient(app_server, codex_home, transcript_path, stderr_path) as client:
        client.send(
            "initialize",
            {
                "clientInfo": {
                    "name": "epiphany-mvp-status",
                    "title": "Epiphany MVP Status",
                    "version": "0.1.0",
                },
                "capabilities": {"experimentalApi": True},
            },
        )
        client.send("initialized", expect_response=False)
        status = collect_status(
            client,
            thread_id=args.thread_id,
            cwd=cwd,
            ephemeral=args.ephemeral,
        )

    status = sanitize_for_operator(status)
    write_transcript_telemetry(transcript_path, transcript_path.with_suffix(".telemetry.json"))
    if args.result is not None:
        result_path = args.result.resolve()
        result_path.parent.mkdir(parents=True, exist_ok=True)
        result_path.write_text(
            json.dumps(status, indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
        write_transcript_telemetry(transcript_path, result_path.with_suffix(".telemetry.json"))
    return status


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Print a compact Epiphany MVP operator view for one thread."
    )
    parser.add_argument("--app-server", type=Path, default=DEFAULT_APP_SERVER)
    parser.add_argument("--codex-home", type=Path, default=DEFAULT_CODEX_HOME)
    parser.add_argument("--thread-id")
    parser.add_argument("--cwd", type=Path, default=ROOT)
    parser.add_argument("--ephemeral", action=argparse.BooleanOptionalAction, default=True)
    parser.add_argument("--json", action="store_true", help="Print operator-safe collected JSON.")
    parser.add_argument("--result", type=Path)
    parser.add_argument("--transcript", type=Path, default=DEFAULT_TRANSCRIPT)
    parser.add_argument("--stderr", type=Path, default=DEFAULT_STDERR)
    args = parser.parse_args()

    status = run(args)
    if args.json:
        print(json.dumps(status, indent=2, ensure_ascii=False))
    else:
        print(render_status(status), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
