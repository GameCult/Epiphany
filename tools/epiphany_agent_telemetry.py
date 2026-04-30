from __future__ import annotations

import argparse
from collections import Counter
from collections import defaultdict
from datetime import datetime
from datetime import timezone
import json
from pathlib import Path
import re
from typing import Any


TEXTLIKE_KEYS = {
    "activeTranscript",
    "body",
    "content",
    "input",
    "inputTranscript",
    "items",
    "message",
    "note",
    "output",
    "prompt",
    "raw",
    "rawResult",
    "reasoning",
    "result",
    "summary",
    "text",
    "turns",
}

SAFE_KEYS = {
    "action",
    "artifactPath",
    "backendJobId",
    "bindingId",
    "changedFields",
    "cwd",
    "ephemeral",
    "expectedRevision",
    "id",
    "jobId",
    "kind",
    "level",
    "maxRuntimeSeconds",
    "method",
    "mode",
    "path",
    "recommendedAction",
    "recommendedSceneAction",
    "revision",
    "roleId",
    "source",
    "stateStatus",
    "status",
    "targetRole",
    "threadId",
    "turnId",
    "type",
    "verdict",
}


def json_dump(value: Any) -> str:
    return json.dumps(value, indent=2, ensure_ascii=False) + "\n"


def scalar_summary(key: str, value: Any) -> Any:
    if value is None or isinstance(value, (bool, int, float)):
        return value
    if isinstance(value, str):
        if key in SAFE_KEYS:
            return value
        return {"sealed": True, "kind": "text", "chars": len(value)}
    return None


def summarize_value(value: Any, *, key: str = "") -> Any:
    scalar = scalar_summary(key, value)
    if scalar is not None:
        return scalar

    if key in TEXTLIKE_KEYS:
        return sealed_summary(key, value)

    if isinstance(value, list):
        return {
            "kind": "list",
            "count": len(value),
            "items": [summarize_value(item) for item in value[:8]],
            "truncated": len(value) > 8,
        }

    if isinstance(value, dict):
        result: dict[str, Any] = {}
        for child_key, child_value in value.items():
            if child_key in TEXTLIKE_KEYS and child_key not in SAFE_KEYS:
                result[child_key] = sealed_summary(child_key, child_value)
            elif child_key in SAFE_KEYS:
                result[child_key] = summarize_value(child_value, key=child_key)
            elif isinstance(child_value, (dict, list)):
                result[child_key] = summarize_value(child_value, key=child_key)
            elif isinstance(child_value, str):
                result[child_key] = {"sealed": True, "kind": "text", "chars": len(child_value)}
            else:
                result[child_key] = child_value
        return result

    return {"sealed": True, "kind": type(value).__name__}


def sealed_summary(key: str, value: Any) -> dict[str, Any]:
    summary: dict[str, Any] = {
        "sealed": True,
        "key": key,
        "reason": "direct agent or transcript content is excluded from telemetry",
    }
    if isinstance(value, str):
        summary["chars"] = len(value)
    elif isinstance(value, list):
        summary["items"] = len(value)
    elif isinstance(value, dict):
        summary["keys"] = sorted(str(k) for k in value.keys())[:24]
        summary["keyCount"] = len(value)
    return summary


def collect_strings(value: Any, key_names: set[str]) -> list[str]:
    found: list[str] = []
    if isinstance(value, dict):
        for key, item in value.items():
            if key in key_names and isinstance(item, str):
                found.append(item)
            elif isinstance(item, (dict, list)):
                found.extend(collect_strings(item, key_names))
    elif isinstance(value, list):
        for item in value:
            found.extend(collect_strings(item, key_names))
    return found


def summarize_command_text(command: str) -> dict[str, Any]:
    compact = " ".join(line.strip() for line in command.splitlines() if line.strip())
    verbs = sorted(
        {
            match.group(0)
            for match in re.finditer(
                r"\b(?:rg|git|Get-Content|Get-ChildItem|Select-String|Test-Path|New-Item|Set-Content|"
                r"python|cargo|dotnet|npm|node|apply_patch)\b",
                command,
                flags=re.IGNORECASE,
            )
        },
        key=str.lower,
    )
    return {
        "chars": len(command),
        "lines": len(command.splitlines()) or 1,
        "preview": compact[:240],
        "truncated": len(compact) > 240,
        "verbs": verbs,
        "hasWriteVerb": any(
            verb.lower() in {"new-item", "set-content", "apply_patch"}
            for verb in verbs
        ),
    }


def collect_command_telemetry(value: Any) -> list[dict[str, Any]]:
    found: list[dict[str, Any]] = []
    if isinstance(value, dict):
        if value.get("type") == "commandExecution" and isinstance(value.get("command"), str):
            command_info = summarize_command_text(value["command"])
            command_info["cwd"] = value.get("cwd")
            command_info["status"] = value.get("status")
            command_info["exitCode"] = value.get("exitCode")
            command_info["durationMs"] = value.get("durationMs")
            found.append(command_info)
        for item in value.values():
            if isinstance(item, (dict, list)):
                found.extend(collect_command_telemetry(item))
    elif isinstance(value, list):
        for item in value:
            found.extend(collect_command_telemetry(item))
    return found


def telemetry_event(index: int, record: dict[str, Any]) -> dict[str, Any]:
    direction = "unknown"
    payload: Any = record
    if "sent" in record:
        direction = "sent"
        payload = record["sent"]
    elif "received" in record:
        direction = "received"
        payload = record["received"]

    event: dict[str, Any] = {"index": index, "direction": direction}
    if not isinstance(payload, dict):
        event["payload"] = sealed_summary("payload", payload)
        return event

    if "id" in payload:
        event["id"] = payload["id"]
    if "method" in payload:
        event["method"] = payload["method"]
    if "error" in payload:
        event["error"] = summarize_value(payload["error"])
    if "params" in payload:
        event["params"] = summarize_value(payload["params"], key="params")
    if "result" in payload:
        event["result"] = summarize_value(payload["result"], key="responseResult")

    command_telemetry = collect_command_telemetry(payload)
    if command_telemetry:
        event["commandTelemetry"] = command_telemetry[:16]
        event["commandTelemetryCount"] = len(command_telemetry)

    names = sorted(set(collect_strings(payload, {"toolName", "tool", "functionName", "name"})))
    if names:
        event["functionNames"] = names[:16]
        event["functionNameCount"] = len(names)

    paths = sorted(set(collect_strings(payload, {"path", "cwd", "artifactPath"})))
    if paths:
        event["paths"] = paths[:16]
        event["pathCount"] = len(paths)

    return event


def build_telemetry(transcript_path: Path) -> dict[str, Any]:
    events: list[dict[str, Any]] = []
    method_counts: Counter[str] = Counter()
    direction_counts: Counter[str] = Counter()
    function_counts: Counter[str] = Counter()
    decode_errors: list[dict[str, Any]] = []

    if not transcript_path.exists():
        return {
            "transcriptPath": str(transcript_path),
            "generatedAt": datetime.now(timezone.utc).isoformat(),
            "status": "missing",
            "events": [],
            "counts": {},
        }

    for index, line in enumerate(transcript_path.read_text(encoding="utf-8").splitlines()):
        if not line.strip():
            continue
        try:
            record = json.loads(line)
        except json.JSONDecodeError as exc:
            decode_errors.append({"index": index, "error": str(exc), "chars": len(line)})
            continue
        if not isinstance(record, dict):
            events.append({"index": index, "payload": sealed_summary("record", record)})
            continue
        event = telemetry_event(index, record)
        events.append(event)
        direction_counts[event.get("direction", "unknown")] += 1
        method = event.get("method")
        if isinstance(method, str):
            method_counts[method] += 1
        for name in event.get("functionNames", []):
            function_counts[name] += 1

    request_latency_shape: dict[str, dict[str, int]] = defaultdict(lambda: {"sent": 0, "received": 0})
    for event in events:
        event_id = event.get("id")
        if event_id is None:
            continue
        direction = str(event.get("direction", "unknown"))
        if direction in {"sent", "received"}:
            request_latency_shape[str(event_id)][direction] += 1

    return {
        "transcriptPath": str(transcript_path),
        "generatedAt": datetime.now(timezone.utc).isoformat(),
        "status": "ok",
        "policy": {
            "directThoughtsSealed": True,
            "note": "Telemetry summarizes function/API shape only; text, raw results, and transcript payloads are sealed.",
        },
        "counts": {
            "events": len(events),
            "directions": dict(direction_counts),
            "methods": dict(method_counts),
            "functionNames": dict(function_counts),
            "decodeErrors": len(decode_errors),
        },
        "requestShape": dict(request_latency_shape),
        "decodeErrors": decode_errors,
        "events": events,
    }


def write_transcript_telemetry(transcript_path: Path, telemetry_path: Path) -> dict[str, Any]:
    telemetry = build_telemetry(transcript_path)
    telemetry_path.parent.mkdir(parents=True, exist_ok=True)
    telemetry_path.write_text(json_dump(telemetry), encoding="utf-8")
    return telemetry


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Generate operator-safe function/API telemetry from a sealed app-server transcript."
    )
    parser.add_argument("transcript", type=Path)
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()

    telemetry = build_telemetry(args.transcript.resolve())
    if args.output:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json_dump(telemetry), encoding="utf-8")
    else:
        print(json_dump(telemetry), end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
