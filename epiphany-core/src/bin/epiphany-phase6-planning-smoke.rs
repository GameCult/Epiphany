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
                .join("phase6-planning-codex-home"),
            result: root
                .join(".epiphany-smoke")
                .join("phase6-planning-smoke-result.json"),
            transcript: root
                .join(".epiphany-smoke")
                .join("phase6-planning-smoke-transcript.jsonl"),
            stderr: root
                .join(".epiphany-smoke")
                .join("phase6-planning-smoke-server.stderr.log"),
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
                "name": "epiphany-phase6-planning-smoke",
                "title": "Epiphany Phase 6 Planning Smoke",
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
    let missing_view = client.send(
        "thread/epiphany/view",
        Some(json!({"threadId": thread_id, "lenses": ["planning"]})),
        true,
    )?;
    let missing_response = missing_view
        .get("planning")
        .ok_or_else(|| anyhow!("view response missing planning lens"))?;
    require(
        missing_response.get("threadId").and_then(Value::as_str) == Some(&thread_id),
        "planning response should echo thread id",
    )?;
    assert_missing_planning(&missing_response)?;
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
            "patch": planning_patch(&root),
        })),
        true,
    )?;
    require(
        update.get("revision").and_then(Value::as_u64) == Some(1),
        "planning patch should advance revision to 1",
    )?;
    let update_notification = client.wait_for_notification(
        "thread/epiphany/stateUpdated",
        update_notification_start,
        Duration::from_secs(10),
    )?;
    require(
        string_array_contains(
            update_notification
                .pointer("/params/changedFields")
                .and_then(Value::as_array),
            "planning",
        ),
        "planning update should report the planning changed field",
    )?;

    let planning_notification_start = client.notification_len();
    let ready_view = client.send(
        "thread/epiphany/view",
        Some(json!({"threadId": thread_id, "lenses": ["planning"]})),
        true,
    )?;
    let ready_response = ready_view
        .get("planning")
        .ok_or_else(|| anyhow!("view response missing planning lens"))?;
    assert_ready_planning(&ready_response)?;
    client.require_no_notification(
        "thread/epiphany/stateUpdated",
        planning_notification_start,
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
        "planning reflection should not mutate state revision",
    )?;

    let result = json!({
        "threadId": thread_id,
        "codexHome": codex_home,
        "missingStateStatus": missing_response["stateStatus"],
        "readyStateStatus": ready_response["stateStatus"],
        "readyRevision": ready_response["stateRevision"],
        "captureCount": ready_response["summary"]["captureCount"],
        "githubIssueCaptureCount": ready_response["summary"]["githubIssueCaptureCount"],
        "backlogItemCount": ready_response["summary"]["backlogItemCount"],
        "objectiveDraftCount": ready_response["summary"]["objectiveDraftCount"],
        "planningNotificationCount": client.notification_count(
            "thread/epiphany/stateUpdated",
            planning_notification_start,
        ),
        "finalReadRevision": final_read["thread"]["epiphanyState"]["revision"],
    });
    write_json(&result_path, &result)?;
    Ok(result)
}

fn planning_patch(root: &Path) -> Value {
    let github_source = json!({
        "kind": "github_issue",
        "provider": "github",
        "repo": "GameCult/EpiphanyAgent",
        "issue_number": 42,
        "url": "https://github.com/GameCult/EpiphanyAgent/issues/42",
        "state": "open",
        "labels": ["planning", "mvp"],
        "assignees": ["Meta"],
        "author": "Meta",
        "created_at": "2026-05-01T10:00:00Z",
        "updated_at": "2026-05-01T10:05:00Z",
        "imported_at": "2026-05-01T10:10:00Z"
    });
    let chat_source = json!({
        "kind": "chat",
        "uri": "codex://threads/planning-smoke",
        "external_id": "turn-planning-smoke"
    });
    json!({
        "objective": "Keep planning state reviewable before adopting an implementation objective.",
        "planning": {
            "workspace_root": root,
            "captures": [
                {
                    "id": "capture-github-planning",
                    "title": "Import GitHub issue into planning inbox",
                    "body": "Issues should become planning captures before becoming objectives.",
                    "confidence": "observed",
                    "status": "inbox",
                    "speaker": "human",
                    "tags": ["github", "backlog"],
                    "source": github_source.clone(),
                    "created_at": "2026-05-01T10:10:00Z",
                    "updated_at": "2026-05-01T10:10:00Z"
                },
                {
                    "id": "capture-chat-planning",
                    "title": "Draft objectives from chat only after bounding them",
                    "body": "User discussion remains planning material until it becomes a firm artifact.",
                    "confidence": "medium",
                    "status": "triaged",
                    "speaker": "human",
                    "tags": ["chat", "objective"],
                    "source": chat_source.clone(),
                    "created_at": "2026-05-01T10:12:00Z",
                    "updated_at": "2026-05-01T10:12:00Z"
                }
            ],
            "backlog_items": [{
                "id": "backlog-planning-dashboard",
                "title": "Expose planning state in the Epiphany dashboard",
                "kind": "feature",
                "summary": "Show captures, backlog, roadmap streams, and objective drafts.",
                "status": "ready",
                "horizon": "near",
                "priority": {
                    "value": "high",
                    "rationale": "Planning needs a visible user review loop before automation.",
                    "impact": "user-steering",
                    "urgency": "soon",
                    "confidence": "medium",
                    "effort": "small",
                    "unblocks": ["objective-adoption"]
                },
                "confidence": "medium",
                "product_area": "epiphany-gui",
                "lane_hints": ["imagination", "hands", "soul"],
                "acceptance_sketch": ["Dashboard can render typed planning records."],
                "source_refs": [github_source, chat_source],
                "updated_at": "2026-05-01T10:15:00Z"
            }],
            "roadmap_streams": [{
                "id": "stream-user-steering",
                "title": "Human-steered planning",
                "purpose": "Keep future work visible and bounded before implementation starts.",
                "status": "active",
                "item_ids": ["backlog-planning-dashboard"],
                "near_term_focus": "backlog-planning-dashboard",
                "review_cadence": "per objective"
            }],
            "objective_drafts": [{
                "id": "draft-planning-dashboard",
                "title": "Build the planning dashboard slice",
                "summary": "Render planning records and let the user adopt one bounded objective.",
                "source_item_ids": ["backlog-planning-dashboard"],
                "scope": {
                    "includes": ["read-only planning projection", "objective draft review"],
                    "excludes": ["automatic objective adoption"]
                },
                "acceptance_criteria": [
                    "Planning records render without changing thread state.",
                    "A draft objective remains review-gated."
                ],
                "evidence_required": ["live smoke", "GUI screenshot"],
                "lane_plan": {
                    "imagination": "organize planning records into a bounded objective candidate",
                    "eyes": "check prior art and GitHub issue metadata shape",
                    "hands": "wire dashboard controls after the read-only surface is stable",
                    "soul": "verify no planning record silently becomes active objective"
                },
                "risks": ["accidental objective adoption"],
                "review_gates": ["human adoption"],
                "status": "draft"
            }]
        }
    })
}

fn assert_missing_planning(response: &Value) -> Result<()> {
    require(
        response.get("source").and_then(Value::as_str) == Some("live"),
        "missing planning should report live source",
    )?;
    require(
        response.get("stateStatus").and_then(Value::as_str) == Some("missing"),
        "missing planning should report missing state",
    )?;
    require(
        response.get("stateRevision").is_none(),
        "missing planning should not invent a revision",
    )?;
    require(
        response.get("planning") == Some(&json!({})),
        "missing planning should return empty planning",
    )?;
    let summary = &response["summary"];
    require(
        summary.get("captureCount").and_then(Value::as_u64) == Some(0),
        "missing planning should not invent captures",
    )?;
    require(
        summary.get("backlogItemCount").and_then(Value::as_u64) == Some(0),
        "missing planning should not invent backlog",
    )?;
    require(
        summary.get("activeObjective").is_none(),
        "missing planning should not invent objective",
    )
}

fn assert_ready_planning(response: &Value) -> Result<()> {
    require(
        response.get("source").and_then(Value::as_str) == Some("live"),
        "ready planning should report live source",
    )?;
    require(
        response.get("stateStatus").and_then(Value::as_str) == Some("ready"),
        "ready planning should report ready state",
    )?;
    require(
        response.get("stateRevision").and_then(Value::as_u64) == Some(1),
        "ready planning should expose current revision",
    )?;
    let planning = &response["planning"];
    require(
        ids_eq(
            planning.get("captures").and_then(Value::as_array),
            &["capture-github-planning", "capture-chat-planning"],
        ),
        "planning should preserve captures",
    )?;
    require(
        planning
            .pointer("/captures/0/source/kind")
            .and_then(Value::as_str)
            == Some("github_issue"),
        "planning should preserve GitHub issue source kind",
    )?;
    require(
        planning
            .pointer("/captures/0/source/issue_number")
            .and_then(Value::as_u64)
            == Some(42),
        "planning should preserve GitHub issue number",
    )?;
    require(
        ids_eq(
            planning.get("backlog_items").and_then(Value::as_array),
            &["backlog-planning-dashboard"],
        ),
        "planning should preserve backlog items",
    )?;
    require(
        planning
            .pointer("/roadmap_streams/0/near_term_focus")
            .and_then(Value::as_str)
            == Some("backlog-planning-dashboard"),
        "planning should preserve roadmap near-term focus",
    )?;
    require(
        planning
            .pointer("/objective_drafts/0/status")
            .and_then(Value::as_str)
            == Some("draft"),
        "planning should preserve draft objective state",
    )?;
    let summary = &response["summary"];
    require(
        summary.get("captureCount").and_then(Value::as_u64) == Some(2),
        "planning summary should count captures",
    )?;
    require(
        summary.get("pendingCaptureCount").and_then(Value::as_u64) == Some(1),
        "planning summary should count inbox captures",
    )?;
    require(
        summary
            .get("githubIssueCaptureCount")
            .and_then(Value::as_u64)
            == Some(1),
        "planning summary should count GitHub captures",
    )?;
    require(
        summary.get("backlogItemCount").and_then(Value::as_u64) == Some(1),
        "planning summary should count backlog",
    )?;
    require(
        summary.get("readyBacklogItemCount").and_then(Value::as_u64) == Some(1),
        "planning summary should count ready backlog",
    )?;
    require(
        summary.get("roadmapStreamCount").and_then(Value::as_u64) == Some(1),
        "planning summary should count roadmap streams",
    )?;
    require(
        summary.get("objectiveDraftCount").and_then(Value::as_u64) == Some(1),
        "planning summary should count drafts",
    )?;
    require(
        summary.get("draftObjectiveCount").and_then(Value::as_u64) == Some(1),
        "planning summary should count draft status",
    )?;
    require(
        summary
            .get("activeObjective")
            .and_then(Value::as_str)
            .is_some_and(|objective| objective.starts_with("Keep planning state reviewable")),
        "planning summary should expose the active thread objective separately",
    )?;
    require(
        summary
            .get("note")
            .and_then(Value::as_str)
            .is_some_and(|note| note.contains("human explicitly adopts an objective")),
        "planning summary should remind clients that adoption is explicit",
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

fn string_array_contains(values: Option<&Vec<Value>>, expected: &str) -> bool {
    values.is_some_and(|values| values.iter().any(|value| value.as_str() == Some(expected)))
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
