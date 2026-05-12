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
                .join("phase6-pressure-codex-home"),
            result: root
                .join(".epiphany-smoke")
                .join("phase6-pressure-smoke-result.json"),
            transcript: root
                .join(".epiphany-smoke")
                .join("phase6-pressure-smoke-transcript.jsonl"),
            stderr: root
                .join(".epiphany-smoke")
                .join("phase6-pressure-smoke-server.stderr.log"),
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
                "name": "epiphany-phase6-pressure-smoke",
                "title": "Epiphany Phase 6 Pressure Smoke",
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

    let notification_start = client.notification_len();
    let response = client.send(
        "thread/epiphany/view",
        Some(json!({"threadId": thread_id, "lenses": ["pressure"]})),
        true,
    )?;
    require(
        response.get("threadId").and_then(Value::as_str) == Some(&thread_id),
        "pressure response should echo thread id",
    )?;
    assert_unknown_pressure(&response)?;
    client.require_no_notification(
        "thread/epiphany/stateUpdated",
        notification_start,
        Duration::from_secs(1),
    )?;

    let final_read = client.send(
        "thread/read",
        Some(json!({"threadId": thread_id, "includeTurns": false})),
        true,
    )?;
    require(
        final_read.pointer("/thread/epiphanyState").is_none(),
        "pressure reflection should not create Epiphany state",
    )?;

    let result = json!({
        "threadId": thread_id,
        "codexHome": codex_home,
        "source": "thread/epiphany/view",
        "status": response["pressure"]["status"],
        "level": response["pressure"]["level"],
        "basis": response["pressure"]["basis"],
        "shouldPrepareCompaction": response["pressure"]["shouldPrepareCompaction"],
        "stateUpdatedNotificationCount": client.notification_count(
            "thread/epiphany/stateUpdated",
            notification_start,
        ),
        "finalReadHasEpiphanyState": final_read.pointer("/thread/epiphanyState").is_some(),
    });
    write_json(&result_path, &result)?;
    Ok(result)
}

fn assert_unknown_pressure(response: &Value) -> Result<()> {
    let pressure = response
        .get("pressure")
        .ok_or_else(|| anyhow!("pressure response missing pressure object"))?;
    require(
        pressure.get("status").and_then(Value::as_str) == Some("unknown"),
        "fresh pressure should be unknown",
    )?;
    require(
        pressure.get("level").and_then(Value::as_str) == Some("unknown"),
        "fresh pressure level should be unknown",
    )?;
    require(
        pressure.get("basis").and_then(Value::as_str) == Some("unknown"),
        "fresh pressure basis should be unknown",
    )?;
    require(
        pressure
            .get("shouldPrepareCompaction")
            .and_then(Value::as_bool)
            == Some(false),
        "unknown pressure must not recommend compaction prep",
    )?;
    require(
        pressure.get("usedTokens").is_none(),
        "fresh pressure should not invent token usage",
    )
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
