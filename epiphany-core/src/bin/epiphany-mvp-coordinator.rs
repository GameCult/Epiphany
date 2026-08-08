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
use epiphany_core::REPO_FRONTIER_HANDS_AUTHORITY_CONTRACT;
use epiphany_core::REPO_FRONTIER_HANDS_AUTHORITY_SCHEMA_VERSION;
use epiphany_core::RepoFrontierHandsAuthority;
use epiphany_core::RuntimeSpineEventOptions;
use epiphany_core::RuntimeSpineInitOptions;
use epiphany_core::RuntimeSpineSessionOptions;
use epiphany_core::append_runtime_event;
use epiphany_core::create_runtime_session;
use epiphany_core::hands_action_review_for_intent;
use epiphany_core::initialize_runtime_spine;
use epiphany_core::load_epiphany_cultmesh_swarm_brake;
use epiphany_core::put_coordinator_run_receipt;
use epiphany_core::put_hands_action_intent;
use epiphany_core::put_hands_action_review;
use epiphany_core::put_repo_frontier_hands_authority;
use epiphany_core::put_substrate_gate_repo_access_grant_receipt;
use epiphany_core::runtime_spine_status;
use epiphany_core::select_and_commit_repo_frontier_route;
use epiphany_core::substrate_gate_coordinator_implementation_grant;
use epiphany_core::{
    LaunchedCoordinator, capture_process_instance, claim_resident_self_preparation_as_child,
};
use serde_json::Value;
use serde_json::json;
use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;
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

const DEFAULT_MODEL_RUNTIME_BIN: &str = "epiphany-model-runtime";
const DEFAULT_TOOL_ADAPTER_BIN: &str = "epiphany-tool-mcp-runtime";
const WORKER_AUTO_TOOL_MAX_ROUNDS: usize = 24;

fn main() -> Result<()> {
    let args = Args::parse()?;
    let summary = run_coordinator(&args)?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

#[derive(Debug)]
struct Args {
    model_runtime_bin: PathBuf,
    tool_adapter_bin: PathBuf,
    model_provider: String,
    thread_id: Option<String>,
    objective: Option<String>,
    cwd: PathBuf,
    codex_home: PathBuf,
    mcp_config: PathBuf,
    artifact_dir: PathBuf,
    agent_memory_dir: PathBuf,
    runtime_store: PathBuf,
    local_verse_store: PathBuf,
    mode: String,
    max_steps: usize,
    poll_seconds: f64,
    timeout_seconds: u64,
    max_runtime_seconds: u64,
    ephemeral: bool,
    auto_review: bool,
    supersede_failed_results: bool,
    approve_manual_regather: bool,
    auto_tools: bool,
    proposal_modeling_request_id: Option<String>,
    imagination_consideration_request_id: Option<String>,
    admitted_model_direction_consideration_request_id: Option<String>,
    resident_binding: BTreeMap<String, String>,
    resident_state_store: Option<PathBuf>,
    resident_preparation_id: Option<String>,
    runtime_id: Option<String>,
}

impl Args {
    fn parse() -> Result<Self> {
        let root = env::current_dir().context("failed to resolve current dir")?;
        let mut args = env::args().skip(1);
        let mut parsed = Args {
            model_runtime_bin: PathBuf::from(DEFAULT_MODEL_RUNTIME_BIN),
            tool_adapter_bin: PathBuf::from(DEFAULT_TOOL_ADAPTER_BIN),
            model_provider: "openai-codex".to_string(),
            thread_id: None,
            objective: None,
            cwd: root.clone(),
            codex_home: env::var_os("CODEX_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir().join(".codex")),
            mcp_config: root.join(".epiphany").join("mcp.toml"),
            artifact_dir: root.join(".epiphany-dogfood").join("coordinator"),
            agent_memory_dir: root.join("state").join("agents.msgpack"),
            runtime_store: root.join("state").join("runtime-spine.msgpack"),
            local_verse_store: root
                .join(".epiphany-run")
                .join("cultmesh")
                .join("local-verse.ccmp"),
            mode: "plan".to_string(),
            max_steps: 4,
            poll_seconds: 5.0,
            timeout_seconds: 600,
            max_runtime_seconds: 180,
            ephemeral: true,
            auto_review: false,
            supersede_failed_results: false,
            approve_manual_regather: false,
            auto_tools: true,
            proposal_modeling_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            resident_binding: BTreeMap::new(),
            resident_state_store: None,
            resident_preparation_id: None,
            runtime_id: None,
        };
        while let Some(arg) = args.next() {
            match arg.as_str() {
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
                "--objective" => parsed.objective = Some(take_string(&mut args, "--objective")?),
                "--proposal-modeling-request-id" => {
                    parsed.proposal_modeling_request_id =
                        Some(take_string(&mut args, "--proposal-modeling-request-id")?);
                }
                "--imagination-consideration-request-id" => {
                    parsed.imagination_consideration_request_id = Some(take_string(
                        &mut args,
                        "--imagination-consideration-request-id",
                    )?);
                }
                "--admitted-model-direction-consideration-request-id" => {
                    parsed.admitted_model_direction_consideration_request_id = Some(take_string(
                        &mut args,
                        "--admitted-model-direction-consideration-request-id",
                    )?);
                }
                flag @ ("--resident-grant-id"
                | "--resident-launch-digest"
                | "--resident-policy-digest"
                | "--resident-argv-digest"
                | "--resident-objective-digest"
                | "--resident-release-commit"
                | "--resident-release-manifest-digest"
                | "--resident-executable-digest") => {
                    parsed.resident_binding.insert(
                        flag.trim_start_matches("--resident-").to_string(),
                        take_string(&mut args, flag)?,
                    );
                }
                "--resident-state-store" => {
                    parsed.resident_state_store =
                        Some(take_path(&mut args, "--resident-state-store")?)
                }
                "--resident-preparation-id" => {
                    parsed.resident_preparation_id =
                        Some(take_string(&mut args, "--resident-preparation-id")?)
                }
                "--runtime-id" => parsed.runtime_id = Some(take_string(&mut args, "--runtime-id")?),
                "--cwd" => parsed.cwd = take_path(&mut args, "--cwd")?,
                "--codex-home" => parsed.codex_home = take_path(&mut args, "--codex-home")?,
                "--mcp-config" => parsed.mcp_config = take_path(&mut args, "--mcp-config")?,
                "--artifact-dir" => parsed.artifact_dir = take_path(&mut args, "--artifact-dir")?,
                "--agent-memory-dir" => {
                    parsed.agent_memory_dir = take_path(&mut args, "--agent-memory-dir")?;
                }
                "--runtime-store" => {
                    parsed.runtime_store = take_path(&mut args, "--runtime-store")?
                }
                "--local-verse-store" => {
                    parsed.local_verse_store = take_path(&mut args, "--local-verse-store")?
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
                "--approve-manual-regather" => parsed.approve_manual_regather = true,
                "--auto-tools" => parsed.auto_tools = true,
                "--no-auto-tools" => parsed.auto_tools = false,
                "--test-complete-backend" => {
                    return Err(anyhow!(
                        "--test-complete-backend was removed: native coordinator refuses direct private state-store job mutation; use live workers or a future CultNet job-result API"
                    ));
                }
                other => return Err(anyhow!("unknown argument: {other}")),
            }
        }
        if parsed.runtime_id.as_deref().is_none_or(str::is_empty) {
            return Err(anyhow!(
                "coordinator requires exact --runtime-id for local Verse authority"
            ));
        }
        Ok(parsed)
    }
}

fn run_coordinator(args: &Args) -> Result<Value> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    match (&args.resident_state_store, &args.resident_preparation_id) {
        (Some(store), Some(preparation_id)) => {
            let identity = capture_process_instance(std::process::id())?;
            claim_resident_self_preparation_as_child(
                store,
                preparation_id,
                &LaunchedCoordinator {
                    process_id: identity.process_id,
                    process_creation_token: identity.creation_token,
                    process_executable_path: identity.executable_path,
                },
                chrono::Utc::now().timestamp_millis().max(0) as u64,
            )?;
        }
        (None, None) => {}
        _ => {
            return Err(anyhow!(
                "resident coordinator bootstrap requires both state store and preparation id"
            ));
        }
    }
    let local_verse_store = status_cli::absolute_path(&args.local_verse_store)?;
    assert_local_verse_brake_released(
        &local_verse_store,
        args.runtime_id.as_deref().expect("validated runtime id"),
        "epiphany-mvp-coordinator",
    )?;
    let cwd = status_cli::absolute_path(&args.cwd)?;
    let model_runtime_bin = resolve_model_runtime_bin(&root, &args.model_runtime_bin)?;
    let tool_adapter_bin = resolve_model_runtime_bin(&root, &args.tool_adapter_bin)?;
    let codex_home = status_cli::absolute_path(&args.codex_home)?;
    let mcp_config = status_cli::absolute_path(&args.mcp_config)?;
    let artifact_dir = status_cli::absolute_path(&args.artifact_dir)?;
    let agent_memory_dir = status_cli::absolute_path(&args.agent_memory_dir)?;
    let runtime_store = status_cli::absolute_path(&args.runtime_store)?;
    reset_artifact_dir(&artifact_dir)?;
    fs::create_dir_all(&codex_home)?;

    let telemetry_path = artifact_dir.join("agent-function-telemetry.json");
    let steps_path = artifact_dir.join("coordinator-steps.jsonl");
    let mut steps = Vec::new();
    let mut snapshots = Vec::new();
    let mut startup_events = Vec::new();
    let mut final_status = Value::Null;
    let mut final_action = Value::Null;

    let thread_id = args
        .thread_id
        .clone()
        .unwrap_or_else(|| format!("epiphany-native-{}", Uuid::new_v4()));
    startup_events.push(json!({
        "type": "nativeCoordinatorThread",
        "threadId": thread_id,
        "ephemeral": args.ephemeral,
        "workspace": cwd,
    }));
    let runtime_session_id = format!("coordinator-{thread_id}");
    let runtime_identity = initialize_runtime_spine(
        &runtime_store,
        coordinator_runtime_identity_options(
            args.runtime_id.as_deref().expect("validated runtime id"),
            now(),
        ),
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
    if let Some(objective) = args.objective.as_deref() {
        let intake = intake_operator_objective(&runtime_store, &thread_id, objective)?;
        startup_events.push(json!({
            "type": "operatorObjectiveIntake",
            "threadId": thread_id,
            "revision": intake.state.revision,
            "changed": intake.changed,
        }));
    }
    if let Some(request_id) = args.imagination_consideration_request_id.as_deref() {
        if args.objective.is_some() {
            return Err(anyhow!(
                "typed consideration cannot share operator objective intake"
            ));
        }
        let service = epiphany_core::EpiphanyCoordinatorService::new(&runtime_store);
        let state = service
            .state()?
            .ok_or_else(|| anyhow!("consideration launch requires coordinator state"))?;
        let request = epiphany_core::build_epiphany_imagination_consideration_launch_request(
            &thread_id,
            Some(state.revision),
            Some(args.max_runtime_seconds),
            &state,
            request_id.to_string(),
        )
        .map_err(anyhow::Error::msg)?;
        let launched = service.launch_job(
            &thread_id,
            &state,
            &request,
            format!("epiphany-resident-consideration-launch-{}", Uuid::new_v4()),
            Uuid::new_v4().to_string(),
            now(),
        )?;
        let worker_job_id = launched.backend_job_id.clone();
        let operator_launch = json!({
            "bindingId": launched.binding_id,
            "launcherJobId": launched.launcher_job_id,
            "backendJobId": launched.backend_job_id,
            "stateRevision": launched.epiphany_state.revision,
        });
        let worker_run = launch_worker_runtime_detached(
            &model_runtime_bin,
            &tool_adapter_bin,
            &args.model_provider,
            &runtime_store,
            &codex_home,
            &mcp_config,
            &cwd,
            &worker_job_id,
            "imagination",
            0,
            &artifact_dir,
            args.max_runtime_seconds,
            args.auto_tools,
        )?;
        startup_events.push(json!({
            "type": "imaginationConsiderationLaunch",
            "requestId": request_id,
            "runtimeJobId": worker_job_id,
            "launch": status_cli::sanitize_for_operator(operator_launch),
            "worker": worker_run,
            "authority": "proposal-only"
        }));
    }

    if let Some(request_id) = args
        .admitted_model_direction_consideration_request_id
        .as_deref()
    {
        if args.objective.is_some() || args.imagination_consideration_request_id.is_some() {
            return Err(anyhow!(
                "typed model direction consideration requires exclusive objective-free intake"
            ));
        }
        let service = epiphany_core::EpiphanyCoordinatorService::new(&runtime_store);
        let state = service
            .state()?
            .ok_or_else(|| anyhow!("model direction launch requires coordinator state"))?;
        let request =
            epiphany_core::build_epiphany_admitted_model_direction_consideration_launch_request(
                &thread_id,
                Some(state.revision),
                Some(args.max_runtime_seconds),
                &state,
                request_id.to_string(),
            )
            .map_err(anyhow::Error::msg)?;
        let launched = service.launch_job(
            &thread_id,
            &state,
            &request,
            format!(
                "epiphany-resident-model-direction-launch-{}",
                Uuid::new_v4()
            ),
            Uuid::new_v4().to_string(),
            now(),
        )?;
        let worker_job_id = launched.backend_job_id.clone();
        let worker_run = launch_worker_runtime_detached(
            &model_runtime_bin,
            &tool_adapter_bin,
            &args.model_provider,
            &runtime_store,
            &codex_home,
            &mcp_config,
            &cwd,
            &worker_job_id,
            "imagination",
            0,
            &artifact_dir,
            args.max_runtime_seconds,
            args.auto_tools,
        )?;
        startup_events.push(json!({
            "type": "admittedModelDirectionConsiderationLaunch",
            "requestId": request_id,
            "runtimeJobId": worker_job_id,
            "worker": worker_run,
            "authority": "proposal-only"
        }));
    }

    if let Some(request_id) = args.proposal_modeling_request_id.as_deref() {
        if args.objective.is_some()
            || args.imagination_consideration_request_id.is_some()
            || args
                .admitted_model_direction_consideration_request_id
                .is_some()
        {
            return Err(anyhow!(
                "typed proposal Modeling requires exclusive objective-free intake"
            ));
        }
        let state = epiphany_core::EpiphanyCoordinatorService::new(&runtime_store)
            .state()?
            .ok_or_else(|| anyhow!("proposal Modeling launch requires coordinator state"))?;
        let launched = launch_role(
            &runtime_store,
            &local_verse_store,
            &thread_id,
            "modeling",
            Some(state.revision as i64),
            args.max_runtime_seconds,
            Some(request_id),
        )?;
        let worker_job_id = worker_job_id_from_launch(&launched)?;
        let worker_run = launch_worker_runtime_detached(
            &model_runtime_bin,
            &tool_adapter_bin,
            &args.model_provider,
            &runtime_store,
            &codex_home,
            &mcp_config,
            &cwd,
            &worker_job_id,
            "modeling",
            0,
            &artifact_dir,
            args.max_runtime_seconds,
            args.auto_tools,
        )?;
        startup_events.push(json!({
            "type": "proposalModelingLaunch",
            "requestId": request_id,
            "runtimeJobId": worker_job_id,
            "launch": status_cli::sanitize_for_operator(launched),
            "worker": worker_run,
            "authority": "proposal-modeling-only"
        }));
    }

    let mut manual_regather_approval = args.approve_manual_regather;
    for index in 0..args.max_steps {
        let status = collect_coordinator_status(&runtime_store, &thread_id)?;
        let mut coordinator = status
            .get("coordinator")
            .cloned()
            .unwrap_or_else(|| json!({"action": "regatherManually"}));
        let mut action = coordinator["action"]
            .as_str()
            .unwrap_or("regatherManually")
            .to_string();
        if manual_regather_approval {
            action = apply_manual_regather_approval(&action, &mut coordinator)?;
            manual_regather_approval = false;
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
        if action == "awaitFrontierProposal"
            || matches!(
                action.as_str(),
                "waitForImaginationResult" | "waitForMindPlanResult"
            )
            || (action == "reviewFrontierPlanningFailure" && !args.supersede_failed_results)
            || (is_stop_action(&action) && !args.auto_review && !is_result_review_action(&action))
        {
            append_operator_step_jsonl(&steps_path, &step)?;
            steps.push(step);
            break;
        }

        let revision = state_revision(&status);
        match action.as_str() {
            "reviewFrontierPlanningFailure" => {
                let lifecycle =
                    epiphany_core::runtime_repo_frontier_planning_lifecycle(&runtime_store)?;
                let (result_id, job_id, role_id, binding_id) = match lifecycle.stage {
                    epiphany_core::RepoFrontierPlanningLifecycleStage::ImaginationFailed => (
                        lifecycle.imagination_result_id.as_deref(),
                        lifecycle.imagination_job_id.as_deref(),
                        "imagination",
                        epiphany_core::EPIPHANY_IMAGINATION_ROLE_BINDING_ID,
                    ),
                    epiphany_core::RepoFrontierPlanningLifecycleStage::MindFailed => (
                        lifecycle.mind_result_id.as_deref(),
                        lifecycle.mind_job_id.as_deref(),
                        "mindAdmissionReview",
                        epiphany_core::EPIPHANY_MIND_ROLE_BINDING_ID,
                    ),
                    _ => return Err(anyhow!("frontier planning failure review stage changed")),
                };
                let result_id = result_id.ok_or_else(|| {
                    anyhow!("frontier planning failure review requires a typed result")
                })?;
                let job_id = job_id.ok_or_else(|| {
                    anyhow!("frontier planning failure review requires a launch job")
                })?;
                let reviewed = supersede_frontier_planning_failure(
                    &runtime_store,
                    &thread_id,
                    revision,
                    result_id,
                    job_id,
                    role_id,
                    binding_id,
                )?;
                push_event(
                    &mut step,
                    json!({"type": "frontierPlanningFailureReview", "review": status_cli::sanitize_for_operator(reviewed)}),
                );
                final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
            }
            "startFrontierPlanning" => {
                let request = epiphany_core::select_and_commit_repo_frontier_planning_request(
                    &runtime_store,
                    &now(),
                )?;
                push_event(
                    &mut step,
                    json!({"type": "frontierPlanningRequest", "request": request}),
                );
                final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
            }
            "launchImagination" => {
                let lifecycle =
                    epiphany_core::runtime_repo_frontier_planning_lifecycle(&runtime_store)?;
                let request_id = lifecycle.planning_request_id.as_deref().ok_or_else(|| {
                    anyhow!("launchImagination requires a current planning request")
                })?;
                let launch = launch_frontier_imagination(
                    &runtime_store,
                    &local_verse_store,
                    &thread_id,
                    revision,
                    args.max_runtime_seconds,
                    request_id,
                )?;
                let worker_job_id = worker_job_id_from_launch(&launch)?;
                push_event(
                    &mut step,
                    json!({"type": "frontierImaginationLaunch", "launch": status_cli::sanitize_for_operator(launch), "runtimeJobId": worker_job_id}),
                );
                let worker_run = launch_worker_runtime_detached(
                    &model_runtime_bin,
                    &tool_adapter_bin,
                    &args.model_provider,
                    &runtime_store,
                    &codex_home,
                    &mcp_config,
                    &cwd,
                    &worker_job_id,
                    "imagination-frontier-planning",
                    index,
                    &artifact_dir,
                    args.max_runtime_seconds,
                    args.auto_tools,
                )?;
                push_event(
                    &mut step,
                    json!({"type": "workerRuntime", "roleId": "imagination", "run": worker_run}),
                );
                final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
            }
            "requestMindPlanReview" => {
                let lifecycle =
                    epiphany_core::runtime_repo_frontier_planning_lifecycle(&runtime_store)?;
                let result_id = lifecycle.imagination_result_id.as_deref().ok_or_else(|| {
                    anyhow!("requestMindPlanReview requires a typed Imagination result")
                })?;
                let request = epiphany_core::commit_repo_frontier_plan_mind_request(
                    &runtime_store,
                    result_id,
                    &now(),
                )?;
                push_event(
                    &mut step,
                    json!({"type": "frontierPlanMindRequest", "request": request}),
                );
                final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
            }
            "launchMindPlanReview" => {
                let lifecycle =
                    epiphany_core::runtime_repo_frontier_planning_lifecycle(&runtime_store)?;
                let request_id = lifecycle.mind_request_id.as_deref().ok_or_else(|| {
                    anyhow!("launchMindPlanReview requires a current Mind request")
                })?;
                let launch = launch_frontier_mind(
                    &runtime_store,
                    &thread_id,
                    revision,
                    args.max_runtime_seconds,
                    request_id,
                )?;
                let worker_job_id = worker_job_id_from_launch(&launch)?;
                push_event(
                    &mut step,
                    json!({"type": "frontierMindLaunch", "launch": status_cli::sanitize_for_operator(launch), "runtimeJobId": worker_job_id}),
                );
                let worker_run = launch_worker_runtime_detached(
                    &model_runtime_bin,
                    &tool_adapter_bin,
                    &args.model_provider,
                    &runtime_store,
                    &codex_home,
                    &mcp_config,
                    &cwd,
                    &worker_job_id,
                    "mind-frontier-plan-review",
                    index,
                    &artifact_dir,
                    args.max_runtime_seconds,
                    args.auto_tools,
                )?;
                push_event(
                    &mut step,
                    json!({"type": "workerRuntime", "roleId": "mind", "run": worker_run}),
                );
                final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
            }
            "commitFrontierPlanDecision" => {
                let lifecycle =
                    epiphany_core::runtime_repo_frontier_planning_lifecycle(&runtime_store)?;
                let result_id = lifecycle.mind_result_id.as_deref().ok_or_else(|| {
                    anyhow!("commitFrontierPlanDecision requires a typed Mind result")
                })?;
                let decision =
                    epiphany_core::commit_repo_frontier_plan_decision(&runtime_store, result_id)?;
                push_event(
                    &mut step,
                    json!({"type": "frontierPlanDecision", "decision": decision}),
                );
                final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
            }
            "reviewResearchResult" | "reviewModelingResult" | "reviewVerificationResult" => {
                let role_id = role_id_for_coordinator_action(&action)
                    .ok_or_else(|| anyhow!("unsupported review action {action}"))?;
                let result = read_role_result(&runtime_store, &thread_id, role_id)?;
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
                        &runtime_store,
                        &thread_id,
                        role_id,
                        revision,
                        &result,
                    )?;
                    push_event(
                        &mut step,
                        json!({"type": "roleFailureReview", "roleId": role_id, "superseded": status_cli::sanitize_for_operator(superseded)}),
                    );
                    final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
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
                let accepted = match accept_role(
                    &runtime_store,
                    &thread_id,
                    role_id,
                    revision.and_then(|value| u64::try_from(value).ok()),
                ) {
                    Ok(accepted) => accepted,
                    Err(error) if args.supersede_failed_results => {
                        let superseded = supersede_role_result(
                            &runtime_store,
                            &thread_id,
                            role_id,
                            revision,
                            &result,
                        )?;
                        push_event(
                            &mut step,
                            json!({
                                "type": "roleAdmissionRejected",
                                "roleId": role_id,
                                "error": format!("{error:#}"),
                                "superseded": status_cli::sanitize_for_operator(superseded),
                            }),
                        );
                        final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
                        append_operator_step_jsonl(&steps_path, &step)?;
                        steps.push(step);
                        continue;
                    }
                    Err(error) => return Err(error),
                };
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
                final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
            }
            "reviewReorientResult" => {
                let result = read_reorient_result(&runtime_store, &thread_id)?;
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
                let can_accept = args.auto_review
                    && reorient_result_auto_acceptable(&result)
                    && revision.is_some();
                if can_accept {
                    let accepted = accept_reorient(
                        &runtime_store,
                        &thread_id,
                        revision.and_then(|value| u64::try_from(value).ok()),
                    )?;
                    push_event(
                        &mut step,
                        json!({"type": "reorientAccept", "accepted": status_cli::sanitize_for_operator(accepted)}),
                    );
                    final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
                    append_operator_step_jsonl(&steps_path, &step)?;
                    steps.push(step);
                    continue;
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
                let pending_proposal_request_id = if role_id == "modeling" {
                    status["findingSignals"]["pendingProposalModelingRequestId"].as_str()
                } else {
                    None
                };
                let proposal_modeling_request_id = if role_id == "modeling" {
                    resolve_proposal_modeling_request_hint(
                        args.proposal_modeling_request_id.as_deref(),
                        pending_proposal_request_id,
                    )?
                } else {
                    None
                };
                let launch = launch_role(
                    &runtime_store,
                    &local_verse_store,
                    &thread_id,
                    role_id,
                    revision,
                    args.max_runtime_seconds,
                    if role_id == "modeling" {
                        proposal_modeling_request_id
                    } else {
                        None
                    },
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
                    &mcp_config,
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
                let result = read_role_result(&runtime_store, &thread_id, role_id)?;
                push_event(
                    &mut step,
                    json!({"type": "roleResult", "roleId": role_id, "result": status_cli::sanitize_for_operator(result.clone())}),
                );
                final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
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
                let launch = launch_reorient(
                    &runtime_store,
                    &local_verse_store,
                    &thread_id,
                    revision,
                    args.max_runtime_seconds,
                )?;
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
                    &mcp_config,
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
                let result = read_reorient_result(&runtime_store, &thread_id)?;
                push_event(
                    &mut step,
                    json!({"type": "reorientResult", "result": status_cli::sanitize_for_operator(result.clone())}),
                );
                final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
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
    let sealed_artifact_manifest = Vec::new();
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
            ("mcpConfig".to_string(), mcp_config.display().to_string()),
            ("autoTools".to_string(), args.auto_tools.to_string()),
        ]),
        resident_grant_id: args.resident_binding.get("grant-id").cloned(),
        resident_launch_digest: args.resident_binding.get("launch-digest").cloned(),
        resident_policy_digest: args.resident_binding.get("policy-digest").cloned(),
        resident_argv_digest: args.resident_binding.get("argv-digest").cloned(),
        resident_objective_digest: args.resident_binding.get("objective-digest").cloned(),
        resident_release_commit: args.resident_binding.get("release-commit").cloned(),
        resident_release_manifest_digest: args
            .resident_binding
            .get("release-manifest-digest")
            .cloned(),
        resident_executable_digest: args.resident_binding.get("executable-digest").cloned(),
    };
    put_coordinator_run_receipt(&runtime_store, &coordinator_run_receipt)?;
    let summary = json!({
        "objective": "Coordinate the Epiphany MVP lanes through native typed state and runtime organs.",
        "artifactDir": artifact_dir,
        "modelRuntimeBin": model_runtime_bin,
        "toolAdapterBin": tool_adapter_bin,
        "autoTools": args.auto_tools,
        "modelProvider": args.model_provider,
        "codexHome": codex_home,
        "mcpConfig": mcp_config,
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
        "sealedArtifactManifest": []
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
    write_json(
        &telemetry_path,
        &json!({
            "source": "epiphany-native-coordinator",
            "transport": "cultcache",
            "appServerCalls": 0,
            "rawTextExposed": false,
        }),
    )?;
    Ok(summary)
}

fn intake_operator_objective(
    runtime_store: &Path,
    thread_id: &str,
    objective: &str,
) -> Result<epiphany_core::UserObjectiveIntakeApplied> {
    let service = epiphany_core::EpiphanyCoordinatorService::new(runtime_store);
    service.intake_user_objective(epiphany_core::UserObjectiveIntakeInput {
        thread_id: thread_id.to_string(),
        objective: objective.to_string(),
        source_actor: "operator".to_string(),
        source_ref: "cli://epiphany-mvp-coordinator".to_string(),
        submitted_at: now(),
    })
}

fn assert_local_verse_brake_released(
    local_verse_store: &Path,
    runtime_id: &str,
    runner_name: &str,
) -> Result<()> {
    if !local_verse_store.exists() {
        return Ok(());
    }
    let Some(brake) = load_epiphany_cultmesh_swarm_brake(local_verse_store, runtime_id)? else {
        return Ok(());
    };
    if brake.status == "engaged" {
        anyhow::bail!(
            "{runner_name} refusing to run: local Verse swarm brake engaged; scope={}; protected={}; affected={}; reason={}",
            brake.scope,
            brake.protected_surfaces.join(","),
            brake.affected_clusters.join(","),
            brake.reason
        );
    }
    Ok(())
}

fn collect_coordinator_status(runtime_store: &Path, thread_id: &str) -> Result<Value> {
    status_cli::native_coordinator_json(runtime_store, thread_id)
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
    _status: &Value,
) -> Result<Value> {
    let requested_at = now();
    let route = select_and_commit_repo_frontier_route(runtime_store, &requested_at)?;
    let suffix = format!(
        "{}-{}-{}",
        sanitize_id(thread_id),
        step_index,
        uuid::Uuid::new_v4()
    );
    let runtime_job_id = format!("hands-implementation-{suffix}");
    let grant_id = format!("substrate-grant-{runtime_job_id}");
    let requested_paths = route.source_scope.clone();

    let substrate_grant = substrate_gate_coordinator_implementation_grant(
        grant_id.clone(),
        runtime_job_id.clone(),
        requested_paths.clone(),
        requested_at.clone(),
    );
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
        frontier_route_id: route
            .adopted_plan
            .as_ref()
            .map(|_| route.route_id.clone())
            .unwrap_or_default(),
        plan_candidate_sha256: route
            .adopted_plan
            .as_ref()
            .map(|plan| plan.candidate_sha256.clone())
            .unwrap_or_default(),
        plan_action: route
            .adopted_plan
            .as_ref()
            .map(|plan| plan.effective_action().to_string())
            .unwrap_or_default(),
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
    let authority = RepoFrontierHandsAuthority {
        schema_version: REPO_FRONTIER_HANDS_AUTHORITY_SCHEMA_VERSION.to_string(),
        authority_id: format!("repo-frontier-hands-authority-{}", intent.intent_id),
        route_id: route.route_id.clone(),
        model_revision: route.model_revision,
        model_hash: route.model_hash.clone(),
        frontier_item_id: route.frontier_item_id.clone(),
        frontier_item_hash: route.frontier_item_hash.clone(),
        hands_intent_id: intent.intent_id.clone(),
        hands_review_id: review.review_id.clone(),
        substrate_grant_receipt_id: grant_id.clone(),
        requested_paths: requested_paths.clone(),
        granted_at: review.reviewed_at.clone(),
        contract: REPO_FRONTIER_HANDS_AUTHORITY_CONTRACT.to_string(),
    };
    put_repo_frontier_hands_authority(runtime_store, &authority)?;

    Ok(json!({
        "status": "ready",
        "runtimeJobId": runtime_job_id,
        "substrateGateGrantReceiptId": grant_id,
        "intentId": intent.intent_id,
        "reviewId": review.review_id,
        "routeId": route.route_id,
        "modelRevision": route.model_revision,
        "modelHash": route.model_hash,
        "frontierItemId": route.frontier_item_id,
        "requestedPaths": requested_paths,
        "requiredReceipts": review.required_receipts,
        "plannedAction": route.adopted_plan.as_ref().map(|plan| plan.effective_action()),
        "plannedCommand": route.adopted_plan.as_ref().map(|plan| plan.effective_command()),
        "originalPlannedAction": route.adopted_plan.as_ref().map(|plan| plan.action.as_str()),
        "originalPlannedCommand": route.adopted_plan.as_ref().map(|plan| plan.command.as_str()),
        "executionAmendmentId": route.adopted_plan.as_ref().and_then(|plan| plan.execution_amendment.as_ref()).map(|amendment| amendment.amendment_id.as_str()),
        "plannedChecks": route.adopted_plan.as_ref().map(|plan| plan.checks.as_slice()),
        "plannedStopConditions": route.adopted_plan.as_ref().map(|plan| plan.stop_conditions.as_slice()),
        "plannedRollbackSteps": route.adopted_plan.as_ref().map(|plan| plan.rollback_steps.as_slice()),
        "plannedCommitMessage": route.adopted_plan.as_ref().map(|plan| plan.commit_message.as_str()),
        "recordPassCommand": hands_record_pass_command(runtime_store, artifact_dir, route.adopted_plan.as_ref()),
        "store": runtime_store,
    }))
}

fn hands_record_pass_command(
    runtime_store: &Path,
    artifact_dir: &Path,
    adopted_plan: Option<&epiphany_core::RepoFrontierAdoptedPlan>,
) -> Value {
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
            adopted_plan
                .map(|plan| plan.effective_command())
                .unwrap_or("<verification command>"),
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

fn launch_role(
    runtime_store: &Path,
    local_verse_store: &Path,
    thread_id: &str,
    role_id: &str,
    expected_revision: Option<i64>,
    max_runtime_seconds: u64,
    proposal_modeling_request_id: Option<&str>,
) -> Result<Value> {
    let started = Instant::now();
    let service = epiphany_core::EpiphanyCoordinatorService::new(runtime_store);
    let state = service
        .state()?
        .ok_or_else(|| anyhow!("cannot launch role without native coordinator state"))?;
    let state_loaded_ms = started.elapsed().as_millis();
    let role = parse_role_id(role_id)?;
    let expected_revision = expected_revision.and_then(|value| u64::try_from(value).ok());
    let focus =
        epiphany_core::role_launch_context_focus(&state, epiphany_core::epiphany_role_label(role));
    let mut context = if role == epiphany_core::EpiphanyRoleResultRoleId::Modeling {
        epiphany_core::render_modeling_launch_dynamic_prompt_context(
            runtime_store,
            local_verse_store,
            &state,
            focus,
        )
    } else {
        epiphany_core::render_launch_dynamic_prompt_context(
            runtime_store,
            local_verse_store,
            &state,
            focus,
        )
    }
    .map_err(anyhow::Error::msg)?;
    let dynamic_context_rendered_ms = started.elapsed().as_millis();
    if role == epiphany_core::EpiphanyRoleResultRoleId::Verification {
        context = epiphany_core::append_verification_hands_receipt_context(
            context,
            runtime_store,
            &state,
        )
        .map_err(anyhow::Error::msg)?;
    } else if role == epiphany_core::EpiphanyRoleResultRoleId::Modeling {
        context = epiphany_core::append_modeling_work_loop_telemetry_context(
            context,
            runtime_store,
            &state,
            proposal_modeling_request_id,
        )
        .map_err(anyhow::Error::msg)?;
    }
    let role_context_augmented_ms = started.elapsed().as_millis();
    let mut request = epiphany_core::build_epiphany_role_launch_request_with_dynamic_context(
        thread_id,
        role,
        expected_revision,
        Some(max_runtime_seconds),
        &state,
        Some(context),
    )
    .map_err(anyhow::Error::msg)?;
    request.proposal_modeling_request_id = proposal_modeling_request_id.map(str::to_string);
    let request_built_ms = started.elapsed().as_millis();
    let launched = service.launch_job(
        thread_id,
        &state,
        &request,
        format!("epiphany-heartbeat-launch-{}", Uuid::new_v4()),
        Uuid::new_v4().to_string(),
        now(),
    )?;
    let job_committed_ms = started.elapsed().as_millis();
    Ok(json!({
        "bindingId": launched.binding_id,
        "launcherJobId": launched.launcher_job_id,
        "backendJobId": launched.backend_job_id,
        "revision": launched.epiphany_state.revision,
        "state": launched.epiphany_state,
        "timingsMs": {
            "stateLoad": state_loaded_ms,
            "dynamicContextRender": dynamic_context_rendered_ms.saturating_sub(state_loaded_ms),
            "roleContextAugment": role_context_augmented_ms.saturating_sub(dynamic_context_rendered_ms),
            "requestBuild": request_built_ms.saturating_sub(role_context_augmented_ms),
            "jobCommit": job_committed_ms.saturating_sub(request_built_ms),
            "total": job_committed_ms,
        },
    }))
}

fn launch_frontier_imagination(
    runtime_store: &Path,
    local_verse_store: &Path,
    thread_id: &str,
    expected_revision: Option<i64>,
    max_runtime_seconds: u64,
    planning_request_id: &str,
) -> Result<Value> {
    let service = epiphany_core::EpiphanyCoordinatorService::new(runtime_store);
    let state = service
        .state()?
        .ok_or_else(|| anyhow!("cannot launch Imagination without native coordinator state"))?;
    let role = epiphany_core::EpiphanyRoleResultRoleId::Imagination;
    let context = epiphany_core::render_launch_dynamic_prompt_context(
        runtime_store,
        local_verse_store,
        &state,
        epiphany_core::role_launch_context_focus(&state, epiphany_core::epiphany_role_label(role)),
    )
    .map_err(anyhow::Error::msg)?;
    let mut request = epiphany_core::build_epiphany_role_launch_request_with_dynamic_context(
        thread_id,
        role,
        expected_revision.and_then(|value| u64::try_from(value).ok()),
        Some(max_runtime_seconds),
        &state,
        Some(context),
    )
    .map_err(anyhow::Error::msg)?;
    request.frontier_planning_request_id = Some(planning_request_id.to_string());
    launch_job_value(&service, thread_id, &state, &request)
}

fn launch_frontier_mind(
    runtime_store: &Path,
    thread_id: &str,
    expected_revision: Option<i64>,
    max_runtime_seconds: u64,
    mind_request_id: &str,
) -> Result<Value> {
    let service = epiphany_core::EpiphanyCoordinatorService::new(runtime_store);
    let state = service
        .state()?
        .ok_or_else(|| anyhow!("cannot launch Mind without native coordinator state"))?;
    let request = epiphany_core::build_epiphany_frontier_plan_mind_launch_request(
        thread_id,
        expected_revision.and_then(|value| u64::try_from(value).ok()),
        Some(max_runtime_seconds),
        &state,
        mind_request_id.to_string(),
    )
    .map_err(anyhow::Error::msg)?;
    launch_job_value(&service, thread_id, &state, &request)
}

fn launch_job_value(
    service: &epiphany_core::EpiphanyCoordinatorService,
    thread_id: &str,
    state: &epiphany_state_model::EpiphanyThreadState,
    request: &epiphany_core::EpiphanyJobLaunchRequest,
) -> Result<Value> {
    let launched = service.launch_job(
        thread_id,
        state,
        request,
        format!("epiphany-heartbeat-launch-{}", Uuid::new_v4()),
        Uuid::new_v4().to_string(),
        now(),
    )?;
    Ok(json!({
        "bindingId": launched.binding_id,
        "launcherJobId": launched.launcher_job_id,
        "backendJobId": launched.backend_job_id,
        "revision": launched.epiphany_state.revision,
        "state": launched.epiphany_state,
    }))
}

fn accept_role(
    runtime_store: &Path,
    thread_id: &str,
    role_id: &str,
    expected_revision: Option<u64>,
) -> Result<Value> {
    let service = epiphany_core::EpiphanyCoordinatorService::new(runtime_store);
    let state = service
        .state()?
        .ok_or_else(|| anyhow!("cannot accept role without native coordinator state"))?;
    let role = parse_role_id(role_id)?;
    let accepted = service.accept_role(
        thread_id,
        &state,
        role,
        &default_binding_id_for_role(role_id),
        expected_revision,
        None,
        now(),
        &Uuid::new_v4().to_string(),
    )?;
    Ok(json!({
        "roleId": role,
        "revision": accepted.state.revision,
        "state": accepted.state,
        "acceptedReceiptId": accepted.update.accepted_receipt_id,
        "acceptedObservationId": accepted.update.accepted_observation_id,
        "acceptedEvidenceId": accepted.update.accepted_evidence_id,
        "appliedPatch": accepted.update.applied_patch,
        "finding": accepted.finding,
    }))
}

fn accept_reorient(
    runtime_store: &Path,
    thread_id: &str,
    expected_revision: Option<u64>,
) -> Result<Value> {
    let service = epiphany_core::EpiphanyCoordinatorService::new(runtime_store);
    let state = service
        .state()?
        .ok_or_else(|| anyhow!("cannot accept reorientation without native coordinator state"))?;
    let snapshot = service.reorient_result(epiphany_core::EPIPHANY_REORIENT_LAUNCH_BINDING_ID)?;
    if let Some(result_id) = snapshot
        .finding
        .as_ref()
        .and_then(|finding| finding.runtime_result_id.as_deref())
        && let Some(existing) = state.acceptance_receipts.iter().find(|receipt| {
            receipt.result_id == result_id
                && receipt.binding_id == epiphany_core::EPIPHANY_REORIENT_LAUNCH_BINDING_ID
                && receipt.surface == "reorientAccept"
                && receipt.status == "accepted"
        })
    {
        return Ok(json!({
            "revision": state.revision,
            "state": state,
            "acceptedReceiptId": existing.id,
            "changed": false,
        }));
    }
    let accepted = service.accept_reorient(
        thread_id,
        &state,
        epiphany_core::EPIPHANY_REORIENT_LAUNCH_BINDING_ID,
        expected_revision,
        None,
        now(),
        &Uuid::new_v4().to_string(),
        true,
        true,
    )?;
    Ok(json!({
        "revision": accepted.state.revision,
        "state": accepted.state,
        "acceptedReceiptId": accepted.update.accepted_receipt_id,
        "acceptedObservationId": accepted.update.accepted_observation_id,
        "acceptedEvidenceId": accepted.update.accepted_evidence_id,
        "finding": accepted.finding,
    }))
}

fn parse_role_id(role_id: &str) -> Result<epiphany_core::EpiphanyRoleResultRoleId> {
    match role_id {
        "imagination" => Ok(epiphany_core::EpiphanyRoleResultRoleId::Imagination),
        "research" => Ok(epiphany_core::EpiphanyRoleResultRoleId::Research),
        "modeling" => Ok(epiphany_core::EpiphanyRoleResultRoleId::Modeling),
        "verification" => Ok(epiphany_core::EpiphanyRoleResultRoleId::Verification),
        _ => Err(anyhow!("unsupported coordinator role {role_id:?}")),
    }
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
        "research" | "imagination" => finding["statePatch"].is_object(),
        "modeling" => finding["repoModelPatch"].is_object(),
        _ => false,
    }
}

fn reorient_result_auto_acceptable(result: &Value) -> bool {
    if result["status"].as_str() != Some("completed") {
        return false;
    }
    let finding = &result["finding"];
    finding["runtimeResultId"]
        .as_str()
        .is_some_and(|id| !id.trim().is_empty())
        && finding["runtimeJobId"]
            .as_str()
            .is_some_and(|id| !id.trim().is_empty())
        && finding["summary"]
            .as_str()
            .is_some_and(|summary| !summary.trim().is_empty())
        && finding["checkpointStillValid"].is_boolean()
        && !finding["jobError"].is_string()
        && !finding["itemError"].is_string()
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
    runtime_store: &Path,
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
    let service = epiphany_core::EpiphanyCoordinatorService::new(runtime_store);
    if let Some(state) = service.state()?
        && let Some(existing) = state
            .acceptance_receipts
            .iter()
            .find(|receipt| receipt.result_id == result_id)
    {
        if existing.job_id == job_id
            && existing.binding_id == binding_id
            && existing.surface == "roleFailureReview"
            && existing.role_id == role_id
            && existing.status == "superseded"
        {
            return Ok(json!({
                "revision": state.revision,
                "changedFields": [],
                "state": state,
                "receipt": existing,
                "changed": false,
            }));
        }
        return Err(anyhow!(
            "runtime result already has conflicting acceptance authority: surface={:?} role={:?} status={:?} binding={:?}",
            existing.surface,
            existing.role_id,
            existing.status,
            existing.binding_id
        ));
    }
    let receipt = epiphany_state_model::EpiphanyAcceptanceReceipt {
        id: format!("role-failure-review-{}", Uuid::new_v4()),
        result_id,
        job_id,
        binding_id,
        surface: "roleFailureReview".to_string(),
        role_id: role_id.to_string(),
        status: "superseded".to_string(),
        accepted_at: now(),
        accepted_observation_id: None,
        accepted_evidence_id: None,
        summary: Some(summary),
    };
    let applied = service.apply_state_update(
        thread_id,
        epiphany_core::EpiphanyStateUpdate {
            expected_revision: expected_revision.and_then(|value| u64::try_from(value).ok()),
            acceptance_receipts: vec![receipt.clone()],
            ..Default::default()
        },
        None,
    )?;
    Ok(json!({
        "revision": applied.revision,
        "changedFields": applied.changed_fields.iter().map(|field| format!("{field:?}")).collect::<Vec<_>>(),
        "state": applied.state,
        "receipt": receipt,
    }))
}

fn supersede_frontier_planning_failure(
    runtime_store: &Path,
    thread_id: &str,
    expected_revision: Option<i64>,
    result_id: &str,
    job_id: &str,
    role_id: &str,
    binding_id: &str,
) -> Result<Value> {
    let service = epiphany_core::EpiphanyCoordinatorService::new(runtime_store);
    let state = service
        .state()?
        .ok_or_else(|| anyhow!("planning failure review requires coordinator state"))?;
    if let Some(existing) = state
        .acceptance_receipts
        .iter()
        .find(|receipt| receipt.result_id == result_id)
    {
        if existing.job_id == job_id
            && existing.binding_id == binding_id
            && existing.surface == "roleFailureReview"
            && existing.role_id == role_id
            && existing.status == "superseded"
        {
            return Ok(json!({"revision": state.revision, "receipt": existing, "changed": false}));
        }
        return Err(anyhow!(
            "planning failure already has conflicting review authority"
        ));
    }
    let receipt = epiphany_state_model::EpiphanyAcceptanceReceipt {
        id: format!("frontier-planning-failure-review-{}", Uuid::new_v4()),
        result_id: result_id.to_string(),
        job_id: job_id.to_string(),
        binding_id: binding_id.to_string(),
        surface: "roleFailureReview".to_string(),
        role_id: role_id.to_string(),
        status: "superseded".to_string(),
        accepted_at: now(),
        accepted_observation_id: None,
        accepted_evidence_id: None,
        summary: Some(
            "Reviewed non-executable frontier Imagination failure; Self may schedule one next immutable attempt."
                .to_string(),
        ),
    };
    let applied = service.apply_state_update(
        thread_id,
        epiphany_core::EpiphanyStateUpdate {
            expected_revision: expected_revision.and_then(|value| u64::try_from(value).ok()),
            acceptance_receipts: vec![receipt.clone()],
            ..Default::default()
        },
        None,
    )?;
    Ok(json!({"revision": applied.revision, "receipt": receipt, "changed": true}))
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
    runtime_store: &Path,
    local_verse_store: &Path,
    thread_id: &str,
    expected_revision: Option<i64>,
    max_runtime_seconds: u64,
) -> Result<Value> {
    let service = epiphany_core::EpiphanyCoordinatorService::new(runtime_store);
    let state = service
        .state()?
        .ok_or_else(|| anyhow!("cannot launch reorientation without native coordinator state"))?;
    let checkpoint = match state.investigation_checkpoint.clone() {
        Some(checkpoint) => checkpoint,
        None => {
            let projection = epiphany_core::runtime_modeling_semantic_projection_input(
                runtime_store,
            )
            .context("cannot launch reorientation without a valid admitted RepoModel checkpoint")?;
            epiphany_state_model::reorient_checkpoint_from_admitted_repo_model(
                projection.snapshot(),
                &projection.obligation().obligation_id,
            )
        }
    };
    let status = collect_coordinator_status(runtime_store, thread_id)?;
    let decision: epiphany_core::EpiphanyReorientDecision =
        serde_json::from_value(status["reorient"]["decision"].clone())
            .context("native reorientation status did not contain a typed decision")?;
    let context = epiphany_core::render_launch_dynamic_prompt_context(
        runtime_store,
        local_verse_store,
        &state,
        epiphany_core::reorient_launch_context_focus(&state, &decision.next_action),
    )
    .map_err(anyhow::Error::msg)?;
    let expected_revision = expected_revision.and_then(|value| u64::try_from(value).ok());
    let request = epiphany_core::build_epiphany_reorient_launch_request_with_dynamic_context(
        thread_id,
        expected_revision,
        Some(max_runtime_seconds),
        &state,
        &checkpoint,
        &decision,
        Some(context),
    );
    let launched = service.launch_job(
        thread_id,
        &state,
        &request,
        format!("epiphany-heartbeat-launch-{}", Uuid::new_v4()),
        Uuid::new_v4().to_string(),
        now(),
    )?;
    Ok(json!({
        "bindingId": launched.binding_id,
        "launcherJobId": launched.launcher_job_id,
        "backendJobId": launched.backend_job_id,
        "revision": launched.epiphany_state.revision,
        "state": launched.epiphany_state,
        "decision": decision,
    }))
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
    mcp_config: &Path,
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
        .arg("--mcp-config")
        .arg(mcp_config)
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

fn read_role_result(runtime_store: &Path, thread_id: &str, role_id: &str) -> Result<Value> {
    Ok(collect_coordinator_status(runtime_store, thread_id)?["roleResults"][role_id].clone())
}

fn read_reorient_result(runtime_store: &Path, thread_id: &str) -> Result<Value> {
    Ok(collect_coordinator_status(runtime_store, thread_id)?["reorientResult"].clone())
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

fn apply_manual_regather_approval(action: &str, coordinator: &mut Value) -> Result<String> {
    if action != "regatherManually" {
        return Err(anyhow!(
            "--approve-manual-regather requires Self to derive regatherManually, got {action}"
        ));
    }
    coordinator["action"] = Value::String("launchResearch".to_string());
    coordinator["canAutoRun"] = Value::Bool(true);
    coordinator["requiresReview"] = Value::Bool(false);
    coordinator["recommendedSceneAction"] = Value::String("roleLaunch".to_string());
    coordinator["reason"] = Value::String(
        "Operator approved the Self-derived manual regather boundary; launch Eyes once through the typed Research lane."
            .to_string(),
    );
    Ok("launchResearch".to_string())
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

fn resolve_proposal_modeling_request_hint<'a>(
    explicit: Option<&'a str>,
    derived: Option<&'a str>,
) -> Result<Option<&'a str>> {
    match (explicit, derived) {
        (Some(explicit), Some(derived)) if explicit == derived => Ok(Some(derived)),
        (None, derived) => Ok(derived),
        (Some(_), Some(_)) => Err(anyhow!(
            "explicit proposal Modeling request does not match Self-derived pending authority"
        )),
        (Some(_), None) => Err(anyhow!(
            "explicit proposal Modeling request has no Self-derived pending authority"
        )),
    }
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

fn coordinator_runtime_identity_options(
    runtime_id: &str,
    created_at: String,
) -> RuntimeSpineInitOptions {
    RuntimeSpineInitOptions {
        runtime_id: runtime_id.to_string(),
        display_name: format!("Epiphany {runtime_id}"),
        created_at,
    }
}

fn reset_artifact_dir(path: &Path) -> Result<()> {
    let cwd = env::current_dir()?;
    let mut roots = Vec::new();
    for name in [".epiphany-dogfood", ".epiphany-smoke"] {
        let root = cwd.join(name);
        fs::create_dir_all(&root)?;
        roots.push(root.canonicalize()?);
    }
    let resolved_parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(resolved_parent)?;
    let resolved = if path.exists() {
        path.canonicalize()?
    } else {
        resolved_parent
            .canonicalize()?
            .join(path.file_name().unwrap())
    };
    if !roots
        .iter()
        .any(|root| resolved != *root && resolved.starts_with(root))
    {
        return Err(anyhow!(
            "refusing to delete artifact dir outside .epiphany-dogfood or .epiphany-smoke: {}",
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
        .pointer("/scene/scene/revision")
        .and_then(Value::as_i64)
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
            | "reviewFrontierPlanningFailure"
            | "waitForImaginationResult"
            | "waitForMindPlanResult"
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
    fn proposal_cli_argument_is_only_an_assertion_over_self_derived_authority() {
        let request = "repo-frontier-proposal-modeling-request";
        assert_eq!(
            resolve_proposal_modeling_request_hint(None, Some(request)).unwrap(),
            Some(request)
        );
        assert_eq!(
            resolve_proposal_modeling_request_hint(Some(request), Some(request)).unwrap(),
            Some(request)
        );
        assert!(resolve_proposal_modeling_request_hint(Some(request), None).is_err());
        assert!(
            resolve_proposal_modeling_request_hint(Some("wrong-request"), Some(request)).is_err()
        );
    }

    #[test]
    fn coordinator_preserves_the_authenticated_runtime_identity() {
        let options = coordinator_runtime_identity_options(
            "epiphany-starfire",
            "2026-08-06T00:00:00Z".into(),
        );
        assert_eq!(options.runtime_id, "epiphany-starfire");
        assert_eq!(options.display_name, "Epiphany epiphany-starfire");
    }

    #[test]
    fn coordinator_binary_has_no_codex_host_or_epiphany_json_rpc_dependency() {
        let source = include_str!("epiphany-mvp-coordinator.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        for forbidden in ["AppServerClient", "thread/epiphany/", "codex-app-server"] {
            assert!(
                !production.contains(forbidden),
                "native coordinator regrew host dependency {forbidden:?}"
            );
        }
        let compatibility_field = ["epiphany", "State"].concat();
        assert!(!production.contains(&compatibility_field));
        assert!(!production.contains("--thread-state-store"));
    }

    #[test]
    fn coordinator_status_and_result_reads_are_native() {
        let source = include_str!("epiphany-mvp-coordinator.rs");
        let status_start = source.find("fn collect_coordinator_status").unwrap();
        let status_end = source.find("fn resolve_model_runtime_bin").unwrap();
        let status = &source[status_start..status_end];
        assert!(status.contains("native_coordinator_json"));
        assert!(!status.contains("native_json("));
        assert!(!status.contains("Command::new"));
        assert!(!status.contains("--native"));
        assert!(!status.contains("AppServerClient"));
        assert!(!status.contains("thread/epiphany/"));

        let reads_start = source.find("fn read_role_result").unwrap();
        let reads_end = source.find("fn maybe_apply_role_self_patch").unwrap();
        let reads = &source[reads_start..reads_end];
        assert!(reads.contains("collect_coordinator_status"));
        assert!(!reads.contains("AppServerClient"));
        assert!(!reads.contains("thread/epiphany/"));
    }

    #[test]
    fn coordinator_role_launch_and_acceptance_are_native() {
        let source = include_str!("epiphany-mvp-coordinator.rs");
        let start = source.find("fn launch_role").unwrap();
        let end = source.find("fn role_id_for_coordinator_action").unwrap();
        let native_roles = &source[start..end];
        for required in [
            "render_launch_dynamic_prompt_context",
            "render_modeling_launch_dynamic_prompt_context",
            "append_verification_hands_receipt_context",
            "append_modeling_work_loop_telemetry_context",
            ".launch_job(",
            ".accept_role(",
        ] {
            assert!(
                native_roles.contains(required),
                "missing native role seam {required:?}"
            );
        }
        assert!(!native_roles.contains("append_modeling_repo_model_shape_context("));
        assert!(!native_roles.contains("AppServerClient"));
        assert!(!native_roles.contains("thread/epiphany/"));
    }

    #[test]
    fn continue_implementation_is_not_a_passive_stop_action() {
        assert!(!is_stop_action("continueImplementation"));
    }

    #[test]
    fn operator_objective_intake_creates_once_and_refuses_replacement() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime.cc");

        let first = intake_operator_objective(&store, "thread-1", "Map the machine")?;
        assert!(first.changed);
        assert_eq!(first.state.revision, 1);

        let repeated = intake_operator_objective(&store, "thread-1", " Map the machine ")?;
        assert!(!repeated.changed);
        assert_eq!(repeated.state.revision, first.state.revision);

        let error = intake_operator_objective(&store, "thread-1", "Replace the machine")
            .expect_err("objective replacement must require a typed adoption flow");
        assert!(error.to_string().contains("refusing to replace"));
        Ok(())
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
    fn auto_review_accepts_modeling_repo_patch_without_generic_state_patch() {
        let result = json!({
            "status": "completed",
            "finding": {
                "runtimeResultId": "result-modeling-typed",
                "runtimeJobId": "job-modeling-typed",
                "repoModelPatch": {
                    "patch_id": "patch-typed",
                    "base_revision": 0,
                    "base_hash": "base",
                    "applied_at": "2026-07-16T00:00:00Z",
                    "purpose": {"kind": "evolution"},
                    "operations": [{"operation": "retire_node", "node_id": "old"}]
                },
                "statePatch": null
            }
        });

        assert!(role_result_auto_acceptable("modeling", &result));
        assert!(!role_result_needs_supersession("modeling", &result));
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
                "repoModelPatch": {
                    "patch_id": "patch-modeling-2",
                    "base_revision": 0,
                    "base_hash": "base",
                    "applied_at": "2026-07-16T00:00:00Z",
                    "purpose": {"kind": "evolution"},
                    "operations": [{"operation": "retire_node", "node_id": "old"}]
                },
                "statePatch": {"scratch": {"summary": "mapped"}}
            }
        });
        let null_patch = json!({
            "status": "completed",
            "finding": {
                "summary": "Explicit null is still no state patch.",
                "runtimeResultId": "result-modeling-null",
                "runtimeJobId": "job-modeling-null",
                "statePatch": null
            }
        });

        assert!(role_result_needs_supersession("modeling", &unreviewable));
        assert!(role_result_needs_supersession("modeling", &null_patch));
        assert!(!role_result_needs_supersession("modeling", &reviewable));
        assert!(!role_result_needs_supersession(
            "verification",
            &unreviewable
        ));
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
        seed_hands_frontier(&store)?;
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
            Some(intent.frontier_route_id.as_str()),
            gate["routeId"].as_str()
        );
        assert_eq!(intent.plan_candidate_sha256, "coordinator-plan-hash-test");
        assert_eq!(
            intent.plan_action,
            "Implement the bounded coordinator gate test."
        );
        assert_eq!(
            gate.pointer("/recordPassCommand/executable")
                .and_then(serde_json::Value::as_str),
            Some("epiphany-hands-action")
        );
        assert_eq!(
            gate["plannedCommand"].as_str(),
            Some("cargo test --bin epiphany-mvp-coordinator")
        );
        assert_eq!(
            gate.pointer("/recordPassCommand/args/10")
                .and_then(serde_json::Value::as_str),
            Some("cargo test --bin epiphany-mvp-coordinator"),
            "the operator command surface must carry the exact Mind-admitted command"
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

    #[test]
    fn operator_can_approve_only_a_self_derived_manual_regather() -> Result<()> {
        let mut coordinator = json!({
            "action": "regatherManually",
            "canAutoRun": false,
            "requiresReview": true,
            "recommendedSceneAction": "roleResult",
        });
        assert_eq!(
            apply_manual_regather_approval("regatherManually", &mut coordinator)?,
            "launchResearch"
        );
        assert_eq!(coordinator["action"], "launchResearch");
        assert_eq!(coordinator["canAutoRun"], true);
        assert_eq!(coordinator["requiresReview"], false);
        assert_eq!(coordinator["recommendedSceneAction"], "roleLaunch");

        let mut unrelated = json!({"action": "launchModeling"});
        let error = apply_manual_regather_approval("launchModeling", &mut unrelated)
            .expect_err("operator approval must not override another Self decision");
        assert!(
            error
                .to_string()
                .contains("requires Self to derive regatherManually")
        );
        assert_eq!(unrelated["action"], "launchModeling");
        Ok(())
    }

    fn seed_hands_frontier(store: &Path) -> Result<()> {
        let mut item = epiphany_core::RepoFrontierItem {
            id: "coordinator-hands-frontier-test".to_string(),
            migration_body: "Exercise the coordinator-owned Hands gate.".to_string(),
            question: "Does Self route the admitted implementation frontier?".to_string(),
            gap: "Hands cannot act without a routed Modeling frontier.".to_string(),
            target_claim_ids: vec!["coordinator-hands-claim-test".to_string()],
            source_scope: vec!["epiphany-core/src".to_string()],
            recommended_next_organ: "Hands".to_string(),
            evidence_refs: vec!["fixture:mvp-coordinator".to_string()],
            status: epiphany_core::RepoFrontierStatus::Active,
            ..Default::default()
        };
        item.adopted_plan = Some(epiphany_core::RepoFrontierAdoptedPlan {
            planning_request_id: "coordinator-planning-request-test".to_string(),
            result_id: "coordinator-imagination-result-test".to_string(),
            job_id: "coordinator-imagination-job-test".to_string(),
            candidate_id: "coordinator-plan-candidate-test".to_string(),
            candidate_sha256: "coordinator-plan-hash-test".to_string(),
            safe_paths: item.source_scope.clone(),
            action: "Implement the bounded coordinator gate test.".to_string(),
            command: "cargo test --bin epiphany-mvp-coordinator".to_string(),
            checks: vec!["focused coordinator test passes".to_string()],
            stop_conditions: vec!["authority scope changes".to_string()],
            rollback_steps: vec!["revert the bounded change".to_string()],
            commit_message: "Bind Hands intent to adopted plan".to_string(),
            execution_amendment: None,
        });
        let mut model = epiphany_core::EpiphanyMemoryGraphSnapshot {
            schema_version: Some(epiphany_core::MEMORY_GRAPH_SCHEMA_VERSION.to_string()),
            graph_id: "mvp-coordinator-test-model".to_string(),
            model_revision: 1,
            domains: vec![epiphany_core::EpiphanyMemoryDomain {
                id: "repo".to_string(),
                profile: epiphany_core::EpiphanyMemoryProfile::RepoArchitecture,
                title: "Repository".to_string(),
                lifecycle: epiphany_core::EpiphanyMemoryLifecycle::Accepted,
                ..Default::default()
            }],
            nodes: vec![epiphany_core::EpiphanyMemoryNode {
                id: "coordinator-hands-claim-test".to_string(),
                domain_id: "repo".to_string(),
                profile: epiphany_core::EpiphanyMemoryProfile::RepoArchitecture,
                kind: epiphany_core::EpiphanyMemoryNodeKind::RuntimeContract,
                title: "Coordinator Hands gate".to_string(),
                claim: "Implementation begins from an admitted routed frontier.".to_string(),
                question: "Is the route current?".to_string(),
                action_implication: "Route the exact admitted source scope.".to_string(),
                source_hashes: vec!["anchor:missing".to_string()],
                lifecycle: epiphany_core::EpiphanyMemoryLifecycle::Accepted,
                ..Default::default()
            }],
            frontier: vec![item],
            ..Default::default()
        };
        model.model_hash = epiphany_core::memory_graph_model_hash(&model)?;
        let entry = epiphany_core::EpiphanyMemoryGraphEntry::from_snapshot(&model)?;
        let admission = epiphany_core::RepoModelAdmissionReceipt {
            schema_version: epiphany_core::REPO_MODEL_ADMISSION_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "coordinator-hands-admission-test".to_string(),
            review_id: "coordinator-hands-model-review-test".to_string(),
            result_id: Some("coordinator-hands-model-result-test".to_string()),
            patch_id: "coordinator-hands-model-patch-test".to_string(),
            patch_sha256: "fixture".to_string(),
            previous_revision: 0,
            previous_hash: String::new(),
            admitted_revision: model.model_revision,
            admitted_hash: model.model_hash.clone(),
            admitted_at: "2026-06-02T00:00:00Z".to_string(),
            contract: epiphany_core::REPO_MODEL_ADMISSION_CONTRACT.to_string(),
            purpose: epiphany_core::RepoModelPatchPurpose::Evolution,
            frontier_route_id: String::new(),
            verification_request_id: String::new(),
            soul_verdict_receipt_id: String::new(),
            frontier_modeling_request_id: String::new(),
            proposal_modeling_request_id: String::new(),
            claim_repair_request_id: String::new(),
            frontier_plan_decision_id: String::new(),
            repository_body_observation_basis: None,
            admission_source: Some(epiphany_core::RepoModelAdmissionSource::WorkerResult {
                result_id: "coordinator-hands-model-result-test".into(),
                job_id: "coordinator-hands-model-job-test".into(),
            }),
        };
        let mut cache = epiphany_core::runtime_spine_cache(store)?;
        cache.put(epiphany_core::MEMORY_GRAPH_KEY, &entry)?;
        cache.put(&admission.receipt_id, &admission)?;
        Ok(())
    }
}
