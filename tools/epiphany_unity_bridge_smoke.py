from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
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


def prepare_project(workspace: Path, *, include_bridge: bool = False) -> None:
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
    if include_bridge:
        bridge = workspace / "Assets" / "Editor" / "Epiphany" / "EpiphanyEditorBridge.cs"
        bridge.parent.mkdir(parents=True, exist_ok=True)
        bridge.write_text("// smoke marker for resident Epiphany editor bridge\n", encoding="utf-8")


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
            str(native_bridge_exe()),
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


def read_command_artifact(summary: dict[str, Any]) -> dict[str, Any]:
    artifact_path = Path(str(summary["artifactPath"]))
    command_path = artifact_path / "unity-command.json"
    require(command_path.exists(), "missing unity-command.json")
    return json.loads(command_path.read_text(encoding="utf-8"))


def require_command_contains(command: list[str], value: str) -> None:
    require(value in command, f"command should contain {value!r}: {command}")


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
    require(ready["editorBridge"]["exists"] is False, "fresh smoke project should not have bridge package yet")

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
    require_command_contains(command, "-batchmode")
    require_command_contains(command, "-quit")
    require_command_contains(command, "-projectPath")
    require_command_contains(command, "-logFile")
    require_command_contains(command, str(workspace))
    require(str(wrong_editor) not in command, "command must not use wrong editor")
    require_artifact(planned, "unity-command.json")

    missing_package = run_bridge(
        workspace,
        artifact_root,
        roots=fake_roots,
        args=[
            "probe",
            "--dry-run",
            "--operation",
            "scene-facts",
            "--scene",
            "Assets/Scenes/Main.unity",
        ],
        expect_success=False,
    )
    require(
        missing_package["status"] == "missingEditorBridgePackage",
        "named probes should require the resident editor package",
    )
    require(missing_package["runStatus"] == "blocked", "missing package probe should be blocked")

    prepare_project(workspace, include_bridge=True)
    bridged = run_bridge(
        workspace,
        artifact_root,
        roots=fake_roots,
        args=["inspect"],
    )
    require(bridged["status"] == "ready", "bridge project should still resolve exact editor")
    require(bridged["editorBridge"]["exists"] is True, "resident editor bridge should be detected")

    scene_probe = run_bridge(
        workspace,
        artifact_root,
        roots=fake_roots,
        args=[
            "probe",
            "--dry-run",
            "--operation",
            "scene-facts",
            "--scene",
            "Assets/Scenes/Main.unity",
            "--max-objects",
            "25",
            "--max-properties",
            "12",
        ],
    )
    require(scene_probe["runStatus"] == "planned", "scene probe should plan under dry run")
    scene_command = scene_probe["command"]
    require(scene_command[0] == str(exact_editor), "scene probe should use exact editor path")
    require_command_contains(scene_command, "-executeMethod")
    require_command_contains(scene_command, "GameCult.Epiphany.Unity.EpiphanyEditorBridge.RunProbe")
    require_command_contains(scene_command, "-epiphanyArtifactDir")
    require_command_contains(scene_command, str(Path(str(scene_probe["artifactPath"]))))
    require_command_contains(scene_command, "-epiphanyOperation")
    require_command_contains(scene_command, "scene-facts")
    require_command_contains(scene_command, "-epiphanyScene")
    require_command_contains(scene_command, "Assets/Scenes/Main.unity")
    require_command_contains(scene_command, "-epiphanyMaxObjects")
    require_command_contains(scene_command, "25")
    scene_artifact = read_command_artifact(scene_probe)
    require(scene_artifact["operation"] == "scene-facts", "scene command artifact should name operation")
    require("scene-facts.json" in scene_artifact["expectedArtifacts"], "scene facts artifact should be expected")

    compilation = run_bridge(
        workspace,
        artifact_root,
        roots=fake_roots,
        args=["check-compilation", "--dry-run"],
    )
    require(compilation["runStatus"] == "planned", "compilation probe should plan under dry run")
    compilation_artifact = read_command_artifact(compilation)
    require(
        "compilation.json" in compilation_artifact["expectedArtifacts"],
        "compilation artifact should be expected",
    )

    tests = run_bridge(
        workspace,
        artifact_root,
        roots=fake_roots,
        args=["run-tests", "--dry-run", "--platform", "editmode", "--filter", "SmokeSuite"],
    )
    require(tests["runStatus"] == "planned", "Unity test run should plan under dry run")
    test_command = tests["command"]
    require_command_contains(test_command, "-runTests")
    require_command_contains(test_command, "-testPlatform")
    require_command_contains(test_command, "editmode")
    require_command_contains(test_command, "-testFilter")
    require_command_contains(test_command, "SmokeSuite")
    tests_artifact = read_command_artifact(tests)
    require("test-results.xml" in tests_artifact["expectedArtifacts"], "test results should be expected")

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
        "missingPackageStatus": missing_package["status"],
        "sceneProbeStatus": scene_probe["runStatus"],
        "testStatus": tests["runStatus"],
        "blockedStatus": blocked["runStatus"],
        "resolvedEditor": ready["editorPath"],
    }
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


def native_bridge_exe() -> Path:
    exe = Path(os.environ.get("CARGO_TARGET_DIR", r"C:\Users\Meta\.cargo-target-codex")) / "debug" / "epiphany-unity-bridge.exe"
    if exe.exists():
        return exe
    completed = subprocess.run(
        [
            "cargo",
            "build",
            "--manifest-path",
            str(ROOT / "epiphany-core" / "Cargo.toml"),
            "--bin",
            "epiphany-unity-bridge",
        ],
        cwd=ROOT,
        capture_output=True,
        encoding="utf-8",
        errors="replace",
        check=False,
    )
    require(completed.returncode == 0, f"failed to build native unity bridge: {completed.stderr}")
    require(exe.exists(), f"native unity bridge executable was not built at {exe}")
    return exe


if __name__ == "__main__":
    raise SystemExit(main())
