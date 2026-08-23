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
use epiphany_core::RuntimeSpineInitOptions;
use epiphany_core::finalize_coordinator_run;
use epiphany_core::hands_action_review_for_intent;
use epiphany_core::initialize_runtime_spine;
use epiphany_core::load_epiphany_cultmesh_swarm_brake;
use epiphany_core::open_coordinator_run;
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
use epiphany_core::coordinator_status as status_projection;
use serde_json::Value;
use serde_json::json;
use sha2::{Digest, Sha256};
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
    model: String,
    provider_credential_path: Option<PathBuf>,
    thread_id: Option<String>,
    objective: Option<String>,
    cwd: PathBuf,
    codex_home: PathBuf,
    mcp_config: PathBuf,
    artifact_dir: PathBuf,
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
    required_action: Option<String>,
}

impl Args {
    fn parse() -> Result<Self> {
        let root = env::current_dir().context("failed to resolve current dir")?;
        let mut args = env::args().skip(1);
        let mut parsed = Args {
            model_runtime_bin: PathBuf::from(DEFAULT_MODEL_RUNTIME_BIN),
            tool_adapter_bin: PathBuf::from(DEFAULT_TOOL_ADAPTER_BIN),
            model_provider: "openai-codex".to_string(),
            model: "gpt-5.4".to_string(),
            provider_credential_path: None,
            thread_id: None,
            objective: None,
            cwd: root.clone(),
            codex_home: env::var_os("CODEX_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| home_dir().join(".codex")),
            mcp_config: root.join(".epiphany").join("mcp.toml"),
            artifact_dir: root.join(".epiphany-dogfood").join("coordinator"),
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
            required_action: None,
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
                "--model" => parsed.model = take_string(&mut args, "--model")?,
                "--provider-credential" => {
                    parsed.provider_credential_path =
                        Some(take_path(&mut args, "--provider-credential")?)
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
                "--required-action" => {
                    parsed.required_action = Some(take_string(&mut args, "--required-action")?)
                }
                "--cwd" => parsed.cwd = take_path(&mut args, "--cwd")?,
                "--codex-home" => parsed.codex_home = take_path(&mut args, "--codex-home")?,
                "--mcp-config" => parsed.mcp_config = take_path(&mut args, "--mcp-config")?,
                "--artifact-dir" => parsed.artifact_dir = take_path(&mut args, "--artifact-dir")?,
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
        if parsed.required_action.is_some() && (parsed.mode != "execute" || parsed.max_steps != 1) {
            return Err(anyhow!(
                "--required-action requires execute mode with exactly one step"
            ));
        }
        Ok(parsed)
    }
}

fn run_coordinator(args: &Args) -> Result<Value> {
    let root = env::current_dir().context("failed to resolve current dir")?;
    let resident_claim = match (&args.resident_state_store, &args.resident_preparation_id) {
        (Some(store), Some(preparation_id)) => {
            let identity = capture_process_instance(std::process::id())?;
            Some(claim_resident_self_preparation_as_child(
                store,
                preparation_id,
                &LaunchedCoordinator {
                    process_id: identity.process_id,
                    process_creation_token: identity.creation_token,
                    process_executable_path: identity.executable_path,
                },
                chrono::Utc::now().timestamp_millis().max(0) as u64,
            )?)
        }
        (None, None) => None,
        _ => {
            return Err(anyhow!(
                "resident coordinator bootstrap requires both state store and preparation id"
            ));
        }
    };
    let local_verse_store = status_projection::absolute_path(&args.local_verse_store)?;
    assert_local_verse_brake_released(
        &local_verse_store,
        args.runtime_id.as_deref().expect("validated runtime id"),
        "epiphany-mvp-coordinator",
    )?;
    let cwd = status_projection::absolute_path(&args.cwd)?;
    let model_runtime_bin = resolve_model_runtime_bin(&root, &args.model_runtime_bin)?;
    let tool_adapter_bin = resolve_model_runtime_bin(&root, &args.tool_adapter_bin)?;
    let codex_home = status_projection::absolute_path(&args.codex_home)?;
    let mcp_config = status_projection::absolute_path(&args.mcp_config)?;
    let artifact_dir = status_projection::absolute_path(&args.artifact_dir)?;
    let runtime_store = status_projection::absolute_path(&args.runtime_store)?;
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
    let resident_launch_digest = args
        .resident_binding
        .get("launch-digest")
        .map(String::as_str);
    let runtime_session_id =
        epiphany_core::coordinator_run_session_id(&thread_id, resident_launch_digest)?;
    let runtime_session_objective = args
        .resident_binding
        .get("objective-digest")
        .map(|digest| format!("Resident coordinator launch {digest}."))
        .unwrap_or_else(|| {
            "Coordinate the Epiphany MVP lanes with native runtime job receipts.".to_string()
        });
    let runtime_identity = initialize_runtime_spine(
        &runtime_store,
        coordinator_runtime_identity_options(
            args.runtime_id.as_deref().expect("validated runtime id"),
            now(),
        ),
    )?;
    let runtime_session = open_coordinator_run(
        &runtime_store,
        &runtime_session_id,
        &thread_id,
        resident_launch_digest,
        &runtime_session_objective,
        &now(),
    )?;
    startup_events.push(json!({
        "type": "runtimeSpineSession",
        "store": runtime_store,
        "runtimeId": runtime_identity.runtime_id,
        "sessionId": runtime_session.session_id,
    }));
    let run_result = (|| -> Result<Value> {
        let mut startup_proposal_modeling_job_id = None;
        if let Some(objective) = args.objective.as_deref() {
            if let (Some(store), Some(preparation_id), Some(claim)) = (
                args.resident_state_store.as_deref(),
                args.resident_preparation_id.as_deref(),
                resident_claim.as_ref(),
            ) {
                epiphany_core::validate_resident_self_prepared_objective(
                    store,
                    preparation_id,
                    claim,
                    objective,
                )?;
                let source_ref = format!("resident-self-grant://{}", claim.grant_id);
                let intake =
                    intake_operator_objective(&runtime_store, &thread_id, objective, &source_ref)?;
                startup_events.push(json!({
                    "type": "residentObjectiveBinding",
                    "threadId": thread_id,
                    "grantId": claim.grant_id,
                    "preparationId": preparation_id,
                    "mindProjectionDigest": intake.mind.projection_digest,
                    "commitReceiptId": intake.commit_receipt.receipt_id,
                    "canonicalObjectiveChanged": intake.changed,
                }));
            } else {
                let intake = intake_operator_objective(
                    &runtime_store,
                    &thread_id,
                    objective,
                    "cli://epiphany-mvp-coordinator",
                )?;
                startup_events.push(json!({
                    "type": "operatorObjectiveIntake",
                    "threadId": thread_id,
                    "mindProjectionDigest": intake.mind.projection_digest,
                    "commitReceiptId": intake.commit_receipt.receipt_id,
                    "changed": intake.changed,
                }));
            }
        }
        if let Some(request_id) = args.imagination_consideration_request_id.as_deref() {
            if args.objective.is_some() {
                return Err(anyhow!(
                    "typed consideration cannot share operator objective intake"
                ));
            }
            let worker_job_id = epiphany_core::launch_current_imagination_consideration_work(
                &runtime_store,
                request_id,
                &now(),
            )?;
            let operator_launch = json!({
                "bindingId": epiphany_core::EPIPHANY_IMAGINATION_ROLE_BINDING_ID,
                "backendJobId": worker_job_id.clone(),
            });
            let worker_run = launch_worker_runtime_detached(
                &model_runtime_bin,
                &tool_adapter_bin,
                &args.model_provider,
                &args.model,
                args.provider_credential_path.as_deref(),
                &runtime_store,
                &codex_home,
                &mcp_config,
                &cwd,
                args.resident_state_store.as_deref(),
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
                "launch": status_projection::sanitize_for_operator(operator_launch),
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
            let worker_job_id =
                epiphany_core::launch_current_admitted_model_direction_consideration_work(
                    &runtime_store,
                    request_id,
                    &now(),
                )?;
            let worker_run = launch_worker_runtime_detached(
                &model_runtime_bin,
                &tool_adapter_bin,
                &args.model_provider,
                &args.model,
                args.provider_credential_path.as_deref(),
                &runtime_store,
                &codex_home,
                &mcp_config,
                &cwd,
                args.resident_state_store.as_deref(),
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
            let current = epiphany_core::project_current_work(&runtime_store)?
                .proposal_modeling
                .filter(|work| {
                    work.attempt.action
                        == epiphany_core::EpiphanyAgentPassContinuationAction::Launch
                })
                .ok_or_else(|| anyhow!("proposal Modeling request is not launchable"))?;
            if current.request.request_id != request_id {
                return Err(anyhow!(
                    "explicit proposal Modeling request does not match current keyed work"
                ));
            }
            let binding = epiphany_core::launch_current_proposal_modeling_work(
                &runtime_store,
                epiphany_core::EpiphanyProposalModelingLaunchOptions { created_at: now() },
            )?;
            let worker_job_id = binding.job_id.clone();
            startup_proposal_modeling_job_id = Some(worker_job_id.clone());
            let worker_run = launch_worker_runtime_detached(
                &model_runtime_bin,
                &tool_adapter_bin,
                &args.model_provider,
                &args.model,
                args.provider_credential_path.as_deref(),
                &runtime_store,
                &codex_home,
                &mcp_config,
                &cwd,
                args.resident_state_store.as_deref(),
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
                "launch": binding,
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
            if matches!(
                coordinator["action"].as_str(),
                Some("waitForModelingResult" | "reviewModelingResult")
            ) && let Some(job_id) = startup_proposal_modeling_job_id.as_deref()
            {
                coordinator["runtimeJobId"] = Value::String(job_id.to_string());
            }
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
                status_projection::render_status(&status_projection::sanitize_for_operator(status.clone())),
            )?;
            snapshots.push(snapshot_name);
            let mut step = json!({
                "index": index,
                "action": action,
                "coordinator": coordinator,
                "events": [],
            });
            final_status = status.clone();
            final_action = coordinator.clone();

            if let Some(required_action) = args.required_action.as_deref()
                && action != required_action
            {
                push_event(
                    &mut step,
                    json!({
                        "type": "requiredActionRefusal",
                        "requiredAction": required_action,
                        "derivedAction": action,
                        "consequence": "none"
                    }),
                );
                append_operator_step_jsonl(&steps_path, &step)?;
                steps.push(step);
                break;
            }

            if args.mode == "plan" {
                append_operator_step_jsonl(&steps_path, &step)?;
                steps.push(step);
                break;
            }
            if action == "awaitFrontierProposal"
                || matches!(
                    action.as_str(),
                    "waitForModelingResult"
                        | "waitForImaginationResult"
                        | "waitForMindPlanResult"
                        | "waitForImaginationConsideration"
                        | "waitForAdmittedModelDirectionConsideration"
                )
                || (action == "reviewFrontierPlanningFailure" && !args.supersede_failed_results)
                || (is_stop_action(&action)
                    && !args.auto_review
                    && !is_result_review_action(&action))
            {
                append_operator_step_jsonl(&steps_path, &step)?;
                steps.push(step);
                break;
            }

            match action.as_str() {
                "reviewFrontierPlanningFailure" => {
                    let lifecycle =
                        epiphany_core::runtime_repo_frontier_planning_lifecycle(&runtime_store)?;
                    let job_id = match lifecycle.stage {
                        epiphany_core::RepoFrontierPlanningLifecycleStage::ImaginationFailed => {
                            lifecycle.imagination_job_id.as_deref()
                        }
                        epiphany_core::RepoFrontierPlanningLifecycleStage::MindFailed => {
                            lifecycle.mind_job_id.as_deref()
                        }
                        _ => return Err(anyhow!("frontier planning failure review stage changed")),
                    };
                    let job_id = job_id.ok_or_else(|| {
                        anyhow!("frontier planning failure review requires a launch job")
                    })?;
                    let reviewed = epiphany_core::review_repo_frontier_planning_failure(
                        &runtime_store,
                        job_id,
                        &now(),
                    )?;
                    push_event(
                        &mut step,
                        json!({"type": "frontierPlanningFailureReview", "review": status_projection::sanitize_for_operator(serde_json::to_value(reviewed)?)}),
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
                    let worker_job_id = epiphany_core::launch_current_frontier_planning_work(
                        &runtime_store,
                        &now(),
                    )?;
                    push_event(
                        &mut step,
                        json!({"type": "frontierImaginationLaunch", "runtimeJobId": worker_job_id}),
                    );
                    let worker_run = launch_worker_runtime_detached(
                        &model_runtime_bin,
                        &tool_adapter_bin,
                        &args.model_provider,
                        &args.model,
                        args.provider_credential_path.as_deref(),
                        &runtime_store,
                        &codex_home,
                        &mcp_config,
                        &cwd,
                        args.resident_state_store.as_deref(),
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
                    let result_id =
                        lifecycle.imagination_result_id.as_deref().ok_or_else(|| {
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
                    let worker_job_id = epiphany_core::launch_current_frontier_plan_mind_work(
                        &runtime_store,
                        &now(),
                    )?;
                    push_event(
                        &mut step,
                        json!({"type": "frontierMindLaunch", "runtimeJobId": worker_job_id}),
                    );
                    let worker_run = launch_worker_runtime_detached(
                        &model_runtime_bin,
                        &tool_adapter_bin,
                        &args.model_provider,
                        &args.model,
                        args.provider_credential_path.as_deref(),
                        &runtime_store,
                        &codex_home,
                        &mcp_config,
                        &cwd,
                        args.resident_state_store.as_deref(),
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
                    let decision = epiphany_core::commit_repo_frontier_plan_decision(
                        &runtime_store,
                        result_id,
                    )?;
                    push_event(
                        &mut step,
                        json!({"type": "frontierPlanDecision", "decision": decision}),
                    );
                    final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
                }
                "reviewResearchResult" | "reviewModelingResult" | "reviewVerificationResult" => {
                    let role_id = role_id_for_coordinator_action(&action)
                        .ok_or_else(|| anyhow!("unsupported review action {action}"))?;
                    if role_id == "research"
                        && let Some(job_id) =
                            epiphany_core::current_frontier_research_review_job_id(&runtime_store)?
                    {
                        let result =
                            epiphany_core::runtime_role_worker_result(&runtime_store, &job_id)?
                                .ok_or_else(|| {
                                    anyhow!("frontier Research review lost its typed result")
                                })?;
                        push_event(
                            &mut step,
                            json!({"type": "frontierResearchResult", "roleId": role_id, "result": result}),
                        );
                        if !args.auto_review {
                            final_action = json!({
                                "action": "reviewResearchResult",
                                "reason": "The exact frontier Research result awaits keyed Mind admission.",
                                "runtimeJobId": job_id,
                            });
                            append_operator_step_jsonl(&steps_path, &step)?;
                            steps.push(step);
                            break;
                        }
                        let accepted = epiphany_core::accept_frontier_research_result(
                            &runtime_store,
                            &job_id,
                            &now(),
                        )?;
                        push_event(
                            &mut step,
                            json!({"type": "frontierResearchAccept", "roleId": role_id, "commit": accepted}),
                        );
                        final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
                        append_operator_step_jsonl(&steps_path, &step)?;
                        steps.push(step);
                        continue;
                    }
                    if role_id == "verification"
                        && let Some(job_id) =
                            epiphany_core::current_frontier_verification_review_job_id(
                                &runtime_store,
                            )?
                    {
                        let result =
                            epiphany_core::runtime_role_worker_result(&runtime_store, &job_id)?
                                .ok_or_else(|| {
                                    anyhow!("frontier Verification review lost its typed result")
                                })?;
                        push_event(
                            &mut step,
                            json!({"type": "frontierVerificationResult", "roleId": role_id, "result": result}),
                        );
                        if !args.auto_review {
                            final_action = json!({
                                "action": "reviewVerificationResult",
                                "reason": "The exact frontier Verification result awaits Soul admission.",
                                "runtimeJobId": job_id,
                            });
                            append_operator_step_jsonl(&steps_path, &step)?;
                            steps.push(step);
                            break;
                        }
                        let admission = epiphany_core::accept_frontier_verification_result(
                            &runtime_store,
                            &job_id,
                            &now(),
                        )?;
                        push_event(
                            &mut step,
                            json!({"type": "frontierVerificationAdmission", "roleId": role_id, "outcome": admission}),
                        );
                        final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
                        append_operator_step_jsonl(&steps_path, &step)?;
                        steps.push(step);
                        continue;
                    }
                    if role_id == "modeling"
                        && let Some(job_id) =
                            epiphany_core::current_proposal_modeling_review_job_id(&runtime_store)?
                    {
                        let result =
                            epiphany_core::runtime_role_worker_result(&runtime_store, &job_id)?
                                .ok_or_else(|| {
                                    anyhow!("proposal Modeling review lost its typed result")
                                })?;
                        push_event(
                            &mut step,
                            json!({"type": "proposalModelingResult", "roleId": role_id, "result": result}),
                        );
                        if !args.auto_review {
                            final_action = json!({
                                "action": "reviewModelingResult",
                                "reason": "The exact proposal Modeling result awaits keyed Mind admission.",
                                "runtimeJobId": job_id,
                            });
                            append_operator_step_jsonl(&steps_path, &step)?;
                            steps.push(step);
                            break;
                        }
                        let admission = epiphany_core::accept_proposal_modeling_result(
                            &runtime_store,
                            &job_id,
                            &now(),
                        )?;
                        push_event(
                            &mut step,
                            json!({"type": "proposalModelingAdmission", "roleId": role_id, "outcome": admission}),
                        );
                        final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
                        append_operator_step_jsonl(&steps_path, &step)?;
                        steps.push(step);
                        continue;
                    }
                    if role_id == "modeling"
                        && let Some(job_id) =
                            epiphany_core::current_body_modeling_review_job_id(&runtime_store)?
                    {
                        let result =
                            epiphany_core::runtime_role_worker_result(&runtime_store, &job_id)?
                                .ok_or_else(|| {
                                    anyhow!("Body Modeling review lost its typed result")
                                })?;
                        push_event(
                            &mut step,
                            json!({"type": "bodyModelingResult", "roleId": role_id, "result": result}),
                        );
                        if !args.auto_review {
                            final_action = json!({
                                "action": "reviewModelingResult",
                                "reason": "The exact Body Modeling result awaits keyed Mind admission.",
                                "runtimeJobId": job_id,
                            });
                            append_operator_step_jsonl(&steps_path, &step)?;
                            steps.push(step);
                            break;
                        }
                        let admission = epiphany_core::accept_body_modeling_result(
                            &runtime_store,
                            &job_id,
                            &now(),
                        )?;
                        push_event(
                            &mut step,
                            json!({"type": "bodyModelingAdmission", "roleId": role_id, "outcome": admission}),
                        );
                        final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
                        append_operator_step_jsonl(&steps_path, &step)?;
                        steps.push(step);
                        continue;
                    }
                    if role_id == "modeling"
                        && let Some(job_id) =
                            epiphany_core::current_frontier_verdict_modeling_review_job_id(
                                &runtime_store,
                            )?
                    {
                        let result =
                            epiphany_core::runtime_role_worker_result(&runtime_store, &job_id)?
                                .ok_or_else(|| {
                                    anyhow!(
                                        "frontier verdict Modeling review lost its typed result"
                                    )
                                })?;
                        push_event(
                            &mut step,
                            json!({"type": "frontierVerdictModelingResult", "roleId": role_id, "result": result}),
                        );
                        if !args.auto_review {
                            final_action = json!({
                                "action": "reviewModelingResult",
                                "reason": "The exact frontier verdict Modeling result awaits keyed Mind admission.",
                                "runtimeJobId": job_id,
                            });
                            append_operator_step_jsonl(&steps_path, &step)?;
                            steps.push(step);
                            break;
                        }
                        let admission = epiphany_core::accept_frontier_verdict_modeling_result(
                            &runtime_store,
                            &job_id,
                            &now(),
                        )?;
                        push_event(
                            &mut step,
                            json!({"type": "frontierVerdictModelingAdmission", "roleId": role_id, "outcome": admission}),
                        );
                        final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
                        append_operator_step_jsonl(&steps_path, &step)?;
                        steps.push(step);
                        continue;
                    }
                    return Err(anyhow!(
                        "{role_id} review has no keyed current-work owner; generic role-lane review is retired"
                    ));
                }
                "reviewReorientResult" => {
                    let result = read_reorient_result(&runtime_store, &thread_id)?;
                    push_event(
                        &mut step,
                        json!({"type": "reorientResult", "result": status_projection::sanitize_for_operator(result.clone())}),
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
                    if result["status"].as_str() == Some("failed") {
                        let recorded = accept_reorient(&runtime_store)?;
                        push_event(
                            &mut step,
                            json!({"type": "reorientFailureRecorded", "recorded": status_projection::sanitize_for_operator(recorded)}),
                        );
                        final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
                        append_operator_step_jsonl(&steps_path, &step)?;
                        steps.push(step);
                        continue;
                    }
                    let can_accept = args.auto_review && reorient_result_auto_acceptable(&result);
                    if can_accept {
                        let accepted = accept_reorient(&runtime_store)?;
                        push_event(
                            &mut step,
                            json!({"type": "reorientAccept", "accepted": status_projection::sanitize_for_operator(accepted)}),
                        );
                        final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
                        append_operator_step_jsonl(&steps_path, &step)?;
                        steps.push(step);
                        continue;
                    }
                    final_action =
                        json!({"action": "reviewReorientResult", "reason": result["note"]});
                    append_operator_step_jsonl(&steps_path, &step)?;
                    steps.push(step);
                    break;
                }
                "continueImplementation" => {
                    let gate = record_hands_implementation_gate(
                        &runtime_store,
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
                "launchImaginationConsideration" => {
                    let work = epiphany_core::project_current_work(&runtime_store)?
                        .imagination_considerations
                        .into_iter()
                        .next()
                        .ok_or_else(|| {
                            anyhow!("no current Persona-feedback Imagination consideration exists")
                        })?;
                    let worker_job_id =
                        epiphany_core::launch_current_imagination_consideration_work(
                            &runtime_store,
                            &work.request.request_id,
                            &now(),
                        )?;
                    push_event(
                        &mut step,
                        json!({
                            "type": "imaginationConsiderationLaunch",
                            "requestId": work.request.request_id,
                            "runtimeJobId": worker_job_id,
                        }),
                    );
                    let worker_run = launch_worker_runtime_detached(
                        &model_runtime_bin,
                        &tool_adapter_bin,
                        &args.model_provider,
                        &args.model,
                        args.provider_credential_path.as_deref(),
                        &runtime_store,
                        &codex_home,
                        &mcp_config,
                        &cwd,
                        args.resident_state_store.as_deref(),
                        &worker_job_id,
                        "imagination",
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
                "launchAdmittedModelDirectionConsideration" => {
                    let work = epiphany_core::project_current_work(&runtime_store)?
                        .admitted_model_direction_consideration
                        .ok_or_else(|| {
                            anyhow!("no current admitted-model direction consideration exists")
                        })?;
                    let worker_job_id =
                        epiphany_core::launch_current_admitted_model_direction_consideration_work(
                            &runtime_store,
                            &work.request.request_id,
                            &now(),
                        )?;
                    push_event(
                        &mut step,
                        json!({
                            "type": "admittedModelDirectionConsiderationLaunch",
                            "requestId": work.request.request_id,
                            "runtimeJobId": worker_job_id,
                        }),
                    );
                    let worker_run = launch_worker_runtime_detached(
                        &model_runtime_bin,
                        &tool_adapter_bin,
                        &args.model_provider,
                        &args.model,
                        args.provider_credential_path.as_deref(),
                        &runtime_store,
                        &codex_home,
                        &mcp_config,
                        &cwd,
                        args.resident_state_store.as_deref(),
                        &worker_job_id,
                        "imagination",
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
                "launchResearch" | "launchModeling" | "launchVerification" => {
                    let role_id = role_id_for_coordinator_action(&action)
                        .ok_or_else(|| anyhow!("unsupported launch action {action}"))?;
                    let pending_proposal_request_id = if role_id == "modeling" {
                        epiphany_core::project_current_work(&runtime_store)?
                            .proposal_modeling
                            .map(|work| work.request.request_id)
                    } else {
                        None
                    };
                    let proposal_modeling_request_id = if role_id == "modeling" {
                        resolve_proposal_modeling_request_hint(
                            args.proposal_modeling_request_id.as_deref(),
                            pending_proposal_request_id.as_deref(),
                        )?
                    } else {
                        None
                    };
                    let launch = if role_id == "research" {
                        let job_id = epiphany_core::launch_current_frontier_research_work(
                            &runtime_store,
                            &now(),
                        )?;
                        json!({
                            "bindingId": epiphany_core::EPIPHANY_RESEARCH_ROLE_BINDING_ID,
                            "backendJobId": job_id,
                        })
                    } else if role_id == "verification" {
                        let job_id = epiphany_core::launch_current_frontier_verification_work(
                            &runtime_store,
                            &now(),
                        )?;
                        json!({
                            "bindingId": epiphany_core::EPIPHANY_VERIFICATION_ROLE_BINDING_ID,
                            "backendJobId": job_id,
                        })
                    } else if role_id == "modeling"
                        && let Some(request_id) = proposal_modeling_request_id
                    {
                        let current = epiphany_core::project_current_work(&runtime_store)?
                            .proposal_modeling
                            .filter(|work| {
                                work.attempt.action
                                    == epiphany_core::EpiphanyAgentPassContinuationAction::Launch
                            })
                            .ok_or_else(|| anyhow!("proposal Modeling work is not launchable"))?;
                        if current.request.request_id != request_id {
                            return Err(anyhow!(
                                "Self-derived proposal Modeling request changed before launch"
                            ));
                        }
                        let binding = epiphany_core::launch_current_proposal_modeling_work(
                            &runtime_store,
                            epiphany_core::EpiphanyProposalModelingLaunchOptions {
                                created_at: now(),
                            },
                        )?;
                        json!({
                            "bindingId": epiphany_core::EPIPHANY_MODELING_ROLE_BINDING_ID,
                            "backendJobId": binding.job_id,
                            "proposalModelingLaunchBinding": binding,
                        })
                    } else if role_id == "modeling"
                        && status["currentWork"]["bodyModeling"].is_object()
                    {
                        let binding = epiphany_core::launch_current_body_modeling_work(
                            &runtime_store,
                            epiphany_core::EpiphanyBodyModelingLaunchOptions {
                                job_id: Uuid::new_v4().to_string(),
                                created_at: now(),
                            },
                        )?;
                        json!({
                            "bindingId": epiphany_core::EPIPHANY_MODELING_ROLE_BINDING_ID,
                            "backendJobId": binding.job_id,
                            "bodyModelingLaunchBinding": binding,
                        })
                    } else if role_id == "modeling"
                        && status["currentWork"]["frontierVerdictModeling"].is_object()
                    {
                        let job_id = epiphany_core::launch_current_frontier_verdict_modeling_work(
                            &runtime_store,
                            &now(),
                        )?;
                        json!({
                            "bindingId": epiphany_core::EPIPHANY_MODELING_ROLE_BINDING_ID,
                            "backendJobId": job_id,
                        })
                    } else {
                        return Err(anyhow!(
                            "{role_id} launch has no keyed current-work owner; generic role-lane launch is retired"
                        ));
                    };
                    let worker_job_id = worker_job_id_from_launch(&launch)?;
                    push_event(
                        &mut step,
                        json!({"type": "roleLaunch", "roleId": role_id, "launch": status_projection::sanitize_for_operator(launch.clone()), "runtimeJobId": worker_job_id}),
                    );
                    let worker_run = launch_worker_runtime_detached(
                        &model_runtime_bin,
                        &tool_adapter_bin,
                        &args.model_provider,
                        &args.model,
                        args.provider_credential_path.as_deref(),
                        &runtime_store,
                        &codex_home,
                        &mcp_config,
                        &cwd,
                        args.resident_state_store.as_deref(),
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
                    let result = match epiphany_core::runtime_role_worker_result(
                        &runtime_store,
                        &worker_job_id,
                    )? {
                        Some(result) => json!({
                            "status": "completed",
                            "note": result.summary,
                        }),
                        None => json!({
                            "status": "running",
                            "note": "The exact keyed role attempt has not produced a terminal result.",
                        }),
                    };
                    push_event(
                        &mut step,
                        json!({"type": "roleResult", "roleId": role_id, "result": status_projection::sanitize_for_operator(result.clone())}),
                    );
                    final_status = collect_coordinator_status(&runtime_store, &thread_id)?;
                    if !matches!(
                        result["status"].as_str(),
                        Some("completed" | "failed" | "cancelled")
                    ) {
                        final_action = json!({
                            "action": wait_action_for_role(role_id),
                            "reason": result["note"],
                            "runtimeJobId": worker_job_id,
                        });
                        append_operator_step_jsonl(&steps_path, &step)?;
                        steps.push(step);
                        break;
                    }
                    if !args.auto_review {
                        final_action = json!({
                            "action": review_action_for_role(role_id),
                            "reason": result["note"],
                            "runtimeJobId": worker_job_id,
                        });
                        append_operator_step_jsonl(&steps_path, &step)?;
                        steps.push(step);
                        break;
                    }
                }
                "launchReorientWorker" => {
                    let launch = launch_reorient(&runtime_store)?;
                    let worker_job_id = worker_job_id_from_launch(&launch)?;
                    push_event(
                        &mut step,
                        json!({"type": "reorientLaunch", "launch": status_projection::sanitize_for_operator(launch.clone()), "runtimeJobId": worker_job_id}),
                    );
                    let worker_run = launch_worker_runtime_detached(
                        &model_runtime_bin,
                        &tool_adapter_bin,
                        &args.model_provider,
                        &args.model,
                        args.provider_credential_path.as_deref(),
                        &runtime_store,
                        &codex_home,
                        &mcp_config,
                        &cwd,
                        args.resident_state_store.as_deref(),
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
                        json!({"type": "reorientResult", "result": status_projection::sanitize_for_operator(result.clone())}),
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
                _ => {}
            }
            append_operator_step_jsonl(&steps_path, &step)?;
            steps.push(step);
        }

        let operator_final_status = status_projection::sanitize_for_operator(final_status);
        let final_rendered = status_projection::render_status(&operator_final_status);
        let operator_steps = status_projection::sanitize_for_operator(Value::Array(steps));
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
            final_runtime_job_id: final_action["runtimeJobId"].as_str().map(str::to_string),
        };
        finalize_coordinator_run(&runtime_store, &coordinator_run_receipt)?;
        let runtime_status = runtime_spine_status(&runtime_store)?;
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
            "finalAction": status_projection::sanitize_for_operator(final_action),
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
    })();
    match run_result {
        Ok(summary) => Ok(summary),
        Err(run_error) => {
            let failure_receipt = failed_coordinator_run_receipt(
                args,
                &runtime_session_id,
                &thread_id,
                &runtime_store,
                &run_error,
            );
            match finalize_coordinator_run(&runtime_store, &failure_receipt) {
                Ok(_) => Err(anyhow!(
                    "coordinator run failed after opening and was terminalized by receipt {:?}: {run_error:#}",
                    failure_receipt.receipt_id
                )),
                Err(finalize_error) => Err(anyhow!(
                    "coordinator run failed after opening ({run_error:#}); failure terminalization also failed: {finalize_error:#}"
                )),
            }
        }
    }
}

fn failed_coordinator_run_receipt(
    args: &Args,
    session_id: &str,
    thread_id: &str,
    runtime_store: &Path,
    error: &anyhow::Error,
) -> EpiphanyCoordinatorRunReceipt {
    let created_at = now();
    EpiphanyCoordinatorRunReceipt {
        schema_version: COORDINATOR_RUN_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: format!(
            "coordinator-run-{}-failed-{}",
            sanitize_id(thread_id),
            chrono::Utc::now().timestamp_millis()
        ),
        session_id: session_id.to_string(),
        thread_id: thread_id.to_string(),
        mode: args.mode.clone(),
        status: "failed".to_string(),
        final_action: "failed".to_string(),
        final_reason: Some(format!("{error:#}")),
        step_count: 0,
        created_at,
        model_provider: Some(args.model_provider.clone()),
        runtime_store: runtime_store.display().to_string(),
        artifact_refs: Vec::new(),
        sealed_artifact_refs: Vec::new(),
        metadata: BTreeMap::from([(
            "terminalizationOwner".to_string(),
            "epiphany-mvp-coordinator-error-boundary".to_string(),
        )]),
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
        final_runtime_job_id: None,
    }
}

fn intake_operator_objective(
    runtime_store: &Path,
    thread_id: &str,
    objective: &str,
    source_ref: &str,
) -> Result<epiphany_core::UserObjectiveIntakeApplied> {
    epiphany_core::intake_user_objective(
        runtime_store,
        epiphany_core::UserObjectiveIntakeInput {
            thread_id: thread_id.to_string(),
            objective: objective.to_string(),
            source_actor: "operator".to_string(),
            source_ref: source_ref.to_string(),
            submitted_at: now(),
        },
    )
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
    status_projection::native_coordinator_json(runtime_store, thread_id)
}

fn resolve_model_runtime_bin(root: &Path, configured: &Path) -> Result<PathBuf> {
    if configured.components().count() != 1 {
        return status_projection::absolute_path(configured);
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
    let requested_paths = route.authorized_paths.clone();

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
        model_projection_digest: route.model_projection_digest.clone(),
        model_source_documents: route.model_source_documents.clone(),
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
        "status": "awaitingHandsExecutor",
        "runtimeJobId": runtime_job_id,
        "substrateGateGrantReceiptId": grant_id,
        "intentId": intent.intent_id,
        "reviewId": review.review_id,
        "routeId": route.route_id,
        "modelProjectionDigest": route.model_projection_digest,
        "modelSourceDocuments": route.model_source_documents,
        "frontierItemId": route.frontier_item_id,
        "requestedPaths": requested_paths,
        "requiredReceipts": review.required_receipts,
        "plannedAction": route.adopted_plan.as_ref().map(|plan| plan.effective_action()),
        "plannedCommand": route.adopted_plan.as_ref().map(|plan| plan.effective_command()),
        "originalPlannedAction": route.adopted_plan.as_ref().map(|plan| plan.action.as_str()),
        "originalPlannedCommand": route.adopted_plan.as_ref().map(|plan| plan.command.as_str()),
        "plannedChecks": route.adopted_plan.as_ref().map(|plan| plan.checks.as_slice()),
        "plannedStopConditions": route.adopted_plan.as_ref().map(|plan| plan.stop_conditions.as_slice()),
        "plannedRollbackSteps": route.adopted_plan.as_ref().map(|plan| plan.rollback_steps.as_slice()),
        "plannedCommitMessage": route.adopted_plan.as_ref().map(|plan| plan.commit_message.as_str()),
        "store": runtime_store,
    }))
}

fn accept_reorient(runtime_store: &Path) -> Result<Value> {
    let work = epiphany_core::project_current_work(runtime_store)?
        .reorientation
        .ok_or_else(|| anyhow!("Mind has no reviewable reorientation work"))?;
    if work.attempt.action != epiphany_core::EpiphanyAgentPassContinuationAction::Review {
        return Err(anyhow!("reorientation work is not reviewable"));
    }
    let job_id = work
        .attempt
        .job_id
        .ok_or_else(|| anyhow!("reorientation review lost its runtime job"))?;
    let snapshot = epiphany_core::runtime_job_snapshot(runtime_store, &job_id)?
        .ok_or_else(|| anyhow!("reorientation review lost its runtime job"))?;
    let accepted = match snapshot.job.status {
        epiphany_core::EpiphanyRuntimeJobStatus::Completed => {
            epiphany_core::accept_reorientation_result(runtime_store, &job_id, &now())?
        }
        epiphany_core::EpiphanyRuntimeJobStatus::Failed => {
            epiphany_core::record_reorientation_pass_failure(runtime_store, &job_id)?
        }
        _ => return Err(anyhow!("reorientation work is not terminally reviewable")),
    };
    Ok(json!({
        "mindCommitReceipt": accepted,
        "changed": true,
    }))
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

fn launch_reorient(runtime_store: &Path) -> Result<Value> {
    let requested_at = now();
    let request = epiphany_core::request_current_reorientation(runtime_store, &requested_at)?;
    let job_id = epiphany_core::launch_current_reorientation_work(runtime_store, &requested_at)?;
    Ok(json!({
        "bindingId": epiphany_core::EPIPHANY_REORIENT_LAUNCH_BINDING_ID,
        "backendJobId": job_id,
        "request": request,
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
    model: &str,
    provider_credential_path: Option<&Path>,
    runtime_store: &Path,
    codex_home: &Path,
    mcp_config: &Path,
    cwd: &Path,
    resident_store: Option<&Path>,
    job_id: &str,
    role_id: &str,
    step_index: usize,
    artifact_dir: &Path,
    max_runtime_seconds: u64,
    auto_tools: bool,
) -> Result<Value> {
    let activation_token = Uuid::new_v4().to_string();
    let activation_token_sha256 = format!("{:x}", Sha256::digest(activation_token.as_bytes()));
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
        .arg("--model")
        .arg(model)
        .arg("--store")
        .arg(runtime_store)
        .arg("--codex-home")
        .arg(codex_home)
        .arg("--mcp-config")
        .arg(mcp_config)
        .arg("--job-id")
        .arg(job_id)
        .arg("--activation-token-sha256")
        .arg(&activation_token_sha256)
        .arg("--max-runtime-seconds")
        .arg(max_runtime_seconds.to_string());
    if let Some(path) = provider_credential_path {
        command.arg("--provider-credential").arg(path);
    }
    if auto_tools {
        command
            .arg("--auto-tools")
            .arg("--tool-adapter-bin")
            .arg(tool_adapter_bin)
            .arg("--cwd")
            .arg(cwd)
            .arg("--max-tool-rounds")
            .arg(WORKER_AUTO_TOOL_MAX_ROUNDS.to_string());
        if let Some(resident_store) = resident_store {
            command.arg("--resident-store").arg(resident_store);
        }
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
    let mut child = command
        .spawn()
        .with_context(|| format!("failed to spawn {}", model_runtime_bin.display()))?;
    let process = capture_process_instance(child.id())?;
    let claim_deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(claim) = epiphany_core::runtime_worker_process_claim(runtime_store, job_id)? {
            if claim.process_id != process.process_id
                || claim.process_creation_token != process.creation_token
                || claim.process_executable_path != process.executable_path.display().to_string()
                || claim.activation_token_sha256 != activation_token_sha256
            {
                return Err(anyhow!(
                    "worker process claim does not bind the spawned incarnation"
                ));
            }
            epiphany_core::activate_runtime_worker_process(
                runtime_store,
                job_id,
                &process,
                &activation_token,
                &chrono::Utc::now().to_rfc3339(),
            )?;
            break;
        }
        if let Some(status) = child.try_wait()? {
            return Err(anyhow!(
                "worker exited with {status} before claiming runtime authority"
            ));
        }
        if std::time::Instant::now() >= claim_deadline {
            return Err(anyhow!(
                "worker did not claim runtime authority before activation deadline"
            ));
        }
        thread::sleep(Duration::from_millis(25));
    }
    Ok(json!({
        "status": "launched",
        "jobId": job_id,
        "pid": process.process_id,
        "stdout": stdout_path,
        "stderr": stderr_path,
        "note": "Worker runtime is detached; coordinator observes completion through runtime-spine role/reorient result polling.",
    }))
}

fn read_reorient_result(runtime_store: &Path, _thread_id: &str) -> Result<Value> {
    let Some(work) = epiphany_core::project_current_work(runtime_store)?.reorientation else {
        return Ok(json!({
            "source": "native",
            "status": "backendMissing",
            "bindingId": epiphany_core::EPIPHANY_REORIENT_LAUNCH_BINDING_ID,
            "note": "No keyed reorientation request is active."
        }));
    };
    let Some(job_id) = work.attempt.job_id else {
        return Ok(json!({
            "source": "native",
            "status": "pending",
            "bindingId": epiphany_core::EPIPHANY_REORIENT_LAUNCH_BINDING_ID,
            "note": "The keyed reorientation request is ready to launch."
        }));
    };
    let snapshot = epiphany_core::read_runtime_reorient_result(Some(runtime_store), &job_id);
    let mut value = json!({
        "source": "native",
        "status": snapshot.status,
        "bindingId": epiphany_core::EPIPHANY_REORIENT_LAUNCH_BINDING_ID,
        "runtimeJobId": job_id,
        "note": snapshot.note,
    });
    if let Some(finding) = snapshot.finding {
        value["finding"] = json!(finding);
    }
    Ok(value)
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
    append_jsonl(path, &status_projection::sanitize_for_operator(step.clone()))
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
    fn coordinator_error_after_opening_terminalizes_the_exact_session() -> Result<()> {
        let smoke_root = env::current_dir()?.join(".epiphany-smoke");
        fs::create_dir_all(&smoke_root)?;
        let temp = tempfile::tempdir_in(smoke_root)?;
        let cwd = temp.path().join("workspace");
        fs::create_dir_all(&cwd)?;
        let current_exe = env::current_exe()?;
        let runtime_store = temp.path().join("runtime.cc");
        let args = Args {
            model_runtime_bin: current_exe.clone(),
            tool_adapter_bin: current_exe,
            model_provider: "test".to_string(),
            model: "test-model".to_string(),
            provider_credential_path: None,
            thread_id: Some("error-terminalization".to_string()),
            objective: Some("Exercise the coordinator error boundary.".to_string()),
            cwd: cwd.clone(),
            codex_home: temp.path().join("codex-home"),
            mcp_config: cwd.join(".epiphany/mcp.toml"),
            artifact_dir: cwd.join(".epiphany-smoke/error-terminalization"),
            runtime_store: runtime_store.clone(),
            local_verse_store: temp.path().join("local-verse.cc"),
            mode: "plan".to_string(),
            max_steps: 1,
            poll_seconds: 0.01,
            timeout_seconds: 1,
            max_runtime_seconds: 1,
            ephemeral: true,
            auto_review: false,
            supersede_failed_results: false,
            approve_manual_regather: false,
            auto_tools: false,
            proposal_modeling_request_id: None,
            imagination_consideration_request_id: Some("conflicting-request".to_string()),
            admitted_model_direction_consideration_request_id: None,
            resident_binding: BTreeMap::new(),
            resident_state_store: None,
            resident_preparation_id: None,
            runtime_id: Some("epiphany-error-terminalization".to_string()),
            required_action: None,
        };

        let error = run_coordinator(&args)
            .expect_err("conflicting typed and operator intake must fail after opening");
        assert!(
            error.to_string().contains("was terminalized by receipt"),
            "unexpected coordinator error: {error:#}"
        );
        let mut cache = epiphany_core::runtime_spine_cache(&runtime_store)?;
        cache.pull_all_backing_stores()?;
        let session = cache.get_required::<epiphany_core::EpiphanyRuntimeSession>(
            "coordinator-error-terminalization",
        )?;
        assert_eq!(
            session.status,
            epiphany_core::EpiphanyRuntimeSessionStatus::Completed
        );
        let receipts = cache.get_all::<EpiphanyCoordinatorRunReceipt>()?;
        assert_eq!(receipts.len(), 1);
        assert_eq!(receipts[0].status, "failed");
        assert_eq!(receipts[0].session_id, session.session_id);
        Ok(())
    }

    #[test]
    fn continue_implementation_is_not_a_passive_stop_action() {
        assert!(!is_stop_action("continueImplementation"));
    }

    #[test]
    fn operator_objective_intake_creates_once_and_refuses_replacement() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime.cc");
        epiphany_core::initialize_runtime_spine(
            &store,
            epiphany_core::RuntimeSpineInitOptions {
                runtime_id: "objective-intake-bin".into(),
                display_name: "Objective intake bin".into(),
                created_at: "2026-08-14T00:00:00Z".into(),
            },
        )?;

        let first = intake_operator_objective(
            &store,
            "thread-1",
            "Map the machine",
            "cli://epiphany-mvp-coordinator",
        )?;
        assert!(first.changed);
        assert_eq!(first.mind.objective.as_deref(), Some("Map the machine"));

        let repeated = intake_operator_objective(
            &store,
            "thread-1",
            " Map the machine ",
            "cli://epiphany-mvp-coordinator",
        )?;
        assert!(!repeated.changed);
        assert_eq!(
            repeated.mind.projection_digest,
            first.mind.projection_digest
        );
        assert_eq!(repeated.commit_receipt, first.commit_receipt);

        let error = intake_operator_objective(
            &store,
            "thread-1",
            "Replace the machine",
            "cli://epiphany-mvp-coordinator",
        )
        .expect_err("objective replacement must require a typed adoption flow");
        assert!(error.to_string().contains("refusing to replace"));
        Ok(())
    }

    #[test]
    fn resident_operator_objective_uses_the_authenticated_grant_as_provenance() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime.cc");
        epiphany_core::initialize_runtime_spine(
            &store,
            epiphany_core::RuntimeSpineInitOptions {
                runtime_id: "resident-objective-intake-bin".into(),
                display_name: "Resident objective intake bin".into(),
                created_at: "2026-08-22T00:00:00Z".into(),
            },
        )?;

        let applied = intake_operator_objective(
            &store,
            "resident-thread",
            "Keep the objective in Mind",
            "resident-self-grant://grant-1",
        )?;
        assert!(applied.changed);
        assert_eq!(applied.intake.source_ref, "resident-self-grant://grant-1");
        assert_eq!(
            applied.mind.objective.as_deref(),
            Some("Keep the objective in Mind")
        );
        assert!(
            applied
                .commit_receipt
                .writes
                .iter()
                .any(|write| write.document_key == epiphany_core::MIND_OBJECTIVE_KEY)
        );
        Ok(())
    }

    #[test]
    fn operator_can_approve_only_a_self_derived_manual_regather() -> Result<()> {
        let mut coordinator = json!({
            "action": "regatherManually",
        });
        assert_eq!(
            apply_manual_regather_approval("regatherManually", &mut coordinator)?,
            "launchResearch"
        );
        assert_eq!(coordinator["action"], "launchResearch");

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
}
