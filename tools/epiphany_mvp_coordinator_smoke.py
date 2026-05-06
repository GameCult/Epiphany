from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path
from types import SimpleNamespace
from typing import Any

from epiphany_agent_memory import project_json_dir
from epiphany_agent_memory import resolve_store_path
from epiphany_mvp_coordinator import DEFAULT_APP_SERVER
from epiphany_mvp_coordinator import DEFAULT_AGENT_MEMORY_DIR
from epiphany_mvp_coordinator import run_coordinator
from epiphany_phase5_smoke import ROOT
from epiphany_phase5_smoke import require


DEFAULT_ARTIFACT_ROOT = ROOT / ".epiphany-dogfood" / "coordinator-smoke"


def reset_artifact_root(path: Path) -> None:
    root = (ROOT / ".epiphany-dogfood").resolve()
    resolved = path.resolve()
    if resolved == root or root not in resolved.parents:
        raise ValueError(f"refusing to delete non-dogfood artifact root: {path}")
    if path.exists():
        shutil.rmtree(path)
    path.mkdir(parents=True, exist_ok=True)


def coordinator_args(
    *,
    app_server: Path,
    artifact_root: Path,
    name: str,
    mode: str = "plan",
    bootstrap_smoke_state: bool = False,
    simulate_source_drift: bool = False,
    simulate_high_pressure: bool = False,
    dry_compact: bool = False,
    auto_review: bool = False,
    max_steps: int = 4,
    agent_memory_store: Path | None = None,
) -> argparse.Namespace:
    artifact_dir = artifact_root / name
    return SimpleNamespace(
        app_server=app_server,
        thread_id=None,
        cwd=ROOT,
        codex_home=artifact_dir / "codex-home",
        artifact_dir=artifact_dir,
        agent_memory_dir=agent_memory_store or artifact_root / "agent-memory.msgpack",
        mode=mode,
        max_steps=max_steps,
        poll_seconds=0.1,
        timeout_seconds=20,
        max_runtime_seconds=30,
        ephemeral=True,
        auto_review=auto_review,
        test_complete_backend=True,
        bootstrap_smoke_state=bootstrap_smoke_state,
        simulate_high_pressure=simulate_high_pressure,
        simulate_source_drift=simulate_source_drift,
        dry_compact=dry_compact,
    )


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
    ):
        require((artifact_dir / name).exists(), f"missing coordinator artifact {name}")


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
    agent_memory_store = artifact_root / "agent-memory.msgpack"
    shutil.copy2(resolve_store_path(DEFAULT_AGENT_MEMORY_DIR), agent_memory_store)

    cold = run_coordinator(
        coordinator_args(
            app_server=app_server,
            artifact_root=artifact_root,
            name="cold",
            agent_memory_store=agent_memory_store,
        )
    )
    require(
        cold["finalAction"]["action"] == "prepareCheckpoint",
        "cold start should stop at prepareCheckpoint",
    )
    require_artifacts(cold)
    require_operator_safe(cold)

    modeling = run_coordinator(
        coordinator_args(
            app_server=app_server,
            artifact_root=artifact_root,
            name="modeling",
            mode="run",
            bootstrap_smoke_state=True,
            max_steps=1,
        )
    )
    require(
        modeling["finalAction"]["action"] == "reviewModelingResult",
        "modeling run should stop with a reviewable modeling result",
    )
    require_artifacts(modeling)
    require_operator_safe(modeling)

    verification = run_coordinator(
        coordinator_args(
            app_server=app_server,
            artifact_root=artifact_root,
            name="verification",
            mode="run",
            bootstrap_smoke_state=True,
            auto_review=True,
            max_steps=4,
        )
    )
    require(
        verification["steps"][-1]["action"] == "reviewVerificationResult",
        "auto-review smoke should drive through verification and stop at review",
    )
    require_artifacts(verification)
    require_operator_safe(verification)
    projected_memory = artifact_root / "agent-memory-projection"
    project_json_dir(agent_memory_store, projected_memory)
    require(
        "mem-body-phase6-role-smoke"
        in (projected_memory / "body.agent-state.json").read_text(encoding="utf-8"),
        "auto-review coordinator smoke should apply accepted selfPatch to isolated agent memory",
    )

    drift = run_coordinator(
        coordinator_args(
            app_server=app_server,
            artifact_root=artifact_root,
            name="drift",
            mode="run",
            bootstrap_smoke_state=True,
            simulate_source_drift=True,
            max_steps=1,
        )
    )
    require(
        drift["finalAction"]["action"] == "reviewReorientResult",
        "source drift should launch reorient worker and stop for review",
    )
    require_artifacts(drift)
    require_operator_safe(drift)

    pressure = run_coordinator(
        coordinator_args(
            app_server=app_server,
            artifact_root=artifact_root,
            name="pressure",
            mode="run",
            bootstrap_smoke_state=True,
            simulate_high_pressure=True,
            dry_compact=True,
            max_steps=1,
        )
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

    result = {
        "artifactRoot": str(artifact_root),
        "coldAction": cold["finalAction"]["action"],
        "modelingAction": modeling["finalAction"]["action"],
        "verificationAction": verification["steps"][-1]["action"],
        "driftAction": drift["finalAction"]["action"],
        "pressureAction": pressure["steps"][0]["action"],
    }
    (artifact_root / "coordinator-smoke-summary.json").write_text(
        json.dumps(result, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    return result


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Smoke the auditable Epiphany MVP coordinator over fixed lanes."
    )
    parser.add_argument("--app-server", type=Path, default=DEFAULT_APP_SERVER)
    parser.add_argument("--artifact-root", type=Path, default=DEFAULT_ARTIFACT_ROOT)
    args = parser.parse_args()
    result = run_smoke(args)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
