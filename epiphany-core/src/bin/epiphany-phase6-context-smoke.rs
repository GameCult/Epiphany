use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[allow(dead_code)]
#[path = "epiphany-mvp-status.rs"]
mod status_cli;

const DEFAULT_APP_SERVER: &str = r"C:\Users\Meta\.cargo-target-codex\debug\codex-app-server.exe";

fn main() -> Result<()> {
    let args = Args::parse()?;
    let result = run_smoke(&args)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Debug)]
struct Args {
    app_server: PathBuf,
    codex_home: PathBuf,
    result: PathBuf,
    transcript: PathBuf,
    stderr: PathBuf,
}

impl Args {
    fn parse() -> Result<Self> {
        let root = env::current_dir().context("failed to resolve current dir")?;
        let mut parsed = Self {
            app_server: PathBuf::from(DEFAULT_APP_SERVER),
            codex_home: root
                .join(".epiphany-smoke")
                .join("phase6-context-codex-home"),
            result: root
                .join(".epiphany-smoke")
                .join("phase6-context-smoke-result.json"),
            transcript: root
                .join(".epiphany-smoke")
                .join("phase6-context-smoke-transcript.jsonl"),
            stderr: root
                .join(".epiphany-smoke")
                .join("phase6-context-smoke-server.stderr.log"),
        };
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--app-server" => parsed.app_server = take_path(&mut args, "--app-server")?,
                "--codex-home" => parsed.codex_home = take_path(&mut args, "--codex-home")?,
                "--result" => parsed.result = take_path(&mut args, "--result")?,
                "--transcript" => parsed.transcript = take_path(&mut args, "--transcript")?,
                "--stderr" => parsed.stderr = take_path(&mut args, "--stderr")?,
                other => return Err(anyhow!("unknown argument: {other}")),
            }
        }
        Ok(parsed)
    }
}

fn run_smoke(args: &Args) -> Result<Value> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let app_server = status_cli::absolute_path(&args.app_server)?;
    if !app_server.exists() {
        return Err(anyhow!(
            "codex app-server binary not found: {}",
            app_server.display()
        ));
    }
    let codex_home = status_cli::absolute_path(&args.codex_home)?;
    let result_path = status_cli::absolute_path(&args.result)?;
    let transcript_path = status_cli::absolute_path(&args.transcript)?;
    let stderr_path = status_cli::absolute_path(&args.stderr)?;
    reset_smoke_paths(
        &root,
        &[
            codex_home.clone(),
            result_path.clone(),
            transcript_path.clone(),
            stderr_path.clone(),
        ],
    )?;
    fs::create_dir_all(&codex_home)
        .with_context(|| format!("failed to create {}", codex_home.display()))?;

    let mut client = status_cli::AppServerClient::start(
        &app_server,
        &codex_home,
        &transcript_path,
        &stderr_path,
    )?;
    client.send(
        "initialize",
        Some(json!({
            "clientInfo": {
                "name": "epiphany-phase6-context-smoke",
                "title": "Epiphany Phase 6 Context Smoke",
                "version": "0.1.0",
            },
            "capabilities": {"experimentalApi": true},
        })),
        true,
    )?;
    client.send("initialized", None, false)?;
    let started = client.send(
        "thread/start",
        Some(json!({"cwd": root.join("epiphany-core"), "ephemeral": true})),
        true,
    )?;
    let thread_id = started
        .pointer("/thread/id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("thread/start response missing thread.id"))?
        .to_string();

    let missing_notification_start = client.notification_len();
    let missing_response = client.send(
        "thread/epiphany/context",
        Some(json!({"threadId": thread_id, "graphNodeIds": ["context-surface"]})),
        true,
    )?;
    require(
        missing_response.get("threadId").and_then(Value::as_str) == Some(&thread_id),
        "context response should echo thread id",
    )?;
    assert_missing_context(&missing_response)?;
    client.require_no_notification(
        "thread/epiphany/stateUpdated",
        missing_notification_start,
        Duration::from_secs(1),
    )?;

    let update_notification_start = client.notification_len();
    let update = client.send(
        "thread/epiphany/update",
        Some(json!({
            "threadId": thread_id,
            "expectedRevision": 0,
            "patch": context_patch(),
        })),
        true,
    )?;
    require(
        update.get("revision").and_then(Value::as_u64) == Some(1),
        "context smoke patch should advance revision to 1",
    )?;
    client.wait_for_notification(
        "thread/epiphany/stateUpdated",
        update_notification_start,
        Duration::from_secs(10),
    )?;

    let context_notification_start = client.notification_len();
    let ready_response = client.send(
        "thread/epiphany/context",
        Some(json!({
            "threadId": thread_id,
            "graphNodeIds": ["missing-node"],
            "graphEdgeIds": ["missing-edge"],
            "observationIds": ["obs-context", "missing-observation"],
            "evidenceIds": ["ev-context-extra", "missing-evidence"],
        })),
        true,
    )?;
    assert_ready_context(&ready_response)?;
    client.require_no_notification(
        "thread/epiphany/stateUpdated",
        context_notification_start,
        Duration::from_secs(1),
    )?;

    let final_read = client.send(
        "thread/read",
        Some(json!({"threadId": thread_id, "includeTurns": false})),
        true,
    )?;
    require(
        final_read
            .pointer("/thread/epiphanyState/revision")
            .and_then(Value::as_u64)
            == Some(1),
        "context reflection should not mutate state revision",
    )?;

    let result = json!({
        "threadId": thread_id,
        "codexHome": codex_home,
        "missingStateStatus": missing_response["stateStatus"],
        "readyStateStatus": ready_response["stateStatus"],
        "readyRevision": ready_response["stateRevision"],
        "architectureNodeIds": ids(&ready_response["context"]["graph"]["architectureNodes"]),
        "investigationCheckpointId": ready_response["context"]["investigationCheckpoint"]["checkpoint_id"],
        "investigationDisposition": ready_response["context"]["investigationCheckpoint"]["disposition"],
        "evidenceIds": ids(&ready_response["context"]["evidence"]),
        "contextNotificationCount": client.notification_count(
            "thread/epiphany/stateUpdated",
            context_notification_start,
        ),
        "finalReadRevision": final_read["thread"]["epiphanyState"]["revision"],
    });
    write_json(&result_path, &result)?;
    Ok(result)
}

fn context_patch() -> Value {
    let code_ref = json!({
        "path": "app-server/src/codex_message_processor.rs",
        "start_line": 10791,
        "end_line": 10989,
        "symbol": "map_epiphany_context"
    });
    json!({
        "objective": "Expose a read-only Epiphany context shard without returning the full state.",
        "activeSubgoalId": "phase6-context-smoke",
        "investigationCheckpoint": {
            "checkpoint_id": "phase6-context-investigation",
            "kind": "source_gathering",
            "disposition": "regather_required",
            "focus": "Make stale planning obvious when context is re-read after compaction.",
            "summary": "Context reflection should expose the full durable checkpoint packet.",
            "next_action": "Re-gather source before editing if the packet no longer matches reality.",
            "captured_at_turn_id": "turn-phase6-context",
            "open_questions": ["How much checkpoint detail belongs in scene versus context?"],
            "code_refs": [code_ref.clone()],
            "evidence_ids": ["ev-context-linked"]
        },
        "subgoals": [{
            "id": "phase6-context-smoke",
            "title": "Live-smoke context shard reflection",
            "status": "active",
            "summary": "The app-server context surface should return targeted graph/evidence state."
        }],
        "graphs": {
            "architecture": {
                "nodes": [{
                    "id": "context-surface",
                    "title": "Context shard surface",
                    "purpose": "Return bounded state context for clients without becoming a writer.",
                    "code_refs": [code_ref.clone()]
                }],
                "edges": [{
                    "id": "context-edge",
                    "source_id": "context-surface",
                    "target_id": "context-surface",
                    "kind": "reflects",
                    "code_refs": [code_ref.clone()]
                }]
            },
            "dataflow": {
                "nodes": [{
                    "id": "typed-state",
                    "title": "Typed Epiphany state",
                    "purpose": "Remain authoritative while context shards reflect a slice."
                }]
            },
            "links": [{
                "dataflow_node_id": "typed-state",
                "architecture_node_id": "context-surface",
                "relationship": "bounded-reflection"
            }]
        },
        "graphFrontier": {
            "active_node_ids": ["context-surface"],
            "active_edge_ids": ["context-edge"]
        },
        "graphCheckpoint": {
            "checkpoint_id": "phase6-context-smoke",
            "graph_revision": 1,
            "summary": "Context shard reflection is the active Phase 6 smoke target.",
            "frontier_node_ids": ["context-surface"]
        },
        "evidence": [
            {
                "id": "ev-context-linked",
                "kind": "smoke-test",
                "status": "ok",
                "summary": "Context smoke linked evidence should come along with the observation.",
                "code_refs": [code_ref.clone()]
            },
            {
                "id": "ev-context-extra",
                "kind": "review",
                "status": "ok",
                "summary": "Context smoke direct evidence should be returned when requested.",
                "code_refs": [code_ref]
            }
        ],
        "observations": [{
            "id": "obs-context",
            "summary": "Context smoke observation should be selected by id.",
            "source_kind": "smoke",
            "status": "ok",
            "code_refs": [{
                "path": "app-server/src/codex_message_processor.rs",
                "start_line": 10791,
                "end_line": 10989,
                "symbol": "map_epiphany_context"
            }],
            "evidence_ids": ["ev-context-linked"]
        }]
    })
}

fn assert_missing_context(response: &Value) -> Result<()> {
    require(
        response.get("source").and_then(Value::as_str) == Some("live"),
        "missing context should report live source",
    )?;
    require(
        response.get("stateStatus").and_then(Value::as_str) == Some("missing"),
        "missing context should report missing state",
    )?;
    require(
        response.get("stateRevision").is_none(),
        "missing context should not invent a revision",
    )?;
    require(
        response.pointer("/context/graph") == Some(&json!({})),
        "missing context should not invent graph records",
    )?;
    require(
        string_array_eq(
            response
                .pointer("/missing/graphNodeIds")
                .and_then(Value::as_array),
            &["context-surface"],
        ),
        "missing context should echo requested missing node ids",
    )
}

fn assert_ready_context(response: &Value) -> Result<()> {
    require(
        response.get("source").and_then(Value::as_str) == Some("live"),
        "ready context should report live source",
    )?;
    require(
        response.get("stateStatus").and_then(Value::as_str) == Some("ready"),
        "ready context should report ready state",
    )?;
    require(
        response.get("stateRevision").and_then(Value::as_u64) == Some(1),
        "ready context should preserve state revision identity",
    )?;
    require(
        ids_eq(
            response
                .pointer("/context/graph/architectureNodes")
                .and_then(Value::as_array),
            &["context-surface"],
        ),
        "context should include active frontier architecture node",
    )?;
    require(
        ids_eq(
            response
                .pointer("/context/graph/architectureEdges")
                .and_then(Value::as_array),
            &["context-edge"],
        ),
        "context should include active frontier architecture edge",
    )?;
    require(
        response
            .pointer("/context/graph/links/0/architecture_node_id")
            .and_then(Value::as_str)
            == Some("context-surface"),
        "context should include links touching selected graph nodes",
    )?;
    require(
        string_array_eq(
            response
                .pointer("/context/frontier/active_node_ids")
                .and_then(Value::as_array),
            &["context-surface"],
        ),
        "context should include frontier when active frontier is requested by default",
    )?;
    require(
        response
            .pointer("/context/checkpoint/checkpoint_id")
            .and_then(Value::as_str)
            == Some("phase6-context-smoke"),
        "context should expose the current graph checkpoint",
    )?;
    require(
        response
            .pointer("/context/investigationCheckpoint/checkpoint_id")
            .and_then(Value::as_str)
            == Some("phase6-context-investigation"),
        "context should expose the investigation checkpoint id",
    )?;
    require(
        response
            .pointer("/context/investigationCheckpoint/disposition")
            .and_then(Value::as_str)
            == Some("regather_required"),
        "context should expose the checkpoint disposition",
    )?;
    require(
        ids_eq(
            response
                .pointer("/context/observations")
                .and_then(Value::as_array),
            &["obs-context"],
        ),
        "context should include requested observation",
    )?;
    require(
        ids_eq(
            response
                .pointer("/context/evidence")
                .and_then(Value::as_array),
            &["ev-context-linked", "ev-context-extra"],
        ),
        "context should include linked and directly requested evidence",
    )?;
    require(
        string_array_eq(
            response
                .pointer("/missing/graphNodeIds")
                .and_then(Value::as_array),
            &["missing-node"],
        ) && string_array_eq(
            response
                .pointer("/missing/graphEdgeIds")
                .and_then(Value::as_array),
            &["missing-edge"],
        ) && string_array_eq(
            response
                .pointer("/missing/observationIds")
                .and_then(Value::as_array),
            &["missing-observation"],
        ) && string_array_eq(
            response
                .pointer("/missing/evidenceIds")
                .and_then(Value::as_array),
            &["missing-evidence"],
        ),
        "context should report unresolved requested ids",
    )
}

fn ids(value: &Value) -> Value {
    json!(
        value
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .filter_map(|item| item.get("id").and_then(Value::as_str))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    )
}

fn ids_eq(values: Option<&Vec<Value>>, expected: &[&str]) -> bool {
    values.is_some_and(|values| {
        values.len() == expected.len()
            && values
                .iter()
                .zip(expected)
                .all(|(value, expected)| value.get("id").and_then(Value::as_str) == Some(*expected))
    })
}

fn string_array_eq(values: Option<&Vec<Value>>, expected: &[&str]) -> bool {
    values.is_some_and(|values| {
        values.len() == expected.len()
            && values
                .iter()
                .zip(expected)
                .all(|(value, expected)| value.as_str() == Some(*expected))
    })
}

fn reset_smoke_paths(root: &Path, paths: &[PathBuf]) -> Result<()> {
    let smoke_root = root.join(".epiphany-smoke").canonicalize().or_else(|_| {
        let smoke_root = root.join(".epiphany-smoke");
        fs::create_dir_all(&smoke_root)?;
        smoke_root.canonicalize()
    })?;
    for path in paths {
        if !path.exists() {
            continue;
        }
        let resolved = path.canonicalize()?;
        if resolved == smoke_root || !resolved.starts_with(&smoke_root) {
            return Err(anyhow!(
                "refusing to delete non-smoke path: {}",
                path.display()
            ));
        }
        if path.is_dir() {
            fs::remove_dir_all(path)?;
        } else {
            fs::remove_file(path)?;
        }
    }
    Ok(())
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))?;
    Ok(())
}

fn require(condition: bool, message: &str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(anyhow!("{message}"))
    }
}

fn take_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("{name} requires a value"))
}
