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
const REORIENT_BINDING_ID: &str = "reorient-worker";
const GRAPH_NODE_ID: &str = "reorient-target";
const WATCHED_RELATIVE_PATH: &str = "src/reorient_target.rs";

fn main() -> Result<()> {
    let args = Args::parse()?;
    let summary = run_coordinator(&args)?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

#[derive(Debug)]
struct Args {
    app_server: PathBuf,
    thread_id: Option<String>,
    cwd: PathBuf,
    codex_home: PathBuf,
    artifact_dir: PathBuf,
    agent_memory_dir: PathBuf,
    mode: String,
    max_steps: usize,
    poll_seconds: f64,
    timeout_seconds: u64,
    max_runtime_seconds: u64,
    ephemeral: bool,
    auto_review: bool,
    bootstrap_smoke_state: bool,
    simulate_high_pressure: bool,
    simulate_source_drift: bool,
    dry_compact: bool,
}

impl Args {
    fn parse() -> Result<Self> {
        let root = env::current_dir().context("failed to resolve current dir")?;
        let mut args = env::args().skip(1);
        let mut parsed = Args {
            app_server: PathBuf::from(DEFAULT_APP_SERVER),
            thread_id: None,
            cwd: root.clone(),
            codex_home: env::var_os("CODEX_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir().join(".codex")),
            artifact_dir: root.join(".epiphany-dogfood").join("coordinator"),
            agent_memory_dir: root.join("state").join("agents.msgpack"),
            mode: "plan".to_string(),
            max_steps: 4,
            poll_seconds: 5.0,
            timeout_seconds: 240,
            max_runtime_seconds: 180,
            ephemeral: true,
            auto_review: false,
            bootstrap_smoke_state: false,
            simulate_high_pressure: false,
            simulate_source_drift: false,
            dry_compact: false,
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--app-server" => parsed.app_server = take_path(&mut args, "--app-server")?,
                "--thread-id" => parsed.thread_id = Some(take_string(&mut args, "--thread-id")?),
                "--cwd" => parsed.cwd = take_path(&mut args, "--cwd")?,
                "--codex-home" => parsed.codex_home = take_path(&mut args, "--codex-home")?,
                "--artifact-dir" => parsed.artifact_dir = take_path(&mut args, "--artifact-dir")?,
                "--agent-memory-dir" => {
                    parsed.agent_memory_dir = take_path(&mut args, "--agent-memory-dir")?;
                }
                "--mode" => parsed.mode = take_string(&mut args, "--mode")?,
                "--max-steps" => {
                    parsed.max_steps = take_string(&mut args, "--max-steps")?.parse()?
                }
                "--poll-seconds" => {
                    parsed.poll_seconds = take_string(&mut args, "--poll-seconds")?.parse()?;
                }
                "--timeout-seconds" => {
                    parsed.timeout_seconds =
                        take_string(&mut args, "--timeout-seconds")?.parse()?;
                }
                "--max-runtime-seconds" => {
                    parsed.max_runtime_seconds =
                        take_string(&mut args, "--max-runtime-seconds")?.parse()?;
                }
                "--ephemeral" => parsed.ephemeral = true,
                "--no-ephemeral" => parsed.ephemeral = false,
                "--auto-review" => parsed.auto_review = true,
                "--test-complete-backend" => {
                    return Err(anyhow!(
                        "--test-complete-backend was removed: native coordinator refuses direct private state-store job mutation; use live workers or a future CultNet job-result API"
                    ));
                }
                "--bootstrap-smoke-state" => parsed.bootstrap_smoke_state = true,
                "--simulate-high-pressure" => parsed.simulate_high_pressure = true,
                "--simulate-source-drift" => parsed.simulate_source_drift = true,
                "--dry-compact" => parsed.dry_compact = true,
                other => return Err(anyhow!("unknown argument: {other}")),
            }
        }
        Ok(parsed)
    }
}

fn run_coordinator(args: &Args) -> Result<Value> {
    let app_server = status_cli::absolute_path(&args.app_server)?;
    let mut cwd = status_cli::absolute_path(&args.cwd)?;
    let codex_home = status_cli::absolute_path(&args.codex_home)?;
    let artifact_dir = status_cli::absolute_path(&args.artifact_dir)?;
    let agent_memory_dir = status_cli::absolute_path(&args.agent_memory_dir)?;
    reset_artifact_dir(&artifact_dir)?;
    fs::create_dir_all(&codex_home)?;
    if args.bootstrap_smoke_state {
        cwd = artifact_dir.join("workspace");
        prepare_workspace(&cwd)?;
    }

    let transcript_path = artifact_dir.join("epiphany-transcript.jsonl");
    let stderr_path = artifact_dir.join("epiphany-server.stderr.log");
    let telemetry_path = artifact_dir.join("agent-function-telemetry.json");
    let steps_path = artifact_dir.join("coordinator-steps.jsonl");
    let mut steps = Vec::new();
    let mut snapshots = Vec::new();
    let mut startup_events = Vec::new();
    let mut final_status = Value::Null;
    let mut final_action = Value::Null;

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
                "name": "epiphany-mvp-coordinator",
                "title": "Epiphany MVP Coordinator",
                "version": "0.1.0",
            },
            "capabilities": {"experimentalApi": true},
        })),
        true,
    )?;
    client.send("initialized", None, false)?;

    let thread_id = if let Some(thread_id) = &args.thread_id {
        let resumed = client.send("thread/resume", Some(json!({"threadId": thread_id})), true)?;
        startup_events.push(thread_lifecycle_event("threadResume", &resumed));
        thread_id.clone()
    } else {
        let started = client.send(
            "thread/start",
            Some(json!({"cwd": cwd, "ephemeral": args.ephemeral})),
            true,
        )?;
        startup_events.push(thread_lifecycle_event("threadStart", &started));
        text_at(&started, &["thread", "id"])?
    };

    if args.bootstrap_smoke_state {
        client.send(
            "thread/epiphany/update",
            Some(json!({"threadId": thread_id, "expectedRevision": 0, "patch": reorient_patch()})),
            true,
        )?;
        if args.simulate_source_drift {
            let _ = client.send(
                "thread/epiphany/freshness",
                Some(json!({"threadId": thread_id})),
                true,
            );
            fs::write(
                cwd.join(WATCHED_RELATIVE_PATH),
                "pub fn reorient_target() -> &'static str {\n    \"after\"\n}\n",
            )?;
            thread::sleep(Duration::from_millis(500));
        }
    }

    for index in 0..args.max_steps {
        let status = collect_coordinator_status(&mut client, &thread_id)?;
        let mut coordinator = status
            .get("coordinator")
            .cloned()
            .unwrap_or_else(|| json!({"action": "regatherManually"}));
        let mut action = coordinator["action"]
            .as_str()
            .unwrap_or("regatherManually")
            .to_string();
        if args.simulate_high_pressure && index == 0 {
            action = "compactRehydrateReorient".to_string();
            coordinator["action"] = json!(action);
            coordinator["canAutoRun"] = json!(true);
            coordinator["requiresReview"] = json!(false);
            coordinator["reason"] = json!("Simulated high pressure requested by smoke test.");
        }

        let snapshot_name = format!("step-{index:02}-{action}.txt");
        fs::write(
            artifact_dir.join(&snapshot_name),
            status_cli::render_status(&status_cli::sanitize_for_operator(status.clone())),
        )?;
        snapshots.push(snapshot_name);
        let mut step = json!({
            "index": index,
            "action": action,
            "coordinator": coordinator,
            "stateRevision": state_revision(&status),
            "events": [],
        });
        final_status = status.clone();
        final_action = coordinator.clone();

        if args.mode == "plan" {
            append_jsonl(&steps_path, &step)?;
            steps.push(step);
            break;
        }
        if is_stop_action(&action) && !args.auto_review {
            append_jsonl(&steps_path, &step)?;
            steps.push(step);
            break;
        }

        let revision = state_revision(&status);
        match action.as_str() {
            "reviewModelingResult" => {
                let result = client.send(
                    "thread/epiphany/roleResult",
                    Some(json!({"threadId": thread_id, "roleId": "modeling"})),
                    true,
                )?;
                push_event(
                    &mut step,
                    json!({"type": "roleResult", "roleId": "modeling", "result": status_cli::sanitize_for_operator(result.clone())}),
                );
                let can_accept = args.auto_review
                    && result.pointer("/finding/statePatch").is_some()
                    && revision.is_some();
                if !can_accept {
                    final_action =
                        json!({"action": "reviewModelingResult", "reason": result["note"]});
                    append_jsonl(&steps_path, &step)?;
                    steps.push(step);
                    break;
                }
                let accepted = client.send(
                    "thread/epiphany/roleAccept",
                    Some(json!({"threadId": thread_id, "roleId": "modeling", "expectedRevision": revision})),
                    true,
                )?;
                if let Some(memory) = maybe_apply_role_self_patch(&accepted, &agent_memory_dir)? {
                    let mut accepted_with_memory = accepted.clone();
                    accepted_with_memory["selfMemoryApply"] = memory;
                    push_event(
                        &mut step,
                        json!({"type": "roleAccept", "roleId": "modeling", "accepted": status_cli::sanitize_for_operator(accepted_with_memory)}),
                    );
                } else {
                    push_event(
                        &mut step,
                        json!({"type": "roleAccept", "roleId": "modeling", "accepted": status_cli::sanitize_for_operator(accepted)}),
                    );
                }
                final_status = collect_coordinator_status(&mut client, &thread_id)?;
            }
            "launchModeling" | "launchVerification" => {
                let role_id = if action == "launchModeling" {
                    "modeling"
                } else {
                    "verification"
                };
                let launch = launch_role(
                    &mut client,
                    &thread_id,
                    role_id,
                    revision,
                    args.max_runtime_seconds,
                )?;
                push_event(
                    &mut step,
                    json!({"type": "roleLaunch", "roleId": role_id, "launch": status_cli::sanitize_for_operator(launch.clone())}),
                );
                let result = wait_for_role_result(&mut client, &thread_id, role_id, args)?;
                push_event(
                    &mut step,
                    json!({"type": "roleResult", "roleId": role_id, "result": status_cli::sanitize_for_operator(result.clone())}),
                );
                final_status = collect_coordinator_status(&mut client, &thread_id)?;
                if !args.auto_review {
                    final_action = json!({
                        "action": if role_id == "modeling" { "reviewModelingResult" } else { "reviewVerificationResult" },
                        "reason": result["note"],
                    });
                    append_jsonl(&steps_path, &step)?;
                    steps.push(step);
                    break;
                }
            }
            "launchReorientWorker" => {
                let launch =
                    launch_reorient(&mut client, &thread_id, revision, args.max_runtime_seconds)?;
                push_event(
                    &mut step,
                    json!({"type": "reorientLaunch", "launch": status_cli::sanitize_for_operator(launch.clone())}),
                );
                let result = wait_for_reorient_result(&mut client, &thread_id, args)?;
                push_event(
                    &mut step,
                    json!({"type": "reorientResult", "result": status_cli::sanitize_for_operator(result.clone())}),
                );
                final_status = collect_coordinator_status(&mut client, &thread_id)?;
                if !args.auto_review {
                    final_action =
                        json!({"action": "reviewReorientResult", "reason": result["note"]});
                    append_jsonl(&steps_path, &step)?;
                    steps.push(step);
                    break;
                }
            }
            "compactRehydrateReorient" => {
                if args.dry_compact {
                    push_event(
                        &mut step,
                        json!({"type": "dryCompact", "threadId": thread_id}),
                    );
                    append_jsonl(&steps_path, &step)?;
                    steps.push(step);
                    continue;
                }
                push_event(
                    &mut step,
                    json!({"type": "compactUnsupportedInNativeSmoke"}),
                );
            }
            _ => {}
        }
        append_jsonl(&steps_path, &step)?;
        steps.push(step);
    }

    let operator_final_status = status_cli::sanitize_for_operator(final_status);
    let final_rendered = status_cli::render_status(&operator_final_status);
    let operator_steps = status_cli::sanitize_for_operator(Value::Array(steps));
    let summary = json!({
        "objective": "Coordinate the Epiphany MVP lanes over existing app-server APIs.",
        "artifactDir": artifact_dir,
        "codexHome": codex_home,
        "workspace": cwd,
        "threadId": operator_final_status["threadId"],
        "mode": args.mode,
        "startupEvents": startup_events,
        "steps": operator_steps,
        "snapshots": snapshots,
        "finalAction": status_cli::sanitize_for_operator(final_action),
        "finalStatus": operator_final_status,
        "artifactManifest": [
            "coordinator-summary.json",
            "coordinator-steps.jsonl",
            "coordinator-final-status.json",
            "coordinator-final-status.txt",
            "coordinator-final-action.txt",
            "agent-function-telemetry.json"
        ],
        "sealedArtifactManifest": [
            {"path": "epiphany-transcript.jsonl", "reason": "sealed JSON-RPC audit trail; do not read during normal supervision"},
            {"path": "epiphany-server.stderr.log", "reason": "sealed app-server diagnostics; inspect only for explicit debugging"}
        ]
    });
    write_json(&artifact_dir.join("coordinator-summary.json"), &summary)?;
    write_json(
        &artifact_dir.join("coordinator-final-status.json"),
        &summary["finalStatus"],
    )?;
    fs::write(
        artifact_dir.join("coordinator-final-status.txt"),
        final_rendered,
    )?;
    fs::write(
        artifact_dir.join("coordinator-final-action.txt"),
        format!(
            "{}\n",
            serde_json::to_string_pretty(&summary["finalAction"])?
        ),
    )?;
    status_cli::write_transcript_telemetry(&transcript_path, &telemetry_path)?;
    Ok(summary)
}

fn collect_coordinator_status(
    client: &mut status_cli::AppServerClient,
    thread_id: &str,
) -> Result<Value> {
    let read = client.send(
        "thread/read",
        Some(json!({"threadId": thread_id, "includeTurns": false})),
        true,
    )?;
    let scene = client.send(
        "thread/epiphany/scene",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let pressure = client.send(
        "thread/epiphany/pressure",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let reorient = client.send(
        "thread/epiphany/reorient",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let jobs = client.send(
        "thread/epiphany/jobs",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let roles = client.send(
        "thread/epiphany/roles",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let planning = client.send(
        "thread/epiphany/planning",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let role_results = json!({
        "imagination": client.send("thread/epiphany/roleResult", Some(json!({"threadId": thread_id, "roleId": "imagination"})), true)?,
        "modeling": client.send("thread/epiphany/roleResult", Some(json!({"threadId": thread_id, "roleId": "modeling"})), true)?,
        "verification": client.send("thread/epiphany/roleResult", Some(json!({"threadId": thread_id, "roleId": "verification"})), true)?,
    });
    let reorient_result = client.send(
        "thread/epiphany/reorientResult",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let crrc = client.send(
        "thread/epiphany/crrc",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let coordinator = client.send(
        "thread/epiphany/coordinator",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let root = env::current_dir()?;
    let heartbeat_dir = root.join(".epiphany-heartbeats");
    let face_dir = root.join(".epiphany-face");
    let heartbeat = status_cli::native_json(
        "epiphany-heartbeat-store",
        &[
            "status",
            "--store",
            "state/agent-heartbeats.msgpack",
            "--artifact-dir",
            &heartbeat_dir.to_string_lossy(),
            "--limit",
            "8",
        ],
    )?;
    let latest_face = status_cli::native_json(
        "epiphany-face-discord",
        &[
            "latest",
            "--artifact-dir",
            &face_dir.to_string_lossy(),
            "--limit",
            "8",
        ],
    )
    .unwrap_or_else(|_| json!({"latestArtifacts": []}));
    Ok(json!({
        "threadId": thread_id,
        "read": read,
        "scene": scene,
        "pressure": pressure,
        "reorient": reorient,
        "jobs": jobs,
        "roles": roles,
        "planning": planning,
        "roleResults": role_results,
        "reorientResult": reorient_result,
        "crrc": crrc,
        "coordinator": coordinator,
        "heartbeat": heartbeat,
        "face": {
            "status": "ready",
            "artifactDir": face_dir,
            "latestArtifacts": latest_face.get("latestArtifacts").cloned().unwrap_or_else(|| json!([])),
            "availableActions": ["faceBubble"],
        },
    }))
}

fn launch_role(
    client: &mut status_cli::AppServerClient,
    thread_id: &str,
    role_id: &str,
    expected_revision: Option<i64>,
    max_runtime_seconds: u64,
) -> Result<Value> {
    let mut payload =
        json!({"threadId": thread_id, "roleId": role_id, "maxRuntimeSeconds": max_runtime_seconds});
    if let Some(revision) = expected_revision {
        payload["expectedRevision"] = json!(revision);
    }
    client.send("thread/epiphany/roleLaunch", Some(payload), true)
}

fn launch_reorient(
    client: &mut status_cli::AppServerClient,
    thread_id: &str,
    expected_revision: Option<i64>,
    max_runtime_seconds: u64,
) -> Result<Value> {
    let mut payload = json!({"threadId": thread_id, "maxRuntimeSeconds": max_runtime_seconds});
    if let Some(revision) = expected_revision {
        payload["expectedRevision"] = json!(revision);
    }
    client.send("thread/epiphany/reorientLaunch", Some(payload), true)
}

fn wait_for_role_result(
    client: &mut status_cli::AppServerClient,
    thread_id: &str,
    role_id: &str,
    args: &Args,
) -> Result<Value> {
    let deadline = Instant::now() + Duration::from_secs(args.timeout_seconds);
    let mut latest = Value::Null;
    while Instant::now() < deadline {
        latest = client.send(
            "thread/epiphany/roleResult",
            Some(json!({"threadId": thread_id, "roleId": role_id})),
            true,
        )?;
        if matches!(
            latest["status"].as_str(),
            Some("completed" | "failed" | "cancelled")
        ) {
            return Ok(latest);
        }
        thread::sleep(Duration::from_secs_f64(args.poll_seconds));
    }
    Ok(latest)
}

fn wait_for_reorient_result(
    client: &mut status_cli::AppServerClient,
    thread_id: &str,
    args: &Args,
) -> Result<Value> {
    let deadline = Instant::now() + Duration::from_secs(args.timeout_seconds);
    let mut latest = Value::Null;
    while Instant::now() < deadline {
        latest = client.send(
            "thread/epiphany/reorientResult",
            Some(json!({"threadId": thread_id, "bindingId": REORIENT_BINDING_ID})),
            true,
        )?;
        if matches!(
            latest["status"].as_str(),
            Some("completed" | "failed" | "cancelled")
        ) {
            return Ok(latest);
        }
        thread::sleep(Duration::from_secs_f64(args.poll_seconds));
    }
    Ok(latest)
}

fn maybe_apply_role_self_patch(accepted: &Value, agent_memory_dir: &Path) -> Result<Option<Value>> {
    let finding = &accepted["finding"];
    let self_patch = &finding["selfPatch"];
    let review = &finding["selfPersistence"];
    if !self_patch.is_object() || !review.is_object() {
        return Ok(None);
    }
    if review["status"].as_str() != Some("accepted") {
        return Ok(Some(json!({
            "status": "rejected",
            "targetAgentId": review["targetAgentId"],
            "targetPath": review["targetPath"],
            "reasons": review["reasons"],
            "applied": false,
        })));
    }
    let role_id = accepted["roleId"]
        .as_str()
        .ok_or_else(|| anyhow!("roleAccept response did not include roleId"))?;
    let patch = serde_json::to_string(self_patch)?;
    let output = status_cli::native_json(
        "epiphany-agent-memory-store",
        &[
            "apply-patch",
            "--store",
            &agent_memory_dir.to_string_lossy(),
            "--role-id",
            role_id,
            "--patch",
            &patch,
        ],
    )?;
    let mut output = output;
    output["appliedFromRoleAccept"] = json!(true);
    Ok(Some(output))
}

fn reorient_patch() -> Value {
    json!({
        "objective": "Decide whether a durable checkpoint still deserves to be resumed after rehydrate.",
        "activeSubgoalId": "phase6-reorient-smoke",
        "subgoals": [{
            "id": "phase6-reorient-smoke",
            "title": "Live-smoke CRRC reorientation policy",
            "status": "active",
            "summary": "Resume when the checkpoint is still aligned; regather when the touched file proves it isn't.",
        }],
        "graphs": {
            "architecture": {"nodes": [{
                "id": GRAPH_NODE_ID,
                "title": "Reorient target",
                "purpose": "Map the file the watcher will touch so reorientation can notice drift.",
                "code_refs": [{"path": WATCHED_RELATIVE_PATH, "start_line": 1, "end_line": 3, "symbol": "reorient_target"}],
            }]},
            "dataflow": {"nodes": []},
            "links": [],
        },
        "graphFrontier": {"active_node_ids": [GRAPH_NODE_ID], "dirty_paths": []},
        "graphCheckpoint": {
            "checkpoint_id": "ck-reorient-1",
            "graph_revision": 1,
            "summary": "Reorientation smoke graph checkpoint",
            "frontier_node_ids": [GRAPH_NODE_ID],
        },
        "investigationCheckpoint": {
            "checkpoint_id": "ix-reorient-1",
            "kind": "source_gathering",
            "disposition": "resume_ready",
            "focus": "Verify the touched file before broad edits.",
            "summary": "This checkpoint should remain resumable until the watched source moves.",
            "next_action": "Resume the bounded slice if the watched source still matches the checkpoint.",
            "captured_at_turn_id": "turn-phase6-reorient",
            "code_refs": [{"path": WATCHED_RELATIVE_PATH, "start_line": 1, "end_line": 3, "symbol": "reorient_target"}],
        },
        "churn": {
            "understanding_status": "ready",
            "diff_pressure": "low",
            "graph_freshness": "fresh",
            "unexplained_writes": 0,
        },
    })
}

fn prepare_workspace(workspace: &Path) -> Result<()> {
    if workspace.exists() {
        fs::remove_dir_all(workspace)?;
    }
    let watched = workspace.join(WATCHED_RELATIVE_PATH);
    fs::create_dir_all(watched.parent().unwrap())?;
    fs::write(
        watched,
        "pub fn reorient_target() -> &'static str {\n    \"before\"\n}\n",
    )?;
    Ok(())
}

fn reset_artifact_dir(path: &Path) -> Result<()> {
    let root = env::current_dir()?
        .join(".epiphany-dogfood")
        .canonicalize()
        .or_else(|_| {
            let root = env::current_dir()?.join(".epiphany-dogfood");
            fs::create_dir_all(&root)?;
            root.canonicalize()
        })?;
    let resolved_parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(resolved_parent)?;
    let resolved = if path.exists() {
        path.canonicalize()?
    } else {
        resolved_parent
            .canonicalize()?
            .join(path.file_name().unwrap())
    };
    if resolved == root || !resolved.starts_with(&root) {
        return Err(anyhow!(
            "refusing to delete non-dogfood artifact dir: {}",
            path.display()
        ));
    }
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    fs::create_dir_all(path)?;
    Ok(())
}

fn state_revision(status: &Value) -> Option<i64> {
    status
        .pointer("/read/thread/epiphanyState/revision")
        .and_then(Value::as_i64)
        .or_else(|| {
            status
                .pointer("/scene/scene/revision")
                .and_then(Value::as_i64)
        })
}

fn thread_lifecycle_event(kind: &str, response: &Value) -> Value {
    json!({
        "type": kind,
        "threadId": response.pointer("/thread/id"),
        "status": response.pointer("/thread/status"),
        "cwd": response.pointer("/thread/cwd"),
        "ephemeral": response.pointer("/thread/ephemeral"),
    })
}

fn push_event(step: &mut Value, event: Value) {
    step["events"].as_array_mut().unwrap().push(event);
}

fn is_stop_action(action: &str) -> bool {
    matches!(
        action,
        "prepareCheckpoint"
            | "reviewReorientResult"
            | "regatherManually"
            | "reviewModelingResult"
            | "reviewVerificationResult"
            | "continueImplementation"
    )
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))?;
    Ok(())
}

fn append_jsonl(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    use std::io::Write;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", serde_json::to_string(value)?)?;
    Ok(())
}

fn text_at(value: &Value, path: &[&str]) -> Result<String> {
    let mut cursor = value;
    for key in path {
        cursor = &cursor[*key];
    }
    cursor
        .as_str()
        .map(ToString::to_string)
        .ok_or_else(|| anyhow!("missing string at {}", path.join(".")))
}

fn take_string(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("{name} requires a value"))
}

fn take_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(take_string(args, name)?))
}

fn home_dir() -> PathBuf {
    env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}
