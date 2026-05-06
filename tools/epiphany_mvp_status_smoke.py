from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess

from epiphany_mvp_status import DEFAULT_APP_SERVER
from epiphany_mvp_status import ROOT
from epiphany_phase5_smoke import require
from epiphany_phase5_smoke import reset_smoke_paths


DEFAULT_CODEX_HOME = ROOT / ".epiphany-smoke" / "mvp-status-codex-home"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "mvp-status-smoke-result.json"
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-smoke" / "mvp-status-smoke-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-smoke" / "mvp-status-smoke-server.stderr.log"
DEFAULT_RENDERED = ROOT / ".epiphany-smoke" / "mvp-status-smoke-rendered.txt"


def native_status_exe() -> Path:
    exe = Path(os.environ.get("CARGO_TARGET_DIR", r"C:\Users\Meta\.cargo-target-codex")) / "debug" / "epiphany-mvp-status.exe"
    if exe.exists():
        return exe
    subprocess.run(
        [
            "cargo",
            "build",
            "--manifest-path",
            str(ROOT / "epiphany-core" / "Cargo.toml"),
            "--bin",
            "epiphany-mvp-status",
        ],
        cwd=ROOT,
        check=True,
    )
    require(exe.exists(), f"native status binary was not built: {exe}")
    return exe


def run_smoke(args: argparse.Namespace) -> dict[str, object]:
    app_server = args.app_server.resolve()
    if not app_server.exists():
        raise FileNotFoundError(f"codex app-server binary not found: {app_server}")

    codex_home = args.codex_home.resolve()
    result_path = args.result.resolve()
    transcript_path = args.transcript.resolve()
    stderr_path = args.stderr.resolve()
    reset_smoke_paths(codex_home, result_path, transcript_path, stderr_path)
    rendered_path = args.rendered.resolve()
    if rendered_path.exists():
        rendered_path.unlink()

    exe = native_status_exe()
    completed = subprocess.run(
        [
            str(exe),
            "--app-server",
            str(app_server),
            "--codex-home",
            str(codex_home),
            "--cwd",
            str(ROOT),
            "--transcript",
            str(transcript_path),
            "--stderr",
            str(stderr_path),
            "--result",
            str(result_path),
            "--json",
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    require(completed.returncode == 0, completed.stderr or completed.stdout)
    status = json.loads(completed.stdout)
    rendered_completed = subprocess.run(
        [
            str(exe),
            "--app-server",
            str(app_server),
            "--codex-home",
            str(codex_home),
            "--cwd",
            str(ROOT),
            "--transcript",
            str(transcript_path.with_name("mvp-status-smoke-render-transcript.jsonl")),
            "--stderr",
            str(stderr_path.with_name("mvp-status-smoke-render-server.stderr.log")),
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    require(rendered_completed.returncode == 0, rendered_completed.stderr or rendered_completed.stdout)
    rendered = rendered_completed.stdout
    rendered_path.write_text(rendered, encoding="utf-8")

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
        lane_ids
        == ["implementation", "imagination", "modeling", "verification", "reorientation"],
        "roles surface should expose the five MVP role lanes",
    )
    require(
        status["roles"]["note"].startswith("Role ownership is derived read-only"),
        "roles surface should declare read-only derived ownership",
    )
    require(
        status["planning"]["stateStatus"] == "missing"
        and status["planning"]["summary"]["captureCount"] == 0,
        "fresh status smoke should expose empty planning state honestly",
    )
    require(
        "Planning" in rendered and "captures: 0" in rendered,
        "rendered status should expose the planning section",
    )
    require(
        "Role Lanes" in rendered and "Verification / Review" in rendered,
        "rendered status should expose the role lane section",
    )
    require(
        "Role Findings" in rendered
        and status["roleResults"]["imagination"]["status"] == "missingState"
        and status["roleResults"]["modeling"]["status"] == "missingState"
        and status["roleResults"]["verification"]["status"] == "missingState",
        "rendered status should expose fixed role result read-back status",
    )
    require(
        status["heartbeat"]["schema_version"] == "epiphany.agent_heartbeat_status.v0",
        "status view should expose heartbeat initiative status for Aquarium",
    )
    require(
        status["face"]["availableActions"] == ["faceBubble"],
        "status view should expose Face bubble action for Aquarium",
    )
    require(
        "Heartbeat" in rendered and "Face" in rendered,
        "rendered status should expose heartbeat and Face sections",
    )

    result = {
        "threadId": status["threadId"],
        "recommendation": status["crrc"]["recommendation"],
        "roles": status["roles"],
        "roleResults": status["roleResults"],
        "planning": status["planning"],
        "heartbeat": status["heartbeat"],
        "face": status["face"],
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
    parser.add_argument("--rendered", type=Path, default=DEFAULT_RENDERED)
    args = parser.parse_args()
    result = run_smoke(args)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
