use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

#[allow(dead_code)]
#[path = "epiphany-mvp-status.rs"]
mod status_cli;

const DEFAULT_APP_SERVER: &str = r"C:\Users\Meta\.cargo-target-codex\debug\codex-app-server.exe";
const WATCHED_RELATIVE_PATH: &str = "src/reorient_target.rs";

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
    workspace: PathBuf,
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
                .join("phase6-reorient-codex-home"),
            workspace: root
                .join(".epiphany-smoke")
                .join("phase6-reorient-workspace"),
            result: root
                .join(".epiphany-smoke")
                .join("phase6-reorient-smoke-result.json"),
            transcript: root
                .join(".epiphany-smoke")
                .join("phase6-reorient-smoke-transcript.jsonl"),
            stderr: root
                .join(".epiphany-smoke")
                .join("phase6-reorient-smoke-server.stderr.log"),
        };
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--app-server" => parsed.app_server = take_path(&mut args, "--app-server")?,
                "--codex-home" => parsed.codex_home = take_path(&mut args, "--codex-home")?,
                "--workspace" => parsed.workspace = take_path(&mut args, "--workspace")?,
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
    let workspace = status_cli::absolute_path(&args.workspace)?;
    let result_path = status_cli::absolute_path(&args.result)?;
    let transcript_path = status_cli::absolute_path(&args.transcript)?;
    let stderr_path = status_cli::absolute_path(&args.stderr)?;
    reset_smoke_paths(
        &root,
        &[
            codex_home.clone(),
            workspace.clone(),
            result_path.clone(),
            transcript_path.clone(),
            stderr_path.clone(),
        ],
    )?;
    fs::create_dir_all(&codex_home)
        .with_context(|| format!("failed to create {}", codex_home.display()))?;
    let watched_file = prepare_workspace(&workspace)?;

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
                "name": "epiphany-phase6-reorient-smoke",
                "title": "Epiphany Phase 6 Reorient Smoke",
                "version": "0.1.0",
            },
            "capabilities": {"experimentalApi": true},
        })),
        true,
    )?;
    client.send("initialized", None, false)?;
    let started = client.send(
        "thread/start",
        Some(json!({"cwd": workspace, "ephemeral": true})),
        true,
    )?;
    let thread_id = started
        .pointer("/thread/id")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("thread/start response missing thread.id"))?
        .to_string();

    let missing_notification_start = client.notification_len();
    let missing_view = client.send(
        "thread/epiphany/view",
        Some(json!({"threadId": thread_id, "lenses": ["reorient"]})),
        true,
    )?;
    let missing_response = missing_view
        .get("reorient")
        .ok_or_else(|| anyhow!("view response missing reorient lens"))?;
    assert_missing_reorient(&missing_response)?;
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
            "patch": reorient_patch(),
        })),
        true,
    )?;
    require(
        update.get("revision").and_then(Value::as_u64) == Some(1),
        "reorient smoke patch should advance revision to 1",
    )?;
    client.wait_for_notification(
        "thread/epiphany/stateUpdated",
        update_notification_start,
        Duration::from_secs(10),
    )?;

    let ready_notification_start = client.notification_len();
    let ready_view = client.send(
        "thread/epiphany/view",
        Some(json!({"threadId": thread_id, "lenses": ["reorient"]})),
        true,
    )?;
    let ready_response = ready_view
        .get("reorient")
        .ok_or_else(|| anyhow!("view response missing reorient lens"))?;
    assert_ready_reorient(&ready_response)?;
    client.require_no_notification(
        "thread/epiphany/stateUpdated",
        ready_notification_start,
        Duration::from_secs(1),
    )?;

    fs::write(
        watched_file,
        "pub fn reorient_target() -> &'static str {\n    \"after\"\n}\n",
    )?;

    let regather_notification_start = client.notification_len();
    let regather_response = wait_for_regather_reorient(&mut client, &thread_id)?;
    assert_regather_reorient(&regather_response)?;
    client.require_no_notification(
        "thread/epiphany/stateUpdated",
        regather_notification_start,
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
        "reorient reflection should not mutate durable state",
    )?;

    let result = json!({
        "threadId": thread_id,
        "codexHome": codex_home,
        "workspace": workspace,
        "missingAction": missing_response["decision"]["action"],
        "readyAction": ready_response["decision"]["action"],
        "regatherAction": regather_response["decision"]["action"],
        "regatherReasons": regather_response["decision"]["reasons"],
        "checkpointChangedPaths": regather_response["decision"]["checkpointChangedPaths"],
        "activeFrontierNodeIds": regather_response["decision"]["activeFrontierNodeIds"],
        "stateUpdatedNotificationCount": client.notification_count(
            "thread/epiphany/stateUpdated",
            regather_notification_start,
        ),
        "finalReadRevision": final_read["thread"]["epiphanyState"]["revision"],
    });
    write_json(&result_path, &result)?;
    Ok(result)
}

fn prepare_workspace(workspace: &Path) -> Result<PathBuf> {
    fs::create_dir_all(workspace)?;
    let watched_file = workspace.join(WATCHED_RELATIVE_PATH);
    fs::create_dir_all(watched_file.parent().expect("watched path has parent"))?;
    fs::write(
        &watched_file,
        "pub fn reorient_target() -> &'static str {\n    \"before\"\n}\n",
    )?;
    Ok(watched_file)
}

fn reorient_patch() -> Value {
    let code_ref = json!({
        "path": WATCHED_RELATIVE_PATH,
        "start_line": 1,
        "end_line": 3,
        "symbol": "reorient_target"
    });
    json!({
        "objective": "Decide whether a durable checkpoint still deserves to be resumed after rehydrate.",
        "activeSubgoalId": "phase6-reorient-smoke",
        "subgoals": [{
            "id": "phase6-reorient-smoke",
            "title": "Live-smoke CRRC reorientation policy",
            "status": "active",
            "summary": "Resume when the checkpoint is still aligned; regather when the touched file proves it isn't."
        }],
        "graphs": {
            "architecture": {
                "nodes": [{
                    "id": "reorient-target",
                    "title": "Reorient target",
                    "purpose": "Map the file the watcher will touch so reorientation can notice drift.",
                    "code_refs": [code_ref.clone()]
                }]
            },
            "dataflow": {"nodes": []},
            "links": []
        },
        "graphFrontier": {
            "active_node_ids": ["reorient-target"],
            "dirty_paths": []
        },
        "graphCheckpoint": {
            "checkpoint_id": "ck-reorient-1",
            "graph_revision": 1,
            "summary": "Reorientation smoke graph checkpoint",
            "frontier_node_ids": ["reorient-target"]
        },
        "investigationCheckpoint": {
            "checkpoint_id": "ix-reorient-1",
            "kind": "source_gathering",
            "disposition": "resume_ready",
            "focus": "Verify the touched file before broad edits.",
            "summary": "This checkpoint should remain resumable until the watched source moves.",
            "next_action": "Resume the bounded slice if the watched source still matches the checkpoint.",
            "captured_at_turn_id": "turn-phase6-reorient",
            "code_refs": [code_ref]
        },
        "churn": {
            "understanding_status": "ready",
            "diff_pressure": "low",
            "graph_freshness": "fresh",
            "unexplained_writes": 0
        }
    })
}

fn assert_missing_reorient(response: &Value) -> Result<()> {
    require(
        response.get("source").and_then(Value::as_str) == Some("live"),
        "missing reorient response should report live source",
    )?;
    require(
        response.get("stateStatus").and_then(Value::as_str) == Some("missing"),
        "missing reorient response should report missing state",
    )?;
    let decision = &response["decision"];
    require(
        decision.get("action").and_then(Value::as_str) == Some("regather"),
        "missing reorient response should regather",
    )?;
    require(
        decision.get("checkpointStatus").and_then(Value::as_str) == Some("missing"),
        "missing reorient response should report missing checkpoint",
    )?;
    require(
        string_array_eq(
            decision.get("reasons").and_then(Value::as_array),
            &["missingState", "missingCheckpoint"],
        ),
        "missing reorient response should explain missing state and checkpoint",
    )
}

fn assert_ready_reorient(response: &Value) -> Result<()> {
    require(
        response.get("source").and_then(Value::as_str) == Some("live"),
        "ready reorient response should report live source",
    )?;
    require(
        response.get("stateStatus").and_then(Value::as_str) == Some("ready"),
        "ready reorient response should report ready state",
    )?;
    require(
        response.get("stateRevision").and_then(Value::as_u64) == Some(1),
        "ready reorient response should preserve revision identity",
    )?;
    let decision = &response["decision"];
    require(
        decision.get("action").and_then(Value::as_str) == Some("resume"),
        "clean checkpoint should remain resumable",
    )?;
    require(
        decision.get("checkpointStatus").and_then(Value::as_str) == Some("resumeReady"),
        "ready reorient response should report resume-ready checkpoint status",
    )?;
    require(
        string_array_eq(
            decision.get("reasons").and_then(Value::as_array),
            &["checkpointReady"],
        ),
        "ready reorient response should explain that the checkpoint is still aligned",
    )?;
    require(
        decision.get("checkpointId").and_then(Value::as_str) == Some("ix-reorient-1"),
        "ready reorient response should expose the checkpoint id",
    )?;
    require(
        decision.get("nextAction").and_then(Value::as_str)
            == Some("Resume the bounded slice if the watched source still matches the checkpoint."),
        "ready reorient response should preserve checkpoint next action",
    )?;
    require(
        decision.get("watcherStatus").and_then(Value::as_str) == Some("clean"),
        "ready reorient response should report a clean watcher before drift",
    )
}

fn wait_for_regather_reorient(
    client: &mut status_cli::AppServerClient,
    thread_id: &str,
) -> Result<Value> {
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut last_response = Value::Null;
    while Instant::now() < deadline {
        let response = client.send(
            "thread/epiphany/view",
            Some(json!({"threadId": thread_id, "lenses": ["reorient"]})),
            true,
        )?;
        let reorient = response
            .get("reorient")
            .ok_or_else(|| anyhow!("view response missing reorient lens"))?;
        if reorient.pointer("/decision/action").and_then(Value::as_str) == Some("regather") {
            return Ok(reorient.clone());
        }
        last_response = response;
        thread::sleep(Duration::from_millis(200));
    }
    Err(anyhow!(
        "reorientation policy did not switch to regather before timeout; last response: {last_response}"
    ))
}

fn assert_regather_reorient(response: &Value) -> Result<()> {
    let decision = &response["decision"];
    require(
        decision.get("action").and_then(Value::as_str) == Some("regather"),
        "touched checkpoint path should force regather",
    )?;
    require(
        decision.get("checkpointStatus").and_then(Value::as_str) == Some("resumeReady"),
        "regather response should preserve the underlying checkpoint disposition",
    )?;
    require(
        string_array_eq(
            decision.get("reasons").and_then(Value::as_array),
            &["checkpointPathsChanged", "frontierChanged"],
        ),
        "regather response should explain watcher path and frontier drift",
    )?;
    let changed_paths = decision
        .get("checkpointChangedPaths")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("regather response missing changed paths"))?;
    require(
        changed_paths
            .iter()
            .filter_map(Value::as_str)
            .map(|path| path.replace('\\', "/"))
            .any(|path| path == WATCHED_RELATIVE_PATH),
        "regather response should report the changed checkpoint path",
    )?;
    require(
        string_array_eq(
            decision
                .get("activeFrontierNodeIds")
                .and_then(Value::as_array),
            &["reorient-target"],
        ),
        "regather response should report touched frontier nodes",
    )?;
    require(
        decision
            .get("note")
            .and_then(Value::as_str)
            .is_some_and(|note| note.starts_with("Re-gather before editing:")),
        "regather response should explain why the checkpoint is no longer safe to resume",
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
