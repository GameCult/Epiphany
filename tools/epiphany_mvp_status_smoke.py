from __future__ import annotations

import argparse
import json
from pathlib import Path

from epiphany_mvp_status import DEFAULT_APP_SERVER
from epiphany_mvp_status import ROOT
from epiphany_mvp_status import render_status
from epiphany_mvp_status import run
from epiphany_phase5_smoke import require
from epiphany_phase5_smoke import reset_smoke_paths


DEFAULT_CODEX_HOME = ROOT / ".epiphany-smoke" / "mvp-status-codex-home"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "mvp-status-smoke-result.json"
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-smoke" / "mvp-status-smoke-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-smoke" / "mvp-status-smoke-server.stderr.log"


def run_smoke(args: argparse.Namespace) -> dict[str, object]:
    app_server = args.app_server.resolve()
    if not app_server.exists():
        raise FileNotFoundError(f"codex app-server binary not found: {app_server}")

    codex_home = args.codex_home.resolve()
    result_path = args.result.resolve()
    transcript_path = args.transcript.resolve()
    stderr_path = args.stderr.resolve()
    reset_smoke_paths(codex_home, result_path, transcript_path, stderr_path)

    status = run(
        argparse.Namespace(
            app_server=app_server,
            codex_home=codex_home,
            thread_id=None,
            cwd=ROOT,
            ephemeral=True,
            result=None,
            transcript=transcript_path,
            stderr=stderr_path,
        )
    )
    rendered = render_status(status)

    require(
        status["scene"]["scene"]["stateStatus"] == "missing",
        "fresh status smoke should honestly report missing Epiphany state",
    )
    require(
        status["crrc"]["recommendation"]["action"] == "regatherManually",
        "fresh status smoke should recommend manual regather without state",
    )
    require(
        "crrc" in status["scene"]["scene"]["availableActions"],
        "status view should expose the CRRC action in the scene",
    )
    require(
        "roles" in status["scene"]["scene"]["availableActions"],
        "status view should expose the role ownership action in the scene",
    )
    require(
        "Epiphany MVP Status" in rendered and "Recommendation" in rendered,
        "rendered status should include the operator view headings",
    )
    require(
        "regatherManually" in rendered,
        "rendered status should expose the CRRC recommendation",
    )
    lane_ids = [lane["id"] for lane in status["roles"]["roles"]]
    require(
        lane_ids == ["implementation", "modeling", "verification", "reorientation"],
        "roles surface should expose the four MVP role lanes",
    )
    require(
        status["roles"]["note"].startswith("Role ownership is derived read-only"),
        "roles surface should declare read-only derived ownership",
    )
    require(
        "Role Lanes" in rendered and "Verification / Review" in rendered,
        "rendered status should expose the role lane section",
    )
    require(
        "Role Findings" in rendered
        and status["roleResults"]["modeling"]["status"] == "missingState"
        and status["roleResults"]["verification"]["status"] == "missingState",
        "rendered status should expose fixed role result read-back status",
    )

    result = {
        "threadId": status["threadId"],
        "recommendation": status["crrc"]["recommendation"],
        "roles": status["roles"],
        "roleResults": status["roleResults"],
        "stateStatus": status["scene"]["scene"]["stateStatus"],
        "availableActions": status["scene"]["scene"]["availableActions"],
        "rendered": rendered,
    }
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(
        json.dumps(result, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    return result


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Live-smoke the Epiphany MVP operator status view."
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
