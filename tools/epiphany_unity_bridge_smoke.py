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


DEFAULT_WORKSPACE = ROOT / ".epiphany-smoke" / "unity-bridge-workspace"
DEFAULT_ARTIFACT_ROOT = ROOT / ".epiphany-smoke" / "unity-bridge-artifacts"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "unity-bridge-smoke-result.json"
PROJECT_VERSION = "6000.1.10f1"
WRONG_VERSION = "6000.4.2f1"


def reset_path(path: Path) -> None:
    if path.exists():
        shutil.rmtree(path)
    path.mkdir(parents=True, exist_ok=True)


def prepare_project(workspace: Path) -> None:
    reset_path(workspace)
    settings = workspace / "ProjectSettings"
    settings.mkdir(parents=True, exist_ok=True)
    (settings / "ProjectVersion.txt").write_text(
        "\n".join(
            [
                f"m_EditorVersion: {PROJECT_VERSION}",
                f"m_EditorVersionWithRevision: {PROJECT_VERSION} (3c681a6c22ff)",
                "",
            ]
        ),
        encoding="utf-8",
    )


def create_fake_editor(root: Path, version: str) -> Path:
    editor = root / version / "Editor" / "Unity.exe"
    editor.parent.mkdir(parents=True, exist_ok=True)
    editor.write_text("fake unity binary for dry-run smoke only\n", encoding="utf-8")
    return editor


def run_bridge(
    workspace: Path,
    artifact_root: Path,
    *,
    roots: Path,
    args: list[str],
    expect_success: bool = True,
) -> dict[str, Any]:
    env = os.environ.copy()
    env["EPIPHANY_UNITY_EDITOR_ROOTS"] = str(roots)
    completed = subprocess.run(
        [
            sys.executable,
            str(ROOT / "tools" / "epiphany_unity_bridge.py"),
            args[0],
            "--project-path",
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
    if expect_success:
        require(
            completed.returncode == 0,
            f"bridge should succeed, got {completed.returncode}: {completed.stderr}",
        )
    else:
        require(completed.returncode != 0, "bridge should fail for blocked runtime execution")
    try:
        return json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        raise AssertionError(f"bridge did not return JSON: {error}\n{completed.stdout}") from error


def require_artifact(summary: dict[str, Any], name: str) -> None:
    artifact_path = Path(str(summary["artifactPath"]))
    require((artifact_path / name).exists(), f"missing bridge artifact: {name}")


def main() -> int:
    workspace = DEFAULT_WORKSPACE.resolve()
    artifact_root = DEFAULT_ARTIFACT_ROOT.resolve()
    fake_roots = ROOT / ".epiphany-smoke" / "unity-bridge-editors"
    result_path = DEFAULT_RESULT.resolve()
    prepare_project(workspace)
    reset_path(artifact_root)
    reset_path(fake_roots)

    wrong_editor = create_fake_editor(fake_roots, WRONG_VERSION)
    missing = run_bridge(
        workspace,
        artifact_root,
        roots=fake_roots,
        args=["inspect"],
    )
    require(missing["status"] == "missingEditor", "wrong-only Hub root should not satisfy pin")
    require(
        missing["projectVersion"] == PROJECT_VERSION,
        "inspection should report the project-pinned version",
    )
    require_artifact(missing, "unity-bridge-summary.json")
    require_artifact(missing, "unity-bridge-inspection.md")

    exact_editor = create_fake_editor(fake_roots, PROJECT_VERSION)
    ready = run_bridge(
        workspace,
        artifact_root,
        roots=fake_roots,
        args=["inspect"],
    )
    require(ready["status"] == "ready", "exact editor should satisfy pin")
    require(ready["editorPath"] == str(exact_editor), "inspection should resolve exact editor")

    planned = run_bridge(
        workspace,
        artifact_root,
        roots=fake_roots,
        args=["run", "--dry-run", "--", "-executeMethod", "Epiphany.SmokeProbe"],
    )
    require(planned["status"] == "ready", "dry run should still require exact editor")
    require(planned["runStatus"] == "planned", "dry run should plan instead of executing")
    command = planned["command"]
    require(command[0] == str(exact_editor), "command should use exact editor path")
    require("-batchmode" in command, "bridge should own -batchmode")
    require("-quit" in command, "bridge should own -quit")
    require("-projectPath" in command, "bridge should own -projectPath")
    require(str(workspace) in command, "command should target the requested project")
    require(str(wrong_editor) not in command, "command must not use wrong editor")
    require_artifact(planned, "unity-command.json")

    blocked = run_bridge(
        workspace,
        artifact_root,
        roots=fake_roots / "missing",
        args=["run", "--dry-run", "--", "-executeMethod", "Epiphany.SmokeProbe"],
        expect_success=False,
    )
    require(blocked["runStatus"] == "blocked", "missing editor run should be blocked")

    result = {
        "workspace": str(workspace),
        "artifactRoot": str(artifact_root),
        "missingStatus": missing["status"],
        "readyStatus": ready["status"],
        "plannedStatus": planned["runStatus"],
        "blockedStatus": blocked["runStatus"],
        "resolvedEditor": ready["editorPath"],
    }
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
