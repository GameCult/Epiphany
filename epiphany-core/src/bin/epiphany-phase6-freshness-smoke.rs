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
                .join("phase6-freshness-codex-home"),
            result: root
                .join(".epiphany-smoke")
                .join("phase6-freshness-smoke-result.json"),
            transcript: root
                .join(".epiphany-smoke")
                .join("phase6-freshness-smoke-transcript.jsonl"),
            stderr: root
                .join(".epiphany-smoke")
                .join("phase6-freshness-smoke-server.stderr.log"),
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
                "name": "epiphany-phase6-freshness-smoke",
                "title": "Epiphany Phase 6 Freshness Smoke",
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
        "thread/epiphany/freshness",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    require(
        missing_response.get("threadId").and_then(Value::as_str) == Some(&thread_id),
        "freshness response should echo thread id",
    )?;
    assert_missing_state_freshness(&missing_response)?;
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
            "patch": freshness_patch(),
        })),
        true,
    )?;
    require(
        update.get("revision").and_then(Value::as_u64) == Some(1),
        "freshness smoke patch should advance revision to 1",
    )?;
    client.wait_for_notification(
        "thread/epiphany/stateUpdated",
        update_notification_start,
        Duration::from_secs(10),
    )?;

    let ready_notification_start = client.notification_len();
    let ready_response = client.send(
        "thread/epiphany/freshness",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    assert_ready_freshness(&ready_response)?;
    client.require_no_notification(
        "thread/epiphany/stateUpdated",
        ready_notification_start,
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
        "freshness reflection should not mutate state revision",
    )?;

    let result = json!({
        "threadId": thread_id,
        "codexHome": codex_home,
        "missingRetrievalStatus": missing_response["retrieval"]["status"],
        "missingGraphStatus": missing_response["graph"]["status"],
        "readyRevision": ready_response["stateRevision"],
        "readyRetrievalStatus": ready_response["retrieval"]["status"],
        "readyGraphStatus": ready_response["graph"]["status"],
        "readyDirtyPathCount": ready_response["graph"]["dirtyPathCount"],
        "freshnessNotificationCount": client.notification_count(
            "thread/epiphany/stateUpdated",
            ready_notification_start,
        ),
        "finalReadRevision": final_read["thread"]["epiphanyState"]["revision"],
    });
    write_json(&result_path, &result)?;
    Ok(result)
}

fn freshness_patch() -> Value {
    json!({
        "objective": "Expose read-only Epiphany freshness reflection without inventing watcher-driven invalidation.",
        "activeSubgoalId": "phase6-freshness-smoke",
        "subgoals": [{
            "id": "phase6-freshness-smoke",
            "title": "Live-smoke freshness reflection",
            "status": "active",
            "summary": "The freshness surface should reflect graph staleness and live retrieval state without mutation."
        }],
        "graphs": {
            "architecture": {
                "nodes": [{
                    "id": "freshness-surface",
                    "title": "Freshness reflection surface",
                    "purpose": "Expose exact retrieval and graph freshness pressure without becoming a scheduler.",
                    "code_refs": [{
                        "path": "app-server/src/codex_message_processor.rs",
                        "start_line": 4200,
                        "end_line": 4340,
                        "symbol": "thread_epiphany_freshness"
                    }]
                }]
            },
            "dataflow": {"nodes": []},
            "links": []
        },
        "graphFrontier": {
            "active_node_ids": ["freshness-surface"],
            "dirty_paths": ["app-server/src/codex_message_processor.rs"],
            "open_question_ids": ["q-freshness-gap"]
        },
        "graphCheckpoint": {
            "checkpoint_id": "ck-freshness-1",
            "graph_revision": 1,
            "summary": "Freshness smoke checkpoint",
            "frontier_node_ids": ["freshness-surface"],
            "open_question_ids": ["q-freshness-gap"]
        },
        "churn": {
            "understanding_status": "ready",
            "diff_pressure": "low",
            "graph_freshness": "stale",
            "unexplained_writes": 0
        }
    })
}

fn assert_missing_state_freshness(response: &Value) -> Result<()> {
    require(
        response.get("source").and_then(Value::as_str) == Some("live"),
        "freshness should report live source for a loaded thread",
    )?;
    require(
        response.get("stateRevision").is_none(),
        "missing-state freshness should not invent a revision",
    )?;
    let retrieval = &response["retrieval"];
    require(
        retrieval.get("status").and_then(Value::as_str) == Some("ready"),
        "live retrieval freshness should be available",
    )?;
    require(
        retrieval.get("note").and_then(Value::as_str) == Some("Retrieval catalog is ready."),
        "fresh live retrieval should report ready note",
    )?;
    let graph = &response["graph"];
    require(
        graph.get("status").and_then(Value::as_str) == Some("missing"),
        "missing Epiphany state should block graph freshness",
    )?;
    require(
        graph.get("note").and_then(Value::as_str)
            == Some("Epiphany state is missing, so graph freshness cannot be assessed."),
        "missing graph freshness should explain itself",
    )
}

fn assert_ready_freshness(response: &Value) -> Result<()> {
    require(
        response.get("source").and_then(Value::as_str) == Some("live"),
        "ready freshness should report live source",
    )?;
    require(
        response.get("stateRevision").and_then(Value::as_u64) == Some(1),
        "freshness should preserve state revision identity",
    )?;
    let retrieval = &response["retrieval"];
    require(
        retrieval.get("status").and_then(Value::as_str) == Some("ready"),
        "live retrieval freshness should stay ready for the smoke workspace",
    )?;
    require(
        retrieval.get("semanticAvailable").and_then(Value::as_bool) == Some(true),
        "live retrieval freshness should report semantic availability",
    )?;
    let graph = &response["graph"];
    require(
        graph.get("status").and_then(Value::as_str) == Some("stale"),
        "dirty graph frontier should report stale freshness",
    )?;
    require(
        graph.get("graphFreshness").and_then(Value::as_str) == Some("stale"),
        "graph freshness should expose the churn hint",
    )?;
    require(
        graph.get("checkpointId").and_then(Value::as_str) == Some("ck-freshness-1"),
        "graph freshness should expose the checkpoint id",
    )?;
    require(
        graph.get("dirtyPathCount").and_then(Value::as_u64) == Some(1),
        "graph freshness should count dirty paths",
    )?;
    require(
        string_array_eq(
            graph.get("dirtyPaths").and_then(Value::as_array),
            &["app-server/src/codex_message_processor.rs"],
        ),
        "graph freshness should expose dirty paths",
    )?;
    require(
        graph.get("openQuestionCount").and_then(Value::as_u64) == Some(1),
        "graph freshness should count open questions",
    )?;
    require(
        graph.get("openGapCount").and_then(Value::as_u64) == Some(0),
        "graph freshness should count open gaps",
    )
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
