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


DEFAULT_CODEX_HOME = ROOT / ".epiphany-smoke" / "phase6-pressure-codex-home"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "phase6-pressure-smoke-result.json"
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-smoke" / "phase6-pressure-smoke-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-smoke" / "phase6-pressure-smoke-server.stderr.log"


def assert_unknown_pressure(response: dict[str, Any]) -> None:
    require(response["source"] == "live", "fresh pressure should report live source")
    pressure = response["pressure"]
    require(pressure["status"] == "unknown", "fresh pressure should be unknown")
    require(pressure["level"] == "unknown", "fresh pressure level should be unknown")
    require(pressure["basis"] == "unknown", "fresh pressure basis should be unknown")
    require(
        pressure["shouldPrepareCompaction"] is False,
        "unknown pressure must not recommend compaction prep",
    )
    require(
        "usedTokens" not in pressure,
        "fresh pressure should not invent token usage",
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
                    "name": "epiphany-phase6-pressure-smoke",
                    "title": "Epiphany Phase 6 Pressure Smoke",
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

        notification_start = len(client.notifications)
        response = client.send("thread/epiphany/pressure", {"threadId": thread_id})
        assert response is not None
        require(response["threadId"] == thread_id, "pressure response should echo thread id")
        assert_unknown_pressure(response)
        client.require_no_notification(
            "thread/epiphany/stateUpdated",
            start_index=notification_start,
        )

        final_read = client.send("thread/read", {"threadId": thread_id, "includeTurns": False})
        assert final_read is not None
        require(
            "epiphanyState" not in final_read["thread"],
            "pressure reflection should not create Epiphany state",
        )

        result = {
            "threadId": thread_id,
            "codexHome": str(codex_home),
            "source": response["source"],
            "status": response["pressure"]["status"],
            "level": response["pressure"]["level"],
            "basis": response["pressure"]["basis"],
            "shouldPrepareCompaction": response["pressure"]["shouldPrepareCompaction"],
            "stateUpdatedNotificationCount": client.count_notifications(
                "thread/epiphany/stateUpdated",
                start_index=notification_start,
            ),
            "finalReadHasEpiphanyState": "epiphanyState" in final_read["thread"],
        }

    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(
        json.dumps(result, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    return result


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Live-smoke the Phase 6 Epiphany pressure reflection surface."
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
