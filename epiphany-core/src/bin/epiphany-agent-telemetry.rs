use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::Utc;
use serde_json::Map;
use serde_json::Value;
use serde_json::json;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const TEXTLIKE_KEYS: &[&str] = &[
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
];

const SAFE_KEYS: &[&str] = &[
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
];

const COMMAND_VERBS: &[&str] = &[
    "rg",
    "git",
    "Get-Content",
    "Get-ChildItem",
    "Select-String",
    "Test-Path",
    "New-Item",
    "Set-Content",
    "python",
    "cargo",
    "dotnet",
    "npm",
    "node",
    "apply_patch",
];

fn main() -> Result<()> {
    let args = parse_args()?;
    let telemetry = build_telemetry(&args.transcript)?;
    if let Some(output) = args.output {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("failed to create {}", parent.display()))?;
        }
        fs::write(
            &output,
            format!("{}\n", serde_json::to_string_pretty(&telemetry)?),
        )
        .with_context(|| format!("failed to write {}", output.display()))?;
    }
    println!("{}", serde_json::to_string_pretty(&telemetry)?);
    Ok(())
}

struct Args {
    transcript: PathBuf,
    output: Option<PathBuf>,
}

fn parse_args() -> Result<Args> {
    let mut args = env::args().skip(1);
    let Some(transcript) = args.next() else {
        return Err(anyhow!(
            "usage: epiphany-agent-telemetry <transcript> [--output <path>]"
        ));
    };
    let mut output = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--output" => {
                output = Some(PathBuf::from(
                    args.next()
                        .ok_or_else(|| anyhow!("--output requires a path"))?,
                ));
            }
            _ => return Err(anyhow!("unknown argument: {arg}")),
        }
    }
    Ok(Args {
        transcript: PathBuf::from(transcript),
        output,
    })
}

fn build_telemetry(transcript_path: &Path) -> Result<Value> {
    if !transcript_path.exists() {
        return Ok(json!({
            "transcriptPath": transcript_path,
            "generatedAt": Utc::now().to_rfc3339(),
            "status": "missing",
            "events": [],
            "counts": {},
        }));
    }

    let raw = fs::read_to_string(transcript_path)
        .with_context(|| format!("failed to read {}", transcript_path.display()))?;
    let mut events = Vec::new();
    let mut method_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut direction_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut function_counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut decode_errors = Vec::new();

    for (index, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let record: Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(error) => {
                decode_errors.push(json!({
                    "index": index,
                    "error": error.to_string(),
                    "chars": line.chars().count(),
                }));
                continue;
            }
        };
        let event = match record {
            Value::Object(map) => telemetry_event(index, &map),
            other => json!({
                "index": index,
                "payload": sealed_summary("record", &other),
            }),
        };
        let direction = event
            .get("direction")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        *direction_counts.entry(direction).or_default() += 1;
        if let Some(method) = event.get("method").and_then(Value::as_str) {
            *method_counts.entry(method.to_string()).or_default() += 1;
        }
        if let Some(names) = event.get("functionNames").and_then(Value::as_array) {
            for name in names.iter().filter_map(Value::as_str) {
                *function_counts.entry(name.to_string()).or_default() += 1;
            }
        }
        events.push(event);
    }

    let mut request_shape: BTreeMap<String, BTreeMap<String, u64>> = BTreeMap::new();
    for event in &events {
        let Some(event_id) = event.get("id") else {
            continue;
        };
        let Some(direction) = event.get("direction").and_then(Value::as_str) else {
            continue;
        };
        if direction != "sent" && direction != "received" {
            continue;
        }
        let id = value_key(event_id);
        let entry = request_shape.entry(id).or_insert_with(|| {
            BTreeMap::from([("sent".to_string(), 0), ("received".to_string(), 0)])
        });
        *entry.entry(direction.to_string()).or_default() += 1;
    }

    Ok(json!({
        "transcriptPath": transcript_path,
        "generatedAt": Utc::now().to_rfc3339(),
        "status": "ok",
        "policy": {
            "directThoughtsSealed": true,
            "note": "Telemetry summarizes function/API shape only; text, raw results, and transcript payloads are sealed.",
        },
        "counts": {
            "events": events.len(),
            "directions": direction_counts,
            "methods": method_counts,
            "functionNames": function_counts,
            "decodeErrors": decode_errors.len(),
        },
        "requestShape": request_shape,
        "decodeErrors": decode_errors,
        "events": events,
    }))
}

fn telemetry_event(index: usize, record: &Map<String, Value>) -> Value {
    let (direction, payload) = if let Some(sent) = record.get("sent") {
        ("sent", sent)
    } else if let Some(received) = record.get("received") {
        ("received", received)
    } else {
        ("unknown", &Value::Object(record.clone()))
    };

    let mut event = Map::new();
    event.insert("index".to_string(), json!(index));
    event.insert("direction".to_string(), json!(direction));

    let Some(payload) = payload.as_object() else {
        event.insert("payload".to_string(), sealed_summary("payload", payload));
        return Value::Object(event);
    };

    for key in ["id", "method"] {
        if let Some(value) = payload.get(key) {
            event.insert(key.to_string(), value.clone());
        }
    }
    if let Some(error) = payload.get("error") {
        event.insert("error".to_string(), summarize_value(error, ""));
    }
    if let Some(params) = payload.get("params") {
        event.insert("params".to_string(), summarize_value(params, "params"));
    }
    if let Some(result) = payload.get("result") {
        event.insert(
            "result".to_string(),
            summarize_value(result, "responseResult"),
        );
    }

    let command_telemetry = collect_command_telemetry(&Value::Object(payload.clone()));
    if !command_telemetry.is_empty() {
        event.insert(
            "commandTelemetry".to_string(),
            Value::Array(command_telemetry.iter().take(16).cloned().collect()),
        );
        event.insert(
            "commandTelemetryCount".to_string(),
            json!(command_telemetry.len()),
        );
    }

    let names = collect_strings(
        &Value::Object(payload.clone()),
        &["toolName", "tool", "functionName", "name"],
    );
    if !names.is_empty() {
        event.insert(
            "functionNames".to_string(),
            Value::Array(names.iter().take(16).map(|s| json!(s)).collect()),
        );
        event.insert("functionNameCount".to_string(), json!(names.len()));
    }

    let paths = collect_strings(
        &Value::Object(payload.clone()),
        &["path", "cwd", "artifactPath"],
    );
    if !paths.is_empty() {
        event.insert(
            "paths".to_string(),
            Value::Array(paths.iter().take(16).map(|s| json!(s)).collect()),
        );
        event.insert("pathCount".to_string(), json!(paths.len()));
    }

    Value::Object(event)
}

fn summarize_value(value: &Value, key: &str) -> Value {
    if let Some(scalar) = scalar_summary(key, value) {
        return scalar;
    }
    if contains(TEXTLIKE_KEYS, key) {
        return sealed_summary(key, value);
    }
    match value {
        Value::Array(items) => json!({
            "kind": "list",
            "count": items.len(),
            "items": items.iter().take(8).map(|item| summarize_value(item, "")).collect::<Vec<_>>(),
            "truncated": items.len() > 8,
        }),
        Value::Object(map) => {
            let mut result = Map::new();
            for (child_key, child_value) in map {
                let summarized =
                    if contains(TEXTLIKE_KEYS, child_key) && !contains(SAFE_KEYS, child_key) {
                        sealed_summary(child_key, child_value)
                    } else if contains(SAFE_KEYS, child_key)
                        || child_value.is_object()
                        || child_value.is_array()
                    {
                        summarize_value(child_value, child_key)
                    } else if let Some(text) = child_value.as_str() {
                        json!({"sealed": true, "kind": "text", "chars": text.chars().count()})
                    } else {
                        child_value.clone()
                    };
                result.insert(child_key.clone(), summarized);
            }
            Value::Object(result)
        }
        other => json!({"sealed": true, "kind": value_kind(other)}),
    }
}

fn scalar_summary(key: &str, value: &Value) -> Option<Value> {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => Some(value.clone()),
        Value::String(text) if contains(SAFE_KEYS, key) => Some(json!(text)),
        Value::String(text) => {
            Some(json!({"sealed": true, "kind": "text", "chars": text.chars().count()}))
        }
        _ => None,
    }
}

fn sealed_summary(key: &str, value: &Value) -> Value {
    let mut summary = Map::new();
    summary.insert("sealed".to_string(), json!(true));
    summary.insert("key".to_string(), json!(key));
    summary.insert(
        "reason".to_string(),
        json!("direct agent or transcript content is excluded from telemetry"),
    );
    match value {
        Value::String(text) => {
            summary.insert("chars".to_string(), json!(text.chars().count()));
        }
        Value::Array(items) => {
            summary.insert("items".to_string(), json!(items.len()));
        }
        Value::Object(map) => {
            let mut keys: Vec<_> = map.keys().map(String::as_str).collect();
            keys.sort_unstable();
            summary.insert(
                "keys".to_string(),
                json!(keys.into_iter().take(24).collect::<Vec<_>>()),
            );
            summary.insert("keyCount".to_string(), json!(map.len()));
        }
        _ => {}
    }
    Value::Object(summary)
}

fn collect_strings(value: &Value, key_names: &[&str]) -> Vec<String> {
    let mut found = BTreeSet::new();
    collect_strings_into(value, key_names, &mut found);
    found.into_iter().collect()
}

fn collect_strings_into(value: &Value, key_names: &[&str], found: &mut BTreeSet<String>) {
    match value {
        Value::Object(map) => {
            for (key, item) in map {
                if contains(key_names, key) {
                    if let Some(text) = item.as_str() {
                        found.insert(text.to_string());
                    }
                } else if item.is_object() || item.is_array() {
                    collect_strings_into(item, key_names, found);
                }
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_strings_into(item, key_names, found);
            }
        }
        _ => {}
    }
}

fn collect_command_telemetry(value: &Value) -> Vec<Value> {
    let mut found = Vec::new();
    collect_command_telemetry_into(value, &mut found);
    found
}

fn collect_command_telemetry_into(value: &Value, found: &mut Vec<Value>) {
    match value {
        Value::Object(map) => {
            if map.get("type").and_then(Value::as_str) == Some("commandExecution")
                && let Some(command) = map.get("command").and_then(Value::as_str)
            {
                let mut info = summarize_command_text(command);
                if let Some(object) = info.as_object_mut() {
                    object.insert(
                        "cwd".to_string(),
                        map.get("cwd").cloned().unwrap_or(Value::Null),
                    );
                    object.insert(
                        "status".to_string(),
                        map.get("status").cloned().unwrap_or(Value::Null),
                    );
                    object.insert(
                        "exitCode".to_string(),
                        map.get("exitCode").cloned().unwrap_or(Value::Null),
                    );
                    object.insert(
                        "durationMs".to_string(),
                        map.get("durationMs").cloned().unwrap_or(Value::Null),
                    );
                }
                found.push(info);
            }
            for item in map
                .values()
                .filter(|item| item.is_object() || item.is_array())
            {
                collect_command_telemetry_into(item, found);
            }
        }
        Value::Array(items) => {
            for item in items {
                collect_command_telemetry_into(item, found);
            }
        }
        _ => {}
    }
}

fn summarize_command_text(command: &str) -> Value {
    let compact = command
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    let verbs = COMMAND_VERBS
        .iter()
        .filter(|verb| contains_token(command, verb))
        .copied()
        .collect::<Vec<_>>();
    let has_write_verb = verbs.iter().any(|verb| {
        ["New-Item", "Set-Content", "apply_patch"]
            .iter()
            .any(|write| verb.eq_ignore_ascii_case(write))
    });
    json!({
        "chars": command.chars().count(),
        "lines": std::cmp::max(command.lines().count(), 1),
        "preview": compact.chars().take(240).collect::<String>(),
        "truncated": compact.chars().count() > 240,
        "verbs": verbs,
        "hasWriteVerb": has_write_verb,
    })
}

fn contains_token(haystack: &str, needle: &str) -> bool {
    let haystack = haystack.to_lowercase();
    let needle = needle.to_lowercase();
    let bytes = haystack.as_bytes();
    let mut start = 0;
    while let Some(offset) = haystack[start..].find(&needle) {
        let idx = start + offset;
        let end = idx + needle.len();
        let before_ok = idx == 0 || !is_word(bytes[idx - 1] as char);
        let after_ok = end >= bytes.len() || !is_word(bytes[end] as char);
        if before_ok && after_ok {
            return true;
        }
        start = end;
    }
    false
}

fn value_key(value: &Value) -> String {
    value
        .as_str()
        .map(ToString::to_string)
        .unwrap_or_else(|| value.to_string())
}

fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "list",
        Value::Object(_) => "dict",
    }
}

fn contains(keys: &[&str], key: &str) -> bool {
    keys.iter().any(|candidate| candidate == &key)
}

fn is_word(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '-'
}
