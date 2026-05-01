from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from pathlib import Path
from typing import Any

from epiphany_phase5_smoke import AppServerClient
from epiphany_phase5_smoke import DEFAULT_APP_SERVER
from epiphany_phase5_smoke import ROOT
from epiphany_phase5_smoke import require
from epiphany_phase5_smoke import reset_smoke_paths
from epiphany_phase6_planning_smoke import planning_patch


DEFAULT_CODEX_HOME = ROOT / ".epiphany-smoke" / "gui-planning-codex-home"
DEFAULT_ARTIFACT_ROOT = ROOT / ".epiphany-smoke" / "gui-planning-actions"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "gui-planning-smoke-result.json"
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-smoke" / "gui-planning-smoke-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-smoke" / "gui-planning-smoke-server.stderr.log"
DRAFT_ID = "draft-planning-dashboard"


def find_draft(planning: dict[str, Any], draft_id: str) -> dict[str, Any]:
    drafts = planning.get("objective_drafts")
    require(isinstance(drafts, list), "planning should expose objective_drafts")
    for draft in drafts:
        if isinstance(draft, dict) and draft.get("id") == draft_id:
            return draft
    raise AssertionError(f"objective draft missing: {draft_id}")


def find_backlog_item(planning: dict[str, Any], item_id: str) -> dict[str, Any]:
    items = planning.get("backlog_items")
    require(isinstance(items, list), "planning should expose backlog_items")
    for item in items:
        if isinstance(item, dict) and item.get("id") == item_id:
            return item
    raise AssertionError(f"backlog item missing: {item_id}")


def run_gui_action(
    *,
    app_server: Path,
    codex_home: Path,
    artifact_root: Path,
    thread_id: str,
    cwd: Path,
) -> dict[str, Any]:
    command = [
        sys.executable,
        str(ROOT / "tools" / "epiphany_gui_action.py"),
        "--app-server",
        str(app_server),
        "--codex-home",
        str(codex_home),
        "--artifact-root",
        str(artifact_root),
        "--cwd",
        str(cwd),
        "--thread-id",
        thread_id,
        "--action",
        "adoptObjectiveDraft",
        "--planning-draft-id",
        DRAFT_ID,
    ]
    completed = subprocess.run(
        command,
        cwd=ROOT,
        capture_output=True,
        check=False,
        encoding="utf-8",
        errors="replace",
    )
    if completed.returncode != 0:
        raise RuntimeError(
            "GUI planning action failed:\n"
            f"stdout:\n{completed.stdout}\n\nstderr:\n{completed.stderr}"
        )
    return json.loads(completed.stdout)


def run_smoke(args: argparse.Namespace) -> dict[str, Any]:
    app_server = args.app_server.resolve()
    if not app_server.exists():
        raise FileNotFoundError(f"codex app-server binary not found: {app_server}")

    codex_home = args.codex_home.resolve()
    artifact_root = args.artifact_root.resolve()
    result_path = args.result.resolve()
    transcript_path = args.transcript.resolve()
    stderr_path = args.stderr.resolve()
    reset_smoke_paths(codex_home, result_path, transcript_path, stderr_path)
    if artifact_root.exists():
        shutil.rmtree(artifact_root)
    artifact_root.mkdir(parents=True, exist_ok=True)

    with AppServerClient(app_server, codex_home, transcript_path, stderr_path) as client:
        client.send(
            "initialize",
            {
                "clientInfo": {
                    "name": "epiphany-gui-planning-smoke",
                    "title": "Epiphany GUI Planning Smoke",
                    "version": "0.1.0",
                },
                "capabilities": {"experimentalApi": True},
            },
        )
        client.send("initialized", expect_response=False)
        started = client.send("thread/start", {"cwd": str(ROOT), "ephemeral": False})
        assert started is not None
        thread_id = started["thread"]["id"]
        update = client.send(
            "thread/epiphany/update",
            {"threadId": thread_id, "expectedRevision": 0, "patch": planning_patch()},
        )
        assert update is not None
        require(update["revision"] == 1, "seed planning update should create revision 1")
        before_planning = client.send("thread/epiphany/planning", {"threadId": thread_id})
        assert before_planning is not None
        require(
            find_draft(before_planning["planning"], DRAFT_ID)["status"] == "draft",
            "seed draft should start as a draft",
        )

    action_result = run_gui_action(
        app_server=app_server,
        codex_home=codex_home,
        artifact_root=artifact_root,
        thread_id=thread_id,
        cwd=ROOT,
    )
    artifact_path = Path(action_result["artifactPath"])
    require(artifact_path.exists(), "GUI action should write an artifact bundle")
    require(
        (artifact_path / "objective-adoption-state-patch.json").exists(),
        "adoption should write the state patch artifact",
    )
    require(
        (artifact_path / "objective-adoption-state-update.json").exists(),
        "adoption should write the state update artifact",
    )

    after_status = json.loads((artifact_path / "after-status.json").read_text(encoding="utf-8"))
    state = after_status["read"]["thread"]["epiphanyState"]
    planning = after_status["planning"]["planning"]
    draft = find_draft(planning, DRAFT_ID)
    backlog_item = find_backlog_item(planning, "backlog-planning-dashboard")
    require(state["objective"] == "Build the planning dashboard slice", "draft title should become objective")
    require(
        state.get("activeSubgoalId") or state.get("active_subgoal_id"),
        "adoption should set an active subgoal id",
    )
    require(draft["status"] == "adopted", "adopted draft should be marked adopted")
    require(backlog_item["status"] == "active", "source backlog item should be marked active")
    require(
        after_status["planning"]["summary"]["activeObjective"]
        == "Build the planning dashboard slice",
        "planning summary should reflect the adopted active objective",
    )
    require(
        "Adopted reviewed Objective Draft" in action_result["summary"],
        "GUI action summary should describe objective adoption",
    )

    result = {
        "threadId": thread_id,
        "codexHome": str(codex_home),
        "artifactPath": str(artifact_path),
        "objective": state["objective"],
        "activeSubgoalId": state.get("activeSubgoalId") or state.get("active_subgoal_id"),
        "draftStatus": draft["status"],
        "backlogStatus": backlog_item["status"],
        "summary": action_result["summary"],
    }
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(
        json.dumps(result, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    return result


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Live-smoke the GUI planning Objective Draft adoption action."
    )
    parser.add_argument("--app-server", type=Path, default=DEFAULT_APP_SERVER)
    parser.add_argument("--codex-home", type=Path, default=DEFAULT_CODEX_HOME)
    parser.add_argument("--artifact-root", type=Path, default=DEFAULT_ARTIFACT_ROOT)
    parser.add_argument("--result", type=Path, default=DEFAULT_RESULT)
    parser.add_argument("--transcript", type=Path, default=DEFAULT_TRANSCRIPT)
    parser.add_argument("--stderr", type=Path, default=DEFAULT_STDERR)
    args = parser.parse_args()
    result = run_smoke(args)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
