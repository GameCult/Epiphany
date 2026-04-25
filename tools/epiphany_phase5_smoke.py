from __future__ import annotations

import argparse
import copy
import json
import os
import queue
import shutil
import subprocess
import threading
import time
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_APP_SERVER = Path(r"C:\Users\Meta\.cargo-target-codex\debug\codex-app-server.exe")
DEFAULT_CODEX_HOME = ROOT / ".epiphany-smoke" / "phase5-rich-codex-home"
DEFAULT_RESULT = ROOT / ".epiphany-smoke" / "phase5-rich-smoke-result.json"
DEFAULT_TRANSCRIPT = ROOT / ".epiphany-smoke" / "phase5-rich-smoke-transcript.jsonl"
DEFAULT_STDERR = ROOT / ".epiphany-smoke" / "phase5-rich-smoke-server.stderr.log"


class AppServerClient:
    def __init__(
        self,
        app_server: Path,
        codex_home: Path,
        transcript_path: Path,
        stderr_path: Path,
    ) -> None:
        self.app_server = app_server
        self.codex_home = codex_home
        self.transcript_path = transcript_path
        self.stderr_path = stderr_path
        self.messages: queue.Queue[dict[str, Any]] = queue.Queue()
        self.next_id = 1
        self.proc: subprocess.Popen[str] | None = None
        self.transcript = None
        self.stderr_file = None

    def __enter__(self) -> "AppServerClient":
        self.transcript_path.parent.mkdir(parents=True, exist_ok=True)
        self.transcript = self.transcript_path.open("w", encoding="utf-8")
        self.stderr_file = self.stderr_path.open("w", encoding="utf-8")

        env = os.environ.copy()
        env["CODEX_HOME"] = str(self.codex_home)
        env.setdefault("CARGO_TARGET_DIR", r"C:\Users\Meta\.cargo-target-codex")

        self.proc = subprocess.Popen(
            [str(self.app_server)],
            cwd=str(ROOT / "vendor" / "codex" / "codex-rs"),
            env=env,
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            encoding="utf-8",
            errors="replace",
            bufsize=1,
        )
        threading.Thread(target=self._read_stdout, daemon=True).start()
        threading.Thread(target=self._read_stderr, daemon=True).start()
        return self

    def __exit__(self, *_: object) -> None:
        if self.proc is not None:
            try:
                if self.proc.stdin is not None:
                    self.proc.stdin.close()
            except OSError:
                pass
            if self.proc.poll() is None:
                self.proc.terminate()
                try:
                    self.proc.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    self.proc.kill()
        if self.transcript is not None:
            self.transcript.close()
        if self.stderr_file is not None:
            self.stderr_file.close()

    def send(
        self,
        method: str,
        params: dict[str, Any] | None = None,
        *,
        expect_response: bool = True,
    ) -> dict[str, Any] | None:
        msg: dict[str, Any] = {"method": method}
        request_id = None
        if expect_response:
            request_id = self.next_id
            self.next_id += 1
            msg["id"] = request_id
        if params is not None:
            msg["params"] = params

        self._record("sent", msg)
        assert self.proc is not None and self.proc.stdin is not None
        self.proc.stdin.write(json.dumps(msg, separators=(",", ":")) + "\n")
        self.proc.stdin.flush()

        if request_id is None:
            return None
        return self._wait_for(request_id)

    def _wait_for(self, request_id: int, timeout: float = 45.0) -> dict[str, Any]:
        assert self.proc is not None
        deadline = time.time() + timeout
        while time.time() < deadline:
            if self.proc.poll() is not None:
                raise RuntimeError(
                    f"app-server exited with {self.proc.returncode} before response {request_id}"
                )
            try:
                msg = self.messages.get(timeout=0.5)
            except queue.Empty:
                continue
            if msg.get("id") != request_id:
                continue
            if "error" in msg:
                raise RuntimeError(f"request {request_id} failed: {msg['error']}")
            result = msg.get("result")
            if not isinstance(result, dict):
                raise RuntimeError(f"request {request_id} returned non-object result: {result!r}")
            return result
        raise TimeoutError(f"timed out waiting for response {request_id}")

    def _read_stdout(self) -> None:
        assert self.proc is not None and self.proc.stdout is not None
        for line in self.proc.stdout:
            line = line.strip()
            if not line:
                continue
            try:
                msg = json.loads(line)
            except json.JSONDecodeError as exc:
                msg = {"_decode_error": str(exc), "raw": line}
            self._record("received", msg)
            self.messages.put(msg)

    def _read_stderr(self) -> None:
        assert self.proc is not None and self.proc.stderr is not None
        assert self.stderr_file is not None
        for line in self.proc.stderr:
            self.stderr_file.write(line)
            self.stderr_file.flush()

    def _record(self, kind: str, payload: dict[str, Any]) -> None:
        assert self.transcript is not None
        self.transcript.write(json.dumps({kind: payload}, ensure_ascii=False) + "\n")
        self.transcript.flush()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def reset_smoke_paths(codex_home: Path, result_path: Path, transcript_path: Path, stderr_path: Path) -> None:
    smoke_root = ROOT / ".epiphany-smoke"
    resolved_home = codex_home.resolve()
    resolved_smoke = smoke_root.resolve()
    if resolved_home == resolved_smoke or resolved_smoke not in resolved_home.parents:
        raise ValueError(f"refusing to delete non-smoke CODEX_HOME: {codex_home}")
    if codex_home.exists():
        shutil.rmtree(codex_home)
    codex_home.mkdir(parents=True, exist_ok=True)
    for path in (result_path, transcript_path, stderr_path):
        if path.exists():
            path.unlink()


def run_smoke(args: argparse.Namespace) -> dict[str, Any]:
    app_server = args.app_server.resolve()
    if not app_server.exists():
        raise FileNotFoundError(f"codex app-server binary not found: {app_server}")

    codex_home = args.codex_home.resolve()
    result_path = args.result.resolve()
    transcript_path = args.transcript.resolve()
    stderr_path = args.stderr.resolve()
    reset_smoke_paths(codex_home, result_path, transcript_path, stderr_path)

    with AppServerClient(app_server, codex_home, transcript_path, stderr_path) as client:
        client.send(
            "initialize",
            {
                "clientInfo": {
                    "name": "epiphany-phase5-rich-smoke",
                    "title": "Epiphany Phase 5 Rich Smoke",
                    "version": "0.1.0",
                },
                "capabilities": {"experimentalApi": True},
            },
        )
        client.send("initialized", expect_response=False)
        started = client.send(
            "thread/start",
            {"cwd": str(ROOT / "epiphany-core"), "ephemeral": True},
        )
        assert started is not None
        thread_id = started["thread"]["id"]

        noisy_tool_text = "\n".join(
            [
                r"Compiling epiphany-core v0.1.0 (E:\Projects\EpiphanyAgent\epiphany-core)",
                "warning: unrelated smoke warning that should not dominate the summary",
                "running 3 tests",
                "test distillation::tests::source_output_summary ... ok",
                "test proposal::tests::match_kind_pressure ... ok",
                "test result: ok. 3 passed; 0 failed; 0 ignored; finished in 0.02s",
                "Finished test profile [unoptimized + debuginfo] target(s) in 0.31s",
            ]
        )
        distill = client.send(
            "thread/epiphany/distill",
            {
                "threadId": thread_id,
                "sourceKind": "shell-tool",
                "status": "ok",
                "subject": "phase5 richer app-server smoke",
                "text": noisy_tool_text,
                "codeRefs": [
                    {
                        "path": "src/distillation.rs",
                        "start_line": 28,
                        "end_line": 160,
                        "symbol": "distill_observation",
                    }
                ],
            },
        )
        assert distill is not None
        distill_patch = distill["patch"]
        evidence = distill_patch["evidence"][0]
        observation = distill_patch["observations"][0]
        summary = evidence["summary"].lower()
        require(distill["expectedRevision"] == 0, "distill expected revision should start at 0")
        require(evidence["kind"] == "tool-output", "distill evidence kind should be tool-output")
        require(
            "test result: ok" in summary or "finished test profile" in summary,
            "distill summary should preserve salient tool output",
        )
        require(
            "unrelated smoke warning" not in summary,
            "distill summary should prioritize final results over generic warnings",
        )

        update = client.send(
            "thread/epiphany/update",
            {"threadId": thread_id, "expectedRevision": 0, "patch": distill_patch},
        )
        assert update is not None
        require(update["epiphanyState"]["revision"] == 1, "update should advance revision to 1")

        propose = client.send("thread/epiphany/propose", {"threadId": thread_id})
        assert propose is not None
        proposal_patch = propose["patch"]
        require(propose["expectedRevision"] == 1, "proposal should see revision 1")
        require(bool(proposal_patch.get("graphs")), "proposal should include graph replacements")
        require(bool(proposal_patch.get("churn")), "proposal should include churn replacement")
        require(bool(proposal_patch["observations"]), "proposal should include a candidate observation")

        read_after_propose = client.send(
            "thread/read", {"threadId": thread_id, "includeTurns": False}
        )
        assert read_after_propose is not None
        state_after_propose = read_after_propose["thread"].get("epiphanyState", {})
        require(state_after_propose.get("revision") == 1, "read-only propose should not mutate")
        require("churn" not in state_after_propose, "propose should not persist churn before promotion")

        risky_missing_warning = copy.deepcopy(proposal_patch)
        risky_missing_warning["churn"]["diff_pressure"] = "medium"
        risky_missing_warning["churn"]["graph_freshness"] = "proposal_broadened"
        risky_missing_warning["churn"].pop("warning", None)
        reject_missing_warning = client.send(
            "thread/epiphany/promote",
            {
                "threadId": thread_id,
                "expectedRevision": 1,
                "patch": risky_missing_warning,
                "verifierEvidence": {
                    "id": "ev-phase5-risky-warning-verifier",
                    "kind": "verification",
                    "status": "ok",
                    "summary": "Intentional missing-warning rejection check",
                },
            },
        )
        assert reject_missing_warning is not None
        require(not reject_missing_warning["accepted"], "risky churn without warning should reject")
        require(
            any("patch.churn.warning" in reason for reason in reject_missing_warning["reasons"]),
            "missing-warning rejection should name patch.churn.warning",
        )

        risky_expansion_low_pressure = copy.deepcopy(proposal_patch)
        risky_expansion_low_pressure["churn"]["diff_pressure"] = "low"
        risky_expansion_low_pressure["churn"]["graph_freshness"] = "proposal-expanded"
        risky_expansion_low_pressure["churn"].pop("warning", None)
        reject_expansion_low_pressure = client.send(
            "thread/epiphany/promote",
            {
                "threadId": thread_id,
                "expectedRevision": 1,
                "patch": risky_expansion_low_pressure,
                "verifierEvidence": {
                    "id": "ev-phase5-expanded-low-pressure-verifier",
                    "kind": "verification",
                    "status": "ok",
                    "summary": "Intentional expanded-low-pressure rejection check",
                },
            },
        )
        assert reject_expansion_low_pressure is not None
        require(
            not reject_expansion_low_pressure["accepted"],
            "expanded churn should reject without warning even when pressure is low",
        )
        require(
            any(
                "patch.churn.warning" in reason
                for reason in reject_expansion_low_pressure["reasons"]
            ),
            "expanded-low-pressure rejection should name patch.churn.warning",
        )

        read_after_warning_reject = client.send(
            "thread/read", {"threadId": thread_id, "includeTurns": False}
        )
        assert read_after_warning_reject is not None
        require(
            read_after_warning_reject["thread"].get("epiphanyState", {}).get("revision") == 1,
            "missing-warning rejection should not mutate state",
        )

        risky_weak_verifier = copy.deepcopy(proposal_patch)
        risky_weak_verifier["churn"]["diff_pressure"] = "medium"
        risky_weak_verifier["churn"]["graph_freshness"] = "proposal_broadened"
        risky_weak_verifier["churn"]["warning"] = (
            "Same-path or medium-pressure smoke delta requires explicit strong verifier evidence."
        )
        reject_weak_verifier = client.send(
            "thread/epiphany/promote",
            {
                "threadId": thread_id,
                "expectedRevision": 1,
                "patch": risky_weak_verifier,
                "verifierEvidence": {
                    "id": "ev-phase5-risky-weak-verifier",
                    "kind": "observation",
                    "status": "ok",
                    "summary": "Intentional weak-kind rejection check",
                },
            },
        )
        assert reject_weak_verifier is not None
        require(not reject_weak_verifier["accepted"], "weak verifier kind should reject")
        require(
            any("verifierEvidence.kind" in reason for reason in reject_weak_verifier["reasons"]),
            "weak-kind rejection should name verifierEvidence.kind",
        )

        risky_substring_verifier = copy.deepcopy(risky_weak_verifier)
        reject_substring_verifier = client.send(
            "thread/epiphany/promote",
            {
                "threadId": thread_id,
                "expectedRevision": 1,
                "patch": risky_substring_verifier,
                "verifierEvidence": {
                    "id": "ev-phase5-risky-substring-verifier",
                    "kind": "contest",
                    "status": "ok",
                    "summary": "Intentional substring verifier-kind rejection check",
                },
            },
        )
        assert reject_substring_verifier is not None
        require(
            not reject_substring_verifier["accepted"],
            "substring verifier kind should not satisfy risky churn policy",
        )
        require(
            any(
                "verifierEvidence.kind" in reason
                for reason in reject_substring_verifier["reasons"]
            ),
            "substring-kind rejection should name verifierEvidence.kind",
        )

        read_after_kind_reject = client.send(
            "thread/read", {"threadId": thread_id, "includeTurns": False}
        )
        assert read_after_kind_reject is not None
        require(
            read_after_kind_reject["thread"].get("epiphanyState", {}).get("revision") == 1,
            "weak-kind rejection should not mutate state",
        )

        accepted = client.send(
            "thread/epiphany/promote",
            {
                "threadId": thread_id,
                "expectedRevision": 1,
                "patch": risky_weak_verifier,
                "verifierEvidence": {
                    "id": "ev-phase5-risky-strong-verifier",
                    "kind": "verification",
                    "status": "ok",
                    "summary": "Verifier accepted the risky churn smoke after warning rationale was present",
                },
            },
        )
        assert accepted is not None
        require(accepted["accepted"], "strong verifier should accept risky churn with warning")
        final_state = accepted["epiphanyState"]
        require(final_state["revision"] == 2, "accepted promotion should advance revision to 2")
        require(
            final_state["churn"]["diff_pressure"] == "medium",
            "accepted state should preserve risky churn pressure",
        )

        final_read = client.send("thread/read", {"threadId": thread_id, "includeTurns": False})
        assert final_read is not None
        require(
            final_read["thread"]["epiphanyState"]["revision"] == 2,
            "final read should return persisted revision 2",
        )

        result = {
            "threadId": thread_id,
            "codexHome": str(codex_home),
            "distillExpectedRevision": distill["expectedRevision"],
            "distillObservationId": observation["id"],
            "distillEvidenceId": evidence["id"],
            "distillEvidenceKind": evidence["kind"],
            "distillSummary": evidence["summary"],
            "proposalExpectedRevision": propose["expectedRevision"],
            "proposalObservationId": proposal_patch["observations"][0]["id"],
            "proposalChurn": proposal_patch["churn"],
            "readOnlyRevisionAfterPropose": state_after_propose.get("revision"),
            "missingWarningAccepted": reject_missing_warning["accepted"],
            "missingWarningReasons": reject_missing_warning["reasons"],
            "expandedLowPressureAccepted": reject_expansion_low_pressure["accepted"],
            "expandedLowPressureReasons": reject_expansion_low_pressure["reasons"],
            "weakVerifierAccepted": reject_weak_verifier["accepted"],
            "weakVerifierReasons": reject_weak_verifier["reasons"],
            "substringVerifierAccepted": reject_substring_verifier["accepted"],
            "substringVerifierReasons": reject_substring_verifier["reasons"],
            "accepted": accepted["accepted"],
            "finalRevision": final_state["revision"],
            "finalChurn": final_state["churn"],
            "graphNodeCount": len(
                final_state.get("graphs", {}).get("architecture", {}).get("nodes", [])
            ),
        }
    result_path.parent.mkdir(parents=True, exist_ok=True)
    result_path.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    return result


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Live-smoke the richer Phase 5 Epiphany app-server chain."
    )
    parser.add_argument("--app-server", type=Path, default=DEFAULT_APP_SERVER)
    parser.add_argument("--codex-home", type=Path, default=DEFAULT_CODEX_HOME)
    parser.add_argument("--result", type=Path, default=DEFAULT_RESULT)
    parser.add_argument("--transcript", type=Path, default=DEFAULT_TRANSCRIPT)
    parser.add_argument("--stderr", type=Path, default=DEFAULT_STDERR)
    args = parser.parse_args()
    result = run_smoke(args)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
