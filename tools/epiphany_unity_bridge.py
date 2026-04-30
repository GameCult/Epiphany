from __future__ import annotations

import argparse
from datetime import datetime
from datetime import timezone
import json
import os
import subprocess
import time
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_ARTIFACT_ROOT = ROOT / ".epiphany-gui" / "runtime"
PROJECT_VERSION_RELATIVE = Path("ProjectSettings") / "ProjectVersion.txt"
UNITY_EDITOR_RELATIVE = Path("Editor") / "Unity.exe"
FORBIDDEN_EXTRA_ARGS = {"-batchmode", "-quit", "-projectpath", "-logfile"}
EDITOR_BRIDGE_RELATIVE = Path("Assets") / "Editor" / "Epiphany" / "EpiphanyEditorBridge.cs"
EDITOR_BRIDGE_EXECUTE_METHOD = "GameCult.Epiphany.Unity.EpiphanyEditorBridge.RunProbe"
NAMED_PROBE_OPERATIONS = {
    "inspect-project": {
        "description": "Capture Unity project/editor/build/render-pipeline facts.",
        "expectedArtifacts": ["project-facts.json", "unity-probe-result.json", "unity-probe-result.md"],
    },
    "check-compilation": {
        "description": "Check whether the editor bridge can run after Unity script compilation.",
        "expectedArtifacts": ["compilation.json", "unity-probe-result.json", "unity-probe-result.md"],
    },
    "scene-facts": {
        "description": "Capture scene hierarchy, components, prefab links, and serialized fields.",
        "expectedArtifacts": ["scene-facts.json", "unity-probe-result.json", "unity-probe-result.md"],
    },
    "prefab-facts": {
        "description": "Capture prefab hierarchy, nested instances, overrides, and serialized fields.",
        "expectedArtifacts": ["prefab-facts.json", "unity-probe-result.json", "unity-probe-result.md"],
    },
    "serialized-object": {
        "description": "Capture SerializedObject properties for one asset.",
        "expectedArtifacts": [
            "serialized-object-facts.json",
            "unity-probe-result.json",
            "unity-probe-result.md",
        ],
    },
    "reference-search": {
        "description": "Search Unity asset dependencies for references to a GUID or asset.",
        "expectedArtifacts": ["reference-search.json", "unity-probe-result.json", "unity-probe-result.md"],
    },
}


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def write_text(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value, encoding="utf-8")


def read_unity_project_version(project_path: Path) -> dict[str, Any]:
    version_path = project_path / PROJECT_VERSION_RELATIVE
    result: dict[str, Any] = {
        "path": str(version_path),
        "exists": version_path.exists(),
        "editorVersion": None,
        "editorVersionWithRevision": None,
    }
    if not version_path.exists():
        result["status"] = "missingProjectVersion"
        result["note"] = "ProjectSettings/ProjectVersion.txt was not found."
        return result

    for line in version_path.read_text(encoding="utf-8", errors="replace").splitlines():
        if line.startswith("m_EditorVersion:"):
            result["editorVersion"] = line.split(":", 1)[1].strip() or None
        elif line.startswith("m_EditorVersionWithRevision:"):
            result["editorVersionWithRevision"] = line.split(":", 1)[1].strip() or None

    if result["editorVersion"]:
        result["status"] = "ready"
    else:
        result["status"] = "missingProjectVersion"
        result["note"] = "ProjectVersion.txt did not contain m_EditorVersion."
    return result


def default_unity_editor_roots() -> list[Path]:
    roots: list[Path] = []
    env_roots = os.environ.get("EPIPHANY_UNITY_EDITOR_ROOTS")
    if env_roots:
        roots.extend(Path(item) for item in env_roots.split(os.pathsep) if item)
    roots.extend(
        [
            Path(os.environ.get("ProgramFiles", r"C:\Program Files"))
            / "Unity"
            / "Hub"
            / "Editor",
            Path(os.environ.get("ProgramFiles(x86)", r"C:\Program Files (x86)"))
            / "Unity"
            / "Hub"
            / "Editor",
        ]
    )
    seen: set[str] = set()
    deduped: list[Path] = []
    for root in roots:
        key = str(root.resolve()) if root.exists() else str(root)
        if key in seen:
            continue
        seen.add(key)
        deduped.append(root)
    return deduped


def editor_path_for_version(root: Path, version: str) -> Path:
    if root.name == version:
        return root / UNITY_EDITOR_RELATIVE
    return root / version / UNITY_EDITOR_RELATIVE


def installed_unity_editors(roots: list[Path] | None = None) -> list[dict[str, Any]]:
    editors: list[dict[str, Any]] = []
    for root in roots or default_unity_editor_roots():
        if not root.exists():
            continue
        if (root / UNITY_EDITOR_RELATIVE).exists():
            editors.append(
                {
                    "version": root.name,
                    "root": str(root),
                    "editorPath": str(root / UNITY_EDITOR_RELATIVE),
                }
            )
            continue
        for child in sorted(root.iterdir(), key=lambda path: path.name.lower()):
            if not child.is_dir():
                continue
            editor_path = child / UNITY_EDITOR_RELATIVE
            if editor_path.exists():
                editors.append(
                    {
                        "version": child.name,
                        "root": str(child),
                        "editorPath": str(editor_path),
                    }
                )
    return editors


def resolve_unity_editor(project_path: Path) -> dict[str, Any]:
    project_path = project_path.resolve()
    version = read_unity_project_version(project_path)
    roots = default_unity_editor_roots()
    editors = installed_unity_editors(roots)
    result: dict[str, Any] = {
        "kind": "unity",
        "projectPath": str(project_path),
        "generatedAt": now_iso(),
        "versionFile": version,
        "searchRoots": [str(root) for root in roots],
        "installedEditors": editors,
        "projectVersion": version.get("editorVersion"),
        "projectVersionWithRevision": version.get("editorVersionWithRevision"),
        "editorPath": None,
        "status": "missingProjectVersion",
        "note": "ProjectSettings/ProjectVersion.txt was not found or did not pin a Unity editor.",
    }
    result["editorBridge"] = detect_editor_bridge(project_path)
    project_version = version.get("editorVersion")
    if not isinstance(project_version, str) or not project_version:
        return result

    exact_candidates = [editor_path_for_version(root, project_version) for root in roots]
    exact = next((candidate for candidate in exact_candidates if candidate.exists()), None)
    if exact is None:
        installed_versions = ", ".join(editor["version"] for editor in editors) or "none detected"
        result["status"] = "missingEditor"
        result["candidatePaths"] = [str(candidate) for candidate in exact_candidates]
        result["note"] = (
            f"Project pins Unity {project_version}, but no exact editor was found. "
            f"Installed Hub versions: {installed_versions}."
        )
        return result

    result["status"] = "ready"
    result["editorPath"] = str(exact)
    result["candidatePaths"] = [str(candidate) for candidate in exact_candidates]
    result["note"] = (
        f"Project pins Unity {project_version}; exact editor resolved at {exact}."
    )
    return result


def detect_editor_bridge(project_path: Path) -> dict[str, Any]:
    package_path = project_path.resolve() / EDITOR_BRIDGE_RELATIVE
    return {
        "kind": "epiphanyUnityEditorBridge",
        "path": str(package_path),
        "relativePath": str(EDITOR_BRIDGE_RELATIVE).replace("\\", "/"),
        "exists": package_path.exists(),
        "executeMethod": EDITOR_BRIDGE_EXECUTE_METHOD,
    }


def render_inspection(summary: dict[str, Any]) -> str:
    installed = summary.get("installedEditors")
    if not isinstance(installed, list) or not installed:
        installed_lines = ["- none detected"]
    else:
        installed_lines = [
            f"- {item.get('version')}: {item.get('editorPath')}"
            for item in installed
            if isinstance(item, dict)
        ]
    candidates = summary.get("candidatePaths")
    if not isinstance(candidates, list) or not candidates:
        candidate_lines = ["- none"]
    else:
        candidate_lines = [f"- {item}" for item in candidates]
    return "\n".join(
        [
            "# Unity Runtime Bridge",
            "",
            f"Status: {summary.get('status')}",
            f"Project: {summary.get('projectPath')}",
            f"Project version: {summary.get('projectVersion') or 'none'}",
            f"Editor path: {summary.get('editorPath') or 'none'}",
            "",
            "Installed Hub editors:",
            *installed_lines,
            "",
            "Exact candidate paths:",
            *candidate_lines,
            "",
            f"Note: {summary.get('note')}",
            "",
        ]
    )


def artifact_dir(root: Path, action: str) -> Path:
    return root.resolve() / f"unity-{action}-{time.time_ns()}-{os.getpid()}"


def write_inspection_artifacts(directory: Path, summary: dict[str, Any]) -> None:
    write_json(directory / "unity-bridge-summary.json", summary)
    write_text(directory / "unity-bridge-inspection.md", render_inspection(summary))


def normalize_unity_args(values: list[str]) -> list[str]:
    if values and values[0] == "--":
        values = values[1:]
    return values


def validate_unity_args(values: list[str]) -> None:
    lowered = {value.lower() for value in values}
    forbidden = sorted(lowered & FORBIDDEN_EXTRA_ARGS)
    if forbidden:
        raise ValueError(
            "Unity bridge owns batchmode, quit, projectPath, and logFile; remove extra args: "
            + ", ".join(forbidden)
        )


def build_unity_command(
    summary: dict[str, Any],
    extra_args: list[str],
    *,
    log_path: Path | None = None,
) -> list[str]:
    editor_path = summary.get("editorPath")
    if not isinstance(editor_path, str) or not editor_path:
        raise ValueError("Unity editor path is unavailable.")
    project_path = summary.get("projectPath")
    if not isinstance(project_path, str) or not project_path:
        raise ValueError("Unity project path is unavailable.")
    validate_unity_args(extra_args)
    command = [
        editor_path,
        "-batchmode",
        "-quit",
        "-projectPath",
        project_path,
    ]
    if log_path is not None:
        command.extend(["-logFile", str(log_path)])
    command.extend(extra_args)
    return command


def editor_bridge_ready(summary: dict[str, Any]) -> bool:
    bridge = summary.get("editorBridge")
    return isinstance(bridge, dict) and bridge.get("exists") is True


def named_probe_extra_args(args: argparse.Namespace, operation: str, directory: Path) -> list[str]:
    extra_args = [
        "-executeMethod",
        EDITOR_BRIDGE_EXECUTE_METHOD,
        "-epiphanyArtifactDir",
        str(directory),
        "-epiphanyOperation",
        operation,
    ]
    optional_values = [
        ("scene", "-epiphanyScene"),
        ("asset", "-epiphanyAsset"),
        ("guid", "-epiphanyGuid"),
        ("max_objects", "-epiphanyMaxObjects"),
        ("max_properties", "-epiphanyMaxProperties"),
        ("max_dependencies", "-epiphanyMaxDependencies"),
    ]
    for attribute, flag in optional_values:
        value = getattr(args, attribute, None)
        if value is None:
            continue
        extra_args.extend([flag, str(value)])
    return extra_args


def write_command_artifact(
    directory: Path,
    *,
    command: list[str],
    dry_run: bool,
    operation: str,
    expected_artifacts: list[str] | None = None,
) -> None:
    write_json(
        directory / "unity-command.json",
        {
            "command": command,
            "dryRun": dry_run,
            "operation": operation,
            "expectedArtifacts": expected_artifacts or [],
        },
    )


def run_unity(args: argparse.Namespace) -> dict[str, Any]:
    directory = args.artifact_dir.resolve() if args.artifact_dir else artifact_dir(args.artifact_root, "run")
    project_path = args.project_path.resolve()
    summary = resolve_unity_editor(project_path)
    summary["artifactPath"] = str(directory)
    summary["operation"] = "run"
    summary["label"] = args.label
    extra_args = normalize_unity_args(list(args.unity_args or []))
    summary["unityArgs"] = extra_args

    if summary.get("status") != "ready":
        summary["runStatus"] = "blocked"
        summary["note"] = (
            f"{summary.get('note')} Runtime execution refused; install the exact pinned editor "
            "or use inspect artifacts as evidence of the missing runtime."
        )
        write_inspection_artifacts(directory, summary)
        return summary

    log_path = directory / "unity.log"
    command = build_unity_command(summary, extra_args, log_path=log_path)
    summary["command"] = command
    summary["logPath"] = str(log_path)
    if args.dry_run:
        summary["runStatus"] = "planned"
        summary["returncode"] = None
        write_inspection_artifacts(directory, summary)
        write_command_artifact(directory, command=command, dry_run=True, operation="run")
        return summary

    stdout_path = directory / "unity-stdout.log"
    stderr_path = directory / "unity-stderr.log"
    started = time.time()
    try:
        completed = subprocess.run(
            command,
            cwd=project_path,
            capture_output=True,
            check=False,
            timeout=args.timeout_seconds,
        )
        duration_seconds = time.time() - started
        stdout_path.parent.mkdir(parents=True, exist_ok=True)
        stdout_path.write_bytes(completed.stdout)
        stderr_path.write_bytes(completed.stderr)
        summary.update(
            {
                "runStatus": "passed" if completed.returncode == 0 else "failed",
                "returncode": completed.returncode,
                "durationSeconds": round(duration_seconds, 3),
                "stdoutPath": str(stdout_path),
                "stderrPath": str(stderr_path),
            }
        )
    except subprocess.TimeoutExpired as error:
        duration_seconds = time.time() - started
        stdout_path.parent.mkdir(parents=True, exist_ok=True)
        if error.stdout:
            stdout_path.write_bytes(error.stdout)
            summary["stdoutPath"] = str(stdout_path)
        if error.stderr:
            stderr_path.write_bytes(error.stderr)
            summary["stderrPath"] = str(stderr_path)
        summary.update(
            {
                "runStatus": "timedOut",
                "returncode": None,
                "durationSeconds": round(duration_seconds, 3),
                "note": f"Unity command timed out after {args.timeout_seconds} seconds.",
            }
        )

    write_inspection_artifacts(directory, summary)
    write_command_artifact(directory, command=command, dry_run=False, operation="run")
    return summary


def run_named_probe(args: argparse.Namespace, operation: str) -> dict[str, Any]:
    directory = (
        args.artifact_dir.resolve()
        if args.artifact_dir
        else artifact_dir(args.artifact_root, operation.replace("-", "_"))
    )
    project_path = args.project_path.resolve()
    summary = resolve_unity_editor(project_path)
    summary["artifactPath"] = str(directory)
    summary["operation"] = operation
    summary["probe"] = NAMED_PROBE_OPERATIONS[operation]

    if summary.get("status") != "ready":
        summary["runStatus"] = "blocked"
        summary["note"] = (
            f"{summary.get('note')} Runtime/editor probe refused; install the exact pinned editor "
            "or use inspect artifacts as evidence of the missing runtime."
        )
        write_inspection_artifacts(directory, summary)
        return summary

    if not editor_bridge_ready(summary):
        bridge = summary.get("editorBridge")
        bridge_path = bridge.get("path") if isinstance(bridge, dict) else str(project_path / EDITOR_BRIDGE_RELATIVE)
        summary["status"] = "missingEditorBridgePackage"
        summary["runStatus"] = "blocked"
        summary["note"] = (
            "The Epiphany Unity editor package is missing. Add "
            f"{bridge_path} before running editor-resident probes."
        )
        write_inspection_artifacts(directory, summary)
        return summary

    extra_args = named_probe_extra_args(args, operation, directory)
    log_path = directory / "unity.log"
    command = build_unity_command(summary, extra_args, log_path=log_path)
    summary["command"] = command
    summary["logPath"] = str(log_path)
    expected_artifacts = NAMED_PROBE_OPERATIONS[operation]["expectedArtifacts"]

    if args.dry_run:
        summary["runStatus"] = "planned"
        summary["returncode"] = None
        write_inspection_artifacts(directory, summary)
        write_command_artifact(
            directory,
            command=command,
            dry_run=True,
            operation=operation,
            expected_artifacts=expected_artifacts,
        )
        return summary

    stdout_path = directory / "unity-stdout.log"
    stderr_path = directory / "unity-stderr.log"
    started = time.time()
    try:
        completed = subprocess.run(
            command,
            cwd=project_path,
            capture_output=True,
            check=False,
            timeout=args.timeout_seconds,
        )
        duration_seconds = time.time() - started
        stdout_path.parent.mkdir(parents=True, exist_ok=True)
        stdout_path.write_bytes(completed.stdout)
        stderr_path.write_bytes(completed.stderr)
        summary.update(
            {
                "runStatus": "passed" if completed.returncode == 0 else "failed",
                "returncode": completed.returncode,
                "durationSeconds": round(duration_seconds, 3),
                "stdoutPath": str(stdout_path),
                "stderrPath": str(stderr_path),
            }
        )
    except subprocess.TimeoutExpired as error:
        duration_seconds = time.time() - started
        stdout_path.parent.mkdir(parents=True, exist_ok=True)
        if error.stdout:
            stdout_path.write_bytes(error.stdout)
            summary["stdoutPath"] = str(stdout_path)
        if error.stderr:
            stderr_path.write_bytes(error.stderr)
            summary["stderrPath"] = str(stderr_path)
        summary.update(
            {
                "runStatus": "timedOut",
                "returncode": None,
                "durationSeconds": round(duration_seconds, 3),
                "note": f"Unity command timed out after {args.timeout_seconds} seconds.",
            }
        )

    write_inspection_artifacts(directory, summary)
    write_command_artifact(
        directory,
        command=command,
        dry_run=False,
        operation=operation,
        expected_artifacts=expected_artifacts,
    )
    return summary


def check_compilation(args: argparse.Namespace) -> dict[str, Any]:
    return run_named_probe(args, "check-compilation")


def probe_unity(args: argparse.Namespace) -> dict[str, Any]:
    return run_named_probe(args, args.operation)


def test_unity(args: argparse.Namespace) -> dict[str, Any]:
    directory = args.artifact_dir.resolve() if args.artifact_dir else artifact_dir(args.artifact_root, "run_tests")
    project_path = args.project_path.resolve()
    summary = resolve_unity_editor(project_path)
    summary["artifactPath"] = str(directory)
    summary["operation"] = "run-tests"
    summary["testPlatform"] = args.platform
    summary["testFilter"] = args.filter

    if summary.get("status") != "ready":
        summary["runStatus"] = "blocked"
        summary["note"] = (
            f"{summary.get('note')} Unity tests refused; install the exact pinned editor "
            "or use inspect artifacts as evidence of the missing runtime."
        )
        write_inspection_artifacts(directory, summary)
        return summary

    log_path = directory / "unity.log"
    test_results = directory / "test-results.xml"
    extra_args = [
        "-runTests",
        "-testPlatform",
        args.platform,
        "-testResults",
        str(test_results),
    ]
    if args.filter:
        extra_args.extend(["-testFilter", args.filter])
    command = build_unity_command(summary, extra_args, log_path=log_path)
    summary["command"] = command
    summary["logPath"] = str(log_path)
    summary["testResultsPath"] = str(test_results)

    if args.dry_run:
        summary["runStatus"] = "planned"
        summary["returncode"] = None
        write_inspection_artifacts(directory, summary)
        write_command_artifact(
            directory,
            command=command,
            dry_run=True,
            operation="run-tests",
            expected_artifacts=["test-results.xml", "unity.log"],
        )
        return summary

    stdout_path = directory / "unity-stdout.log"
    stderr_path = directory / "unity-stderr.log"
    started = time.time()
    try:
        completed = subprocess.run(
            command,
            cwd=project_path,
            capture_output=True,
            check=False,
            timeout=args.timeout_seconds,
        )
        duration_seconds = time.time() - started
        stdout_path.parent.mkdir(parents=True, exist_ok=True)
        stdout_path.write_bytes(completed.stdout)
        stderr_path.write_bytes(completed.stderr)
        summary.update(
            {
                "runStatus": "passed" if completed.returncode == 0 else "failed",
                "returncode": completed.returncode,
                "durationSeconds": round(duration_seconds, 3),
                "stdoutPath": str(stdout_path),
                "stderrPath": str(stderr_path),
            }
        )
    except subprocess.TimeoutExpired as error:
        duration_seconds = time.time() - started
        stdout_path.parent.mkdir(parents=True, exist_ok=True)
        if error.stdout:
            stdout_path.write_bytes(error.stdout)
            summary["stdoutPath"] = str(stdout_path)
        if error.stderr:
            stderr_path.write_bytes(error.stderr)
            summary["stderrPath"] = str(stderr_path)
        summary.update(
            {
                "runStatus": "timedOut",
                "returncode": None,
                "durationSeconds": round(duration_seconds, 3),
                "note": f"Unity test command timed out after {args.timeout_seconds} seconds.",
            }
        )

    write_inspection_artifacts(directory, summary)
    write_command_artifact(
        directory,
        command=command,
        dry_run=False,
        operation="run-tests",
        expected_artifacts=["test-results.xml", "unity.log"],
    )
    return summary


def inspect_unity(args: argparse.Namespace) -> dict[str, Any]:
    directory = (
        args.artifact_dir.resolve() if args.artifact_dir else artifact_dir(args.artifact_root, "inspect")
    )
    summary = resolve_unity_editor(args.project_path.resolve())
    summary["artifactPath"] = str(directory)
    summary["operation"] = "inspect"
    write_inspection_artifacts(directory, summary)
    return summary


def bridge_guidance(project_path: Path, repo_root: Path | None = None) -> str:
    repo_root = repo_root or ROOT
    summary = resolve_unity_editor(project_path)
    script_path = repo_root / "tools" / "epiphany_unity_bridge.py"
    inspect_command = (
        f"`python {script_path} inspect --project-path {project_path}`"
    )
    if summary.get("status") == "ready":
        if editor_bridge_ready(summary):
            return (
                f"- Unity bridge: project pins {summary.get('projectVersion')} and the exact editor "
                f"resolved to `{summary.get('editorPath')}`. If Unity execution is needed, use named "
                f"bridge operations such as `python {script_path} probe --project-path {project_path} "
                "--operation scene-facts --scene <Assets/...unity>` or "
                f"`python {script_path} check-compilation --project-path {project_path}`. The bridge "
                "owns -batchmode, -quit, -projectPath, -logFile, -executeMethod, and artifacts. Do not "
                "invoke `Unity`, `Unity.exe`, default installs, or PATH-resolved editors directly, and "
                "do not refactor Unity-owned scene/prefab state as raw text when the editor bridge can "
                "inspect it."
            )
        return (
            f"- Unity bridge: project pins {summary.get('projectVersion')} and the exact editor "
            f"resolved to `{summary.get('editorPath')}`, but the resident editor package is missing. "
            f"Add `{EDITOR_BRIDGE_RELATIVE.as_posix()}` before claiming scene, prefab, or runtime "
            "facts. Do not invoke `Unity`, `Unity.exe`, default installs, or PATH-resolved editors "
            "directly."
        )
    return (
        f"- Unity bridge: {summary.get('note')} Run {inspect_command} to write an auditable "
        "runtime evidence artifact. Do not invoke `Unity`, `Unity.exe`, default installs, or "
        "PATH-resolved editors directly; if runtime parity is needed, stop with the bridge "
        "artifact as the evidence gap."
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Auditable pinned Unity editor bridge.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    inspect_parser = subparsers.add_parser("inspect", help="Resolve the project-pinned editor.")
    inspect_parser.add_argument("--project-path", type=Path, default=Path.cwd())
    inspect_parser.add_argument("--artifact-root", type=Path, default=DEFAULT_ARTIFACT_ROOT)
    inspect_parser.add_argument("--artifact-dir", type=Path)

    run_parser = subparsers.add_parser("run", help="Run an explicit pinned-editor batch command.")
    run_parser.add_argument("--project-path", type=Path, default=Path.cwd())
    run_parser.add_argument("--artifact-root", type=Path, default=DEFAULT_ARTIFACT_ROOT)
    run_parser.add_argument("--artifact-dir", type=Path)
    run_parser.add_argument("--label", default="unity-bridge-run")
    run_parser.add_argument("--timeout-seconds", type=int, default=600)
    run_parser.add_argument("--dry-run", action="store_true")
    run_parser.add_argument("unity_args", nargs=argparse.REMAINDER)

    check_parser = subparsers.add_parser(
        "check-compilation",
        help="Run the resident editor bridge compilation probe.",
    )
    check_parser.add_argument("--project-path", type=Path, default=Path.cwd())
    check_parser.add_argument("--artifact-root", type=Path, default=DEFAULT_ARTIFACT_ROOT)
    check_parser.add_argument("--artifact-dir", type=Path)
    check_parser.add_argument("--timeout-seconds", type=int, default=600)
    check_parser.add_argument("--dry-run", action="store_true")

    probe_parser = subparsers.add_parser(
        "probe",
        help="Run a named resident-editor probe through the pinned editor.",
    )
    probe_parser.add_argument("--project-path", type=Path, default=Path.cwd())
    probe_parser.add_argument("--artifact-root", type=Path, default=DEFAULT_ARTIFACT_ROOT)
    probe_parser.add_argument("--artifact-dir", type=Path)
    probe_parser.add_argument("--timeout-seconds", type=int, default=600)
    probe_parser.add_argument("--dry-run", action="store_true")
    probe_parser.add_argument("--operation", choices=sorted(NAMED_PROBE_OPERATIONS), required=True)
    probe_parser.add_argument("--scene")
    probe_parser.add_argument("--asset")
    probe_parser.add_argument("--guid")
    probe_parser.add_argument("--max-objects", type=int)
    probe_parser.add_argument("--max-properties", type=int)
    probe_parser.add_argument("--max-dependencies", type=int)

    test_parser = subparsers.add_parser(
        "run-tests",
        help="Run Unity editmode/playmode tests through the pinned editor.",
    )
    test_parser.add_argument("--project-path", type=Path, default=Path.cwd())
    test_parser.add_argument("--artifact-root", type=Path, default=DEFAULT_ARTIFACT_ROOT)
    test_parser.add_argument("--artifact-dir", type=Path)
    test_parser.add_argument("--timeout-seconds", type=int, default=900)
    test_parser.add_argument("--dry-run", action="store_true")
    test_parser.add_argument("--platform", choices=["editmode", "playmode"], default="editmode")
    test_parser.add_argument("--filter")

    args = parser.parse_args()
    try:
        if args.command == "inspect":
            result = inspect_unity(args)
        elif args.command == "run":
            result = run_unity(args)
        elif args.command == "check-compilation":
            result = check_compilation(args)
        elif args.command == "probe":
            result = probe_unity(args)
        elif args.command == "run-tests":
            result = test_unity(args)
        else:
            raise ValueError(f"Unsupported command: {args.command}")
    except Exception as error:
        print(json.dumps({"status": "error", "error": str(error)}, indent=2), flush=True)
        return 1
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0 if result.get("runStatus") != "blocked" else 2


if __name__ == "__main__":
    raise SystemExit(main())
