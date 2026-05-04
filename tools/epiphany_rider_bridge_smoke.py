from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
from typing import Any

from epiphany_phase5_smoke import ROOT
from epiphany_phase5_smoke import require


DEFAULT_WORKSPACE = ROOT / ".epiphany-smoke" / "rider-bridge-workspace"
DEFAULT_ARTIFACT_ROOT = ROOT / ".epiphany-smoke" / "rider-bridge-artifacts"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "rider-bridge-smoke-result.json"


def reset_path(path: Path) -> None:
    if path.exists():
        shutil.rmtree(path)
    path.mkdir(parents=True, exist_ok=True)


def prepare_workspace(workspace: Path) -> Path:
    reset_path(workspace)
    (workspace / "Aetheria.sln").write_text("Microsoft Visual Studio Solution File\n", encoding="utf-8")
    source = workspace / "Assets" / "Scripts" / "GravityTileRenderer.cs"
    source.parent.mkdir(parents=True, exist_ok=True)
    source.write_text(
        "\n".join(
            [
                "namespace Aetheria.Rendering",
                "{",
                "    public sealed class GravityTileRenderer",
                "    {",
                "    }",
                "}",
                "",
            ]
        ),
        encoding="utf-8",
    )
    return source


def create_fake_rider(root: Path) -> Path:
    rider = root / "JetBrains Rider 2026.1.0.1" / "bin" / "rider64.exe"
    rider.parent.mkdir(parents=True, exist_ok=True)
    rider.write_text("fake rider binary for smoke planning only\n", encoding="utf-8")
    return rider


def run_bridge(workspace: Path, artifact_root: Path, env: dict[str, str], args: list[str]) -> dict[str, Any]:
    completed = subprocess.run(
        [
            sys.executable,
            str(ROOT / "tools" / "epiphany_rider_bridge.py"),
            args[0],
            "--project-root",
            str(workspace),
            "--artifact-root",
            str(artifact_root),
            *args[1:],
        ],
        cwd=ROOT,
        env=env,
        capture_output=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )
    require(completed.returncode == 0, f"rider bridge failed: {completed.stderr}")
    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise AssertionError(f"rider bridge did not return JSON: {error}\n{completed.stdout}") from error


def require_artifact(summary: dict[str, Any], name: str) -> None:
    artifact_path = Path(str(summary["artifactPath"]))
    require((artifact_path / name).exists(), f"missing rider artifact: {name}")


def main() -> int:
    workspace = DEFAULT_WORKSPACE.resolve()
    artifact_root = DEFAULT_ARTIFACT_ROOT.resolve()
    fake_rider_root = ROOT / ".epiphany-smoke" / "rider-bridge-install"
    result_path = DEFAULT_RESULT.resolve()
    source = prepare_workspace(workspace)
    reset_path(artifact_root)
    reset_path(fake_rider_root)
    fake_rider = create_fake_rider(fake_rider_root)

    env = os.environ.copy()
    env["EPIPHANY_RIDER_PATH"] = str(fake_rider)

    status = run_bridge(workspace, artifact_root, env, ["status"])
    require(status["status"] == "ready", "fake Rider path should make status ready")
    require(status["riderPath"] == str(fake_rider), "status should resolve fake Rider path")
    require(status["solutionPath"].endswith("Aetheria.sln"), "status should find the workspace solution")
    require_artifact(status, "rider-bridge-summary.json")
    require_artifact(status, "rider-bridge-status.md")

    context = run_bridge(
        workspace,
        artifact_root,
        env,
        [
            "context",
            "--file",
            str(source),
            "--selection-start",
            "3",
            "--selection-end",
            "5",
            "--symbol-name",
            "GravityTileRenderer",
            "--symbol-kind",
            "class",
            "--symbol-namespace",
            "Aetheria.Rendering",
        ],
    )
    require(context["status"] == "captured", "context command should capture a packet")
    require(context["filePath"] == "Assets/Scripts/GravityTileRenderer.cs", "context file should be project-relative")
    require_artifact(context, "rider-context.json")
    require_artifact(context, "rider-context.md")

    open_ref = run_bridge(
        workspace,
        artifact_root,
        env,
        ["open-ref", "--file", str(source), "--line", "3"],
    )
    require(open_ref["status"] == "planned", "open-ref should plan without launch")
    require(open_ref["command"][0] == str(fake_rider), "open-ref should use discovered Rider path")
    require_artifact(open_ref, "rider-open-ref.json")

    result = {
        "workspace": str(workspace),
        "artifactRoot": str(artifact_root),
        "status": status["status"],
        "contextStatus": context["status"],
        "openRefStatus": open_ref["status"],
        "riderPath": status["riderPath"],
    }
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
