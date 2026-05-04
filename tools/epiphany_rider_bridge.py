from __future__ import annotations

import argparse
from datetime import datetime
from datetime import timezone
import json
import os
from pathlib import Path
import shutil
import subprocess
import time
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_ARTIFACT_ROOT = ROOT / ".epiphany-gui" / "rider"
RIDER_EXECUTABLE_NAMES = ("rider64.exe", "rider.exe", "rider.bat", "rider")


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def write_text(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value, encoding="utf-8")


def artifact_dir(root: Path, action: str) -> Path:
    return root.resolve() / f"rider-{action}-{time.time_ns()}-{os.getpid()}"


def maybe_resolve(path: Path) -> str:
    try:
        return str(path.resolve())
    except OSError:
        return str(path)


def is_inside(path: Path, root: Path) -> bool:
    try:
        path.resolve().relative_to(root.resolve())
        return True
    except ValueError:
        return False


def relative_or_absolute(path: Path, root: Path) -> str:
    try:
        return str(path.resolve().relative_to(root.resolve())).replace("\\", "/")
    except ValueError:
        return str(path)


def dedupe_paths(paths: list[Path]) -> list[Path]:
    seen: set[str] = set()
    result: list[Path] = []
    for path in paths:
        key = str(path.resolve()) if path.exists() else str(path)
        if key.lower() in seen:
            continue
        seen.add(key.lower())
        result.append(path)
    return result


def rider_search_roots() -> list[Path]:
    roots: list[Path] = []
    env_roots = os.environ.get("EPIPHANY_RIDER_ROOTS")
    if env_roots:
        roots.extend(Path(item) for item in env_roots.split(os.pathsep) if item)
    for variable, fallback in (
        ("ProgramFiles", r"C:\Program Files"),
        ("ProgramFiles(x86)", r"C:\Program Files (x86)"),
        ("LOCALAPPDATA", str(Path.home() / "AppData" / "Local")),
    ):
        roots.append(Path(os.environ.get(variable, fallback)) / "JetBrains")
    return dedupe_paths(roots)


def discover_rider_installations() -> list[dict[str, Any]]:
    candidates: list[Path] = []
    env_path = os.environ.get("EPIPHANY_RIDER_PATH")
    if env_path:
        candidates.append(Path(env_path))
    for name in RIDER_EXECUTABLE_NAMES:
        found = shutil.which(name)
        if found:
            candidates.append(Path(found))
    for root in rider_search_roots():
        if not root.exists():
            continue
        for directory in root.glob("JetBrains Rider*"):
            for name in RIDER_EXECUTABLE_NAMES:
                candidates.append(directory / "bin" / name)

    installations: list[dict[str, Any]] = []
    for path in dedupe_paths(candidates):
        exists = path.exists()
        if not exists:
            continue
        version_hint = None
        for part in reversed(path.parts):
            if "Rider" in part:
                version_hint = part
                break
        installations.append(
            {
                "path": maybe_resolve(path),
                "exists": exists,
                "versionHint": version_hint,
            }
        )
    return installations


def discover_solution(project_root: Path) -> dict[str, Any]:
    root = project_root.resolve()
    top_level = sorted(root.glob("*.sln"))
    if top_level:
        return {
            "status": "ready",
            "path": str(top_level[0]),
            "candidates": [str(path) for path in top_level[:8]],
        }

    candidates: list[Path] = []
    for path in root.rglob("*.sln"):
        if any(
            part.startswith(".") or part in {"Library", "Temp", "obj", "bin", "node_modules"}
            for part in path.relative_to(root).parts
        ):
            continue
        candidates.append(path)
        if len(candidates) >= 8:
            break
    return {
        "status": "ready" if candidates else "missingSolution",
        "path": str(candidates[0]) if candidates else None,
        "candidates": [str(path) for path in candidates],
    }


def run_git(project_root: Path, args: list[str]) -> str | None:
    try:
        completed = subprocess.run(
            ["git", "-C", str(project_root), *args],
            capture_output=True,
            check=False,
            encoding="utf-8",
            errors="replace",
            timeout=10,
        )
    except (OSError, subprocess.TimeoutExpired):
        return None
    if completed.returncode != 0:
        return None
    return completed.stdout.strip()


def vcs_summary(project_root: Path) -> dict[str, Any]:
    branch = run_git(project_root, ["rev-parse", "--abbrev-ref", "HEAD"])
    status = run_git(project_root, ["status", "--porcelain=v1"])
    changed_files = run_git(project_root, ["diff", "--name-only"])
    staged_files = run_git(project_root, ["diff", "--cached", "--name-only"])
    visible_changed = changed_files.splitlines() if changed_files else []
    if status:
        for line in status.splitlines():
            path = line[3:].strip()
            if path and path not in visible_changed:
                visible_changed.append(path)
    return {
        "status": "ready" if branch is not None else "notGit",
        "branch": branch,
        "dirty": bool(status),
        "changedFiles": visible_changed,
        "stagedFiles": staged_files.splitlines() if staged_files else [],
        "changedRangesKnown": branch is not None,
    }


def render_status(summary: dict[str, Any]) -> str:
    lines = [
        "# Rider Bridge Status",
        "",
        f"- status: {summary.get('status')}",
        f"- workspace: `{summary.get('workspace')}`",
        f"- solution: `{summary.get('solutionPath') or 'none'}`",
        f"- rider: `{summary.get('riderPath') or 'missing'}`",
        f"- installations: {summary.get('installationCount')}",
        f"- branch: {summary.get('vcs', {}).get('branch') or 'unknown'}",
        f"- dirty: {summary.get('vcs', {}).get('dirty')}",
        f"- note: {summary.get('note')}",
        "",
        "Rider is a source-context organ. This artifact is an operator-safe projection, not durable Epiphany truth.",
    ]
    return "\n".join(lines) + "\n"


def render_context(packet: dict[str, Any], summary: dict[str, Any]) -> str:
    symbol = packet.get("symbol") or {}
    selection = packet.get("selection") or {}
    lines = [
        "# Rider Context Packet",
        "",
        f"- status: {summary.get('status')}",
        f"- project: `{packet.get('projectRoot')}`",
        f"- solution: `{packet.get('solutionPath') or 'none'}`",
        f"- file: `{packet.get('filePath') or 'none'}`",
        f"- selection: {selection.get('startLine') or 'none'}-{selection.get('endLine') or 'none'}",
        f"- symbol: {symbol.get('name') or 'none'}",
        f"- artifact: `{summary.get('artifactPath')}`",
        "",
        "Modeling may use this packet as scratch/context. It is not accepted map state until reviewed.",
    ]
    return "\n".join(lines) + "\n"


def bridge_guidance(project_root: Path, repo_root: Path | None = None) -> str:
    repo_root = repo_root or ROOT
    script_path = repo_root / "tools" / "epiphany_rider_bridge.py"
    installations = discover_rider_installations()
    solution = discover_solution(project_root)
    status_command = f"`python {script_path} status --project-root {project_root}`"
    if installations:
        solution_text = solution.get("path") or "no solution found yet"
        return (
            f"- Rider bridge: Rider is available at `{installations[0].get('path')}` and solution "
            f"context is `{solution_text}`. Use {status_command} for an auditable source/IDE "
            "status receipt, and use `context --file <path> --selection-start <line> --selection-end "
            "<line> --symbol-name <name>` when the future plugin or operator sends a bounded source "
            "slice. Rider facts are scratch/context until modeling or verification accepts them."
        )
    return (
        f"- Rider bridge: no Rider executable was found. Use {status_command} to write an "
        "auditable missing-IDE artifact, or set EPIPHANY_RIDER_PATH. Do not pretend IDE "
        "diagnostics, changed ranges, or source navigation were captured until the bridge has a "
        "receipt."
    )


def status_rider(args: argparse.Namespace) -> dict[str, Any]:
    project_root = args.project_root.resolve()
    directory = args.artifact_dir.resolve() if args.artifact_dir else artifact_dir(args.artifact_root, "inspect")
    installations = discover_rider_installations()
    solution = discover_solution(project_root)
    first_rider = installations[0] if installations else None
    status = "ready" if first_rider else "missingRider"
    note = (
        "Rider executable is available for explicit source navigation and context capture."
        if first_rider
        else "No Rider executable was found. Install Rider or set EPIPHANY_RIDER_PATH."
    )
    summary: dict[str, Any] = {
        "kind": "riderBridgeStatus",
        "status": status,
        "workspace": str(project_root),
        "solutionPath": solution.get("path"),
        "solutionStatus": solution.get("status"),
        "solutionCandidates": solution.get("candidates", []),
        "riderPath": first_rider.get("path") if first_rider else None,
        "installationCount": len(installations),
        "installations": installations,
        "searchRoots": [str(path) for path in rider_search_roots()],
        "vcs": vcs_summary(project_root),
        "capturedAt": now_iso(),
        "artifactPath": str(directory),
        "note": note,
    }
    write_json(directory / "rider-bridge-summary.json", summary)
    write_text(directory / "rider-bridge-status.md", render_status(summary))
    return summary


def packet_from_args(args: argparse.Namespace, project_root: Path) -> dict[str, Any]:
    if args.packet:
        packet = json.loads(args.packet.read_text(encoding="utf-8"))
        if not isinstance(packet, dict):
            raise ValueError("context packet must be a JSON object")
        return packet

    solution = discover_solution(project_root)
    file_path = args.file.resolve() if args.file else None
    if file_path and not is_inside(file_path, project_root):
        raise ValueError(f"context file is outside project root: {file_path}")
    packet: dict[str, Any] = {
        "kind": "riderContext",
        "capturedAt": now_iso(),
        "projectRoot": str(project_root),
        "solutionPath": solution.get("path"),
        "filePath": relative_or_absolute(file_path, project_root) if file_path else None,
        "caret": {
            "line": args.line,
            "column": args.column,
        }
        if args.line
        else None,
        "selection": {
            "startLine": args.selection_start,
            "endLine": args.selection_end,
        }
        if args.selection_start or args.selection_end
        else None,
        "symbol": {
            "name": args.symbol_name,
            "kind": args.symbol_kind,
            "namespace": args.symbol_namespace,
        }
        if args.symbol_name
        else None,
        "vcs": vcs_summary(project_root),
    }
    return packet


def context_rider(args: argparse.Namespace) -> dict[str, Any]:
    project_root = args.project_root.resolve()
    directory = args.artifact_dir.resolve() if args.artifact_dir else artifact_dir(args.artifact_root, "context")
    packet = packet_from_args(args, project_root)
    summary = status_rider(
        argparse.Namespace(
            project_root=project_root,
            artifact_root=args.artifact_root,
            artifact_dir=directory,
        )
    )
    summary.update(
        {
            "kind": "riderContextCapture",
            "status": "captured",
            "contextPath": str(directory / "rider-context.json"),
            "filePath": packet.get("filePath"),
            "symbol": packet.get("symbol"),
        }
    )
    write_json(directory / "rider-context.json", packet)
    write_json(directory / "rider-bridge-summary.json", summary)
    write_text(directory / "rider-context.md", render_context(packet, summary))
    return summary


def open_ref_rider(args: argparse.Namespace) -> dict[str, Any]:
    project_root = args.project_root.resolve()
    directory = args.artifact_dir.resolve() if args.artifact_dir else artifact_dir(args.artifact_root, "open-ref")
    status = status_rider(
        argparse.Namespace(
            project_root=project_root,
            artifact_root=args.artifact_root,
            artifact_dir=directory,
        )
    )
    file_path = args.file.resolve()
    if not is_inside(file_path, project_root):
        raise ValueError(f"code ref is outside project root: {file_path}")
    rider_path = status.get("riderPath")
    command = [rider_path, str(file_path)] if rider_path else []
    if rider_path and args.line:
        command.extend(["--line", str(args.line)])
    summary = {
        **status,
        "kind": "riderOpenRef",
        "status": "planned" if rider_path else "missingRider",
        "filePath": str(file_path),
        "line": args.line,
        "column": args.column,
        "command": command,
        "launched": False,
    }
    if rider_path and args.launch:
        subprocess.Popen(command, cwd=project_root)
        summary["status"] = "launched"
        summary["launched"] = True
    write_json(directory / "rider-open-ref.json", summary)
    write_json(directory / "rider-bridge-summary.json", summary)
    write_text(directory / "rider-bridge-status.md", render_status(summary))
    return summary


def main() -> int:
    parser = argparse.ArgumentParser(description="Operator-safe Rider source context bridge.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    status_parser = subparsers.add_parser("status", help="Inspect Rider and source workspace status.")
    status_parser.add_argument("--project-root", type=Path, default=Path.cwd())
    status_parser.add_argument("--artifact-root", type=Path, default=DEFAULT_ARTIFACT_ROOT)
    status_parser.add_argument("--artifact-dir", type=Path)

    context_parser = subparsers.add_parser("context", help="Capture a Rider-style source context packet.")
    context_parser.add_argument("--project-root", type=Path, default=Path.cwd())
    context_parser.add_argument("--artifact-root", type=Path, default=DEFAULT_ARTIFACT_ROOT)
    context_parser.add_argument("--artifact-dir", type=Path)
    context_parser.add_argument("--packet", type=Path)
    context_parser.add_argument("--file", type=Path)
    context_parser.add_argument("--line", type=int)
    context_parser.add_argument("--column", type=int)
    context_parser.add_argument("--selection-start", type=int)
    context_parser.add_argument("--selection-end", type=int)
    context_parser.add_argument("--symbol-name")
    context_parser.add_argument("--symbol-kind")
    context_parser.add_argument("--symbol-namespace")

    open_parser = subparsers.add_parser("open-ref", help="Plan or launch Rider at one code ref.")
    open_parser.add_argument("--project-root", type=Path, default=Path.cwd())
    open_parser.add_argument("--artifact-root", type=Path, default=DEFAULT_ARTIFACT_ROOT)
    open_parser.add_argument("--artifact-dir", type=Path)
    open_parser.add_argument("--file", type=Path, required=True)
    open_parser.add_argument("--line", type=int)
    open_parser.add_argument("--column", type=int)
    open_parser.add_argument("--launch", action="store_true")

    args = parser.parse_args()
    try:
        if args.command == "status":
            result = status_rider(args)
        elif args.command == "context":
            result = context_rider(args)
        elif args.command == "open-ref":
            result = open_ref_rider(args)
        else:
            raise ValueError(f"Unsupported command: {args.command}")
    except Exception as error:
        print(json.dumps({"status": "error", "error": str(error)}, indent=2), flush=True)
        return 1
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
