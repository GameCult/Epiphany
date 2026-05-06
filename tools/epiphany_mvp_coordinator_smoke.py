from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
from typing import Any

from epiphany_mvp_status import DEFAULT_APP_SERVER
from epiphany_mvp_status import ROOT
from epiphany_phase5_smoke import require


DEFAULT_ARTIFACT_ROOT = ROOT / ".epiphany-dogfood" / "coordinator-smoke"


def native_coordinator_exe() -> Path:
    exe = Path(os.environ.get("CARGO_TARGET_DIR", r"C:\Users\Meta\.cargo-target-codex")) / "debug" / "epiphany-mvp-coordinator.exe"
    subprocess.run(
        [
            "cargo",
            "build",
            "--manifest-path",
            str(ROOT / "epiphany-core" / "Cargo.toml"),
            "--bin",
            "epiphany-mvp-coordinator",
        ],
        cwd=ROOT,
        check=True,
    )
    require(exe.exists(), f"native coordinator binary was not built: {exe}")
    return exe


def reset_artifact_root(path: Path) -> None:
    root = (ROOT / ".epiphany-dogfood").resolve()
    resolved = path.resolve()
    if resolved == root or root not in resolved.parents:
        raise ValueError(f"refusing to delete non-dogfood artifact root: {path}")
    if path.exists():
        shutil.rmtree(path)
    path.mkdir(parents=True, exist_ok=True)


def run_native(
    *,
    exe: Path,
    app_server: Path,
    artifact_root: Path,
    name: str,
    mode: str = "plan",
    bootstrap_smoke_state: bool = False,
    simulate_high_pressure: bool = False,
    dry_compact: bool = False,
    max_steps: int = 1,
) -> dict[str, Any]:
    artifact_dir = artifact_root / name
    command = [
        str(exe),
        "--app-server",
        str(app_server),
        "--artifact-dir",
        str(artifact_dir),
        "--runtime-store",
        str(artifact_dir / "runtime-spine.msgpack"),
        "--codex-home",
        str(artifact_dir / "codex-home"),
        "--cwd",
        str(ROOT),
        "--mode",
        mode,
        "--max-steps",
        str(max_steps),
        "--poll-seconds",
        "0.1",
        "--timeout-seconds",
        "3",
    ]
    if bootstrap_smoke_state:
        command.append("--bootstrap-smoke-state")
    if simulate_high_pressure:
        command.append("--simulate-high-pressure")
    if dry_compact:
        command.append("--dry-compact")
    completed = subprocess.run(command, cwd=ROOT, text=True, capture_output=True, check=False)
    require(completed.returncode == 0, completed.stderr or completed.stdout)
    return json.loads(completed.stdout)


def require_artifacts(summary: dict[str, Any]) -> None:
    artifact_dir = Path(summary["artifactDir"])
    for name in (
        "coordinator-summary.json",
        "coordinator-steps.jsonl",
        "coordinator-final-status.json",
        "coordinator-final-status.txt",
        "coordinator-final-action.txt",
        "epiphany-transcript.jsonl",
        "epiphany-server.stderr.log",
        "agent-function-telemetry.json",
        "runtime-spine-status.json",
    ):
        require((artifact_dir / name).exists(), f"missing coordinator artifact {name}")
    require((artifact_dir / "runtime-spine.msgpack").exists(), "missing native runtime spine store")
    runtime_status = json.loads((artifact_dir / "runtime-spine-status.json").read_text(encoding="utf-8"))
    require(runtime_status["present"] is True, "native runtime spine should be present")
    require(runtime_status["sessions"] >= 1, "native runtime spine should record a session")


def require_operator_safe(value: Any, path: str = "$") -> None:
    if isinstance(value, dict):
        if "rawResult" in value:
            raw_result = value["rawResult"]
            require(
                isinstance(raw_result, dict) and raw_result.get("sealed") is True,
                f"{path}.rawResult should be sealed in operator-facing artifacts",
            )
        for key, item in value.items():
            require_operator_safe(item, f"{path}.{key}")
    elif isinstance(value, list):
        for index, item in enumerate(value):
            require_operator_safe(item, f"{path}[{index}]")


def run_smoke(args: argparse.Namespace) -> dict[str, Any]:
    app_server = args.app_server.resolve()
    if not app_server.exists():
        raise FileNotFoundError(f"codex app-server binary not found: {app_server}")

    artifact_root = args.artifact_root.resolve()
    reset_artifact_root(artifact_root)
    exe = native_coordinator_exe()

    cold = run_native(
        exe=exe,
        app_server=app_server,
        artifact_root=artifact_root,
        name="cold",
    )
    require(
        cold["finalAction"]["action"] == "prepareCheckpoint",
        "cold start should stop at prepareCheckpoint",
    )
    require_artifacts(cold)
    require_operator_safe(cold)

    pressure = run_native(
        exe=exe,
        app_server=app_server,
        artifact_root=artifact_root,
        name="pressure",
        mode="run",
        bootstrap_smoke_state=True,
        simulate_high_pressure=True,
        dry_compact=True,
    )
    require(
        pressure["steps"][0]["action"] == "compactRehydrateReorient",
        "simulated high pressure should select compact/rehydrate/reorient",
    )
    require(
        pressure["steps"][0]["events"][0]["type"] == "dryCompact",
        "dry compact smoke should record the compaction action",
    )
    require_artifacts(pressure)
    require_operator_safe(pressure)

    rejected = subprocess.run(
        [
            str(exe),
            "--artifact-dir",
            str(artifact_root / "rejected-private-store-completion"),
            "--test-complete-backend",
        ],
        cwd=ROOT,
        text=True,
        capture_output=True,
        check=False,
    )
    require(
        rejected.returncode != 0 and "CultNet job-result API" in rejected.stderr,
        "native coordinator should reject direct backend-completion mutation",
    )

    result = {
        "artifactRoot": str(artifact_root),
        "coldAction": cold["finalAction"]["action"],
        "pressureAction": pressure["steps"][0]["action"],
        "directBackendCompletionRejected": True,
        "note": "Native smoke does not fake specialist completion by mutating Codex state storage; full completion smoke needs a CultNet job-result API.",
    }
    (artifact_root / "coordinator-smoke-summary.json").write_text(
        json.dumps(result, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    return result


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Smoke the native auditable Epiphany MVP coordinator."
    )
    parser.add_argument("--app-server", type=Path, default=DEFAULT_APP_SERVER)
    parser.add_argument("--artifact-root", type=Path, default=DEFAULT_ARTIFACT_ROOT)
    args = parser.parse_args()
    result = run_smoke(args)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
