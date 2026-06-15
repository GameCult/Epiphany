use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::SecondsFormat;
use epiphany_core::COORDINATOR_RUN_RECEIPT_SCHEMA_VERSION;
use epiphany_core::COORDINATOR_RUN_RECEIPT_TYPE;
use epiphany_core::EpiphanyCoordinatorRunReceipt;
use epiphany_core::HANDS_ACTION_INTENT_SCHEMA_VERSION;
use epiphany_core::HANDS_COMMAND_RECEIPT_TYPE;
use epiphany_core::HANDS_COMMIT_RECEIPT_TYPE;
use epiphany_core::HANDS_PATCH_RECEIPT_TYPE;
use epiphany_core::HandsActionIntent;
use epiphany_core::RuntimeSpineEventOptions;
use epiphany_core::RuntimeSpineInitOptions;
use epiphany_core::RuntimeSpineSessionOptions;
use epiphany_core::SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION;
use epiphany_core::SubstrateGateRepoAccessGrantReceipt;
use epiphany_core::append_runtime_event;
use epiphany_core::create_runtime_session;
use epiphany_core::hands_action_review_for_intent;
use epiphany_core::initialize_runtime_spine;
use epiphany_core::put_coordinator_run_receipt;
use epiphany_core::put_hands_action_intent;
use epiphany_core::put_hands_action_review;
use epiphany_core::put_substrate_gate_repo_access_grant_receipt;
use epiphany_core::runtime_spine_status;
use serde_json::Value;
use serde_json::json;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;
#[cfg(windows)]
const DETACHED_PROCESS: u32 = 0x0000_0008;

#[allow(dead_code)]
#[path = "epiphany-mvp-status.rs"]
mod status_cli;

const DEFAULT_APP_SERVER: &str = r"C:\Users\Meta\.cargo-target-codex\debug\codex-app-server.exe";
const DEFAULT_MODEL_RUNTIME_BIN: &str = "epiphany-model-runtime";
const DEFAULT_TOOL_ADAPTER_BIN: &str = "epiphany-tool-codex-mcp-spine";
const WORKER_AUTO_TOOL_MAX_ROUNDS: usize = 24;
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
    model_runtime_bin: PathBuf,
    tool_adapter_bin: PathBuf,
    model_provider: String,
    thread_id: Option<String>,
    cwd: PathBuf,
    codex_home: PathBuf,
    artifact_dir: PathBuf,
    agent_memory_dir: PathBuf,
    runtime_store: PathBuf,
    mode: String,
    max_steps: usize,
    poll_seconds: f64,
    timeout_seconds: u64,
    max_runtime_seconds: u64,
    ephemeral: bool,
    auto_review: bool,
    supersede_failed_results: bool,
    auto_tools: bool,
    bootstrap_smoke_state: bool,
    bootstrap_local_state: bool,
    bootstrap_objective: Option<String>,
    simulate_high_pressure: bool,
    simulate_continue_implementation: bool,
    simulate_source_drift: bool,
    dry_compact: bool,
}

impl Args {
    fn parse() -> Result<Self> {
        let root = env::current_dir().context("failed to resolve current dir")?;
        let mut args = env::args().skip(1);
        let mut parsed = Args {
            app_server: PathBuf::from(DEFAULT_APP_SERVER),
            model_runtime_bin: PathBuf::from(DEFAULT_MODEL_RUNTIME_BIN),
            tool_adapter_bin: PathBuf::from(DEFAULT_TOOL_ADAPTER_BIN),
            model_provider: "openai-codex".to_string(),
            thread_id: None,
            cwd: root.clone(),
            codex_home: env::var_os("CODEX_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir().join(".codex")),
            artifact_dir: root.join(".epiphany-dogfood").join("coordinator"),
            agent_memory_dir: root.join("state").join("agents.msgpack"),
            runtime_store: root.join("state").join("runtime-spine.msgpack"),
            mode: "plan".to_string(),
            max_steps: 4,
            poll_seconds: 5.0,
            timeout_seconds: 600,
            max_runtime_seconds: 180,
            ephemeral: true,
            auto_review: false,
            supersede_failed_results: false,
            auto_tools: true,
            bootstrap_smoke_state: false,
            bootstrap_local_state: false,
            bootstrap_objective: None,
            simulate_high_pressure: false,
            simulate_continue_implementation: false,
            simulate_source_drift: false,
            dry_compact: false,
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--app-server" => parsed.app_server = take_path(&mut args, "--app-server")?,
                "--model-runtime-bin" => {
                    parsed.model_runtime_bin = take_path(&mut args, "--model-runtime-bin")?;
                }
                "--tool-adapter-bin" => {
                    parsed.tool_adapter_bin = take_path(&mut args, "--tool-adapter-bin")?;
                }
                "--openai-runtime-bin" => {
                    parsed.model_runtime_bin = take_path(&mut args, "--openai-runtime-bin")?;
                }
                "--model-provider" => {
                    parsed.model_provider = take_string(&mut args, "--model-provider")?;
                }
                "--thread-id" => parsed.thread_id = Some(take_string(&mut args, "--thread-id")?),
                "--cwd" => parsed.cwd = take_path(&mut args, "--cwd")?,
                "--codex-home" => parsed.codex_home = take_path(&mut args, "--codex-home")?,
                "--artifact-dir" => parsed.artifact_dir = take_path(&mut args, "--artifact-dir")?,
                "--agent-memory-dir" => {
                    parsed.agent_memory_dir = take_path(&mut args, "--agent-memory-dir")?;
                }
                "--runtime-store" => {
                    parsed.runtime_store = take_path(&mut args, "--runtime-store")?
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
                "--supersede-failed-results" => parsed.supersede_failed_results = true,
                "--auto-tools" => parsed.auto_tools = true,
                "--no-auto-tools" => parsed.auto_tools = false,
                "--test-complete-backend" => {
                    return Err(anyhow!(
                        "--test-complete-backend was removed: native coordinator refuses direct private state-store job mutation; use live workers or a future CultNet job-result API"
                    ));
                }
                "--bootstrap-smoke-state" => parsed.bootstrap_smoke_state = true,
                "--bootstrap-local-state" => parsed.bootstrap_local_state = true,
                "--bootstrap-objective" => {
                    parsed.bootstrap_objective =
                        Some(take_string(&mut args, "--bootstrap-objective")?);
                }
                "--simulate-high-pressure" => parsed.simulate_high_pressure = true,
                "--simulate-continue-implementation" => {
                    parsed.simulate_continue_implementation = true
                }
                "--simulate-source-drift" => parsed.simulate_source_drift = true,
                "--dry-compact" => parsed.dry_compact = true,
                other => return Err(anyhow!("unknown argument: {other}")),
            }
        }
        Ok(parsed)
    }
}

fn run_coordinator(args: &Args) -> Result<Value> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let app_server = status_cli::absolute_path(&args.app_server)?;
    let mut cwd = status_cli::absolute_path(&args.cwd)?;
    let model_runtime_bin = resolve_model_runtime_bin(&root, &args.model_runtime_bin)?;
    let tool_adapter_bin = resolve_model_runtime_bin(&root, &args.tool_adapter_bin)?;
    let codex_home = status_cli::absolute_path(&args.codex_home)?;
    let artifact_dir = status_cli::absolute_path(&args.artifact_dir)?;
    let agent_memory_dir = status_cli::absolute_path(&args.agent_memory_dir)?;
    let runtime_store = status_cli::absolute_path(&args.runtime_store)?;
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
    let runtime_session_id = format!("coordinator-{thread_id}");
    let runtime_identity = initialize_runtime_spine(
        &runtime_store,
        RuntimeSpineInitOptions {
            runtime_id: "epiphany-local".to_string(),
            display_name: "Epiphany Local".to_string(),
            created_at: now(),
        },
    )?;
    let runtime_session = ensure_runtime_session(
        &runtime_store,
        &runtime_session_id,
        "Coordinate the Epiphany MVP lanes with native runtime job receipts.",
    )?;
    let runtime_event = append_runtime_event(
        &runtime_store,
        RuntimeSpineEventOptions {
            event_id: format!("event-coordinator-started-{}", sanitize_id(&thread_id)),
            occurred_at: now(),
            event_type: "coordinator.started".to_string(),
            source: "epiphany-mvp-coordinator".to_string(),
            session_id: Some(runtime_session_id.clone()),
            job_id: None,
            summary: "Native coordinator session opened.".to_string(),
        },
    )
    .or_else(|_| {
        append_runtime_event(
            &runtime_store,
            RuntimeSpineEventOptions {
                event_id: format!(
                    "event-coordinator-started-{}-{}",
                    sanitize_id(&thread_id),
                    uuid::Uuid::new_v4()
                ),
                occurred_at: now(),
                event_type: "coordinator.started".to_string(),
                source: "epiphany-mvp-coordinator".to_string(),
                session_id: Some(runtime_session_id.clone()),
                job_id: None,
                summary: "Native coordinator session opened.".to_string(),
            },
        )
    })?;
    startup_events.push(json!({
        "type": "runtimeSpineSession",
        "store": runtime_store,
        "runtimeId": runtime_identity.runtime_id,
        "sessionId": runtime_session.session_id,
        "eventId": runtime_event.event_id,
    }));

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
    } else if args.bootstrap_local_state {
        client.send(
            "thread/epiphany/update",
            Some(json!({
                "threadId": thread_id,
                "expectedRevision": 0,
                "patch": local_mvp_checkpoint_patch(&cwd, args.bootstrap_objective.as_deref()),
            })),
            true,
        )?;
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
        } else if args.simulate_continue_implementation && index == 0 {
            action = "continueImplementation".to_string();
            coordinator["action"] = json!(action);
            coordinator["canAutoRun"] = json!(true);
            coordinator["requiresReview"] = json!(false);
            coordinator["targetRole"] = json!("implementation");
            coordinator["reason"] =
                json!("Simulated implementation continuation requested by smoke test.");
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
            append_operator_step_jsonl(&steps_path, &step)?;
            steps.push(step);
            break;
        }
        if is_stop_action(&action) && !args.auto_review && !is_result_review_action(&action) {
            append_operator_step_jsonl(&steps_path, &step)?;
            steps.push(step);
            break;
        }

        let revision = state_revision(&status);
        match action.as_str() {
            "reviewResearchResult" | "reviewModelingResult" | "reviewVerificationResult" => {
                let role_id = role_id_for_coordinator_action(&action)
                    .ok_or_else(|| anyhow!("unsupported review action {action}"))?;
                let result = client.send(
                    "thread/epiphany/roleResult",
                    Some(json!({"threadId": thread_id, "roleId": role_id})),
                    true,
                )?;
                push_event(
                    &mut step,
                    json!({"type": "roleResult", "roleId": role_id, "result": status_cli::sanitize_for_operator(result.clone())}),
                );
                if !matches!(
                    result["status"].as_str(),
                    Some("completed" | "failed" | "cancelled")
                ) {
                    final_action = json!({
                        "action": wait_action_for_role(role_id),
                        "reason": result["note"],
                    });
                    append_operator_step_jsonl(&steps_path, &step)?;
                    steps.push(step);
                    break;
                }
                if args.supersede_failed_results && role_result_needs_supersession(role_id, &result)
                {
                    let superseded = supersede_role_result(
                        &mut client,
                        &thread_id,
                        role_id,
                        revision,
                        &result,
                    )?;
                    push_event(
                        &mut step,
                        json!({"type": "roleFailureReview", "roleId": role_id, "superseded": status_cli::sanitize_for_operator(superseded)}),
                    );
                    final_status = collect_coordinator_status(&mut client, &thread_id)?;
                    append_operator_step_jsonl(&steps_path, &step)?;
                    steps.push(step);
                    continue;
                }
                let can_accept = args.auto_review
                    && role_result_auto_acceptable(role_id, &result)
                    && revision.is_some();
                if !can_accept {
                    final_action = json!({
                        "action": review_action_for_role(role_id),
                        "reason": result["note"]
                    });
                    append_operator_step_jsonl(&steps_path, &step)?;
                    steps.push(step);
                    break;
                }
                let accepted = client.send(
                    "thread/epiphany/roleAccept",
                    Some(json!({"threadId": thread_id, "roleId": role_id, "expectedRevision": revision})),
                    true,
                )?;
                if let Some(memory) = maybe_apply_role_self_patch(&accepted, &agent_memory_dir)? {
                    let mut accepted_with_memory = accepted.clone();
                    accepted_with_memory["selfMemoryApply"] = memory;
                    push_event(
                        &mut step,
                        json!({"type": "roleAccept", "roleId": role_id, "accepted": status_cli::sanitize_for_operator(accepted_with_memory)}),
                    );
                } else {
                    push_event(
                        &mut step,
                        json!({"type": "roleAccept", "roleId": role_id, "accepted": status_cli::sanitize_for_operator(accepted)}),
                    );
                }
                final_status = collect_coordinator_status(&mut client, &thread_id)?;
            }
            "reviewReorientResult" => {
                let result = read_reorient_result(&mut client, &thread_id)?;
                push_event(
                    &mut step,
                    json!({"type": "reorientResult", "result": status_cli::sanitize_for_operator(result.clone())}),
                );
                if !matches!(
                    result["status"].as_str(),
                    Some("completed" | "failed" | "cancelled")
                ) {
                    final_action =
                        json!({"action": "waitForReorientResult", "reason": result["note"]});
                    append_operator_step_jsonl(&steps_path, &step)?;
                    steps.push(step);
                    break;
                }
                final_action = json!({"action": "reviewReorientResult", "reason": result["note"]});
                append_operator_step_jsonl(&steps_path, &step)?;
                steps.push(step);
                break;
            }
            "continueImplementation" => {
                let gate = record_hands_implementation_gate(
                    &runtime_store,
                    &artifact_dir,
                    &thread_id,
                    index,
                    &status,
                )?;
                push_event(
                    &mut step,
                    json!({"type": "handsActionGate", "gate": gate.clone()}),
                );
                coordinator["handsActionGate"] = gate;
                final_action = coordinator;
                append_operator_step_jsonl(&steps_path, &step)?;
                steps.push(step);
                break;
            }
            "launchResearch" | "launchModeling" | "launchVerification" => {
                let role_id = role_id_for_coordinator_action(&action)
                    .ok_or_else(|| anyhow!("unsupported launch action {action}"))?;
                let launch = launch_role(
                    &mut client,
                    &thread_id,
                    role_id,
                    revision,
                    args.max_runtime_seconds,
                )?;
                let worker_job_id = worker_job_id_from_launch(&launch)?;
                push_event(
                    &mut step,
                    json!({"type": "roleLaunch", "roleId": role_id, "launch": status_cli::sanitize_for_operator(launch.clone()), "runtimeJobId": worker_job_id}),
                );
                let worker_run = launch_worker_runtime_detached(
                    &model_runtime_bin,
                    &tool_adapter_bin,
                    &args.model_provider,
                    &runtime_store,
                    &codex_home,
                    &cwd,
                    &worker_job_id,
                    role_id,
                    index,
                    &artifact_dir,
                    args.max_runtime_seconds,
                    args.auto_tools,
                )?;
                push_event(
                    &mut step,
                    json!({"type": "workerRuntime", "roleId": role_id, "run": worker_run}),
                );
                let result = read_role_result(&mut client, &thread_id, role_id)?;
                push_event(
                    &mut step,
                    json!({"type": "roleResult", "roleId": role_id, "result": status_cli::sanitize_for_operator(result.clone())}),
                );
                final_status = collect_coordinator_status(&mut client, &thread_id)?;
                if !matches!(
                    result["status"].as_str(),
                    Some("completed" | "failed" | "cancelled")
                ) {
                    final_action = json!({
                        "action": wait_action_for_role(role_id),
                        "reason": result["note"],
                    });
                    append_operator_step_jsonl(&steps_path, &step)?;
                    steps.push(step);
                    break;
                }
                if !args.auto_review {
                    final_action = json!({
                        "action": review_action_for_role(role_id),
                        "reason": result["note"],
                    });
                    append_operator_step_jsonl(&steps_path, &step)?;
                    steps.push(step);
                    break;
                }
            }
            "launchReorientWorker" => {
                let launch =
                    launch_reorient(&mut client, &thread_id, revision, args.max_runtime_seconds)?;
                let worker_job_id = worker_job_id_from_launch(&launch)?;
                push_event(
                    &mut step,
                    json!({"type": "reorientLaunch", "launch": status_cli::sanitize_for_operator(launch.clone()), "runtimeJobId": worker_job_id}),
                );
                let worker_run = launch_worker_runtime_detached(
                    &model_runtime_bin,
                    &tool_adapter_bin,
                    &args.model_provider,
                    &runtime_store,
                    &codex_home,
                    &cwd,
                    &worker_job_id,
                    "reorient-worker",
                    index,
                    &artifact_dir,
                    args.max_runtime_seconds,
                    args.auto_tools,
                )?;
                push_event(
                    &mut step,
                    json!({"type": "workerRuntime", "roleId": "reorient-worker", "run": worker_run}),
                );
                let result = read_reorient_result(&mut client, &thread_id)?;
                push_event(
                    &mut step,
                    json!({"type": "reorientResult", "result": status_cli::sanitize_for_operator(result.clone())}),
                );
                final_status = collect_coordinator_status(&mut client, &thread_id)?;
                if !matches!(
                    result["status"].as_str(),
                    Some("completed" | "failed" | "cancelled")
                ) {
                    final_action =
                        json!({"action": "waitForReorientResult", "reason": result["note"]});
                    append_operator_step_jsonl(&steps_path, &step)?;
                    steps.push(step);
                    break;
                }
                if !args.auto_review {
                    final_action =
                        json!({"action": "reviewReorientResult", "reason": result["note"]});
                    append_operator_step_jsonl(&steps_path, &step)?;
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
                    append_operator_step_jsonl(&steps_path, &step)?;
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
        append_operator_step_jsonl(&steps_path, &step)?;
        steps.push(step);
    }

    let operator_final_status = status_cli::sanitize_for_operator(final_status);
    let runtime_status = runtime_spine_status(&runtime_store)?;
    let final_rendered = status_cli::render_status(&operator_final_status);
    let operator_steps = status_cli::sanitize_for_operator(Value::Array(steps));
    let artifact_manifest = vec![
        "coordinator-summary.json".to_string(),
        "coordinator-steps.jsonl".to_string(),
        "coordinator-final-status.json".to_string(),
        "coordinator-final-status.txt".to_string(),
        "coordinator-final-action.txt".to_string(),
        "agent-function-telemetry.json".to_string(),
        "runtime-spine-status.json".to_string(),
    ];
    let sealed_artifact_manifest = vec![
        "epiphany-transcript.jsonl".to_string(),
        "epiphany-server.stderr.log".to_string(),
    ];
    let receipt_created_at = now();
    let coordinator_run_receipt = EpiphanyCoordinatorRunReceipt {
        schema_version: COORDINATOR_RUN_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: format!(
            "coordinator-run-{}-{}",
            sanitize_id(&thread_id),
            chrono::Utc::now().timestamp_millis()
        ),
        session_id: runtime_session_id.clone(),
        thread_id: thread_id.clone(),
        mode: args.mode.clone(),
        status: coordinator_receipt_status(&args.mode, &final_action),
        final_action: final_action_name(&final_action),
        final_reason: final_action_reason(&final_action),
        step_count: operator_steps
            .as_array()
            .map_or(0, |items| items.len() as u64),
        created_at: receipt_created_at,
        model_provider: Some(args.model_provider.clone()),
        runtime_store: runtime_store.display().to_string(),
        artifact_refs: artifact_manifest.clone(),
        sealed_artifact_refs: sealed_artifact_manifest.clone(),
        metadata: BTreeMap::from([
            (
                "artifactDir".to_string(),
                artifact_dir.display().to_string(),
            ),
            (
                "modelRuntimeBin".to_string(),
                model_runtime_bin.display().to_string(),
            ),
            (
                "toolAdapterBin".to_string(),
                tool_adapter_bin.display().to_string(),
            ),
            ("autoTools".to_string(), args.auto_tools.to_string()),
        ]),
    };
    put_coordinator_run_receipt(&runtime_store, &coordinator_run_receipt)?;
    let summary = json!({
        "objective": "Coordinate the Epiphany MVP lanes over existing app-server APIs.",
        "artifactDir": artifact_dir,
        "modelRuntimeBin": model_runtime_bin,
        "toolAdapterBin": tool_adapter_bin,
        "autoTools": args.auto_tools,
        "modelProvider": args.model_provider,
        "codexHome": codex_home,
        "runtimeStore": runtime_store,
        "runtimeSpine": runtime_status,
        "workspace": cwd,
        "threadId": operator_final_status["threadId"],
        "mode": args.mode,
        "startupEvents": startup_events,
        "steps": operator_steps,
        "snapshots": snapshots,
        "finalAction": status_cli::sanitize_for_operator(final_action),
        "finalStatus": operator_final_status,
        "coordinatorRunReceipt": {
            "documentType": COORDINATOR_RUN_RECEIPT_TYPE,
            "receiptId": coordinator_run_receipt.receipt_id,
            "store": runtime_store,
        },
        "artifactManifest": artifact_manifest,
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
    write_json(
        &artifact_dir.join("runtime-spine-status.json"),
        &summary["runtimeSpine"],
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
    let view = client.send(
        "thread/epiphany/view",
        Some(json!({"threadId": thread_id, "lenses": ["scene", "pressure", "jobs", "roles", "planning", "reorient", "crrc", "coordinator"]})),
        true,
    )?;
    let scene = json!({
        "threadId": thread_id,
        "scene": view.get("scene").cloned().unwrap_or_else(|| json!(null)),
    });
    let pressure = json!({
        "threadId": thread_id,
        "source": "live",
        "pressure": view.get("pressure").cloned().unwrap_or_else(|| json!(null)),
    });
    let reorient = view.get("reorient").cloned().unwrap_or_else(|| json!(null));
    let jobs = json!({
        "threadId": thread_id,
        "source": "live",
        "jobs": view.get("jobs").cloned().unwrap_or_else(|| json!([])),
    });
    let roles = view.get("roles").cloned().unwrap_or_else(|| json!(null));
    let planning = view.get("planning").cloned().unwrap_or_else(|| json!(null));
    let role_results = json!({
        "imagination": client.send("thread/epiphany/roleResult", Some(json!({"threadId": thread_id, "roleId": "imagination"})), true)?,
        "research": client.send("thread/epiphany/roleResult", Some(json!({"threadId": thread_id, "roleId": "research"})), true)?,
        "modeling": client.send("thread/epiphany/roleResult", Some(json!({"threadId": thread_id, "roleId": "modeling"})), true)?,
        "verification": client.send("thread/epiphany/roleResult", Some(json!({"threadId": thread_id, "roleId": "verification"})), true)?,
    });
    let reorient_result = client.send(
        "thread/epiphany/reorientResult",
        Some(json!({"threadId": thread_id})),
        true,
    )?;
    let crrc = view.get("crrc").cloned().unwrap_or_else(|| json!(null));
    let coordinator = view
        .get("coordinator")
        .cloned()
        .unwrap_or_else(|| json!(null));
    let root = env::current_dir()?;
    let heartbeat_dir = root.join(".epiphany-heartbeats");
    let persona_dir = root.join(".epiphany-persona");
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
    let latest_persona = status_cli::native_json(
        "epiphany-persona-discord",
        &[
            "latest",
            "--artifact-dir",
            &persona_dir.to_string_lossy(),
            "--limit",
            "8",
        ],
    )
    .unwrap_or_else(|_| json!({"latestArtifacts": []}));
    Ok(json!({
        "threadId": thread_id,
        "read": read,
        "view": view,
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
        "persona": {
            "status": "ready",
            "artifactDir": persona_dir,
            "latestArtifacts": latest_persona.get("latestArtifacts").cloned().unwrap_or_else(|| json!([])),
            "availableActions": ["PersonaBubble", "characterTurn"],
        },
    }))
}

fn resolve_model_runtime_bin(root: &Path, configured: &Path) -> Result<PathBuf> {
    if configured.components().count() != 1 {
        return status_cli::absolute_path(configured);
    }
    let exe_name = format!("{}.exe", configured.display());
    let cargo_target_candidate = env::var_os("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .map(|path| path.join("debug").join(&exe_name));
    let candidates = [
        cargo_target_candidate,
        Some(
            root.join("epiphany-openai-runtime")
                .join("target")
                .join("debug")
                .join(&exe_name),
        ),
        Some(root.join("target").join("debug").join(&exe_name)),
    ];
    for candidate in candidates.into_iter().flatten() {
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    Ok(configured.to_path_buf())
}

fn record_hands_implementation_gate(
    runtime_store: &Path,
    artifact_dir: &Path,
    thread_id: &str,
    step_index: usize,
    status: &Value,
) -> Result<Value> {
    let requested_at = now();
    let suffix = format!(
        "{}-{}-{}",
        sanitize_id(thread_id),
        step_index,
        uuid::Uuid::new_v4()
    );
    let runtime_job_id = format!("hands-implementation-{suffix}");
    let grant_id = format!("substrate-grant-{runtime_job_id}");
    let requested_paths = implementation_requested_paths(status);

    let substrate_grant = SubstrateGateRepoAccessGrantReceipt {
        schema_version: SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: grant_id.clone(),
        runtime_job_id: runtime_job_id.clone(),
        binding_id: "implementation-worker".to_string(),
        role: "epiphany-hands".to_string(),
        authority_scope: "epiphany.role.implementation".to_string(),
        granted_operations: vec![
            "read".to_string(),
            "snapshot".to_string(),
            "patch".to_string(),
            "command".to_string(),
            "commit".to_string(),
        ],
        granted_paths: requested_paths.clone(),
        granted_at: requested_at.clone(),
        contract: "Substrate Gate grants the main Hands agent scoped repository access for the coordinator-approved implementation continuation; every mutation still needs Hands receipts."
            .to_string(),
    };
    put_substrate_gate_repo_access_grant_receipt(runtime_store, &substrate_grant)?;

    let intent = HandsActionIntent {
        schema_version: HANDS_ACTION_INTENT_SCHEMA_VERSION.to_string(),
        intent_id: format!("hands-intent-{suffix}"),
        runtime_job_id: runtime_job_id.clone(),
        binding_id: "implementation-worker".to_string(),
        role: "epiphany-hands".to_string(),
        authority_scope: "epiphany.role.implementation".to_string(),
        requested_action: "continueImplementation".to_string(),
        requested_paths: requested_paths.clone(),
        substrate_gate_grant_receipt_id: grant_id.clone(),
        requested_at: requested_at.clone(),
        contract: "Coordinator continuation becomes a typed Hands action intent before any file edit, command, or commit may count as implementation evidence."
            .to_string(),
    };
    put_hands_action_intent(runtime_store, &intent)?;

    let mut review = hands_action_review_for_intent(
        format!("hands-review-{suffix}"),
        &intent,
        "approved".to_string(),
        vec![
            "patch".to_string(),
            "command".to_string(),
            "commit".to_string(),
        ],
        vec![
            "Coordinator selected continueImplementation for the current thread state."
                .to_string(),
            "Hands remains the action owner; Mind and Soul still own state admission and verification."
                .to_string(),
        ],
        requested_at,
    );
    review.required_receipts = vec![
        HANDS_PATCH_RECEIPT_TYPE.to_string(),
        HANDS_COMMAND_RECEIPT_TYPE.to_string(),
        HANDS_COMMIT_RECEIPT_TYPE.to_string(),
    ];
    put_hands_action_review(runtime_store, &review)?;

    Ok(json!({
        "status": "ready",
        "runtimeJobId": runtime_job_id,
        "substrateGateGrantReceiptId": grant_id,
        "intentId": intent.intent_id,
        "reviewId": review.review_id,
        "requestedPaths": requested_paths,
        "requiredReceipts": review.required_receipts,
        "recordPassCommand": hands_record_pass_command(runtime_store, artifact_dir),
        "store": runtime_store,
    }))
}

fn hands_record_pass_command(runtime_store: &Path, artifact_dir: &Path) -> Value {
    json!({
        "executable": "epiphany-hands-action",
        "args": [
            "--store",
            runtime_store,
            "record-pass",
            "--gate-summary",
            artifact_dir.join("coordinator-summary.json"),
            "--summary",
            "<implementation pass summary>",
            "--changed-path",
            "<changed path>",
            "--command",
            "<verification command>",
            "--exit-code",
            "<exit code>",
            "--stdout-artifact",
            "<stdout artifact path>",
            "--stderr-artifact",
            "<stderr artifact path>",
            "--commit-sha",
            "<commit sha>",
            "--branch",
            "<branch>"
        ],
    })
}

fn implementation_requested_paths(status: &Value) -> Vec<String> {
    let mut paths = Vec::new();
    for pointer in [
        "/read/thread/epiphanyState/investigationCheckpoint/codeRefs",
        "/read/thread/epiphanyState/investigationCheckpoint/code_refs",
        "/read/thread/epiphanyState/investigation_checkpoint/code_refs",
        "/scene/scene/investigationCheckpoint/codeRefs",
        "/scene/scene/investigationCheckpoint/code_refs",
        "/read/thread/epiphanyState/graphFrontier/dirtyPaths",
        "/read/thread/epiphanyState/graphFrontier/dirty_paths",
        "/read/thread/epiphanyState/graph_frontier/dirty_paths",
        "/scene/scene/graphFrontier/dirtyPaths",
        "/scene/scene/graphFrontier/dirty_paths",
    ] {
        collect_requested_paths_at(status, pointer, &mut paths);
    }
    paths.sort();
    paths.dedup();
    if paths.is_empty() {
        paths.push(".".to_string());
    }
    paths
}

fn collect_requested_paths_at(status: &Value, pointer: &str, paths: &mut Vec<String>) {
    match status.pointer(pointer) {
        Some(Value::Array(items)) => {
            for item in items {
                if let Some(path) = item
                    .as_str()
                    .or_else(|| item.get("path").and_then(Value::as_str))
                {
                    push_requested_path(paths, path);
                }
            }
        }
        Some(Value::String(path)) => push_requested_path(paths, path),
        _ => {}
    }
}

fn push_requested_path(paths: &mut Vec<String>, path: &str) {
    let normalized = path.trim().replace('\\', "/");
    if normalized.is_empty() {
        return;
    }
    paths.push(normalized);
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

fn role_id_for_coordinator_action(action: &str) -> Option<&'static str> {
    match action {
        "launchResearch" | "reviewResearchResult" => Some("research"),
        "launchModeling" | "reviewModelingResult" => Some("modeling"),
        "launchVerification" | "reviewVerificationResult" => Some("verification"),
        _ => None,
    }
}

fn wait_action_for_role(role_id: &str) -> &'static str {
    match role_id {
        "research" => "waitForResearchResult",
        "modeling" => "waitForModelingResult",
        "verification" => "waitForVerificationResult",
        _ => "waitForRoleResult",
    }
}

fn review_action_for_role(role_id: &str) -> &'static str {
    match role_id {
        "research" => "reviewResearchResult",
        "modeling" => "reviewModelingResult",
        "verification" => "reviewVerificationResult",
        _ => "reviewRoleResult",
    }
}

fn role_result_auto_acceptable(role_id: &str, result: &Value) -> bool {
    if result["status"].as_str() != Some("completed") {
        return false;
    }
    let finding = &result["finding"];
    if finding["jobError"].is_string() || finding["itemError"].is_string() {
        return false;
    }
    let has_runtime_identity = finding["runtimeResultId"]
        .as_str()
        .is_some_and(|id| !id.trim().is_empty())
        && finding["runtimeJobId"]
            .as_str()
            .is_some_and(|id| !id.trim().is_empty());
    if !has_runtime_identity {
        return false;
    }
    match role_id {
        "verification" => true,
        "research" | "modeling" | "imagination" => finding.get("statePatch").is_some(),
        _ => false,
    }
}

fn role_result_needs_supersession(role_id: &str, result: &Value) -> bool {
    match result["status"].as_str() {
        Some("failed") => true,
        Some("completed") if matches!(role_id, "research" | "modeling" | "imagination") => {
            !role_result_auto_acceptable(role_id, result)
        }
        _ => false,
    }
}

fn supersede_role_result(
    client: &mut status_cli::AppServerClient,
    thread_id: &str,
    role_id: &str,
    expected_revision: Option<i64>,
    result: &Value,
) -> Result<Value> {
    let result_id = first_string_at(result, &[&["finding", "runtimeResultId"]])
        .context("failed role result did not include finding.runtimeResultId")?;
    let job_id = first_string_at(result, &[&["finding", "runtimeJobId"]])
        .context("failed role result did not include finding.runtimeJobId")?;
    let binding_id = first_string_at(result, &[&["bindingId"]])
        .unwrap_or_else(|| default_binding_id_for_role(role_id));
    let summary = first_string_at(result, &[&["finding", "summary"], &["note"]])
        .unwrap_or_else(|| "Role result reviewed and superseded.".to_string());
    let receipt = json!({
        "id": format!("role-failure-review-{}", Uuid::new_v4()),
        "result_id": result_id,
        "job_id": job_id,
        "binding_id": binding_id,
        "surface": "roleFailureReview",
        "role_id": role_id,
        "status": "superseded",
        "accepted_at": now(),
        "summary": summary,
    });
    let mut payload = json!({
        "threadId": thread_id,
        "patch": {
            "acceptanceReceipts": [receipt],
        },
    });
    if let Some(revision) = expected_revision {
        payload["expectedRevision"] = json!(revision);
    }
    client.send("thread/epiphany/update", Some(payload), true)
}

fn default_binding_id_for_role(role_id: &str) -> String {
    match role_id {
        "imagination" => epiphany_core::EPIPHANY_IMAGINATION_ROLE_BINDING_ID.to_string(),
        "research" => epiphany_core::EPIPHANY_RESEARCH_ROLE_BINDING_ID.to_string(),
        "modeling" => epiphany_core::EPIPHANY_MODELING_ROLE_BINDING_ID.to_string(),
        "verification" => epiphany_core::EPIPHANY_VERIFICATION_ROLE_BINDING_ID.to_string(),
        _ => role_id.to_string(),
    }
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

fn worker_job_id_from_launch(launch: &Value) -> Result<String> {
    first_string_at(
        launch,
        &[
            &["job", "backendJobId"],
            &["job", "runtimeAgentJobId"],
            &["job", "runtimeJobId"],
            &["backendJobId"],
            &["runtimeAgentJobId"],
            &["runtimeJobId"],
        ],
    )
    .ok_or_else(|| anyhow!("launch response did not include a runtime worker job id"))
}

fn launch_worker_runtime_detached(
    model_runtime_bin: &Path,
    tool_adapter_bin: &Path,
    model_provider: &str,
    runtime_store: &Path,
    codex_home: &Path,
    cwd: &Path,
    job_id: &str,
    role_id: &str,
    step_index: usize,
    artifact_dir: &Path,
    max_runtime_seconds: u64,
    auto_tools: bool,
) -> Result<Value> {
    let stdout_path = artifact_dir.join(format!(
        "step-{step_index:02}-{}-worker-runtime.stdout.json",
        sanitize_id(role_id)
    ));
    let stderr_path = artifact_dir.join(format!(
        "step-{step_index:02}-{}-worker-runtime.stderr.log",
        sanitize_id(role_id)
    ));
    let mut command = Command::new(model_runtime_bin);
    command
        .arg("run-worker")
        .arg("--provider")
        .arg(model_provider)
        .arg("--store")
        .arg(runtime_store)
        .arg("--codex-home")
        .arg(codex_home)
        .arg("--job-id")
        .arg(job_id)
        .arg("--max-runtime-seconds")
        .arg(max_runtime_seconds.to_string());
    if auto_tools {
        command
            .arg("--auto-tools")
            .arg("--tool-adapter-bin")
            .arg(tool_adapter_bin)
            .arg("--cwd")
            .arg(cwd)
            .arg("--max-tool-rounds")
            .arg(WORKER_AUTO_TOOL_MAX_ROUNDS.to_string());
    }
    let stdout_file = fs::File::create(&stdout_path)
        .with_context(|| format!("failed to create {}", stdout_path.display()))?;
    let stderr_file = fs::File::create(&stderr_path)
        .with_context(|| format!("failed to create {}", stderr_path.display()))?;
    command
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file));
    #[cfg(windows)]
    command.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
    let child = command
        .spawn()
        .with_context(|| format!("failed to spawn {}", model_runtime_bin.display()))?;
    Ok(json!({
        "status": "launched",
        "jobId": job_id,
        "pid": child.id(),
        "stdout": stdout_path,
        "stderr": stderr_path,
        "note": "Worker runtime is detached; coordinator observes completion through runtime-spine role/reorient result polling.",
    }))
}

fn read_role_result(
    client: &mut status_cli::AppServerClient,
    thread_id: &str,
    role_id: &str,
) -> Result<Value> {
    client.send(
        "thread/epiphany/roleResult",
        Some(json!({"threadId": thread_id, "roleId": role_id})),
        true,
    )
}

fn read_reorient_result(
    client: &mut status_cli::AppServerClient,
    thread_id: &str,
) -> Result<Value> {
    client.send(
        "thread/epiphany/reorientResult",
        Some(json!({"threadId": thread_id, "bindingId": REORIENT_BINDING_ID})),
        true,
    )
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

fn ensure_runtime_session(
    runtime_store: &Path,
    session_id: &str,
    objective: &str,
) -> Result<epiphany_core::EpiphanyRuntimeSession> {
    match create_runtime_session(
        runtime_store,
        RuntimeSpineSessionOptions {
            session_id: session_id.to_string(),
            objective: objective.to_string(),
            created_at: now(),
            coordinator_note:
                "Coordinator owns native runtime receipts before Codex execution dies.".to_string(),
        },
    ) {
        Ok(session) => Ok(session),
        Err(err) if err.to_string().contains("already exists") => {
            let mut cache = epiphany_core::runtime_spine_cache(runtime_store)?;
            cache.pull_all_backing_stores()?;
            cache.get_required::<epiphany_core::EpiphanyRuntimeSession>(session_id)
        }
        Err(err) => Err(err),
    }
}

fn first_string_at(value: &Value, paths: &[&[&str]]) -> Option<String> {
    'paths: for path in paths {
        let mut cursor = value;
        for key in *path {
            let Some(next) = cursor.get(*key) else {
                continue 'paths;
            };
            cursor = next;
        }
        if let Some(value) = cursor.as_str()
            && !value.trim().is_empty()
        {
            return Some(value.to_string());
        }
    }
    None
}

fn coordinator_receipt_status(mode: &str, final_action: &Value) -> String {
    if mode == "plan" {
        return "planned".to_string();
    }
    if final_action_name(final_action) == "regatherManually" {
        "needsReview".to_string()
    } else {
        "completed".to_string()
    }
}

fn final_action_name(final_action: &Value) -> String {
    final_action
        .get("action")
        .and_then(Value::as_str)
        .filter(|action| !action.trim().is_empty())
        .unwrap_or("unknown")
        .to_string()
}

fn final_action_reason(final_action: &Value) -> Option<String> {
    final_action
        .get("reason")
        .and_then(Value::as_str)
        .filter(|reason| !reason.trim().is_empty())
        .map(ToString::to_string)
}

fn sanitize_id(value: &str) -> String {
    value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
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

fn local_mvp_checkpoint_patch(cwd: &Path, objective: Option<&str>) -> Value {
    let objective = objective
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Run the local Epiphany MVP cycle through Persona, coordinator, and sleep.");
    let local_refs = local_mvp_code_refs(cwd);
    let transport_refs = local_mvp_transport_code_refs(cwd);
    let mut checkpoint_refs = local_refs.clone();
    checkpoint_refs.extend(transport_refs.clone());
    json!({
        "objective": objective,
        "activeSubgoalId": "local-mvp-cycle",
        "subgoals": [{
            "id": "local-mvp-cycle",
            "title": "Local MVP cycle",
            "status": "active",
            "summary": "Use Persona as the human-facing entrypoint, run bounded coordinator/swarm work, and close with heartbeat sleep/dream maintenance.",
        }],
        "graphs": {
            "architecture": {"nodes": [
                {
                    "id": "local-mvp-runner",
                    "title": "Local MVP runner",
                    "purpose": "One local operator cycle that enters through Persona, runs coordinator-owned work, and invokes Continuity/heartbeat sleep afterward.",
                    "code_refs": local_refs,
                },
                {
                    "id": "model-runtime-transport",
                    "title": "Model runtime transport",
                    "purpose": "Provider-neutral worker execution resolves the model runtime binary and then uses the quarantined Codex/OpenAI transport for model calls.",
                    "code_refs": transport_refs,
                },
            ]},
            "dataflow": {"nodes": [
                {
                    "id": "Persona-coordinator-sleep-cycle",
                    "title": "Persona to coordinator to sleep",
                    "purpose": "Persona expression is display state; coordinator owns lane routing; heartbeat owns sleep physiology.",
                },
                {
                    "id": "coordinator-model-runtime-cycle",
                    "title": "Coordinator to model runtime",
                    "purpose": "Coordinator launches a typed worker request; epiphany-model-runtime chooses the provider/model and the Codex/OpenAI spine only transports the turn.",
                },
            ]},
            "links": [
                {
                    "dataflow_node_id": "Persona-coordinator-sleep-cycle",
                    "architecture_node_id": "local-mvp-runner",
                },
                {
                    "dataflow_node_id": "coordinator-model-runtime-cycle",
                    "architecture_node_id": "model-runtime-transport",
                },
            ],
        },
        "graphFrontier": {
            "active_node_ids": ["local-mvp-runner", "Persona-coordinator-sleep-cycle", "model-runtime-transport", "coordinator-model-runtime-cycle"],
            "dirty_paths": [],
        },
        "graphCheckpoint": {
            "checkpoint_id": "ck-local-mvp-cycle",
            "graph_revision": 2,
            "summary": "Local MVP runner checkpoint: Persona front door, coordinator run, model runtime transport, sleep maintenance.",
            "frontier_node_ids": ["local-mvp-runner", "Persona-coordinator-sleep-cycle", "model-runtime-transport", "coordinator-model-runtime-cycle"],
        },
        "investigationCheckpoint": {
            "checkpoint_id": "ix-local-mvp-cycle",
            "kind": "source_gathering",
            "disposition": "resume_ready",
            "focus": "Continue the local MVP cycle from the typed launcher/coordinator/model-runtime/sleep artifacts.",
            "summary": "The local MVP path has enough startup state for the coordinator to route bounded work and for worker transport failures to be attributed to the provider/runtime seam instead of generic missing state.",
            "next_action": "Run the coordinator's recommended bounded lane action, then review generated artifacts before accepting state changes.",
            "captured_at_turn_id": "local-mvp-bootstrap",
            "code_refs": checkpoint_refs,
        },
        "churn": {
            "understanding_status": "ready",
            "diff_pressure": "low",
            "graph_freshness": "fresh",
            "unexplained_writes": 0,
        },
    })
}

fn local_mvp_code_refs(cwd: &Path) -> Vec<Value> {
    let candidates = [
        (
            "tools/epiphany_local_run.ps1",
            1_u64,
            30_u64,
            "epiphany_local_run.ps1",
        ),
        ("README.md", 82, 111, "Run Locally"),
        (
            "notes/fresh-workspace-handoff.md",
            33,
            45,
            "Current Orientation",
        ),
        ("state/map.yaml", 477, 487, "Runnability"),
    ];
    candidates
        .into_iter()
        .filter(|(path, _, _, _)| cwd.join(path).exists())
        .map(|(path, start_line, end_line, symbol)| {
            json!({
                "path": path,
                "start_line": start_line,
                "end_line": end_line,
                "symbol": symbol,
            })
        })
        .collect()
}

fn local_mvp_transport_code_refs(cwd: &Path) -> Vec<Value> {
    let candidates = [
        (
            "epiphany-core/src/bin/epiphany-mvp-coordinator.rs",
            719_u64,
            787_u64,
            "resolve_model_runtime_bin / run_worker_runtime",
        ),
        (
            "epiphany-openai-runtime/src/bin/epiphany-openai-runtime.rs",
            431,
            438,
            "default_worker_model",
        ),
        (
            "epiphany-openai-codex-spine/src/lib.rs",
            146,
            170,
            "open_responses_stream",
        ),
        (
            "epiphany-openai-codex-spine/src/lib.rs",
            447,
            463,
            "attach_codex_auth_headers",
        ),
    ];
    candidates
        .into_iter()
        .filter(|(path, _, _, _)| cwd.join(path).exists())
        .map(|(path, start_line, end_line, symbol)| {
            json!({
                "path": path,
                "start_line": start_line,
                "end_line": end_line,
                "symbol": symbol,
            })
        })
        .collect()
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
    )
}

fn is_result_review_action(action: &str) -> bool {
    matches!(
        action,
        "reviewModelingResult" | "reviewVerificationResult" | "reviewReorientResult"
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

fn append_operator_step_jsonl(path: &Path, step: &Value) -> Result<()> {
    append_jsonl(path, &status_cli::sanitize_for_operator(step.clone()))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn continue_implementation_is_not_a_passive_stop_action() {
        assert!(!is_stop_action("continueImplementation"));
    }

    #[test]
    fn auto_review_accepts_receipt_only_verification_findings() {
        let result = json!({
            "status": "completed",
            "finding": {
                "verdict": "needs-evidence",
                "summary": "Soul needs a source-backed receipt path.",
                "runtimeResultId": "result-verification-1",
                "runtimeJobId": "job-verification-1"
            }
        });

        assert!(role_result_auto_acceptable("verification", &result));
        assert!(!role_result_auto_acceptable("modeling", &result));
    }

    #[test]
    fn auto_review_refuses_identityless_or_errored_findings() {
        let missing_identity = json!({
            "status": "completed",
            "finding": {
                "statePatch": {"scratch": {"summary": "mapped"}},
                "runtimeResultId": "",
                "runtimeJobId": "job-modeling-1"
            }
        });
        let errored = json!({
            "status": "completed",
            "finding": {
                "statePatch": {"scratch": {"summary": "mapped"}},
                "runtimeResultId": "result-modeling-1",
                "runtimeJobId": "job-modeling-1",
                "itemError": "missing required field"
            }
        });

        assert!(!role_result_auto_acceptable("modeling", &missing_identity));
        assert!(!role_result_auto_acceptable("modeling", &errored));
    }

    #[test]
    fn supersession_includes_unreviewable_modeling_results() {
        let unreviewable = json!({
            "status": "completed",
            "finding": {
                "verdict": "checkpoint-update-needed",
                "summary": "Mapped in prose only.",
                "runtimeResultId": "result-modeling-1",
                "runtimeJobId": "job-modeling-1",
                "itemError": "modeling result is not reviewable: missing required statePatch"
            }
        });
        let reviewable = json!({
            "status": "completed",
            "finding": {
                "verdict": "checkpoint-update-needed",
                "summary": "Mapped with state.",
                "runtimeResultId": "result-modeling-2",
                "runtimeJobId": "job-modeling-2",
                "statePatch": {"scratch": {"summary": "mapped"}}
            }
        });

        assert!(role_result_needs_supersession("modeling", &unreviewable));
        assert!(!role_result_needs_supersession("modeling", &reviewable));
        assert!(!role_result_needs_supersession(
            "verification",
            &unreviewable
        ));
    }

    #[test]
    fn implementation_requested_paths_use_checkpoint_and_frontier_scope() {
        let status = json!({
            "read": {
                "thread": {
                    "epiphanyState": {
                        "investigationCheckpoint": {
                            "codeRefs": [
                                {"path": "src\\main.rs"},
                                {"path": "src/main.rs"},
                                {"path": " notes/demo.md "}
                            ]
                        },
                        "graphFrontier": {
                            "dirtyPaths": ["tests/demo.rs"]
                        }
                    }
                }
            }
        });

        assert_eq!(
            implementation_requested_paths(&status),
            vec![
                "notes/demo.md".to_string(),
                "src/main.rs".to_string(),
                "tests/demo.rs".to_string()
            ]
        );
    }

    #[test]
    fn hands_implementation_gate_persists_grant_intent_and_review() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime-spine.msgpack");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "epiphany-test".to_string(),
                display_name: "Epiphany Test".to_string(),
                created_at: "2026-06-02T00:00:00Z".to_string(),
            },
        )?;
        let status = json!({
            "scene": {
                "scene": {
                    "investigationCheckpoint": {
                        "codeRefs": [{"path": "epiphany-core/src/lib.rs"}]
                    }
                }
            }
        });

        let gate =
            record_hands_implementation_gate(&store, temp.path(), "thread-test", 2, &status)?;
        let grant_id = gate["substrateGateGrantReceiptId"]
            .as_str()
            .expect("grant id");
        let intent_id = gate["intentId"].as_str().expect("intent id");
        let review_id = gate["reviewId"].as_str().expect("review id");

        let grant =
            epiphany_core::runtime_substrate_gate_repo_access_grant_receipt(&store, grant_id)?
                .expect("stored substrate grant");
        let intent = epiphany_core::runtime_hands_action_intent(&store, intent_id)?
            .expect("stored Hands intent");
        let review = epiphany_core::runtime_hands_action_review(&store, review_id)?
            .expect("stored Hands review");

        assert_eq!(grant.receipt_id, intent.substrate_gate_grant_receipt_id);
        assert_eq!(intent.intent_id, review.intent_id);
        assert_eq!(intent.requested_action, "continueImplementation");
        assert_eq!(
            gate.pointer("/recordPassCommand/executable")
                .and_then(serde_json::Value::as_str),
            Some("epiphany-hands-action")
        );
        assert_eq!(
            review.required_receipts,
            vec![
                HANDS_PATCH_RECEIPT_TYPE.to_string(),
                HANDS_COMMAND_RECEIPT_TYPE.to_string(),
                HANDS_COMMIT_RECEIPT_TYPE.to_string(),
            ]
        );
        Ok(())
    }
}
