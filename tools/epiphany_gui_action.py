from __future__ import annotations

import argparse
import hashlib
from copy import deepcopy
from datetime import datetime
from datetime import timezone
import json
import os
import subprocess
import time
import tomllib
from pathlib import Path
from typing import Any

from epiphany_agent_telemetry import write_transcript_telemetry
from epiphany_mvp_status import DEFAULT_APP_SERVER
from epiphany_mvp_status import collect_status
from epiphany_mvp_status import render_status
from epiphany_mvp_status import sanitize_for_operator
from epiphany_phase5_smoke import AppServerClient
from epiphany_phase5_smoke import ROOT
from epiphany_phase6_reorient_launch_smoke import BINDING_ID as REORIENT_BINDING_ID
from epiphany_unity_bridge import bridge_guidance as unity_bridge_guidance


DEFAULT_CODEX_HOME = ROOT / ".epiphany-gui" / "codex-home"
DEFAULT_ARTIFACT_ROOT = ROOT / ".epiphany-gui" / "actions"
EPIPHANY_PROMPTS_PATH = (
    ROOT
    / "vendor"
    / "codex"
    / "codex-rs"
    / "app-server"
    / "src"
    / "prompts"
    / "epiphany_specialists.toml"
)
TERMINAL_ROLE_STATUSES = {"completed", "failed", "cancelled"}
TERMINAL_REORIENT_STATUSES = {"completed", "failed", "cancelled"}


def load_epiphany_prompt_config() -> dict[str, Any]:
    with EPIPHANY_PROMPTS_PATH.open("rb") as handle:
        config = tomllib.load(handle)
    if not isinstance(config, dict):
        raise ValueError(f"invalid Epiphany prompt config: {EPIPHANY_PROMPTS_PATH}")
    return config


def implementation_continue_template() -> str:
    config = load_epiphany_prompt_config()
    template = nested_get(config, "implementation", "continue_template")
    if not isinstance(template, str) or not template.strip():
        raise ValueError(
            "missing [implementation].continue_template in "
            f"{EPIPHANY_PROMPTS_PATH}"
        )
    return template.strip()


def prompt_text(value: Any, default: str = "none") -> str:
    if value is None:
        return default
    text = str(value)
    if not text.strip():
        return default
    return text


def run_git(cwd: Path, *args: str) -> dict[str, Any]:
    try:
        completed = subprocess.run(
            ["git", *args],
            cwd=cwd,
            capture_output=True,
            check=False,
            encoding="utf-8",
            errors="replace",
        )
    except OSError as error:
        return {"ok": False, "error": str(error), "args": list(args)}
    return {
        "ok": completed.returncode == 0,
        "returncode": completed.returncode,
        "args": list(args),
        "stdout": completed.stdout.strip(),
        "stderr": completed.stderr.strip(),
    }


def git_diff_hash(cwd: Path) -> dict[str, Any]:
    diff = run_git(cwd, "diff", "--binary")
    if not diff.get("ok"):
        return diff
    text = str(diff.get("stdout") or "")
    return {
        "ok": True,
        "sha256": hashlib.sha256(text.encode("utf-8", errors="replace")).hexdigest(),
        "bytes": len(text.encode("utf-8", errors="replace")),
    }


def git_snapshot(cwd: Path) -> dict[str, Any]:
    return {
        "head": run_git(cwd, "rev-parse", "--short", "HEAD"),
        "branch": run_git(cwd, "branch", "--show-current"),
        "status": run_git(cwd, "status", "--short", "--branch"),
        "changedFiles": run_git(cwd, "diff", "--name-only"),
        "untrackedFiles": run_git(cwd, "ls-files", "--others", "--exclude-standard"),
        "diffStat": run_git(cwd, "diff", "--stat"),
        "diffHash": git_diff_hash(cwd),
    }


def split_git_lines(value: Any) -> list[str]:
    text = str(value or "").strip()
    return [line for line in text.splitlines() if line.strip()]


def normalize_repo_path(path: str) -> str:
    return path.strip().replace("\\", "/")


def parse_review_file_note(note: str) -> list[str]:
    if ":" not in note:
        return []
    _, raw_paths = note.split(":", 1)
    paths: list[str] = []
    for raw_path in raw_paths.split(","):
        path = normalize_repo_path(raw_path)
        if not path or path.lower() in {"none", "unknown files"} or "*" in path:
            continue
        paths.append(path)
    return paths


def parent_path(path: str) -> str:
    normalized = normalize_repo_path(path)
    if "/" not in normalized:
        return ""
    return normalized.rsplit("/", 1)[0]


def is_under(path: str, root: str) -> bool:
    normalized = normalize_repo_path(path)
    if not root:
        return "/" not in normalized
    return normalized == root or normalized.startswith(f"{root}/")


def review_untracked_files(
    *,
    changed_files: list[str],
    new_untracked_files: list[str],
    after_untracked: set[str],
    active_review_files: list[str] | None = None,
) -> list[str]:
    active_review_file_set = {
        normalize_repo_path(path)
        for path in active_review_files or []
        if normalize_repo_path(path)
    }
    review_roots = {
        parent_path(path)
        for path in [*changed_files, *new_untracked_files, *active_review_file_set]
        if path
    }
    if not review_roots:
        return sorted(new_untracked_files)
    return sorted(
        path
        for path in after_untracked
        if path in new_untracked_files
        or path in active_review_file_set
        or any(is_under(path, root) for root in review_roots)
    )


def git_change_summary(
    before: dict[str, Any],
    after: dict[str, Any],
    *,
    active_review_files: list[str] | None = None,
) -> dict[str, Any]:
    before_status = set(str(nested_get(before, "status", "stdout") or "").splitlines())
    after_status = set(str(nested_get(after, "status", "stdout") or "").splitlines())
    added_status = sorted(after_status - before_status)
    removed_status = sorted(before_status - after_status)
    before_changed_files = split_git_lines(nested_get(before, "changedFiles", "stdout"))
    changed_files = split_git_lines(nested_get(after, "changedFiles", "stdout"))
    before_untracked = set(split_git_lines(nested_get(before, "untrackedFiles", "stdout")))
    after_untracked = set(split_git_lines(nested_get(after, "untrackedFiles", "stdout")))
    new_untracked_files = sorted(after_untracked - before_untracked)
    active_review_files = sorted(
        {
            normalize_repo_path(path)
            for path in active_review_files or []
            if normalize_repo_path(path)
        }
    )
    untracked_files = review_untracked_files(
        changed_files=changed_files,
        new_untracked_files=new_untracked_files,
        after_untracked=after_untracked,
        active_review_files=active_review_files,
    )
    workspace_files = sorted({*changed_files, *untracked_files})
    ignored_untracked_count = max(0, len(after_untracked) - len(set(untracked_files)))
    before_diff_stat = str(nested_get(before, "diffStat", "stdout") or "").strip()
    diff_stat = str(nested_get(after, "diffStat", "stdout") or "").strip()
    before_diff_hash = nested_get(before, "diffHash", "sha256")
    after_diff_hash = nested_get(after, "diffHash", "sha256")
    if isinstance(before_diff_hash, str) and isinstance(after_diff_hash, str):
        tracked_diff_changed = before_diff_hash != after_diff_hash
    else:
        tracked_diff_changed = before_changed_files != changed_files or before_diff_stat != diff_stat
    workspace_changed = bool(
        added_status or removed_status or new_untracked_files or tracked_diff_changed
    )
    return {
        "workspaceChanged": workspace_changed,
        "dirtyWorkspace": bool(changed_files or after_untracked or diff_stat),
        "trackedDiffPresent": bool(changed_files or diff_stat),
        "trackedDiffChanged": tracked_diff_changed,
        "beforeChangedFiles": before_changed_files,
        "changedFiles": changed_files,
        "untrackedFiles": untracked_files,
        "newUntrackedFiles": new_untracked_files,
        "activeReviewFiles": active_review_files,
        "workspaceFiles": workspace_files,
        "ignoredUntrackedFileCount": ignored_untracked_count,
        "statusAdded": added_status,
        "statusRemoved": removed_status,
        "diffStat": diff_stat,
    }


def render_implementation_audit(result: dict[str, Any]) -> str:
    changed_files = result.get("changedFiles")
    if not isinstance(changed_files, list):
        changed_files = []
    status_added = result.get("statusAdded")
    if not isinstance(status_added, list):
        status_added = []
    status_removed = result.get("statusRemoved")
    if not isinstance(status_removed, list):
        status_removed = []
    untracked_files = result.get("untrackedFiles")
    if not isinstance(untracked_files, list):
        untracked_files = []
    new_untracked_files = result.get("newUntrackedFiles")
    if not isinstance(new_untracked_files, list):
        new_untracked_files = []
    active_review_files = result.get("activeReviewFiles")
    if not isinstance(active_review_files, list):
        active_review_files = []
    ignored_untracked_count = result.get("ignoredUntrackedFileCount")
    if not isinstance(ignored_untracked_count, int):
        ignored_untracked_count = 0
    workspace_files = result.get("workspaceFiles")
    if not isinstance(workspace_files, list):
        workspace_files = []

    outcome = (
        "reviewable new workspace diff"
        if result.get("workspaceChanged")
        else "no new workspace diff"
    )
    next_action = (
        "Review the changed files before accepting this implementation slice."
        if result.get("workspaceChanged")
        else "Stop and review this as an implementation-lane failure before rerunning; the worker completed without changing the target workspace further."
    )
    return "\n".join(
        [
            "# Implementation Audit",
            "",
            f"Outcome: {outcome}",
            f"Dirty workspace present: {bool(result.get('dirtyWorkspace'))}",
            f"Tracked diff present: {bool(result.get('trackedDiffPresent'))}",
            f"Tracked diff changed this turn: {bool(result.get('trackedDiffChanged'))}",
            "",
            "Changed files:",
            *(f"- {path}" for path in changed_files),
            *([] if changed_files else ["- none"]),
            "",
            "Untracked files:",
            *(f"- {path}" for path in untracked_files),
            *([] if untracked_files else ["- none"]),
            "",
            "New untracked files this turn:",
            *(f"- {path}" for path in new_untracked_files),
            *([] if new_untracked_files else ["- none"]),
            "",
            "Carried review files from Epiphany scratch:",
            *(f"- {path}" for path in active_review_files),
            *([] if active_review_files else ["- none"]),
            "",
            f"Ignored unrelated untracked files: {ignored_untracked_count}",
            "",
            "Workspace files needing review:",
            *(f"- {path}" for path in workspace_files),
            *([] if workspace_files else ["- none"]),
            "",
            "Git status delta:",
            *(f"- added: {line}" for line in status_added),
            *(f"- removed: {line}" for line in status_removed),
            *([] if status_added or status_removed else ["- none"]),
            "",
            f"Next action: {next_action}",
            "",
        ]
    )


def nested_get(value: dict[str, Any], *keys: str) -> Any:
    current: Any = value
    for key in keys:
        if not isinstance(current, dict):
            return None
        current = current.get(key)
    return current


def active_review_files_from_status(status: dict[str, Any]) -> list[str]:
    state = nested_get(status, "read", "thread", "epiphanyState")
    if not isinstance(state, dict):
        return []
    scratch = state.get("scratch")
    if not isinstance(scratch, dict):
        return []
    notes = scratch.get("notes")
    if not isinstance(notes, list):
        return []
    files: list[str] = []
    for note in notes:
        if not isinstance(note, str):
            continue
        if note.startswith(
            (
                "Tracked files:",
                "Review untracked files:",
                "Untracked files:",
                "New untracked files this turn:",
            )
        ):
            files.extend(parse_review_file_note(note))
    return sorted(set(files))


def unity_guidance(cwd: Path) -> str:
    return unity_bridge_guidance(cwd, ROOT)


def first_present(value: dict[str, Any], *paths: tuple[str, ...]) -> Any:
    for path in paths:
        item = nested_get(value, *path)
        if item not in (None, "", []):
            return item
    return None


def summarized_recent_evidence(state: dict[str, Any], limit: int = 8) -> list[str]:
    recent = state.get("recent_evidence")
    if not isinstance(recent, list):
        return []
    lines: list[str] = []
    for item in recent[:limit]:
        if not isinstance(item, dict):
            continue
        summary = item.get("summary")
        if not isinstance(summary, str) or not summary.strip():
            continue
        evidence_id = item.get("id", "unknown")
        kind = item.get("kind", "unknown")
        status = item.get("status", "unknown")
        lines.append(f"- {evidence_id} [{kind}/{status}]: {summary}")
    return lines


def build_implementation_prompt(status: dict[str, Any], cwd: Path) -> str:
    operator_status = sanitize_for_operator(status)
    state = first_present(
        operator_status,
        ("read", "thread", "epiphanyState"),
        ("scene", "scene", "epiphanyState"),
    )
    if not isinstance(state, dict):
        state = {}
    coordinator = operator_status.get("coordinator")
    if not isinstance(coordinator, dict):
        coordinator = {}
    crrc = nested_get(operator_status, "crrc", "decision")
    if not isinstance(crrc, dict):
        crrc = {}

    checkpoint = first_present(
        state,
        ("graph_checkpoint",),
        ("graphCheckpoint",),
    )
    if not isinstance(checkpoint, dict):
        checkpoint = {}
    frontier = first_present(state, ("graph_frontier",), ("graphFrontier",))
    if not isinstance(frontier, dict):
        frontier = {}
    scratch = state.get("scratch")
    if not isinstance(scratch, dict):
        scratch = {}

    evidence_lines = summarized_recent_evidence(state)
    if not evidence_lines:
        evidence_lines = ["- none recorded"]

    active_node_ids = frontier.get("active_node_ids") or frontier.get("activeNodeIds") or []
    if isinstance(active_node_ids, list):
        active_node_text = ", ".join(str(item) for item in active_node_ids)
    else:
        active_node_text = str(active_node_ids or "none")

    template = implementation_continue_template()
    return template.format(
        coordinator_action=prompt_text(coordinator.get("action")),
        coordinator_target_role=prompt_text(coordinator.get("targetRole")),
        coordinator_reason=prompt_text(coordinator.get("reason")),
        checkpoint_id=prompt_text(
            checkpoint.get("checkpoint_id") or checkpoint.get("checkpointId")
        ),
        checkpoint_summary=prompt_text(checkpoint.get("summary")),
        frontier_active_nodes=prompt_text(active_node_text),
        continuity_next_action=prompt_text(
            crrc.get("nextAction")
            or nested_get(operator_status, "reorient", "decision", "nextAction")
        ),
        scratch_summary=prompt_text(scratch.get("summary")),
        accepted_evidence="\n".join(evidence_lines),
        unity_guidance=unity_guidance(cwd),
    )


def implementation_no_diff_patch(
    status: dict[str, Any],
    *,
    artifact_dir: Path,
    implementation_result: dict[str, Any],
) -> dict[str, Any] | None:
    operator_status = sanitize_for_operator(status)
    state = first_present(
        operator_status,
        ("read", "thread", "epiphanyState"),
        ("scene", "scene", "epiphanyState"),
    )
    if not isinstance(state, dict):
        return None
    checkpoint = first_present(
        state,
        ("investigation_checkpoint",),
        ("investigationCheckpoint",),
    )
    if not isinstance(checkpoint, dict):
        return None

    evidence_id = f"ev-implementation-no-diff-{artifact_dir.name}"
    observation_id = f"obs-implementation-no-diff-{artifact_dir.name}"
    updated_checkpoint = deepcopy(checkpoint)
    updated_checkpoint["disposition"] = "regather_required"
    updated_checkpoint["summary"] = (
        "Implementation turn completed with no new workspace diff; the current "
        "checkpoint is insufficient to drive the coding lane safely."
    )
    updated_checkpoint["next_action"] = (
        "Review the no-diff implementation audit, then repair the implementation "
        "lane through modeling or reorientation before continuing code work."
    )
    evidence_ids = updated_checkpoint.get("evidence_ids")
    if not isinstance(evidence_ids, list):
        evidence_ids = []
    if evidence_id not in evidence_ids:
        evidence_ids = [*evidence_ids, evidence_id]
    updated_checkpoint["evidence_ids"] = evidence_ids

    return {
        "investigationCheckpoint": updated_checkpoint,
        "observations": [
            {
                "id": observation_id,
                "summary": (
                    "The implementation lane completed a bounded turn but left "
                    "no new tracked or untracked workspace delta, so continuation "
                    "needs review or checkpoint repair."
                ),
                "source_kind": "implementation",
                "status": "blocked",
                "evidence_ids": [evidence_id],
            }
        ],
        "evidence": [
            {
                "id": evidence_id,
                "kind": "implementation-audit",
                "status": "blocked",
                "summary": (
                    "continueImplementation produced no new workspace diff "
                    f"(trackedDiffPresent={bool(implementation_result.get('trackedDiffPresent'))}); "
                    f"artifact={artifact_dir.name}."
                ),
            }
        ],
    }


def implementation_diff_patch(
    *,
    artifact_dir: Path,
    implementation_result: dict[str, Any],
) -> dict[str, Any]:
    evidence_id = f"ev-implementation-diff-{artifact_dir.name}"
    observation_id = f"obs-implementation-diff-{artifact_dir.name}"
    workspace_files = implementation_result.get("workspaceFiles")
    if not isinstance(workspace_files, list):
        workspace_files = []
    changed_files = implementation_result.get("changedFiles")
    if not isinstance(changed_files, list):
        changed_files = []
    untracked_files = implementation_result.get("untrackedFiles")
    if not isinstance(untracked_files, list):
        untracked_files = []
    new_untracked_files = implementation_result.get("newUntrackedFiles")
    if not isinstance(new_untracked_files, list):
        new_untracked_files = []
    ignored_untracked_count = implementation_result.get("ignoredUntrackedFileCount")
    if not isinstance(ignored_untracked_count, int):
        ignored_untracked_count = 0

    file_text = ", ".join(str(path) for path in workspace_files[:12]) or "unknown files"
    code_refs = [
        {
            "path": str(path),
            "note": "Changed by bounded continueImplementation evidence-gathering turn.",
        }
        for path in workspace_files[:20]
    ]
    summary = (
        "continueImplementation produced a reviewable workspace diff "
        f"(trackedDiffPresent={bool(implementation_result.get('trackedDiffPresent'))}); "
        f"artifact={artifact_dir.name}; files={file_text}."
    )
    return {
        "observations": [
            {
                "id": observation_id,
                "summary": (
                    "The implementation lane produced a bounded workspace diff "
                    "from the current accepted modeling checkpoint; verification "
                    "should review the changed files before implementation continues."
                ),
                "source_kind": "implementation",
                "status": "accepted",
                "evidence_ids": [evidence_id],
                "code_refs": code_refs,
            }
        ],
        "evidence": [
            {
                "id": evidence_id,
                "kind": "implementation-audit",
                "status": "ok",
                "summary": summary,
                "code_refs": code_refs,
            }
        ],
        "scratch": {
            "summary": (
                "Implementation produced a reviewable diff; next safe move is "
                "verification of the changed workspace files, including untracked files."
            ),
            "next_probe": "Run verification/review against the implementation diff before continuing.",
            "notes": [
                f"Tracked files: {', '.join(str(path) for path in changed_files) or 'none'}",
                f"Untracked files: {', '.join(str(path) for path in untracked_files) or 'none'}",
                f"New untracked files this turn: {', '.join(str(path) for path in new_untracked_files) or 'none'}",
                f"Ignored unrelated untracked files: {ignored_untracked_count}",
            ],
        },
    }


def wait_for_role_result(
    client: AppServerClient,
    *,
    thread_id: str,
    role_id: str,
    timeout_seconds: int,
    poll_seconds: float,
) -> dict[str, Any]:
    deadline = time.time() + timeout_seconds
    latest: dict[str, Any] | None = None
    while time.time() < deadline:
        latest = client.send(
            "thread/epiphany/roleResult",
            {"threadId": thread_id, "roleId": role_id},
        )
        assert latest is not None
        if latest.get("status") in TERMINAL_ROLE_STATUSES:
            return latest
        time.sleep(poll_seconds)
    assert latest is not None
    return latest


def wait_for_reorient_result(
    client: AppServerClient,
    *,
    thread_id: str,
    timeout_seconds: int,
    poll_seconds: float,
) -> dict[str, Any]:
    deadline = time.time() + timeout_seconds
    latest: dict[str, Any] | None = None
    while time.time() < deadline:
        latest = client.send(
            "thread/epiphany/reorientResult",
            {"threadId": thread_id, "bindingId": REORIENT_BINDING_ID},
        )
        assert latest is not None
        if latest.get("status") in TERMINAL_REORIENT_STATUSES:
            return latest
        time.sleep(poll_seconds)
    assert latest is not None
    return latest


def checkpoint_patch(cwd: Path) -> dict[str, Any]:
    generated_at = datetime.now(timezone.utc).isoformat()
    return {
        "objective": "Reach a testable Epiphany MVP through the local GUI/operator workflow.",
        "activeSubgoalId": "epiphany-gui-mvp",
        "subgoals": [
            {
                "id": "epiphany-gui-mvp",
                "title": "Make the Epiphany GUI usable for local operator testing",
                "status": "active",
                "summary": "The operator can prepare durable state, inspect coordinator guidance, and launch/read fixed specialist lanes without terminal work.",
            }
        ],
        "graphs": {
            "architecture": {
                "nodes": [
                    {
                        "id": "gui-operator-console",
                        "title": "Tauri/React operator console",
                        "purpose": "Human-facing surface for status, artifacts, and bounded lane actions.",
                        "code_refs": [
                            {
                                "path": "apps/epiphany-gui/src/App.tsx",
                                "start_line": 1,
                                "end_line": 1,
                                "symbol": "App",
                            }
                        ],
                    },
                    {
                        "id": "gui-action-bridge",
                        "title": "GUI action bridge",
                        "purpose": "Python bridge that invokes fixed app-server Epiphany lane APIs and writes auditable artifacts.",
                        "code_refs": [
                            {
                                "path": "tools/epiphany_gui_action.py",
                                "start_line": 1,
                                "end_line": 1,
                                "symbol": "run_action",
                            }
                        ],
                    },
                    {
                        "id": "app-server-epiphany-control-plane",
                        "title": "App-server Epiphany control plane",
                        "purpose": "Typed state, coordinator, CRRC, role, reorient, and job surfaces used by the GUI.",
                        "code_refs": [
                            {
                                "path": "vendor/codex/codex-rs/app-server/src/codex_message_processor.rs",
                                "start_line": 1,
                                "end_line": 1,
                                "symbol": "thread/epiphany",
                            }
                        ],
                    },
                ]
            },
            "dataflow": {
                "nodes": [
                    {
                        "id": "operator-action-flow",
                        "title": "Operator action flow",
                        "purpose": "GUI button -> Tauri command -> Python bridge -> app-server API -> artifact bundle -> refreshed GUI status.",
                    }
                ]
            },
            "links": [
                {
                    "dataflow_node_id": "operator-action-flow",
                    "architecture_node_id": "gui-operator-console",
                    "relationship": "operator-surface",
                },
                {
                    "dataflow_node_id": "operator-action-flow",
                    "architecture_node_id": "gui-action-bridge",
                    "relationship": "bridge",
                },
                {
                    "dataflow_node_id": "operator-action-flow",
                    "architecture_node_id": "app-server-epiphany-control-plane",
                    "relationship": "control-plane",
                },
            ],
        },
        "graphFrontier": {
            "active_node_ids": [
                "gui-operator-console",
                "gui-action-bridge",
                "app-server-epiphany-control-plane",
            ],
            "dirty_paths": [],
        },
        "graphCheckpoint": {
            "checkpoint_id": f"gui-operator-{int(time.time())}",
            "graph_revision": 1,
            "summary": "GUI-prepared checkpoint for local Epiphany MVP operator testing.",
            "frontier_node_ids": [
                "gui-operator-console",
                "gui-action-bridge",
                "app-server-epiphany-control-plane",
            ],
        },
        "investigationCheckpoint": {
            "checkpoint_id": f"ix-gui-operator-{int(time.time())}",
            "kind": "source_gathering",
            "disposition": "resume_ready",
            "focus": "Validate the GUI operator loop against the fixed Epiphany lane APIs.",
            "summary": "The GUI is the operator surface; app-server typed state remains the source of truth.",
            "next_action": "Use the GUI to run coordinator plan, launch/read specialist lanes, and inspect written artifacts.",
            "captured_at_turn_id": f"gui-prepare-{int(time.time())}",
            "code_refs": [
                {
                    "path": "apps/epiphany-gui/src/App.tsx",
                    "start_line": 1,
                    "end_line": 1,
                    "symbol": "App",
                },
                {
                    "path": "tools/epiphany_gui_action.py",
                    "start_line": 1,
                    "end_line": 1,
                    "symbol": "run_action",
                },
            ],
        },
        "scratch": {
            "summary": "Prepared from the GUI so the operator has a durable thread before launching fixed lanes.",
            "current_focus": "Test the local GUI MVP workflow end to end.",
            "next_action": "Run coordinator guidance, then launch/read modeling, verification, or reorient lanes as recommended.",
            "updated_at": generated_at,
            "cwd": str(cwd),
        },
        "churn": {
            "understanding_status": "ready",
            "diff_pressure": "low",
            "graph_freshness": "fresh",
            "unexplained_writes": 0,
        },
    }


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(value, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")


def write_text(path: Path, value: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value, encoding="utf-8")


def state_revision(status: dict[str, Any]) -> int | None:
    state = status.get("read", {}).get("thread", {}).get("epiphanyState")
    if isinstance(state, dict):
        revision = state.get("revision")
        if isinstance(revision, int):
            return revision
    scene_revision = status.get("scene", {}).get("scene", {}).get("revision")
    return scene_revision if isinstance(scene_revision, int) else None


def status_epiphany_state(status: dict[str, Any]) -> dict[str, Any]:
    state = first_present(
        status,
        ("read", "thread", "epiphanyState"),
        ("scene", "scene", "epiphanyState"),
    )
    return state if isinstance(state, dict) else {}


def status_planning_state(status: dict[str, Any]) -> dict[str, Any]:
    planning_response = status.get("planning")
    if isinstance(planning_response, dict):
        planning = planning_response.get("planning")
        if isinstance(planning, dict):
            return planning
    state = status_epiphany_state(status)
    planning = state.get("planning")
    return planning if isinstance(planning, dict) else {}


def planning_drafts(planning: dict[str, Any]) -> list[dict[str, Any]]:
    drafts = planning.get("objective_drafts") or planning.get("objectiveDrafts")
    if not isinstance(drafts, list):
        return []
    return [draft for draft in drafts if isinstance(draft, dict)]


def planning_backlog_items(planning: dict[str, Any]) -> list[dict[str, Any]]:
    items = planning.get("backlog_items") or planning.get("backlogItems")
    if not isinstance(items, list):
        return []
    return [item for item in items if isinstance(item, dict)]


def string_list(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [str(item) for item in value if str(item).strip()]


def objective_subgoal_id(draft_id: str) -> str:
    cleaned = "".join(
        char if char.isalnum() or char in {"-", "_", "."} else "-"
        for char in draft_id.strip()
    ).strip("-")
    return f"objective-{cleaned or int(time.time())}"


def update_or_append_subgoal(
    state: dict[str, Any],
    *,
    subgoal_id: str,
    title: str,
    summary: str,
) -> list[dict[str, Any]]:
    raw_subgoals = state.get("subgoals")
    subgoals = deepcopy(raw_subgoals) if isinstance(raw_subgoals, list) else []
    subgoal = {
        "id": subgoal_id,
        "title": title,
        "status": "active",
        "summary": summary,
    }
    for index, item in enumerate(subgoals):
        if isinstance(item, dict) and item.get("id") == subgoal_id:
            subgoals[index] = {**item, **subgoal}
            return subgoals
    subgoals.append(subgoal)
    return subgoals


def adoption_note_list(
    scratch: dict[str, Any],
    *,
    draft_id: str,
    acceptance_criteria: list[str],
    review_gates: list[str],
) -> list[str]:
    notes = [
        f"Adopted Objective Draft {draft_id} from planning state.",
        "Acceptance criteria: "
        + (", ".join(acceptance_criteria[:6]) if acceptance_criteria else "none recorded"),
        "Review gates: " + (", ".join(review_gates[:6]) if review_gates else "none recorded"),
    ]
    existing = scratch.get("notes")
    if isinstance(existing, list):
        notes.extend(str(note) for note in existing if str(note).strip())
    return notes[:20]


def adopted_investigation_checkpoint(
    state: dict[str, Any],
    *,
    artifact_dir: Path,
    evidence_id: str,
    draft_id: str,
    title: str,
    summary: str,
) -> dict[str, Any]:
    existing = first_present(
        state,
        ("investigation_checkpoint",),
        ("investigationCheckpoint",),
    )
    checkpoint = existing if isinstance(existing, dict) else {}
    evidence_ids = string_list(
        checkpoint.get("evidence_ids") or checkpoint.get("evidenceIds")
    )
    if evidence_id not in evidence_ids:
        evidence_ids = [evidence_id, *evidence_ids]
    return {
        "checkpoint_id": str(
            checkpoint.get("checkpoint_id")
            or checkpoint.get("checkpointId")
            or f"objective-adoption-{int(time.time())}"
        ),
        "kind": "objective_adoption",
        "disposition": "resume_ready",
        "focus": title,
        "summary": f"Human adopted Objective Draft {draft_id}: {summary}",
        "next_action": (
            "Run coordinator guidance from the adopted objective; launch modeling "
            "before implementation if the current graph/checkpoint is insufficient."
        ),
        "captured_at_turn_id": f"objective-adoption-{artifact_dir.name}",
        "evidence_ids": evidence_ids,
    }


def adopt_objective_draft_patch(
    status: dict[str, Any],
    *,
    draft_id: str,
    cwd: Path,
    artifact_dir: Path,
) -> dict[str, Any]:
    if not draft_id.strip():
        raise ValueError("adoptObjectiveDraft requires --planning-draft-id")
    state = status_epiphany_state(status)
    planning = deepcopy(status_planning_state(status))
    if not planning:
        raise ValueError("adoptObjectiveDraft requires existing planning state")

    draft_items = planning_drafts(planning)
    selected_draft: dict[str, Any] | None = None
    for draft in draft_items:
        if draft.get("id") == draft_id:
            selected_draft = draft
            break
    if selected_draft is None:
        raise ValueError(f"objective draft not found: {draft_id}")

    draft_status = str(selected_draft.get("status") or "").strip().lower()
    if draft_status in {"adopted", "rejected", "superseded"}:
        raise ValueError(f"objective draft {draft_id} is {draft_status} and cannot be adopted")

    title = str(selected_draft.get("title") or "").strip()
    summary = str(selected_draft.get("summary") or "").strip()
    if not title or not summary:
        raise ValueError(f"objective draft {draft_id} is missing title or summary")

    generated_at = datetime.now(timezone.utc).isoformat()
    source_item_ids = string_list(
        selected_draft.get("source_item_ids") or selected_draft.get("sourceItemIds")
    )
    acceptance_criteria = string_list(
        selected_draft.get("acceptance_criteria")
        or selected_draft.get("acceptanceCriteria")
    )
    review_gates = string_list(
        selected_draft.get("review_gates") or selected_draft.get("reviewGates")
    )

    for draft in draft_items:
        if draft.get("id") == draft_id:
            draft["status"] = "adopted"
    backlog_items = planning_backlog_items(planning)
    for item in backlog_items:
        if item.get("id") in source_item_ids and str(item.get("status", "")).lower() in {
            "ready",
            "triaged",
            "draft",
        }:
            item["status"] = "active"
            item["updated_at"] = generated_at

    subgoal_id = objective_subgoal_id(draft_id)
    evidence_id = f"ev-objective-adoption-{artifact_dir.name}"
    observation_id = f"obs-objective-adoption-{artifact_dir.name}"
    scratch = deepcopy(state.get("scratch")) if isinstance(state.get("scratch"), dict) else {}
    scratch.update(
        {
            "summary": f"Adopted planning draft {draft_id} as the active objective: {summary}",
            "hypothesis": summary,
            "next_probe": (
                "Use coordinator/modeling guidance to strengthen the graph and "
                "checkpoint before implementation starts chasing this objective."
            ),
            "notes": adoption_note_list(
                scratch,
                draft_id=draft_id,
                acceptance_criteria=acceptance_criteria,
                review_gates=review_gates,
            ),
        }
    )

    patch = {
        "objective": title,
        "activeSubgoalId": subgoal_id,
        "subgoals": update_or_append_subgoal(
            state,
            subgoal_id=subgoal_id,
            title=title,
            summary=summary,
        ),
        "planning": planning,
        "scratch": scratch,
        "investigationCheckpoint": adopted_investigation_checkpoint(
            state,
            artifact_dir=artifact_dir,
            evidence_id=evidence_id,
            draft_id=draft_id,
            title=title,
            summary=summary,
        ),
        "observations": [
            {
                "id": observation_id,
                "summary": (
                    f"Objective Draft {draft_id} was explicitly adopted as the "
                    "active implementation objective by the operator."
                ),
                "source_kind": "planning",
                "status": "accepted",
                "evidence_ids": [evidence_id],
            }
        ],
        "evidence": [
            {
                "id": evidence_id,
                "kind": "planning-adoption",
                "status": "accepted",
                "summary": (
                    f"Human-gated planning adoption set active objective to {title!r}; "
                    f"source items={', '.join(source_item_ids) or 'none'}; cwd={cwd}."
                ),
            }
        ],
    }
    return patch


def require_thread_id(thread_id: str | None, action: str) -> str:
    if not thread_id:
        raise ValueError(f"{action} requires a persistent thread id")
    return thread_id


def is_unresumable_empty_thread_error(error: RuntimeError) -> bool:
    message = str(error)
    return "no rollout found for thread id" in message or "thread not loaded" in message


def run_action(args: argparse.Namespace) -> dict[str, Any]:
    thread_id = args.thread_id or ""
    if args.action != "prepareCheckpoint":
        thread_id = require_thread_id(args.thread_id, args.action)
    codex_home = args.codex_home.resolve()
    artifact_dir = args.artifact_root.resolve() / f"{args.action}-{time.time_ns()}-{os.getpid()}"
    transcript_path = artifact_dir / "transcript.jsonl"
    stderr_path = artifact_dir / "server.stderr.log"
    telemetry_path = artifact_dir / "agent-function-telemetry.json"
    cwd = args.cwd.resolve()
    codex_home.mkdir(parents=True, exist_ok=True)
    artifact_dir.mkdir(parents=True, exist_ok=True)

    with AppServerClient(args.app_server.resolve(), codex_home, transcript_path, stderr_path) as client:
        client.send(
            "initialize",
            {
                "clientInfo": {
                    "name": "epiphany-gui-action",
                    "title": "Epiphany GUI Action",
                    "version": "0.1.0",
                },
                "capabilities": {"experimentalApi": True},
            },
        )
        client.send("initialized", expect_response=False)
        if args.action == "prepareCheckpoint":
            if args.thread_id:
                try:
                    client.send("thread/resume", {"threadId": args.thread_id})
                except RuntimeError as error:
                    if not is_unresumable_empty_thread_error(error):
                        raise
                    thread_id = ""
            if not thread_id:
                started = client.send("thread/start", {"cwd": str(cwd), "ephemeral": False})
                if started is None:
                    raise RuntimeError("thread/start returned no response")
                thread_id = started["thread"]["id"]
            before = collect_status(client, thread_id=thread_id, cwd=cwd, ephemeral=True)
            revision = state_revision(before)
            response = client.send(
                "thread/epiphany/update",
                {
                    "threadId": thread_id,
                    "expectedRevision": revision if revision is not None else 0,
                    "patch": checkpoint_patch(cwd),
                },
            )
            summary = "Prepared durable Epiphany checkpoint."
        else:
            client.send("thread/resume", {"threadId": thread_id})
            before = collect_status(client, thread_id=thread_id, cwd=cwd, ephemeral=False)
            revision = state_revision(before)

            if args.action == "adoptObjectiveDraft":
                if revision is None:
                    raise ValueError(
                        "adoptObjectiveDraft requires ready Epiphany state with a revision"
                    )
                patch = adopt_objective_draft_patch(
                    before,
                    draft_id=args.planning_draft_id or "",
                    cwd=cwd,
                    artifact_dir=artifact_dir,
                )
                write_json(artifact_dir / "objective-adoption-state-patch.json", patch)
                response = client.send(
                    "thread/epiphany/update",
                    {
                        "threadId": thread_id,
                        "expectedRevision": revision,
                        "patch": patch,
                    },
                )
                write_json(artifact_dir / "objective-adoption-state-update.json", response)
                summary = (
                    "Adopted reviewed Objective Draft "
                    f"{args.planning_draft_id} as the active objective."
                )
            elif args.action in {"launchImagination", "launchModeling", "launchVerification"}:
                role_id = {
                    "launchImagination": "imagination",
                    "launchModeling": "modeling",
                    "launchVerification": "verification",
                }[args.action]
                payload: dict[str, Any] = {
                    "threadId": thread_id,
                    "roleId": role_id,
                    "maxRuntimeSeconds": args.max_runtime_seconds,
                }
                if revision is not None:
                    payload["expectedRevision"] = revision
                launch = client.send("thread/epiphany/roleLaunch", payload)
                if args.wait:
                    result = wait_for_role_result(
                        client,
                        thread_id=thread_id,
                        role_id=role_id,
                        timeout_seconds=args.timeout_seconds,
                        poll_seconds=args.poll_seconds,
                    )
                    response = {"launch": launch, "result": result}
                    summary = f"Launched {role_id} role worker and waited for a reviewable result."
                else:
                    response = launch
                    summary = f"Launched {role_id} role worker without waiting."
            elif args.action in {
                "readImaginationResult",
                "readModelingResult",
                "readVerificationResult",
            }:
                role_id = {
                    "readImaginationResult": "imagination",
                    "readModelingResult": "modeling",
                    "readVerificationResult": "verification",
                }[args.action]
                response = client.send(
                    "thread/epiphany/roleResult", {"threadId": thread_id, "roleId": role_id}
                )
                summary = f"Read {role_id} role result."
            elif args.action == "acceptImagination":
                if revision is None:
                    raise ValueError("acceptImagination requires ready Epiphany state with a revision")
                response = client.send(
                    "thread/epiphany/roleAccept",
                    {
                        "threadId": thread_id,
                        "roleId": "imagination",
                        "expectedRevision": revision,
                    },
                )
                summary = "Accepted reviewed imagination planning patch."
            elif args.action == "acceptModeling":
                if revision is None:
                    raise ValueError("acceptModeling requires ready Epiphany state with a revision")
                response = client.send(
                    "thread/epiphany/roleAccept",
                    {
                        "threadId": thread_id,
                        "roleId": "modeling",
                        "expectedRevision": revision,
                    },
                )
                summary = "Accepted reviewed modeling graph/checkpoint patch."
            elif args.action == "acceptVerification":
                if revision is None:
                    raise ValueError("acceptVerification requires ready Epiphany state with a revision")
                response = client.send(
                    "thread/epiphany/roleAccept",
                    {
                        "threadId": thread_id,
                        "roleId": "verification",
                        "expectedRevision": revision,
                    },
                )
                summary = "Accepted reviewed verification finding."
            elif args.action == "launchReorient":
                payload = {"threadId": thread_id, "maxRuntimeSeconds": args.max_runtime_seconds}
                if revision is not None:
                    payload["expectedRevision"] = revision
                launch = client.send("thread/epiphany/reorientLaunch", payload)
                if args.wait:
                    result = wait_for_reorient_result(
                        client,
                        thread_id=thread_id,
                        timeout_seconds=args.timeout_seconds,
                        poll_seconds=args.poll_seconds,
                    )
                    response = {"launch": launch, "result": result}
                    summary = "Launched fixed reorient-worker and waited for a reviewable result."
                else:
                    response = launch
                    summary = "Launched fixed reorient-worker without waiting."
            elif args.action == "readReorientResult":
                response = client.send(
                    "thread/epiphany/reorientResult",
                    {"threadId": thread_id, "bindingId": REORIENT_BINDING_ID},
                )
                summary = "Read reorient-worker result."
            elif args.action == "acceptReorient":
                if revision is None:
                    raise ValueError("acceptReorient requires ready Epiphany state with a revision")
                response = client.send(
                    "thread/epiphany/reorientAccept",
                    {
                        "threadId": thread_id,
                        "expectedRevision": revision,
                        "bindingId": REORIENT_BINDING_ID,
                        "updateScratch": True,
                        "updateInvestigationCheckpoint": True,
                    },
                )
                summary = "Accepted reviewed reorientation finding."
            elif args.action == "continueImplementation":
                coordinator = before.get("coordinator")
                if not isinstance(coordinator, dict):
                    raise ValueError("continueImplementation requires coordinator status")
                if coordinator.get("action") != "continueImplementation" and not args.force:
                    raise ValueError(
                        "coordinator is not recommending continueImplementation "
                        f"(got {coordinator.get('action')!r})"
                    )
                pre_git = git_snapshot(cwd)
                prompt = build_implementation_prompt(before, cwd)
                write_text(artifact_dir / "implementation-prompt.md", prompt)
                turn = client.send(
                    "turn/start",
                    {
                        "threadId": thread_id,
                        "cwd": str(cwd),
                        "sandboxPolicy": {
                            "type": "workspaceWrite",
                            "writableRoots": [str(cwd)],
                            "readOnlyAccess": {"type": "fullAccess"},
                            "networkAccess": False,
                            "excludeTmpdirEnvVar": False,
                            "excludeSlashTmp": False,
                        },
                        "input": [
                            {
                                "type": "text",
                                "text": prompt,
                                "textElements": [],
                            }
                        ],
                    },
                )
                completed = client.wait_for_notification(
                    "turn/completed", timeout=args.timeout_seconds
                )
                post_git = git_snapshot(cwd)
                implementation_result = git_change_summary(
                    pre_git,
                    post_git,
                    active_review_files=active_review_files_from_status(before),
                )
                write_json(artifact_dir / "git-before.json", pre_git)
                write_json(artifact_dir / "git-after.json", post_git)
                write_json(artifact_dir / "implementation-result.json", implementation_result)
                write_text(
                    artifact_dir / "implementation-audit.md",
                    render_implementation_audit(implementation_result),
                )
                update_status = collect_status(client, thread_id=thread_id, cwd=cwd, ephemeral=False)
                update_revision = state_revision(update_status)
                response = {
                    "turn": turn,
                    "completed": sanitize_for_operator(completed),
                    "preGit": pre_git,
                    "postGit": post_git,
                    "implementationResult": implementation_result,
                }
                if implementation_result["workspaceChanged"]:
                    diff_patch = implementation_diff_patch(
                        artifact_dir=artifact_dir,
                        implementation_result=implementation_result,
                    )
                    if update_revision is not None:
                        write_json(artifact_dir / "implementation-state-patch.json", diff_patch)
                        diff_update = client.send(
                            "thread/epiphany/update",
                            {
                                "threadId": thread_id,
                                "expectedRevision": update_revision,
                                "patch": diff_patch,
                            },
                        )
                        write_json(artifact_dir / "implementation-state-update.json", diff_update)
                        response["implementationUpdate"] = sanitize_for_operator(diff_update)
                    summary = (
                        "Ran bounded implementation turn and produced a reviewable workspace diff."
                    )
                else:
                    no_diff_patch = implementation_no_diff_patch(
                        before,
                        artifact_dir=artifact_dir,
                        implementation_result=implementation_result,
                    )
                    if no_diff_patch is not None and revision is not None:
                        write_json(artifact_dir / "no-diff-state-patch.json", no_diff_patch)
                        no_diff_update = client.send(
                            "thread/epiphany/update",
                            {
                                "threadId": thread_id,
                                "expectedRevision": update_revision
                                if update_revision is not None
                                else revision,
                                "patch": no_diff_patch,
                            },
                        )
                        write_json(artifact_dir / "no-diff-state-update.json", no_diff_update)
                        response["noDiffUpdate"] = sanitize_for_operator(no_diff_update)
                    summary = (
                        "Ran bounded implementation turn, but it produced no new workspace diff."
                    )
                    if response.get("noDiffUpdate"):
                        summary += " Marked the checkpoint for repair before retry."
            else:
                raise ValueError(f"unsupported GUI action: {args.action}")

        assert response is not None
        after = collect_status(client, thread_id=thread_id, cwd=cwd, ephemeral=True)

    operator_before = sanitize_for_operator(before)
    operator_response = sanitize_for_operator(response)
    operator_after = sanitize_for_operator(after)
    write_json(artifact_dir / "before-status.json", operator_before)
    write_json(artifact_dir / "action-response.json", operator_response)
    write_json(artifact_dir / "after-status.json", operator_after)
    write_text(artifact_dir / "after-status.txt", render_status(operator_after))
    result = {
        "action": args.action,
        "artifactPath": str(artifact_dir),
        "summary": summary,
        "threadId": thread_id,
        "response": operator_response,
        "sealedArtifactManifest": [
            {
                "path": "transcript.jsonl",
                "reason": "sealed JSON-RPC audit trail; do not read during normal supervision",
            },
            {
                "path": "server.stderr.log",
                "reason": "sealed app-server diagnostics; inspect only for explicit debugging",
            },
        ],
        "telemetryPath": str(telemetry_path),
    }
    write_json(artifact_dir / "gui-action-summary.json", result)
    write_transcript_telemetry(transcript_path, telemetry_path)
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description="Run one bounded Epiphany GUI action.")
    parser.add_argument("--app-server", type=Path, default=DEFAULT_APP_SERVER)
    parser.add_argument("--codex-home", type=Path, default=DEFAULT_CODEX_HOME)
    parser.add_argument("--cwd", type=Path, default=ROOT)
    parser.add_argument("--thread-id")
    parser.add_argument("--artifact-root", type=Path, default=DEFAULT_ARTIFACT_ROOT)
    parser.add_argument("--max-runtime-seconds", type=int, default=180)
    parser.add_argument("--timeout-seconds", type=int, default=300)
    parser.add_argument("--poll-seconds", type=float, default=5.0)
    parser.add_argument("--wait", action=argparse.BooleanOptionalAction, default=True)
    parser.add_argument(
        "--action",
        required=True,
        choices=[
            "launchImagination",
            "readImaginationResult",
            "acceptImagination",
            "launchModeling",
            "readModelingResult",
            "acceptModeling",
            "launchVerification",
            "readVerificationResult",
            "acceptVerification",
            "launchReorient",
            "readReorientResult",
            "acceptReorient",
            "adoptObjectiveDraft",
            "continueImplementation",
            "prepareCheckpoint",
        ],
    )
    parser.add_argument(
        "--planning-draft-id",
        help="Objective Draft id to adopt for adoptObjectiveDraft.",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Allow continueImplementation even when the coordinator is not recommending it.",
    )
    args = parser.parse_args()
    print(json.dumps(run_action(args), indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
