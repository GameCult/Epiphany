use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::DateTime;
use chrono::Utc;
use epiphany_core::EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_REPO_WORK_MAP_ENTRY_LATEST_KEY;
use epiphany_core::EPIPHANY_CULTMESH_REPO_WORK_MAP_ENTRY_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_REPO_WORK_OVERVIEW_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_REPO_WORK_PUBLIC_PROOF_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_REPO_WORK_READINESS_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_WEKSA_LOWERING_RECEIPT_SCHEMA_VERSION;
use epiphany_core::EpiphanyCultMeshRepoWorkMapEntry;
use epiphany_core::EpiphanyCultMeshRepoWorkOverviewEntry;
use epiphany_core::EpiphanyCultMeshRepoWorkPublicProofEntry;
use epiphany_core::EpiphanyCultMeshRepoWorkReadinessEntry;
use epiphany_core::EpiphanyCultMeshWeksaLoweringReceiptEntry;
use epiphany_core::EpiphanyRepoWorkModelingLaunchDocument;
use epiphany_core::EpiphanyWorkerLaunchDocument;
use epiphany_core::HANDS_ACTION_INTENT_SCHEMA_VERSION;
use epiphany_core::HANDS_COMMAND_RECEIPT_TYPE;
use epiphany_core::HANDS_COMMIT_RECEIPT_TYPE;
use epiphany_core::HANDS_PATCH_RECEIPT_TYPE;
use epiphany_core::HandsActionIntent;
use epiphany_core::MIND_GATEWAY_REVIEW_SCHEMA_VERSION;
use epiphany_core::MindGatewayDecision;
use epiphany_core::MindGatewayReview;
use epiphany_core::PersonaMemoryCacheConfig;
use epiphany_core::REPO_WORK_MAP_ENTRY_SCHEMA_VERSION;
use epiphany_core::REPO_WORK_MODELING_OUTPUT_CONTRACT_ID;
use epiphany_core::REPO_WORK_MODELING_REQUEST_SCHEMA_VERSION;
use epiphany_core::REPO_WORK_MODELING_ROUTE_SCHEMA_VERSION;
use epiphany_core::RepoWorkMapEntry;
use epiphany_core::RepoWorkModelingRequest;
use epiphany_core::RepoWorkModelingRoute;
use epiphany_core::RuntimeSpineHeartbeatJobOptions;
use epiphany_core::RuntimeSpineInitOptions;
use epiphany_core::SOUL_VERDICT_RECEIPT_SCHEMA_VERSION;
use epiphany_core::SoulVerdictReceipt;
use epiphany_core::WeksaInterlinguaInput;
use epiphany_core::WeksaSpeakerContext;
use epiphany_core::advance_repo_work_modeling_route;
use epiphany_core::build_weksa_interlingua_packet;
use epiphany_core::build_weksa_target_lowering_request;
use epiphany_core::commit_initial_repo_work_modeling_route;
use epiphany_core::commit_repo_work_map_admission;
use epiphany_core::default_launch_organ_contract;
use epiphany_core::hands_action_review_for_intent;
use epiphany_core::hands_command_receipt_for_review;
use epiphany_core::hands_commit_receipt_for_review;
use epiphany_core::hands_patch_receipt_for_review;
use epiphany_core::initialize_runtime_spine;
use epiphany_core::load_agent_memory_entry_for_role;
use epiphany_core::load_epiphany_cultmesh_idunn_aftercare_audit_receipt;
use epiphany_core::load_epiphany_cultmesh_idunn_deployment_receipt;
use epiphany_core::load_epiphany_cultmesh_repo_work_overviews;
use epiphany_core::load_epiphany_cultmesh_swarm_brake;
use epiphany_core::load_latest_epiphany_cultmesh_idunn_aftercare_audit_receipt;
use epiphany_core::load_latest_epiphany_cultmesh_idunn_deployment_receipt;
use epiphany_core::load_latest_epiphany_cultmesh_repo_work_overview;
use epiphany_core::memory_graph_from_agent_memories;
use epiphany_core::mind_state_commit_receipt;
use epiphany_core::open_runtime_spine_heartbeat_job;
use epiphany_core::plan_memory_graph_context_cut;
use epiphany_core::put_hands_action_intent;
use epiphany_core::put_hands_action_review;
use epiphany_core::put_hands_command_receipt;
use epiphany_core::put_hands_commit_receipt;
use epiphany_core::put_hands_patch_receipt;
use epiphany_core::put_soul_verdict_receipt;
use epiphany_core::put_substrate_gate_repo_access_grant_receipt;
use epiphany_core::record_weksa_target_lowering_receipt;
use epiphany_core::render_persona_memory_recall_with_cache;
use epiphany_core::runtime_hands_action_intent;
use epiphany_core::runtime_hands_action_review;
use epiphany_core::runtime_hands_commit_receipt;
use epiphany_core::runtime_job_snapshot;
use epiphany_core::runtime_repo_work_map_entry;
use epiphany_core::runtime_repo_work_modeling_finding;
use epiphany_core::runtime_repo_work_modeling_request;
use epiphany_core::runtime_repo_work_modeling_route;
use epiphany_core::substrate_gate_repo_work_planning_grant;
use epiphany_core::write_epiphany_cultmesh_repo_work_map_entry;
use epiphany_core::write_epiphany_cultmesh_repo_work_overview;
use epiphany_core::write_epiphany_cultmesh_repo_work_public_proof;
use epiphany_core::write_epiphany_cultmesh_repo_work_readiness;
use epiphany_core::write_epiphany_cultmesh_weksa_lowering_receipt;
use epiphany_state_model::EpiphanyMemoryContextQuery;
use epiphany_state_model::EpiphanyMemoryProfile;
use serde_json::Value;
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

#[path = "epiphany_work/closure_contracts.rs"]
mod closure_contracts;
use closure_contracts::*;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print_usage();
        std::process::exit(2);
    };
    let result = match command.as_str() {
        "accept" => run_accept(parse_accept_args(args)?),
        "persona-intake" | "intake" | "repo-persona-intake" => {
            run_persona_intake(parse_persona_intake_args(args)?)
        }
        "derive-plan" | "imagine" | "plan-from-pressure" => {
            run_derive_plan(parse_derive_plan_args(args)?)
        }
        "plan" => run_plan(parse_plan_args(args)?),
        "run" => run_work(parse_run_args(args)?),
        "adopt" | "promote" => run_adopt(parse_adopt_args(args)?),
        "execute" | "exec" => run_execute(parse_execute_args(args)?),
        "verify" | "soul-verify" => run_verify(parse_close_args(args)?),
        "close" | "closure" | "verify-close" => run_close(parse_close_args(args)?),
        "revise-modeling" | "modeling-retry" => {
            run_revise_modeling(parse_revise_modeling_args(args)?)
        }
        "overview" | "proof-bundle" | "status" => run_overview(parse_overview_args(args)?),
        "readiness" | "readiness-report" | "mvp-readiness" => {
            run_readiness(parse_readiness_args(args)?)
        }
        "deployment-config-audit" | "audit-deployment-config" | "idunn-deployment-audit" => {
            run_deployment_config_audit(parse_deployment_config_audit_args(args)?)
        }
        "deployment-execution-runbook" | "idunn-deployment-runbook" | "deployment-push-runbook" => {
            run_deployment_execution_runbook(parse_deployment_execution_runbook_args(args)?)
        }
        "deployment-aftercare-audit" | "idunn-aftercare-audit" | "deployment-receipt-audit" => {
            run_deployment_aftercare_audit(parse_deployment_aftercare_audit_args(args)?)
        }
        "export-proof" | "public-proof" => run_export_proof(parse_export_proof_args(args)?),
        "tick" | "pulse" | "schedule" => run_tick(parse_tick_args(args)?),
        "queue-run" | "run-queue" | "queue-tick" | "scheduler-run" => {
            run_queue(parse_queue_args(args)?)
        }
        "serve" | "loop" | "daemon" => run_serve(parse_serve_args(args)?),
        other => Err(anyhow!("unknown epiphany-work command {other:?}")),
    }?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Clone, Debug)]
struct AcceptArgs {
    workspace: PathBuf,
    epiphany_root: PathBuf,
    source: String,
    item: String,
    summary: Option<String>,
    topic: Option<String>,
    local_verse_store: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    runtime_id: Option<String>,
    online_receipt: Option<PathBuf>,
    eve_connection_receipt_id: Option<String>,
    public_discussion_refs: Vec<String>,
    candidate_action_refs: Vec<String>,
}

#[derive(Clone, Debug)]
struct PersonaIntakeArgs {
    workspace: PathBuf,
    epiphany_root: PathBuf,
    item: String,
    message: String,
    topic: Option<String>,
    local_verse_store: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    runtime_id: Option<String>,
    online_receipt: Option<PathBuf>,
    mood: String,
}

#[derive(Clone, Debug)]
struct RunArgs {
    workspace: PathBuf,
    epiphany_root: PathBuf,
    item: Option<String>,
    accept_receipt: Option<PathBuf>,
    runtime_store: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    requested_paths: Vec<String>,
}

#[derive(Clone, Debug)]
struct PlanArgs {
    workspace: PathBuf,
    item: Option<String>,
    accept_receipt: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    objective: String,
    plan_summary: String,
    command: String,
    changed_paths: Vec<String>,
    commit_message: String,
    adoption_evidence_refs: Vec<String>,
    verification_asks: Vec<String>,
    stop_conditions: Vec<String>,
    rollback_hints: Vec<String>,
}

#[derive(Clone, Debug)]
struct DerivePlanArgs {
    workspace: PathBuf,
    item: Option<String>,
    accept_receipt: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    target_path: Option<String>,
    action_family: String,
    model_ref: Option<String>,
    model_authored: bool,
    action_summary: Option<String>,
    verification_asks: Vec<String>,
    stop_conditions: Vec<String>,
    escalation_reasons: Vec<String>,
    assumptions: Vec<String>,
    constraints: Vec<String>,
    non_goals: Vec<String>,
    open_questions: Vec<String>,
    decision_points: Vec<String>,
    evidence_needs: Vec<String>,
}

#[derive(Clone, Debug)]
struct PlanReceiptInputs {
    objective: String,
    plan_summary: String,
    command: String,
    changed_paths: Vec<String>,
    commit_message: String,
    adoption_evidence_refs: Vec<String>,
    verification_asks: Vec<String>,
    stop_conditions: Vec<String>,
    rollback_hints: Vec<String>,
    derivation: Option<Value>,
}

#[derive(Clone, Debug)]
struct AdoptArgs {
    workspace: PathBuf,
    epiphany_root: PathBuf,
    item: Option<String>,
    run_receipt: Option<PathBuf>,
    plan_receipt: Option<PathBuf>,
    runtime_store: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    plan_summary: Option<String>,
    adoption_evidence_refs: Vec<String>,
    mind_adoption_rationale: Option<String>,
}

#[derive(Clone, Debug)]
struct ExecuteArgs {
    workspace: PathBuf,
    epiphany_root: PathBuf,
    item: Option<String>,
    adopt_receipt: Option<PathBuf>,
    plan_receipt: Option<PathBuf>,
    runtime_store: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    command: Option<String>,
    changed_paths: Vec<String>,
    commit_message: Option<String>,
    summary: Option<String>,
}

#[derive(Clone, Debug)]
struct CloseArgs {
    workspace: PathBuf,
    item: Option<String>,
    execute_receipt: Option<PathBuf>,
    runtime_store: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
}

#[derive(Clone, Debug)]
struct ReviseModelingArgs {
    workspace: PathBuf,
    item: String,
    runtime_store: Option<PathBuf>,
    rationale: String,
    review_ref: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ClosurePhase {
    SoulOnly,
    Full,
}

#[derive(Clone, Debug)]
struct OverviewArgs {
    workspace: PathBuf,
    item: Option<String>,
    accept_receipt: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    write_receipt: bool,
}

#[derive(Clone, Debug)]
struct ReadinessArgs {
    workspace: PathBuf,
    item: Option<String>,
    accept_receipt: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    public_proof: Option<PathBuf>,
    idunn_lifecycle_receipt: Option<PathBuf>,
    tool_directory_receipt: Option<PathBuf>,
    deployment_aftercare_audit_receipt: Option<PathBuf>,
    deployment_aftercare_audit_receipt_ref: Option<String>,
    write_receipt: bool,
}

#[derive(Clone, Debug)]
struct DeploymentConfigAuditArgs {
    workspace: PathBuf,
    artifact_dir: Option<PathBuf>,
    write_receipt: bool,
}

#[derive(Clone, Debug)]
struct DeploymentExecutionRunbookArgs {
    workspace: PathBuf,
    artifact_dir: Option<PathBuf>,
    remote: String,
    write_receipt: bool,
}

#[derive(Clone, Debug)]
struct DeploymentAftercareAuditArgs {
    workspace: PathBuf,
    artifact_dir: Option<PathBuf>,
    local_verse_store: Option<PathBuf>,
    runtime_id: String,
    runbook_receipt: Option<PathBuf>,
    idunn_deployment_receipt: Option<PathBuf>,
    idunn_deployment_receipt_ref: Option<String>,
    aftercare_audit_receipt: Option<PathBuf>,
    aftercare_audit_receipt_ref: Option<String>,
    write_receipt: bool,
}

#[derive(Clone, Debug)]
struct ExportProofArgs {
    workspace: PathBuf,
    item: Option<String>,
    accept_receipt: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    output: Option<PathBuf>,
    local_verse_store: Option<PathBuf>,
    runtime_id: String,
}

#[derive(Clone, Debug)]
struct TickArgs {
    workspace: PathBuf,
    epiphany_root: PathBuf,
    item: Option<String>,
    local_verse_store: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    runtime_store: Option<PathBuf>,
    cooldown_seconds: u64,
    active_timeout_seconds: u64,
    dry_run: bool,
}

#[derive(Clone, Debug)]
struct QueueArgs {
    workspace: PathBuf,
    epiphany_root: PathBuf,
    local_verse_store: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    runtime_store: Option<PathBuf>,
    runtime_id: String,
    max_items: u64,
    cooldown_seconds: u64,
    active_timeout_seconds: u64,
    dry_run: bool,
}

#[derive(Clone, Debug)]
struct ServeArgs {
    tick: TickArgs,
    scheduler_id: String,
    loop_interval_seconds: u64,
    max_iterations: u64,
}

fn parse_accept_args(args: impl Iterator<Item = String>) -> Result<AcceptArgs> {
    let mut workspace = None;
    let mut epiphany_root = None;
    let mut source = None;
    let mut item = None;
    let mut summary = None;
    let mut topic = None;
    let mut local_verse_store = None;
    let mut artifact_dir = None;
    let mut runtime_id = None;
    let mut online_receipt = None;
    let mut eve_connection_receipt_id = None;
    let mut public_discussion_refs = Vec::new();
    let mut candidate_action_refs = Vec::new();

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--epiphany-root" => epiphany_root = Some(take_path(&mut args, "--epiphany-root")?),
            "--from" => source = Some(take_string(&mut args, "--from")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--summary" => summary = Some(take_string(&mut args, "--summary")?),
            "--topic" => topic = Some(take_string(&mut args, "--topic")?),
            "--local-verse-store" | "--store" => {
                local_verse_store = Some(take_path(&mut args, "--local-verse-store")?);
            }
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--runtime-id" => runtime_id = Some(take_string(&mut args, "--runtime-id")?),
            "--online-receipt" => online_receipt = Some(take_path(&mut args, "--online-receipt")?),
            "--eve-connection-receipt-id" => {
                eve_connection_receipt_id =
                    Some(take_string(&mut args, "--eve-connection-receipt-id")?);
            }
            "--public-discussion-ref" | "--public-ref" => {
                public_discussion_refs.push(take_string(&mut args, "--public-discussion-ref")?);
            }
            "--candidate-action-ref" | "--candidate-ref" => {
                candidate_action_refs.push(take_string(&mut args, "--candidate-action-ref")?);
            }
            other => return Err(anyhow!("unexpected accept argument {other:?}")),
        }
    }
    let source = source.context("missing --from persona-or-bifrost")?;
    if !matches!(
        source.as_str(),
        "persona" | "bifrost" | "persona-or-bifrost"
    ) {
        return Err(anyhow!(
            "--from must be persona, bifrost, or persona-or-bifrost"
        ));
    }
    Ok(AcceptArgs {
        workspace: workspace.context("missing --workspace")?,
        epiphany_root: epiphany_root
            .unwrap_or(env::current_dir().context("failed to resolve current directory")?),
        source,
        item: item.context("missing --item")?,
        summary,
        topic,
        local_verse_store,
        artifact_dir,
        runtime_id,
        online_receipt,
        eve_connection_receipt_id,
        public_discussion_refs,
        candidate_action_refs,
    })
}

fn parse_persona_intake_args(args: impl Iterator<Item = String>) -> Result<PersonaIntakeArgs> {
    let mut workspace = None;
    let mut epiphany_root = None;
    let mut item = None;
    let mut message = None;
    let mut topic = None;
    let mut local_verse_store = None;
    let mut artifact_dir = None;
    let mut runtime_id = None;
    let mut online_receipt = None;
    let mut mood = "attentive".to_string();

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--epiphany-root" => epiphany_root = Some(take_path(&mut args, "--epiphany-root")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--message" | "--content" | "--persona-input" => {
                message = Some(take_string(&mut args, "--message")?);
            }
            "--topic" => topic = Some(take_string(&mut args, "--topic")?),
            "--local-verse-store" | "--store" => {
                local_verse_store = Some(take_path(&mut args, "--local-verse-store")?);
            }
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--runtime-id" => runtime_id = Some(take_string(&mut args, "--runtime-id")?),
            "--online-receipt" => online_receipt = Some(take_path(&mut args, "--online-receipt")?),
            "--mood" => mood = take_string(&mut args, "--mood")?,
            other => return Err(anyhow!("unexpected persona-intake argument {other:?}")),
        }
    }
    Ok(PersonaIntakeArgs {
        workspace: workspace.context("missing --workspace")?,
        epiphany_root: epiphany_root
            .unwrap_or(env::current_dir().context("failed to resolve current directory")?),
        item: item.context("missing --item")?,
        message: message.context("missing --message")?,
        topic,
        local_verse_store,
        artifact_dir,
        runtime_id,
        online_receipt,
        mood,
    })
}

fn parse_run_args(args: impl Iterator<Item = String>) -> Result<RunArgs> {
    let mut workspace = None;
    let mut epiphany_root = None;
    let mut item = None;
    let mut accept_receipt = None;
    let mut runtime_store = None;
    let mut artifact_dir = None;
    let mut requested_paths = Vec::new();

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--epiphany-root" => epiphany_root = Some(take_path(&mut args, "--epiphany-root")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--accept-receipt" => accept_receipt = Some(take_path(&mut args, "--accept-receipt")?),
            "--runtime-store" => runtime_store = Some(take_path(&mut args, "--runtime-store")?),
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--path" | "--requested-path" => {
                requested_paths.push(take_string(&mut args, "--requested-path")?);
            }
            other => return Err(anyhow!("unexpected run argument {other:?}")),
        }
    }
    Ok(RunArgs {
        workspace: workspace.context("missing --workspace")?,
        epiphany_root: epiphany_root
            .unwrap_or(env::current_dir().context("failed to resolve current directory")?),
        item,
        accept_receipt,
        runtime_store,
        artifact_dir,
        requested_paths,
    })
}

fn parse_plan_args(args: impl Iterator<Item = String>) -> Result<PlanArgs> {
    let mut workspace = None;
    let mut item = None;
    let mut accept_receipt = None;
    let mut artifact_dir = None;
    let mut objective = None;
    let mut plan_summary = None;
    let mut command = None;
    let mut changed_paths = Vec::new();
    let mut commit_message = None;
    let mut adoption_evidence_refs = Vec::new();
    let mut verification_asks = Vec::new();
    let mut stop_conditions = Vec::new();
    let mut rollback_hints = Vec::new();

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--accept-receipt" => accept_receipt = Some(take_path(&mut args, "--accept-receipt")?),
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--objective" => objective = Some(take_string(&mut args, "--objective")?),
            "--plan-summary" | "--summary" => {
                plan_summary = Some(take_string(&mut args, "--plan-summary")?);
            }
            "--command" => command = Some(take_string(&mut args, "--command")?),
            "--changed-path" | "--path" | "--requested-path" => {
                changed_paths.push(take_string(&mut args, "--changed-path")?);
            }
            "--commit-message" => {
                commit_message = Some(take_string(&mut args, "--commit-message")?);
            }
            "--adoption-evidence-ref" | "--evidence-ref" => {
                adoption_evidence_refs.push(take_string(&mut args, "--adoption-evidence-ref")?);
            }
            "--verification-ask" => {
                verification_asks.push(take_string(&mut args, "--verification-ask")?);
            }
            "--stop-condition" => {
                stop_conditions.push(take_string(&mut args, "--stop-condition")?);
            }
            "--rollback-hint" => {
                rollback_hints.push(take_string(&mut args, "--rollback-hint")?);
            }
            other => return Err(anyhow!("unexpected plan argument {other:?}")),
        }
    }
    if changed_paths.is_empty() {
        return Err(anyhow!(
            "plan requires at least one --changed-path for the future Hands gate"
        ));
    }
    Ok(PlanArgs {
        workspace: workspace.context("missing --workspace")?,
        item,
        accept_receipt,
        artifact_dir,
        objective: objective.context("missing --objective")?,
        plan_summary: plan_summary.context("missing --plan-summary")?,
        command: command.context("missing --command")?,
        changed_paths,
        commit_message: commit_message.context("missing --commit-message")?,
        adoption_evidence_refs: if adoption_evidence_refs.is_empty() {
            vec!["imagination.plan:operator-reviewed".to_string()]
        } else {
            adoption_evidence_refs
        },
        verification_asks: if verification_asks.is_empty() {
            vec!["Soul verifies the declared changed paths and command artifacts.".to_string()]
        } else {
            verification_asks
        },
        stop_conditions: if stop_conditions.is_empty() {
            vec!["Stop if the command exits nonzero or changes paths outside the plan.".to_string()]
        } else {
            stop_conditions
        },
        rollback_hints,
    })
}

fn parse_derive_plan_args(args: impl Iterator<Item = String>) -> Result<DerivePlanArgs> {
    let mut workspace = None;
    let mut item = None;
    let mut accept_receipt = None;
    let mut artifact_dir = None;
    let mut target_path = None;
    let mut action_family = "append-worklog".to_string();
    let mut model_ref = None;
    let mut model_authored = false;
    let mut action_summary = None;
    let mut verification_asks = Vec::new();
    let mut stop_conditions = Vec::new();
    let mut escalation_reasons = Vec::new();
    let mut assumptions = Vec::new();
    let mut constraints = Vec::new();
    let mut non_goals = Vec::new();
    let mut open_questions = Vec::new();
    let mut decision_points = Vec::new();
    let mut evidence_needs = Vec::new();

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--accept-receipt" => accept_receipt = Some(take_path(&mut args, "--accept-receipt")?),
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--target-path" | "--changed-path" | "--path" => {
                target_path = Some(take_string(&mut args, "--target-path")?);
            }
            "--action-family" | "--family" | "--mode" => {
                action_family = take_string(&mut args, "--action-family")?;
            }
            "--model-ref" | "--imagination-ref" => {
                model_ref = Some(take_string(&mut args, "--model-ref")?);
            }
            "--model-authored" => model_authored = true,
            "--action-summary" | "--action-item-summary" => {
                action_summary = Some(take_string(&mut args, "--action-summary")?);
            }
            "--verification-ask" => {
                verification_asks.push(take_string(&mut args, "--verification-ask")?);
            }
            "--stop-condition" => {
                stop_conditions.push(take_string(&mut args, "--stop-condition")?);
            }
            "--escalation-reason" => {
                escalation_reasons.push(take_string(&mut args, "--escalation-reason")?);
            }
            "--assumption" | "--planning-assumption" => {
                assumptions.push(take_string(&mut args, "--assumption")?);
            }
            "--constraint" | "--planning-constraint" => {
                constraints.push(take_string(&mut args, "--constraint")?);
            }
            "--non-goal" | "--nongoal" => {
                non_goals.push(take_string(&mut args, "--non-goal")?);
            }
            "--open-question" => {
                open_questions.push(take_string(&mut args, "--open-question")?);
            }
            "--decision-point" => {
                decision_points.push(take_string(&mut args, "--decision-point")?);
            }
            "--evidence-need" | "--evidence-needed" => {
                evidence_needs.push(take_string(&mut args, "--evidence-need")?);
            }
            other => return Err(anyhow!("unexpected derive-plan argument {other:?}")),
        }
    }
    Ok(DerivePlanArgs {
        workspace: workspace.context("missing --workspace")?,
        item,
        accept_receipt,
        artifact_dir,
        target_path,
        action_family,
        model_ref,
        model_authored,
        action_summary,
        verification_asks,
        stop_conditions,
        escalation_reasons,
        assumptions,
        constraints,
        non_goals,
        open_questions,
        decision_points,
        evidence_needs,
    })
}

fn parse_adopt_args(args: impl Iterator<Item = String>) -> Result<AdoptArgs> {
    let mut workspace = None;
    let mut epiphany_root = None;
    let mut item = None;
    let mut run_receipt = None;
    let mut plan_receipt = None;
    let mut runtime_store = None;
    let mut artifact_dir = None;
    let mut plan_summary = None;
    let mut adoption_evidence_refs = Vec::new();
    let mut mind_adoption_rationale = None;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--epiphany-root" => epiphany_root = Some(take_path(&mut args, "--epiphany-root")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--run-receipt" => run_receipt = Some(take_path(&mut args, "--run-receipt")?),
            "--from-plan" | "--plan-receipt" => {
                plan_receipt = Some(take_path(&mut args, "--from-plan")?);
            }
            "--runtime-store" => runtime_store = Some(take_path(&mut args, "--runtime-store")?),
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--plan-summary" => plan_summary = Some(take_string(&mut args, "--plan-summary")?),
            "--adoption-evidence-ref" | "--evidence-ref" => {
                adoption_evidence_refs.push(take_string(&mut args, "--adoption-evidence-ref")?);
            }
            "--mind-adoption-rationale" | "--adoption-rationale" => {
                mind_adoption_rationale =
                    Some(take_string(&mut args, "--mind-adoption-rationale")?);
            }
            other => return Err(anyhow!("unexpected adopt argument {other:?}")),
        }
    }
    if adoption_evidence_refs.is_empty() && plan_receipt.is_none() {
        return Err(anyhow!(
            "adopt requires at least one --adoption-evidence-ref or --from-plan proving Imagination/Self/Mind adoption"
        ));
    }
    Ok(AdoptArgs {
        workspace: workspace.context("missing --workspace")?,
        epiphany_root: epiphany_root
            .unwrap_or(env::current_dir().context("failed to resolve current directory")?),
        item,
        run_receipt,
        plan_receipt,
        runtime_store,
        artifact_dir,
        plan_summary,
        adoption_evidence_refs,
        mind_adoption_rationale,
    })
}

fn parse_execute_args(args: impl Iterator<Item = String>) -> Result<ExecuteArgs> {
    let mut workspace = None;
    let mut epiphany_root = None;
    let mut item = None;
    let mut adopt_receipt = None;
    let mut plan_receipt = None;
    let mut runtime_store = None;
    let mut artifact_dir = None;
    let mut command = None;
    let mut changed_paths = Vec::new();
    let mut commit_message = None;
    let mut summary = None;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--epiphany-root" => epiphany_root = Some(take_path(&mut args, "--epiphany-root")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--adopt-receipt" => adopt_receipt = Some(take_path(&mut args, "--adopt-receipt")?),
            "--from-plan" | "--plan-receipt" => {
                plan_receipt = Some(take_path(&mut args, "--from-plan")?);
            }
            "--runtime-store" => runtime_store = Some(take_path(&mut args, "--runtime-store")?),
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--command" => command = Some(take_string(&mut args, "--command")?),
            "--changed-path" | "--path" => {
                changed_paths.push(take_string(&mut args, "--changed-path")?);
            }
            "--commit-message" => {
                commit_message = Some(take_string(&mut args, "--commit-message")?)
            }
            "--summary" => summary = Some(take_string(&mut args, "--summary")?),
            other => return Err(anyhow!("unexpected execute argument {other:?}")),
        }
    }
    if changed_paths.is_empty() && plan_receipt.is_none() {
        return Err(anyhow!(
            "execute requires at least one --changed-path or --from-plan scoped by the approved Hands gate"
        ));
    }
    Ok(ExecuteArgs {
        workspace: workspace.context("missing --workspace")?,
        epiphany_root: epiphany_root
            .unwrap_or(env::current_dir().context("failed to resolve current directory")?),
        item,
        adopt_receipt,
        plan_receipt,
        runtime_store,
        artifact_dir,
        command,
        changed_paths,
        commit_message,
        summary,
    })
}

fn parse_close_args(args: impl Iterator<Item = String>) -> Result<CloseArgs> {
    let mut workspace = None;
    let mut item = None;
    let mut execute_receipt = None;
    let mut runtime_store = None;
    let mut artifact_dir = None;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--execute-receipt" | "--from-execute" => {
                execute_receipt = Some(take_path(&mut args, "--execute-receipt")?);
            }
            "--runtime-store" => runtime_store = Some(take_path(&mut args, "--runtime-store")?),
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            other => return Err(anyhow!("unexpected close argument {other:?}")),
        }
    }
    Ok(CloseArgs {
        workspace: workspace.context("missing --workspace")?,
        item,
        execute_receipt,
        runtime_store,
        artifact_dir,
    })
}

fn parse_revise_modeling_args(args: impl Iterator<Item = String>) -> Result<ReviseModelingArgs> {
    let mut workspace = None;
    let mut item = None;
    let mut runtime_store = None;
    let mut rationale = None;
    let mut review_ref = None;
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--runtime-store" => runtime_store = Some(take_path(&mut args, "--runtime-store")?),
            "--rationale" => rationale = Some(take_string(&mut args, "--rationale")?),
            "--review-ref" => review_ref = Some(take_string(&mut args, "--review-ref")?),
            other => return Err(anyhow!("unexpected revise-modeling argument {other:?}")),
        }
    }
    Ok(ReviseModelingArgs {
        workspace: workspace.context("missing --workspace")?,
        item: item.context("missing --item")?,
        runtime_store,
        rationale: rationale.context("missing --rationale")?,
        review_ref: review_ref.context("missing --review-ref")?,
    })
}

fn parse_overview_args(args: impl Iterator<Item = String>) -> Result<OverviewArgs> {
    let mut workspace = None;
    let mut item = None;
    let mut accept_receipt = None;
    let mut artifact_dir = None;
    let mut write_receipt = true;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--accept-receipt" => accept_receipt = Some(take_path(&mut args, "--accept-receipt")?),
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--no-write" | "--read-only" => write_receipt = false,
            other => return Err(anyhow!("unexpected overview argument {other:?}")),
        }
    }
    Ok(OverviewArgs {
        workspace: workspace.context("missing --workspace")?,
        item,
        accept_receipt,
        artifact_dir,
        write_receipt,
    })
}

fn parse_readiness_args(args: impl Iterator<Item = String>) -> Result<ReadinessArgs> {
    let mut workspace = None;
    let mut item = None;
    let mut accept_receipt = None;
    let mut artifact_dir = None;
    let mut public_proof = None;
    let mut idunn_lifecycle_receipt = None;
    let mut tool_directory_receipt = None;
    let mut deployment_aftercare_audit_receipt = None;
    let mut deployment_aftercare_audit_receipt_ref = None;
    let mut write_receipt = true;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--accept-receipt" => accept_receipt = Some(take_path(&mut args, "--accept-receipt")?),
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--public-proof" | "--public-proof-ref" => {
                public_proof = Some(take_path(&mut args, "--public-proof")?);
            }
            "--idunn-lifecycle-receipt" | "--service-lifecycle-receipt" => {
                idunn_lifecycle_receipt = Some(take_path(&mut args, "--idunn-lifecycle-receipt")?);
            }
            "--tool-directory-receipt" | "--tool-directory-ref" => {
                tool_directory_receipt = Some(take_path(&mut args, "--tool-directory-receipt")?);
            }
            "--deployment-aftercare-audit-receipt" | "--deployment-aftercare-receipt" => {
                deployment_aftercare_audit_receipt = Some(take_path(
                    &mut args,
                    "--deployment-aftercare-audit-receipt",
                )?);
            }
            "--deployment-aftercare-audit-receipt-ref" | "--deployment-aftercare-receipt-ref" => {
                deployment_aftercare_audit_receipt_ref = Some(take_string(
                    &mut args,
                    "--deployment-aftercare-audit-receipt-ref",
                )?);
            }
            "--no-write" | "--read-only" => write_receipt = false,
            other => return Err(anyhow!("unexpected readiness argument {other:?}")),
        }
    }
    Ok(ReadinessArgs {
        workspace: workspace.context("missing --workspace")?,
        item,
        accept_receipt,
        artifact_dir,
        public_proof,
        idunn_lifecycle_receipt,
        tool_directory_receipt,
        deployment_aftercare_audit_receipt,
        deployment_aftercare_audit_receipt_ref,
        write_receipt,
    })
}

fn parse_deployment_config_audit_args(
    args: impl Iterator<Item = String>,
) -> Result<DeploymentConfigAuditArgs> {
    let mut workspace = None;
    let mut artifact_dir = None;
    let mut write_receipt = true;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--no-write" | "--read-only" => write_receipt = false,
            other => {
                return Err(anyhow!(
                    "unexpected deployment-config-audit argument {other:?}"
                ));
            }
        }
    }
    Ok(DeploymentConfigAuditArgs {
        workspace: workspace.context("missing --workspace")?,
        artifact_dir,
        write_receipt,
    })
}

fn parse_deployment_execution_runbook_args(
    args: impl Iterator<Item = String>,
) -> Result<DeploymentExecutionRunbookArgs> {
    let mut workspace = None;
    let mut artifact_dir = None;
    let mut remote = "origin".to_string();
    let mut write_receipt = true;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--remote" => remote = take_string(&mut args, "--remote")?,
            "--no-write" | "--read-only" => write_receipt = false,
            other => {
                return Err(anyhow!(
                    "unexpected deployment-execution-runbook argument {other:?}"
                ));
            }
        }
    }
    Ok(DeploymentExecutionRunbookArgs {
        workspace: workspace.context("missing --workspace")?,
        artifact_dir,
        remote,
        write_receipt,
    })
}

fn parse_deployment_aftercare_audit_args(
    args: impl Iterator<Item = String>,
) -> Result<DeploymentAftercareAuditArgs> {
    let mut workspace = None;
    let mut artifact_dir = None;
    let mut local_verse_store = None;
    let mut runtime_id = "repo-swarm-local".to_string();
    let mut runbook_receipt = None;
    let mut idunn_deployment_receipt = None;
    let mut idunn_deployment_receipt_ref = None;
    let mut aftercare_audit_receipt = None;
    let mut aftercare_audit_receipt_ref = None;
    let mut write_receipt = true;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--local-verse-store" | "--store" => {
                local_verse_store = Some(take_path(&mut args, "--local-verse-store")?);
            }
            "--runtime-id" => runtime_id = take_string(&mut args, "--runtime-id")?,
            "--runbook-receipt" => {
                runbook_receipt = Some(take_path(&mut args, "--runbook-receipt")?)
            }
            "--idunn-deployment-receipt" | "--deployment-receipt" => {
                idunn_deployment_receipt = Some(take_path(&mut args, "--idunn-deployment-receipt")?)
            }
            "--idunn-deployment-receipt-ref" | "--deployment-receipt-ref" => {
                idunn_deployment_receipt_ref =
                    Some(take_string(&mut args, "--idunn-deployment-receipt-ref")?)
            }
            "--aftercare-audit-receipt" | "--aftercare-receipt" => {
                aftercare_audit_receipt = Some(take_path(&mut args, "--aftercare-audit-receipt")?)
            }
            "--aftercare-audit-receipt-ref" | "--aftercare-receipt-ref" => {
                aftercare_audit_receipt_ref =
                    Some(take_string(&mut args, "--aftercare-audit-receipt-ref")?)
            }
            "--no-write" | "--read-only" => write_receipt = false,
            other => {
                return Err(anyhow!(
                    "unexpected deployment-aftercare-audit argument {other:?}"
                ));
            }
        }
    }
    Ok(DeploymentAftercareAuditArgs {
        workspace: workspace.context("missing --workspace")?,
        artifact_dir,
        local_verse_store,
        runtime_id,
        runbook_receipt,
        idunn_deployment_receipt,
        idunn_deployment_receipt_ref,
        aftercare_audit_receipt,
        aftercare_audit_receipt_ref,
        write_receipt,
    })
}

fn parse_export_proof_args(args: impl Iterator<Item = String>) -> Result<ExportProofArgs> {
    let mut workspace = None;
    let mut item = None;
    let mut accept_receipt = None;
    let mut artifact_dir = None;
    let mut output = None;
    let mut local_verse_store = None;
    let mut runtime_id = "repo-swarm-local".to_string();

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--accept-receipt" => accept_receipt = Some(take_path(&mut args, "--accept-receipt")?),
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--output" | "--export-path" => output = Some(take_path(&mut args, "--output")?),
            "--local-verse-store" | "--store" => {
                local_verse_store = Some(take_path(&mut args, "--local-verse-store")?);
            }
            "--runtime-id" => runtime_id = take_string(&mut args, "--runtime-id")?,
            other => return Err(anyhow!("unexpected export-proof argument {other:?}")),
        }
    }
    Ok(ExportProofArgs {
        workspace: workspace.context("missing --workspace")?,
        item,
        accept_receipt,
        artifact_dir,
        output,
        local_verse_store,
        runtime_id,
    })
}

fn parse_tick_args(args: impl Iterator<Item = String>) -> Result<TickArgs> {
    let mut workspace = None;
    let mut epiphany_root = None;
    let mut item = None;
    let mut local_verse_store = None;
    let mut artifact_dir = None;
    let mut runtime_store = None;
    let mut cooldown_seconds = 0_u64;
    let mut active_timeout_seconds = 900_u64;
    let mut dry_run = false;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--epiphany-root" => epiphany_root = Some(take_path(&mut args, "--epiphany-root")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--local-verse-store" | "--store" => {
                local_verse_store = Some(take_path(&mut args, "--local-verse-store")?);
            }
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--runtime-store" => runtime_store = Some(take_path(&mut args, "--runtime-store")?),
            "--cooldown-seconds" => {
                cooldown_seconds = take_u64(&mut args, "--cooldown-seconds")?;
            }
            "--active-timeout-seconds" => {
                active_timeout_seconds = take_u64(&mut args, "--active-timeout-seconds")?;
            }
            "--dry-run" | "--no-execute" => dry_run = true,
            other => return Err(anyhow!("unexpected tick argument {other:?}")),
        }
    }
    Ok(TickArgs {
        workspace: workspace.context("missing --workspace")?,
        epiphany_root: epiphany_root
            .unwrap_or(env::current_dir().context("failed to resolve current directory")?),
        item,
        local_verse_store,
        artifact_dir,
        runtime_store,
        cooldown_seconds,
        active_timeout_seconds,
        dry_run,
    })
}

fn parse_serve_args(args: impl Iterator<Item = String>) -> Result<ServeArgs> {
    let mut workspace = None;
    let mut epiphany_root = None;
    let mut item = None;
    let mut local_verse_store = None;
    let mut artifact_dir = None;
    let mut runtime_store = None;
    let mut cooldown_seconds = 30_u64;
    let mut active_timeout_seconds = 900_u64;
    let mut dry_run = false;
    let mut scheduler_id = "epiphany-repo-work-scheduler".to_string();
    let mut loop_interval_seconds = 30_u64;
    let mut max_iterations = 0_u64;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--epiphany-root" => epiphany_root = Some(take_path(&mut args, "--epiphany-root")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--local-verse-store" | "--store" => {
                local_verse_store = Some(take_path(&mut args, "--local-verse-store")?);
            }
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--runtime-store" => runtime_store = Some(take_path(&mut args, "--runtime-store")?),
            "--cooldown-seconds" => {
                cooldown_seconds = take_u64(&mut args, "--cooldown-seconds")?;
            }
            "--active-timeout-seconds" => {
                active_timeout_seconds = take_u64(&mut args, "--active-timeout-seconds")?;
            }
            "--scheduler-id" => scheduler_id = take_string(&mut args, "--scheduler-id")?,
            "--loop-interval-seconds" | "--serve-interval-seconds" => {
                loop_interval_seconds = take_u64(&mut args, "--loop-interval-seconds")?;
            }
            "--max-iterations" => {
                max_iterations = take_u64(&mut args, "--max-iterations")?;
            }
            "--dry-run" | "--no-execute" => dry_run = true,
            other => return Err(anyhow!("unexpected serve argument {other:?}")),
        }
    }
    Ok(ServeArgs {
        tick: TickArgs {
            workspace: workspace.context("missing --workspace")?,
            epiphany_root: epiphany_root
                .unwrap_or(env::current_dir().context("failed to resolve current directory")?),
            item,
            local_verse_store,
            artifact_dir,
            runtime_store,
            cooldown_seconds,
            active_timeout_seconds,
            dry_run,
        },
        scheduler_id,
        loop_interval_seconds,
        max_iterations,
    })
}

fn parse_queue_args(args: impl Iterator<Item = String>) -> Result<QueueArgs> {
    let mut workspace = None;
    let mut epiphany_root = None;
    let mut local_verse_store = None;
    let mut artifact_dir = None;
    let mut runtime_store = None;
    let mut runtime_id = "repo-swarm-local".to_string();
    let mut max_items = 1_u64;
    let mut cooldown_seconds = 0_u64;
    let mut active_timeout_seconds = 900_u64;
    let mut dry_run = false;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--epiphany-root" => epiphany_root = Some(take_path(&mut args, "--epiphany-root")?),
            "--local-verse-store" | "--store" => {
                local_verse_store = Some(take_path(&mut args, "--local-verse-store")?);
            }
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--runtime-store" => runtime_store = Some(take_path(&mut args, "--runtime-store")?),
            "--runtime-id" => runtime_id = take_string(&mut args, "--runtime-id")?,
            "--max-items" => max_items = take_u64(&mut args, "--max-items")?,
            "--cooldown-seconds" => {
                cooldown_seconds = take_u64(&mut args, "--cooldown-seconds")?;
            }
            "--active-timeout-seconds" => {
                active_timeout_seconds = take_u64(&mut args, "--active-timeout-seconds")?;
            }
            "--dry-run" | "--no-execute" => dry_run = true,
            other => return Err(anyhow!("unexpected queue-run argument {other:?}")),
        }
    }
    Ok(QueueArgs {
        workspace: workspace.context("missing --workspace")?,
        epiphany_root: epiphany_root
            .unwrap_or(env::current_dir().context("failed to resolve current directory")?),
        local_verse_store,
        artifact_dir,
        runtime_store,
        runtime_id,
        max_items,
        cooldown_seconds,
        active_timeout_seconds,
        dry_run,
    })
}

fn run_accept(args: AcceptArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let epiphany_root = args
        .epiphany_root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.epiphany_root.display()))?;
    let manifest_path = epiphany_root.join("epiphany-core").join("Cargo.toml");
    if !manifest_path.exists() {
        return Err(anyhow!(
            "could not find epiphany-core manifest at {}",
            manifest_path.display()
        ));
    }
    let online_receipt_path = args.online_receipt.unwrap_or_else(|| {
        workspace
            .join(".epiphany")
            .join("swarm-online")
            .join("repo-swarm-online-receipt.json")
    });
    let online_receipt = read_json(&online_receipt_path).with_context(
        || "repo swarm online receipt is required; run epiphany-swarm online first",
    )?;
    let local_verse_store = args.local_verse_store.unwrap_or_else(|| {
        path_from_json(&online_receipt, &["localVerseStore"])
            .unwrap_or_else(|| workspace.join(".epiphany").join("local-verse.ccmp"))
    });
    let runtime_id = args.runtime_id.unwrap_or_else(|| {
        string_from_json(&online_receipt, &["runtimeId"])
            .unwrap_or_else(|| "repo-swarm-local".to_string())
    });
    let artifact_dir = args
        .artifact_dir
        .unwrap_or_else(|| workspace.join(".epiphany").join("work"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;

    let item_slug = sanitize(&args.item);
    let source_kind = if args.source == "persona-or-bifrost" {
        "persona"
    } else {
        args.source.as_str()
    };
    let topic = args
        .topic
        .unwrap_or_else(|| format!("repo-work-item-{item_slug}"));
    let summary = args.summary.unwrap_or_else(|| {
        format!(
            "Accepted repo work item {} from {} for Imagination consensus discovery.",
            args.item, source_kind
        )
    });
    let public_refs = if args.public_discussion_refs.is_empty() {
        vec![format!("repo-work://{runtime_id}/{item_slug}")]
    } else {
        args.public_discussion_refs
    };
    let candidate_refs = if args.candidate_action_refs.is_empty() {
        vec![format!("candidate-action://{runtime_id}/{item_slug}")]
    } else {
        args.candidate_action_refs
    };
    let feedback_id = format!("repo-work-feedback-{item_slug}");
    let consensus_receipt_id = format!("repo-work-consensus-{item_slug}");
    let eve_connection_receipt_id = args
        .eve_connection_receipt_id
        .unwrap_or_else(|| format!("repo-work-eve-connection-{item_slug}"));
    let (source_persona_id, source_cluster_id, public_room_id) = match source_kind {
        "bifrost" => (
            "gamecult.Bifrost".to_string(),
            "gamecult.cluster.bifrost".to_string(),
            "gamecult-local/bifrost/work-items".to_string(),
        ),
        _ => (
            "epiphany.Persona".to_string(),
            "epiphany.cluster.persona".to_string(),
            "epiphany-global/collaboration".to_string(),
        ),
    };
    let mut verse_args = vec![
        "collaboration-feedback".to_string(),
        "--store".to_string(),
        local_verse_store.display().to_string(),
        "--runtime-id".to_string(),
        runtime_id.clone(),
        "--feedback-id".to_string(),
        feedback_id.clone(),
        "--receipt-id".to_string(),
        consensus_receipt_id.clone(),
        "--source-persona-id".to_string(),
        source_persona_id,
        "--source-cluster-id".to_string(),
        source_cluster_id,
        "--public-room-id".to_string(),
        public_room_id,
        "--eve-connection-receipt-id".to_string(),
        eve_connection_receipt_id,
        "--collaboration-topic".to_string(),
        topic.clone(),
        "--feedback-summary".to_string(),
        summary.clone(),
        "--consensus-packet-ref".to_string(),
        format!("gamecult-local/imagination/consensus-packets/{feedback_id}"),
    ];
    for public_ref in &public_refs {
        verse_args.extend(["--public-discussion-ref".to_string(), public_ref.clone()]);
    }
    for candidate_ref in &candidate_refs {
        verse_args.extend(["--candidate-action-ref".to_string(), candidate_ref.clone()]);
    }
    let feedback = cargo_json(&manifest_path, "epiphany-verse-query", &verse_args)?;
    let receipt = json!({
        "schemaVersion": "epiphany.repo_work_accept_receipt.v0",
        "createdAt": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "workspace": workspace,
        "runtimeId": runtime_id,
        "localVerseStore": local_verse_store,
        "onlineReceiptPath": online_receipt_path,
        "source": source_kind,
        "item": args.item,
        "topic": topic,
        "summary": summary,
        "feedback": {
            "feedbackId": feedback["feedbackId"],
            "consensusReceiptId": feedback["consensusReceiptId"],
            "requestedConsensusRoute": feedback["requestedConsensusRoute"],
            "consensusPacketRef": feedback["consensusPacketRef"],
            "adoptionGate": feedback["adoptionGate"],
            "publicDiscussionRefs": feedback["publicDiscussionRefs"],
            "candidateActionRefs": feedback["candidateActionRefs"],
            "privateStateIncluded": feedback["privateStateIncluded"],
            "privateStateExposed": feedback["privateStateExposed"],
        },
        "authority": {
            "status": "accepted-for-imagination-consensus",
            "handsAuthorityGranted": false,
            "durableStateAdmitted": false,
            "publicationAuthorized": false,
            "nextGate": "mind.review_then_bifrost_adoption"
        },
        "nextSafeMove": "Run epiphany-work run --workspace <repo> after Imagination/Self/Mind gates adopt a concrete action plan."
    });
    let receipt_path = artifact_dir.join(format!("work-accept-{item_slug}.json"));
    write_json(&receipt_path, &receipt)?;
    Ok(json!({
        "schemaVersion": "epiphany.repo_work_accept.v0",
        "status": "accepted-for-imagination-consensus",
        "workspace": receipt["workspace"],
        "runtimeId": receipt["runtimeId"],
        "localVerseStore": receipt["localVerseStore"],
        "receiptPath": receipt_path,
        "source": receipt["source"],
        "item": receipt["item"],
        "feedback": receipt["feedback"],
        "authority": receipt["authority"],
        "privateStateExposed": false,
        "nextSafeMove": receipt["nextSafeMove"],
    }))
}

fn run_persona_intake(args: PersonaIntakeArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let epiphany_root = args
        .epiphany_root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.epiphany_root.display()))?;
    let manifest_path = epiphany_root.join("epiphany-core").join("Cargo.toml");
    if !manifest_path.exists() {
        return Err(anyhow!(
            "could not find epiphany-core manifest at {}",
            manifest_path.display()
        ));
    }
    let online_receipt_path = args.online_receipt.clone().unwrap_or_else(|| {
        workspace
            .join(".epiphany")
            .join("swarm-online")
            .join("repo-swarm-online-receipt.json")
    });
    let online_receipt = read_json(&online_receipt_path).with_context(
        || "repo swarm online receipt is required; run epiphany-swarm online first",
    )?;
    let local_verse_store = args.local_verse_store.clone().unwrap_or_else(|| {
        path_from_json(&online_receipt, &["localVerseStore"])
            .unwrap_or_else(|| workspace.join(".epiphany").join("local-verse.ccmp"))
    });
    let state_dir = path_from_json(&online_receipt, &["stateDir"])
        .unwrap_or_else(|| workspace.join(".epiphany").join("state"));
    let agent_store = state_dir.join("agents.msgpack");
    let runtime_id = args.runtime_id.clone().unwrap_or_else(|| {
        string_from_json(&online_receipt, &["runtimeId"])
            .unwrap_or_else(|| "repo-swarm-local".to_string())
    });
    let artifact_dir = args
        .artifact_dir
        .clone()
        .unwrap_or_else(|| workspace.join(".epiphany").join("persona-intake"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;

    let item_slug = sanitize(&args.item);
    let persona = cargo_json(
        &manifest_path,
        "epiphany-persona-discord",
        &[
            "bubble".to_string(),
            "--artifact-dir".to_string(),
            artifact_dir.display().to_string(),
            "--cultmesh-store".to_string(),
            local_verse_store.display().to_string(),
            "--runtime-id".to_string(),
            runtime_id.clone(),
            "--content".to_string(),
            args.message.clone(),
            "--source".to_string(),
            "epiphany/Persona/repo-intake".to_string(),
            "--status".to_string(),
            "accepted-for-imagination-consensus".to_string(),
            "--mood".to_string(),
            args.mood.clone(),
        ],
    )?;
    let speech_audit = persona
        .get("speechAudit")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let audit_id = string_from_json(&speech_audit, &["auditId"])
        .unwrap_or_else(|| format!("persona-speech-audit-{item_slug}"));
    let bubble_path = string_from_json(&persona, &["bubblePath"]).unwrap_or_default();
    let content_fingerprint =
        string_from_json(&speech_audit, &["contentFingerprint"]).unwrap_or_default();
    let topic = args
        .topic
        .clone()
        .unwrap_or_else(|| format!("repo-persona-intake-{item_slug}"));
    let summary = compact_text(&args.message, 480);
    let memory_recall =
        record_repo_persona_intake_memory_recall(&agent_store, &item_slug, &args.message);
    let public_ref = format!("eve://epiphany/persona#repo-intake/{item_slug}/{audit_id}");
    let candidate_ref = format!("candidate-action://{runtime_id}/{item_slug}/{audit_id}");
    let weksa = record_repo_persona_intake_weksa(
        &local_verse_store,
        &runtime_id,
        &item_slug,
        &audit_id,
        &args.message,
        &public_ref,
    )?;
    let accept = run_accept(AcceptArgs {
        workspace: workspace.clone(),
        epiphany_root: epiphany_root.clone(),
        source: "persona".to_string(),
        item: args.item.clone(),
        summary: Some(summary.clone()),
        topic: Some(topic.clone()),
        local_verse_store: Some(local_verse_store.clone()),
        artifact_dir: Some(workspace.join(".epiphany").join("work")),
        runtime_id: Some(runtime_id.clone()),
        online_receipt: Some(online_receipt_path.clone()),
        eve_connection_receipt_id: Some(format!("repo-persona-intake-eve-{item_slug}")),
        public_discussion_refs: vec![public_ref.clone()],
        candidate_action_refs: vec![candidate_ref.clone()],
    })?;
    let receipt = json!({
        "schemaVersion": "epiphany.repo_persona_intake_receipt.v0",
        "createdAt": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "workspace": workspace,
        "runtimeId": runtime_id,
        "localVerseStore": local_verse_store,
        "onlineReceiptPath": online_receipt_path,
        "source": "persona",
        "item": args.item,
        "topic": topic,
        "messageSummary": summary,
        "persona": {
            "bubblePath": bubble_path,
            "speechAudit": speech_audit,
            "contentFingerprint": content_fingerprint,
            "publicDiscussionRef": public_ref,
            "candidateActionRef": candidate_ref,
            "memoryRecall": memory_recall,
            "weksa": weksa
        },
        "accept": accept,
        "authority": {
            "status": "accepted-for-imagination-consensus",
            "personaSpeechAudited": true,
            "handsAuthorityGranted": false,
            "durableStateAdmitted": false,
            "publicationAuthorized": false,
            "privateStateExposed": false
        },
        "nextSafeMove": "Run epiphany-work derive-plan --workspace <repo> --item <id> after Imagination forms a concrete safe-family plan."
    });
    let receipt_path = workspace
        .join(".epiphany")
        .join("work")
        .join(format!("work-persona-intake-{item_slug}.json"));
    write_json(&receipt_path, &receipt)?;
    Ok(json!({
        "schemaVersion": "epiphany.repo_persona_intake.v0",
        "status": "accepted-for-imagination-consensus",
        "workspace": receipt["workspace"],
        "runtimeId": receipt["runtimeId"],
        "localVerseStore": receipt["localVerseStore"],
        "receiptPath": receipt_path,
        "item": receipt["item"],
        "speechAuditId": audit_id,
        "memoryRecallStatus": receipt["persona"]["memoryRecall"]["status"],
        "memoryRecallCacheStatus": receipt["persona"]["memoryRecall"]["cacheStatus"],
        "memoryRecallHitCount": receipt["persona"]["memoryRecall"]["hitCount"],
        "interpreterDynamicRecallStatus": receipt["persona"]["memoryRecall"]["interpreterDynamicRecall"]["status"],
        "interpreterDynamicRecallCacheStatus": receipt["persona"]["memoryRecall"]["interpreterDynamicRecall"]["cacheStatus"],
        "interpreterDynamicRecallHitCount": receipt["persona"]["memoryRecall"]["interpreterDynamicRecall"]["hitCount"],
        "weksaLoweringReceiptId": receipt["persona"]["weksa"]["receiptId"],
        "bubblePath": receipt["persona"]["bubblePath"],
        "acceptReceiptPath": accept["receiptPath"],
        "feedback": accept["feedback"],
        "authority": receipt["authority"],
        "privateStateExposed": false,
        "nextSafeMove": receipt["nextSafeMove"],
    }))
}

fn record_repo_persona_intake_memory_recall(
    agent_store: &Path,
    item_slug: &str,
    message: &str,
) -> Value {
    let entry = match load_agent_memory_entry_for_role(agent_store, "Persona") {
        Ok(Some(entry)) => entry,
        Ok(None) => {
            return json!({
                "schemaVersion": "epiphany.persona_memory_cache.v0",
                "status": "unavailable",
                "cacheStatus": "persona-memory-missing",
                "identityId": "epiphany.Persona",
                "roleId": "Persona",
                "chunkCount": 0,
                "hitCount": 0,
                "renderedRecall": "- semantic Persona memory recall unavailable: Persona memory entry is missing",
                "warnings": ["Persona memory entry is missing"],
                "privateStateExposed": false,
            });
        }
        Err(error) => {
            return json!({
                "schemaVersion": "epiphany.persona_memory_cache.v0",
                "status": "unavailable",
                "cacheStatus": "persona-memory-load-failed",
                "identityId": "epiphany.Persona",
                "roleId": "Persona",
                "chunkCount": 0,
                "hitCount": 0,
                "renderedRecall": format!("- semantic Persona memory recall unavailable: {}", compact_text(&format!("{error:#}"), 320)),
                "warnings": [format!("Persona memory load failed: {}", compact_text(&format!("{error:#}"), 240))],
                "privateStateExposed": false,
            });
        }
    };

    let query = format!("Repo Persona intake {item_slug}\n{message}");
    let graph = memory_graph_from_agent_memories(
        "repo-persona-intake-memory",
        std::slice::from_ref(&entry),
    );
    let fallback = plan_memory_graph_context_cut(
        &graph,
        &EpiphanyMemoryContextQuery {
            id: format!("repo-persona-intake-{item_slug}"),
            profile: Some(EpiphanyMemoryProfile::RoleSelf),
            domain_ids: Vec::new(),
            node_ids: Vec::new(),
            edge_ids: Vec::new(),
            text: Some(query.clone()),
            budget: Some(8),
        },
    );
    let mut config = PersonaMemoryCacheConfig::from_env();
    config.qdrant_timeout_ms = config.qdrant_timeout_ms.min(1_000);
    config.ollama_timeout_ms = config.ollama_timeout_ms.min(1_000);
    let recall = render_persona_memory_recall_with_cache(
        &entry,
        format!("{}#Persona", agent_store.display()),
        &query,
        8,
        Some(&fallback),
        &config,
    );
    let synthetic_persona_prompt = format!(
        "Repo Persona intake message:\n{}\n\nInitial semantic memory recall:\n{}",
        message, recall.rendered_recall
    );
    let dynamic_recall = epiphany_core::render_dynamic_persona_memory_recall_for_output(
        &entry,
        format!("{}#Persona", agent_store.display()),
        &synthetic_persona_prompt,
        message,
        &recall.rendered_recall,
        8,
        Some(&fallback),
        &config,
    );

    json!({
        "schemaVersion": recall.schema_version,
        "status": recall.status,
        "cacheStatus": recall.cache_status,
        "identityId": recall.identity_id,
        "roleId": recall.role_id,
        "chunkCount": recall.chunk_count,
        "hitCount": recall.hit_count,
        "renderedRecall": recall.rendered_recall,
        "interpreterDynamicRecall": {
            "schemaVersion": dynamic_recall.schema_version,
            "status": dynamic_recall.status,
            "cacheStatus": dynamic_recall.cache_status,
            "identityId": dynamic_recall.identity_id,
            "roleId": dynamic_recall.role_id,
            "chunkCount": dynamic_recall.chunk_count,
            "hitCount": dynamic_recall.hit_count,
            "renderedRecall": dynamic_recall.rendered_recall,
            "warnings": dynamic_recall.warnings,
            "queryBasis": "candidate-say-meaning",
            "authority": {
                "scaffoldedFromOperatorMessage": true,
                "fullAutonomousPersonaTurn": false,
                "durableStateAdmitted": false,
                "publicationAuthorized": false,
                "privateStateExposed": false
            },
            "privateStateExposed": dynamic_recall.private_state_exposed
        },
        "warnings": recall.warnings,
        "privateStateExposed": recall.private_state_exposed,
    })
}

fn record_repo_persona_intake_weksa(
    local_verse_store: &Path,
    runtime_id: &str,
    item_slug: &str,
    audit_id: &str,
    message: &str,
    public_ref: &str,
) -> Result<Value> {
    let audit_slug = sanitize(audit_id);
    let packet_id = format!("weksa-packet-repo-intake-{item_slug}-{audit_slug}");
    let request_id = format!("weksa-lower-repo-intake-{item_slug}-{audit_slug}");
    let receipt_id = format!("weksa-lowering-repo-intake-{item_slug}-{audit_slug}");
    let delivery_surface = "eve://epiphany/persona";
    let target_language = "en";
    let target_register = "repo-public-intake";
    let packet = build_weksa_interlingua_packet(WeksaInterlinguaInput {
        packet_id: packet_id.clone(),
        source_interpreter_ref: format!("persona-intake:{runtime_id}:{item_slug}:{audit_slug}"),
        source_speech_audit_ref: audit_id.to_string(),
        speaker: WeksaSpeakerContext {
            persona_id: "epiphany.Persona".to_string(),
            display_name: "Epiphany".to_string(),
            source_surface: delivery_surface.to_string(),
            source_language: "en".to_string(),
            utterance_state_ref: "state/agents.msgpack#Persona:utterance-state".to_string(),
        },
        meaning: message.to_string(),
        speech_act: "repo-intake-say".to_string(),
        delivery_register: target_register.to_string(),
        target_audience: "repo-public-room".to_string(),
        safety_notes: vec![
            "Do not claim Hands, Bifrost, publication, merge, deployment, or service lifecycle authority.".to_string(),
            "Do not expose private worker thought, raw result payloads, or private Verse state.".to_string(),
        ],
    })?;
    let request = build_weksa_target_lowering_request(
        request_id.clone(),
        packet,
        target_language,
        target_register,
        delivery_surface,
    )?;
    let receipt = record_weksa_target_lowering_receipt(
        &request,
        receipt_id.clone(),
        message.to_string(),
        "repo-persona-intake-local-text-lowering",
    )?;
    let lowered_text_preview = compact_text(&receipt.lowered_text, 480);
    let cultmesh_receipt = EpiphanyCultMeshWeksaLoweringReceiptEntry {
        schema_version: EPIPHANY_CULTMESH_WEKSA_LOWERING_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: receipt.receipt_id.clone(),
        runtime_id: runtime_id.to_string(),
        verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
        packet_id: receipt.packet_id.clone(),
        request_id: receipt.request_id.clone(),
        persona_agent_id: request.packet.speaker.persona_id.clone(),
        target_language: receipt.target_language.clone(),
        target_register: receipt.target_register.clone(),
        delivery_surface: receipt.delivery_surface.clone(),
        lowering_method: receipt.lowering_method.clone(),
        transport_authority: receipt.transport_authority.clone(),
        publication_authorized: false,
        lowered_text_ref: public_ref.to_string(),
        lowered_text_preview,
        created_at_utc: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        private_state_exposed: false,
        notes: vec![
            "Repo Persona intake routed public SAY through Weksa interlingua/lowering before any mouth transport publication.".to_string(),
        ],
    };
    write_epiphany_cultmesh_weksa_lowering_receipt(local_verse_store, cultmesh_receipt)?;

    Ok(json!({
        "schemaVersion": "epiphany.repo_persona_intake_weksa.v0",
        "packetSchemaVersion": request.packet.schema_version,
        "requestSchemaVersion": request.schema_version,
        "receiptSchemaVersion": receipt.schema_version,
        "cultmeshSchemaVersion": EPIPHANY_CULTMESH_WEKSA_LOWERING_RECEIPT_SCHEMA_VERSION,
        "packetId": packet_id,
        "requestId": request_id,
        "receiptId": receipt_id,
        "sourceSpeechAuditRef": audit_id,
        "targetLanguage": receipt.target_language,
        "targetRegister": receipt.target_register,
        "deliverySurface": receipt.delivery_surface,
        "loweringMethod": receipt.lowering_method,
        "transportAuthority": receipt.transport_authority,
        "loweredTextRef": public_ref,
        "modelRequired": request.model_required,
        "publicationAuthorized": false,
        "privateStateExposed": false,
    }))
}

fn run_work(args: RunArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let _epiphany_root = args
        .epiphany_root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.epiphany_root.display()))?;
    let accept_receipt_path =
        resolve_accept_receipt(&workspace, args.item.as_deref(), args.accept_receipt)?;
    let accept_receipt = read_json(&accept_receipt_path)?;
    let online_receipt_path = path_from_json(&accept_receipt, &["onlineReceiptPath"])
        .unwrap_or_else(|| {
            workspace
                .join(".epiphany")
                .join("swarm-online")
                .join("repo-swarm-online-receipt.json")
        });
    let online_receipt = read_json(&online_receipt_path)
        .with_context(|| "repo swarm online receipt is required before work run")?;
    let runtime_id = string_from_json(&accept_receipt, &["runtimeId"])
        .or_else(|| string_from_json(&online_receipt, &["runtimeId"]))
        .unwrap_or_else(|| "repo-swarm-local".to_string());
    let state_dir = path_from_json(&online_receipt, &["stateDir"])
        .unwrap_or_else(|| workspace.join(".epiphany").join("state"));
    let runtime_store = args
        .runtime_store
        .unwrap_or_else(|| state_dir.join("runtime-spine.msgpack"));
    let artifact_dir = args
        .artifact_dir
        .unwrap_or_else(|| workspace.join(".epiphany").join("work"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    if let Some(parent) = runtime_store.parent() {
        fs::create_dir_all(parent)?;
    }
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    initialize_runtime_spine(
        &runtime_store,
        RuntimeSpineInitOptions {
            runtime_id: runtime_id.clone(),
            display_name: format!("Repo swarm runtime for {}", workspace.display()),
            created_at: now.clone(),
        },
    )?;

    let item = accept_receipt
        .get("item")
        .and_then(Value::as_str)
        .unwrap_or("work-item")
        .to_string();
    let item_slug = sanitize(&item);
    let requested_paths = if args.requested_paths.is_empty() {
        vec![".".to_string()]
    } else {
        args.requested_paths
    };
    let run_id = format!("repo-work-run-{item_slug}");
    let runtime_job_id = format!("{run_id}-job");
    let substrate_grant_id = format!("{run_id}-substrate-grant");
    let intent_id = format!("{run_id}-hands-intent");
    let review_id = format!("{run_id}-hands-review");

    let substrate_grant = substrate_gate_repo_work_planning_grant(
        substrate_grant_id.clone(),
        runtime_job_id.clone(),
        requested_paths.clone(),
        now.clone(),
    );
    put_substrate_gate_repo_access_grant_receipt(&runtime_store, &substrate_grant)?;

    let intent = HandsActionIntent {
        schema_version: HANDS_ACTION_INTENT_SCHEMA_VERSION.to_string(),
        intent_id: intent_id.clone(),
        runtime_job_id: runtime_job_id.clone(),
        binding_id: "repo-work-runner".to_string(),
        role: "epiphany-hands".to_string(),
        authority_scope: "repo.branch_local_work".to_string(),
        requested_action: "runAcceptedWorkItem".to_string(),
        requested_paths: requested_paths.clone(),
        substrate_gate_grant_receipt_id: substrate_grant_id.clone(),
        requested_at: now.clone(),
        contract: "Repo work run intent converts accepted Persona/Bifrost pressure into a bounded Hands gate; this gate is not mutation authority until reviewed as approved.".to_string(),
    };
    put_hands_action_intent(&runtime_store, &intent)?;

    let review = hands_action_review_for_intent(
        review_id.clone(),
        &intent,
        "queued-for-adoption".to_string(),
        vec!["plan".to_string()],
        vec![
            "Accepted work item has entered Imagination consensus discovery.".to_string(),
            "Hands mutation remains blocked until Imagination/Self/Mind adopt a concrete plan and approve this gate.".to_string(),
        ],
        now.clone(),
    );
    put_hands_action_review(&runtime_store, &review)?;

    let gate = json!({
        "intentId": intent_id,
        "reviewId": review_id,
        "runtimeJobId": runtime_job_id,
        "substrateGateGrantReceiptId": substrate_grant_id,
        "decision": review.decision,
        "allowedOperations": review.allowed_operations,
        "recordPassCommand": format!(
            "epiphany-hands-action --store {} record-pass --intent-id {} --review-id {} --summary <summary> --changed-path <path> --command <command> --exit-code <code> --commit-sha <sha>",
            runtime_store.display(),
            intent.intent_id,
            review.review_id
        )
    });
    let receipt = json!({
        "schemaVersion": "epiphany.repo_work_run_receipt.v0",
        "createdAt": now,
        "workspace": workspace,
        "runtimeId": runtime_id,
        "runtimeStore": runtime_store,
        "acceptReceiptPath": accept_receipt_path,
        "onlineReceiptPath": online_receipt_path,
        "item": item,
        "status": "queued-for-adoption",
        "authority": {
            "handsAuthorityGranted": false,
            "durableStateAdmitted": false,
            "publicationAuthorized": false,
            "mutationBlockedBy": "hands.review.decision != approved",
            "nextGate": "imagination.self.mind.adoption"
        },
        "handsActionGate": gate,
        "nextSafeMove": "Promote this queued gate only after Imagination/Self/Mind adopt a concrete plan; epiphany-hands-action will refuse record-pass until the Hands review decision is approved."
    });
    let receipt_path = artifact_dir.join(format!("work-run-{item_slug}.json"));
    write_json(&receipt_path, &receipt)?;
    Ok(json!({
        "schemaVersion": "epiphany.repo_work_run.v0",
        "status": "queued-for-adoption",
        "workspace": receipt["workspace"],
        "runtimeId": receipt["runtimeId"],
        "runtimeStore": receipt["runtimeStore"],
        "receiptPath": receipt_path,
        "item": receipt["item"],
        "handsActionGate": receipt["handsActionGate"],
        "authority": receipt["authority"],
        "privateStateExposed": false,
        "nextSafeMove": receipt["nextSafeMove"],
    }))
}

fn run_plan(args: PlanArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let accept_receipt_path =
        resolve_accept_receipt(&workspace, args.item.as_deref(), args.accept_receipt)?;
    let accept_receipt = read_json(&accept_receipt_path)?;
    let artifact_dir = args
        .artifact_dir
        .unwrap_or_else(|| workspace.join(".epiphany").join("work"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;

    write_plan_receipt(
        workspace,
        accept_receipt_path,
        &accept_receipt,
        artifact_dir,
        PlanReceiptInputs {
            objective: args.objective,
            plan_summary: args.plan_summary,
            command: args.command,
            changed_paths: args.changed_paths,
            commit_message: args.commit_message,
            adoption_evidence_refs: args.adoption_evidence_refs,
            verification_asks: args.verification_asks,
            stop_conditions: args.stop_conditions,
            rollback_hints: args.rollback_hints,
            derivation: None,
        },
    )
}

fn run_derive_plan(args: DerivePlanArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let accept_receipt_path =
        resolve_accept_receipt(&workspace, args.item.as_deref(), args.accept_receipt)?;
    let accept_receipt = read_json(&accept_receipt_path)?;
    let artifact_dir = args
        .artifact_dir
        .unwrap_or_else(|| workspace.join(".epiphany").join("work"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;

    let item = accept_receipt
        .get("item")
        .and_then(Value::as_str)
        .unwrap_or("work-item")
        .to_string();
    let summary = accept_receipt
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or("Accepted repo work item.");
    let source = accept_receipt
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("persona");
    let objective = format!(
        "Respond to {source} work item {item}: {}",
        compact_line(summary)
    );
    let mut evidence_refs = vec![format!("accept-receipt:{}", accept_receipt_path.display())];
    if let Some(consensus_id) =
        string_from_json(&accept_receipt, &["feedback", "consensusReceiptId"])
    {
        evidence_refs.push(format!("imagination-consensus:{consensus_id}"));
    }
    let model_ref = args.model_ref;
    let model_authored = args.model_authored || model_ref.is_some();
    if let Some(model_ref) = model_ref.as_ref() {
        evidence_refs.push(format!("imagination-model:{model_ref}"));
    } else {
        evidence_refs.push(format!(
            "imagination-derivation:deterministic-{}.v0",
            normalize_action_family(&args.action_family)?
        ));
    }
    let mut derived_plan = derive_safe_plan_family(DeriveSafePlanInput {
        action_family: &args.action_family,
        target_path: args.target_path.as_deref(),
        item: &item,
        summary,
        source,
        accept_receipt: &accept_receipt,
        model_ref: model_ref.as_deref(),
        model_authored,
    })?;
    let action_verification_asks = if args.verification_asks.is_empty() {
        derived_plan.verification_asks.clone()
    } else {
        args.verification_asks
    };
    let action_stop_conditions = if args.stop_conditions.is_empty() {
        vec![
            "Stop if the command exits nonzero or changes paths outside the derived plan."
                .to_string(),
        ]
    } else {
        args.stop_conditions
    };
    let action_escalation_reasons = if args.escalation_reasons.is_empty() {
        vec![
            "Escalate if the accepted pressure requires paths outside the requested safe family."
                .to_string(),
            "Escalate if the work needs publication, merge, service lifecycle, elevation, or cross-repo authority.".to_string(),
        ]
    } else {
        args.escalation_reasons
    };
    let planning_facets = PlanningFacets::for_derive_plan(
        source,
        &derived_plan.safe_action_family,
        &derived_plan.target_path,
        args.assumptions,
        args.constraints,
        args.non_goals,
        args.open_questions,
        args.decision_points,
        args.evidence_needs,
    );
    let action_item_summary = args
        .action_summary
        .unwrap_or_else(|| derived_plan.plan_summary.clone());
    let action_items = write_imagination_action_items_receipt(
        &artifact_dir,
        &workspace,
        &accept_receipt_path,
        &accept_receipt,
        ImaginationActionItemReceiptInputs {
            item: &item,
            source,
            summary,
            derive_plan_mode: &normalize_action_family(&args.action_family)?,
            safe_action_family: &derived_plan.safe_action_family,
            requested_paths: vec![derived_plan.target_path.clone()],
            action_summary: &action_item_summary,
            verification_asks: action_verification_asks.clone(),
            stop_conditions: action_stop_conditions.clone(),
            escalation_reasons: action_escalation_reasons,
            rollback_hints: derived_plan.rollback_hints.clone(),
            planning_facets: planning_facets.clone(),
            model_ref: model_ref.as_deref(),
            model_authored,
        },
    )?;
    let action_items_receipt_id = string_from_json(&action_items, &["receiptId"])
        .unwrap_or_else(|| format!("repo-work-action-items-{}", sanitize(&item)));
    evidence_refs.push(format!(
        "imagination-action-items:{action_items_receipt_id}"
    ));
    derived_plan.derivation["actionItemReceipt"] = json!({
        "receiptId": action_items_receipt_id,
        "receiptPath": action_items["receiptPath"],
        "schemaVersion": action_items["schemaVersion"],
        "status": action_items["status"],
        "modelAuthored": model_authored,
        "safeActionFamily": derived_plan.safe_action_family,
        "requestedPaths": [derived_plan.target_path.clone()],
        "verificationAsks": action_verification_asks.clone(),
        "planningFacets": planning_facets.to_json(),
    });

    write_plan_receipt(
        workspace,
        accept_receipt_path,
        &accept_receipt,
        artifact_dir,
        PlanReceiptInputs {
            objective,
            plan_summary: derived_plan.plan_summary,
            command: derived_plan.command,
            changed_paths: vec![derived_plan.target_path.clone()],
            commit_message: derived_plan.commit_message,
            adoption_evidence_refs: evidence_refs,
            verification_asks: action_verification_asks,
            stop_conditions: action_stop_conditions,
            rollback_hints: derived_plan.rollback_hints,
            derivation: Some(derived_plan.derivation),
        },
    )
}

struct DeriveSafePlanInput<'a> {
    action_family: &'a str,
    target_path: Option<&'a str>,
    item: &'a str,
    summary: &'a str,
    source: &'a str,
    accept_receipt: &'a Value,
    model_ref: Option<&'a str>,
    model_authored: bool,
}

struct DerivedSafePlan {
    safe_action_family: String,
    target_path: String,
    plan_summary: String,
    command: String,
    commit_message: String,
    verification_asks: Vec<String>,
    rollback_hints: Vec<String>,
    derivation: Value,
}

fn derive_safe_plan_family(input: DeriveSafePlanInput<'_>) -> Result<DerivedSafePlan> {
    let action_family = normalize_action_family(input.action_family)?;
    match action_family.as_str() {
        "append-worklog" => derive_append_worklog_plan(input, &action_family),
        "planning-note" => derive_planning_note_plan(input, &action_family),
        "checklist-note" => derive_checklist_note_plan(input, &action_family),
        "section-note" | "markdown-section" | "section-update" => {
            derive_section_note_plan(input, &action_family)
        }
        "repo-status-section" | "status-section" | "readme-status" => {
            derive_repo_status_section_plan(input, &action_family)
        }
        "task-card" | "action-card" | "plan-card" => derive_task_card_plan(input, &action_family),
        "repo-tool-request"
        | "tool-request"
        | "daemon-tool-request"
        | "repo-daemon-tool-request" => derive_repo_tool_request_plan(input, &action_family),
        "repo-collaboration-policy" | "collaboration-policy" | "repo-collab-policy" => {
            derive_repo_collaboration_policy_plan(input, &action_family)
        }
        "repo-collaboration-topic" | "collaboration-topic" | "eve-collaboration" => {
            derive_repo_collaboration_topic_plan(input, &action_family)
        }
        "repo-consensus-brief" | "consensus-brief" | "imagination-consensus" => {
            derive_repo_consensus_brief_plan(input, &action_family)
        }
        "repo-interpreter-brief" | "interpreter-brief" | "mind-interpreter-brief" => {
            derive_repo_interpreter_brief_plan(input, &action_family)
        }
        "repo-objective-draft" | "objective-draft" | "imagination-objective" => {
            derive_repo_objective_draft_plan(input, &action_family)
        }
        "repo-adoption-request" | "adoption-request" | "mind-adoption-request" => {
            derive_repo_adoption_request_plan(input, &action_family)
        }
        "repo-scheduling-request" | "scheduling-request" | "self-scheduling-request" => {
            derive_repo_scheduling_request_plan(input, &action_family)
        }
        "repo-work-order" | "work-order" | "hands-work-order" => {
            derive_repo_work_order_plan(input, &action_family)
        }
        "repo-verification-request" | "verification-request" | "soul-verification-request" => {
            derive_repo_verification_request_plan(input, &action_family)
        }
        "repo-publication-request" | "publication-request" | "bifrost-publication-request" => {
            derive_repo_publication_request_plan(input, &action_family)
        }
        "repo-sync-request" | "sync-request" | "upstream-sync-request" => {
            derive_repo_sync_request_plan(input, &action_family)
        }
        "repo-maintainer-review-request" | "maintainer-review-request" | "review-request" => {
            derive_repo_maintainer_review_request_plan(input, &action_family)
        }
        "repo-pr-request" | "pr-request" | "pull-request-request" | "github-pr-request" => {
            derive_repo_pr_request_plan(input, &action_family)
        }
        "repo-credit-request" | "credit-request" | "bifrost-credit-request" => {
            derive_repo_credit_request_plan(input, &action_family)
        }
        "repo-artifact-acceptance-request"
        | "artifact-acceptance-request"
        | "accepted-artifact-request" => {
            derive_repo_artifact_acceptance_request_plan(input, &action_family)
        }
        "repo-metrics-request" | "metrics-request" | "accounting-request" => {
            derive_repo_metrics_request_plan(input, &action_family)
        }
        "repo-readiness-review-request" | "readiness-review-request" | "mvp-readiness-request" => {
            derive_repo_readiness_review_request_plan(input, &action_family)
        }
        "repo-doctrine-update-request"
        | "doctrine-update-request"
        | "agents-update-request"
        | "repo-agents-request" => derive_repo_doctrine_update_request_plan(input, &action_family),
        "repo-secret-policy-request"
        | "secret-policy-request"
        | "security-policy-request"
        | "repo-security-request" => derive_repo_secret_policy_request_plan(input, &action_family),
        "repo-dependency-policy-request"
        | "dependency-policy-request"
        | "dependency-policy"
        | "supply-chain-policy-request"
        | "repo-supply-chain-request" => {
            derive_repo_dependency_policy_request_plan(input, &action_family)
        }
        "repo-deployment-config"
        | "deployment-config"
        | "idunn-deployment-config"
        | "repo-deploy-config" => derive_repo_deployment_config_plan(input, &action_family),
        "repo-deployment-request"
        | "deployment-request"
        | "idunn-deployment-request"
        | "repo-deploy-request" => derive_repo_deployment_request_plan(input, &action_family),
        other => Err(anyhow!(
            "unsupported derive-plan action family {other:?}; supported families are append-worklog, planning-note, checklist-note, section-note, repo-status-section, task-card, repo-tool-request, repo-collaboration-policy, repo-collaboration-topic, repo-consensus-brief, repo-interpreter-brief, repo-objective-draft, repo-adoption-request, repo-scheduling-request, repo-work-order, repo-verification-request, repo-publication-request, repo-sync-request, repo-maintainer-review-request, repo-pr-request, repo-credit-request, repo-artifact-acceptance-request, repo-metrics-request, repo-readiness-review-request, repo-doctrine-update-request, repo-secret-policy-request, repo-dependency-policy-request, repo-deployment-config, and repo-deployment-request"
        )),
    }
}

fn derive_append_worklog_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let target_path =
        validate_plan_target_path(input.target_path.unwrap_or("EPIPHANY_WORKLOG.md"))?;
    let worklog_line = format!(
        "- {} [{}]: {}",
        Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        input.item,
        compact_line(input.summary)
    );
    let command = format!(
        "Add-Content -LiteralPath {} -Value {}",
        powershell_single_quoted(&target_path),
        powershell_single_quoted(&worklog_line)
    );
    Ok(DerivedSafePlan {
        safe_action_family: "repo.append_worklog".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a safe append-only worklog update from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Record repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the worklog path changed and the appended line matches the accepted pressure.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the appended worklog line if the pressure was misinterpreted.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.append_worklog"),
    })
}

fn derive_planning_note_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let default_target = format!("notes/epiphany-work/{}.md", sanitize(input.item));
    let target_path = validate_plan_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let lines = vec![
        format!("# Epiphany Work Note: {}", compact_line(input.item)),
        String::new(),
        format!("- Created: {}", Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
        format!("- Source: {}", compact_line(input.source)),
        format!("- Summary: {}", compact_line(input.summary)),
        format!("- Candidate action refs: {}", compact_join(&candidate_refs)),
        format!("- Public discussion refs: {}", compact_join(&public_refs)),
        String::new(),
        "## Imagination Plan".to_string(),
        String::new(),
        "- Safe action family: planning-note".to_string(),
        "- Intended consequence: preserve a concrete planning note for Self adoption and later Hands work.".to_string(),
        "- Authority seal: this note is branch-local planning cargo, not publication, merge, or durable Mind admission.".to_string(),
        String::new(),
    ];
    let command = powershell_append_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.markdown_planning_note".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a contained markdown planning note from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add planning note for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the planning note path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies no paths outside the declared planning note changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated planning note if the accepted pressure was misinterpreted.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.markdown_planning_note"),
    })
}

fn derive_checklist_note_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let default_target = format!("notes/epiphany-work/{}-checklist.md", sanitize(input.item));
    let target_path = validate_plan_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let lines = vec![
        format!("# Epiphany Work Checklist: {}", compact_line(input.item)),
        String::new(),
        format!(
            "- Created: {}",
            Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        ),
        format!("- Source: {}", compact_line(input.source)),
        format!("- Summary: {}", compact_line(input.summary)),
        format!("- Candidate action refs: {}", compact_join(&candidate_refs)),
        format!("- Public discussion refs: {}", compact_join(&public_refs)),
        String::new(),
        "## Checklist".to_string(),
        String::new(),
        "- [ ] Confirm the accepted pressure is represented without private-state leakage."
            .to_string(),
        "- [ ] Identify the branch-local files that a later Hands pass may lawfully change."
            .to_string(),
        "- [ ] Name the Soul check that would prove the next implementation pass.".to_string(),
        "- [ ] Escalate to Bifrost/Mind instead of mutating publication, merge, service, or cross-repo state.".to_string(),
        String::new(),
        "## Authority".to_string(),
        String::new(),
        "- Safe action family: checklist-note".to_string(),
        "- Intended consequence: preserve an operator-safe checklist for Self and later Hands work.".to_string(),
        "- Authority seal: this checklist is branch-local planning cargo, not execution authority.".to_string(),
        String::new(),
    ];
    let command = powershell_append_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.checklist_note".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a contained markdown checklist from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add checklist note for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the checklist note path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies checklist items preserve branch-local scope and do not grant publication, merge, service, or cross-repo authority.".to_string(),
            "Soul verifies no paths outside the declared checklist note changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated checklist note if the accepted pressure was misinterpreted.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.checklist_note"),
    })
}

fn derive_section_note_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!("notes/epiphany-work/{item_slug}-section.md");
    let target_path = validate_markdown_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let section_name = format!("epiphany:{item_slug}");
    let start_marker = format!("<!-- epiphany-section:{item_slug}:start -->");
    let end_marker = format!("<!-- epiphany-section:{item_slug}:end -->");
    let lines = vec![
        start_marker.clone(),
        format!("## Epiphany Managed Section: {}", compact_line(input.item)),
        String::new(),
        format!(
            "- Updated: {}",
            Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        ),
        format!("- Source: {}", compact_line(input.source)),
        format!("- Summary: {}", compact_line(input.summary)),
        format!("- Candidate action refs: {}", compact_join(&candidate_refs)),
        format!("- Public discussion refs: {}", compact_join(&public_refs)),
        String::new(),
        "### Proposed Branch-Local Consequence".to_string(),
        String::new(),
        "- Convert the accepted pressure into a bounded repo-local documentation section."
            .to_string(),
        "- Preserve the section markers so later Imagination passes update this same surface instead of appending duplicate state.".to_string(),
        "- Escalate if the work requires code mutation, publication, merge, service lifecycle, elevation, or cross-repo authority.".to_string(),
        String::new(),
        "### Verification".to_string(),
        String::new(),
        "- Soul verifies only the declared markdown path changed.".to_string(),
        "- Soul verifies the managed section contains the accepted pressure summary and both Epiphany section markers.".to_string(),
        String::new(),
        "### Authority".to_string(),
        String::new(),
        "- Safe action family: section-note".to_string(),
        "- Section id: ".to_string() + &section_name,
        "- Authority seal: this is branch-local documentation cargo, not durable Mind admission or publication.".to_string(),
        end_marker.clone(),
        String::new(),
    ];
    let command = powershell_replace_managed_section_command(
        &target_path,
        &start_marker,
        &end_marker,
        &lines,
    );
    Ok(DerivedSafePlan {
        safe_action_family: "repo.markdown_managed_section".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a bounded managed markdown section from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Update managed section for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the managed markdown section path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies both Epiphany section markers are present so later runs update the same bounded section.".to_string(),
            "Soul verifies no paths outside the declared markdown section target changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the managed section between its Epiphany markers if the accepted pressure was misinterpreted.".to_string(),
            "Restore the prior marked section from git if a later section update regressed the note.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.markdown_managed_section"),
    })
}

fn derive_repo_status_section_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let target_path = validate_markdown_target_path(input.target_path.unwrap_or("README.md"))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let section_name = format!("epiphany-status:{item_slug}");
    let start_marker = format!("<!-- epiphany-status:{item_slug}:start -->");
    let end_marker = format!("<!-- epiphany-status:{item_slug}:end -->");
    let lines = vec![
        start_marker.clone(),
        format!("## Epiphany Status: {}", compact_line(input.item)),
        String::new(),
        format!(
            "- Updated: {}",
            Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        ),
        format!("- Source: {}", compact_line(input.source)),
        format!("- Summary: {}", compact_line(input.summary)),
        format!("- Candidate action refs: {}", compact_join(&candidate_refs)),
        format!("- Public discussion refs: {}", compact_join(&public_refs)),
        String::new(),
        "### Repo-Visible Consequence".to_string(),
        String::new(),
        "- Publish an operator-safe status section inside the owned repo Body.".to_string(),
        "- Keep the section bounded by Epiphany status markers so later passes update the same surface instead of duplicating it.".to_string(),
        "- Escalate if the accepted pressure requires code mutation, publication, merge, deployment, service lifecycle, elevation, or cross-repo authority.".to_string(),
        String::new(),
        "### Verification".to_string(),
        String::new(),
        "- Soul verifies only the declared README/status markdown path changed.".to_string(),
        "- Soul verifies the status section contains the accepted pressure summary and both Epiphany status markers.".to_string(),
        "- Soul verifies the section preserves private-state and publication seals.".to_string(),
        String::new(),
        "### Authority".to_string(),
        String::new(),
        "- Safe action family: repo-status-section".to_string(),
        "- Section id: ".to_string() + &section_name,
        "- Branch local only: true".to_string(),
        "- Publication authorized: false".to_string(),
        "- Merge authorized: false".to_string(),
        "- Service lifecycle authority: false".to_string(),
        "- Cross-repo mutation: false".to_string(),
        "- Private state exposed: false".to_string(),
        end_marker.clone(),
        String::new(),
    ];
    let command = powershell_replace_managed_section_command(
        &target_path,
        &start_marker,
        &end_marker,
        &lines,
    );
    Ok(DerivedSafePlan {
        safe_action_family: "repo.status_section".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a repo-visible status section from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Update repo status section for work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo status section path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies both Epiphany status markers are present so later runs update the same bounded section.".to_string(),
            "Soul verifies no paths outside the declared status section target changed.".to_string(),
            "Soul verifies the section does not grant publication, merge, service lifecycle, elevation, cross-repo mutation, or private-state exposure.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the managed status section between its Epiphany markers if the accepted pressure was misinterpreted.".to_string(),
            "Restore the prior marked status section from git if a later status update regressed the repo-facing surface.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.status_section"),
    })
}

fn derive_task_card_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!("notes/epiphany-work/{item_slug}-task-card.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let lines = vec![
        "# Epiphany repo work task card.".to_string(),
        "# Branch-local planning cargo; not publication, merge, service, or cross-repo authority."
            .to_string(),
        format!("schema_version = {}", toml_basic_string("epiphany.repo_work_task_card.v0")),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.task_card")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[scope]".to_string(),
        format!("target_path = {}", toml_basic_string(&target_path)),
        "branch_local_only = true".to_string(),
        "requires_epiphany_branch = true".to_string(),
        String::new(),
        "[next_action]".to_string(),
        "owner = \"Self\"".to_string(),
        "gate = \"Mind\"".to_string(),
        "summary = \"Adopt one bounded task card before Hands lowers an executable command.\""
            .to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the task card path changed and contains the accepted pressure summary.\","
            .to_string(),
        "  \"Soul verifies the task card preserves branch-local authority seals and does not grant publication, merge, service lifecycle, elevation, or cross-repo authority.\","
            .to_string(),
        "  \"Soul verifies no paths outside the declared task card changed.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the task card if the accepted pressure was misinterpreted.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.task_card".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a structured task card from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add task card for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the task card path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the structured task card preserves branch-local scope and all authority seals.".to_string(),
            "Soul verifies no paths outside the declared task card changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated task card if the accepted pressure was misinterpreted.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.task_card"),
    })
}

fn derive_repo_tool_request_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/tool-requests/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request_id = format!("repo-tool-request:{item_slug}");
    let requester_body = format!("repo:{item_slug}");
    let intent_contract = "epiphany.cultmesh.daemon_tool_invocation_intent.v0";
    let receipt_contract = "epiphany.cultmesh.daemon_tool_invocation_receipt.v0";
    let lines = vec![
        "# Epiphany repo daemon tool request.".to_string(),
        "# Branch-local request cargo; CultMesh carries intent and receipts, host daemons own execution.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_tool_request.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.tool_request")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[request]".to_string(),
        format!("id = {}", toml_basic_string(&request_id)),
        format!("requester_body = {}", toml_basic_string(&requester_body)),
        "requesting_agent = \"repo Persona/Self\"".to_string(),
        "target_directory = \"gamecult-local/daemon-tool-directory\"".to_string(),
        "target_capability = \"daemon-tool-capability:selected-by-review\"".to_string(),
        "operation = \"submitTypedToolIntent\"".to_string(),
        String::new(),
        "[cultmesh]".to_string(),
        format!(
            "intent_contract = {}",
            toml_basic_string(intent_contract)
        ),
        format!(
            "receipt_contract = {}",
            toml_basic_string(receipt_contract)
        ),
        "host_daemon_owns_execution = true".to_string(),
        "requester_owns_request = false".to_string(),
        "requires_host_liveness_ready = true".to_string(),
        "requires_cultmesh_receipts = true".to_string(),
        String::new(),
        "[odin]".to_string(),
        "discoverable = true".to_string(),
        "preserves_provider_ownership = true".to_string(),
        "private_verse_passthrough = false".to_string(),
        String::new(),
        "[authority]".to_string(),
        "direct_tool_execution = false".to_string(),
        "arbitrary_shell_authority = false".to_string(),
        "hands_action_authority = false".to_string(),
        "state_commit_authority = false".to_string(),
        "publication_authority = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authority = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the tool request path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the request names CultMesh intent and receipt contracts without executing the tool.\",".to_string(),
        "  \"Soul verifies host daemon ownership, Odin provider ownership, and private-state seals remain intact.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the repo tool request if the accepted pressure was misderived or targets the wrong daemon capability.\"]".to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.tool_request".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a daemon-hosted tool request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add repo tool request for work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo tool request path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the request names CultMesh typed invocation contracts while leaving execution with host daemons.".to_string(),
            "Soul verifies no execution, lifecycle, publication, cross-body, or private-rummaging authority was granted.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated repo tool request if the accepted pressure was misinterpreted or targets the wrong daemon capability.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.tool_request"),
    })
}

fn derive_repo_collaboration_policy_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = ".epiphany/collaboration-policy.toml".to_string();
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let surface_id = format!("eve://epiphany/repo/{item_slug}/collaboration-policy");
    let feedback_route = format!("imagination://repo/{item_slug}/consensus-discovery");
    let lines = vec![
        "# Epiphany repo collaboration policy.".to_string(),
        "# Branch-local policy cargo; it describes collaboration law without granting action authority.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_collaboration_policy.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.collaboration_policy")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[policy]".to_string(),
        "status = \"proposed\"".to_string(),
        "authoring_owner = \"Imagination\"".to_string(),
        "required_reviewers = [\"Persona\", \"Mind\"]".to_string(),
        "policy_admitted = false".to_string(),
        "publication_owner = \"Bifrost\"".to_string(),
        String::new(),
        "[body]".to_string(),
        format!("domain = {}", toml_basic_string(&format!("repo:{item_slug}"))),
        "provider_owns_truth = true".to_string(),
        "renderer_owns_truth = false".to_string(),
        String::new(),
        "[verses]".to_string(),
        "private = \"epiphany-internal\"".to_string(),
        "local = \"gamecult-local\"".to_string(),
        "public = \"epiphany-global\"".to_string(),
        "private_state_may_leave_repo = false".to_string(),
        "public_projection_allowed = true".to_string(),
        "local_projection_allowed = true".to_string(),
        "odin_discoverable = true".to_string(),
        String::new(),
        "[eve]".to_string(),
        format!("surface = {}", toml_basic_string(&surface_id)),
        "compact_tui_required = true".to_string(),
        "connection_receipt_required = true".to_string(),
        "supported_actions = [\"read-queue\", \"discuss\", \"submit-feedback\"]".to_string(),
        String::new(),
        "[persona]".to_string(),
        "public_discussion_allowed = true".to_string(),
        "human_discussion_allowed = true".to_string(),
        "peer_persona_discussion_allowed = true".to_string(),
        "speech_audit_required = true".to_string(),
        "feedback_must_route_to_imagination = true".to_string(),
        String::new(),
        "[imagination]".to_string(),
        format!("feedback_route = {}", toml_basic_string(&feedback_route)),
        "consensus_required_before_adoption = true".to_string(),
        "candidate_actions_non_authoritative = true".to_string(),
        "mind_adoption_required = true".to_string(),
        "bifrost_publication_required = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "direct_hands_authority = false".to_string(),
        "direct_mind_state_commit = false".to_string(),
        "direct_publication_authority = false".to_string(),
        "direct_merge_authority = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authority = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "requires_cultmesh_receipts = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the collaboration policy path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the policy names private/local/public Verse boundaries, Odin discovery, Eve connection receipts, Persona discussion, and Imagination feedback routing without granting action authority.\",".to_string(),
        "  \"Soul verifies no paths outside the declared collaboration policy changed.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the collaboration policy if the accepted pressure was misderived.\"]".to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.collaboration_policy".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a repo collaboration policy from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add repo collaboration policy for work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo collaboration policy path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the policy names Verse boundaries, Odin/Eve discovery, Persona discussion, and Imagination consensus routing while denying Hands, Mind, publication, merge, service, and cross-body authority.".to_string(),
            "Soul verifies no paths outside the declared collaboration policy changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated repo collaboration policy if the accepted pressure was misinterpreted.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.collaboration_policy"),
    })
}

fn derive_repo_collaboration_topic_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/collaboration-topics/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let topic_id = format!("repo-collaboration:{item_slug}");
    let public_room = format!("epiphany-global/persona-collaboration/{item_slug}");
    let eve_surface = format!("eve://epiphany/repo/{item_slug}/collaboration");
    let consensus_route = format!("imagination://repo/{item_slug}/consensus-discovery");
    let lines = vec![
        "# Epiphany repo collaboration topic.".to_string(),
        "# Branch-local discussion cargo; not adoption, action, publication, or private-state authority.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_collaboration_topic.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.collaboration_topic")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[topic]".to_string(),
        format!("id = {}", toml_basic_string(&topic_id)),
        "status = \"proposed\"".to_string(),
        "authoring_owner = \"Imagination\"".to_string(),
        "discussion_owner = \"Persona\"".to_string(),
        "publication_owner = \"Bifrost\"".to_string(),
        format!("requested_public_room = {}", toml_basic_string(&public_room)),
        format!("requested_eve_surface = {}", toml_basic_string(&eve_surface)),
        "public_room_published = false".to_string(),
        "eve_surface_published = false".to_string(),
        "provider_receipt_required = true".to_string(),
        "persona_discussion_allowed = true".to_string(),
        "human_discussion_allowed = true".to_string(),
        "agent_friendly_tui = true".to_string(),
        String::new(),
        "[imagination]".to_string(),
        format!("consensus_route = {}", toml_basic_string(&consensus_route)),
        "consensus_required_before_action = true".to_string(),
        "candidate_actions_are_non_authoritative = true".to_string(),
        "mind_adoption_required = true".to_string(),
        "bifrost_publication_required = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "adoption_authorized = false".to_string(),
        "hands_action_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the collaboration topic path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the topic names a public room, Eve surface, and Imagination consensus route without granting action authority.\",".to_string(),
        "  \"Soul verifies no paths outside the declared collaboration topic changed.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the collaboration topic manifest if the accepted pressure was misderived.\"]".to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.collaboration_topic".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a repo collaboration topic from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add collaboration topic for work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo collaboration topic path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the topic routes public Persona/human discussion to Imagination consensus without granting Hands, Mind, publication, merge, service, or cross-body authority.".to_string(),
            "Soul verifies no paths outside the declared collaboration topic changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated collaboration topic manifest if the accepted pressure was misinterpreted.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.collaboration_topic"),
    })
}

fn derive_repo_consensus_brief_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/consensus-briefs/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let brief_id = format!("repo-consensus:{item_slug}");
    let topic_ref = public_refs
        .first()
        .cloned()
        .unwrap_or_else(|| format!("epiphany-global/persona-collaboration/{item_slug}"));
    let recommended_family = "repo.task_card";
    let lines = vec![
        "# Epiphany repo consensus brief.".to_string(),
        "# Branch-local Imagination cargo; not objective adoption, Hands authority, or publication."
            .to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_consensus_brief.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.consensus_brief")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[consensus]".to_string(),
        format!("id = {}", toml_basic_string(&brief_id)),
        format!("topic_ref = {}", toml_basic_string(&topic_ref)),
        "status = \"draft\"".to_string(),
        "converged = false".to_string(),
        "conflicts_remaining = true".to_string(),
        "requires_human_or_persona_review = true".to_string(),
        format!(
            "recommended_next_safe_family = {}",
            toml_basic_string(recommended_family)
        ),
        String::new(),
        "[imagination]".to_string(),
        "role = \"consensus-discovery\"".to_string(),
        "candidate_actions_non_authoritative = true".to_string(),
        "may_emit_action_items_receipt = true".to_string(),
        "must_preserve_public_refs = true".to_string(),
        "must_not_read_private_verses = true".to_string(),
        String::new(),
        "[inputs]".to_string(),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        "feedback_source = \"Persona public discussion\"".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "objective_adoption_authorized = false".to_string(),
        "hands_action_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "mind_adoption_required = true".to_string(),
        "bifrost_publication_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the consensus brief path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies candidate actions remain non-authoritative and require Mind/Bifrost gates before consequence.\",".to_string(),
        "  \"Soul verifies no paths outside the declared consensus brief changed.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the consensus brief if the public feedback was misderived.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.consensus_brief".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a repo consensus brief from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add consensus brief for work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo consensus brief path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the brief keeps candidate actions non-authoritative and requires Mind/Bifrost gates before consequence.".to_string(),
            "Soul verifies no paths outside the declared consensus brief changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated consensus brief if the public feedback was misinterpreted.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.consensus_brief"),
    })
}

fn derive_repo_interpreter_brief_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/interpreter-briefs/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let brief_id = format!("repo-interpreter-brief:{item_slug}");
    let candidate_families = vec![
        "repo.consensus_brief".to_string(),
        "repo.objective_draft".to_string(),
        "repo.adoption_request".to_string(),
        "repo.work_order".to_string(),
        "repo.verification_request".to_string(),
        "repo.publication_request".to_string(),
    ];
    let lines = vec![
        "# Epiphany repo interpreter brief.".to_string(),
        "# Branch-local Mind interpretation cargo; not objective adoption, scheduling, Hands authority, or publication.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_interpreter_brief.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.interpreter_brief")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "deployment_execution_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[interpreter]".to_string(),
        format!("id = {}", toml_basic_string(&brief_id)),
        "status = \"awaiting-mind-interpretation\"".to_string(),
        "authoring_owner = \"Imagination\"".to_string(),
        "requested_interpreter = \"Mind\"".to_string(),
        "interpretation_admitted = false".to_string(),
        "purpose = \"public-pressure-to-action-semantics\"".to_string(),
        "requires_consensus_readback = true".to_string(),
        "requires_safe_family_choice = true".to_string(),
        "requires_requested_paths = true".to_string(),
        "requires_verification_asks = true".to_string(),
        "requires_evidence_needs = true".to_string(),
        "candidate_actions_non_authoritative = true".to_string(),
        String::new(),
        "[inputs]".to_string(),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        "private_worker_transcripts_allowed = false".to_string(),
        "raw_result_payloads_allowed = false".to_string(),
        String::new(),
        "[semantic_checks]".to_string(),
        "intent_summary_required = true".to_string(),
        "scope_boundary_required = true".to_string(),
        "requested_paths_required = true".to_string(),
        "verification_required = true".to_string(),
        "evidence_required = true".to_string(),
        "rollback_required = true".to_string(),
        "non_goals_required = true".to_string(),
        "open_questions_required = true".to_string(),
        "consensus_alignment_required = true".to_string(),
        String::new(),
        "[allowed_outputs]".to_string(),
        format!("candidate_safe_families = {}", toml_array(&candidate_families)),
        "may_request_replanning = true".to_string(),
        "may_request_more_consensus = true".to_string(),
        "may_adopt_objective = false".to_string(),
        "may_schedule_work = false".to_string(),
        "may_touch_substrate = false".to_string(),
        "may_publish = false".to_string(),
        "may_deploy = false".to_string(),
        String::new(),
        "[required_gates]".to_string(),
        "imagination_consensus_required = true".to_string(),
        "mind_review_required = true".to_string(),
        "soul_source_grounding_required = true".to_string(),
        "bifrost_publication_review_required = true".to_string(),
        "hands_receipt_required_before_state_change = true".to_string(),
        "substrate_receipt_required_before_mutation = true".to_string(),
        "idunn_receipt_required_before_deployment = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "direct_state_commit_authorized = false".to_string(),
        "objective_adoption_authorized = false".to_string(),
        "self_scheduling_authorized = false".to_string(),
        "substrate_access_authorized = false".to_string(),
        "hands_action_authorized = false".to_string(),
        "shell_command_authorized = false".to_string(),
        "commit_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "deployment_execution_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_worker_transcripts_allowed = false".to_string(),
        "raw_result_payloads_allowed = false".to_string(),
        "private_state_exposed = false".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the interpreter brief path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies Imagination authored a non-authoritative request for Mind interpretation.\",".to_string(),
        "  \"Soul verifies semantic checks, required gates, and authority denials are present.\",".to_string(),
        "  \"Soul verifies no paths outside the declared interpreter brief changed.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the interpreter brief if Imagination misrepresented the consensus or the semantic checks are incomplete.\"]".to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.interpreter_brief".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a request for Mind interpretation from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add interpreter brief for work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo interpreter brief path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the brief keeps interpretation non-authoritative and requires consensus, semantic checks, Mind review, Soul grounding, and downstream receipts.".to_string(),
            "Soul verifies no paths outside the declared interpreter brief changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated interpreter brief if Imagination misrepresented the consensus.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.interpreter_brief"),
    })
}

fn derive_repo_objective_draft_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/objective-drafts/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let draft_id = format!("repo-objective:{item_slug}");
    let lines = vec![
        "# Epiphany repo Objective Draft.".to_string(),
        "# Branch-local Imagination cargo; not an adopted objective or Hands command."
            .to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_objective_draft.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.objective_draft")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[draft]".to_string(),
        format!("id = {}", toml_basic_string(&draft_id)),
        "status = \"review-required\"".to_string(),
        "owner = \"Imagination\"".to_string(),
        "adoption_gate = \"Mind\"".to_string(),
        "scheduler_gate = \"Self\"".to_string(),
        "publication_gate = \"Bifrost\"".to_string(),
        "objective_adopted = false".to_string(),
        String::new(),
        "[objective]".to_string(),
        format!("title = {}", toml_basic_string(&compact_line(input.item))),
        format!("statement = {}", toml_basic_string(&compact_line(input.summary))),
        "scope = \"repo-local branch work proposal\"".to_string(),
        "preferred_next_safe_family = \"repo.task_card\"".to_string(),
        String::new(),
        "[acceptance]".to_string(),
        "criteria = [".to_string(),
        "  \"Mind explicitly accepts or rejects this Objective Draft before Self schedules it.\",".to_string(),
        "  \"Self schedules only after Mind adoption and a safe-family action plan exist.\",".to_string(),
        "  \"Hands acts only through a later receipt-backed plan and declared path scope.\",".to_string(),
        "  \"Bifrost gates publication, credit, and upstream-main sync.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[inputs]".to_string(),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        "consensus_brief_required = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "objective_adoption_authorized = false".to_string(),
        "self_scheduling_authorized = false".to_string(),
        "hands_action_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "mind_adoption_required = true".to_string(),
        "bifrost_publication_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the Objective Draft path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the draft is review-required and not adopted.\",".to_string(),
        "  \"Soul verifies no paths outside the declared Objective Draft changed.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the Objective Draft if the consensus was misderived.\"]".to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.objective_draft".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a repo Objective Draft from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add Objective Draft for work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo Objective Draft path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the draft remains review-required, unadopted, and gated by Mind/Self/Bifrost before consequence.".to_string(),
            "Soul verifies no paths outside the declared Objective Draft changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated Objective Draft if the consensus was misinterpreted.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.objective_draft"),
    })
}

fn derive_repo_adoption_request_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/adoption-requests/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request_id = format!("repo-adoption-request:{item_slug}");
    let objective_draft_ref = format!(".epiphany/objective-drafts/{item_slug}.toml");
    let lines = vec![
        "# Epiphany repo adoption request.".to_string(),
        "# Branch-local Mind review cargo; not state admission or scheduling authority.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_adoption_request.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.adoption_request")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[request]".to_string(),
        format!("id = {}", toml_basic_string(&request_id)),
        "status = \"awaiting-mind-review\"".to_string(),
        "requested_decision = \"adopt-or-refuse-objective-draft\"".to_string(),
        format!(
            "objective_draft_ref = {}",
            toml_basic_string(&objective_draft_ref)
        ),
        "mind_review_required = true".to_string(),
        "mind_state_commit_required = true".to_string(),
        "self_scheduling_after_mind_only = true".to_string(),
        String::new(),
        "[decision_contract]".to_string(),
        "allowed_verdicts = [\"adopted\", \"refused\", \"needs-more-consensus\"]".to_string(),
        "requires_review_finding = true".to_string(),
        "requires_receipt = \"epiphany.mind.gateway_review\"".to_string(),
        "requires_commit_receipt_if_adopted = \"epiphany.mind.state_commit_receipt\"".to_string(),
        "does_not_modify_state = true".to_string(),
        String::new(),
        "[inputs]".to_string(),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        "objective_draft_required = true".to_string(),
        "consensus_brief_required = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "objective_adoption_authorized = false".to_string(),
        "state_commit_authorized = false".to_string(),
        "self_scheduling_authorized = false".to_string(),
        "hands_action_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the adoption request path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the request is awaiting Mind review and cannot commit state by itself.\",".to_string(),
        "  \"Soul verifies no paths outside the declared adoption request changed.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the adoption request if the Objective Draft should not be reviewed yet.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.adoption_request".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a Mind adoption request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add adoption request for work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo adoption request path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the request awaits Mind review and does not authorize state commit, scheduling, Hands action, publication, or cross-body mutation.".to_string(),
            "Soul verifies no paths outside the declared adoption request changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated adoption request if the Objective Draft is not ready for Mind review.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.adoption_request"),
    })
}

fn derive_repo_scheduling_request_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/scheduling-requests/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request_id = format!("repo-scheduling-request:{item_slug}");
    let objective_draft_ref = format!(".epiphany/objective-drafts/{item_slug}.toml");
    let adoption_request_ref = format!(".epiphany/adoption-requests/{item_slug}.toml");
    let lines = vec![
        "# Epiphany repo scheduling request.".to_string(),
        "# Branch-local Self queue cargo; inert until Mind adoption receipts exist.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_scheduling_request.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.scheduling_request")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[request]".to_string(),
        format!("id = {}", toml_basic_string(&request_id)),
        "status = \"awaiting-mind-adoption\"".to_string(),
        "requested_scheduler = \"Self\"".to_string(),
        format!(
            "objective_draft_ref = {}",
            toml_basic_string(&objective_draft_ref)
        ),
        format!(
            "adoption_request_ref = {}",
            toml_basic_string(&adoption_request_ref)
        ),
        "mind_adoption_receipt_required = true".to_string(),
        "self_may_schedule_after_mind_only = true".to_string(),
        "queue_run_allowed_after_adoption = true".to_string(),
        String::new(),
        "[queue]".to_string(),
        "target_gate = \"repo-work-queue\"".to_string(),
        "preferred_next_safe_family = \"repo.task_card\"".to_string(),
        "max_items_per_pulse = 1".to_string(),
        "requires_epiphany_branch = true".to_string(),
        "publish_blocker = \"bifrost-publication-missing\"".to_string(),
        String::new(),
        "[required_receipts]".to_string(),
        "mind_review = \"epiphany.mind.gateway_review\"".to_string(),
        "mind_commit = \"epiphany.mind.state_commit_receipt\"".to_string(),
        "expected_self_receipt = \"epiphany.repo_work_queue_selection.v0\"".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "self_scheduling_authorized = false".to_string(),
        "queue_mutation_authorized = false".to_string(),
        "hands_action_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "mind_adoption_required = true".to_string(),
        "bifrost_publication_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the scheduling request path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the request awaits Mind adoption before Self may schedule.\",".to_string(),
        "  \"Soul verifies the request grants no queue mutation, Hands action, publication, or cross-body authority.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the scheduling request if Mind has not adopted the Objective Draft.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.scheduling_request".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a Self scheduling request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add scheduling request for work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo scheduling request path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the request awaits Mind adoption and does not authorize queue mutation, Hands action, publication, or cross-body mutation.".to_string(),
            "Soul verifies no paths outside the declared scheduling request changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated scheduling request if the Objective Draft is not adopted by Mind.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.scheduling_request"),
    })
}

fn derive_repo_work_order_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/work-orders/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let work_order_id = format!("repo-work-order:{item_slug}");
    let objective_draft_ref = format!(".epiphany/objective-drafts/{item_slug}.toml");
    let adoption_request_ref = format!(".epiphany/adoption-requests/{item_slug}.toml");
    let scheduling_request_ref = format!(".epiphany/scheduling-requests/{item_slug}.toml");
    let lines = vec![
        "# Epiphany repo work order request.".to_string(),
        "# Branch-local Hands intent cargo; not a command, patch, or commit.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_work_order.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.work_order")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[work_order]".to_string(),
        format!("id = {}", toml_basic_string(&work_order_id)),
        "status = \"awaiting-hands-review\"".to_string(),
        "requested_owner = \"Hands\"".to_string(),
        "requested_effect = \"branch-local-implementation\"".to_string(),
        "implementation_intent = \"prepare a reviewable branch-local change from the adopted objective\"".to_string(),
        "subgoal = \"derive the smallest coherent implementation slice that satisfies the accepted pressure\"".to_string(),
        String::new(),
        "[antecedents]".to_string(),
        format!(
            "objective_draft_ref = {}",
            toml_basic_string(&objective_draft_ref)
        ),
        format!(
            "adoption_request_ref = {}",
            toml_basic_string(&adoption_request_ref)
        ),
        format!(
            "scheduling_request_ref = {}",
            toml_basic_string(&scheduling_request_ref)
        ),
        "mind_adoption_required = true".to_string(),
        "self_queue_selection_required = true".to_string(),
        String::new(),
        "[required_receipts]".to_string(),
        "substrate_gate = \"epiphany.substrate_gate.grant\"".to_string(),
        "hands_intent = \"epiphany.hands.action_intent\"".to_string(),
        "hands_review = \"epiphany.hands.action_review\"".to_string(),
        "hands_patch = \"epiphany.hands.patch_receipt\"".to_string(),
        "hands_command = \"epiphany.hands.command_receipt\"".to_string(),
        "hands_commit = \"epiphany.hands.commit_receipt\"".to_string(),
        "soul_verdict = \"epiphany.soul.verification_verdict\"".to_string(),
        "mind_commit = \"epiphany.mind.state_commit_receipt\"".to_string(),
        String::new(),
        "[scope]".to_string(),
        "branch_required = true".to_string(),
        "allowed_branch_prefix = \"epiphany/\"".to_string(),
        "requested_paths = [\"README.md\", \"notes/epiphany-work/\"]".to_string(),
        "max_changed_paths = 3".to_string(),
        "requires_reviewable_diff = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "substrate_access_authorized = false".to_string(),
        "hands_action_authorized = false".to_string(),
        "shell_command_authorized = false".to_string(),
        "commit_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "bifrost_publication_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the work order path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the work order awaits Hands review and grants no substrate, shell, commit, or publication authority.\",".to_string(),
        "  \"Soul verifies the requested scope is branch-local, bounded, and receipt-backed.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the work order if the adopted objective is not ready for Hands review.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.work_order".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a Hands work-order request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add work order for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo work order path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the work order awaits Hands review and does not authorize substrate access, shell commands, commits, publication, or cross-body mutation.".to_string(),
            "Soul verifies no paths outside the declared work order changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated work order if the accepted pressure is not ready for Hands review.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.work_order"),
    })
}

fn derive_repo_verification_request_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/verification-requests/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request_id = format!("repo-verification-request:{item_slug}");
    let work_order_ref = format!(".epiphany/work-orders/{item_slug}.toml");
    let lines = vec![
        "# Epiphany repo verification request.".to_string(),
        "# Branch-local Soul request cargo; not a verdict or Mind admission.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_verification_request.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.verification_request")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "soul_verdict_granted = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[request]".to_string(),
        format!("id = {}", toml_basic_string(&request_id)),
        "status = \"awaiting-soul-review\"".to_string(),
        "requested_owner = \"Soul\"".to_string(),
        "requested_effect = \"verify-branch-local-hands-work\"".to_string(),
        format!("work_order_ref = {}", toml_basic_string(&work_order_ref)),
        "verification_scope = \"declared changed paths, Hands receipts, and visible repo diff\""
            .to_string(),
        String::new(),
        "[antecedents]".to_string(),
        "substrate_gate_required = true".to_string(),
        "hands_intent_required = true".to_string(),
        "hands_review_required = true".to_string(),
        "hands_patch_required = true".to_string(),
        "hands_command_required = true".to_string(),
        "hands_commit_required = true".to_string(),
        "work_order_required = true".to_string(),
        String::new(),
        "[required_receipts]".to_string(),
        "hands_patch = \"epiphany.hands.patch_receipt\"".to_string(),
        "hands_command = \"epiphany.hands.command_receipt\"".to_string(),
        "hands_commit = \"epiphany.hands.commit_receipt\"".to_string(),
        "soul_verdict = \"epiphany.soul.verification_verdict\"".to_string(),
        "closure_review = \"epiphany.repo_work_closure_review.v0\"".to_string(),
        "mind_review = \"epiphany.mind.gateway_review\"".to_string(),
        "mind_commit = \"epiphany.mind.state_commit_receipt\"".to_string(),
        String::new(),
        "[checks]".to_string(),
        "required = [".to_string(),
        "  \"declared-paths-match-commit\",".to_string(),
        "  \"hands-receipts-present\",".to_string(),
        "  \"visible-diff-supports-summary\",".to_string(),
        "  \"no-private-state-exposure\",".to_string(),
        "  \"publication-remains-gated\"".to_string(),
        "]".to_string(),
        "model_verdict_allowed = true".to_string(),
        "failure_blocks_mind_admission = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "soul_verdict_authorized = false".to_string(),
        "state_commit_authorized = false".to_string(),
        "hands_action_authorized = false".to_string(),
        "rerun_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "bifrost_publication_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the verification request path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the request names required Hands evidence but does not itself grant a verdict.\",".to_string(),
        "  \"Soul verifies the request grants no rerun, state commit, publication, or cross-body authority.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the verification request if no Hands consequence is ready for Soul review.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.verification_request".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a Soul verification request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add verification request for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo verification request path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the request names required Hands evidence and closure checks without authorizing a verdict, rerun, state commit, publication, or cross-body mutation.".to_string(),
            "Soul verifies no paths outside the declared verification request changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated verification request if there is no Hands consequence ready for Soul review.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.verification_request"),
    })
}

fn derive_repo_publication_request_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/publication-requests/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request_id = format!("repo-publication-request:{item_slug}");
    let verification_request_ref = format!(".epiphany/verification-requests/{item_slug}.toml");
    let lines = vec![
        "# Epiphany repo publication request.".to_string(),
        "# Branch-local Bifrost request cargo; not publication, merge, or sync authority.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_publication_request.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.publication_request")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "upstream_sync_authorized = false".to_string(),
        "credit_authorized = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[request]".to_string(),
        format!("id = {}", toml_basic_string(&request_id)),
        "status = \"awaiting-bifrost-review\"".to_string(),
        "requested_owner = \"Bifrost\"".to_string(),
        "requested_effect = \"publish-redacted-proof-and-route-maintainer-review\"".to_string(),
        format!(
            "verification_request_ref = {}",
            toml_basic_string(&verification_request_ref)
        ),
        "publication_scope = \"redacted public proof, maintainer review, credit ledger, and upstream-main sync proof\"".to_string(),
        String::new(),
        "[antecedents]".to_string(),
        "closure_review_required = true".to_string(),
        "soul_verdict_required = true".to_string(),
        "mind_commit_required = true".to_string(),
        "public_proof_export_required = true".to_string(),
        "private_state_redaction_required = true".to_string(),
        String::new(),
        "[required_receipts]".to_string(),
        "closure_review = \"epiphany.repo_work_closure_review.v0\"".to_string(),
        "soul_verdict = \"epiphany.soul.verification_verdict\"".to_string(),
        "mind_commit = \"epiphany.mind.state_commit_receipt\"".to_string(),
        "public_proof = \"epiphany.repo_work_public_proof_bundle.v0\"".to_string(),
        "bifrost_publication = \"gamecult.bifrost.public_proof_publication_receipt.v0\""
            .to_string(),
        "github_publication = \"gamecult.github.publication_receipt.v0\"".to_string(),
        "credit_ledger = \"gamecult.bifrost.credit_receipt.v0\"".to_string(),
        "upstream_sync = \"epiphany.repo_work_upstream_main_sync.v0\"".to_string(),
        String::new(),
        "[public_export]".to_string(),
        "redacted_only = true".to_string(),
        "raw_receipts_allowed = false".to_string(),
        "private_paths_allowed = false".to_string(),
        "worker_thought_allowed = false".to_string(),
        "operator_context_allowed = false".to_string(),
        "credit_required = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "bifrost_publication_authorized = false".to_string(),
        "github_publication_authorized = false".to_string(),
        "credit_ledger_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "upstream_sync_authorized = false".to_string(),
        "hands_action_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "maintainer_review_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the publication request path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the request names public proof and Bifrost receipt requirements without authorizing publication.\",".to_string(),
        "  \"Soul verifies raw receipts, private paths, worker thought, and operator context remain excluded from public export.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the publication request if the work is not ready for Bifrost review.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.publication_request".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a Bifrost publication request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add publication request for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo publication request path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the request requires redacted public proof and Bifrost/GitHub/credit/upstream receipts without authorizing publication, merge, sync, service lifecycle, or cross-body mutation.".to_string(),
            "Soul verifies no paths outside the declared publication request changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated publication request if the work is not ready for Bifrost review.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.publication_request"),
    })
}

fn derive_repo_sync_request_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/sync-requests/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request_id = format!("repo-sync-request:{item_slug}");
    let publication_request_ref = format!(".epiphany/publication-requests/{item_slug}.toml");
    let lines = vec![
        "# Epiphany repo upstream-main sync request.".to_string(),
        "# Branch-local sync proof request cargo; not merge, push, or sync authority.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_sync_request.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.sync_request")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "merge_authorized = false".to_string(),
        "push_authorized = false".to_string(),
        "upstream_sync_authorized = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[request]".to_string(),
        format!("id = {}", toml_basic_string(&request_id)),
        "status = \"awaiting-upstream-main-proof\"".to_string(),
        "proof_owner = \"Bifrost\"".to_string(),
        "requested_effect = \"prove-published-commit-contained-by-upstream-main\"".to_string(),
        format!(
            "publication_request_ref = {}",
            toml_basic_string(&publication_request_ref)
        ),
        "sync_scope = \"prove maintainer-reviewed published commit is contained by upstream main\""
            .to_string(),
        String::new(),
        "[antecedents]".to_string(),
        "publication_receipt_required = true".to_string(),
        "github_publication_required = true".to_string(),
        "maintainer_review_required = true".to_string(),
        "credit_ledger_required = true".to_string(),
        "public_proof_required = true".to_string(),
        String::new(),
        "[required_receipts]".to_string(),
        "bifrost_publication = \"gamecult.bifrost.public_proof_publication_receipt.v0\""
            .to_string(),
        "github_publication = \"gamecult.github.publication_receipt.v0\"".to_string(),
        "maintainer_review = \"gamecult.maintainer.review_receipt.v0\"".to_string(),
        "credit_ledger = \"gamecult.bifrost.credit_receipt.v0\"".to_string(),
        "upstream_sync = \"epiphany.repo_work_upstream_main_sync.v0\"".to_string(),
        "ancestry_proof = \"git.merge_base_is_ancestor\"".to_string(),
        String::new(),
        "[sync_proof]".to_string(),
        "target_ref = \"origin/main\"".to_string(),
        "requires_fetch_before_check = true".to_string(),
        "requires_merge_base_ancestor_check = true".to_string(),
        "requires_clean_public_proof_readback = true".to_string(),
        "records_upstream_main_sha = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "merge_authorized = false".to_string(),
        "push_authorized = false".to_string(),
        "upstream_sync_authorized = false".to_string(),
        "github_publication_authorized = false".to_string(),
        "credit_ledger_authorized = false".to_string(),
        "hands_action_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "bifrost_upstream_proof_required = true".to_string(),
        "maintainer_review_receipt_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the sync request path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the request names upstream-main ancestry proof without authorizing merge, push, or sync.\",".to_string(),
        "  \"Soul verifies maintainer review and publication receipts are required before any upstream proof is accepted.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the sync request if Bifrost publication or maintainer review has not happened.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.sync_request".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived an upstream-main sync proof request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add sync request for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo sync request path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the request requires publication, maintainer review, credit, and ancestry proof receipts without authorizing merge, push, sync, service lifecycle, or cross-body mutation.".to_string(),
            "Soul verifies no paths outside the declared sync request changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated sync request if the work is not ready for upstream-main proof.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.sync_request"),
    })
}

fn derive_repo_maintainer_review_request_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/review-requests/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request_id = format!("repo-maintainer-review-request:{item_slug}");
    let verification_request_ref = format!(".epiphany/verification-requests/{item_slug}.toml");
    let publication_request_ref = format!(".epiphany/publication-requests/{item_slug}.toml");
    let lines = vec![
        "# Epiphany repo maintainer review request.".to_string(),
        "# Branch-local human review cargo; not approval, merge, or publication authority."
            .to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_maintainer_review_request.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.maintainer_review_request")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "maintainer_approval_granted = false".to_string(),
        "merge_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "upstream_sync_authorized = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[request]".to_string(),
        format!("id = {}", toml_basic_string(&request_id)),
        "status = \"awaiting-maintainer-review\"".to_string(),
        "requested_owner = \"Maintainer\"".to_string(),
        "requested_effect = \"review-redacted-proof-and-branch-diff\"".to_string(),
        format!(
            "verification_request_ref = {}",
            toml_basic_string(&verification_request_ref)
        ),
        format!(
            "publication_request_ref = {}",
            toml_basic_string(&publication_request_ref)
        ),
        "review_scope = \"changed paths, closure proof, public proof bundle, and Bifrost credit context\""
            .to_string(),
        String::new(),
        "[antecedents]".to_string(),
        "closure_review_required = true".to_string(),
        "soul_verdict_required = true".to_string(),
        "mind_commit_required = true".to_string(),
        "public_proof_required = true".to_string(),
        "bifrost_publication_request_required = true".to_string(),
        String::new(),
        "[required_receipts]".to_string(),
        "closure_review = \"epiphany.repo_work_closure_review.v0\"".to_string(),
        "soul_verdict = \"epiphany.soul.verification_verdict\"".to_string(),
        "mind_commit = \"epiphany.mind.state_commit_receipt\"".to_string(),
        "public_proof = \"epiphany.repo_work_public_proof_bundle.v0\"".to_string(),
        "maintainer_review = \"gamecult.maintainer.review_receipt.v0\"".to_string(),
        "bifrost_publication = \"gamecult.bifrost.public_proof_publication_receipt.v0\""
            .to_string(),
        String::new(),
        "[review_packet]".to_string(),
        "requires_reviewer_identity = true".to_string(),
        "requires_review_verdict = true".to_string(),
        "allowed_verdicts = [\"approved\", \"changes-requested\", \"rejected\", \"needs-human-context\"]"
            .to_string(),
        "requires_changed_path_list = true".to_string(),
        "requires_public_proof_ref = true".to_string(),
        "requires_private_state_redaction_check = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "maintainer_approval_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "push_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "upstream_sync_authorized = false".to_string(),
        "hands_action_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "human_or_maintainer_response_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the maintainer review request path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the request names reviewer identity, verdict, changed paths, and redacted proof requirements without approving or merging.\",".to_string(),
        "  \"Soul verifies the request grants no publication, sync, Hands, service lifecycle, or cross-body authority.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the maintainer review request if the work is not ready for human review.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.maintainer_review_request".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a maintainer review request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add maintainer review request for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo maintainer review request path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the request requires reviewer identity, verdict, changed paths, redacted proof, and private-state redaction without authorizing approval, merge, publication, sync, service lifecycle, or cross-body mutation.".to_string(),
            "Soul verifies no paths outside the declared maintainer review request changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated maintainer review request if the work is not ready for human review.".to_string(),
        ],
        derivation: plan_derivation_receipt(
            input,
            action_family,
            "repo.maintainer_review_request",
        ),
    })
}

fn derive_repo_pr_request_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/pr-requests/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request_id = format!("repo-pr-request:{item_slug}");
    let maintainer_review_request_ref = format!(".epiphany/review-requests/{item_slug}.toml");
    let publication_request_ref = format!(".epiphany/publication-requests/{item_slug}.toml");
    let sync_request_ref = format!(".epiphany/sync-requests/{item_slug}.toml");
    let lines = vec![
        "# Epiphany repo pull request request.".to_string(),
        "# Branch-local GitHub/Bifrost request cargo; not push, PR, merge, or sync authority."
            .to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_pr_request.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.pr_request")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "github_pr_authorized = false".to_string(),
        "branch_push_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "upstream_sync_authorized = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[request]".to_string(),
        format!("id = {}", toml_basic_string(&request_id)),
        "status = \"awaiting-pr-publication-review\"".to_string(),
        "routing_owner = \"Self\"".to_string(),
        "publication_owner = \"Bifrost\"".to_string(),
        "execution_owner = \"Hands\"".to_string(),
        "provider = \"GitHub\"".to_string(),
        "provider_receipt_required = true".to_string(),
        "requested_effect = \"open-or-update-review-pr-from-redacted-proof-and-maintainer-context\""
            .to_string(),
        format!(
            "maintainer_review_request_ref = {}",
            toml_basic_string(&maintainer_review_request_ref)
        ),
        format!(
            "publication_request_ref = {}",
            toml_basic_string(&publication_request_ref)
        ),
        format!("sync_request_ref = {}", toml_basic_string(&sync_request_ref)),
        "pr_scope = \"branch diff, redacted public proof, maintainer review, Bifrost credit context, and upstream-main follow-up\""
            .to_string(),
        String::new(),
        "[antecedents]".to_string(),
        "closure_review_required = true".to_string(),
        "soul_verdict_required = true".to_string(),
        "mind_commit_required = true".to_string(),
        "public_proof_required = true".to_string(),
        "maintainer_review_required = true".to_string(),
        "bifrost_publication_required = true".to_string(),
        "credit_ledger_required = true".to_string(),
        String::new(),
        "[required_receipts]".to_string(),
        "closure_review = \"epiphany.repo_work_closure_review.v0\"".to_string(),
        "soul_verdict = \"epiphany.soul.verification_verdict\"".to_string(),
        "mind_commit = \"epiphany.mind.state_commit_receipt\"".to_string(),
        "public_proof = \"epiphany.repo_work_public_proof_bundle.v0\"".to_string(),
        "maintainer_review = \"gamecult.maintainer.review_receipt.v0\"".to_string(),
        "bifrost_publication = \"gamecult.bifrost.public_proof_publication_receipt.v0\""
            .to_string(),
        "credit_ledger = \"gamecult.bifrost.credit_receipt.v0\"".to_string(),
        "pr_publication = \"gamecult.github.pull_request_publication_receipt.v0\""
            .to_string(),
        String::new(),
        "[pr_packet]".to_string(),
        "base_ref = \"origin/main\"".to_string(),
        "requires_branch_name = true".to_string(),
        "requires_title = true".to_string(),
        "requires_body = true".to_string(),
        "requires_changed_path_list = true".to_string(),
        "requires_public_proof_ref = true".to_string(),
        "requires_maintainer_review_ref = true".to_string(),
        "requires_credit_ref = true".to_string(),
        "requires_private_state_redaction_check = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "github_pr_authorized = false".to_string(),
        "branch_push_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "upstream_sync_authorized = false".to_string(),
        "hands_action_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "maintainer_review_required = true".to_string(),
        "bifrost_publication_gate_required = true".to_string(),
        "hands_execution_required = true".to_string(),
        "github_provider_receipt_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the PR request path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the request names PR packet requirements without authorizing branch push, PR publication, merge, or sync.\",".to_string(),
        "  \"Soul verifies maintainer review, public proof, Bifrost publication, and credit receipts are required before any GitHub PR action.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the PR request if the work is not ready for GitHub/Bifrost publication review.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.pr_request".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a GitHub PR request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add PR request for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo PR request path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the request requires maintainer review, redacted public proof, Bifrost publication, credit, and PR packet fields without authorizing push, PR publication, merge, sync, service lifecycle, or cross-body mutation.".to_string(),
            "Soul verifies no paths outside the declared PR request changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated PR request if the work is not ready for GitHub/Bifrost publication review.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.pr_request"),
    })
}

fn derive_repo_credit_request_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/credit-requests/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request_id = format!("repo-credit-request:{item_slug}");
    let publication_request_ref = format!(".epiphany/publication-requests/{item_slug}.toml");
    let maintainer_review_request_ref = format!(".epiphany/review-requests/{item_slug}.toml");
    let pr_request_ref = format!(".epiphany/pr-requests/{item_slug}.toml");
    let lines = vec![
        "# Epiphany repo Bifrost credit request.".to_string(),
        "# Branch-local credit ledger request cargo; not credit, publication, PR, merge, or sync authority."
            .to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_credit_request.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.credit_request")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "credit_ledger_authorized = false".to_string(),
        "bifrost_publication_authorized = false".to_string(),
        "github_pr_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "upstream_sync_authorized = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[request]".to_string(),
        format!("id = {}", toml_basic_string(&request_id)),
        "status = \"awaiting-bifrost-credit-review\"".to_string(),
        "requested_owner = \"Bifrost\"".to_string(),
        "requested_effect = \"record-credit-for-redacted-proof-and-accepted-artifact\""
            .to_string(),
        format!(
            "publication_request_ref = {}",
            toml_basic_string(&publication_request_ref)
        ),
        format!(
            "maintainer_review_request_ref = {}",
            toml_basic_string(&maintainer_review_request_ref)
        ),
        format!("pr_request_ref = {}", toml_basic_string(&pr_request_ref)),
        "credit_scope = \"authorship, maintainer review, accepted artifact, public proof, and Bifrost ledger readback\""
            .to_string(),
        String::new(),
        "[antecedents]".to_string(),
        "closure_review_required = true".to_string(),
        "soul_verdict_required = true".to_string(),
        "mind_commit_required = true".to_string(),
        "public_proof_required = true".to_string(),
        "maintainer_review_required = true".to_string(),
        "accepted_artifact_required = true".to_string(),
        "authorship_context_required = true".to_string(),
        String::new(),
        "[required_receipts]".to_string(),
        "closure_review = \"epiphany.repo_work_closure_review.v0\"".to_string(),
        "soul_verdict = \"epiphany.soul.verification_verdict\"".to_string(),
        "mind_commit = \"epiphany.mind.state_commit_receipt\"".to_string(),
        "public_proof = \"epiphany.repo_work_public_proof_bundle.v0\"".to_string(),
        "maintainer_review = \"gamecult.maintainer.review_receipt.v0\"".to_string(),
        "accepted_artifact = \"gamecult.artifact.acceptance_receipt.v0\"".to_string(),
        "credit_ledger = \"gamecult.bifrost.credit_receipt.v0\"".to_string(),
        "credit_readback = \"gamecult.bifrost.credit_readback_receipt.v0\"".to_string(),
        String::new(),
        "[credit_packet]".to_string(),
        "requires_author_identity = true".to_string(),
        "requires_reviewer_identity = true".to_string(),
        "requires_accepted_artifact_ref = true".to_string(),
        "requires_public_proof_ref = true".to_string(),
        "requires_changed_path_list = true".to_string(),
        "requires_credit_ledger_target = true".to_string(),
        "requires_private_state_redaction_check = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "credit_ledger_authorized = false".to_string(),
        "bifrost_publication_authorized = false".to_string(),
        "github_pr_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "upstream_sync_authorized = false".to_string(),
        "hands_action_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "bifrost_credit_authority_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the credit request path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the request names authorship, reviewer, accepted artifact, public proof, and redaction requirements without authorizing credit ledger writes.\",".to_string(),
        "  \"Soul verifies no publication, PR, merge, sync, Hands, service lifecycle, or cross-body authority is granted.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the credit request if the artifact has not been accepted or credited by Bifrost.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.credit_request".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a Bifrost credit request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add credit request for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo credit request path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the request requires authorship, maintainer review, accepted artifact, public proof, credit ledger, and readback receipts without authorizing credit, publication, PR, merge, sync, service lifecycle, or cross-body mutation.".to_string(),
            "Soul verifies no paths outside the declared credit request changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated credit request if the artifact is not ready for Bifrost credit review.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.credit_request"),
    })
}

fn derive_repo_artifact_acceptance_request_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/artifact-acceptance-requests/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request_id = format!("repo-artifact-acceptance-request:{item_slug}");
    let verification_request_ref = format!(".epiphany/verification-requests/{item_slug}.toml");
    let maintainer_review_request_ref = format!(".epiphany/review-requests/{item_slug}.toml");
    let publication_request_ref = format!(".epiphany/publication-requests/{item_slug}.toml");
    let lines = vec![
        "# Epiphany repo artifact acceptance request.".to_string(),
        "# Branch-local accepted-artifact request cargo; not acceptance, credit, publication, PR, merge, or sync authority.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_artifact_acceptance_request.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.artifact_acceptance_request")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "artifact_acceptance_authorized = false".to_string(),
        "credit_ledger_authorized = false".to_string(),
        "github_pr_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "upstream_sync_authorized = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[request]".to_string(),
        format!("id = {}", toml_basic_string(&request_id)),
        "status = \"awaiting-artifact-acceptance-review\"".to_string(),
        "routing_owner = \"Self\"".to_string(),
        "acceptance_owner = \"Maintainer\"".to_string(),
        "accounting_owner = \"Bifrost\"".to_string(),
        "acceptance_receipt_required = true".to_string(),
        "requested_effect = \"record-accepted-artifact-for-reviewed-branch-work\"".to_string(),
        format!(
            "verification_request_ref = {}",
            toml_basic_string(&verification_request_ref)
        ),
        format!(
            "maintainer_review_request_ref = {}",
            toml_basic_string(&maintainer_review_request_ref)
        ),
        format!(
            "publication_request_ref = {}",
            toml_basic_string(&publication_request_ref)
        ),
        "acceptance_scope = \"changed paths, Hands commit, Soul closure, maintainer review, public proof, and accepted artifact readback\""
            .to_string(),
        String::new(),
        "[antecedents]".to_string(),
        "closure_review_required = true".to_string(),
        "soul_verdict_required = true".to_string(),
        "mind_commit_required = true".to_string(),
        "public_proof_required = true".to_string(),
        "maintainer_review_required = true".to_string(),
        "hands_commit_required = true".to_string(),
        String::new(),
        "[required_receipts]".to_string(),
        "closure_review = \"epiphany.repo_work_closure_review.v0\"".to_string(),
        "soul_verdict = \"epiphany.soul.verification_verdict\"".to_string(),
        "mind_commit = \"epiphany.mind.state_commit_receipt\"".to_string(),
        "public_proof = \"epiphany.repo_work_public_proof_bundle.v0\"".to_string(),
        "maintainer_review = \"gamecult.maintainer.review_receipt.v0\"".to_string(),
        "hands_commit = \"epiphany.hands.commit_receipt\"".to_string(),
        "accepted_artifact = \"gamecult.artifact.acceptance_receipt.v0\"".to_string(),
        String::new(),
        "[artifact_packet]".to_string(),
        "requires_artifact_ref = true".to_string(),
        "requires_commit_sha = true".to_string(),
        "requires_changed_path_list = true".to_string(),
        "requires_review_verdict = true".to_string(),
        "requires_public_proof_ref = true".to_string(),
        "requires_acceptance_rationale = true".to_string(),
        "requires_private_state_redaction_check = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "artifact_acceptance_authorized = false".to_string(),
        "credit_ledger_authorized = false".to_string(),
        "github_pr_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "upstream_sync_authorized = false".to_string(),
        "hands_action_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "maintainer_acceptance_authority_required = true".to_string(),
        "bifrost_accounting_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the artifact acceptance request path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the request names artifact, commit, changed paths, review, proof, and rationale requirements without authorizing artifact acceptance.\",".to_string(),
        "  \"Soul verifies no credit, publication, PR, merge, sync, Hands, service lifecycle, or cross-body authority is granted.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the artifact acceptance request if the work has not been reviewed as an accepted artifact.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.artifact_acceptance_request".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived an artifact acceptance request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add artifact acceptance request for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo artifact acceptance request path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the request requires artifact, commit, changed paths, review verdict, public proof, and acceptance rationale without authorizing acceptance, credit, publication, PR, merge, sync, service lifecycle, or cross-body mutation.".to_string(),
            "Soul verifies no paths outside the declared artifact acceptance request changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated artifact acceptance request if the work is not ready for maintainer/Bifrost artifact acceptance review.".to_string(),
        ],
        derivation: plan_derivation_receipt(
            input,
            action_family,
            "repo.artifact_acceptance_request",
        ),
    })
}

fn derive_repo_metrics_request_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/metrics-requests/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request_id = format!("repo-metrics-request:{item_slug}");
    let publication_request_ref = format!(".epiphany/publication-requests/{item_slug}.toml");
    let credit_request_ref = format!(".epiphany/credit-requests/{item_slug}.toml");
    let artifact_acceptance_request_ref =
        format!(".epiphany/artifact-acceptance-requests/{item_slug}.toml");
    let lines = vec![
        "# Epiphany repo metrics request.".to_string(),
        "# Branch-local accounting request cargo; not spend, review, credit, publication, PR, merge, or sync authority.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_metrics_request.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.metrics_request")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "metrics_ledger_authorized = false".to_string(),
        "spend_authorized = false".to_string(),
        "review_load_authorized = false".to_string(),
        "credit_ledger_authorized = false".to_string(),
        "github_pr_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "upstream_sync_authorized = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[request]".to_string(),
        format!("id = {}", toml_basic_string(&request_id)),
        "status = \"awaiting-metrics-review\"".to_string(),
        "routing_owner = \"Self\"".to_string(),
        "accounting_owner = \"Bifrost\"".to_string(),
        "review_evidence_owner = \"Maintainer\"".to_string(),
        "spend_receipt_required = true".to_string(),
        "review_load_receipt_required = true".to_string(),
        "requested_effect = \"record-compute-review-and-artifact-accounting\"".to_string(),
        format!(
            "publication_request_ref = {}",
            toml_basic_string(&publication_request_ref)
        ),
        format!("credit_request_ref = {}", toml_basic_string(&credit_request_ref)),
        format!(
            "artifact_acceptance_request_ref = {}",
            toml_basic_string(&artifact_acceptance_request_ref)
        ),
        "metrics_scope = \"model spend, review load, accepted artifact, public proof, and credit readback\""
            .to_string(),
        String::new(),
        "[antecedents]".to_string(),
        "closure_review_required = true".to_string(),
        "soul_verdict_required = true".to_string(),
        "mind_commit_required = true".to_string(),
        "public_proof_required = true".to_string(),
        "accepted_artifact_required = true".to_string(),
        "credit_request_required = true".to_string(),
        String::new(),
        "[required_receipts]".to_string(),
        "closure_review = \"epiphany.repo_work_closure_review.v0\"".to_string(),
        "soul_verdict = \"epiphany.soul.verification_verdict\"".to_string(),
        "mind_commit = \"epiphany.mind.state_commit_receipt\"".to_string(),
        "public_proof = \"epiphany.repo_work_public_proof_bundle.v0\"".to_string(),
        "accepted_artifact = \"gamecult.artifact.acceptance_receipt.v0\"".to_string(),
        "model_spend = \"gamecult.metrics.model_spend_receipt.v0\"".to_string(),
        "review_load = \"gamecult.metrics.review_load_receipt.v0\"".to_string(),
        "credit_readback = \"gamecult.bifrost.credit_readback_receipt.v0\"".to_string(),
        String::new(),
        "[metrics_packet]".to_string(),
        "requires_model_call_count = true".to_string(),
        "requires_token_or_cost_summary = true".to_string(),
        "requires_review_minutes_or_count = true".to_string(),
        "requires_accepted_artifact_ref = true".to_string(),
        "requires_public_proof_ref = true".to_string(),
        "requires_credit_readback_ref = true".to_string(),
        "requires_private_state_redaction_check = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "metrics_ledger_authorized = false".to_string(),
        "spend_authorized = false".to_string(),
        "review_load_authorized = false".to_string(),
        "credit_ledger_authorized = false".to_string(),
        "github_pr_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "upstream_sync_authorized = false".to_string(),
        "hands_action_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "bifrost_accounting_required = true".to_string(),
        "maintainer_review_evidence_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the metrics request path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the request names model spend, review load, accepted artifact, public proof, credit readback, and redaction requirements without authorizing ledger writes.\",".to_string(),
        "  \"Soul verifies no publication, PR, merge, sync, Hands, service lifecycle, or cross-body authority is granted.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the metrics request if the work is not ready for Bifrost/maintainer accounting review.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.metrics_request".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a repo metrics request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add metrics request for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo metrics request path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the request requires model spend, review load, accepted artifact, public proof, and credit readback receipts without authorizing metrics ledger writes, spend, review-load mutation, publication, PR, merge, sync, service lifecycle, or cross-body mutation.".to_string(),
            "Soul verifies no paths outside the declared metrics request changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated metrics request if the work is not ready for accounting review.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.metrics_request"),
    })
}

fn derive_repo_readiness_review_request_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/readiness-reviews/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request_id = format!("repo-readiness-review-request:{item_slug}");
    let lines = vec![
        "# Epiphany repo MVP readiness review request.".to_string(),
        "# Branch-local readiness review cargo; not publication, merge, deployment, service, or state authority.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_readiness_review_request.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.readiness_review_request")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "readiness_approved = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "deployment_authority = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[request]".to_string(),
        format!("id = {}", toml_basic_string(&request_id)),
        "status = \"awaiting-mvp-readiness-review\"".to_string(),
        "routing_owner = \"Self\"".to_string(),
        "required_reviewers = [\"Maintainer\", \"Soul\", \"Mind\", \"Bifrost\"]".to_string(),
        "readiness_approval_owner = \"Maintainer\"".to_string(),
        "requested_effect = \"review-redacted-repo-swarm-mvp-proof-bundle\"".to_string(),
        "readiness_scope = \"fresh repo init, online swarm, Persona intake, Imagination planning, Self queue-run, Hands branch work, Soul closure, Modeling map update, Mind admission, Bifrost public proof, upstream-main sync, Idunn lifecycle, global tool directory, and private-state redaction\"".to_string(),
        "maintainer_readiness_acceptance_required = true".to_string(),
        "bifrost_publication_review_required = true".to_string(),
        String::new(),
        "[antecedents]".to_string(),
        "repo_init_required = true".to_string(),
        "swarm_online_required = true".to_string(),
        "persona_intake_required = true".to_string(),
        "imagination_plan_required = true".to_string(),
        "self_queue_run_required = true".to_string(),
        "hands_commit_required = true".to_string(),
        "soul_closure_required = true".to_string(),
        "modeling_map_update_required = true".to_string(),
        "mind_admission_required = true".to_string(),
        "public_proof_required = true".to_string(),
        "bifrost_publication_required = true".to_string(),
        "upstream_main_sync_required = true".to_string(),
        "idunn_lifecycle_readiness_required = true".to_string(),
        "tool_directory_readiness_required = true".to_string(),
        "private_state_redaction_required = true".to_string(),
        String::new(),
        "[required_receipts]".to_string(),
        "repo_init = \"epiphany.repo_swarm_init_receipt.v0\"".to_string(),
        "swarm_online = \"epiphany.repo_swarm_online_receipt.v0\"".to_string(),
        "persona_speech_audit = \"epiphany.persona_speech_audit.v0\"".to_string(),
        "imagination_action_items = \"epiphany.repo_work_imagination_action_items_receipt.v0\"".to_string(),
        "queue_run = \"epiphany.repo_work_queue_run_receipt.v0\"".to_string(),
        "hands_commit = \"epiphany.hands.commit_receipt\"".to_string(),
        "closure_review = \"epiphany.repo_work_closure_review.v0\"".to_string(),
        "soul_verdict = \"epiphany.soul.verification_verdict\"".to_string(),
        "modeling_map = \"epiphany.repo_work_map_entry.v0\"".to_string(),
        "mind_commit = \"epiphany.mind.state_commit_receipt\"".to_string(),
        "public_proof = \"epiphany.repo_work_public_proof_bundle.v0\"".to_string(),
        "bifrost_publication = \"gamecult.bifrost.public_proof_publication_receipt.v0\"".to_string(),
        "upstream_sync = \"epiphany.repo_work_upstream_sync_receipt.v0\"".to_string(),
        "idunn_lifecycle = \"epiphany.repo_work_service_audit.v0\"".to_string(),
        "tool_directory = \"epiphany.cultmesh.daemon_tool_directory.v0\"".to_string(),
        String::new(),
        "[readiness_packet]".to_string(),
        "requires_proof_bundle_ref = true".to_string(),
        "requires_changed_path_list = true".to_string(),
        "requires_branch_name = true".to_string(),
        "requires_upstream_main_ref = true".to_string(),
        "requires_public_proof_ref = true".to_string(),
        "requires_bifrost_ledger_ref = true".to_string(),
        "requires_idunn_lifecycle_ref = true".to_string(),
        "requires_tool_directory_ref = true".to_string(),
        "requires_redaction_report = true".to_string(),
        "requires_reviewer_identity = true".to_string(),
        "allowed_verdicts = [\"ready\", \"ready-with-caveats\", \"not-ready\", \"needs-human-review\"]".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "readiness_approval_authorized = false".to_string(),
        "durable_state_commit_authorized = false".to_string(),
        "publication_authorized = false".to_string(),
        "bifrost_publication_authorized = false".to_string(),
        "github_pr_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "upstream_sync_authorized = false".to_string(),
        "deployment_authority = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "hands_action_authorized = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "maintainer_review_required = true".to_string(),
        "soul_verification_required = true".to_string(),
        "mind_admission_required = true".to_string(),
        "bifrost_publication_review_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the readiness review request path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the request names init, online, Persona, Imagination, Self, Hands, Soul, Modeling, Mind, Bifrost, upstream-main, Idunn, tool-directory, and redaction proof requirements.\",".to_string(),
        "  \"Soul verifies the request grants no readiness approval, state commit, publication, PR, merge, sync, deployment, service lifecycle, Hands, cross-body, or private Verse authority.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the readiness review request if the proof bundle is not ready for maintainer/Soul/Mind/Bifrost review.\"]".to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.readiness_review_request".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived an MVP readiness review request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add readiness review request for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo readiness review request path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the request requires a redacted proof bundle with repo init, online swarm, Persona, Imagination, Self, Hands, Soul, Modeling, Mind, Bifrost, upstream-main, Idunn, tool-directory, and redaction receipts without authorizing readiness approval, state commit, publication, merge, deployment, service lifecycle, or cross-body mutation.".to_string(),
            "Soul verifies no paths outside the declared readiness review request changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated readiness review request if the proof bundle is not ready for review.".to_string(),
        ],
        derivation: plan_derivation_receipt(
            input,
            action_family,
            "repo.readiness_review_request",
        ),
    })
}

fn derive_repo_doctrine_update_request_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/doctrine-update-requests/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request_id = format!("repo-doctrine-update-request:{item_slug}");
    let doctrine_target = "AGENTS.md";
    let lines = vec![
        "# Epiphany repo doctrine update request.".to_string(),
        "# Branch-local governance request cargo; not direct doctrine mutation authority.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_doctrine_update_request.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.doctrine_update_request")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "doctrine_mutation_authorized = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[request]".to_string(),
        format!("id = {}", toml_basic_string(&request_id)),
        "status = \"awaiting-doctrine-review\"".to_string(),
        "routing_owner = \"Self\"".to_string(),
        "required_reviewers = [\"Maintainer\", \"Mind\", \"Soul\"]".to_string(),
        "doctrine_admission_owner = \"Mind\"".to_string(),
        "mutation_owner = \"Hands\"".to_string(),
        "requested_effect = \"review-repo-agent-doctrine-update\"".to_string(),
        format!("doctrine_target = {}", toml_basic_string(doctrine_target)),
        "change_surface = \"repo-local agent instructions and operating doctrine\"".to_string(),
        "requires_source_grounding = true".to_string(),
        "requires_human_or_maintainer_review = true".to_string(),
        String::new(),
        "[antecedents]".to_string(),
        "persona_or_human_feedback_required = true".to_string(),
        "imagination_plan_required = true".to_string(),
        "mind_adoption_required = true".to_string(),
        "soul_review_required = true".to_string(),
        "maintainer_review_required = true".to_string(),
        String::new(),
        "[required_receipts]".to_string(),
        "imagination_plan = \"epiphany.repo_work_imagination_action_items_receipt.v0\""
            .to_string(),
        "mind_adoption = \"epiphany.repo_work_mind_adoption_decision.v0\"".to_string(),
        "soul_review = \"epiphany.repo_work_closure_review.v0\"".to_string(),
        "maintainer_review = \"gamecult.maintainer.review_receipt.v0\"".to_string(),
        "hands_commit = \"epiphany.hands.commit_receipt\"".to_string(),
        String::new(),
        "[doctrine_packet]".to_string(),
        "requires_current_doctrine_ref = true".to_string(),
        "requires_proposed_change_summary = true".to_string(),
        "requires_invariant_impact = true".to_string(),
        "requires_rehydration_impact = true".to_string(),
        "requires_rollback_plan = true".to_string(),
        "requires_private_state_redaction_check = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "direct_doctrine_mutation_authority = false".to_string(),
        "direct_hands_authority = false".to_string(),
        "direct_mind_state_commit = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "maintainer_review_required = true".to_string(),
        "mind_admission_required = true".to_string(),
        "hands_receipts_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the doctrine update request path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the request names AGENTS.md as review target and requires source grounding, Mind adoption, Soul review, maintainer review, and rollback planning before doctrine mutation.\",".to_string(),
        "  \"Soul verifies no direct doctrine, Hands, publication, merge, service lifecycle, cross-body, or private Verse authority is granted.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the doctrine update request if the proposed doctrine change is not ready for review.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.doctrine_update_request".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a repo doctrine update request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add doctrine update request for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo doctrine update request path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the request names AGENTS.md review scope, source grounding, Mind adoption, Soul review, maintainer review, rollback planning, and no direct doctrine mutation authority.".to_string(),
            "Soul verifies no paths outside the declared doctrine update request changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated doctrine update request if the proposed doctrine change is not ready for review.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.doctrine_update_request"),
    })
}

fn derive_repo_secret_policy_request_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/security/secret-policy-requests/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request_id = format!("repo-secret-policy-request:{item_slug}");
    let lines = vec![
        "# Epiphany repo secret policy request.".to_string(),
        "# Branch-local security request cargo; not secret access, write-permission, or publication authority.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_secret_policy_request.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.secret_policy_request")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "secret_access_authorized = false".to_string(),
        "write_permission_authorized = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[request]".to_string(),
        format!("id = {}", toml_basic_string(&request_id)),
        "status = \"awaiting-security-review\"".to_string(),
        "routing_owner = \"Self\"".to_string(),
        "required_reviewers = [\"Maintainer\", \"Soul\", \"Mind\", \"Bifrost\"]".to_string(),
        "policy_admission_owner = \"Mind\"".to_string(),
        "requested_effect = \"review-repo-secret-and-write-permission-policy\"".to_string(),
        "security_scope = \"secrets, credentials, write permissions, public/private export, and deployment authority\"".to_string(),
        "requires_secret_inventory_without_values = true".to_string(),
        "requires_write_permission_scope = true".to_string(),
        "requires_public_private_export_boundary = true".to_string(),
        String::new(),
        "[antecedents]".to_string(),
        "source_grounding_required = true".to_string(),
        "soul_review_required = true".to_string(),
        "mind_adoption_required = true".to_string(),
        "maintainer_review_required = true".to_string(),
        "bifrost_publication_review_required = true".to_string(),
        String::new(),
        "[required_receipts]".to_string(),
        "source_grounding = \"epiphany.eyes.evidence_packet\"".to_string(),
        "soul_review = \"epiphany.repo_work_closure_review.v0\"".to_string(),
        "mind_adoption = \"epiphany.repo_work_mind_adoption_decision.v0\"".to_string(),
        "maintainer_review = \"gamecult.maintainer.review_receipt.v0\"".to_string(),
        "bifrost_publication_review = \"gamecult.bifrost.publication_review_receipt.v0\""
            .to_string(),
        String::new(),
        "[security_packet]".to_string(),
        "requires_secret_locations_without_values = true".to_string(),
        "requires_credential_owner = true".to_string(),
        "requires_write_scope_matrix = true".to_string(),
        "requires_public_export_redaction_rules = true".to_string(),
        "requires_deployment_authority_owner = true".to_string(),
        "requires_incident_rollback_plan = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "direct_secret_access_authority = false".to_string(),
        "secret_value_materialization = false".to_string(),
        "write_permission_authority = false".to_string(),
        "deployment_authority = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "maintainer_security_review_required = true".to_string(),
        "soul_security_verification_required = true".to_string(),
        "mind_policy_admission_required = true".to_string(),
        "bifrost_publication_review_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the secret policy request path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the request names secret locations without values, credential ownership, write-permission scope, public/private export boundaries, deployment authority ownership, and rollback planning.\",".to_string(),
        "  \"Soul verifies no secret access, write permission, deployment, publication, merge, service lifecycle, cross-body, or private Verse authority is granted.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the secret policy request if the security review is not ready for maintainer/Soul/Bifrost review.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.secret_policy_request".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a repo secret policy request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add secret policy request for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo secret policy request path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the request names secret-location-without-values, credential ownership, write scope, public/private export boundaries, deployment authority ownership, rollback planning, and no direct secret or write authority.".to_string(),
            "Soul verifies no paths outside the declared secret policy request changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated secret policy request if the security review is not ready.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.secret_policy_request"),
    })
}

fn derive_repo_dependency_policy_request_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/dependency-policy-requests/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request_id = format!("repo-dependency-policy-request:{item_slug}");
    let lines = vec![
        "# Epiphany repo dependency policy request.".to_string(),
        "# Branch-local supply-chain policy request cargo; not package install, lockfile mutation, network fetch, CI mutation, deployment, or publication authority.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_dependency_policy_request.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.dependency_policy_request")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "dependency_update_authorized = false".to_string(),
        "package_install_authorized = false".to_string(),
        "lockfile_mutation_authorized = false".to_string(),
        "network_fetch_authorized = false".to_string(),
        "ci_mutation_authorized = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "deployment_authority = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[request]".to_string(),
        format!("id = {}", toml_basic_string(&request_id)),
        "status = \"awaiting-dependency-policy-review\"".to_string(),
        "routing_owner = \"Self\"".to_string(),
        "required_reviewers = [\"Maintainer\", \"Soul\", \"Mind\", \"Bifrost\"]".to_string(),
        "policy_admission_owner = \"Mind\"".to_string(),
        "requested_effect = \"review-repo-dependency-and-supply-chain-policy\"".to_string(),
        "dependency_scope = \"dependency manifests, lockfiles, package-manager commands, vendored code, update cadence, license posture, vulnerability review, and provenance requirements\"".to_string(),
        "requires_manifest_inventory = true".to_string(),
        "requires_lockfile_policy = true".to_string(),
        "requires_package_manager_command_review = true".to_string(),
        "requires_network_fetch_policy = true".to_string(),
        "requires_vulnerability_review = true".to_string(),
        "requires_license_review = true".to_string(),
        "requires_rollback_plan = true".to_string(),
        String::new(),
        "[antecedents]".to_string(),
        "source_grounding_required = true".to_string(),
        "eyes_evidence_required = true".to_string(),
        "soul_review_required = true".to_string(),
        "mind_adoption_required = true".to_string(),
        "maintainer_review_required = true".to_string(),
        "bifrost_publication_review_required = true".to_string(),
        String::new(),
        "[required_receipts]".to_string(),
        "source_grounding = \"epiphany.eyes.evidence_packet\"".to_string(),
        "soul_review = \"epiphany.repo_work_closure_review.v0\"".to_string(),
        "mind_adoption = \"epiphany.repo_work_mind_adoption_decision.v0\"".to_string(),
        "maintainer_review = \"gamecult.maintainer.review_receipt.v0\"".to_string(),
        "bifrost_publication_review = \"gamecult.bifrost.publication_review_receipt.v0\""
            .to_string(),
        "dependency_audit = \"gamecult.supply_chain.dependency_audit_receipt.v0\""
            .to_string(),
        String::new(),
        "[dependency_packet]".to_string(),
        "requires_manifest_paths = true".to_string(),
        "requires_lockfile_paths = true".to_string(),
        "requires_package_manager_commands = true".to_string(),
        "requires_vulnerability_sources = true".to_string(),
        "requires_license_constraints = true".to_string(),
        "requires_vendored_code_policy = true".to_string(),
        "requires_update_cadence = true".to_string(),
        "requires_private_state_redaction_check = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "direct_dependency_update_authority = false".to_string(),
        "direct_package_install_authority = false".to_string(),
        "direct_lockfile_mutation_authority = false".to_string(),
        "direct_network_fetch_authority = false".to_string(),
        "direct_ci_mutation_authority = false".to_string(),
        "direct_hands_authority = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "deployment_authority = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "maintainer_dependency_review_required = true".to_string(),
        "soul_dependency_verification_required = true".to_string(),
        "mind_policy_admission_required = true".to_string(),
        "bifrost_publication_review_required = true".to_string(),
        "supply_chain_audit_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the dependency policy request path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the request names manifest inventory, lockfile policy, package-manager command review, network fetch policy, vulnerability review, license review, vendored-code policy, update cadence, and rollback planning.\",".to_string(),
        "  \"Soul verifies no direct dependency update, package install, lockfile mutation, network fetch, CI mutation, Hands, publication, merge, deployment, service lifecycle, cross-body, or private Verse authority is granted.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the dependency policy request if the supply-chain review is not ready for maintainer/Soul/Bifrost review.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.dependency_policy_request".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a repo dependency policy request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add dependency policy request for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo dependency policy request path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the request names manifest inventory, lockfile policy, package-manager command review, network fetch policy, vulnerability review, license review, vendored-code policy, update cadence, rollback planning, and no direct dependency/package/network/CI authority.".to_string(),
            "Soul verifies no paths outside the declared dependency policy request changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated dependency policy request if the supply-chain review is not ready.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.dependency_policy_request"),
    })
}

fn derive_repo_deployment_request_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let default_target = format!(".epiphany/deployment-requests/{item_slug}.toml");
    let target_path = validate_toml_target_path(input.target_path.unwrap_or(&default_target))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request_id = format!("repo-deployment-request:{item_slug}");
    let lines = vec![
        "# Epiphany repo deployment request.".to_string(),
        "# Branch-local deployment request cargo; Idunn owns deployment execution.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_deployment_request.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.deployment_request")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "operator_authored_shell_details = false".to_string(),
        "hands_authority_granted = false".to_string(),
        "deployment_authority = false".to_string(),
        "ssh_authority = false".to_string(),
        "git_push_authority = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[request]".to_string(),
        format!("id = {}", toml_basic_string(&request_id)),
        "status = \"awaiting-idunn-review\"".to_string(),
        "routing_owner = \"Self\"".to_string(),
        "required_reviewers = [\"Maintainer\", \"Soul\", \"Mind\", \"Bifrost\"]".to_string(),
        "execution_owner = \"Idunn\"".to_string(),
        "requested_effect = \"review-repo-deployment-trigger-and-script\"".to_string(),
        "deployment_trigger = \"git-push-observed-by-idunn\"".to_string(),
        "deployment_surface = \"reviewed repo deployment script or runbook\"".to_string(),
        "requires_explicit_deployment_policy = true".to_string(),
        "requires_idunn_receipt = true".to_string(),
        "requires_aftercare_audit = true".to_string(),
        String::new(),
        "[antecedents]".to_string(),
        "source_grounding_required = true".to_string(),
        "mind_adoption_required = true".to_string(),
        "soul_review_required = true".to_string(),
        "maintainer_review_required = true".to_string(),
        "secret_policy_review_required = true".to_string(),
        "bifrost_publication_review_required = true".to_string(),
        String::new(),
        "[required_receipts]".to_string(),
        "source_grounding = \"epiphany.eyes.evidence_packet\"".to_string(),
        "mind_adoption = \"epiphany.repo_work_mind_adoption_decision.v0\"".to_string(),
        "soul_review = \"epiphany.repo_work_closure_review.v0\"".to_string(),
        "maintainer_review = \"gamecult.maintainer.review_receipt.v0\"".to_string(),
        "secret_policy = \"epiphany.repo_secret_policy_request.v0\"".to_string(),
        "bifrost_publication_review = \"gamecult.bifrost.publication_review_receipt.v0\""
            .to_string(),
        "idunn_deployment = \"gamecult.idunn.deployment_receipt.v0\"".to_string(),
        "aftercare_audit = \"gamecult.idunn.deployment_aftercare_audit.v0\"".to_string(),
        String::new(),
        "[deployment_packet]".to_string(),
        "requires_target_environment = true".to_string(),
        "requires_git_ref_or_branch = true".to_string(),
        "requires_deployment_script_ref = true".to_string(),
        "requires_script_hash_or_review_ref = true".to_string(),
        "requires_host_access_policy_ref = true".to_string(),
        "requires_secret_policy_ref = true".to_string(),
        "requires_rollback_plan = true".to_string(),
        "requires_aftercare_checks = true".to_string(),
        String::new(),
        "[authority]".to_string(),
        "branch_local_only = true".to_string(),
        "direct_deployment_authority = false".to_string(),
        "direct_ssh_authority = false".to_string(),
        "direct_git_push_authority = false".to_string(),
        "direct_service_lifecycle_authority = false".to_string(),
        "direct_hands_authority = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "idunn_deployment_authority_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the deployment request path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the request names git-push-triggered Idunn ownership, deployment script review, host access policy, secret policy, rollback, aftercare audit, and Idunn deployment receipts.\",".to_string(),
        "  \"Soul verifies no direct SSH, deployment, git push, service lifecycle, publication, merge, Hands, cross-body, or private Verse authority is granted.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the deployment request if the deployment path is not ready for Idunn review.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.deployment_request".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a repo deployment request from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add deployment request for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo deployment request path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the request names Idunn-owned git-push-triggered deployment, script review, host access policy, secret policy, rollback, aftercare, and no direct deployment/SSH/push authority.".to_string(),
            "Soul verifies no paths outside the declared deployment request changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated deployment request if the deployment review is not ready.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.deployment_request"),
    })
}

fn derive_repo_deployment_config_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let target_path =
        validate_toml_target_path(input.target_path.unwrap_or(".epiphany/deployment.toml"))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let deployment_id = format!("repo-deployment-config:{}", sanitize(input.item));
    let lines = vec![
        "# Epiphany repo deployment config.".to_string(),
        "# Branch-local configuration cargo; Idunn owns deployment execution.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_deployment_config.v0")
        ),
        format!("id = {}", toml_basic_string(&deployment_id)),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.deployment_config")
        ),
        format!("model_authored = {}", input.model_authored),
        format!(
            "model_ref = {}",
            toml_basic_string(input.model_ref.unwrap_or("deterministic-fallback"))
        ),
        "hands_authority_granted = false".to_string(),
        "deployment_authority = false".to_string(),
        "ssh_authority = false".to_string(),
        "git_push_authority = false".to_string(),
        "service_lifecycle_authority = false".to_string(),
        "durable_state_admitted = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "cross_repo_mutation = false".to_string(),
        "private_state_exposed = false".to_string(),
        format!("candidate_action_refs = {}", toml_array(&candidate_refs)),
        format!("public_discussion_refs = {}", toml_array(&public_refs)),
        String::new(),
        "[deployment]".to_string(),
        "enabled = false".to_string(),
        "owner = \"Idunn\"".to_string(),
        "trigger = \"git-push-observed-by-idunn\"".to_string(),
        "target_environment = \"review-required\"".to_string(),
        "watched_ref = \"refs/heads/main\"".to_string(),
        "deployment_script_ref = \"deploy/idunn-deploy.ps1\"".to_string(),
        "deployment_script_hash_required = true".to_string(),
        "deployment_script_review_required = true".to_string(),
        "host_access_policy_ref_required = true".to_string(),
        "secret_policy_ref = \".epiphany/security/secret-policy.toml\"".to_string(),
        "secret_values_embedded = false".to_string(),
        "rollback_plan_ref_required = true".to_string(),
        "aftercare_checks_required = true".to_string(),
        "idunn_receipt_required = true".to_string(),
        "aftercare_audit_required = true".to_string(),
        String::new(),
        "[cultmesh]".to_string(),
        "local_verse = \"gamecult-local\"".to_string(),
        "capability_family = \"gamecult.idunn.deployment\"".to_string(),
        "intent_contract = \"gamecult.idunn.deployment_intent.v0\"".to_string(),
        "receipt_contract = \"gamecult.idunn.deployment_receipt.v0\"".to_string(),
        "aftercare_contract = \"gamecult.idunn.deployment_aftercare_audit.v0\"".to_string(),
        "daemon_owns_execution = true".to_string(),
        String::new(),
        "[required_receipts]".to_string(),
        "mind_adoption = \"epiphany.repo_work_mind_adoption_decision.v0\"".to_string(),
        "soul_review = \"epiphany.repo_work_closure_review.v0\"".to_string(),
        "maintainer_review = \"gamecult.maintainer.review_receipt.v0\"".to_string(),
        "secret_policy = \"epiphany.repo_secret_policy_request.v0\"".to_string(),
        "idunn_deployment = \"gamecult.idunn.deployment_receipt.v0\"".to_string(),
        "aftercare_audit = \"gamecult.idunn.deployment_aftercare_audit.v0\"".to_string(),
        String::new(),
        "[authority]".to_string(),
        "configuration_only = true".to_string(),
        "direct_deployment_authority = false".to_string(),
        "direct_ssh_authority = false".to_string(),
        "direct_git_push_authority = false".to_string(),
        "direct_service_lifecycle_authority = false".to_string(),
        "direct_hands_authority = false".to_string(),
        "publication_authorized = false".to_string(),
        "merge_authorized = false".to_string(),
        "cross_body_mutation_authorized = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "idunn_deployment_authority_required = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the deployment config path changed and contains the accepted pressure summary.\",".to_string(),
        "  \"Soul verifies the config names git-push-observed-by-Idunn trigger, script review/hash, secret policy, rollback, aftercare, and Idunn receipt contracts.\",".to_string(),
        "  \"Soul verifies this config is disabled until reviewed and grants no direct deployment, SSH, git push, service lifecycle, publication, merge, Hands, cross-body, or private Verse authority.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove or disable the deployment config if the Idunn deployment path is not ready.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.deployment_config".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived an Idunn-facing repo deployment config from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add deployment config for repo work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo deployment config path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the config names git-push-observed-by-Idunn trigger, script review/hash, secret policy, rollback, aftercare, Idunn receipts, and no direct deployment/SSH/push authority.".to_string(),
            "Soul verifies no paths outside the declared deployment config changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove or disable the generated deployment config if Idunn review is not ready.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.deployment_config"),
    })
}

fn closure_family_assertions(
    workspace: &Path,
    commit_sha: &str,
    execute_receipt: &Value,
    item: &str,
) -> Result<(Value, bool)> {
    let Some(plan_receipt_path) = path_from_json(execute_receipt, &["planReceiptPath"]) else {
        return Ok((
            json!({
                "status": "failed",
                "reason": "execute receipt has no planReceiptPath",
                "assertions": []
            }),
            false,
        ));
    };
    let plan_receipt = read_json_if_exists(&plan_receipt_path)?.unwrap_or(Value::Null);
    if plan_receipt.is_null() {
        return Ok((
            json!({
                "status": "failed",
                "reason": "plan receipt path is missing",
                "planReceiptPath": plan_receipt_path,
                "assertions": []
            }),
            false,
        ));
    }
    let safe_family = string_from_json(&plan_receipt, &["derivation", "safeActionFamily"])
        .unwrap_or_else(|| "manual-or-unknown".to_string());
    let presentation_only = matches!(
        safe_family.as_str(),
        "repo.append_worklog"
            | "repo.markdown_planning_note"
            | "repo.checklist_note"
            | "repo.markdown_managed_section"
            | "repo.status_section"
            | "repo.task_card"
    );
    let target_paths = first_plan_action(&plan_receipt)
        .map(|action| string_array_field(action, "changedPaths"))
        .unwrap_or_else(|| string_array_from_json(execute_receipt, &["changedPaths"]));
    let target_path = target_paths.first().cloned().unwrap_or_default();
    if target_path.is_empty() {
        return Ok((
            json!({
                "status": "failed",
                "safeActionFamily": safe_family,
                "reason": "plan and execute receipts have no target changed path",
                "assertions": []
            }),
            false,
        ));
    }
    let summary = string_from_json(&plan_receipt, &["derivation", "inputPressure", "summary"])
        .or_else(|| string_from_json(&plan_receipt, &["objective"]))
        .unwrap_or_else(|| item.to_string());
    let compact_summary = compact_line(&summary);
    let committed_content = read_committed_file(workspace, commit_sha, &target_path)?;
    let mut assertions = Vec::new();
    push_assertion(
        &mut assertions,
        "target-blob-present",
        committed_content.is_some(),
        format!("Committed target {target_path} exists at {commit_sha}."),
    );
    let content = committed_content.unwrap_or_default();
    match safe_family.as_str() {
        "repo.append_worklog"
        | "repo.markdown_planning_note"
        | "repo.checklist_note"
        | "repo.markdown_managed_section"
        | "repo.status_section"
        | "repo.task_card" => {
            push_assertion(
                &mut assertions,
                "presentation-only-family",
                true,
                "Presentation formatting carries no closure authority; committed target/path evidence remains authoritative.".to_string(),
            );
        }
        "repo.tool_request" => {
            let request = parse_repo_tool_request(&content).ok();
            push_assertion(
                &mut assertions,
                "tool-request-schema-present",
                request
                    .as_ref()
                    .is_some_and(RepoToolRequest::has_canonical_identity),
                "Committed tool request carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "tool-request-family-present",
                request
                    .as_ref()
                    .is_some_and(RepoToolRequest::has_canonical_identity),
                "Committed tool request carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "tool-request-summary-present",
                request
                    .as_ref()
                    .is_some_and(|request| request.summary == compact_summary),
                "Committed tool request contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "tool-request-section-present",
                request
                    .as_ref()
                    .is_some_and(RepoToolRequest::has_request_contract),
                "Committed tool request names the daemon tool directory and typed intent operation."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "tool-request-cultmesh-contracts",
                request
                    .as_ref()
                    .is_some_and(RepoToolRequest::has_cultmesh_contract),
                "Committed tool request names typed CultMesh contracts, host liveness, and host execution ownership.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "tool-request-odin-provider-ownership",
                request
                    .as_ref()
                    .is_some_and(RepoToolRequest::has_odin_contract),
                "Committed tool request preserves Odin discovery and provider ownership boundaries."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "tool-request-authority-seals",
                request
                    .as_ref()
                    .is_some_and(RepoToolRequest::has_authority_seals),
                "Committed tool request denies direct execution, shell, Hands, state, publication, lifecycle, cross-body, and private-rummaging authority.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "tool-request-private-seal",
                request
                    .as_ref()
                    .is_some_and(|request| !request.private_state_exposed),
                "Committed tool request preserves the private-state seal.".to_string(),
            );
        }
        "repo.collaboration_policy" => {
            let policy = parse_repo_collaboration_policy(&content).ok();
            push_assertion(
                &mut assertions,
                "collaboration-policy-schema-present",
                policy
                    .as_ref()
                    .is_some_and(RepoCollaborationPolicy::has_canonical_identity),
                "Committed collaboration policy carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-policy-family-present",
                policy
                    .as_ref()
                    .is_some_and(RepoCollaborationPolicy::has_canonical_identity),
                "Committed collaboration policy carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-policy-summary-present",
                policy
                    .as_ref()
                    .is_some_and(|value| value.summary == compact_summary),
                "Committed collaboration policy contains the accepted pressure summary."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-policy-body-truth",
                policy
                    .as_ref()
                    .is_some_and(RepoCollaborationPolicy::preserves_provider_truth)
                    && policy
                        .as_ref()
                        .is_some_and(RepoCollaborationPolicy::remains_proposal),
                "Committed collaboration policy preserves provider ownership.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-policy-verse-boundaries",
                policy.as_ref().is_some_and(RepoCollaborationPolicy::has_verse_boundaries),
                "Committed collaboration policy names private/local/public Verse boundaries and Odin discovery.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-policy-eve-connection",
                policy
                    .as_ref()
                    .is_some_and(RepoCollaborationPolicy::has_eve_request),
                "Committed collaboration policy names the Eve connection contract.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-policy-persona-feedback",
                policy.as_ref().is_some_and(RepoCollaborationPolicy::has_persona_route),
                "Committed collaboration policy routes Persona/human/peer feedback through audited public discussion.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-policy-imagination-route",
                policy.as_ref().is_some_and(RepoCollaborationPolicy::has_imagination_route),
                "Committed collaboration policy routes feedback to Imagination before Mind/Bifrost gates.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-policy-authority-seals",
                policy.as_ref().is_some_and(RepoCollaborationPolicy::has_authority_seals),
                "Committed collaboration policy denies direct action/state/publication/service/cross-body authority.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-policy-private-seal",
                policy
                    .as_ref()
                    .is_some_and(|value| !value.private_state_exposed),
                "Committed collaboration policy preserves the private-state seal.".to_string(),
            );
        }
        "repo.collaboration_topic" => {
            let topic = parse_repo_collaboration_topic(&content).ok();
            push_assertion(
                &mut assertions,
                "collaboration-topic-schema-present",
                topic
                    .as_ref()
                    .is_some_and(RepoCollaborationTopic::has_canonical_identity),
                "Committed collaboration topic carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-topic-family-present",
                topic
                    .as_ref()
                    .is_some_and(RepoCollaborationTopic::has_canonical_identity),
                "Committed collaboration topic carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-topic-summary-present",
                topic
                    .as_ref()
                    .is_some_and(|value| value.summary == compact_summary),
                "Committed collaboration topic contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-topic-public-surface",
                topic
                    .as_ref()
                    .is_some_and(RepoCollaborationTopic::remains_unpublished_proposal),
                "Committed collaboration topic names public discussion and Eve surfaces."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-topic-imagination-route",
                topic.as_ref().is_some_and(RepoCollaborationTopic::has_imagination_route),
                "Committed collaboration topic routes feedback to Imagination consensus before adoption.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-topic-authority-seals",
                topic.as_ref().is_some_and(RepoCollaborationTopic::has_authority_seals),
                "Committed collaboration topic denies action, publication, cross-body, and private-rummaging authority.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-topic-private-seal",
                topic
                    .as_ref()
                    .is_some_and(|value| !value.private_state_exposed),
                "Committed collaboration topic preserves the private-state seal.".to_string(),
            );
        }
        "repo.consensus_brief" => {
            let brief = parse_repo_consensus_brief(&content).ok();
            push_assertion(
                &mut assertions,
                "consensus-brief-schema-present",
                brief
                    .as_ref()
                    .is_some_and(RepoConsensusBrief::has_canonical_identity),
                "Committed consensus brief carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "consensus-brief-family-present",
                brief
                    .as_ref()
                    .is_some_and(RepoConsensusBrief::has_canonical_identity),
                "Committed consensus brief carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "consensus-brief-summary-present",
                brief
                    .as_ref()
                    .is_some_and(|value| value.summary == compact_summary),
                "Committed consensus brief contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "consensus-brief-draft-state",
                brief
                    .as_ref()
                    .is_some_and(RepoConsensusBrief::remains_unconverged_draft),
                "Committed consensus brief remains a draft requiring review.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "consensus-brief-imagination-route",
                brief.as_ref().is_some_and(RepoConsensusBrief::has_imagination_route),
                "Committed consensus brief routes through Imagination without private Verse rummaging.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "consensus-brief-inputs-present",
                brief
                    .as_ref()
                    .is_some_and(RepoConsensusBrief::has_input_contract),
                "Committed consensus brief preserves public feedback and candidate-action inputs."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "consensus-brief-authority-seals",
                brief.as_ref().is_some_and(RepoConsensusBrief::has_authority_seals),
                "Committed consensus brief denies adoption/action/publication/cross-body authority and requires Mind/Bifrost gates.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "consensus-brief-private-seal",
                brief
                    .as_ref()
                    .is_some_and(|value| !value.private_state_exposed),
                "Committed consensus brief preserves the private-state seal.".to_string(),
            );
        }
        "repo.interpreter_brief" => {
            let brief = parse_repo_interpreter_brief(&content).ok();
            push_assertion(
                &mut assertions,
                "interpreter-brief-schema-present",
                brief
                    .as_ref()
                    .is_some_and(RepoInterpreterBrief::has_canonical_identity),
                "Committed interpreter brief carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "interpreter-brief-family-present",
                brief
                    .as_ref()
                    .is_some_and(RepoInterpreterBrief::has_canonical_identity),
                "Committed interpreter brief carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "interpreter-brief-summary-present",
                brief
                    .as_ref()
                    .is_some_and(|value| value.summary == compact_summary),
                "Committed interpreter brief contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "interpreter-brief-draft-state",
                brief.as_ref().is_some_and(RepoInterpreterBrief::is_imagination_request_for_mind),
                "Committed interpreter brief remains an Imagination-authored request for Mind interpretation."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "interpreter-brief-semantic-checks",
                brief
                    .as_ref()
                    .is_some_and(RepoInterpreterBrief::has_semantic_checks),
                "Committed interpreter brief names semantic checks before action adoption."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "interpreter-brief-allowed-outputs",
                brief.as_ref().is_some_and(RepoInterpreterBrief::has_allowed_outputs),
                "Committed interpreter brief limits outputs to candidate families or more planning."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "interpreter-brief-gates",
                brief.as_ref().is_some_and(RepoInterpreterBrief::has_required_gates),
                "Committed interpreter brief names consensus, Mind, Soul, Bifrost, Hands, Substrate, and Idunn gates."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "interpreter-brief-authority-seals",
                brief.as_ref().is_some_and(RepoInterpreterBrief::has_authority_seals),
                "Committed interpreter brief denies state/adoption/scheduling/substrate/action/shell/commit/publication/deployment/cross-body authority."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "interpreter-brief-private-seal",
                brief
                    .as_ref()
                    .is_some_and(|value| !value.private_state_exposed)
                    && brief
                        .as_ref()
                        .is_some_and(RepoInterpreterBrief::has_authority_seals),
                "Committed interpreter brief preserves private-state and transcript seals."
                    .to_string(),
            );
        }
        "repo.objective_draft" => {
            let draft = parse_repo_objective_draft(&content).ok();
            push_assertion(
                &mut assertions,
                "objective-draft-schema-present",
                draft
                    .as_ref()
                    .is_some_and(RepoObjectiveDraft::has_canonical_identity),
                "Committed Objective Draft carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "objective-draft-family-present",
                draft
                    .as_ref()
                    .is_some_and(RepoObjectiveDraft::has_canonical_identity),
                "Committed Objective Draft carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "objective-draft-summary-present",
                draft
                    .as_ref()
                    .is_some_and(|value| value.summary == compact_summary),
                "Committed Objective Draft contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "objective-draft-review-state",
                draft
                    .as_ref()
                    .is_some_and(RepoObjectiveDraft::remains_imagination_draft),
                "Committed Objective Draft remains review-required and unadopted.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "objective-draft-acceptance-criteria",
                draft
                    .as_ref()
                    .is_some_and(RepoObjectiveDraft::has_acceptance_contract),
                "Committed Objective Draft names acceptance criteria and downstream gates."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "objective-draft-inputs-present",
                draft.as_ref().is_some_and(RepoObjectiveDraft::has_input_contract),
                "Committed Objective Draft preserves discussion/action refs and requires consensus."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "objective-draft-authority-seals",
                draft.as_ref().is_some_and(RepoObjectiveDraft::has_authority_seals),
                "Committed Objective Draft denies adoption/scheduling/action/publication/cross-body authority and requires Mind/Bifrost gates.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "objective-draft-private-seal",
                draft
                    .as_ref()
                    .is_some_and(|value| !value.private_state_exposed),
                "Committed Objective Draft preserves the private-state seal.".to_string(),
            );
        }
        "repo.adoption_request" => {
            let request = parse_repo_adoption_request(&content).ok();
            push_assertion(
                &mut assertions,
                "adoption-request-schema-present",
                request
                    .as_ref()
                    .is_some_and(RepoAdoptionRequest::has_canonical_identity),
                "Committed adoption request carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "adoption-request-family-present",
                request
                    .as_ref()
                    .is_some_and(RepoAdoptionRequest::has_canonical_identity),
                "Committed adoption request carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "adoption-request-summary-present",
                request
                    .as_ref()
                    .is_some_and(|value| value.summary == compact_summary),
                "Committed adoption request contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "adoption-request-awaits-mind",
                request.as_ref().is_some_and(RepoAdoptionRequest::awaits_mind_review),
                "Committed adoption request waits for Mind review before state or scheduling consequence.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "adoption-request-decision-contract",
                request.as_ref().is_some_and(RepoAdoptionRequest::has_decision_contract),
                "Committed adoption request names the Mind decision contract without modifying state.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "adoption-request-inputs-present",
                request.as_ref().is_some_and(RepoAdoptionRequest::has_input_contract),
                "Committed adoption request preserves public inputs and requires draft/consensus antecedents.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "adoption-request-authority-seals",
                request.as_ref().is_some_and(RepoAdoptionRequest::has_authority_seals),
                "Committed adoption request denies adoption/state/scheduling/action/publication/cross-body authority.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "adoption-request-private-seal",
                request
                    .as_ref()
                    .is_some_and(|value| !value.private_state_exposed),
                "Committed adoption request preserves the private-state seal.".to_string(),
            );
        }
        "repo.scheduling_request" => {
            let request = parse_repo_scheduling_request(&content).ok();
            push_assertion(
                &mut assertions,
                "scheduling-request-schema-present",
                request
                    .as_ref()
                    .is_some_and(RepoSchedulingRequest::has_canonical_identity),
                "Committed scheduling request carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "scheduling-request-family-present",
                request
                    .as_ref()
                    .is_some_and(RepoSchedulingRequest::has_canonical_identity),
                "Committed scheduling request carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "scheduling-request-summary-present",
                request
                    .as_ref()
                    .is_some_and(|value| value.summary == compact_summary),
                "Committed scheduling request contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "scheduling-request-awaits-mind-adoption",
                request.as_ref().is_some_and(RepoSchedulingRequest::awaits_mind_adoption),
                "Committed scheduling request waits for Mind adoption before Self queue consequence.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "scheduling-request-queue-contract",
                request
                    .as_ref()
                    .is_some_and(RepoSchedulingRequest::has_bounded_queue_contract),
                "Committed scheduling request names a bounded queue pulse contract.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "scheduling-request-receipt-contract",
                request.as_ref().is_some_and(RepoSchedulingRequest::has_receipt_contract),
                "Committed scheduling request requires Mind receipts and names the later Self receipt.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "scheduling-request-authority-seals",
                request.as_ref().is_some_and(RepoSchedulingRequest::has_authority_seals),
                "Committed scheduling request denies scheduling/queue/action/publication/cross-body authority and requires Mind/Bifrost gates.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "scheduling-request-private-seal",
                request
                    .as_ref()
                    .is_some_and(|value| !value.private_state_exposed),
                "Committed scheduling request preserves the private-state seal.".to_string(),
            );
        }
        "repo.work_order" => {
            let work_order = parse_repo_work_order(&content).ok();
            push_assertion(
                &mut assertions,
                "work-order-schema-present",
                work_order
                    .as_ref()
                    .is_some_and(RepoWorkOrder::has_canonical_identity),
                "Committed work order carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "work-order-family-present",
                work_order
                    .as_ref()
                    .is_some_and(RepoWorkOrder::has_canonical_identity),
                "Committed work order carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "work-order-summary-present",
                work_order
                    .as_ref()
                    .is_some_and(|value| value.summary == compact_summary),
                "Committed work order contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "work-order-awaits-hands-review",
                work_order
                    .as_ref()
                    .is_some_and(RepoWorkOrder::awaits_hands_review),
                "Committed work order waits for Hands review before implementation consequence."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "work-order-antecedents-present",
                work_order
                    .as_ref()
                    .is_some_and(RepoWorkOrder::has_antecedent_contract),
                "Committed work order preserves adoption and scheduling antecedents.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "work-order-receipt-contract",
                work_order
                    .as_ref()
                    .is_some_and(RepoWorkOrder::has_receipt_contract),
                "Committed work order names the Substrate/Hands/Soul/Mind receipt chain."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "work-order-scope-bounded",
                work_order
                    .as_ref()
                    .is_some_and(RepoWorkOrder::has_bounded_scope),
                "Committed work order scopes later action to bounded branch-local reviewable work."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "work-order-authority-seals",
                work_order.as_ref().is_some_and(RepoWorkOrder::has_authority_seals),
                "Committed work order denies substrate/shell/commit/action/publication/cross-body authority."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "work-order-private-seal",
                work_order
                    .as_ref()
                    .is_some_and(|value| !value.private_state_exposed),
                "Committed work order preserves the private-state seal.".to_string(),
            );
        }
        "repo.verification_request" => {
            let request = parse_repo_verification_request(&content).ok();
            push_assertion(
                &mut assertions,
                "verification-request-schema-present",
                request
                    .as_ref()
                    .is_some_and(RepoVerificationRequest::has_canonical_identity),
                "Committed verification request carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "verification-request-family-present",
                request
                    .as_ref()
                    .is_some_and(RepoVerificationRequest::has_canonical_identity),
                "Committed verification request carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "verification-request-summary-present",
                request
                    .as_ref()
                    .is_some_and(|request| request.summary == compact_summary),
                "Committed verification request contains the accepted pressure summary."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "verification-request-awaits-soul-review",
                request
                    .as_ref()
                    .is_some_and(RepoVerificationRequest::awaits_soul_review),
                "Committed verification request waits for Soul review before proof consequence."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "verification-request-antecedents-present",
                request
                    .as_ref()
                    .is_some_and(RepoVerificationRequest::has_antecedent_contract),
                "Committed verification request requires Substrate and Hands antecedents."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "verification-request-receipt-contract",
                request
                    .as_ref()
                    .is_some_and(RepoVerificationRequest::has_receipt_contract),
                "Committed verification request names Hands/Soul/closure/Mind receipt chain."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "verification-request-checks-present",
                request
                    .as_ref()
                    .is_some_and(RepoVerificationRequest::has_check_contract),
                "Committed verification request names the closure checks that Soul should run."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "verification-request-authority-seals",
                request.as_ref().is_some_and(RepoVerificationRequest::has_authority_seals),
                "Committed verification request denies verdict/state/action/rerun/publication/cross-body authority."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "verification-request-private-seal",
                request
                    .as_ref()
                    .is_some_and(|request| !request.private_state_exposed),
                "Committed verification request preserves the private-state seal.".to_string(),
            );
        }
        "repo.publication_request" => {
            let request = parse_repo_publication_request(&content).ok();
            push_assertion(
                &mut assertions,
                "publication-request-schema-present",
                request
                    .as_ref()
                    .is_some_and(RepoPublicationRequest::has_canonical_identity),
                "Committed publication request carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "publication-request-family-present",
                request
                    .as_ref()
                    .is_some_and(RepoPublicationRequest::has_canonical_identity),
                "Committed publication request carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "publication-request-summary-present",
                request
                    .as_ref()
                    .is_some_and(|request| request.summary == compact_summary),
                "Committed publication request contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "publication-request-awaits-bifrost-review",
                request
                    .as_ref()
                    .is_some_and(RepoPublicationRequest::awaits_bifrost_review),
                "Committed publication request waits for Bifrost review before public consequence."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "publication-request-antecedents-present",
                request.as_ref().is_some_and(RepoPublicationRequest::has_antecedent_contract),
                "Committed publication request requires closure, Soul, Mind, and redacted proof antecedents."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "publication-request-receipt-contract",
                request.as_ref().is_some_and(RepoPublicationRequest::has_receipt_contract),
                "Committed publication request names the Bifrost/GitHub/credit/upstream receipt chain."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "publication-request-redaction-contract",
                request
                    .as_ref()
                    .is_some_and(RepoPublicationRequest::has_redaction_contract),
                "Committed publication request preserves the public export redaction contract."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "publication-request-authority-seals",
                request.as_ref().is_some_and(RepoPublicationRequest::has_authority_seals),
                "Committed publication request denies publication/credit/merge/sync/action/cross-body authority."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "publication-request-private-seal",
                request
                    .as_ref()
                    .is_some_and(|request| !request.private_state_exposed),
                "Committed publication request preserves the private-state seal.".to_string(),
            );
        }
        "repo.sync_request" => {
            let request = parse_repo_sync_request(&content).ok();
            push_assertion(
                &mut assertions,
                "sync-request-schema-present",
                request
                    .as_ref()
                    .is_some_and(RepoSyncRequest::has_canonical_identity),
                "Committed sync request carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "sync-request-family-present",
                request
                    .as_ref()
                    .is_some_and(RepoSyncRequest::has_canonical_identity),
                "Committed sync request carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "sync-request-summary-present",
                request
                    .as_ref()
                    .is_some_and(|request| request.summary == compact_summary),
                "Committed sync request contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "sync-request-awaits-upstream-proof",
                request
                    .as_ref()
                    .is_some_and(RepoSyncRequest::awaits_upstream_proof),
                "Committed sync request waits for upstream-main proof instead of performing sync."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "sync-request-antecedents-present",
                request.as_ref().is_some_and(RepoSyncRequest::has_antecedent_contract),
                "Committed sync request requires publication, maintainer review, credit, and proof antecedents."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "sync-request-receipt-contract",
                request.as_ref().is_some_and(RepoSyncRequest::has_receipt_contract),
                "Committed sync request names publication, review, credit, sync, and ancestry proof receipts."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "sync-request-proof-contract",
                request
                    .as_ref()
                    .is_some_and(RepoSyncRequest::has_proof_contract),
                "Committed sync request names the upstream-main ancestry proof contract."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "sync-request-authority-seals",
                request.as_ref().is_some_and(RepoSyncRequest::has_authority_seals),
                "Committed sync request denies merge/push/sync/publication/credit/action/cross-body authority."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "sync-request-private-seal",
                request
                    .as_ref()
                    .is_some_and(|request| !request.private_state_exposed),
                "Committed sync request preserves the private-state seal.".to_string(),
            );
        }
        "repo.maintainer_review_request" => {
            let request = parse_repo_maintainer_review_request(&content).ok();
            push_assertion(
                &mut assertions,
                "maintainer-review-request-schema-present",
                request
                    .as_ref()
                    .is_some_and(RepoMaintainerReviewRequest::has_canonical_identity),
                "Committed maintainer review request carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "maintainer-review-request-family-present",
                request
                    .as_ref()
                    .is_some_and(RepoMaintainerReviewRequest::has_canonical_identity),
                "Committed maintainer review request carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "maintainer-review-request-summary-present",
                request
                    .as_ref()
                    .is_some_and(|request| request.summary == compact_summary),
                "Committed maintainer review request contains the accepted pressure summary."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "maintainer-review-request-awaits-review",
                request
                    .as_ref()
                    .is_some_and(RepoMaintainerReviewRequest::awaits_review),
                "Committed maintainer review request waits for human review before consequence."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "maintainer-review-request-antecedents-present",
                request.as_ref().is_some_and(RepoMaintainerReviewRequest::has_antecedent_contract),
                "Committed maintainer review request requires closure, Soul, Mind, proof, and Bifrost request antecedents."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "maintainer-review-request-receipt-contract",
                request.as_ref().is_some_and(RepoMaintainerReviewRequest::has_receipt_contract),
                "Committed maintainer review request names closure, proof, maintainer, and Bifrost receipts."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "maintainer-review-request-review-packet",
                request.as_ref().is_some_and(RepoMaintainerReviewRequest::has_review_packet),
                "Committed maintainer review request names reviewer identity, verdict, changed path, proof, and redaction requirements."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "maintainer-review-request-authority-seals",
                request.as_ref().is_some_and(RepoMaintainerReviewRequest::has_authority_seals),
                "Committed maintainer review request denies approval/merge/push/publication/sync/action/cross-body authority."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "maintainer-review-request-private-seal",
                request
                    .as_ref()
                    .is_some_and(|request| !request.private_state_exposed),
                "Committed maintainer review request preserves the private-state seal.".to_string(),
            );
        }
        "repo.pr_request" => {
            let request = parse_repo_pr_request(&content).ok();
            push_assertion(
                &mut assertions,
                "pr-request-schema-present",
                request
                    .as_ref()
                    .is_some_and(RepoPrRequest::has_canonical_identity),
                "Committed PR request carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "pr-request-family-present",
                request
                    .as_ref()
                    .is_some_and(RepoPrRequest::has_canonical_identity),
                "Committed PR request carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "pr-request-summary-present",
                request
                    .as_ref()
                    .is_some_and(|request| request.summary == compact_summary),
                "Committed PR request contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "pr-request-awaits-publication-review",
                request
                    .as_ref()
                    .is_some_and(RepoPrRequest::awaits_owned_review),
                "Committed PR request separates Self routing, Bifrost publication, Hands execution, and GitHub provider outcome."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "pr-request-antecedents-present",
                request.as_ref().is_some_and(RepoPrRequest::has_antecedent_contract),
                "Committed PR request requires closure, Soul, Mind, proof, review, Bifrost, and credit antecedents."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "pr-request-receipt-contract",
                request.as_ref().is_some_and(RepoPrRequest::has_receipt_contract),
                "Committed PR request names closure, proof, review, Bifrost, credit, and PR publication receipts."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "pr-request-packet-contract",
                request.as_ref().is_some_and(RepoPrRequest::has_packet_contract),
                "Committed PR request names branch, title/body, path, proof, review, credit, and redaction requirements."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "pr-request-authority-seals",
                request.as_ref().is_some_and(RepoPrRequest::has_authority_seals),
                "Committed PR request denies PR/push/merge/publication/sync/action/cross-body authority."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "pr-request-private-seal",
                request
                    .as_ref()
                    .is_some_and(|request| !request.private_state_exposed),
                "Committed PR request preserves the private-state seal.".to_string(),
            );
        }
        "repo.credit_request" => {
            let request = parse_repo_credit_request(&content).ok();
            push_assertion(
                &mut assertions,
                "credit-request-schema-present",
                request
                    .as_ref()
                    .is_some_and(RepoCreditRequest::has_canonical_identity),
                "Committed credit request carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "credit-request-family-present",
                request
                    .as_ref()
                    .is_some_and(RepoCreditRequest::has_canonical_identity),
                "Committed credit request carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "credit-request-summary-present",
                request
                    .as_ref()
                    .is_some_and(|request| request.summary == compact_summary),
                "Committed credit request contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "credit-request-awaits-bifrost-review",
                request
                    .as_ref()
                    .is_some_and(RepoCreditRequest::awaits_bifrost_review),
                "Committed credit request waits for Bifrost credit review before consequence."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "credit-request-antecedents-present",
                request.as_ref().is_some_and(RepoCreditRequest::has_antecedent_contract),
                "Committed credit request requires closure, Soul, Mind, proof, review, accepted artifact, and authorship antecedents."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "credit-request-receipt-contract",
                request.as_ref().is_some_and(RepoCreditRequest::has_receipt_contract),
                "Committed credit request names closure, proof, review, accepted artifact, ledger, and readback receipts."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "credit-request-packet-contract",
                request.as_ref().is_some_and(RepoCreditRequest::has_credit_packet),
                "Committed credit request names author, reviewer, artifact, proof, path, ledger, and redaction requirements."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "credit-request-authority-seals",
                request.as_ref().is_some_and(RepoCreditRequest::has_authority_seals),
                "Committed credit request denies credit/publication/PR/merge/sync/action/cross-body authority."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "credit-request-private-seal",
                request
                    .as_ref()
                    .is_some_and(|request| !request.private_state_exposed),
                "Committed credit request preserves the private-state seal.".to_string(),
            );
        }
        "repo.artifact_acceptance_request" => {
            let request = parse_repo_artifact_acceptance_request(&content).ok();
            push_assertion(
                &mut assertions,
                "artifact-acceptance-request-schema-present",
                request
                    .as_ref()
                    .is_some_and(RepoArtifactAcceptanceRequest::has_canonical_identity),
                "Committed artifact acceptance request carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "artifact-acceptance-request-family-present",
                request
                    .as_ref()
                    .is_some_and(RepoArtifactAcceptanceRequest::has_canonical_identity),
                "Committed artifact acceptance request carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "artifact-acceptance-request-summary-present",
                request
                    .as_ref()
                    .is_some_and(|request| request.summary == compact_summary),
                "Committed artifact acceptance request contains the accepted pressure summary."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "artifact-acceptance-request-awaits-review",
                request
                    .as_ref()
                    .is_some_and(RepoArtifactAcceptanceRequest::awaits_owned_review),
                "Committed artifact acceptance request waits for maintainer/Bifrost review before consequence."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "artifact-acceptance-request-antecedents-present",
                request
                    .as_ref()
                    .is_some_and(RepoArtifactAcceptanceRequest::has_antecedent_contract),
                "Committed artifact acceptance request requires closure, Soul, Mind, proof, review, and Hands commit antecedents."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "artifact-acceptance-request-receipt-contract",
                request
                    .as_ref()
                    .is_some_and(RepoArtifactAcceptanceRequest::has_receipt_contract),
                "Committed artifact acceptance request names closure, proof, review, Hands commit, and accepted artifact receipts."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "artifact-acceptance-request-packet-contract",
                request
                    .as_ref()
                    .is_some_and(RepoArtifactAcceptanceRequest::has_artifact_packet),
                "Committed artifact acceptance request names artifact, commit, path, review, proof, rationale, and redaction requirements."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "artifact-acceptance-request-authority-seals",
                request
                    .as_ref()
                    .is_some_and(RepoArtifactAcceptanceRequest::has_authority_seals),
                "Committed artifact acceptance request denies acceptance/credit/PR/merge/publication/sync/action/cross-body authority."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "artifact-acceptance-request-private-seal",
                request
                    .as_ref()
                    .is_some_and(|request| !request.private_state_exposed),
                "Committed artifact acceptance request preserves the private-state seal."
                    .to_string(),
            );
        }
        "repo.metrics_request" => {
            let request = parse_repo_metrics_request(&content).ok();
            push_assertion(
                &mut assertions,
                "metrics-request-schema-present",
                request
                    .as_ref()
                    .is_some_and(RepoMetricsRequest::has_canonical_identity),
                "Committed metrics request carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "metrics-request-family-present",
                request
                    .as_ref()
                    .is_some_and(RepoMetricsRequest::has_canonical_identity),
                "Committed metrics request carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "metrics-request-summary-present",
                request
                    .as_ref()
                    .is_some_and(|request| request.summary == compact_summary),
                "Committed metrics request contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "metrics-request-awaits-review",
                request
                    .as_ref()
                    .is_some_and(RepoMetricsRequest::awaits_owned_review),
                "Committed metrics request waits for Bifrost/maintainer review before consequence."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "metrics-request-antecedents-present",
                request
                    .as_ref()
                    .is_some_and(RepoMetricsRequest::has_antecedent_contract),
                "Committed metrics request requires closure, Soul, Mind, proof, accepted artifact, and credit request antecedents."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "metrics-request-receipt-contract",
                request
                    .as_ref()
                    .is_some_and(RepoMetricsRequest::has_receipt_contract),
                "Committed metrics request names closure, proof, artifact, spend, review-load, and credit readback receipts."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "metrics-request-packet-contract",
                request
                    .as_ref()
                    .is_some_and(RepoMetricsRequest::has_metrics_packet),
                "Committed metrics request names model-call, cost, review-load, artifact, proof, credit, and redaction requirements."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "metrics-request-authority-seals",
                request
                    .as_ref()
                    .is_some_and(RepoMetricsRequest::has_authority_seals),
                "Committed metrics request denies metrics/spend/review/credit/PR/merge/publication/sync/action/cross-body authority."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "metrics-request-private-seal",
                request
                    .as_ref()
                    .is_some_and(|request| !request.private_state_exposed),
                "Committed metrics request preserves the private-state seal.".to_string(),
            );
        }
        "repo.readiness_review_request" => {
            let request = parse_repo_readiness_review_request(&content).ok();
            push_assertion(
                &mut assertions,
                "readiness-review-request-schema-present",
                request
                    .as_ref()
                    .is_some_and(RepoReadinessReviewRequest::has_canonical_identity),
                "Committed readiness review request carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "readiness-review-request-family-present",
                request
                    .as_ref()
                    .is_some_and(RepoReadinessReviewRequest::has_canonical_identity),
                "Committed readiness review request carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "readiness-review-request-summary-present",
                request
                    .as_ref()
                    .is_some_and(|value| value.summary == compact_summary),
                "Committed readiness review request contains the accepted pressure summary."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "readiness-review-request-awaits-review",
                request.as_ref().is_some_and(RepoReadinessReviewRequest::has_coherent_routing),
                "Committed readiness review request waits for maintainer/Soul/Mind/Bifrost review before consequence."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "readiness-review-request-antecedents-present",
                request
                    .as_ref()
                    .is_some_and(RepoReadinessReviewRequest::has_antecedent_contract),
                "Committed readiness review request requires all repo-swarm MVP organ proofs."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "readiness-review-request-receipt-contract",
                request
                    .as_ref()
                    .is_some_and(RepoReadinessReviewRequest::has_receipt_contract),
                "Committed readiness review request names the MVP proof receipt contract."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "readiness-review-request-packet-contract",
                request.as_ref().is_some_and(RepoReadinessReviewRequest::has_packet_contract),
                "Committed readiness review request names proof bundle, branch, upstream, Bifrost, Idunn, tool, redaction, reviewer, and verdict requirements."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "readiness-review-request-authority-seals",
                request.as_ref().is_some_and(RepoReadinessReviewRequest::has_authority_seals),
                "Committed readiness review request denies readiness/state/publication/PR/merge/sync/deploy/service/Hands/cross-body authority."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "readiness-review-request-private-seal",
                request
                    .as_ref()
                    .is_some_and(|value| !value.private_state_exposed),
                "Committed readiness review request preserves the private-state seal.".to_string(),
            );
        }
        "repo.doctrine_update_request" => {
            let request = parse_repo_doctrine_update_request(&content).ok();
            push_assertion(
                &mut assertions,
                "doctrine-update-request-schema-present",
                request
                    .as_ref()
                    .is_some_and(RepoDoctrineUpdateRequest::has_canonical_identity),
                "Committed doctrine update request carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "doctrine-update-request-family-present",
                request
                    .as_ref()
                    .is_some_and(RepoDoctrineUpdateRequest::has_canonical_identity),
                "Committed doctrine update request carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "doctrine-update-request-summary-present",
                request
                    .as_ref()
                    .is_some_and(|value| value.summary == compact_summary),
                "Committed doctrine update request contains the accepted pressure summary."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "doctrine-update-request-awaits-review",
                request.as_ref().is_some_and(RepoDoctrineUpdateRequest::has_coherent_routing),
                "Committed doctrine update request waits for Mind/maintainer review before doctrine consequence."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "doctrine-update-request-antecedents-present",
                request.as_ref().is_some_and(RepoDoctrineUpdateRequest::has_antecedent_contract),
                "Committed doctrine update request requires feedback, Imagination, Mind, Soul, and maintainer antecedents."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "doctrine-update-request-receipt-contract",
                request.as_ref().is_some_and(RepoDoctrineUpdateRequest::has_receipt_contract),
                "Committed doctrine update request names Imagination, Mind, Soul, maintainer, and Hands receipts."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "doctrine-update-request-packet-contract",
                request.as_ref().is_some_and(RepoDoctrineUpdateRequest::has_packet_contract),
                "Committed doctrine update request names doctrine diff, invariant, rehydration, rollback, and redaction requirements."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "doctrine-update-request-authority-seals",
                request.as_ref().is_some_and(RepoDoctrineUpdateRequest::has_authority_seals),
                "Committed doctrine update request denies doctrine/action/state/publication/service/cross-body authority."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "doctrine-update-request-private-seal",
                request
                    .as_ref()
                    .is_some_and(|value| !value.private_state_exposed),
                "Committed doctrine update request preserves the private-state seal.".to_string(),
            );
        }
        "repo.secret_policy_request" => {
            let parsed = parse_repo_secret_policy_request(&content);
            let request = parsed.as_ref().ok();
            push_assertion(
                &mut assertions,
                "secret-policy-request-typed-toml",
                parsed.is_ok(),
                match parsed.as_ref() {
                    Ok(_) => "Committed secret policy request parses as typed TOML.".to_string(),
                    Err(error) => {
                        format!("Committed secret policy request parse failed: {error:#}")
                    }
                },
            );
            push_assertion(
                &mut assertions,
                "secret-policy-request-schema-present",
                request.is_some_and(RepoSecretPolicyRequest::has_canonical_identity),
                "Committed secret policy request carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "secret-policy-request-family-present",
                request.is_some_and(RepoSecretPolicyRequest::has_canonical_identity),
                "Committed secret policy request carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "secret-policy-request-summary-present",
                request.is_some_and(|request| request.summary == compact_summary),
                "Committed secret policy request contains the accepted pressure summary."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "secret-policy-request-awaits-review",
                request.is_some_and(RepoSecretPolicyRequest::awaits_security_review),
                "Committed secret policy request waits for security review before consequence."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "secret-policy-request-antecedents-present",
                request.is_some_and(RepoSecretPolicyRequest::has_review_antecedents),
                "Committed secret policy request requires source grounding, Soul, Mind, maintainer, and Bifrost review antecedents."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "secret-policy-request-receipt-contract",
                request.is_some_and(RepoSecretPolicyRequest::has_receipt_contract),
                "Committed secret policy request names Eyes, Soul, Mind, maintainer, and Bifrost review receipts."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "secret-policy-request-packet-contract",
                request.is_some_and(RepoSecretPolicyRequest::has_security_packet_contract),
                "Committed secret policy request names secret-location, credential-owner, write-scope, export, deployment, and rollback requirements."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "secret-policy-request-authority-seals",
                request.is_some_and(RepoSecretPolicyRequest::has_authority_seals),
                "Committed secret policy request denies secret/write/deployment/publication/service/cross-body authority."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "secret-policy-request-private-seal",
                request.is_some_and(|request| !request.private_state_exposed),
                "Committed secret policy request preserves the private-state seal.".to_string(),
            );
        }
        "repo.dependency_policy_request" => {
            let parsed = parse_repo_dependency_policy_request(&content);
            let request = parsed.as_ref().ok();
            push_assertion(
                &mut assertions,
                "dependency-policy-request-typed-toml",
                parsed.is_ok(),
                match parsed.as_ref() {
                    Ok(_) => {
                        "Committed dependency policy request parses as typed TOML.".to_string()
                    }
                    Err(error) => {
                        format!("Committed dependency policy request parse failed: {error:#}")
                    }
                },
            );
            push_assertion(
                &mut assertions,
                "dependency-policy-request-schema-present",
                request.is_some_and(RepoDependencyPolicyRequest::has_canonical_identity),
                "Committed dependency policy request carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "dependency-policy-request-family-present",
                request.is_some_and(RepoDependencyPolicyRequest::has_canonical_identity),
                "Committed dependency policy request carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "dependency-policy-request-summary-present",
                request.is_some_and(|request| request.summary == compact_summary),
                "Committed dependency policy request contains the accepted pressure summary."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "dependency-policy-request-awaits-review",
                request.is_some_and(RepoDependencyPolicyRequest::awaits_review),
                "Committed dependency policy request waits for maintainer/Soul/Bifrost review before consequence."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "dependency-policy-request-antecedents-present",
                request.is_some_and(RepoDependencyPolicyRequest::has_antecedents),
                "Committed dependency policy request requires Eyes, Soul, Mind, maintainer, and Bifrost antecedents."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "dependency-policy-request-receipt-contract",
                request.is_some_and(RepoDependencyPolicyRequest::has_receipt_contract),
                "Committed dependency policy request names Eyes, Soul, Mind, maintainer, Bifrost, and dependency-audit receipts."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "dependency-policy-request-packet-contract",
                request.is_some_and(RepoDependencyPolicyRequest::has_packet_contract),
                "Committed dependency policy request names manifest, lockfile, package-manager, vulnerability, license, vendored-code, update cadence, and redaction requirements."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "dependency-policy-request-authority-seals",
                request.is_some_and(RepoDependencyPolicyRequest::has_authority_seals),
                "Committed dependency policy request denies dependency/package/lockfile/network/CI/action/publication/service/cross-body authority."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "dependency-policy-request-private-seal",
                request.is_some_and(|request| !request.private_state_exposed),
                "Committed dependency policy request preserves the private-state seal.".to_string(),
            );
        }
        "repo.deployment_request" => {
            let parsed = parse_repo_deployment_request(&content);
            let request = parsed.as_ref().ok();
            push_assertion(
                &mut assertions,
                "deployment-request-typed-toml",
                parsed.is_ok(),
                match parsed.as_ref() {
                    Ok(_) => "Committed deployment request parses as typed TOML.".to_string(),
                    Err(error) => format!("Committed deployment request parse failed: {error:#}"),
                },
            );
            push_assertion(
                &mut assertions,
                "deployment-request-schema-present",
                request.is_some_and(RepoDeploymentRequest::has_canonical_identity),
                "Committed deployment request carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "deployment-request-family-present",
                request.is_some_and(RepoDeploymentRequest::has_canonical_identity),
                "Committed deployment request carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "deployment-request-summary-present",
                request.is_some_and(|request| request.summary == compact_summary),
                "Committed deployment request contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "deployment-request-awaits-idunn-review",
                request.is_some_and(RepoDeploymentRequest::awaits_idunn_review),
                "Committed deployment request waits for Idunn/maintainer review before deployment consequence."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "deployment-request-antecedents-present",
                request.is_some_and(RepoDeploymentRequest::has_antecedents),
                "Committed deployment request requires Eyes, Mind, Soul, maintainer, secret-policy, and Bifrost antecedents."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "deployment-request-receipt-contract",
                request.is_some_and(RepoDeploymentRequest::has_receipt_contract),
                "Committed deployment request names Eyes, Mind, Soul, maintainer, secret-policy, Idunn deployment, and aftercare receipts."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "deployment-request-packet-contract",
                request.is_some_and(RepoDeploymentRequest::has_packet_contract),
                "Committed deployment request names environment, git ref, script review, host policy, secret policy, rollback, and aftercare requirements."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "deployment-request-authority-seals",
                request.is_some_and(RepoDeploymentRequest::has_authority_seals),
                "Committed deployment request denies deployment/SSH/push/service/action/publication/cross-body authority."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "deployment-request-private-seal",
                request.is_some_and(|request| !request.private_state_exposed),
                "Committed deployment request preserves the private-state seal.".to_string(),
            );
        }
        "repo.deployment_config" => {
            let parsed = parse_repo_deployment_config(&content);
            let config = parsed.as_ref().ok();
            push_assertion(
                &mut assertions,
                "deployment-config-typed-toml",
                parsed.is_ok(),
                match parsed.as_ref() {
                    Ok(_) => "Committed deployment config parses as typed TOML.".to_string(),
                    Err(error) => format!("Committed deployment config parse failed: {error:#}"),
                },
            );
            push_assertion(
                &mut assertions,
                "deployment-config-schema-present",
                config.is_some_and(|config| config.has_canonical_identity()),
                "Committed deployment config carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "deployment-config-family-present",
                config.is_some_and(|config| config.has_canonical_identity()),
                "Committed deployment config carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "deployment-config-summary-present",
                config.is_some_and(|config| config.summary == compact_summary),
                "Committed deployment config contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "deployment-config-idunn-trigger",
                config.is_some_and(|config| config.has_idunn_trigger_contract()),
                "Committed deployment config names disabled Idunn git-push trigger, reviewed script/hash, policy, rollback, and aftercare requirements."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "deployment-config-cultmesh-contract",
                config.is_some_and(|config| config.has_cultmesh_contract()),
                "Committed deployment config routes deployment through Idunn CultMesh contracts."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "deployment-config-receipt-contract",
                config.is_some_and(|config| config.has_required_receipt_contract()),
                "Committed deployment config names Mind, Soul, maintainer, secret-policy, Idunn deployment, and aftercare receipts."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "deployment-config-authority-seals",
                config.is_some_and(|config| config.has_authority_seals()),
                "Committed deployment config denies deployment/SSH/push/service/action/publication/cross-body authority."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "deployment-config-private-seal",
                config.is_some_and(|config| !config.private_state_exposed),
                "Committed deployment config preserves the private-state seal.".to_string(),
            );
        }
        _ => {
            return Ok((
                json!({
                    "status": "skipped",
                    "safeActionFamily": safe_family,
                    "targetPath": target_path,
                    "reason": "safe family is manual or has no deterministic closure assertions",
                    "assertions": assertions
                }),
                true,
            ));
        }
    }
    let passed = assertions
        .iter()
        .all(|assertion| assertion.get("passed").and_then(Value::as_bool) == Some(true));
    Ok((
        json!({
            "status": if passed { "passed" } else { "failed" },
            "safeActionFamily": safe_family,
            "presentationOnly": presentation_only,
            "targetPath": target_path,
            "planReceiptPath": plan_receipt_path,
            "assertions": assertions
        }),
        passed,
    ))
}

fn push_assertion(assertions: &mut Vec<Value>, id: &str, passed: bool, summary: String) {
    assertions.push(json!({
        "assertionId": id,
        "passed": passed,
        "summary": summary,
    }));
}

fn closure_mind_adoption_review(execute_receipt: &Value) -> Result<(Value, bool)> {
    let Some(adopt_receipt_path) = path_from_json(execute_receipt, &["adoptReceiptPath"]) else {
        return Ok((
            json!({
                "schemaVersion": "epiphany.repo_work_mind_adoption_closure_review.v0",
                "status": "failed",
                "reason": "execute receipt has no adoptReceiptPath",
                "assertions": []
            }),
            false,
        ));
    };
    let adopt_receipt = read_json_if_exists(&adopt_receipt_path)?.unwrap_or(Value::Null);
    if adopt_receipt.is_null() {
        return Ok((
            json!({
                "schemaVersion": "epiphany.repo_work_mind_adoption_closure_review.v0",
                "status": "failed",
                "reason": "adopt receipt path is missing",
                "adoptReceiptPath": adopt_receipt_path,
                "assertions": []
            }),
            false,
        ));
    }
    let Some(mind_decision) = adopt_receipt.get("mindAdoptionDecision") else {
        return Ok((
            json!({
                "schemaVersion": "epiphany.repo_work_mind_adoption_closure_review.v0",
                "status": "failed",
                "reason": "adopt receipt has no mindAdoptionDecision",
                "adoptReceiptPath": adopt_receipt_path,
                "assertions": []
            }),
            false,
        ));
    };
    let plan_derived = execute_receipt.get("planReceiptPath").is_some();
    let mut assertions = Vec::new();
    push_assertion(
        &mut assertions,
        "decision-schema",
        string_from_json(mind_decision, &["schemaVersion"]).as_deref()
            == Some("epiphany.repo_work_mind_adoption_decision.v0"),
        "Mind adoption decision carries the expected schema.".to_string(),
    );
    push_assertion(
        &mut assertions,
        "decision-status-adopted",
        string_from_json(mind_decision, &["status"]).as_deref()
            == Some("adopted-for-branch-local-hands"),
        "Mind adoption decision accepted branch-local Hands authority.".to_string(),
    );
    push_assertion(
        &mut assertions,
        "decision-owner-mind",
        string_from_json(mind_decision, &["owner"]).as_deref() == Some("Mind"),
        "Mind is the recorded adoption decision owner.".to_string(),
    );
    push_assertion(
        &mut assertions,
        "decision-interpreter-mind",
        string_from_json(mind_decision, &["interpreter"]).as_deref() == Some("Mind"),
        "Mind is the recorded adoption interpreter.".to_string(),
    );
    push_assertion(
        &mut assertions,
        "action-item-accepted",
        bool_from_json(
            mind_decision,
            &["interpretation", "classification", "actionItemAccepted"],
        ) == Some(true),
        "Mind classified the plan cargo as an accepted action item.".to_string(),
    );
    push_assertion(
        &mut assertions,
        "safe-family-recognized",
        bool_from_json(
            mind_decision,
            &["interpretation", "classification", "safeFamilyRecognized"],
        ) == Some(true)
            && bool_from_json(mind_decision, &["gates", "safeFamilyRequired"]) == Some(true),
        "Mind recognized the safe family and recorded the safe-family gate.".to_string(),
    );
    push_assertion(
        &mut assertions,
        "requested-paths-match-plan",
        !plan_derived
            || (bool_from_json(
                mind_decision,
                &[
                    "interpretation",
                    "classification",
                    "requestedPathsMatchPlan",
                ],
            ) == Some(true)
                && bool_from_json(mind_decision, &["gates", "branchLocalOnly"]) == Some(true)),
        "Mind confirmed requested paths match the plan before branch-local Hands authority."
            .to_string(),
    );
    push_assertion(
        &mut assertions,
        "private-state-sealed",
        bool_from_json(mind_decision, &["privateStateExposed"]) == Some(false),
        "Mind adoption decision preserves the private-state seal.".to_string(),
    );

    let mind_decision_path = path_from_json(mind_decision, &["receiptPath"]);
    let standalone_review = if let Some(path) = mind_decision_path.as_ref() {
        match read_json_if_exists(path)? {
            Some(standalone) => {
                let matching_decision_id = string_from_json(&standalone, &["decisionId"])
                    == string_from_json(mind_decision, &["decisionId"]);
                let matching_status = string_from_json(&standalone, &["status"])
                    == string_from_json(mind_decision, &["status"]);
                let matching_schema = string_from_json(&standalone, &["schemaVersion"])
                    == string_from_json(mind_decision, &["schemaVersion"]);
                let matching_private_seal = bool_from_json(&standalone, &["privateStateExposed"])
                    == bool_from_json(mind_decision, &["privateStateExposed"]);
                push_assertion(
                    &mut assertions,
                    "standalone-decision-matches-snapshot",
                    matching_decision_id
                        && matching_status
                        && matching_schema
                        && matching_private_seal,
                    "Standalone Mind decision matches the adopt receipt snapshot.".to_string(),
                );
                json!({
                    "status": "present",
                    "receiptPath": path,
                    "decisionIdMatched": matching_decision_id,
                    "statusMatched": matching_status,
                    "schemaMatched": matching_schema,
                    "privateStateSealMatched": matching_private_seal
                })
            }
            None => {
                push_assertion(
                    &mut assertions,
                    "standalone-decision-present",
                    false,
                    "Standalone Mind decision path exists in snapshot but the file is missing."
                        .to_string(),
                );
                json!({
                    "status": "missing",
                    "receiptPath": path
                })
            }
        }
    } else {
        push_assertion(
            &mut assertions,
            "standalone-decision-path-present",
            false,
            "Adopt receipt snapshot names the standalone Mind decision receipt path.".to_string(),
        );
        json!({
            "status": "missing-path"
        })
    };

    let passed = assertions
        .iter()
        .all(|assertion| assertion.get("passed").and_then(Value::as_bool) == Some(true));
    Ok((
        json!({
            "schemaVersion": "epiphany.repo_work_mind_adoption_closure_review.v0",
            "status": if passed { "passed" } else { "failed" },
            "adoptReceiptPath": adopt_receipt_path,
            "mindDecisionPath": mind_decision_path,
            "planDerived": plan_derived,
            "assertions": assertions,
            "standaloneDecisionReview": standalone_review,
            "privateStateExposed": false
        }),
        passed,
    ))
}

fn record_repo_work_map_admission(
    runtime_store: &Path,
    item: &str,
    branch: &str,
    changed_paths: Vec<String>,
    commit_sha: &str,
    closure_review: &Value,
    modeling_summary: &str,
    modeling_finding_receipt_id: &str,
    modeling_route_id: &str,
    modeling_generation: u64,
    soul_verdict_receipt_id: &str,
    mind_gateway_review_id: &str,
    mind_state_commit_receipt_id: &str,
    execute_receipt_path: &Path,
    closure_review_path: &Path,
    closure_receipt_path: &Path,
    mind_review: &MindGatewayReview,
    mind_commit: &epiphany_core::MindStateCommitReceipt,
) -> Result<RepoWorkMapEntry> {
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let safe_action_family =
        string_from_json(closure_review, &["familyAssertions", "safeActionFamily"])
            .unwrap_or_else(|| "manual-or-unknown".to_string());
    let entry = RepoWorkMapEntry {
        schema_version: REPO_WORK_MAP_ENTRY_SCHEMA_VERSION.to_string(),
        map_entry_id: format!("repo-work-map-{}", sanitize(item)),
        admitted_at: now.clone(),
        item: item.to_string(),
        branch: branch.to_string(),
        changed_paths,
        commit_sha: commit_sha.to_string(),
        safe_action_family,
        modeling_summary: modeling_summary.to_string(),
        modeling_finding_receipt_id: modeling_finding_receipt_id.to_string(),
        soul_verdict_receipt_id: soul_verdict_receipt_id.to_string(),
        mind_gateway_review_id: mind_gateway_review_id.to_string(),
        mind_state_commit_receipt_id: mind_state_commit_receipt_id.to_string(),
        execute_receipt_path: normalize_path_for_receipt(execute_receipt_path),
        closure_review_path: normalize_path_for_receipt(closure_review_path),
        closure_receipt_path: normalize_path_for_receipt(closure_receipt_path),
        publication_gate: "Bifrost".to_string(),
        durable_state_admitted: true,
        private_state_exposed: false,
        modeling_route_id: modeling_route_id.to_string(),
        modeling_generation,
    };
    commit_repo_work_map_admission(runtime_store, &entry, mind_review, mind_commit)?;
    runtime_repo_work_map_entry(runtime_store, &entry.map_entry_id)?
        .ok_or_else(|| anyhow!("committed repo-work map entry could not be reread"))
}

fn resolve_local_verse_store_from_execute_receipt(
    workspace: &Path,
    execute_receipt: &Value,
) -> Result<Option<PathBuf>> {
    if let Some(store) = path_from_json(execute_receipt, &["localVerseStore"])
        .or_else(|| path_from_json(execute_receipt, &["localVerseStorePath"]))
    {
        return Ok(Some(store));
    }
    let Some(adopt_receipt_path) = path_from_json(execute_receipt, &["adoptReceiptPath"]) else {
        return Ok(None);
    };
    let adopt_receipt = read_json(&adopt_receipt_path)?;
    if let Some(store) = path_from_json(&adopt_receipt, &["localVerseStore"])
        .or_else(|| path_from_json(&adopt_receipt, &["localVerseStorePath"]))
    {
        return Ok(Some(store));
    }
    let Some(run_receipt_path) = path_from_json(&adopt_receipt, &["runReceiptPath"]) else {
        return Ok(None);
    };
    let run_receipt = read_json(&run_receipt_path)?;
    if let Some(store) = path_from_json(&run_receipt, &["localVerseStore"])
        .or_else(|| path_from_json(&run_receipt, &["localVerseStorePath"]))
    {
        return Ok(Some(store));
    }
    let Some(online_receipt_path) = path_from_json(&run_receipt, &["onlineReceiptPath"]) else {
        return Ok(None);
    };
    let online_receipt = read_json(&online_receipt_path)?;
    Ok(path_from_json(&online_receipt, &["localVerseStore"])
        .or_else(|| path_from_json(&online_receipt, &["localVerseStorePath"]))
        .or_else(|| Some(workspace.join(".epiphany").join("local-verse.ccmp"))))
}

fn project_repo_work_map_entry_to_local_verse(
    local_verse_store: &Path,
    runtime_id: &str,
    workspace: &Path,
    source_store_path: &Path,
    entry: &RepoWorkMapEntry,
) -> Result<EpiphanyCultMeshRepoWorkMapEntry> {
    let item_slug = sanitize(&entry.item);
    let commit_short = short_commit(&entry.commit_sha);
    let tui_rows = vec![
        format!("item {}", entry.item),
        format!("branch {}", entry.branch),
        format!("commit {}", entry.commit_sha),
        format!("family {}", entry.safe_action_family),
        format!("modeling {}", entry.modeling_finding_receipt_id),
        format!("modelingRoute {}", entry.modeling_route_id),
        format!("modelingGeneration {}", entry.modeling_generation),
        format!("mind {}", entry.mind_state_commit_receipt_id),
        format!("publicationGate {}", entry.publication_gate),
        "private false".to_string(),
    ];
    let map_entry = EpiphanyCultMeshRepoWorkMapEntry {
        schema_version: EPIPHANY_CULTMESH_REPO_WORK_MAP_ENTRY_SCHEMA_VERSION.to_string(),
        runtime_id: runtime_id.to_string(),
        verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
        map_entry_id: format!("repo-work-map-{item_slug}"),
        admitted_at: entry.admitted_at.clone(),
        mirrored_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        workspace: workspace.display().to_string(),
        item: entry.item.clone(),
        branch: entry.branch.clone(),
        changed_paths: entry.changed_paths.clone(),
        commit_sha: entry.commit_sha.clone(),
        safe_action_family: entry.safe_action_family.clone(),
        modeling_summary: entry.modeling_summary.clone(),
        modeling_finding_receipt_id: entry.modeling_finding_receipt_id.clone(),
        modeling_route_id: entry.modeling_route_id.clone(),
        modeling_generation: entry.modeling_generation,
        soul_verdict_receipt_id: entry.soul_verdict_receipt_id.clone(),
        mind_gateway_review_id: entry.mind_gateway_review_id.clone(),
        mind_state_commit_receipt_id: entry.mind_state_commit_receipt_id.clone(),
        publication_gate: entry.publication_gate.clone(),
        durable_state_admitted: entry.durable_state_admitted,
        source_store_path: normalize_path_for_receipt(source_store_path),
        tui_rows: vec![format!(
            "REPO-WORK-MAP | item={} | branch={} | commit={} | family={} | mind={} | gate={} | private=false",
            entry.item,
            entry.branch,
            commit_short,
            entry.safe_action_family,
            entry.mind_state_commit_receipt_id,
            entry.publication_gate
        )]
        .into_iter()
        .chain(tui_rows)
        .collect(),
        private_state_exposed: false,
        notes: vec![
            "Repo work map entry is compact local Verse sight over Mind-admitted durable state; raw worker thoughts and receipt payload bodies remain sealed.".to_string(),
            "Downstream consumers may project this row; the projection grants no scheduling, publication, merge, service lifecycle, deployment, or cross-repo mutation authority.".to_string(),
        ],
    };
    write_epiphany_cultmesh_repo_work_map_entry(local_verse_store, map_entry)
}

fn repo_work_map_entry_json(entry: &RepoWorkMapEntry) -> Value {
    json!({
        "schemaVersion": entry.schema_version,
        "mapEntryId": entry.map_entry_id,
        "admittedAt": entry.admitted_at,
        "item": entry.item,
        "branch": entry.branch,
        "changedPaths": entry.changed_paths,
        "commitSha": entry.commit_sha,
        "safeActionFamily": entry.safe_action_family,
        "modelingSummary": entry.modeling_summary,
        "modelingFindingReceiptId": entry.modeling_finding_receipt_id,
        "modelingRouteId": entry.modeling_route_id,
        "modelingGeneration": entry.modeling_generation,
        "soulVerdictReceiptId": entry.soul_verdict_receipt_id,
        "mindGatewayReviewId": entry.mind_gateway_review_id,
        "mindStateCommitReceiptId": entry.mind_state_commit_receipt_id,
        "executeReceiptPath": entry.execute_receipt_path,
        "closureReviewPath": entry.closure_review_path,
        "closureReceiptPath": entry.closure_receipt_path,
        "publicationGate": entry.publication_gate,
        "durableStateAdmitted": entry.durable_state_admitted,
        "privateStateExposed": entry.private_state_exposed,
    })
}

fn short_commit(commit_sha: &str) -> String {
    commit_sha.chars().take(12).collect::<String>()
}

fn closure_verification_source_grounding_review(
    declared_changed_paths: &[String],
    verification_stdout: &[u8],
    verification_stderr: &[u8],
    commit_stat: &str,
    required: bool,
) -> (Value, bool) {
    let stdout_text = String::from_utf8_lossy(verification_stdout);
    let stderr_text = String::from_utf8_lossy(verification_stderr);
    let verification_output = format!("{stdout_text}\n{stderr_text}");
    let output_lower = verification_output.to_ascii_lowercase();
    let commit_stat_lower = commit_stat.to_ascii_lowercase();
    let mut path_rows = Vec::new();
    for path in declared_changed_paths {
        let path_lower = path.to_ascii_lowercase();
        let slash_variant = path_lower.replace('\\', "/");
        let backslash_variant = path_lower.replace('/', "\\");
        let mentioned_in_verification_output = output_lower.contains(&slash_variant)
            || output_lower.contains(&backslash_variant)
            || output_lower.contains(&path_lower);
        let present_in_commit_stat = commit_stat_lower.contains(&slash_variant)
            || commit_stat_lower.contains(&backslash_variant)
            || commit_stat_lower.contains(&path_lower);
        path_rows.push(json!({
            "path": path,
            "mentionedInVerificationOutput": mentioned_in_verification_output,
            "presentInCommitStat": present_in_commit_stat,
            "passed": mentioned_in_verification_output && present_in_commit_stat
        }));
    }
    let all_paths_mentioned = !declared_changed_paths.is_empty()
        && path_rows
            .iter()
            .all(|row| row.get("passed").and_then(Value::as_bool) == Some(true));
    let passed = !required || all_paths_mentioned;
    let status = if all_paths_mentioned {
        "passed"
    } else if required {
        "failed"
    } else {
        "informational"
    };
    (
        json!({
            "schemaVersion": "epiphany.repo_work_verification_source_grounding_review.v0",
            "status": status,
            "passed": passed,
            "required": required,
            "declaredPathCount": declared_changed_paths.len(),
            "allDeclaredPathsMentionedByVerificationOutput": all_paths_mentioned,
            "pathRows": path_rows,
            "reason": if all_paths_mentioned {
                "verification output and commit stat cite every declared changed path"
            } else if required {
                "source-grounded closure required verification output to cite every declared changed path"
            } else {
                "verification output did not cite every declared changed path; recorded as advisory source-grounding signal"
            }
        }),
        passed,
    )
}

fn read_committed_file(
    workspace: &Path,
    commit_sha: &str,
    repo_path: &str,
) -> Result<Option<String>> {
    let spec = format!("{commit_sha}:{repo_path}");
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["show", &spec])
        .output()
        .with_context(|| format!("failed to run git show {spec}"))?;
    if output.status.success() {
        Ok(Some(String::from_utf8_lossy(&output.stdout).to_string()))
    } else {
        Ok(None)
    }
}

fn plan_derivation_receipt(input: DeriveSafePlanInput<'_>, mode: &str, safe_family: &str) -> Value {
    json!({
        "schemaVersion": "epiphany.repo_work_plan_derivation.v0",
        "mode": mode,
        "safeActionFamily": safe_family,
        "owner": "Imagination",
        "router": "Self",
        "inputPressure": {
            "source": input.source,
            "summary": input.summary,
            "candidateActionRefs": input.accept_receipt["feedback"]["candidateActionRefs"],
            "publicDiscussionRefs": input.accept_receipt["feedback"]["publicDiscussionRefs"],
        },
        "operatorAuthoredShellDetails": false,
        "modelAuthored": input.model_authored,
        "modelRef": input.model_ref,
        "deterministicQuarantine": true,
        "authoritySeal": {
            "branchLocalOnly": true,
            "publicationAuthorized": false,
            "mergeAuthorized": false,
            "serviceLifecycleAuthority": false,
            "crossRepoMutation": false,
            "privateStateExposed": false
        },
        "nextUpgrade": "deepen model-authored Imagination action items while keeping Hands command lowering inside allowlisted safe families"
    })
}

struct ImaginationActionItemReceiptInputs<'a> {
    item: &'a str,
    source: &'a str,
    summary: &'a str,
    derive_plan_mode: &'a str,
    safe_action_family: &'a str,
    requested_paths: Vec<String>,
    action_summary: &'a str,
    verification_asks: Vec<String>,
    stop_conditions: Vec<String>,
    escalation_reasons: Vec<String>,
    rollback_hints: Vec<String>,
    planning_facets: PlanningFacets,
    model_ref: Option<&'a str>,
    model_authored: bool,
}

#[derive(Clone, Debug)]
struct PlanningFacets {
    assumptions: Vec<String>,
    constraints: Vec<String>,
    non_goals: Vec<String>,
    open_questions: Vec<String>,
    decision_points: Vec<String>,
    evidence_needs: Vec<String>,
}

impl PlanningFacets {
    fn for_derive_plan(
        source: &str,
        safe_action_family: &str,
        target_path: &str,
        assumptions: Vec<String>,
        constraints: Vec<String>,
        non_goals: Vec<String>,
        open_questions: Vec<String>,
        decision_points: Vec<String>,
        evidence_needs: Vec<String>,
    ) -> Self {
        Self {
            assumptions: default_if_empty(
                assumptions,
                vec![format!(
                    "Accepted {source} pressure can be represented by safe family {safe_action_family}."
                )],
            ),
            constraints: default_if_empty(
                constraints,
                vec![
                    format!("Hands may only change the requested path {target_path}."),
                    "Publication, merge, service lifecycle, elevation, cross-repo mutation, and private-state exposure remain outside this plan.".to_string(),
                ],
            ),
            non_goals: default_if_empty(
                non_goals,
                vec![
                    "Do not convert model-authored planning text into arbitrary shell authority.".to_string(),
                    "Do not admit durable Mind state without the later Mind gate.".to_string(),
                ],
            ),
            open_questions: default_if_empty(
                open_questions,
                vec!["Does Self/Mind accept this candidate as the next branch-local move?".to_string()],
            ),
            decision_points: default_if_empty(
                decision_points,
                vec![
                    "Self/Mind may adopt this action item, ask Imagination for a narrower plan, or route it to Bifrost/Maintainer review.".to_string(),
                ],
            ),
            evidence_needs: default_if_empty(
                evidence_needs,
                vec![
                    "Soul needs the derived plan receipt, changed-path proof, safe-family closure assertions, and private-state seal before closure.".to_string(),
                ],
            ),
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "assumptions": self.assumptions,
            "constraints": self.constraints,
            "nonGoals": self.non_goals,
            "openQuestions": self.open_questions,
            "decisionPoints": self.decision_points,
            "evidenceNeeds": self.evidence_needs,
            "handsCommandAuthority": false,
            "durableStateAuthority": false,
            "privateStateExposed": false
        })
    }
}

fn write_imagination_action_items_receipt(
    artifact_dir: &Path,
    workspace: &Path,
    accept_receipt_path: &Path,
    accept_receipt: &Value,
    inputs: ImaginationActionItemReceiptInputs<'_>,
) -> Result<Value> {
    if inputs.requested_paths.is_empty() {
        return Err(anyhow!(
            "Imagination action-item receipts require at least one requested path"
        ));
    }
    let normalized_paths = normalize_paths(inputs.requested_paths.clone());
    let item_slug = sanitize(inputs.item);
    let receipt_id = format!("repo-work-action-items-{item_slug}");
    let action_item_id = format!("{receipt_id}-action-1");
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let planning_facets = inputs.planning_facets.to_json();
    let receipt = json!({
        "schemaVersion": "epiphany.repo_work_imagination_action_items_receipt.v0",
        "createdAt": now,
        "workspace": workspace,
        "acceptReceiptPath": accept_receipt_path,
        "receiptId": receipt_id,
        "item": inputs.item,
        "status": "proposed-for-self-mind-review",
        "owner": "Imagination",
        "router": "Self",
        "stateGate": "Mind",
        "inputPressure": {
            "source": inputs.source,
            "summary": inputs.summary,
            "feedbackId": accept_receipt["feedback"]["feedbackId"],
            "consensusReceiptId": accept_receipt["feedback"]["consensusReceiptId"],
            "candidateActionRefs": accept_receipt["feedback"]["candidateActionRefs"],
            "publicDiscussionRefs": accept_receipt["feedback"]["publicDiscussionRefs"]
        },
        "model": {
            "modelAuthored": inputs.model_authored,
            "modelRef": inputs.model_ref,
            "deterministicFallback": !inputs.model_authored,
            "operatorAuthoredShellDetails": false
        },
        "actionItems": [{
            "actionItemId": action_item_id,
            "status": "candidate",
            "derivePlanMode": inputs.derive_plan_mode,
            "safeActionFamily": inputs.safe_action_family,
            "requestedPaths": normalized_paths,
            "summary": inputs.action_summary,
            "intendedConsequence": inputs.action_summary,
            "verificationAsks": inputs.verification_asks,
            "stopConditions": inputs.stop_conditions,
            "escalationReasons": inputs.escalation_reasons,
            "rollbackHints": inputs.rollback_hints,
            "planningFacets": planning_facets,
            "handsCommandDerived": false
        }],
        "authority": {
            "handsAuthorityGranted": false,
            "durableStateAdmitted": false,
            "publicationAuthorized": false,
            "mergeAuthorized": false,
            "serviceLifecycleAuthority": false,
            "crossRepoMutation": false,
            "privateStateExposed": false,
            "nextGate": "self.mind.adoption_then_plan_derivation"
        },
        "privateStateExposed": false,
        "nextSafeMove": "Self/Mind may adopt one Imagination action item, then derive a Hands plan through an allowlisted safe family."
    });
    let receipt_path = artifact_dir.join(format!("work-action-items-{item_slug}.json"));
    write_json(&receipt_path, &receipt)?;
    Ok(json!({
        "schemaVersion": receipt["schemaVersion"],
        "status": receipt["status"],
        "workspace": receipt["workspace"],
        "receiptPath": receipt_path,
        "receiptId": receipt["receiptId"],
        "item": receipt["item"],
        "actionItems": receipt["actionItems"],
        "model": receipt["model"],
        "authority": receipt["authority"],
        "privateStateExposed": false,
        "nextSafeMove": receipt["nextSafeMove"],
    }))
}

fn write_plan_receipt(
    workspace: PathBuf,
    accept_receipt_path: PathBuf,
    accept_receipt: &Value,
    artifact_dir: PathBuf,
    inputs: PlanReceiptInputs,
) -> Result<Value> {
    let item = accept_receipt
        .get("item")
        .and_then(Value::as_str)
        .unwrap_or("work-item")
        .to_string();
    let item_slug = sanitize(&item);
    let normalized_paths = normalize_paths(inputs.changed_paths.clone());
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let plan_id = format!("repo-work-plan-{item_slug}");
    let action_id = format!("{plan_id}-action-1");
    let mut plan_receipt = json!({
        "schemaVersion": "epiphany.repo_work_action_plan_receipt.v0",
        "createdAt": now,
        "workspace": workspace,
        "acceptReceiptPath": accept_receipt_path,
        "item": item,
        "planId": plan_id,
        "status": "planned-for-self-adoption",
        "planner": {
            "owner": "Imagination",
            "router": "Self",
            "stateGate": "Mind"
        },
        "objective": inputs.objective,
        "planSummary": inputs.plan_summary,
        "adoptionEvidenceRefs": inputs.adoption_evidence_refs,
        "actions": [{
            "actionId": action_id,
            "kind": "repo.branch_local_command",
            "command": inputs.command,
            "changedPaths": normalized_paths,
            "commitMessage": inputs.commit_message,
            "verificationAsks": inputs.verification_asks,
            "stopConditions": inputs.stop_conditions,
            "rollbackHints": inputs.rollback_hints
        }],
        "authority": {
            "handsAuthorityGranted": false,
            "durableStateAdmitted": false,
            "publicationAuthorized": false,
            "nextGate": "self.mind.adoption_then_hands"
        },
        "privateStateExposed": false,
        "nextSafeMove": "Run epiphany-work adopt --from-plan <receipt> after Self/Mind accept this Imagination action plan."
    });
    if let Some(derivation) = inputs.derivation {
        plan_receipt["derivation"] = derivation;
    }
    let receipt_path = artifact_dir.join(format!("work-plan-{item_slug}.json"));
    write_json(&receipt_path, &plan_receipt)?;
    Ok(json!({
        "schemaVersion": "epiphany.repo_work_plan.v0",
        "status": plan_receipt["status"],
        "workspace": plan_receipt["workspace"],
        "receiptPath": receipt_path,
        "item": plan_receipt["item"],
        "planId": plan_receipt["planId"],
        "objective": plan_receipt["objective"],
        "action": plan_receipt["actions"][0],
        "derivation": plan_receipt.get("derivation").cloned().unwrap_or(Value::Null),
        "authority": plan_receipt["authority"],
        "privateStateExposed": false,
        "nextSafeMove": plan_receipt["nextSafeMove"],
    }))
}

fn run_adopt(args: AdoptArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let _epiphany_root = args
        .epiphany_root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.epiphany_root.display()))?;
    let run_receipt_path = resolve_run_receipt(&workspace, args.item.as_deref(), args.run_receipt)?;
    let run_receipt = read_json(&run_receipt_path)?;
    let plan_receipt_path = if let Some(path) = args.plan_receipt {
        Some(path)
    } else {
        None
    };
    let plan_receipt = if let Some(path) = plan_receipt_path.as_ref() {
        Some(read_json(path)?)
    } else {
        None
    };
    let runtime_store = args.runtime_store.unwrap_or_else(|| {
        path_from_json(&run_receipt, &["runtimeStore"]).unwrap_or_else(|| {
            workspace
                .join(".epiphany")
                .join("state")
                .join("runtime-spine.msgpack")
        })
    });
    let artifact_dir = args
        .artifact_dir
        .unwrap_or_else(|| workspace.join(".epiphany").join("work"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    let gate = run_receipt
        .get("handsActionGate")
        .ok_or_else(|| anyhow!("run receipt has no handsActionGate"))?;
    let intent_id = gate
        .get("intentId")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("run receipt handsActionGate has no intentId"))?;
    let review_id = gate
        .get("reviewId")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("run receipt handsActionGate has no reviewId"))?;
    let intent = runtime_hands_action_intent(&runtime_store, intent_id)?
        .ok_or_else(|| anyhow!("runtime-spine has no Hands intent {intent_id}"))?;
    let queued_review = runtime_hands_action_review(&runtime_store, review_id)?
        .ok_or_else(|| anyhow!("runtime-spine has no Hands review {review_id}"))?;
    if queued_review.intent_id != intent.intent_id {
        return Err(anyhow!(
            "Hands review {} belongs to {}, not {}",
            queued_review.review_id,
            queued_review.intent_id,
            intent.intent_id
        ));
    }
    if queued_review.decision != "queued-for-adoption" {
        return Err(anyhow!(
            "Hands review {} decision is {}, not queued-for-adoption",
            queued_review.review_id,
            queued_review.decision
        ));
    }
    let item = run_receipt
        .get("item")
        .and_then(Value::as_str)
        .unwrap_or("work-item")
        .to_string();
    let item_slug = sanitize(&item);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let plan_summary = if let Some(summary) = args.plan_summary {
        summary
    } else if let Some(plan) = plan_receipt.as_ref() {
        string_from_json(plan, &["planSummary"])
            .ok_or_else(|| anyhow!("plan receipt has no planSummary"))?
    } else {
        return Err(anyhow!("missing --plan-summary or --from-plan"));
    };
    let mut adoption_evidence_refs = args.adoption_evidence_refs;
    if let Some(plan) = plan_receipt.as_ref() {
        adoption_evidence_refs.extend(string_array_from_json(plan, &["adoptionEvidenceRefs"]));
        if adoption_evidence_refs.is_empty() {
            adoption_evidence_refs.push(format!(
                "repo-work-plan:{}",
                plan.get("planId")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
            ));
        }
    }
    let adopted_action_item = plan_receipt
        .as_ref()
        .map(adopted_action_item_from_plan)
        .unwrap_or(Value::Null);
    let action_item_receipt_id = adopted_action_item
        .get("receiptId")
        .and_then(Value::as_str)
        .unwrap_or("manual-plan");
    let action_item_summary = adopted_action_item
        .get("summary")
        .and_then(Value::as_str)
        .unwrap_or(&plan_summary);
    let action_item_safe_family = adopted_action_item
        .get("safeActionFamily")
        .and_then(Value::as_str)
        .unwrap_or("manual-plan");
    let requested_path_count = adopted_action_item
        .get("requestedPaths")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let requested_paths = sorted_normalized_paths(string_array_from_json(
        &adopted_action_item,
        &["requestedPaths"],
    ));
    let plan_changed_paths = plan_receipt
        .as_ref()
        .and_then(|plan| first_plan_action(plan))
        .map(|action| sorted_normalized_paths(string_array_field(action, "changedPaths")))
        .unwrap_or_default();
    let requested_paths_match_plan = plan_receipt.is_none()
        || (!requested_paths.is_empty() && requested_paths == plan_changed_paths);
    let planning_facets_present = adopted_action_item
        .get("planningFacets")
        .is_some_and(|value| !value.is_null());
    let verification_asks = string_array_from_json(&adopted_action_item, &["verificationAsks"]);
    let verification_asks_present = !verification_asks.is_empty();
    let verification_ask_count = verification_asks.len();
    let evidence_needs =
        string_array_from_json(&adopted_action_item, &["planningFacets", "evidenceNeeds"]);
    let evidence_needs_present = !evidence_needs.is_empty();
    let evidence_need_count = evidence_needs.len();
    let action_item_model_authored = adopted_action_item
        .get("modelAuthored")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let evidence_ref_count = adoption_evidence_refs.len();
    let safe_family_recognized = repo_work_safe_family_is_recognized(action_item_safe_family);
    let unsupported_plan_family = plan_receipt.is_some() && !safe_family_recognized;
    let path_scope_mismatch = plan_receipt.is_some() && !requested_paths_match_plan;
    let evidence_readiness_missing =
        plan_receipt.is_some() && (!verification_asks_present || !evidence_needs_present);
    let mut refusal_reasons = Vec::new();
    if unsupported_plan_family {
        refusal_reasons.push(format!(
            "Unsupported repo-work safe action family {action_item_safe_family}; Mind refused to convert this plan into branch-local Hands authority."
        ));
    }
    if path_scope_mismatch {
        refusal_reasons.push(format!(
            "Requested paths {:?} do not match plan changed paths {:?}; Mind refused to convert this plan into branch-local Hands authority.",
            requested_paths, plan_changed_paths
        ));
    }
    if evidence_readiness_missing {
        refusal_reasons.push(format!(
            "Action item lacks verification or evidence needs; Mind refused to convert this plan into branch-local Hands authority until Soul proof targets are explicit. verificationAsksPresent={verification_asks_present}, evidenceNeedsPresent={evidence_needs_present}."
        ));
    }
    let mind_refused_adoption =
        unsupported_plan_family || path_scope_mismatch || evidence_readiness_missing;
    let mind_interpretation = json!({
        "schemaVersion": "epiphany.repo_work_mind_interpretation.v0",
        "owner": "Mind",
        "interpreter": "Mind",
        "router": "Self",
        "source": if plan_receipt.is_some() { "plan-receipt" } else { "manual-adoption-evidence" },
        "inputSummary": {
            "planReceiptPresent": plan_receipt.is_some(),
            "runReceiptPresent": true,
            "actionItemReceiptId": action_item_receipt_id,
            "safeActionFamily": action_item_safe_family,
            "requestedPathCount": requested_path_count,
            "requestedPaths": requested_paths,
            "planChangedPaths": plan_changed_paths,
            "modelAuthored": action_item_model_authored,
            "planningFacetsPresent": planning_facets_present,
            "verificationAsks": verification_asks,
            "verificationAskCount": verification_ask_count,
            "evidenceNeeds": evidence_needs,
            "evidenceNeedCount": evidence_need_count,
            "adoptionEvidenceRefCount": evidence_ref_count
        },
        "classification": {
            "decisionKind": "branch-local-hands-adoption",
            "actionItemAccepted": !mind_refused_adoption,
            "safeFamilyRecognized": safe_family_recognized,
            "requestedPathsDeclared": requested_path_count > 0,
            "requestedPathsMatchPlan": requested_paths_match_plan,
            "verificationAsksPresent": verification_asks_present,
            "evidenceNeedsPresent": evidence_needs_present,
            "evidenceRefsPresent": evidence_ref_count > 0,
            "durableStateAdmission": "not-admitted",
            "publicationGate": "Bifrost",
            "closureGate": "Soul",
            "privateStateSeal": true
        },
        "allowedTransitions": [
            "hands.branch_local_action"
        ],
        "forbiddenTransitions": [
            "mind.durable_state_commit",
            "bifrost.publication",
            "git.merge",
            "idunn.service_lifecycle",
            "cross_body_mutation",
            "private_verse_export"
        ],
        "refusalReasons": refusal_reasons,
        "privateStateExposed": false
    });
    let mind_adoption_id = format!("repo-work-mind-adoption-{item_slug}");
    if mind_refused_adoption {
        let status = if unsupported_plan_family {
            "refused-unsupported-safe-family"
        } else if evidence_readiness_missing {
            "refused-missing-evidence-needs"
        } else {
            "refused-requested-path-mismatch"
        };
        let next_gate = if unsupported_plan_family {
            "imagination.replan_with_allowed_safe_family"
        } else if evidence_readiness_missing {
            "imagination.replan_with_explicit_soul_evidence_needs"
        } else {
            "imagination.replan_with_matching_requested_paths"
        };
        let next_safe_move = if unsupported_plan_family {
            "Ask Imagination for an allowed repo-work safe family before Hands authority can be granted."
        } else if evidence_readiness_missing {
            "Ask Imagination to add explicit verification asks and evidence needs before Hands authority can be granted."
        } else {
            "Ask Imagination for an action item whose requested paths exactly match the plan changed paths before Hands authority can be granted."
        };
        let refusal_decision = json!({
            "schemaVersion": "epiphany.repo_work_mind_adoption_decision.v0",
            "createdAt": now,
            "workspace": workspace,
            "runtimeId": run_receipt["runtimeId"],
            "runtimeStore": runtime_store,
            "decisionId": mind_adoption_id,
            "item": item,
            "status": status,
            "owner": "Mind",
            "interpreter": "Mind",
            "router": "Self",
            "sourcePlanReceiptPath": plan_receipt_path,
            "sourceRunReceiptPath": run_receipt_path,
            "adoptedActionItemReceiptId": action_item_receipt_id,
            "adoptedActionItem": adopted_action_item,
            "adoptionEvidenceRefs": adoption_evidence_refs,
            "interpretation": mind_interpretation,
            "rationale": refusal_reasons.join(" "),
            "gates": {
                "selfPresentedActionItem": true,
                "mindReviewedEvidence": true,
                "mindInterpretedActionItem": true,
                "safeFamilyRequired": true,
                "safeFamilyRecognized": safe_family_recognized,
                "requestedPathsMatchPlan": requested_paths_match_plan,
                "verificationAsksPresent": verification_asks_present,
                "evidenceNeedsPresent": evidence_needs_present,
                "branchLocalOnly": false,
                "bifrostPublicationRequired": true,
                "soulClosureRequired": true
            },
            "authority": {
                "durableStateAdmitted": false,
                "handsAuthorityGranted": false,
                "publicationAuthorized": false,
                "mergeAuthorized": false,
                "serviceLifecycleAuthority": false,
                "crossRepoMutation": false,
                "privateStateExposed": false,
                "nextGate": next_gate
            },
            "privateStateExposed": false,
            "nextSafeMove": next_safe_move
        });
        let mind_adoption_path = artifact_dir.join(format!("work-mind-adopt-{item_slug}.json"));
        write_json(&mind_adoption_path, &refusal_decision)?;
        if unsupported_plan_family {
            return Err(anyhow!(
                "Mind refused adoption for unsupported repo-work safe family {action_item_safe_family}: {}; refusal decision written to {}",
                refusal_reasons.join(" "),
                mind_adoption_path.display()
            ));
        }
        return Err(anyhow!(
            "Mind refused adoption: {}; refusal decision written to {}",
            refusal_reasons.join(" "),
            mind_adoption_path.display()
        ));
    }
    let mind_adoption_rationale = args.mind_adoption_rationale.unwrap_or_else(|| {
        format!(
            "Mind adopted the selected Imagination action item for branch-local Hands work because Self presented explicit adoption evidence and the safe family remains bounded: {}",
            compact_line(action_item_summary)
        )
    });
    let mind_adoption_decision = json!({
        "schemaVersion": "epiphany.repo_work_mind_adoption_decision.v0",
        "createdAt": now,
        "workspace": workspace,
        "runtimeId": run_receipt["runtimeId"],
        "runtimeStore": runtime_store,
        "decisionId": mind_adoption_id,
        "item": item,
        "status": "adopted-for-branch-local-hands",
        "owner": "Mind",
        "interpreter": "Mind",
        "router": "Self",
        "sourcePlanReceiptPath": plan_receipt_path,
        "sourceRunReceiptPath": run_receipt_path,
        "adoptedActionItemReceiptId": action_item_receipt_id,
        "adoptedActionItem": adopted_action_item,
        "adoptionEvidenceRefs": adoption_evidence_refs,
        "interpretation": mind_interpretation,
        "rationale": mind_adoption_rationale,
        "gates": {
            "selfPresentedActionItem": true,
            "mindReviewedEvidence": true,
            "mindInterpretedActionItem": true,
            "safeFamilyRequired": true,
            "branchLocalOnly": true,
            "bifrostPublicationRequired": true,
            "soulClosureRequired": true
        },
        "authority": {
            "durableStateAdmitted": false,
            "handsAuthorityGranted": false,
            "publicationAuthorized": false,
            "mergeAuthorized": false,
            "serviceLifecycleAuthority": false,
            "crossRepoMutation": false,
            "privateStateExposed": false,
            "nextGate": "hands.branch_local_action"
        },
        "privateStateExposed": false,
        "nextSafeMove": "The adoption decision may be cited by the Hands review; it does not itself execute, publish, merge, or mutate durable project state."
    });
    let mind_adoption_path = artifact_dir.join(format!("work-mind-adopt-{item_slug}.json"));
    write_json(&mind_adoption_path, &mind_adoption_decision)?;
    let reread_mind_adoption = read_json(&mind_adoption_path)?;
    if reread_mind_adoption["schemaVersion"] != "epiphany.repo_work_mind_adoption_decision.v0" {
        return Err(anyhow!(
            "Mind adoption decision {} failed reread schema verification",
            mind_adoption_path.display()
        ));
    }
    if reread_mind_adoption["privateStateExposed"] != json!(false) {
        return Err(anyhow!(
            "Mind adoption decision {} exposed private state",
            mind_adoption_path.display()
        ));
    }
    let mut approved_review = hands_action_review_for_intent(
        review_id.to_string(),
        &intent,
        "approved".to_string(),
        vec![
            "patch".to_string(),
            "command".to_string(),
            "commit".to_string(),
        ],
        vec![
            format!("Adopted plan: {plan_summary}"),
            format!(
                "Adoption evidence refs: {}",
                adoption_evidence_refs.join(", ")
            ),
            "Bifrost still gates publication and merge.".to_string(),
        ],
        now.clone(),
    );
    approved_review.required_receipts = vec![
        HANDS_PATCH_RECEIPT_TYPE.to_string(),
        HANDS_COMMAND_RECEIPT_TYPE.to_string(),
        HANDS_COMMIT_RECEIPT_TYPE.to_string(),
    ];
    put_hands_action_review(&runtime_store, &approved_review)?;

    let adoption_receipt = json!({
        "schemaVersion": "epiphany.repo_work_adoption_receipt.v0",
        "createdAt": now,
        "workspace": workspace,
        "runtimeId": run_receipt["runtimeId"],
        "runtimeStore": runtime_store,
        "runReceiptPath": run_receipt_path,
        "planReceiptPath": plan_receipt_path,
        "item": item,
        "status": "approved-for-branch-local-hands-action",
        "planSummary": plan_summary,
        "adoptionEvidenceRefs": adoption_evidence_refs,
        "mindAdoptionDecision": {
            "decisionId": mind_adoption_decision["decisionId"],
            "receiptPath": mind_adoption_path,
            "schemaVersion": mind_adoption_decision["schemaVersion"],
            "status": mind_adoption_decision["status"],
            "owner": mind_adoption_decision["owner"],
            "interpreter": mind_adoption_decision["interpreter"],
            "interpretation": mind_adoption_decision["interpretation"],
            "rationale": mind_adoption_decision["rationale"],
            "gates": mind_adoption_decision["gates"],
            "authority": mind_adoption_decision["authority"],
            "privateStateExposed": false
        },
        "adoptedActionItem": adopted_action_item,
        "handsActionGate": {
            "intentId": intent.intent_id,
            "reviewId": approved_review.review_id,
            "runtimeJobId": intent.runtime_job_id,
            "substrateGateGrantReceiptId": intent.substrate_gate_grant_receipt_id,
            "decision": approved_review.decision,
            "allowedOperations": approved_review.allowed_operations,
            "requiredReceipts": approved_review.required_receipts,
            "recordPassCommand": format!(
                "epiphany-hands-action --store {} record-pass --intent-id {} --review-id {} --summary <summary> --changed-path <path> --command <command> --exit-code <code> --stdout-artifact <stdout> --stderr-artifact <stderr> --commit-sha <sha> --branch <branch>",
                runtime_store.display(),
                intent.intent_id,
                approved_review.review_id
            )
        },
        "authority": {
            "handsAuthorityGranted": true,
            "durableStateAdmitted": false,
            "publicationAuthorized": false,
            "publicationGate": "Bifrost",
            "requiredReceiptsBeforeCompletion": approved_review.required_receipts
        },
        "privateStateExposed": false,
        "nextSafeMove": "Execute branch-local work through Hands and record patch/command/commit receipts; Bifrost still gates publish/merge."
    });
    let receipt_path = artifact_dir.join(format!("work-adopt-{item_slug}.json"));
    write_json(&receipt_path, &adoption_receipt)?;
    Ok(json!({
        "schemaVersion": "epiphany.repo_work_adoption.v0",
        "status": "approved-for-branch-local-hands-action",
        "workspace": adoption_receipt["workspace"],
        "runtimeId": adoption_receipt["runtimeId"],
        "runtimeStore": adoption_receipt["runtimeStore"],
        "receiptPath": receipt_path,
        "item": adoption_receipt["item"],
        "mindAdoptionDecision": adoption_receipt["mindAdoptionDecision"],
        "adoptedActionItem": adoption_receipt["adoptedActionItem"],
        "handsActionGate": adoption_receipt["handsActionGate"],
        "authority": adoption_receipt["authority"],
        "privateStateExposed": adoption_receipt["privateStateExposed"],
        "nextSafeMove": adoption_receipt["nextSafeMove"],
    }))
}

fn run_execute(args: ExecuteArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let _epiphany_root = args
        .epiphany_root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.epiphany_root.display()))?;
    let adopt_receipt_path =
        resolve_adopt_receipt(&workspace, args.item.as_deref(), args.adopt_receipt)?;
    let adopt_receipt = read_json(&adopt_receipt_path)?;
    let plan_receipt_path = if let Some(path) = args.plan_receipt {
        Some(path)
    } else {
        path_from_json(&adopt_receipt, &["planReceiptPath"])
    };
    let plan_receipt = if let Some(path) = plan_receipt_path.as_ref() {
        Some(read_json(path)?)
    } else {
        None
    };
    let runtime_store = args.runtime_store.unwrap_or_else(|| {
        path_from_json(&adopt_receipt, &["runtimeStore"]).unwrap_or_else(|| {
            workspace
                .join(".epiphany")
                .join("state")
                .join("runtime-spine.msgpack")
        })
    });
    let artifact_dir = args
        .artifact_dir
        .unwrap_or_else(|| workspace.join(".epiphany").join("work"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    let gate = adopt_receipt
        .get("handsActionGate")
        .ok_or_else(|| anyhow!("adopt receipt has no handsActionGate"))?;
    let intent_id = gate
        .get("intentId")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("adopt receipt handsActionGate has no intentId"))?;
    let review_id = gate
        .get("reviewId")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("adopt receipt handsActionGate has no reviewId"))?;
    let intent = runtime_hands_action_intent(&runtime_store, intent_id)?
        .ok_or_else(|| anyhow!("runtime-spine has no Hands intent {intent_id}"))?;
    let review = runtime_hands_action_review(&runtime_store, review_id)?
        .ok_or_else(|| anyhow!("runtime-spine has no Hands review {review_id}"))?;
    ensure_hands_review_allows(&intent, &review, "patch")?;
    ensure_hands_review_allows(&intent, &review, "command")?;
    ensure_hands_review_allows(&intent, &review, "commit")?;
    let planned_action = plan_receipt.as_ref().and_then(first_plan_action).cloned();
    let command = if let Some(command) = args.command {
        command
    } else if let Some(action) = planned_action.as_ref() {
        string_from_value(action, "command").ok_or_else(|| anyhow!("plan action has no command"))?
    } else {
        return Err(anyhow!("missing --command or --from-plan"));
    };
    let changed_paths = if args.changed_paths.is_empty() {
        if let Some(action) = planned_action.as_ref() {
            string_array_field(action, "changedPaths")
        } else {
            Vec::new()
        }
    } else {
        args.changed_paths
    };
    if changed_paths.is_empty() {
        return Err(anyhow!(
            "execute requires at least one changed path from --changed-path or --from-plan"
        ));
    }
    let commit_message = if let Some(message) = args.commit_message {
        message
    } else if let Some(action) = planned_action.as_ref() {
        string_from_value(action, "commitMessage")
            .ok_or_else(|| anyhow!("plan action has no commitMessage"))?
    } else {
        return Err(anyhow!("missing --commit-message or --from-plan"));
    };
    validate_paths_within_gate(&intent, &changed_paths)?;
    let branch = git_output(&workspace, &["branch", "--show-current"])?;
    if !branch.starts_with("epiphany/") {
        return Err(anyhow!(
            "execute requires an epiphany/* work branch; current branch is {branch:?}"
        ));
    }

    let item = adopt_receipt
        .get("item")
        .and_then(Value::as_str)
        .unwrap_or("work-item")
        .to_string();
    let item_slug = sanitize(&item);
    let stdout_artifact = artifact_dir.join(format!("work-execute-{item_slug}.stdout.log"));
    let stderr_artifact = artifact_dir.join(format!("work-execute-{item_slug}.stderr.log"));
    let execution = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(&command)
        .current_dir(&workspace)
        .output()
        .with_context(|| format!("failed to execute command in {}", workspace.display()))?;
    fs::write(&stdout_artifact, &execution.stdout)
        .with_context(|| format!("failed to write {}", stdout_artifact.display()))?;
    fs::write(&stderr_artifact, &execution.stderr)
        .with_context(|| format!("failed to write {}", stderr_artifact.display()))?;
    let exit_code = execution
        .status
        .code()
        .map(|code| code.to_string())
        .unwrap_or_else(|| "terminated-by-signal".to_string());
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let summary = args
        .summary
        .unwrap_or_else(|| format!("Executed approved repo work item {item} on branch {branch}."));
    let command_receipt_id = format!("repo-work-execute-{item_slug}-hands-command");
    let command_receipt = hands_command_receipt_for_review(
        command_receipt_id,
        &intent,
        &review,
        command.clone(),
        exit_code.clone(),
        normalize_path_for_receipt(&stdout_artifact),
        normalize_path_for_receipt(&stderr_artifact),
        summary.clone(),
        now.clone(),
    );
    put_hands_command_receipt(&runtime_store, &command_receipt)?;
    if !execution.status.success() {
        let failed_receipt = json!({
            "schemaVersion": "epiphany.repo_work_execute_receipt.v0",
            "createdAt": now,
            "workspace": workspace,
            "runtimeStore": runtime_store,
            "adoptReceiptPath": adopt_receipt_path,
            "item": item,
            "status": "command-failed",
            "branch": branch,
            "changedPaths": changed_paths,
            "planReceiptPath": plan_receipt_path,
            "commandReceiptId": command_receipt.receipt_id,
            "exitCode": command_receipt.exit_code,
            "stdoutArtifact": command_receipt.stdout_artifact,
            "stderrArtifact": command_receipt.stderr_artifact,
            "privateStateExposed": false,
        });
        let receipt_path = artifact_dir.join(format!("work-execute-{item_slug}.json"));
        write_json(&receipt_path, &failed_receipt)?;
        return Ok(json!({
            "schemaVersion": "epiphany.repo_work_execute.v0",
            "status": failed_receipt["status"],
            "workspace": failed_receipt["workspace"],
            "receiptPath": receipt_path,
            "item": failed_receipt["item"],
            "handsReceipts": {
                "commandReceiptId": failed_receipt["commandReceiptId"]
            },
            "privateStateExposed": false,
            "nextSafeMove": "Inspect command artifacts and either rerun execution after a new adoption review or record a failure review."
        }));
    }

    let normalized_paths = normalize_paths(changed_paths.clone());
    let patch_receipt_id = format!("repo-work-execute-{item_slug}-hands-patch");
    let patch_receipt = hands_patch_receipt_for_review(
        patch_receipt_id,
        &intent,
        &review,
        normalized_paths.clone(),
        summary.clone(),
        now.clone(),
    );
    put_hands_patch_receipt(&runtime_store, &patch_receipt)?;
    git_add_paths(&workspace, &normalized_paths)?;
    ensure_staged_changes(&workspace)?;
    git_commit(&workspace, &commit_message)?;
    let commit_sha = git_output(&workspace, &["rev-parse", "HEAD"])?;
    let commit_receipt_id = format!("repo-work-execute-{item_slug}-hands-commit");
    let commit_receipt = hands_commit_receipt_for_review(
        commit_receipt_id,
        &intent,
        &review,
        commit_sha,
        branch.clone(),
        normalized_paths.clone(),
        summary.clone(),
        Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    );
    put_hands_commit_receipt(&runtime_store, &commit_receipt)?;

    let execute_receipt = json!({
        "schemaVersion": "epiphany.repo_work_execute_receipt.v0",
        "createdAt": now,
        "workspace": workspace,
        "runtimeStore": runtime_store,
        "adoptReceiptPath": adopt_receipt_path,
        "planReceiptPath": plan_receipt_path,
        "item": item,
        "status": "branch-local-commit-recorded",
        "branch": branch,
        "changedPaths": normalized_paths,
        "command": command_receipt.command,
        "exitCode": command_receipt.exit_code,
        "stdoutArtifact": command_receipt.stdout_artifact,
        "stderrArtifact": command_receipt.stderr_artifact,
        "handsReceipts": {
            "patchReceiptId": patch_receipt.receipt_id,
            "commandReceiptId": command_receipt.receipt_id,
            "commitReceiptId": commit_receipt.receipt_id,
            "commitSha": commit_receipt.commit_sha
        },
        "authority": {
            "branchLocalCommitCreated": true,
            "publicationAuthorized": false,
            "durableStateAdmitted": false,
            "privateStateExposed": false
        },
        "privateStateExposed": false,
        "nextSafeMove": "Route Soul verification and Mind review, then submit a Bifrost publication intent through the owning integration."
    });
    let receipt_path = artifact_dir.join(format!("work-execute-{item_slug}.json"));
    write_json(&receipt_path, &execute_receipt)?;
    Ok(json!({
        "schemaVersion": "epiphany.repo_work_execute.v0",
        "status": execute_receipt["status"],
        "workspace": execute_receipt["workspace"],
        "runtimeStore": execute_receipt["runtimeStore"],
        "receiptPath": receipt_path,
        "item": execute_receipt["item"],
        "branch": execute_receipt["branch"],
        "changedPaths": execute_receipt["changedPaths"],
        "handsReceipts": execute_receipt["handsReceipts"],
        "authority": execute_receipt["authority"],
        "privateStateExposed": false,
        "nextSafeMove": execute_receipt["nextSafeMove"],
    }))
}

fn run_verify(args: CloseArgs) -> Result<Value> {
    run_closure_pipeline(args, ClosurePhase::SoulOnly)
}

fn run_revise_modeling(args: ReviseModelingArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let item_slug = sanitize(&args.item);
    let close_path = work_receipt_path(&workspace, "close", &args.item);
    let close = read_json(&close_path)?;
    if close.get("status").and_then(Value::as_str) != Some("awaiting-modeling") {
        return Err(anyhow!(
            "revise-modeling requires an awaiting-modeling closure projection"
        ));
    }
    let runtime_store = args
        .runtime_store
        .or_else(|| path_from_json(&close, &["runtimeStore"]))
        .ok_or_else(|| anyhow!("awaiting-modeling closure has no runtime store"))?;
    let route_id = format!("repo-work-modeling-route-{item_slug}");
    let current = runtime_repo_work_modeling_route(&runtime_store, &route_id)?
        .ok_or_else(|| anyhow!("typed Modeling route {route_id:?} is missing"))?;
    let current_request = runtime_repo_work_modeling_request(&runtime_store, &current.request_id)?
        .ok_or_else(|| anyhow!("current Modeling request is missing"))?;
    let previous_finding_id = format!("{}-finding", current.request_id);
    let previous_finding =
        runtime_repo_work_modeling_finding(&runtime_store, &previous_finding_id)?
            .ok_or_else(|| anyhow!("revise-modeling requires the current typed finding"))?;
    if previous_finding
        .verdict
        .trim()
        .eq_ignore_ascii_case("passed")
    {
        return Err(anyhow!(
            "passing Modeling truth cannot be revised into a retry"
        ));
    }
    let generation = current.generation.saturating_add(1);
    let request_id = format!("repo-work-close-{item_slug}-modeling-request-g{generation}");
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let request = RepoWorkModelingRequest {
        schema_version: REPO_WORK_MODELING_REQUEST_SCHEMA_VERSION.to_string(),
        request_id: request_id.clone(),
        item: current.item.clone(),
        requester: "mind".to_string(),
        soul_verdict_receipt_id: current_request.soul_verdict_receipt_id.clone(),
        commit_sha: current_request.commit_sha.clone(),
        changed_paths: current_request.changed_paths.clone(),
        instruction: format!(
            "Re-model the same Soul-verified consequence after reviewed correction {}. Previous finding {} said: {}. Review rationale: {}",
            args.review_ref,
            previous_finding.receipt_id,
            compact_line(&previous_finding.finding),
            compact_line(&args.rationale)
        ),
        requested_at: now.clone(),
        private_state_exposed: false,
        contract: "Mind-reviewed next generation of one immutable repo-work Modeling route; preserves the Soul-verified consequence and previous finding."
            .to_string(),
    };
    let gateway_id = format!("repo-work-modeling-route-{item_slug}-g{generation}-mind-review");
    let review = MindGatewayReview {
        schema_version: MIND_GATEWAY_REVIEW_SCHEMA_VERSION.to_string(),
        gateway_id: gateway_id.clone(),
        source_kind: "repo_work_modeling_revision".to_string(),
        source_role_id: "mind".to_string(),
        decision: MindGatewayDecision::Accept,
        allowed_effects: vec!["repoWork.modelingRoute".to_string()],
        refused_effects: vec![
            "repoWork.map".to_string(),
            "publication".to_string(),
            "merge".to_string(),
            "handsAction".to_string(),
        ],
        reasons: vec![
            format!("Reviewed correction reference: {}.", args.review_ref),
            format!("Previous immutable verdict: {}.", previous_finding.verdict),
            args.rationale.clone(),
        ],
        contract: "Mind review advances only the current Modeling request generation; it admits no map, publication, merge, or Hands effect."
            .to_string(),
    };
    let route = RepoWorkModelingRoute {
        schema_version: REPO_WORK_MODELING_ROUTE_SCHEMA_VERSION.to_string(),
        route_id: current.route_id.clone(),
        item: current.item.clone(),
        generation,
        request_id: request.request_id.clone(),
        previous_finding_receipt_id: previous_finding.receipt_id.clone(),
        authority_owner: "mind".to_string(),
        authority_witness_id: review.gateway_id.clone(),
        updated_at: now,
        private_state_exposed: false,
        contract: "Sole current-generation pointer for repo-work Modeling; older requests and findings remain immutable evidence."
            .to_string(),
    };
    let route = advance_repo_work_modeling_route(&runtime_store, &request, &route, &review)?;
    Ok(json!({
        "schemaVersion": REPO_WORK_MODELING_ROUTE_SCHEMA_VERSION,
        "status": "modeling-route-advanced",
        "workspace": workspace,
        "item": route.item,
        "routeId": route.route_id,
        "generation": route.generation,
        "requestId": route.request_id,
        "previousFindingReceiptId": route.previous_finding_receipt_id,
        "mindReviewId": route.authority_witness_id,
        "reviewRef": args.review_ref,
        "privateStateExposed": false,
        "nextSafeMove": "Pulse the scheduler to launch the reviewed Modeling generation through Idunn."
    }))
}

fn run_close(args: CloseArgs) -> Result<Value> {
    run_closure_pipeline(args, ClosurePhase::Full)
}

fn run_closure_pipeline(args: CloseArgs, phase: ClosurePhase) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let execute_receipt_path =
        resolve_execute_receipt(&workspace, args.item.as_deref(), args.execute_receipt)?;
    let execute_receipt = read_json(&execute_receipt_path)?;
    if execute_receipt.get("status").and_then(Value::as_str) != Some("branch-local-commit-recorded")
    {
        return Err(anyhow!(
            "close requires a branch-local-commit-recorded execute receipt"
        ));
    }
    let runtime_store = args.runtime_store.unwrap_or_else(|| {
        path_from_json(&execute_receipt, &["runtimeStore"]).unwrap_or_else(|| {
            workspace
                .join(".epiphany")
                .join("state")
                .join("runtime-spine.msgpack")
        })
    });
    let artifact_dir = args
        .artifact_dir
        .unwrap_or_else(|| workspace.join(".epiphany").join("work"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    let item = execute_receipt
        .get("item")
        .and_then(Value::as_str)
        .unwrap_or("work-item")
        .to_string();
    let item_slug = sanitize(&item);
    let commit_sha = string_from_json(&execute_receipt, &["handsReceipts", "commitSha"])
        .ok_or_else(|| anyhow!("execute receipt has no handsReceipts.commitSha"))?;
    let patch_receipt_id = string_from_json(&execute_receipt, &["handsReceipts", "patchReceiptId"])
        .ok_or_else(|| anyhow!("execute receipt has no handsReceipts.patchReceiptId"))?;
    let command_receipt_id =
        string_from_json(&execute_receipt, &["handsReceipts", "commandReceiptId"])
            .ok_or_else(|| anyhow!("execute receipt has no handsReceipts.commandReceiptId"))?;
    let commit_receipt_id =
        string_from_json(&execute_receipt, &["handsReceipts", "commitReceiptId"])
            .ok_or_else(|| anyhow!("execute receipt has no handsReceipts.commitReceiptId"))?;
    let commit_receipt = runtime_hands_commit_receipt(&runtime_store, &commit_receipt_id)?
        .ok_or_else(|| anyhow!("runtime-spine has no Hands commit receipt {commit_receipt_id}"))?;
    if commit_receipt.commit_sha != commit_sha {
        return Err(anyhow!(
            "execute receipt commit sha does not match runtime Hands commit receipt"
        ));
    }
    let verification_command = format!("git show --stat --oneline {commit_sha}");
    let stdout_artifact = artifact_dir.join(format!("work-close-{item_slug}.stdout.log"));
    let stderr_artifact = artifact_dir.join(format!("work-close-{item_slug}.stderr.log"));
    let verification = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(&verification_command)
        .current_dir(&workspace)
        .output()
        .with_context(|| {
            format!(
                "failed to run closure verification in {}",
                workspace.display()
            )
        })?;
    fs::write(&stdout_artifact, &verification.stdout)
        .with_context(|| format!("failed to write {}", stdout_artifact.display()))?;
    fs::write(&stderr_artifact, &verification.stderr)
        .with_context(|| format!("failed to write {}", stderr_artifact.display()))?;
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let declared_changed_paths =
        sorted_normalized_paths(string_array_from_json(&execute_receipt, &["changedPaths"]));
    let actual_changed_paths = sorted_normalized_paths(
        git_output(
            &workspace,
            &[
                "diff-tree",
                "--no-commit-id",
                "--name-only",
                "-r",
                &commit_sha,
            ],
        )?
        .lines()
        .map(ToString::to_string)
        .collect(),
    );
    let path_scope_matched = declared_changed_paths == actual_changed_paths;
    let commit_stat = git_output(&workspace, &["show", "--stat", "--oneline", &commit_sha])?;
    let (verification_source_grounding, verification_source_grounded) =
        closure_verification_source_grounding_review(
            &declared_changed_paths,
            &verification.stdout,
            &verification.stderr,
            &commit_stat,
            true,
        );
    let verification_output_mentions_changed_paths = bool_from_json(
        &verification_source_grounding,
        &["allDeclaredPathsMentionedByVerificationOutput"],
    )
    .unwrap_or(false);
    let (family_assertions, family_assertions_passed) =
        closure_family_assertions(&workspace, &commit_sha, &execute_receipt, &item)?;
    let (mind_adoption_review, mind_adoption_passed) =
        closure_mind_adoption_review(&execute_receipt)?;
    let model_closure_review = json!({
        "schemaVersion": "epiphany.repo_work_model_closure_review.v0",
        "status": "deferred-to-typed-modeling",
        "passed": false,
        "required": true,
        "source": "runtime-spine Modeling finding",
        "callerEchoAccepted": false,
        "privateStateExposed": false
    });
    let closure_review_id = format!("repo-work-close-{item_slug}-closure-review");
    let closure_review_path = artifact_dir.join(format!("work-close-{item_slug}-review.json"));
    let closure_review = json!({
        "schemaVersion": "epiphany.repo_work_closure_review.v0",
        "createdAt": now,
        "workspace": workspace,
        "receiptId": closure_review_id,
        "item": item,
        "owner": "Soul",
        "stateGate": "Mind",
        "executeReceiptPath": execute_receipt_path,
        "runtimeStore": runtime_store,
        "handsReceipts": {
            "patchReceiptId": patch_receipt_id,
            "commandReceiptId": command_receipt_id,
            "commitReceiptId": commit_receipt_id,
            "commitSha": commit_sha
        },
        "verification": {
            "command": verification_command,
            "exitCode": verification.status.code(),
            "stdoutArtifact": normalize_path_for_receipt(&stdout_artifact),
            "stderrArtifact": normalize_path_for_receipt(&stderr_artifact),
            "passed": verification.status.success()
        },
        "sourceGrounding": {
            "declaredChangedPaths": declared_changed_paths,
            "actualChangedPaths": actual_changed_paths,
            "pathScopeMatched": path_scope_matched,
            "commitStat": compact_multiline(&commit_stat),
            "commitReceiptMatchedExecuteReceipt": true,
            "verificationOutputMentionsChangedPaths": verification_output_mentions_changed_paths,
            "sourceGroundingRequired": true,
            "mindAdoptionPassed": mind_adoption_passed,
            "familyAssertionsPassed": family_assertions_passed,
            "modelClosurePassed": false
        },
        "verificationSourceGrounding": verification_source_grounding,
        "mindAdoptionReview": mind_adoption_review,
        "familyAssertions": family_assertions,
        "modelingReview": {
            "modelAuthored": false,
            "modelRef": Value::Null,
            "deterministicFallback": false,
            "source": "runtime-spine Modeling finding",
            "callerEchoAccepted": false,
            "operatorAuthoredShellDetails": false,
            "summary": "Modeling truth is resolved from the runtime-spine finding, not caller text.",
            "closureReview": model_closure_review
        },
        "authoritySeal": {
            "branchLocalOnly": true,
            "publicationAuthorized": false,
            "mergeAuthorized": false,
            "serviceLifecycleAuthority": false,
            "crossRepoMutation": false,
            "privateStateExposed": false
        },
        "privateStateExposed": false,
        "nextSafeMove": "Soul may pass closure only when verification succeeds, actual git changed paths match the Hands-declared path scope, the accepted Mind adoption proof is present, safe-family assertions pass for known Imagination families, and any supplied or required model-authored closure verdict passes."
    });
    write_json(&closure_review_path, &closure_review)?;
    let soul_verification_passed = verification.status.success()
        && path_scope_matched
        && verification_source_grounded
        && mind_adoption_passed
        && family_assertions_passed;
    let soul_verdict_id = format!("repo-work-close-{item_slug}-soul-verdict");
    let soul_summary = if soul_verification_passed {
        format!("Soul verified branch-local commit {commit_sha} for repo work item {item}.")
    } else if !path_scope_matched {
        format!(
            "Soul verification failed for branch-local commit {commit_sha}: actual changed paths did not match declared Hands path scope."
        )
    } else if !verification_source_grounded {
        format!(
            "Soul verification failed for branch-local commit {commit_sha}: verification output did not cite every declared changed path."
        )
    } else if !mind_adoption_passed {
        format!(
            "Soul verification failed for branch-local commit {commit_sha}: Mind adoption proof did not pass."
        )
    } else if !family_assertions_passed {
        format!(
            "Soul verification failed for branch-local commit {commit_sha}: safe-family assertions did not pass."
        )
    } else {
        format!("Soul verification failed for branch-local commit {commit_sha}.")
    };
    let mut evidence_ids = vec![
        patch_receipt_id.clone(),
        command_receipt_id.clone(),
        commit_receipt_id.clone(),
        normalize_path_for_receipt(&stdout_artifact),
        normalize_path_for_receipt(&stderr_artifact),
        normalize_path_for_receipt(&closure_review_path),
    ];
    evidence_ids.extend(declared_changed_paths.clone());
    let mut soul_verdict = SoulVerdictReceipt {
        schema_version: SOUL_VERDICT_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: soul_verdict_id.clone(),
        source_result_id: format!("repo-work-execute-{item_slug}"),
        source_job_id: format!("repo-work-close-{item_slug}"),
        verdict: if soul_verification_passed {
            "passed".to_string()
        } else {
            "failed".to_string()
        },
        summary: soul_summary.clone(),
        evidence_ids: evidence_ids.clone(),
        risks: if soul_verification_passed {
            Vec::new()
        } else if !path_scope_matched {
            vec![
                "Closure refused because actual git changed paths differ from Hands-declared scope."
                    .to_string(),
            ]
        } else if !verification_source_grounded {
            vec![
                "Closure refused because the verification output did not cite every declared changed path while source grounding was required."
                    .to_string(),
            ]
        } else if !mind_adoption_passed {
            vec![
                "Closure refused because the Mind adoption decision was missing, tampered, or non-affirmative."
                    .to_string(),
            ]
        } else if !family_assertions_passed {
            vec![
                "Closure refused because the committed content did not satisfy known safe-family assertions."
                    .to_string(),
            ]
        } else {
            vec!["Closure verification command failed; publication remains blocked.".to_string()]
        },
        emitted_at: now.clone(),
        contract: "Soul verdict for repo-work closure; verifies the Hands patch/command/commit consequence before Modeling/Mind closure and Bifrost publication.".to_string(),
    };
    if let Some(existing) =
        epiphany_core::runtime_soul_verdict_receipt(&runtime_store, &soul_verdict.receipt_id)?
    {
        soul_verdict.emitted_at = existing.emitted_at.clone();
        if soul_verdict != existing {
            return Err(anyhow!(
                "closure retry conflicts with immutable Soul verdict receipt"
            ));
        }
        soul_verdict = existing;
    } else {
        put_soul_verdict_receipt(&runtime_store, &soul_verdict)?;
    }

    let closure_receipt_path = artifact_dir.join(format!("work-close-{item_slug}.json"));
    if !soul_verification_passed {
        let failed_receipt = json!({
            "schemaVersion": "epiphany.repo_work_closure_receipt.v0",
            "createdAt": now,
            "workspace": workspace,
            "runtimeStore": runtime_store,
            "executeReceiptPath": execute_receipt_path,
            "item": item,
            "status": "verification-failed",
            "soul": {
                "verdictReceiptId": soul_verdict.receipt_id,
                "verdict": soul_verdict.verdict,
                "summary": soul_verdict.summary,
                "stdoutArtifact": normalize_path_for_receipt(&stdout_artifact),
                "stderrArtifact": normalize_path_for_receipt(&stderr_artifact),
                "closureReviewReceiptId": closure_review_id,
                "closureReviewPath": normalize_path_for_receipt(&closure_review_path)
            },
            "closureReview": closure_review,
            "authority": {
                "branchLocalOnly": true,
                "publicationGateSatisfied": false,
                "durableStateAdmitted": false,
                "privateStateExposed": false
            },
            "privateStateExposed": false,
            "nextSafeMove": "Repair or re-run branch-local work, then close again before publication."
        });
        write_json(&closure_receipt_path, &failed_receipt)?;
        return Ok(json!({
            "schemaVersion": "epiphany.repo_work_closure.v0",
            "status": failed_receipt["status"],
            "workspace": failed_receipt["workspace"],
            "receiptPath": closure_receipt_path,
            "item": failed_receipt["item"],
            "soul": failed_receipt["soul"],
            "closureReview": failed_receipt["closureReview"],
            "authority": failed_receipt["authority"],
            "privateStateExposed": false,
            "nextSafeMove": failed_receipt["nextSafeMove"],
        }));
    }

    let modeling_route_id = format!("repo-work-modeling-route-{item_slug}");
    let (modeling_request, modeling_route) = if phase == ClosurePhase::Full {
        let route = runtime_repo_work_modeling_route(&runtime_store, &modeling_route_id)?
            .ok_or_else(|| anyhow!("closure requires the typed Modeling route"))?;
        let request = runtime_repo_work_modeling_request(&runtime_store, &route.request_id)?
            .ok_or_else(|| anyhow!("closure requires the current typed Modeling request"))?;
        (request, route)
    } else {
        let modeling_request_id = format!("repo-work-close-{item_slug}-modeling-request");
        let mut request = RepoWorkModelingRequest {
            schema_version: REPO_WORK_MODELING_REQUEST_SCHEMA_VERSION.to_string(),
            request_id: modeling_request_id.clone(),
            item: item.clone(),
            requester: "self".to_string(),
            soul_verdict_receipt_id: soul_verdict.receipt_id.clone(),
            commit_sha: commit_sha.clone(),
            changed_paths: declared_changed_paths.clone(),
            instruction: "Model the Soul-verified consequence and return a bounded repo-map finding; do not admit state or route the next action.".to_string(),
            requested_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            private_state_exposed: false,
            contract: "Self-to-Modeling request over a passing Soul verdict; Self may route but cannot author the result.".to_string(),
        };
        if let Some(existing) =
            runtime_repo_work_modeling_request(&runtime_store, &modeling_request_id)?
        {
            request.requested_at = existing.requested_at.clone();
            if request != existing {
                return Err(anyhow!(
                    "closure retry conflicts with immutable Modeling request"
                ));
            }
            request = existing;
        }
        let mut route = RepoWorkModelingRoute {
            schema_version: REPO_WORK_MODELING_ROUTE_SCHEMA_VERSION.to_string(),
            route_id: modeling_route_id.clone(),
            item: item.clone(),
            generation: 0,
            request_id: request.request_id.clone(),
            previous_finding_receipt_id: String::new(),
            authority_owner: "soul".to_string(),
            authority_witness_id: soul_verdict.receipt_id.clone(),
            updated_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            private_state_exposed: false,
            contract: "Current repo-work Modeling generation; generation zero is established atomically with its Soul-backed request, and later movement requires Mind review."
                .to_string(),
        };
        if let Some(existing) =
            runtime_repo_work_modeling_route(&runtime_store, &modeling_route_id)?
        {
            route.updated_at = existing.updated_at.clone();
        }
        let route = commit_initial_repo_work_modeling_route(&runtime_store, &request, &route)?;
        (request, route)
    };
    if phase == ClosurePhase::SoulOnly {
        let awaiting_receipt = json!({
            "schemaVersion": "epiphany.repo_work_closure_receipt.v0",
            "createdAt": now,
            "workspace": workspace,
            "runtimeStore": runtime_store,
            "executeReceiptPath": execute_receipt_path,
            "item": item,
            "status": "awaiting-modeling",
            "soul": {
                "verdictReceiptId": soul_verdict.receipt_id,
                "verdict": soul_verdict.verdict,
                "summary": soul_verdict.summary,
                "closureReviewReceiptId": closure_review_id,
                "closureReviewPath": normalize_path_for_receipt(&closure_review_path)
            },
            "closureReview": closure_review,
            "modeling": {
                "requestId": modeling_request.request_id,
                "routeId": modeling_route.route_id,
                "generation": modeling_route.generation,
                "requester": modeling_request.requester,
                "findingReceiptId": Value::Null
            },
            "authority": {
                "owner": "Self",
                "routedSoulResult": true,
                "authoredModelingFinding": false,
                "publicationGateSatisfied": false,
                "durableStateAdmitted": false,
                "privateStateExposed": false
            },
            "privateStateExposed": false,
            "nextSafeMove": "Route the typed request to Modeling and resume closure only with its matching finding."
        });
        write_json(&closure_receipt_path, &awaiting_receipt)?;
        return Ok(awaiting_receipt);
    }
    let modeling_finding_receipt_id = format!("{}-finding", modeling_request.request_id);
    let modeling_finding =
        runtime_repo_work_modeling_finding(&runtime_store, &modeling_finding_receipt_id)?
            .ok_or_else(|| {
                anyhow!(
                    "closure requires the persisted Modeling finding {}; caller-authored echoes are not accepted",
                    modeling_finding_receipt_id
                )
            })?;
    if modeling_finding.soul_verdict_receipt_id != soul_verdict.receipt_id
        || modeling_finding.commit_sha != commit_sha
        || modeling_finding.changed_paths != declared_changed_paths
        || modeling_finding.verdict.trim().to_ascii_lowercase() != "passed"
    {
        return Err(anyhow!(
            "persisted Modeling finding does not match Soul-verified consequence"
        ));
    }
    let gateway_id = format!("repo-work-close-{item_slug}-mind-review");
    let mind_review = MindGatewayReview {
        schema_version: MIND_GATEWAY_REVIEW_SCHEMA_VERSION.to_string(),
        gateway_id: gateway_id.clone(),
        source_kind: "repo_work_closure".to_string(),
        source_role_id: "modeling".to_string(),
        decision: MindGatewayDecision::Accept,
        allowed_effects: vec![
            "repoWorkClosure".to_string(),
            "modelingFinding".to_string(),
            "publicationGate".to_string(),
        ],
        refused_effects: Vec::new(),
        reasons: vec![
            "Soul passed the branch-local Hands consequence.".to_string(),
            format!("Mind reread typed Modeling finding {}.", modeling_finding.receipt_id),
            "Mind admits closure metadata only; Bifrost still gates publication and merge.".to_string(),
        ],
        contract: "Mind review for repo-work closure; admits the verified Modeling summary and publication gate without granting merge or service authority.".to_string(),
    };
    let mind_commit_id = format!("repo-work-close-{item_slug}-mind-commit");
    let mind_commit = mind_state_commit_receipt(
        mind_commit_id.clone(),
        &mind_review,
        0,
        vec![
            "repoWork.closure".to_string(),
            "repoWork.modelingFinding".to_string(),
            "repoWork.map".to_string(),
        ],
        Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    );
    let branch = git_output(&workspace, &["branch", "--show-current"])?;
    let repo_map_entry = record_repo_work_map_admission(
        &runtime_store,
        &item,
        &branch,
        declared_changed_paths.clone(),
        &commit_sha,
        &closure_review,
        &modeling_finding.summary,
        &modeling_finding.receipt_id,
        &modeling_route.route_id,
        modeling_route.generation,
        &soul_verdict.receipt_id,
        &mind_review.gateway_id,
        &mind_commit.receipt_id,
        &execute_receipt_path,
        &closure_review_path,
        &closure_receipt_path,
        &mind_review,
        &mind_commit,
    )?;
    let runtime_id = string_from_json(&execute_receipt, &["runtimeId"])
        .unwrap_or_else(|| "repo-swarm-local".to_string());
    let local_verse_store =
        resolve_local_verse_store_from_execute_receipt(&workspace, &execute_receipt)?;
    let repo_map_local_verse_projection = if let Some(local_verse_store) =
        local_verse_store.as_ref()
    {
        let written = project_repo_work_map_entry_to_local_verse(
            local_verse_store,
            &runtime_id,
            &workspace,
            &runtime_store,
            &repo_map_entry,
        )?;
        json!({
            "projected": true,
            "localVerseStore": normalize_path_for_receipt(local_verse_store),
            "documentType": "epiphany.cultmesh.repo_work_map_entry",
            "schemaVersion": written.schema_version,
            "mapEntryId": written.map_entry_id,
            "latestKey": EPIPHANY_CULTMESH_REPO_WORK_MAP_ENTRY_LATEST_KEY,
            "tuiRows": written.tui_rows,
            "privateStateExposed": written.private_state_exposed
        })
    } else {
        json!({
            "projected": false,
            "reason": "local Verse store could not be resolved from execute/adopt/run/online receipts",
            "privateStateExposed": false
        })
    };

    let closure_receipt = json!({
        "schemaVersion": "epiphany.repo_work_closure_receipt.v0",
        "createdAt": now,
        "workspace": workspace,
        "runtimeStore": runtime_store,
        "executeReceiptPath": execute_receipt_path,
        "item": item,
        "status": "closed",
        "handsReceipts": {
            "patchReceiptId": patch_receipt_id,
            "commandReceiptId": command_receipt_id,
            "commitReceiptId": commit_receipt_id,
            "commitSha": commit_sha
        },
        "soul": {
            "verdictReceiptId": soul_verdict.receipt_id,
            "verdict": soul_verdict.verdict,
            "summary": soul_verdict.summary,
            "stdoutArtifact": normalize_path_for_receipt(&stdout_artifact),
            "stderrArtifact": normalize_path_for_receipt(&stderr_artifact),
            "closureReviewReceiptId": closure_review_id,
            "closureReviewPath": normalize_path_for_receipt(&closure_review_path)
        },
        "closureReview": closure_review,
        "modeling": {
            "requestId": modeling_request.request_id,
            "findingReceiptId": modeling_finding.receipt_id,
            "summary": modeling_finding.summary,
            "changedPaths": execute_receipt["changedPaths"],
            "commitSha": execute_receipt["handsReceipts"]["commitSha"],
            "source": "runtime-spine Modeling finding",
            "modelAuthored": true
        },
        "mind": {
            "gatewayReviewId": mind_review.gateway_id,
            "stateCommitReceiptId": mind_commit.receipt_id,
            "stateRevision": mind_commit.state_revision,
            "changedFields": mind_commit.changed_fields,
            "repoMapStorePath": normalize_path_for_receipt(&runtime_store),
            "repoMapEntry": repo_work_map_entry_json(&repo_map_entry),
            "repoMapLocalVerseProjection": repo_map_local_verse_projection
        },
        "authority": {
            "branchLocalOnly": true,
            "publicationGateSatisfied": true,
            "publicationAuthorized": false,
            "mergeAuthorized": false,
            "serviceLifecycleAuthorized": false,
            "crossRepoMutationAuthorized": false,
            "durableStateAdmitted": true,
            "privateStateExposed": false
        },
        "privateStateExposed": false,
        "nextSafeMove": "Submit a Bifrost publication intent through the owning integration; merge remains gated by provider-authored maintainer evidence."
    });
    write_json(&closure_receipt_path, &closure_receipt)?;
    Ok(json!({
        "schemaVersion": "epiphany.repo_work_closure.v0",
        "status": closure_receipt["status"],
        "workspace": closure_receipt["workspace"],
        "runtimeStore": closure_receipt["runtimeStore"],
        "receiptPath": closure_receipt_path,
        "item": closure_receipt["item"],
        "handsReceipts": closure_receipt["handsReceipts"],
        "soul": closure_receipt["soul"],
        "closureReview": closure_receipt["closureReview"],
        "modeling": closure_receipt["modeling"],
        "mind": closure_receipt["mind"],
        "authority": closure_receipt["authority"],
        "privateStateExposed": false,
        "nextSafeMove": closure_receipt["nextSafeMove"],
    }))
}

fn run_overview(args: OverviewArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let artifact_dir = args
        .artifact_dir
        .unwrap_or_else(|| workspace.join(".epiphany").join("work"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;

    let accept_receipt_path =
        resolve_accept_receipt(&workspace, args.item.as_deref(), args.accept_receipt)?;
    let accept_receipt = read_json(&accept_receipt_path)?;
    let runtime_id = string_from_json(&accept_receipt, &["runtimeId"])
        .unwrap_or_else(|| "repo-swarm-local".to_string());
    let local_verse_store = path_from_json(&accept_receipt, &["localVerseStore"])
        .or_else(|| path_from_json(&accept_receipt, &["localVerseStorePath"]));
    let item = accept_receipt
        .get("item")
        .and_then(Value::as_str)
        .unwrap_or("work-item")
        .to_string();
    let item_slug = sanitize(&item);

    let plan_receipt_path = work_receipt_path(&workspace, "plan", &item);
    let run_receipt_path = work_receipt_path(&workspace, "run", &item);
    let adopt_receipt_path = work_receipt_path(&workspace, "adopt", &item);
    let execute_receipt_path = work_receipt_path(&workspace, "execute", &item);
    let close_receipt_path = work_receipt_path(&workspace, "close", &item);
    let close_review_receipt_path =
        artifact_dir.join(format!("work-close-{item_slug}-review.json"));
    let overview_receipt_path = artifact_dir.join(format!("work-overview-{item_slug}.json"));

    let plan_receipt = read_json_if_exists(&plan_receipt_path)?;
    let run_receipt = read_json_if_exists(&run_receipt_path)?;
    let adopt_receipt = read_json_if_exists(&adopt_receipt_path)?;
    let execute_receipt = read_json_if_exists(&execute_receipt_path)?;
    let close_receipt = read_json_if_exists(&close_receipt_path)?;

    let branch = git_output(&workspace, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let commit_sha = execute_receipt
        .as_ref()
        .and_then(|receipt| string_from_json(receipt, &["handsReceipts", "commitSha"]));
    let changed_paths = execute_receipt
        .as_ref()
        .map(|receipt| string_array_from_json(receipt, &["changedPaths"]))
        .or_else(|| {
            plan_receipt.as_ref().and_then(|receipt| {
                first_plan_action(receipt).map(|action| string_array_field(action, "changedPaths"))
            })
        })
        .unwrap_or_default();
    let closure_status = close_receipt
        .as_ref()
        .and_then(|receipt| receipt.get("status").and_then(Value::as_str))
        .unwrap_or("missing");
    let soul_verdict = close_receipt
        .as_ref()
        .and_then(|receipt| string_from_json(receipt, &["soul", "verdict"]))
        .unwrap_or_else(|| "missing".to_string());
    let publication_status = "provider-evidence-not-resolved";
    let sync_status = "provider-evidence-not-resolved";
    let (gate, blocker, next_safe_move) = repo_work_overview_gate(
        plan_receipt.as_ref(),
        run_receipt.as_ref(),
        adopt_receipt.as_ref(),
        execute_receipt.as_ref(),
        close_receipt.as_ref(),
    );
    let public_discussion_refs =
        string_array_from_json(&accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let candidate_action_refs =
        string_array_from_json(&accept_receipt, &["feedback", "candidateActionRefs"]);
    let feedback_id = string_from_json(&accept_receipt, &["feedback", "feedbackId"])
        .unwrap_or_else(|| "missing".to_string());
    let consensus_receipt_id =
        string_from_json(&accept_receipt, &["feedback", "consensusReceiptId"])
            .unwrap_or_else(|| "missing".to_string());
    let consensus_route =
        string_from_json(&accept_receipt, &["feedback", "requestedConsensusRoute"])
            .unwrap_or_else(|| "missing".to_string());
    let consensus_packet_ref =
        string_from_json(&accept_receipt, &["feedback", "consensusPacketRef"])
            .unwrap_or_else(|| "missing".to_string());
    let adoption_gate = string_from_json(&accept_receipt, &["feedback", "adoptionGate"])
        .unwrap_or_else(|| "missing".to_string());
    let plan_action_item = plan_receipt
        .as_ref()
        .and_then(|receipt| receipt.get("derivation"))
        .and_then(|value| value.get("actionItemReceipt"));
    let plan_action_item_receipt_id = plan_action_item
        .and_then(|value| value.get("receiptId"))
        .and_then(Value::as_str)
        .unwrap_or("missing")
        .to_string();
    let plan_safe_action_family = plan_action_item
        .and_then(|value| value.get("safeActionFamily"))
        .and_then(Value::as_str)
        .unwrap_or("missing")
        .to_string();
    let plan_model_authored = plan_action_item
        .and_then(|value| value.get("modelAuthored"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let intake_consensus_readback = json!({
        "schemaVersion": "epiphany.repo_work_intake_consensus_readback.v0",
        "owner": "Persona->Imagination",
        "item": item,
        "feedbackId": feedback_id,
        "consensusReceiptId": consensus_receipt_id,
        "requestedConsensusRoute": consensus_route,
        "consensusPacketRef": consensus_packet_ref,
        "adoptionGate": adoption_gate,
        "publicDiscussionRefs": public_discussion_refs,
        "publicDiscussionRefCount": public_discussion_refs.len(),
        "candidateActionRefs": candidate_action_refs,
        "candidateActionRefCount": candidate_action_refs.len(),
        "planActionItemsReceiptId": plan_action_item_receipt_id,
        "planSafeActionFamily": plan_safe_action_family,
        "planModelAuthored": plan_model_authored,
        "handsAuthorityGranted": false,
        "durableStateAdmitted": false,
        "publicationAuthorized": false,
        "privateStateExposed": false
    });
    let tui_rows = vec![
        format!("item {item}"),
        format!("branch {branch}"),
        format!("gate {gate}"),
        format!("blocker {blocker}"),
        format!("next {next_safe_move}"),
        format!(
            "CONSENSUS | feedback={} | receipt={} | route={} | publicRefs={} | candidates={} | planFamily={} | modelAuthored={} | private=false",
            intake_consensus_readback["feedbackId"]
                .as_str()
                .unwrap_or("missing"),
            intake_consensus_readback["consensusReceiptId"]
                .as_str()
                .unwrap_or("missing"),
            intake_consensus_readback["requestedConsensusRoute"]
                .as_str()
                .unwrap_or("missing"),
            intake_consensus_readback["publicDiscussionRefCount"]
                .as_u64()
                .unwrap_or(0),
            intake_consensus_readback["candidateActionRefCount"]
                .as_u64()
                .unwrap_or(0),
            intake_consensus_readback["planSafeActionFamily"]
                .as_str()
                .unwrap_or("missing"),
            intake_consensus_readback["planModelAuthored"]
                .as_bool()
                .unwrap_or(false),
        ),
        format!("closure {closure_status} soul {soul_verdict}"),
        format!("publication {publication_status} sync {sync_status}"),
        "private false".to_string(),
    ];
    let receipt_refs = repo_work_existing_receipt_refs(&[
        ("accept", &accept_receipt_path),
        ("plan", &plan_receipt_path),
        ("run", &run_receipt_path),
        ("adopt", &adopt_receipt_path),
        ("execute", &execute_receipt_path),
        ("close-review", &close_review_receipt_path),
        ("close", &close_receipt_path),
    ]);
    let changed_paths_for_entry = changed_paths.clone();
    let commit_sha_for_entry = commit_sha.clone().unwrap_or_default();

    let receipt_chain = repo_work_receipt_state(
        &accept_receipt_path,
        &plan_receipt_path,
        &run_receipt_path,
        &adopt_receipt_path,
        &execute_receipt_path,
        &close_receipt_path,
    );
    let proof_artifacts = repo_work_proof_artifact_rows(&[
        ("accept", &accept_receipt_path),
        ("plan", &plan_receipt_path),
        ("run", &run_receipt_path),
        ("adopt", &adopt_receipt_path),
        ("execute", &execute_receipt_path),
        ("close-review", &close_review_receipt_path),
        ("close", &close_receipt_path),
    ])?;
    let proof_publication_rows = Vec::<Value>::new();
    let proof_bundle_tui_rows = repo_work_proof_bundle_tui_rows(
        &item,
        &branch,
        gate,
        blocker,
        closure_status,
        publication_status,
        sync_status,
        proof_artifacts
            .iter()
            .filter(|row| row.get("artifactStatus").and_then(Value::as_str) == Some("present"))
            .count(),
    );
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let now_for_entry = now.clone();
    let proof_bundle = json!({
        "schemaVersion": "epiphany.repo_work_proof_bundle.v0",
        "bundleId": format!("repo-work-proof-bundle-{item_slug}"),
        "generatedAt": now,
        "workspace": workspace,
        "item": item,
        "branch": branch,
        "currentGate": gate,
        "blocker": blocker,
        "nextSafeMove": next_safe_move,
        "acceptReceiptPath": accept_receipt_path,
        "planReceiptPath": existing_path_value(&plan_receipt_path),
        "runReceiptPath": existing_path_value(&run_receipt_path),
        "adoptReceiptPath": existing_path_value(&adopt_receipt_path),
        "executeReceiptPath": existing_path_value(&execute_receipt_path),
        "closeReviewReceiptPath": existing_path_value(&close_review_receipt_path),
        "closeReceiptPath": existing_path_value(&close_receipt_path),
        "changedPaths": changed_paths,
        "commitSha": commit_sha,
        "intakeConsensus": intake_consensus_readback,
        "soulVerdict": soul_verdict,
        "mindStateCommitReceiptId": close_receipt.as_ref().and_then(|receipt| string_from_json(receipt, &["mind", "stateCommitReceiptId"])),
        "bifrostPublicationReceiptId": null,
        "githubPublicationReceiptId": null,
        "upstreamMainSynced": false,
        "artifactRows": proof_artifacts,
        "publicationRows": proof_publication_rows,
        "tuiRows": proof_bundle_tui_rows,
        "privateStateExposed": false
    });
    let rows = json!([
        {"key": "item", "value": item, "status": "current"},
        {"key": "branch", "value": branch, "status": "current"},
        {"key": "gate", "value": gate, "status": if blocker == "none" { "ready" } else { "blocked" }},
        {"key": "blocker", "value": blocker, "status": if blocker == "none" { "clear" } else { "attention" }},
        {"key": "consensus", "value": proof_bundle["intakeConsensus"]["consensusReceiptId"], "status": proof_bundle["intakeConsensus"]["requestedConsensusRoute"]},
        {"key": "closure", "value": closure_status, "status": closure_status},
        {"key": "publication", "value": publication_status, "status": publication_status},
        {"key": "sync", "value": sync_status, "status": sync_status},
        {"key": "private", "value": "false", "status": "sealed"}
    ]);
    let receipt = json!({
        "schemaVersion": "epiphany.repo_work_overview_receipt.v0",
        "createdAt": now,
        "workspace": workspace,
        "item": item,
        "branch": branch,
        "currentGate": gate,
        "blocker": blocker,
        "nextSafeMove": next_safe_move,
        "receiptChain": receipt_chain,
        "proofBundle": proof_bundle,
        "intakeConsensus": proof_bundle["intakeConsensus"],
        "rows": rows,
        "authority": {
            "owner": "Eyes",
            "sightOnly": true,
            "branchLocalOnly": false,
            "publicationAuthorized": false,
            "mergeAuthorized": false,
            "serviceLifecycleAuthorized": false,
            "crossRepoMutationAuthorized": false,
            "privateStateExposed": false
        },
        "privateStateExposed": false
    });
    if args.write_receipt {
        write_json(&overview_receipt_path, &receipt)?;
    }
    let mut verse_projection = Value::Null;
    if args.write_receipt {
        if let Some(store) = local_verse_store.as_ref() {
            let entry = EpiphanyCultMeshRepoWorkOverviewEntry {
                schema_version: EPIPHANY_CULTMESH_REPO_WORK_OVERVIEW_SCHEMA_VERSION.to_string(),
                runtime_id: runtime_id.clone(),
                verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
                overview_id: format!("repo-work-overview-{item_slug}"),
                generated_at: now_for_entry,
                workspace: workspace.display().to_string(),
                item: item.clone(),
                branch: branch.clone(),
                current_gate: gate.to_string(),
                blocker: blocker.to_string(),
                next_safe_move: next_safe_move.to_string(),
                changed_paths: changed_paths_for_entry.clone(),
                commit_sha: commit_sha_for_entry.clone(),
                soul_verdict: soul_verdict.to_string(),
                publication_status: publication_status.to_string(),
                sync_status: sync_status.to_string(),
                receipt_refs: receipt_refs.clone(),
                tui_rows: tui_rows.clone(),
                proof_bundle_ref: overview_receipt_path.display().to_string(),
                private_state_exposed: false,
                notes: vec![
                    "Repo work overview is compact local Verse sight; raw worker thoughts and receipt payload bodies remain sealed.".to_string(),
                    "Persona/public feedback and candidate-action refs are shown only as compact Imagination consensus readback; adoption still requires Mind and Bifrost gates.".to_string(),
                    "downstream Eve lowerers may project these rows, but they do not own scheduling, publication, merge, service lifecycle, or cross-repo mutation.".to_string(),
                ],
            };
            let written = write_epiphany_cultmesh_repo_work_overview(store, entry)?;
            verse_projection = json!({
                "localVerseStore": store,
                "documentType": "epiphany.cultmesh.repo_work_overview",
                "overviewId": written.overview_id,
                "latestKey": "gamecult-local/repo-work-overview/latest",
                "tuiRows": written.tui_rows,
                "privateStateExposed": written.private_state_exposed
            });
        }
    }
    Ok(json!({
        "schemaVersion": "epiphany.repo_work_overview.v0",
        "status": "overview-ready",
        "workspace": receipt["workspace"],
        "item": receipt["item"],
        "branch": receipt["branch"],
        "currentGate": receipt["currentGate"],
        "blocker": receipt["blocker"],
        "nextSafeMove": receipt["nextSafeMove"],
        "receiptPath": if args.write_receipt { Value::String(overview_receipt_path.display().to_string()) } else { Value::Null },
        "proofBundle": receipt["proofBundle"],
        "intakeConsensus": receipt["intakeConsensus"],
        "rows": receipt["rows"],
        "verseProjection": verse_projection,
        "authority": receipt["authority"],
        "privateStateExposed": false
    }))
}

fn run_readiness(args: ReadinessArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let artifact_dir = args
        .artifact_dir
        .clone()
        .unwrap_or_else(|| workspace.join(".epiphany").join("work"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;

    let overview = run_overview(OverviewArgs {
        workspace: workspace.clone(),
        item: args.item.clone(),
        accept_receipt: args.accept_receipt.clone(),
        artifact_dir: Some(artifact_dir.clone()),
        write_receipt: args.write_receipt,
    })?;
    let item = overview
        .get("item")
        .and_then(Value::as_str)
        .unwrap_or("work-item")
        .to_string();
    let item_slug = sanitize(&item);
    let accept_receipt_path = resolve_accept_receipt(&workspace, Some(&item), args.accept_receipt)?;
    let plan_receipt_path = work_receipt_path(&workspace, "plan", &item);
    let execute_receipt_path = work_receipt_path(&workspace, "execute", &item);
    let close_receipt_path = work_receipt_path(&workspace, "close", &item);
    let queue_run_receipt_path = artifact_dir.join("work-queue-run.json");
    let init_receipt_path = workspace
        .join(".epiphany")
        .join("repo-init")
        .join("repo-swarm-init-receipt.json");
    let online_receipt_path = workspace
        .join(".epiphany")
        .join("swarm-online")
        .join("repo-swarm-online-receipt.json");
    let public_proof_path = args.public_proof.unwrap_or_else(|| {
        workspace
            .join(".epiphany")
            .join("public")
            .join("proof-bundles")
            .join(format!("repo-work-public-proof-{item_slug}.json"))
    });
    let readiness_receipt_path = artifact_dir.join(format!("work-readiness-{item_slug}.json"));
    let accept_receipt = read_json(&accept_receipt_path)?;
    let runtime_id = string_from_json(&accept_receipt, &["runtimeId"])
        .unwrap_or_else(|| "repo-swarm-local".to_string());
    let local_verse_store = path_from_json(&accept_receipt, &["localVerseStore"])
        .or_else(|| path_from_json(&accept_receipt, &["localVerseStorePath"]));

    let close_receipt = read_json_if_exists(&close_receipt_path)?;
    let close_status = close_receipt
        .as_ref()
        .and_then(|receipt| receipt.get("status").and_then(Value::as_str));
    let soul_passed = close_receipt
        .as_ref()
        .and_then(|receipt| string_from_json(receipt, &["soul", "verdict"]))
        .as_deref()
        == Some("passed");
    let modeling_map_admitted = close_receipt
        .as_ref()
        .and_then(|receipt| bool_from_json(receipt, &["authority", "durableStateAdmitted"]))
        .unwrap_or(false)
        || close_receipt
            .as_ref()
            .and_then(|receipt| string_from_json(receipt, &["mind", "stateCommitReceiptId"]))
            .is_some();

    let mut rows = vec![
        readiness_path_row(
            "repo-init",
            "Self",
            "epiphany.repo_swarm_init_receipt.v0",
            &init_receipt_path,
            init_receipt_path.exists(),
            "Fresh repo was initialized as an Epiphany Body.",
        )?,
        readiness_path_row(
            "swarm-online",
            "Self",
            "epiphany.repo_swarm_online_receipt.v0",
            &online_receipt_path,
            online_receipt_path.exists(),
            "Repo-local swarm published compact CultMesh sight.",
        )?,
        readiness_path_row(
            "persona-intake",
            "Persona/Imagination",
            "epiphany.repo_work_accept_receipt.v0",
            &accept_receipt_path,
            accept_receipt_path.exists(),
            "Persona or Bifrost pressure was accepted without Hands authority.",
        )?,
        readiness_path_row(
            "imagination-plan",
            "Imagination",
            "epiphany.repo_work_action_plan_receipt.v0",
            &plan_receipt_path,
            plan_receipt_path.exists(),
            "Imagination produced action-item/plan cargo.",
        )?,
        readiness_path_row(
            "self-queue-run",
            "Self",
            "epiphany.repo_work_queue_run_receipt.v0",
            &queue_run_receipt_path,
            queue_run_receipt_path.exists(),
            "Self queue-run pulse selected or advanced repo work.",
        )?,
        readiness_path_row(
            "hands-branch-work",
            "Hands",
            "epiphany.repo_work_execute_receipt.v0",
            &execute_receipt_path,
            execute_receipt_path.exists(),
            "Hands executed branch-local work with receipts.",
        )?,
        readiness_path_row(
            "soul-closure",
            "Soul",
            "epiphany.repo_work_closure_receipt.v0",
            &close_receipt_path,
            close_status == Some("closed") && soul_passed,
            "Soul passed closure for the Hands consequence.",
        )?,
        readiness_path_row(
            "modeling-mind-admission",
            "Modeling/Mind",
            "epiphany.mind.state_commit_receipt",
            &close_receipt_path,
            modeling_map_admitted,
            "Modeling map update was admitted through Mind.",
        )?,
        readiness_path_row(
            "public-proof",
            "Bifrost",
            "epiphany.repo_work_public_proof_bundle.v0",
            &public_proof_path,
            public_proof_path.exists(),
            "Redacted public proof bundle exists.",
        )?,
        readiness_missing_row(
            "bifrost-publication",
            "Bifrost",
            "provider-authored publication receipts",
            "Wait for Bifrost and the GitHub publication adapter; legacy local publish aggregates are not authority.",
        ),
        readiness_missing_row(
            "upstream-main-sync",
            "Bifrost/GitHub/Maintainer",
            "provider-authored merge evidence + Git ancestry",
            "Wait for provider-authored merge evidence, then inspect upstream ancestry; legacy local sync aggregates are not authority.",
        ),
    ];
    let default_idunn_lifecycle_receipt = artifact_dir.join("repo-work-service-audit.json");
    let idunn_lifecycle_path = args.idunn_lifecycle_receipt.as_ref().or_else(|| {
        if default_idunn_lifecycle_receipt.exists() {
            Some(&default_idunn_lifecycle_receipt)
        } else {
            None
        }
    });
    if let Some(path) = idunn_lifecycle_path {
        rows.push(idunn_lifecycle_readiness_row(path)?);
    } else {
        rows.push(readiness_missing_row(
            "idunn-lifecycle",
            "Idunn",
            "epiphany.repo_work_service_audit.v0",
            "Supply --idunn-lifecycle-receipt or run repo-work-service-audit into the repo work artifact directory.",
        ));
    }
    let default_deployment_aftercare_audit_receipt =
        artifact_dir.join("deployment-aftercare-audit.json");
    let deployment_aftercare_audit_path =
        args.deployment_aftercare_audit_receipt
            .as_ref()
            .or_else(|| {
                if default_deployment_aftercare_audit_receipt.exists() {
                    Some(&default_deployment_aftercare_audit_receipt)
                } else {
                    None
                }
            });
    if let Some(receipt_ref) = args.deployment_aftercare_audit_receipt_ref.as_ref() {
        let readiness_store = local_verse_store
            .clone()
            .unwrap_or_else(|| workspace.join(".epiphany").join("local-verse.ccmp"));
        let receipt = if receipt_ref.trim().is_empty() || receipt_ref.trim() == "latest" {
            load_latest_epiphany_cultmesh_idunn_aftercare_audit_receipt(
                &readiness_store,
                runtime_id.clone(),
            )?
        } else {
            load_epiphany_cultmesh_idunn_aftercare_audit_receipt(
                &readiness_store,
                runtime_id.clone(),
                receipt_ref,
            )?
        };
        if let Some(receipt) = receipt {
            let status_ok = matches!(receipt.status.as_str(), "ok" | "complete" | "passed");
            let satisfied = receipt.schema_version
                == "gamecult.idunn.deployment_aftercare_audit.v0"
                && status_ok
                && !receipt.private_state_exposed;
            rows.push(json!({
                "kind": "deployment-aftercare",
                "owner": "Idunn/Soul",
                "requiredSchema": "gamecult.idunn.deployment_aftercare_audit.v0",
                "evidenceRef": receipt_ref,
                "artifactStatus": "cultmesh",
                "schemaVersion": receipt.schema_version,
                "documentStatus": receipt.status,
                "source": "cultmesh",
                "localVerseStore": readiness_store.display().to_string(),
                "runtimeId": runtime_id.clone(),
                "receiptId": receipt.receipt_id,
                "deploymentReceiptId": receipt.deployment_receipt_id,
                "auditRef": receipt.audit_ref,
                "satisfied": satisfied,
                "status": if satisfied { "satisfied" } else { "missing" },
                "note": "Idunn deployment aftercare audit was supplied from repo-local CultMesh sight.",
                "deploymentAuthority": false,
                "gitPushAuthority": false,
                "serviceLifecycleAuthority": false,
                "privateStateExposed": receipt.private_state_exposed
            }));
        } else {
            rows.push(readiness_missing_row(
                "deployment-aftercare",
                "Idunn/Soul",
                "gamecult.idunn.deployment_aftercare_audit.v0",
                "Supply --deployment-aftercare-audit-receipt-ref after Idunn records deployment aftercare in the repo-local Verse.",
            ));
        }
    } else if let Some(path) = deployment_aftercare_audit_path {
        let receipt = read_json_if_exists(path)?;
        let satisfied = receipt.as_ref().is_some_and(|receipt| {
            string_from_json(receipt, &["schemaVersion"]).as_deref()
                == Some("epiphany.repo_deployment_aftercare_audit.v0")
                && string_from_json(receipt, &["status"]).as_deref() == Some("complete")
                && bool_from_json(receipt, &["deploymentComplete"]) == Some(true)
                && bool_from_json(receipt, &["deploymentAuthority"]) == Some(false)
                && bool_from_json(receipt, &["gitPushAuthority"]) == Some(false)
                && bool_from_json(receipt, &["serviceLifecycleAuthority"]) == Some(false)
                && bool_from_json(receipt, &["privateStateExposed"]) == Some(false)
        });
        rows.push(readiness_path_row(
            "deployment-aftercare",
            "Idunn/Soul",
            "epiphany.repo_deployment_aftercare_audit.v0",
            path,
            satisfied,
            "Idunn deployment aftercare audit completed and stayed sight-only.",
        )?);
    } else {
        rows.push(readiness_missing_row(
            "deployment-aftercare",
            "Idunn/Soul",
            "epiphany.repo_deployment_aftercare_audit.v0",
            "Supply --deployment-aftercare-audit-receipt, --deployment-aftercare-audit-receipt-ref, or run deployment-aftercare-audit into the repo work artifact directory.",
        ));
    }
    let default_tool_directory_receipt = artifact_dir.join("tool-directory.json");
    let tool_directory_path = args.tool_directory_receipt.as_ref().or_else(|| {
        if default_tool_directory_receipt.exists() {
            Some(&default_tool_directory_receipt)
        } else {
            None
        }
    });
    if let Some(path) = tool_directory_path {
        rows.push(tool_directory_readiness_row(path)?);
    } else {
        rows.push(readiness_missing_row(
            "tool-directory",
            "Odin",
            "epiphany.cultmesh.daemon_tool_directory.v0",
            "Supply --tool-directory-receipt or run tool-directory sight into the repo work artifact directory.",
        ));
    }
    rows.push(json!({
        "kind": "private-state-redaction",
        "owner": "Soul/Bifrost",
        "requiredSchema": "epiphany.private_state_redaction_report.v0",
        "evidenceRef": overview["proofBundle"].get("bundleId").cloned().unwrap_or(Value::String("missing".to_string())),
        "satisfied": overview
            .get("privateStateExposed")
            .and_then(Value::as_bool)
            .map(|exposed| !exposed)
            .unwrap_or(false),
        "status": if overview.get("privateStateExposed").and_then(Value::as_bool) == Some(false) { "satisfied" } else { "missing" },
        "note": "Readiness sight observed privateStateExposed=false on overview/proof-bundle projection."
    }));

    let missing_rows = rows
        .iter()
        .filter(|row| row.get("satisfied").and_then(Value::as_bool) != Some(true))
        .count();
    let satisfied_rows = rows.len().saturating_sub(missing_rows);
    let missing_kinds = rows
        .iter()
        .filter(|row| row.get("satisfied").and_then(Value::as_bool) != Some(true))
        .filter_map(|row| row.get("kind").and_then(Value::as_str))
        .map(str::to_string)
        .collect::<Vec<_>>();
    let verdict = if missing_rows == 0 {
        "ready"
    } else {
        "not-ready"
    };
    let next_safe_move = if missing_rows == 0 {
        "Route this sight receipt to Maintainer/Soul/Mind/Bifrost review; readiness approval is still external to this command."
    } else {
        "Complete the missing readiness rows, then rerun epiphany-work readiness before asking for review."
    };
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let readiness_tui_rows = vec![
        format!(
            "REPO-WORK-READINESS | item={} | status={} | missing={} | satisfied={} | receipt={} | approvalAuth=false | publicationAuth=false | serviceAuth=false | handsAuth=false | private=false",
            item,
            verdict,
            missing_rows,
            satisfied_rows,
            readiness_receipt_path.display()
        ),
        format!(
            "REPO-WORK-READINESS-GAPS | item={} | missingKinds={} | next={} | private=false",
            item,
            if missing_kinds.is_empty() {
                "none".to_string()
            } else {
                missing_kinds.join(",")
            },
            next_safe_move
        ),
    ];
    let receipt = json!({
        "schemaVersion": "epiphany.repo_work_readiness_report.v0",
        "createdAt": now,
        "workspace": workspace,
        "item": item,
        "status": verdict,
        "verdict": verdict,
        "missingRequiredCount": missing_rows,
        "overviewReceiptPath": overview["receiptPath"],
        "proofBundleId": overview["proofBundle"]["bundleId"],
        "rows": rows,
        "allowedVerdicts": ["ready", "ready-with-caveats", "not-ready", "needs-human-review"],
        "authority": {
            "owner": "Soul/Mind/Bifrost/Maintainer",
            "sightOnly": true,
            "readinessApprovalAuthorized": false,
            "durableStateCommitAuthorized": false,
            "publicationAuthorized": false,
            "bifrostPublicationAuthorized": false,
            "githubPrAuthorized": false,
            "mergeAuthorized": false,
            "upstreamSyncAuthorized": false,
            "deploymentAuthority": false,
            "serviceLifecycleAuthority": false,
            "handsActionAuthorized": false,
            "crossBodyMutationAuthorized": false,
            "privateVerseRummaging": false,
            "privateStateExposed": false
        },
        "privateStateExposed": false,
        "nextSafeMove": next_safe_move
    });
    if args.write_receipt {
        write_json(&readiness_receipt_path, &receipt)?;
    }
    let mut verse_projection = Value::Null;
    if args.write_receipt {
        if let Some(store) = local_verse_store.as_ref() {
            let entry = EpiphanyCultMeshRepoWorkReadinessEntry {
                schema_version: EPIPHANY_CULTMESH_REPO_WORK_READINESS_SCHEMA_VERSION.to_string(),
                runtime_id: runtime_id.clone(),
                verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
                readiness_id: format!("repo-work-readiness-{item_slug}"),
                generated_at: now.clone(),
                workspace: workspace.display().to_string(),
                item: item.clone(),
                status: verdict.to_string(),
                missing_required_count: missing_rows as u32,
                satisfied_required_count: satisfied_rows as u32,
                readiness_receipt_ref: readiness_receipt_path.display().to_string(),
                overview_receipt_ref: overview["receiptPath"]
                    .as_str()
                    .unwrap_or("missing")
                    .to_string(),
                proof_bundle_id: overview["proofBundle"]["bundleId"]
                    .as_str()
                    .unwrap_or("missing")
                    .to_string(),
                missing_kinds: missing_kinds.clone(),
                tui_rows: readiness_tui_rows.clone(),
                sight_only: true,
                readiness_approval_authorized: false,
                publication_authorized: false,
                service_lifecycle_authority: false,
                hands_action_authorized: false,
                private_state_exposed: false,
                notes: vec![
                    "Repo work readiness is reviewable sight only; Maintainer/Soul/Mind/Bifrost own any readiness approval.".to_string(),
                    "Bifrost owns publication gates; Hands performs bounded PR actions; GitHub supplies provider receipts; Idunn owns service lifecycle.".to_string(),
                    "Downstream consumers may project these rows without scheduling, publication, merge, deployment, service lifecycle, cross-body mutation, or private Verse authority.".to_string(),
                ],
            };
            let written = write_epiphany_cultmesh_repo_work_readiness(store, entry)?;
            verse_projection = json!({
                "localVerseStore": store,
                "documentType": "epiphany.cultmesh.repo_work_readiness",
                "readinessId": written.readiness_id,
                "latestKey": "gamecult-local/repo-work-readiness/latest",
                "tuiRows": written.tui_rows,
                "privateStateExposed": written.private_state_exposed
            });
        }
    }
    Ok(json!({
        "schemaVersion": "epiphany.repo_work_readiness.v0",
        "status": verdict,
        "workspace": receipt["workspace"],
        "item": receipt["item"],
        "receiptPath": if args.write_receipt { Value::String(readiness_receipt_path.display().to_string()) } else { Value::Null },
        "missingRequiredCount": missing_rows,
        "rows": receipt["rows"],
        "verseProjection": verse_projection,
        "authority": receipt["authority"],
        "privateStateExposed": false,
        "nextSafeMove": receipt["nextSafeMove"]
    }))
}

fn run_deployment_config_audit(args: DeploymentConfigAuditArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let artifact_dir = args
        .artifact_dir
        .unwrap_or_else(|| workspace.join(".epiphany").join("work"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;

    let config_path = workspace.join(".epiphany").join("deployment.toml");
    let receipt_path = artifact_dir.join("deployment-config-audit.json");
    let generated_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let mut assertions = Vec::new();
    let mut parsed_config = None;

    if config_path.exists() {
        let config_text = fs::read_to_string(&config_path)
            .with_context(|| format!("failed to read {}", config_path.display()))?;
        match parse_repo_deployment_config(&config_text) {
            Ok(config) => {
                push_assertion(
                    &mut assertions,
                    "deployment-config-typed-toml",
                    true,
                    "Deployment config parses as the typed TOML contract.".to_string(),
                );
                parsed_config = Some(config);
            }
            Err(error) => push_assertion(
                &mut assertions,
                "deployment-config-typed-toml",
                false,
                format!("Deployment config parse failed: {error:#}"),
            ),
        }
        push_assertion(
            &mut assertions,
            "deployment-config-schema-present",
            parsed_config.as_ref().is_some_and(|config| {
                config.schema_version == "epiphany.repo_deployment_config.v0"
            }),
            "Deployment config carries the schema version.".to_string(),
        );
        push_assertion(
            &mut assertions,
            "deployment-config-family-present",
            parsed_config
                .as_ref()
                .is_some_and(|config| config.safe_action_family == "repo.deployment_config"),
            "Deployment config carries the safe action family.".to_string(),
        );
        push_assertion(
            &mut assertions,
            "deployment-config-idunn-trigger",
            parsed_config.as_ref().is_some_and(|config| {
                let deployment = &config.deployment;
                !deployment.enabled
                    && deployment.owner == "Idunn"
                    && deployment.trigger == "git-push-observed-by-idunn"
                    && deployment.watched_ref == "refs/heads/main"
                    && deployment.deployment_script_ref == "deploy/idunn-deploy.ps1"
                    && deployment.deployment_script_hash_required
                    && deployment.deployment_script_review_required
                    && deployment.host_access_policy_ref_required
                    && !deployment.secret_values_embedded
                    && deployment.rollback_plan_ref_required
                    && deployment.aftercare_checks_required
                    && deployment.idunn_receipt_required
                    && deployment.aftercare_audit_required
            }),
            "Deployment config names disabled Idunn git-push trigger, reviewed script/hash, policy, rollback, and aftercare requirements."
                .to_string(),
        );
        push_assertion(
            &mut assertions,
            "deployment-config-cultmesh-contract",
            parsed_config.as_ref().is_some_and(|config| {
                let cultmesh = &config.cultmesh;
                cultmesh.local_verse == "gamecult-local"
                    && cultmesh.capability_family == "gamecult.idunn.deployment"
                    && cultmesh.intent_contract == "gamecult.idunn.deployment_intent.v0"
                    && cultmesh.receipt_contract == "gamecult.idunn.deployment_receipt.v0"
                    && cultmesh.aftercare_contract == "gamecult.idunn.deployment_aftercare_audit.v0"
                    && cultmesh.daemon_owns_execution
            }),
            "Deployment config routes execution through Idunn CultMesh contracts.".to_string(),
        );
        push_assertion(
            &mut assertions,
            "deployment-config-receipt-contract",
            parsed_config.as_ref().is_some_and(|config| {
                let receipts = &config.required_receipts;
                receipts.mind_adoption == "epiphany.repo_work_mind_adoption_decision.v0"
                    && receipts.soul_review == "epiphany.repo_work_closure_review.v0"
                    && receipts.maintainer_review == "gamecult.maintainer.review_receipt.v0"
                    && receipts.secret_policy == "epiphany.repo_secret_policy_request.v0"
                    && receipts.idunn_deployment == "gamecult.idunn.deployment_receipt.v0"
                    && receipts.aftercare_audit
                        == "gamecult.idunn.deployment_aftercare_audit.v0"
            }),
            "Deployment config names Mind, Soul, maintainer, secret-policy, Idunn deployment, and aftercare receipts."
                .to_string(),
        );
        push_assertion(
            &mut assertions,
            "deployment-config-authority-seals",
            parsed_config.as_ref().is_some_and(|config| {
                let authority = &config.authority;
                authority.configuration_only
                    && !authority.direct_deployment_authority
                    && !authority.direct_ssh_authority
                    && !authority.direct_git_push_authority
                    && !authority.direct_service_lifecycle_authority
                    && !authority.direct_hands_authority
                    && !authority.publication_authorized
                    && !authority.merge_authorized
                    && !authority.cross_body_mutation_authorized
                    && !authority.private_verse_rummaging
                    && authority.idunn_deployment_authority_required
            }),
            "Deployment config denies deployment, SSH, git-push, service lifecycle, Hands, publication, merge, and cross-body authority."
                .to_string(),
        );
        push_assertion(
            &mut assertions,
            "deployment-config-private-seal",
            parsed_config
                .as_ref()
                .is_some_and(|config| !config.private_state_exposed),
            "Deployment config preserves the private-state seal.".to_string(),
        );
    } else {
        push_assertion(
            &mut assertions,
            "deployment-config-present",
            false,
            format!("Deployment config missing at {}", config_path.display()),
        );
    }

    let all_passed = assertions
        .iter()
        .all(|assertion| assertion.get("passed").and_then(Value::as_bool) == Some(true));
    let status = if all_passed {
        "ready-for-idunn-review"
    } else if config_path.exists() {
        "invalid"
    } else {
        "missing"
    };
    let daemon_owns_execution = all_passed
        && parsed_config
            .as_ref()
            .is_some_and(|config| config.cultmesh.daemon_owns_execution);
    let receipt = json!({
        "schemaVersion": "epiphany.repo_deployment_config_audit.v0",
        "auditId": "repo-deployment-config-audit",
        "generatedAt": generated_at,
        "status": status,
        "workspace": workspace,
        "configPath": config_path,
        "receiptPath": receipt_path,
        "readyForIdunnReview": status == "ready-for-idunn-review",
        "executionAuthorized": false,
        "deploymentAuthority": false,
        "sshAuthority": false,
        "gitPushAuthority": false,
        "serviceLifecycleAuthority": false,
        "handsAuthority": false,
        "publicationAuthorized": false,
        "mergeAuthorized": false,
        "crossBodyMutationAuthorized": false,
        "daemonOwnsExecution": daemon_owns_execution,
        "nextGate": if status == "ready-for-idunn-review" {
            "idunn.review_deployment_config"
        } else {
            "repo.fix_deployment_config"
        },
        "assertions": assertions,
        "privateStateExposed": false
    });
    if args.write_receipt {
        write_json(&receipt_path, &receipt)?;
    }
    Ok(receipt)
}

fn run_deployment_execution_runbook(args: DeploymentExecutionRunbookArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let artifact_dir = args
        .artifact_dir
        .unwrap_or_else(|| workspace.join(".epiphany").join("work"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;

    let config_path = workspace.join(".epiphany").join("deployment.toml");
    let config_text = if config_path.exists() {
        fs::read_to_string(&config_path)
            .with_context(|| format!("failed to read {}", config_path.display()))?
    } else {
        String::new()
    };
    let parsed_config = parse_repo_deployment_config(&config_text).ok();
    let watched_ref = parsed_config
        .as_ref()
        .map(|config| config.deployment.watched_ref.clone())
        .unwrap_or_else(|| "missing".to_string());
    let deployment_script_ref = parsed_config
        .as_ref()
        .map(|config| config.deployment.deployment_script_ref.clone())
        .unwrap_or_else(|| "missing".to_string());
    let current_branch = git_output(&workspace, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let current_commit = git_output(&workspace, &["rev-parse", "HEAD"])?;
    let generated_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let receipt_path = artifact_dir.join("deployment-execution-runbook.json");
    let runbook_dir = artifact_dir.join("idunn-deployment");
    let runbook_path = runbook_dir.join("idunn-git-push-runbook.ps1");
    let audit = run_deployment_config_audit(DeploymentConfigAuditArgs {
        workspace: workspace.clone(),
        artifact_dir: Some(artifact_dir.clone()),
        write_receipt: args.write_receipt,
    })?;
    let ready_for_idunn = audit.get("readyForIdunnReview").and_then(Value::as_bool) == Some(true);
    let push_command = format!("git push {} HEAD:{watched_ref}", args.remote);
    let operator_execution_command = format!(
        "powershell -ExecutionPolicy Bypass -File {} -Remote {}",
        powershell_single_quoted(&runbook_path.display().to_string()),
        powershell_single_quoted(&args.remote)
    );
    let mut runbook_written = false;
    let mut runbook_sha256 = Value::Null;

    if ready_for_idunn && args.write_receipt {
        fs::create_dir_all(&runbook_dir)
            .with_context(|| format!("failed to create {}", runbook_dir.display()))?;
        let runbook = [
            "# schema_version = \"epiphany.repo_deployment_execution_runbook.v0\"".to_string(),
            "# owner = \"Idunn\"".to_string(),
            "# authority = \"explicit-operator-git-push\"".to_string(),
            "# generated_by = \"epiphany-work deployment-execution-runbook\"".to_string(),
            "param([string]$Remote = 'origin')".to_string(),
            "$ErrorActionPreference = 'Stop'".to_string(),
            format!("Set-Location -LiteralPath {}", powershell_single_quoted(&workspace.display().to_string())),
            "Write-Host 'Epiphany Idunn deployment handoff: operator-owned git push.'".to_string(),
            "Write-Host 'This runbook mutates the remote ref; run only with explicit operator authority.'".to_string(),
            "git status --short".to_string(),
            format!("git push $Remote HEAD:{watched_ref}"),
            "Write-Host 'Aftercare: wait for Idunn deployment receipt and aftercare audit on CultMesh.'".to_string(),
            "Write-Host 'Required receipts: gamecult.idunn.deployment_receipt.v0 and gamecult.idunn.deployment_aftercare_audit.v0.'".to_string(),
        ]
        .join("\n");
        fs::write(&runbook_path, runbook)
            .with_context(|| format!("failed to write {}", runbook_path.display()))?;
        runbook_written = true;
        runbook_sha256 = json!(file_sha256(&runbook_path)?);
    }

    let status = if ready_for_idunn {
        "ready-for-operator-git-push"
    } else {
        "blocked-config-not-ready"
    };
    let receipt = json!({
        "schemaVersion": "epiphany.repo_deployment_execution_runbook.v0",
        "runbookId": "repo-idunn-git-push-runbook",
        "generatedAt": generated_at,
        "status": status,
        "workspace": workspace,
        "configPath": config_path,
        "configAudit": audit,
        "receiptPath": receipt_path,
        "runbookPath": if runbook_written { json!(runbook_path) } else { Value::Null },
        "runbookSha256": runbook_sha256,
        "runbookWritten": runbook_written,
        "operatorExecutionCommand": if runbook_written {
            json!(operator_execution_command)
        } else {
            Value::Null
        },
        "gitPushCommand": push_command,
        "remote": args.remote,
        "watchedRef": watched_ref,
        "currentBranch": current_branch,
        "currentCommit": current_commit,
        "deploymentScriptRef": deployment_script_ref,
        "requiredIdunnReceipts": [
            "gamecult.idunn.deployment_receipt.v0",
            "gamecult.idunn.deployment_aftercare_audit.v0"
        ],
        "requiresExplicitOperatorAuthority": true,
        "mutatesRemoteWhenRun": true,
        "executionAuthorized": false,
        "deploymentAuthority": false,
        "sshAuthority": false,
        "gitPushAuthority": false,
        "serviceLifecycleAuthority": false,
        "handsAuthority": false,
        "publicationAuthorized": false,
        "mergeAuthorized": false,
        "crossBodyMutationAuthorized": false,
        "daemonOwnsExecution": true,
        "privateStateExposed": false
    });
    if args.write_receipt {
        write_json(&receipt_path, &receipt)?;
    }
    Ok(receipt)
}

fn run_deployment_aftercare_audit(args: DeploymentAftercareAuditArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let artifact_dir = args
        .artifact_dir
        .unwrap_or_else(|| workspace.join(".epiphany").join("work"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;

    let receipt_path = artifact_dir.join("deployment-aftercare-audit.json");
    let runbook_receipt_path = args
        .runbook_receipt
        .unwrap_or_else(|| artifact_dir.join("deployment-execution-runbook.json"));
    let local_verse_store = args
        .local_verse_store
        .unwrap_or_else(|| workspace.join(".epiphany").join("local-verse.ccmp"));
    let idunn_deployment_receipt_path = args.idunn_deployment_receipt;
    let idunn_deployment_receipt_ref = args.idunn_deployment_receipt_ref;
    let aftercare_audit_receipt_path = args.aftercare_audit_receipt;
    let aftercare_audit_receipt_ref = args.aftercare_audit_receipt_ref;
    let runtime_id = args.runtime_id;
    let local_verse_store_display = local_verse_store.display().to_string();
    let generated_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let mut assertions = Vec::new();

    let runbook_receipt = read_json_if_exists(&runbook_receipt_path)?;
    let runbook_ready = runbook_receipt.as_ref().is_some_and(|receipt| {
        string_from_json(receipt, &["schemaVersion"]).as_deref()
            == Some("epiphany.repo_deployment_execution_runbook.v0")
            && string_from_json(receipt, &["status"]).as_deref()
                == Some("ready-for-operator-git-push")
            && bool_from_json(receipt, &["requiresExplicitOperatorAuthority"]) == Some(true)
            && bool_from_json(receipt, &["mutatesRemoteWhenRun"]) == Some(true)
            && bool_from_json(receipt, &["privateStateExposed"]) == Some(false)
    });
    push_assertion(
        &mut assertions,
        "deployment-runbook-receipt-ready",
        runbook_ready,
        "Deployment execution runbook receipt is present, operator-owned, and ready for explicit git-push handoff."
            .to_string(),
    );

    let idunn_deployment_summary = if let Some(receipt_ref) = idunn_deployment_receipt_ref.as_ref()
    {
        let receipt = if receipt_ref.trim().is_empty() || receipt_ref.trim() == "latest" {
            load_latest_epiphany_cultmesh_idunn_deployment_receipt(
                &local_verse_store,
                runtime_id.clone(),
            )?
        } else {
            load_epiphany_cultmesh_idunn_deployment_receipt(
                &local_verse_store,
                runtime_id.clone(),
                receipt_ref,
            )?
        };
        if let Some(receipt) = receipt {
            let schema_ok = receipt.schema_version == "gamecult.idunn.deployment_receipt.v0";
            let status_ok = matches!(
                receipt.status.as_str(),
                "ok" | "complete" | "deployed" | "passed"
            );
            let private_ok = !receipt.private_state_exposed;
            push_assertion(
                &mut assertions,
                "idunn-deployment-receipt-valid",
                schema_ok && status_ok && private_ok,
                "Idunn deployment receipt carries the expected CultMesh contract, successful status, and private-state seal."
                    .to_string(),
            );
            json!({
                "source": "cultmesh",
                "store": local_verse_store_display.clone(),
                "runtimeId": runtime_id.clone(),
                "receiptRef": receipt_ref,
                "receiptId": receipt.receipt_id,
                "schemaVersion": receipt.schema_version,
                "status": receipt.status,
                "trigger": receipt.trigger,
                "watchedRef": receipt.watched_ref,
                "resultRef": receipt.result_ref,
                "privateStateExposed": receipt.private_state_exposed
            })
        } else {
            push_assertion(
                &mut assertions,
                "idunn-deployment-receipt-present",
                false,
                format!(
                    "Idunn deployment receipt ref {receipt_ref:?} was not found in the local Verse store."
                ),
            );
            Value::Null
        }
    } else if let Some(path) = idunn_deployment_receipt_path.as_ref() {
        let receipt = read_json(path)?;
        let schema_ok = string_from_json(&receipt, &["schemaVersion"]).as_deref()
            == Some("gamecult.idunn.deployment_receipt.v0");
        let status =
            string_from_json(&receipt, &["status"]).unwrap_or_else(|| "missing".to_string());
        let status_ok = matches!(status.as_str(), "ok" | "complete" | "deployed" | "passed");
        let private_ok =
            bool_from_json(&receipt, &["privateStateExposed"]).unwrap_or(false) == false;
        push_assertion(
            &mut assertions,
            "idunn-deployment-receipt-valid",
            schema_ok && status_ok && private_ok,
            "Idunn deployment receipt carries the expected contract, successful status, and private-state seal."
                .to_string(),
        );
        json!({
            "source": "file",
            "path": path,
            "schemaVersion": string_from_json(&receipt, &["schemaVersion"]).unwrap_or_else(|| "missing".to_string()),
            "status": status,
            "privateStateExposed": bool_from_json(&receipt, &["privateStateExposed"]).unwrap_or(false)
        })
    } else {
        push_assertion(
            &mut assertions,
            "idunn-deployment-receipt-present",
            false,
            "Idunn deployment receipt was not supplied.".to_string(),
        );
        Value::Null
    };

    let aftercare_summary = if let Some(receipt_ref) = aftercare_audit_receipt_ref.as_ref() {
        let receipt = if receipt_ref.trim().is_empty() || receipt_ref.trim() == "latest" {
            load_latest_epiphany_cultmesh_idunn_aftercare_audit_receipt(
                &local_verse_store,
                runtime_id.clone(),
            )?
        } else {
            load_epiphany_cultmesh_idunn_aftercare_audit_receipt(
                &local_verse_store,
                runtime_id.clone(),
                receipt_ref,
            )?
        };
        if let Some(receipt) = receipt {
            let schema_ok =
                receipt.schema_version == "gamecult.idunn.deployment_aftercare_audit.v0";
            let status_ok = matches!(receipt.status.as_str(), "ok" | "complete" | "passed");
            let private_ok = !receipt.private_state_exposed;
            push_assertion(
                &mut assertions,
                "idunn-aftercare-audit-receipt-valid",
                schema_ok && status_ok && private_ok,
                "Idunn aftercare audit receipt carries the expected CultMesh contract, successful status, and private-state seal."
                    .to_string(),
            );
            json!({
                "source": "cultmesh",
                "store": local_verse_store_display.clone(),
                "runtimeId": runtime_id.clone(),
                "receiptRef": receipt_ref,
                "receiptId": receipt.receipt_id,
                "schemaVersion": receipt.schema_version,
                "status": receipt.status,
                "checkedRef": receipt.checked_ref,
                "deploymentReceiptId": receipt.deployment_receipt_id,
                "auditRef": receipt.audit_ref,
                "privateStateExposed": receipt.private_state_exposed
            })
        } else {
            push_assertion(
                &mut assertions,
                "idunn-aftercare-audit-receipt-present",
                false,
                format!(
                    "Idunn aftercare audit receipt ref {receipt_ref:?} was not found in the local Verse store."
                ),
            );
            Value::Null
        }
    } else if let Some(path) = aftercare_audit_receipt_path.as_ref() {
        let receipt = read_json(path)?;
        let schema_ok = string_from_json(&receipt, &["schemaVersion"]).as_deref()
            == Some("gamecult.idunn.deployment_aftercare_audit.v0");
        let status =
            string_from_json(&receipt, &["status"]).unwrap_or_else(|| "missing".to_string());
        let status_ok = matches!(status.as_str(), "ok" | "complete" | "passed");
        let private_ok =
            bool_from_json(&receipt, &["privateStateExposed"]).unwrap_or(false) == false;
        push_assertion(
            &mut assertions,
            "idunn-aftercare-audit-receipt-valid",
            schema_ok && status_ok && private_ok,
            "Idunn aftercare audit receipt carries the expected contract, successful status, and private-state seal."
                .to_string(),
        );
        json!({
            "source": "file",
            "path": path,
            "schemaVersion": string_from_json(&receipt, &["schemaVersion"]).unwrap_or_else(|| "missing".to_string()),
            "status": status,
            "privateStateExposed": bool_from_json(&receipt, &["privateStateExposed"]).unwrap_or(false)
        })
    } else {
        push_assertion(
            &mut assertions,
            "idunn-aftercare-audit-receipt-present",
            false,
            "Idunn aftercare audit receipt was not supplied.".to_string(),
        );
        Value::Null
    };

    let all_passed = assertions
        .iter()
        .all(|assertion| assertion.get("passed").and_then(Value::as_bool) == Some(true));
    let status = if all_passed {
        "complete"
    } else if runbook_ready {
        "awaiting-idunn-receipts"
    } else {
        "runbook-not-ready"
    };
    let receipt = json!({
        "schemaVersion": "epiphany.repo_deployment_aftercare_audit.v0",
        "auditId": "repo-deployment-aftercare-audit",
        "generatedAt": generated_at,
        "status": status,
        "workspace": workspace,
        "receiptPath": receipt_path,
        "runbookReceiptPath": runbook_receipt_path,
        "localVerseStore": local_verse_store_display,
        "runtimeId": runtime_id,
        "idunnDeploymentReceipt": idunn_deployment_summary,
        "idunnAftercareAuditReceipt": aftercare_summary,
        "deploymentComplete": status == "complete",
        "requiresExplicitOperatorAuthority": false,
        "mutatesRemoteWhenRun": false,
        "executionAuthorized": false,
        "deploymentAuthority": false,
        "sshAuthority": false,
        "gitPushAuthority": false,
        "serviceLifecycleAuthority": false,
        "handsAuthority": false,
        "publicationAuthorized": false,
        "mergeAuthorized": false,
        "crossBodyMutationAuthorized": false,
        "daemonOwnsExecution": true,
        "assertions": assertions,
        "privateStateExposed": false
    });
    if args.write_receipt {
        write_json(&receipt_path, &receipt)?;
    }
    Ok(receipt)
}

fn run_export_proof(args: ExportProofArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    let overview = run_overview(OverviewArgs {
        workspace: workspace.clone(),
        item: args.item,
        accept_receipt: args.accept_receipt,
        artifact_dir: args.artifact_dir.clone(),
        write_receipt: true,
    })?;
    let proof_bundle = overview
        .get("proofBundle")
        .ok_or_else(|| anyhow!("overview did not return a proofBundle"))?;
    let item = proof_bundle
        .get("item")
        .and_then(Value::as_str)
        .unwrap_or("work-item")
        .to_string();
    let item_slug = sanitize(&item);
    let output = args.output.unwrap_or_else(|| {
        workspace
            .join(".epiphany")
            .join("public")
            .join("proof-bundles")
            .join(format!("repo-work-public-proof-{item_slug}.json"))
    });
    let public_bundle = repo_work_public_proof_bundle(&overview)?;
    write_json(&output, &public_bundle)?;
    let public_proof_sha256 = file_sha256(&output)?;
    let local_verse_store = args
        .local_verse_store
        .unwrap_or_else(|| workspace.join(".epiphany").join("local-verse.ccmp"));
    let artifact_row_count = public_bundle
        .get("artifactRows")
        .and_then(Value::as_array)
        .map(|rows| rows.len())
        .unwrap_or(0);
    let publication_row_count = public_bundle
        .get("publicationRows")
        .and_then(Value::as_array)
        .map(|rows| rows.len())
        .unwrap_or(0);
    let public_proof_tui_rows = repo_work_public_proof_tui_rows(
        &public_bundle,
        &output,
        &public_proof_sha256,
        artifact_row_count,
        publication_row_count,
    );
    let entry = EpiphanyCultMeshRepoWorkPublicProofEntry {
        schema_version: EPIPHANY_CULTMESH_REPO_WORK_PUBLIC_PROOF_SCHEMA_VERSION.to_string(),
        runtime_id: args.runtime_id.clone(),
        verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
        public_proof_id: format!("repo-work-public-proof-{item_slug}"),
        generated_at: string_from_json(&public_bundle, &["generatedAt"])
            .unwrap_or_else(|| Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true)),
        workspace: workspace.display().to_string(),
        item: item.clone(),
        branch: string_from_json(&public_bundle, &["branch"]).unwrap_or_else(|| "unknown".to_string()),
        current_gate: string_from_json(&public_bundle, &["currentGate"])
            .unwrap_or_else(|| "unknown".to_string()),
        blocker: string_from_json(&public_bundle, &["blocker"]).unwrap_or_else(|| "none".to_string()),
        next_safe_move: string_from_json(&public_bundle, &["nextSafeMove"])
            .unwrap_or_else(|| "none".to_string()),
        changed_paths: public_bundle
            .get("changedPaths")
            .and_then(Value::as_array)
            .map(|paths| {
                paths
                    .iter()
                    .filter_map(Value::as_str)
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        commit_sha: string_from_json(&public_bundle, &["commitSha"]).unwrap_or_else(|| "none".to_string()),
        soul_verdict: string_from_json(&public_bundle, &["soulVerdict"]).unwrap_or_else(|| "none".to_string()),
        upstream_main_synced: public_bundle
            .get("upstreamMainSynced")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        artifact_row_count: artifact_row_count as u32,
        publication_row_count: publication_row_count as u32,
        public_proof_ref: output.display().to_string(),
        public_proof_sha256: public_proof_sha256.clone(),
        tui_rows: public_proof_tui_rows,
        private_state_exposed: false,
        notes: vec![
            "Repo work public proof is a redacted local Verse index for public/Bifrost transport; raw receipt bodies, local receipt paths, worker thought, and private Verse contents remain sealed.".to_string(),
            "Odin may discover this proof, but Bifrost still owns publication, labor ledger, credit, and public consequence.".to_string(),
        ],
    };
    let written_public_proof =
        write_epiphany_cultmesh_repo_work_public_proof(&local_verse_store, entry)?;
    let verse_projection = json!({
        "localVerseStore": local_verse_store,
        "documentType": "epiphany.cultmesh.repo_work_public_proof",
        "publicProofId": written_public_proof.public_proof_id,
        "latestKey": "gamecult-local/repo-work-public-proof/latest",
        "publicProofRef": written_public_proof.public_proof_ref,
        "publicProofSha256": written_public_proof.public_proof_sha256,
        "tuiRows": written_public_proof.tui_rows,
        "privateStateExposed": written_public_proof.private_state_exposed
    });
    Ok(json!({
        "schemaVersion": "epiphany.repo_work_public_proof_export.v0",
        "status": "public-proof-exported",
        "workspace": workspace,
        "item": item,
        "outputPath": output,
        "outputSha256": public_proof_sha256,
        "sourceOverviewReceiptPath": overview["receiptPath"],
        "publicProofBundle": public_bundle,
        "verseProjection": verse_projection,
        "privateStateExposed": false,
        "nextSafeMove": "Share the public proof bundle with maintainers or Bifrost; keep raw worker thoughts and local receipt bodies sealed."
    }))
}

fn run_serve(args: ServeArgs) -> Result<Value> {
    if args.max_iterations == 0 && args.loop_interval_seconds == 0 {
        return Err(anyhow!(
            "unbounded repo-work serve mode requires --loop-interval-seconds greater than 0"
        ));
    }

    let workspace = args
        .tick
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.tick.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let artifact_dir = args
        .tick
        .artifact_dir
        .clone()
        .unwrap_or_else(|| workspace.join(".epiphany").join("work"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;
    let accept_receipt_path = resolve_accept_receipt(&workspace, args.tick.item.as_deref(), None)?;
    let accept_receipt = read_json(&accept_receipt_path)?;
    let item = accept_receipt
        .get("item")
        .and_then(Value::as_str)
        .unwrap_or("work-item")
        .to_string();
    let item_slug = sanitize(&item);
    let started_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let serve_mode = if args.max_iterations == 0 {
        "unbounded-service"
    } else {
        "bounded-proof"
    };
    let receipt_path = scheduler_serve_receipt_path(&artifact_dir, &item_slug);
    let start_receipt = json!({
        "schemaVersion": "epiphany.repo_work_scheduler_serve_receipt.v0",
        "createdAt": started_at,
        "startedAt": started_at,
        "completedAt": Value::Null,
        "status": "serve-running",
        "workspace": workspace,
        "item": item,
        "scheduler": {
            "owner": "Self",
            "schedulerId": args.scheduler_id,
            "serveMode": serve_mode,
            "pulseKind": "repo-work-local",
            "oneStepPerPulse": true,
            "loopIntervalSeconds": args.loop_interval_seconds,
            "maxIterations": args.max_iterations,
            "cooldownSeconds": args.tick.cooldown_seconds,
            "activeTimeoutSeconds": args.tick.active_timeout_seconds,
            "dryRun": args.tick.dry_run
        },
        "iterations": 0,
        "outputs": [],
        "lastOutput": Value::Null,
        "authority": {
            "branchLocalOnly": true,
            "publicationAuthorized": false,
            "mergeAuthorized": false,
            "serviceLifecycleAuthorized": false,
            "crossRepoMutationAuthorized": false,
            "privateStateExposed": false
        },
        "nextSafeMove": "Repo-work scheduler cadence is running; inspect per-pulse tick receipts for the durable trail."
    });
    write_json(&receipt_path, &start_receipt)?;

    let mut outputs = Vec::new();
    let mut iteration = 0_u64;
    let last_output = loop {
        iteration = iteration.saturating_add(1);
        let next_wake_utc = if args.max_iterations == 0 || iteration < args.max_iterations {
            Some(
                (Utc::now() + chrono::Duration::seconds(args.loop_interval_seconds as i64))
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            )
        } else {
            None
        };
        let tick_output = run_tick(args.tick.clone())?;
        let iteration_output = json!({
            "iteration": iteration,
            "nextWakeUtc": next_wake_utc,
            "tick": tick_output
        });
        if args.max_iterations != 0 {
            outputs.push(iteration_output.clone());
        }
        if args.max_iterations != 0 && iteration >= args.max_iterations {
            break iteration_output;
        }
        std::thread::sleep(std::time::Duration::from_secs(args.loop_interval_seconds));
    };

    let completed_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let receipt = json!({
        "schemaVersion": "epiphany.repo_work_scheduler_serve_receipt.v0",
        "createdAt": completed_at,
        "startedAt": started_at,
        "completedAt": completed_at,
        "status": "serve-complete",
        "workspace": workspace,
        "item": item,
        "scheduler": {
            "owner": "Self",
            "schedulerId": args.scheduler_id,
            "serveMode": serve_mode,
            "pulseKind": "repo-work-local",
            "oneStepPerPulse": true,
            "loopIntervalSeconds": args.loop_interval_seconds,
            "maxIterations": args.max_iterations,
            "cooldownSeconds": args.tick.cooldown_seconds,
            "activeTimeoutSeconds": args.tick.active_timeout_seconds,
            "dryRun": args.tick.dry_run
        },
        "iterations": iteration,
        "outputs": outputs,
        "lastOutput": last_output,
        "authority": {
            "branchLocalOnly": true,
            "publicationAuthorized": false,
            "mergeAuthorized": false,
            "serviceLifecycleAuthorized": false,
            "crossRepoMutationAuthorized": false,
            "privateStateExposed": false
        },
        "nextSafeMove": "Continue cadence only while repo-work receipts still identify a safe branch-local step; route Soul/Modeling/Mind closure after execution."
    });
    write_json(&receipt_path, &receipt)?;
    Ok(json!({
        "schemaVersion": "epiphany.repo_work_scheduler_serve.v0",
        "status": "serve-complete",
        "workspace": receipt["workspace"],
        "item": receipt["item"],
        "scheduler": receipt["scheduler"],
        "iterations": receipt["iterations"],
        "receiptPath": receipt_path,
        "lastOutput": receipt["lastOutput"],
        "authority": receipt["authority"],
        "privateStateExposed": false,
        "nextSafeMove": receipt["nextSafeMove"],
    }))
}

fn run_queue(args: QueueArgs) -> Result<Value> {
    if args.max_items == 0 {
        return Err(anyhow!("queue-run requires --max-items greater than 0"));
    }

    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let local_verse_store = args
        .local_verse_store
        .clone()
        .unwrap_or_else(|| workspace.join(".epiphany").join("local-verse.ccmp"));
    if !local_verse_store.exists() {
        return Err(anyhow!(
            "repo-work queue-run requires a local Verse store at {}; run epiphany-swarm online and epiphany-work overview first, or pass --local-verse-store",
            local_verse_store.display()
        ));
    }
    let artifact_dir = args
        .artifact_dir
        .clone()
        .unwrap_or_else(|| workspace.join(".epiphany").join("work"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;

    let (latest_repo_work_overview, repo_work_overviews) =
        load_repo_work_overview_queue_from_store(&local_verse_store, &args.runtime_id)?;
    let selected = repo_work_overviews
        .iter()
        .filter(|overview| overview_workspace_matches(overview, &workspace))
        .filter(|overview| repo_work_gate_is_tick_actionable(&overview.current_gate))
        .take(args.max_items as usize)
        .cloned()
        .collect::<Vec<_>>();
    let queue_rows = repo_work_queue_selection_rows(&repo_work_overviews, &workspace);
    let selected_rows = repo_work_queue_selection_rows(&selected, &workspace);

    let started_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let mut outputs = Vec::new();
    for overview in &selected {
        let tick_output = run_tick(TickArgs {
            workspace: workspace.clone(),
            epiphany_root: args.epiphany_root.clone(),
            item: Some(overview.item.clone()),
            local_verse_store: Some(local_verse_store.clone()),
            artifact_dir: Some(artifact_dir.clone()),
            runtime_store: args.runtime_store.clone(),
            cooldown_seconds: args.cooldown_seconds,
            active_timeout_seconds: args.active_timeout_seconds,
            dry_run: args.dry_run,
        })?;
        let refreshed_overview = if args.dry_run {
            Value::Null
        } else {
            run_overview(OverviewArgs {
                workspace: workspace.clone(),
                item: Some(overview.item.clone()),
                accept_receipt: None,
                artifact_dir: Some(artifact_dir.clone()),
                write_receipt: true,
            })?
        };
        outputs.push(json!({
            "item": overview.item,
            "overviewId": overview.overview_id,
            "gateBefore": overview.current_gate,
            "blockerBefore": overview.blocker,
            "tick": tick_output,
            "refreshedOverview": refreshed_overview
        }));
    }

    let status = if selected.is_empty() {
        "blocked-or-noop"
    } else if args.dry_run {
        "would-advance"
    } else {
        "advanced"
    };
    let next_safe_move = if selected.is_empty() {
        "No tick-actionable repo-work queue rows were found; inspect queue rows for plan, closure, publication, or sync gates."
    } else if args.dry_run {
        "Rerun queue-run without --dry-run to pulse the selected repo-work items."
    } else {
        "Inspect refreshed repo-work overview rows, then continue queue-run only while branch-local gates remain tick-actionable."
    };
    let completed_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let receipt_path = artifact_dir.join("work-queue-run.json");
    let receipt = json!({
        "schemaVersion": "epiphany.repo_work_queue_run_receipt.v0",
        "createdAt": completed_at,
        "startedAt": started_at,
        "completedAt": completed_at,
        "status": status,
        "workspace": workspace,
        "localVerseStore": local_verse_store,
        "runtimeId": args.runtime_id,
        "latestRepoWorkOverview": latest_repo_work_overview.as_ref().map(|overview| overview.overview_id.clone()),
        "queueCount": repo_work_overviews.len(),
        "actionableCount": selected.len(),
        "maxItems": args.max_items,
        "dryRun": args.dry_run,
        "queueRows": queue_rows,
        "selectedRows": selected_rows,
        "outputs": outputs,
        "authority": {
            "owner": "Self",
            "selectionOnly": false,
            "delegatesActuationTo": "epiphany.repo_work_scheduler_tick",
            "branchLocalOnly": true,
            "publicationAuthorized": false,
            "mergeAuthorized": false,
            "serviceLifecycleAuthorized": false,
            "crossRepoMutationAuthorized": false,
            "privateStateExposed": false
        },
        "nextSafeMove": next_safe_move,
        "privateStateExposed": false
    });
    write_json(&receipt_path, &receipt)?;
    Ok(json!({
        "schemaVersion": "epiphany.repo_work_queue_run.v0",
        "status": receipt["status"],
        "workspace": receipt["workspace"],
        "localVerseStore": receipt["localVerseStore"],
        "runtimeId": receipt["runtimeId"],
        "queueCount": receipt["queueCount"],
        "actionableCount": receipt["actionableCount"],
        "maxItems": receipt["maxItems"],
        "dryRun": receipt["dryRun"],
        "receiptPath": receipt_path,
        "queueRows": receipt["queueRows"],
        "selectedRows": receipt["selectedRows"],
        "outputs": receipt["outputs"],
        "authority": receipt["authority"],
        "privateStateExposed": false,
        "nextSafeMove": receipt["nextSafeMove"],
    }))
}

fn repo_work_modeling_job_id(item_slug: &str, generation: u64) -> String {
    format!("repo-work-modeling-{item_slug}-g{generation}")
}

fn resolve_repo_work_model_runtime(epiphany_root: &Path) -> Result<PathBuf> {
    if let Some(configured) = env::var_os("EPIPHANY_MODEL_RUNTIME_BIN") {
        let configured = PathBuf::from(configured);
        return configured.canonicalize().with_context(|| {
            format!(
                "failed to resolve EPIPHANY_MODEL_RUNTIME_BIN {}",
                configured.display()
            )
        });
    }
    let exe_name = if cfg!(windows) {
        "epiphany-openai-runtime.exe"
    } else {
        "epiphany-openai-runtime"
    };
    let mut candidates = Vec::new();
    if let Some(target) = env::var_os("CARGO_TARGET_DIR") {
        candidates.push(PathBuf::from(target).join("debug").join(exe_name));
    }
    candidates.push(
        epiphany_root
            .join("epiphany-openai-runtime")
            .join("target")
            .join("debug")
            .join(exe_name),
    );
    candidates.push(epiphany_root.join("target").join("debug").join(exe_name));
    candidates
        .into_iter()
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| anyhow!("epiphany-openai-runtime binary is not built"))
}

fn resolve_daemon_supervisor(epiphany_root: &Path) -> Result<PathBuf> {
    let exe_name = if cfg!(windows) {
        "epiphany-daemon-supervisor.exe"
    } else {
        "epiphany-daemon-supervisor"
    };
    let mut candidates = Vec::new();
    if let Some(target) = env::var_os("CARGO_TARGET_DIR") {
        candidates.push(PathBuf::from(target).join("debug").join(exe_name));
    }
    candidates.push(
        epiphany_root
            .join("epiphany-core")
            .join("target")
            .join("debug")
            .join(exe_name),
    );
    candidates.push(epiphany_root.join("target").join("debug").join(exe_name));
    candidates
        .into_iter()
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| anyhow!("epiphany-daemon-supervisor binary is not built"))
}

fn launch_repo_work_modeling_worker(
    epiphany_root: &Path,
    workspace: &Path,
    artifact_dir: &Path,
    runtime_store: &Path,
    local_verse_store: &Path,
    request: &RepoWorkModelingRequest,
    item_slug: &str,
    generation: u64,
) -> Result<Value> {
    let job_id = repo_work_modeling_job_id(item_slug, generation);
    let verified_diff = git_output(
        workspace,
        &[
            "show",
            "--format=fuller",
            "--stat",
            "--patch",
            &request.commit_sha,
        ],
    )?;
    let verified_diff = verified_diff.chars().take(24_000).collect::<String>();
    let document = EpiphanyWorkerLaunchDocument::RepoWorkModeling(
        EpiphanyRepoWorkModelingLaunchDocument {
            thread_id: format!("repo-work-{}", request.item),
            request_id: request.request_id.clone(),
            item: request.item.clone(),
            soul_verdict_receipt_id: request.soul_verdict_receipt_id.clone(),
            commit_sha: request.commit_sha.clone(),
            changed_paths: request.changed_paths.clone(),
            instruction: request.instruction.clone(),
            dynamic_prompt_context: Some(format!(
                "The request is immutable and already Soul-verified. Interpret commit {} across [{}]. Return a bounded finding only; Mind owns admission and Bifrost owns publication.\n\n<soul_verified_commit_diff>\n{}\n</soul_verified_commit_diff>",
                request.commit_sha,
                request.changed_paths.join(", "),
                verified_diff
            )),
        },
    );
    let runtime_bin = resolve_repo_work_model_runtime(epiphany_root)?;
    let supervisor_bin = resolve_daemon_supervisor(epiphany_root)?;
    let required_document_types = [
        epiphany_core::REPO_WORK_MODELING_ROUTE_TYPE,
        epiphany_core::REPO_WORK_MODELING_REQUEST_TYPE,
        epiphany_core::REPO_WORK_MODELING_FINDING_TYPE,
        epiphany_core::RUNTIME_WORKER_LAUNCH_REQUEST_TYPE,
    ];
    let mut preflight_command = Command::new(&runtime_bin);
    preflight_command
        .arg("preflight")
        .arg("--store")
        .arg(runtime_store);
    for document_type in required_document_types {
        preflight_command
            .arg("--require-document-type")
            .arg(document_type);
    }
    let preflight_output = preflight_command.output().with_context(|| {
        format!(
            "failed to preflight Modeling runtime {}",
            runtime_bin.display()
        )
    })?;
    if !preflight_output.status.success() {
        return Err(anyhow!(
            "Modeling runtime preflight refused launch before opening a job: {}",
            String::from_utf8_lossy(&preflight_output.stderr)
        ));
    }
    let preflight: Value = serde_json::from_slice(&preflight_output.stdout)
        .context("Modeling runtime preflight did not emit JSON")?;
    if preflight["schemaPreflightPassed"].as_bool() != Some(true) {
        return Err(anyhow!("Modeling runtime schema preflight did not pass"));
    }
    let executable_sha256 = preflight["executableSha256"]
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("Modeling runtime preflight omitted executable SHA-256"))?
        .to_string();
    let schema_catalog_sha256 = preflight["schemaCatalogSha256"]
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("Modeling runtime preflight omitted schema catalog SHA-256"))?
        .to_string();
    let preflight_witness_id = preflight["preflightWitnessId"]
        .as_str()
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("Modeling runtime preflight omitted witness identity"))?
        .to_string();
    let created_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    open_runtime_spine_heartbeat_job(
        runtime_store,
        RuntimeSpineHeartbeatJobOptions {
            runtime_id: "repo-work-local".to_string(),
            display_name: "Epiphany Repo Work".to_string(),
            session_id: format!("repo-work-session-{item_slug}"),
            objective: format!("Model Soul-verified repo work item {}.", request.item),
            coordinator_note: "Self routes the typed request; Modeling owns the result."
                .to_string(),
            job_id: job_id.clone(),
            role: "modeling".to_string(),
            binding_id: "repo-work-modeling-worker".to_string(),
            authority_scope: "epiphany.role.modeling.repo-work".to_string(),
            instruction: "Act as Modeling. Interpret only the typed repo-work request and return the required bounded result. Do not claim Mind, Hands, publication, merge, or service authority."
                .to_string(),
            launch_document: document,
            output_contract_id: REPO_WORK_MODELING_OUTPUT_CONTRACT_ID.to_string(),
            organ_launch_contract: default_launch_organ_contract(
                "epiphany.role.modeling.repo-work",
                "repo-work-modeling",
                REPO_WORK_MODELING_OUTPUT_CONTRACT_ID,
            ),
            created_at,
        },
    )?;
    let stdout_path = artifact_dir.join(format!("{job_id}.stdout.log"));
    let stderr_path = artifact_dir.join(format!("{job_id}.stderr.log"));
    let service_id = format!("idunn-{job_id}");
    let output = Command::new(&supervisor_bin)
        .arg("service-launch")
        .arg("--store")
        .arg(local_verse_store)
        .arg("--runtime-id")
        .arg("repo-work-local")
        .arg("--daemon-id")
        .arg("epiphany-daemon-modeling")
        .arg("--service-id")
        .arg(&service_id)
        .arg("--service-command")
        .arg(&runtime_bin)
        .arg("--service-arg")
        .arg("run-worker")
        .arg("--service-arg")
        .arg("--store")
        .arg("--service-arg")
        .arg(runtime_store)
        .arg("--service-arg")
        .arg("--job-id")
        .arg("--service-arg")
        .arg(&job_id)
        .arg("--service-arg")
        .arg("--cwd")
        .arg("--service-arg")
        .arg(workspace)
        .arg("--service-arg")
        .arg("--max-runtime-seconds")
        .arg("--service-arg")
        .arg("300")
        .arg("--cwd")
        .arg(workspace)
        .arg("--stdout-artifact")
        .arg(&stdout_path)
        .arg("--stderr-artifact")
        .arg(&stderr_path)
        .arg("--reason")
        .arg(format!(
            "Idunn launches typed repo-work Modeling job {job_id}."
        ))
        .output()
        .with_context(|| {
            format!(
                "failed to invoke Idunn supervisor {}",
                supervisor_bin.display()
            )
        })?;
    if !output.status.success() {
        return Err(anyhow!(
            "Idunn supervisor refused repo-work Modeling launch: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    let lifecycle: Value = serde_json::from_slice(&output.stdout)
        .context("Idunn supervisor did not emit its lifecycle receipt JSON")?;
    Ok(json!({
        "jobId": job_id,
        "processId": lifecycle["processId"],
        "runtimeBin": runtime_bin,
        "supervisorBin": supervisor_bin,
        "lifecycleOwner": "Idunn",
        "lifecycleReceiptId": lifecycle["receiptId"],
        "serviceId": lifecycle["serviceId"],
        "executableSha256": executable_sha256,
        "schemaCatalogSha256": schema_catalog_sha256,
        "preflightWitnessId": preflight_witness_id,
        "requiredDocumentTypes": required_document_types,
        "schemaPreflightPassed": true,
        "stdoutArtifact": stdout_path,
        "stderrArtifact": stderr_path,
        "requestId": request.request_id,
        "privateStateExposed": false
    }))
}

fn run_tick(args: TickArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let epiphany_root = args
        .epiphany_root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.epiphany_root.display()))?;
    let artifact_dir = args
        .artifact_dir
        .clone()
        .unwrap_or_else(|| workspace.join(".epiphany").join("work"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;

    let accept_receipt_path = resolve_accept_receipt(&workspace, args.item.as_deref(), None)?;
    let accept_receipt = read_json(&accept_receipt_path)?;
    let item = accept_receipt
        .get("item")
        .and_then(Value::as_str)
        .unwrap_or("work-item")
        .to_string();
    let item_slug = sanitize(&item);
    let local_verse_store = args.local_verse_store.clone().or_else(|| {
        path_from_json(&accept_receipt, &["localVerseStore"])
            .or_else(|| path_from_json(&accept_receipt, &["localVerseStorePath"]))
    });

    let plan_receipt_path = work_receipt_path(&workspace, "plan", &item);
    let run_receipt_path = work_receipt_path(&workspace, "run", &item);
    let adopt_receipt_path = work_receipt_path(&workspace, "adopt", &item);
    let execute_receipt_path = work_receipt_path(&workspace, "execute", &item);
    let close_receipt_path = work_receipt_path(&workspace, "close", &item);

    let before_receipts = repo_work_receipt_state(
        &accept_receipt_path,
        &plan_receipt_path,
        &run_receipt_path,
        &adopt_receipt_path,
        &execute_receipt_path,
        &close_receipt_path,
    );

    if let Some(brake_store) = local_verse_store.as_ref() {
        if brake_store.exists() {
            if let Some(brake) = load_epiphany_cultmesh_swarm_brake(brake_store, "epiphany-local")?
            {
                if brake.status == "engaged" {
                    let after_receipts = repo_work_receipt_state(
                        &accept_receipt_path,
                        &plan_receipt_path,
                        &run_receipt_path,
                        &adopt_receipt_path,
                        &execute_receipt_path,
                        &close_receipt_path,
                    );
                    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
                    let receipt = json!({
                        "schemaVersion": "epiphany.repo_work_scheduler_tick_receipt.v0",
                        "createdAt": now,
                        "workspace": workspace,
                        "item": item,
                        "localVerseStore": brake_store,
                        "scheduler": {
                            "owner": "Self",
                            "pulseKind": "repo-work-local",
                            "oneStepOnly": true,
                            "dryRun": args.dry_run
                        },
                        "status": "refused-by-swarm-brake",
                        "action": "none",
                        "reason": format!(
                            "local Verse swarm brake engaged; scope={}; protected={}; affected={}; reason={}",
                            brake.scope,
                            brake.protected_surfaces.join(","),
                            brake.affected_clusters.join(","),
                            brake.reason
                        ),
                        "brake": {
                            "brakeId": brake.brake_id,
                            "status": brake.status,
                            "scope": brake.scope,
                            "reason": brake.reason,
                            "affectedClusters": brake.affected_clusters,
                            "protectedSurfaces": brake.protected_surfaces,
                            "privateStateExposed": brake.private_state_exposed
                        },
                        "beforeReceipts": before_receipts,
                        "afterReceipts": after_receipts,
                        "advancedResult": Value::Null,
                        "authority": {
                            "branchLocalOnly": true,
                            "publicationAuthorized": false,
                            "mergeAuthorized": false,
                            "serviceLifecycleAuthorized": false,
                            "crossRepoMutationAuthorized": false,
                            "privateStateExposed": false,
                            "mutationBlockedBy": "epiphany.cultmesh.swarm_brake"
                        },
                        "nextSafeMove": "Release or narrow the local Verse swarm brake before repo-work scheduler mutation."
                    });
                    let receipt_path = tick_receipt_path(&artifact_dir, &item_slug);
                    write_json(&receipt_path, &receipt)?;
                    return Ok(json!({
                        "schemaVersion": "epiphany.repo_work_scheduler_tick.v0",
                        "status": receipt["status"],
                        "action": receipt["action"],
                        "workspace": receipt["workspace"],
                        "item": receipt["item"],
                        "receiptPath": receipt_path,
                        "reason": receipt["reason"],
                        "authority": receipt["authority"],
                        "privateStateExposed": false,
                        "nextSafeMove": receipt["nextSafeMove"],
                    }));
                }
            }
        }
    }

    let active_receipt_path = tick_active_receipt_path(&artifact_dir, &item_slug);
    let last_completed_receipt_path = tick_last_completed_receipt_path(&artifact_dir, &item_slug);
    let mut recovered_active_turn = Value::Null;

    if active_receipt_path.exists() {
        let active_receipt = read_json(&active_receipt_path)?;
        let active_started_at = parse_utc_rfc3339(&active_receipt, "startedAt")
            .or_else(|| parse_utc_rfc3339(&active_receipt, "createdAt"));
        let active_age_seconds = active_started_at.map(seconds_since);
        let active_is_stale = active_age_seconds
            .map(|age| age >= args.active_timeout_seconds as i64)
            .unwrap_or(false);

        if args.active_timeout_seconds == 0 || !active_is_stale {
            let after_receipts = repo_work_receipt_state(
                &accept_receipt_path,
                &plan_receipt_path,
                &run_receipt_path,
                &adopt_receipt_path,
                &execute_receipt_path,
                &close_receipt_path,
            );
            let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
            let receipt = json!({
                "schemaVersion": "epiphany.repo_work_scheduler_tick_receipt.v0",
                "createdAt": now,
                "workspace": workspace,
                "item": item,
                "localVerseStore": local_verse_store,
                "scheduler": {
                    "owner": "Self",
                    "pulseKind": "repo-work-local",
                    "oneStepOnly": true,
                    "dryRun": args.dry_run,
                    "cooldownSeconds": args.cooldown_seconds,
                    "activeTimeoutSeconds": args.active_timeout_seconds
                },
                "status": "refused-active-turn",
                "action": "none",
                "reason": "repo-work scheduler pulse already active for this work item",
                "physiology": {
                    "activeReceiptPath": active_receipt_path,
                    "lastCompletedReceiptPath": last_completed_receipt_path,
                    "activeAgeSeconds": active_age_seconds,
                    "activeTimeoutSeconds": args.active_timeout_seconds,
                    "privateStateExposed": false
                },
                "beforeReceipts": before_receipts,
                "afterReceipts": after_receipts,
                "advancedResult": Value::Null,
                "authority": {
                    "branchLocalOnly": true,
                    "publicationAuthorized": false,
                    "mergeAuthorized": false,
                    "serviceLifecycleAuthorized": false,
                    "crossRepoMutationAuthorized": false,
                    "privateStateExposed": false,
                    "mutationBlockedBy": "epiphany.repo_work_scheduler_active_turn"
                },
                "nextSafeMove": "Wait for the active scheduler pulse to finish, or let the active-turn timeout recover a stale marker."
            });
            let receipt_path = tick_receipt_path(&artifact_dir, &item_slug);
            write_json(&receipt_path, &receipt)?;
            return Ok(json!({
                "schemaVersion": "epiphany.repo_work_scheduler_tick.v0",
                "status": receipt["status"],
                "action": receipt["action"],
                "workspace": receipt["workspace"],
                "item": receipt["item"],
                "receiptPath": receipt_path,
                "reason": receipt["reason"],
                "authority": receipt["authority"],
                "privateStateExposed": false,
                "nextSafeMove": receipt["nextSafeMove"],
            }));
        }

        fs::remove_file(&active_receipt_path).with_context(|| {
            format!(
                "failed to clear stale active tick receipt {}",
                active_receipt_path.display()
            )
        })?;
        recovered_active_turn = json!({
            "activeReceiptPath": active_receipt_path,
            "activeAgeSeconds": active_age_seconds,
            "activeTimeoutSeconds": args.active_timeout_seconds,
            "recovered": true,
            "privateStateExposed": false
        });
    }

    if args.cooldown_seconds > 0 && last_completed_receipt_path.exists() {
        let last_completed_receipt = read_json(&last_completed_receipt_path)?;
        if let Some(last_created_at) = parse_utc_rfc3339(&last_completed_receipt, "createdAt") {
            let elapsed_seconds = seconds_since(last_created_at);
            if elapsed_seconds < args.cooldown_seconds as i64 {
                let after_receipts = repo_work_receipt_state(
                    &accept_receipt_path,
                    &plan_receipt_path,
                    &run_receipt_path,
                    &adopt_receipt_path,
                    &execute_receipt_path,
                    &close_receipt_path,
                );
                let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
                let receipt = json!({
                    "schemaVersion": "epiphany.repo_work_scheduler_tick_receipt.v0",
                    "createdAt": now,
                    "workspace": workspace,
                    "item": item,
                    "localVerseStore": local_verse_store,
                    "scheduler": {
                        "owner": "Self",
                        "pulseKind": "repo-work-local",
                        "oneStepOnly": true,
                        "dryRun": args.dry_run,
                        "cooldownSeconds": args.cooldown_seconds,
                        "activeTimeoutSeconds": args.active_timeout_seconds
                    },
                    "status": "refused-by-cooldown",
                    "action": "none",
                    "reason": format!(
                        "repo-work scheduler cooldown has not elapsed: {elapsed_seconds}s < {}s",
                        args.cooldown_seconds
                    ),
                    "physiology": {
                        "activeReceiptPath": active_receipt_path,
                        "lastCompletedReceiptPath": last_completed_receipt_path,
                        "elapsedSeconds": elapsed_seconds,
                        "cooldownSeconds": args.cooldown_seconds,
                        "privateStateExposed": false
                    },
                    "beforeReceipts": before_receipts,
                    "afterReceipts": after_receipts,
                    "advancedResult": Value::Null,
                    "authority": {
                        "branchLocalOnly": true,
                        "publicationAuthorized": false,
                        "mergeAuthorized": false,
                        "serviceLifecycleAuthorized": false,
                        "crossRepoMutationAuthorized": false,
                        "privateStateExposed": false,
                        "mutationBlockedBy": "epiphany.repo_work_scheduler_cooldown"
                    },
                    "nextSafeMove": "Wait for scheduler cooldown to elapse before pulsing this work item again."
                });
                let receipt_path = tick_receipt_path(&artifact_dir, &item_slug);
                write_json(&receipt_path, &receipt)?;
                return Ok(json!({
                    "schemaVersion": "epiphany.repo_work_scheduler_tick.v0",
                    "status": receipt["status"],
                    "action": receipt["action"],
                    "workspace": receipt["workspace"],
                    "item": receipt["item"],
                    "receiptPath": receipt_path,
                    "reason": receipt["reason"],
                    "authority": receipt["authority"],
                    "privateStateExposed": false,
                    "nextSafeMove": receipt["nextSafeMove"],
                }));
            }
        }
    }

    let active_started_at = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let active_marker = json!({
        "schemaVersion": "epiphany.repo_work_scheduler_tick_active.v0",
        "createdAt": active_started_at,
        "startedAt": active_started_at,
        "workspace": workspace,
        "item": item,
        "scheduler": {
            "owner": "Self",
            "pulseKind": "repo-work-local",
            "oneStepOnly": true,
            "dryRun": args.dry_run,
            "cooldownSeconds": args.cooldown_seconds,
            "activeTimeoutSeconds": args.active_timeout_seconds
        },
        "authority": {
            "branchLocalOnly": true,
            "publicationAuthorized": false,
            "mergeAuthorized": false,
            "serviceLifecycleAuthorized": false,
            "crossRepoMutationAuthorized": false,
            "privateStateExposed": false
        }
    });
    write_json(&active_receipt_path, &active_marker)?;

    let mut action = "none".to_string();
    let status;
    let reason;
    let next_safe_move;
    let mut advanced_result = Value::Null;

    if close_receipt_path.exists()
        && read_json(&close_receipt_path)?
            .get("status")
            .and_then(Value::as_str)
            == Some("closed")
    {
        action = "none".to_string();
        status = "noop".to_string();
        reason = "Soul/Modeling/Mind closure is recorded; scheduler stops before Bifrost publication authority".to_string();
        next_safe_move =
            "Submit a Bifrost publication intent through the owning integration.".to_string();
    } else if close_receipt_path.exists() {
        let partial_close = read_json(&close_receipt_path)?;
        let runtime_store = args
            .runtime_store
            .clone()
            .or_else(|| path_from_json(&partial_close, &["runtimeStore"]))
            .ok_or_else(|| anyhow!("awaiting-modeling closure has no runtime store"))?;
        let route_id = format!("repo-work-modeling-route-{item_slug}");
        let route = runtime_repo_work_modeling_route(&runtime_store, &route_id)?
            .ok_or_else(|| anyhow!("typed Modeling route {route_id:?} is missing"))?;
        let request = runtime_repo_work_modeling_request(&runtime_store, &route.request_id)?
            .ok_or_else(|| anyhow!("current Modeling request {:?} is missing", route.request_id))?;
        let finding_id = format!("{}-finding", request.request_id);
        if let Some(finding) = runtime_repo_work_modeling_finding(&runtime_store, &finding_id)? {
            if finding.verdict.trim().eq_ignore_ascii_case("passed") {
                action = "admit-modeling".to_string();
                if args.dry_run {
                    status = "would-advance".to_string();
                    reason = "a passing typed Modeling finding is ready for immutable Mind/map admission"
                        .to_string();
                    next_safe_move =
                        "Rerun without --dry-run to resume closure through Mind admission."
                            .to_string();
                } else {
                    advanced_result = run_close(CloseArgs {
                        workspace: workspace.clone(),
                        item: Some(item.clone()),
                        execute_receipt: Some(execute_receipt_path.clone()),
                        runtime_store: Some(runtime_store.clone()),
                        artifact_dir: Some(artifact_dir.clone()),
                    })?;
                    status = advanced_result["status"]
                        .as_str()
                        .unwrap_or("closed")
                        .to_string();
                    reason = "Mind admitted the matching model-authored finding and repo-work map"
                        .to_string();
                    next_safe_move = "Stop before Bifrost publication authority.".to_string();
                }
            } else {
                action = "await-modeling-revision".to_string();
                status = "blocked".to_string();
                reason = format!(
                    "Modeling returned verdict {}; immutable request requires reviewed revision before another attempt",
                    finding.verdict
                );
                next_safe_move = "Review the Modeling finding and create a new bounded work request; do not overwrite immutable evidence."
                    .to_string();
            }
        } else {
            let job_id = repo_work_modeling_job_id(&item_slug, route.generation);
            match runtime_job_snapshot(&runtime_store, &job_id)? {
                Some(snapshot)
                    if matches!(
                        snapshot.job.status,
                        epiphany_core::EpiphanyRuntimeJobStatus::Queued
                            | epiphany_core::EpiphanyRuntimeJobStatus::Running
                    ) =>
                {
                    action = "await-modeling".to_string();
                    status = "waiting".to_string();
                    reason = format!("Modeling worker job {job_id} is still active");
                    next_safe_move =
                        "Wait for the Modeling runtime result; do not launch a duplicate job."
                            .to_string();
                }
                Some(snapshot) => {
                    action = "await-modeling".to_string();
                    status = "blocked".to_string();
                    reason = format!(
                        "Modeling worker job {job_id} ended as {:?} without a typed finding",
                        snapshot.job.status
                    );
                    next_safe_move = "Inspect the sealed worker stdout/stderr and runtime result before a reviewed retry."
                        .to_string();
                }
                None if args.dry_run => {
                    action = "launch-modeling".to_string();
                    status = "would-advance".to_string();
                    reason = "typed Modeling request is ready for the existing worker runtime"
                        .to_string();
                    advanced_result = json!({
                        "schemaVersion": "epiphany.repo_work_modeling_launch_projection.v0",
                        "status": "launch-ready",
                        "routeId": route.route_id,
                        "generation": route.generation,
                        "requestId": request.request_id,
                        "jobId": job_id,
                        "preflightRequired": true,
                        "lifecycleOwner": "Idunn",
                        "privateStateExposed": false
                    });
                    next_safe_move =
                        "Rerun without --dry-run to launch the repo-work Modeling worker."
                            .to_string();
                }
                None => {
                    action = "launch-modeling".to_string();
                    let lifecycle_store = local_verse_store
                        .clone()
                        .unwrap_or_else(|| workspace.join(".epiphany").join("local-verse.ccmp"));
                    advanced_result = launch_repo_work_modeling_worker(
                        &epiphany_root,
                        &workspace,
                        &artifact_dir,
                        &runtime_store,
                        &lifecycle_store,
                        &request,
                        &item_slug,
                        route.generation,
                    )?;
                    status = "modeling-launched".to_string();
                    reason =
                        "Self routed the typed request to the existing model runtime".to_string();
                    next_safe_move =
                        "Let a later scheduler pulse consume the typed Modeling finding."
                            .to_string();
                }
            }
        }
    } else if execute_receipt_path.exists() {
        action = "soul-verify".to_string();
        if args.dry_run {
            status = "would-advance".to_string();
            reason = "executed work is ready for Soul verification".to_string();
            next_safe_move = "Rerun without --dry-run to verify the Hands consequence and emit the typed Modeling request."
                .to_string();
        } else {
            advanced_result = run_verify(CloseArgs {
                workspace: workspace.clone(),
                item: Some(item.clone()),
                execute_receipt: Some(execute_receipt_path.clone()),
                runtime_store: args.runtime_store.clone(),
                artifact_dir: Some(artifact_dir.clone()),
            })?;
            status = advanced_result
                .get("status")
                .and_then(Value::as_str)
                .unwrap_or("awaiting-modeling")
                .to_string();
            reason =
                "Soul verified the Hands consequence and Self routed the typed Modeling request"
                    .to_string();
            next_safe_move =
                "Let the next scheduler pulse await or collect the matching Modeling finding."
                    .to_string();
        }
    } else if adopt_receipt_path.exists() {
        if !plan_receipt_path.exists() {
            status = "blocked".to_string();
            reason = "adoption receipt exists but no matching plan receipt exists".to_string();
            next_safe_move = "Restore or pass a plan receipt before execution.".to_string();
        } else if args.dry_run {
            action = "execute-from-plan".to_string();
            status = "would-advance".to_string();
            reason = "approved adoption and plan receipt exist".to_string();
            next_safe_move =
                "Rerun without --dry-run to execute the adopted branch-local plan.".to_string();
        } else {
            action = "execute-from-plan".to_string();
            advanced_result = run_execute(ExecuteArgs {
                workspace: workspace.clone(),
                epiphany_root: epiphany_root.clone(),
                item: Some(item.clone()),
                adopt_receipt: Some(adopt_receipt_path.clone()),
                plan_receipt: Some(plan_receipt_path.clone()),
                runtime_store: args.runtime_store.clone(),
                artifact_dir: Some(artifact_dir.clone()),
                command: None,
                changed_paths: Vec::new(),
                commit_message: None,
                summary: Some(format!(
                    "Scheduler pulse executed adopted repo work item {item}."
                )),
            })?;
            status = "advanced".to_string();
            reason = "executed approved branch-local plan through Hands".to_string();
            next_safe_move =
                "Route Soul verification and Mind review before publication.".to_string();
        }
    } else if run_receipt_path.exists() {
        if !plan_receipt_path.exists() {
            status = "blocked".to_string();
            reason = "run receipt exists but no matching plan receipt exists".to_string();
            next_safe_move =
                "Write an Imagination/Self action plan before adopting Hands authority."
                    .to_string();
        } else if args.dry_run {
            action = "adopt-from-plan".to_string();
            status = "would-advance".to_string();
            reason = "queued run packet and plan receipt exist".to_string();
            next_safe_move =
                "Rerun without --dry-run to adopt the plan into branch-local Hands authority."
                    .to_string();
        } else {
            action = "adopt-from-plan".to_string();
            advanced_result = run_adopt(AdoptArgs {
                workspace: workspace.clone(),
                epiphany_root: epiphany_root.clone(),
                item: Some(item.clone()),
                run_receipt: Some(run_receipt_path.clone()),
                plan_receipt: Some(plan_receipt_path.clone()),
                runtime_store: args.runtime_store.clone(),
                artifact_dir: Some(artifact_dir.clone()),
                plan_summary: None,
                adoption_evidence_refs: vec![format!("self.scheduler:repo-work-tick-{item_slug}")],
                mind_adoption_rationale: Some(format!(
                    "Mind adopted the scheduler-presented plan for item {item} after Self found a queued run packet and matching Imagination plan receipt."
                )),
            })?;
            status = "advanced".to_string();
            reason = "adopted queued Hands run packet from typed action plan".to_string();
            next_safe_move =
                "Run another scheduler pulse to execute the approved branch-local plan."
                    .to_string();
        }
    } else if plan_receipt_path.exists() {
        let plan_receipt = read_json(&plan_receipt_path)?;
        let requested_paths = first_plan_action(&plan_receipt)
            .map(|action| string_array_field(action, "changedPaths"))
            .unwrap_or_default();
        if requested_paths.is_empty() {
            status = "blocked".to_string();
            reason = "plan receipt has no changedPaths for the Substrate Gate".to_string();
            next_safe_move =
                "Repair the plan receipt or write a new plan with changed paths.".to_string();
        } else if args.dry_run {
            action = "run-from-plan".to_string();
            status = "would-advance".to_string();
            reason = "accepted item and plan receipt exist".to_string();
            next_safe_move =
                "Rerun without --dry-run to open the queued Substrate Gate and Hands run packet."
                    .to_string();
        } else {
            action = "run-from-plan".to_string();
            advanced_result = run_work(RunArgs {
                workspace: workspace.clone(),
                epiphany_root: epiphany_root.clone(),
                item: Some(item.clone()),
                accept_receipt: Some(accept_receipt_path.clone()),
                runtime_store: args.runtime_store.clone(),
                artifact_dir: Some(artifact_dir.clone()),
                requested_paths,
            })?;
            status = "advanced".to_string();
            reason =
                "opened queued Substrate Gate and Hands run packet from plan paths".to_string();
            next_safe_move =
                "Run another scheduler pulse to adopt the plan into Hands authority.".to_string();
        }
    } else {
        status = "blocked".to_string();
        reason = "accepted work item has no matching action plan receipt".to_string();
        next_safe_move =
            "Create an Imagination/Self plan receipt before scheduler can advance work."
                .to_string();
    }

    let after_receipts = repo_work_receipt_state(
        &accept_receipt_path,
        &plan_receipt_path,
        &run_receipt_path,
        &adopt_receipt_path,
        &execute_receipt_path,
        &close_receipt_path,
    );
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let receipt = json!({
        "schemaVersion": "epiphany.repo_work_scheduler_tick_receipt.v0",
        "createdAt": now,
        "workspace": workspace,
        "item": item,
        "localVerseStore": local_verse_store,
        "scheduler": {
            "owner": "Self",
            "pulseKind": "repo-work-local",
            "oneStepOnly": true,
            "dryRun": args.dry_run,
            "cooldownSeconds": args.cooldown_seconds,
            "activeTimeoutSeconds": args.active_timeout_seconds
        },
        "status": status,
        "action": action,
        "reason": reason,
        "physiology": {
            "activeReceiptPath": active_receipt_path,
            "lastCompletedReceiptPath": last_completed_receipt_path,
            "recoveredActiveTurn": recovered_active_turn,
            "cooldownSeconds": args.cooldown_seconds,
            "activeTimeoutSeconds": args.active_timeout_seconds,
            "privateStateExposed": false
        },
        "beforeReceipts": before_receipts,
        "afterReceipts": after_receipts,
        "advancedResult": advanced_result,
        "authority": {
            "branchLocalOnly": true,
            "publicationAuthorized": false,
            "mergeAuthorized": false,
            "serviceLifecycleAuthorized": false,
            "crossRepoMutationAuthorized": false,
            "privateStateExposed": false
        },
        "nextSafeMove": next_safe_move
    });
    let receipt_path = tick_receipt_path(&artifact_dir, &item_slug);
    write_json(&receipt_path, &receipt)?;
    write_json(&last_completed_receipt_path, &receipt)?;
    if active_receipt_path.exists() {
        fs::remove_file(&active_receipt_path).with_context(|| {
            format!(
                "failed to clear active tick receipt {}",
                active_receipt_path.display()
            )
        })?;
    }
    Ok(json!({
        "schemaVersion": "epiphany.repo_work_scheduler_tick.v0",
        "status": receipt["status"],
        "action": receipt["action"],
        "workspace": receipt["workspace"],
        "item": receipt["item"],
        "receiptPath": receipt_path,
        "reason": receipt["reason"],
        "authority": receipt["authority"],
        "privateStateExposed": false,
        "nextSafeMove": receipt["nextSafeMove"],
    }))
}

fn resolve_accept_receipt(
    workspace: &Path,
    item: Option<&str>,
    explicit: Option<PathBuf>,
) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }
    let work_dir = workspace.join(".epiphany").join("work");
    if let Some(item) = item {
        return Ok(work_dir.join(format!("work-accept-{}.json", sanitize(item))));
    }
    let mut candidates = Vec::new();
    if work_dir.exists() {
        for entry in fs::read_dir(&work_dir)? {
            let entry = entry?;
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if name.starts_with("work-accept-") && name.ends_with(".json") {
                let modified = entry.metadata()?.modified()?;
                candidates.push((modified, path));
            }
        }
    }
    candidates.sort_by_key(|(modified, _path)| *modified);
    candidates
        .pop()
        .map(|(_modified, path)| path)
        .ok_or_else(|| anyhow!("no work accept receipt found; run epiphany-work accept first or pass --accept-receipt"))
}

fn resolve_run_receipt(
    workspace: &Path,
    item: Option<&str>,
    explicit: Option<PathBuf>,
) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }
    let work_dir = workspace.join(".epiphany").join("work");
    if let Some(item) = item {
        return Ok(work_dir.join(format!("work-run-{}.json", sanitize(item))));
    }
    latest_receipt_in(&work_dir, "work-run-").ok_or_else(|| {
        anyhow!("no work run receipt found; run epiphany-work run first or pass --run-receipt")
    })
}

fn resolve_adopt_receipt(
    workspace: &Path,
    item: Option<&str>,
    explicit: Option<PathBuf>,
) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }
    let work_dir = workspace.join(".epiphany").join("work");
    if let Some(item) = item {
        return Ok(work_dir.join(format!("work-adopt-{}.json", sanitize(item))));
    }
    latest_receipt_in(&work_dir, "work-adopt-").ok_or_else(|| {
        anyhow!(
            "no work adopt receipt found; run epiphany-work adopt first or pass --adopt-receipt"
        )
    })
}

fn resolve_execute_receipt(
    workspace: &Path,
    item: Option<&str>,
    explicit: Option<PathBuf>,
) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }
    let work_dir = workspace.join(".epiphany").join("work");
    if let Some(item) = item {
        return Ok(work_dir.join(format!("work-execute-{}.json", sanitize(item))));
    }
    latest_receipt_in(&work_dir, "work-execute-").ok_or_else(|| {
        anyhow!(
            "no work execute receipt found; run epiphany-work execute first or pass --execute-receipt"
        )
    })
}

fn work_receipt_path(workspace: &Path, kind: &str, item: &str) -> PathBuf {
    workspace
        .join(".epiphany")
        .join("work")
        .join(format!("work-{kind}-{}.json", sanitize(item)))
}

fn repo_work_receipt_state(
    accept: &Path,
    plan: &Path,
    run: &Path,
    adopt: &Path,
    execute: &Path,
    close: &Path,
) -> Value {
    json!({
        "accept": receipt_path_state(accept),
        "plan": receipt_path_state(plan),
        "run": receipt_path_state(run),
        "adopt": receipt_path_state(adopt),
        "execute": receipt_path_state(execute),
        "close": receipt_path_state(close),
    })
}

fn receipt_path_state(path: &Path) -> Value {
    json!({
        "path": path,
        "exists": path.exists(),
    })
}

fn existing_path_value(path: &Path) -> Value {
    if path.exists() {
        Value::String(path.display().to_string())
    } else {
        Value::Null
    }
}

fn readiness_path_row(
    kind: &str,
    owner: &str,
    required_schema: &str,
    path: &Path,
    satisfied: bool,
    note: &str,
) -> Result<Value> {
    let document = read_json_if_exists(path)?;
    let schema_version = document
        .as_ref()
        .and_then(|value| string_from_json(value, &["schemaVersion"]))
        .unwrap_or_else(|| "missing".to_string());
    let document_status = document
        .as_ref()
        .and_then(|value| string_from_json(value, &["status"]))
        .unwrap_or_else(|| "missing".to_string());
    Ok(json!({
        "kind": kind,
        "owner": owner,
        "requiredSchema": required_schema,
        "evidenceRef": existing_path_value(path),
        "artifactStatus": if path.exists() { "present" } else { "missing" },
        "schemaVersion": schema_version,
        "documentStatus": document_status,
        "satisfied": satisfied,
        "status": if satisfied { "satisfied" } else { "missing" },
        "note": note,
        "privateStateExposed": false
    }))
}

fn readiness_missing_row(kind: &str, owner: &str, required_schema: &str, note: &str) -> Value {
    json!({
        "kind": kind,
        "owner": owner,
        "requiredSchema": required_schema,
        "evidenceRef": Value::Null,
        "artifactStatus": "missing",
        "schemaVersion": "missing",
        "documentStatus": "missing",
        "satisfied": false,
        "status": "missing",
        "note": note,
        "privateStateExposed": false
    })
}

fn idunn_lifecycle_readiness_row(path: &Path) -> Result<Value> {
    let document = read_json_if_exists(path)?;
    let schema_version = document
        .as_ref()
        .and_then(|value| string_from_json(value, &["schemaVersion"]))
        .unwrap_or_else(|| "missing".to_string());
    let document_status = document
        .as_ref()
        .and_then(|value| string_from_json(value, &["status"]))
        .unwrap_or_else(|| "missing".to_string());
    let plan_status = document
        .as_ref()
        .and_then(|value| string_from_json(value, &["planStatus"]))
        .unwrap_or_else(|| "missing".to_string());
    let runbook_status = document
        .as_ref()
        .and_then(|value| string_from_json(value, &["runbookStatus"]))
        .unwrap_or_else(|| "missing".to_string());
    let runbook_artifact_status = document
        .as_ref()
        .and_then(|value| string_from_json(value, &["runbookArtifactStatus"]))
        .unwrap_or_else(|| "missing".to_string());
    let launch_status = document
        .as_ref()
        .and_then(|value| string_from_json(value, &["launchStatus"]))
        .unwrap_or_else(|| "missing".to_string());
    let lifecycle_owner = document
        .as_ref()
        .and_then(|value| string_from_json(value, &["lifecycleOwner"]))
        .unwrap_or_else(|| "missing".to_string());
    let hosted_body = document
        .as_ref()
        .and_then(|value| string_from_json(value, &["hostedBody"]))
        .unwrap_or_else(|| "missing".to_string());
    let private_state_exposed = document
        .as_ref()
        .and_then(|value| bool_from_json(value, &["privateStateExposed"]))
        .unwrap_or(true);
    let missing_count = document
        .as_ref()
        .and_then(|value| value.get("missingChecks"))
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(usize::MAX);
    let failed_count = document
        .as_ref()
        .and_then(|value| value.get("failedChecks"))
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(usize::MAX);
    let satisfied = schema_version == "epiphany.repo_work_service_audit.v0"
        && document_status == "complete"
        && plan_status == "present"
        && runbook_status == "present"
        && runbook_artifact_status == "present"
        && launch_status == "ok"
        && lifecycle_owner == "Idunn"
        && hosted_body == "repo-work"
        && document
            .as_ref()
            .and_then(|value| bool_from_json(value, &["mutatesServiceManager"]))
            == Some(false)
        && document
            .as_ref()
            .and_then(|value| bool_from_json(value, &["requiresElevatedAuthority"]))
            == Some(false)
        && !private_state_exposed
        && missing_count == 0
        && failed_count == 0;

    Ok(json!({
        "kind": "idunn-lifecycle",
        "owner": "Idunn",
        "requiredSchema": "epiphany.repo_work_service_audit.v0",
        "evidenceRef": existing_path_value(path),
        "artifactStatus": if path.exists() { "present" } else { "missing" },
        "schemaVersion": schema_version,
        "documentStatus": document_status,
        "planStatus": plan_status,
        "runbookStatus": runbook_status,
        "runbookArtifactStatus": runbook_artifact_status,
        "launchStatus": launch_status,
        "missingCheckCount": missing_count,
        "failedCheckCount": failed_count,
        "lifecycleOwner": lifecycle_owner,
        "hostedBody": hosted_body,
        "mutatesServiceManager": document
            .as_ref()
            .and_then(|value| bool_from_json(value, &["mutatesServiceManager"]))
            .unwrap_or(true),
        "requiresElevatedAuthority": document
            .as_ref()
            .and_then(|value| bool_from_json(value, &["requiresElevatedAuthority"]))
            .unwrap_or(true),
        "privateStateExposed": private_state_exposed,
        "satisfied": satisfied,
        "status": if satisfied { "satisfied" } else { "missing" },
        "note": "Idunn repo-work service audit proves the queue runner lifecycle path without service-manager mutation.",
        "serviceLifecycleAuthority": false,
        "deploymentAuthority": false,
        "handsActionAuthorized": false
    }))
}

fn tool_directory_readiness_row(path: &Path) -> Result<Value> {
    let document = read_json_if_exists(path)?;
    let schema_version = document
        .as_ref()
        .and_then(|value| string_from_json(value, &["schemaVersion"]))
        .unwrap_or_else(|| "missing".to_string());
    let document_status = document
        .as_ref()
        .and_then(|value| string_from_json(value, &["status"]))
        .unwrap_or_else(|| "missing".to_string());
    let tool_count = document
        .as_ref()
        .and_then(|value| value.get("toolCount"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let host_ready_count = document
        .as_ref()
        .and_then(|value| value.get("hostReadyCount"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let host_attention_count = document
        .as_ref()
        .and_then(|value| value.get("hostAttentionCount"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let invariant_all_agents = document
        .as_ref()
        .and_then(|value| bool_from_json(value, &["invariants", "availableToAllAgents"]))
        .unwrap_or(false);
    let invariant_receipts = document
        .as_ref()
        .and_then(|value| bool_from_json(value, &["invariants", "requiresReceipt"]))
        .unwrap_or(false);
    let invariant_private = document
        .as_ref()
        .and_then(|value| bool_from_json(value, &["invariants", "privateStateExposed"]))
        .unwrap_or(true);
    let has_invoke_command = document
        .as_ref()
        .and_then(|value| string_from_json(value, &["invocationCommand"]))
        .is_some_and(|command| command.contains("invoke-tool"));
    let has_wrapper_command = document
        .as_ref()
        .and_then(|value| string_from_json(value, &["wrapperInvocationCommand"]))
        .is_some_and(|command| command.contains("tool-invoke"));
    let satisfied = schema_version == "epiphany.cultmesh.daemon_tool_directory.v0"
        && document_status == "ok"
        && tool_count > 0
        && host_ready_count > 0
        && host_attention_count == 0
        && invariant_all_agents
        && invariant_receipts
        && !invariant_private
        && has_invoke_command
        && has_wrapper_command;

    Ok(json!({
        "kind": "tool-directory",
        "owner": "Odin",
        "requiredSchema": "epiphany.cultmesh.daemon_tool_directory.v0",
        "evidenceRef": existing_path_value(path),
        "artifactStatus": if path.exists() { "present" } else { "missing" },
        "schemaVersion": schema_version,
        "documentStatus": document_status,
        "toolCount": tool_count,
        "hostReadyCount": host_ready_count,
        "hostAttentionCount": host_attention_count,
        "availableToAllAgents": invariant_all_agents,
        "requiresReceipt": invariant_receipts,
        "hasInvocationCommand": has_invoke_command,
        "hasWrapperInvocationCommand": has_wrapper_command,
        "privateStateExposed": invariant_private,
        "satisfied": satisfied,
        "status": if satisfied { "satisfied" } else { "missing" },
        "note": "Odin tool-directory sight proves globally available daemon-hosted tools through typed intents and receipts.",
        "toolExecutionAuthority": false,
        "serviceLifecycleAuthority": false,
        "privateVerseRummagingAuthorized": false
    }))
}

fn read_json_if_exists(path: &Path) -> Result<Option<Value>> {
    if path.exists() {
        read_json(path).map(Some)
    } else {
        Ok(None)
    }
}

fn repo_work_existing_receipt_refs(receipts: &[(&str, &Path)]) -> Vec<String> {
    receipts
        .iter()
        .filter_map(|(kind, path)| {
            if path.exists() {
                Some(format!("{kind}:{}", path.display()))
            } else {
                None
            }
        })
        .collect()
}

fn repo_work_proof_artifact_rows(receipts: &[(&str, &Path)]) -> Result<Vec<Value>> {
    receipts
        .iter()
        .map(|(kind, path)| {
            if !path.exists() {
                return Ok(json!({
                    "kind": kind,
                    "expectedPath": path.display().to_string(),
                    "path": Value::Null,
                    "artifactStatus": "missing",
                    "artifactSha256": "none",
                    "schemaVersion": "missing",
                    "documentStatus": "missing",
                    "privateStateExposed": false
                }));
            }

            let document = read_json(path).with_context(|| {
                format!(
                    "failed to read proof artifact {} at {}",
                    kind,
                    path.display()
                )
            })?;
            let schema_version = string_from_json(&document, &["schemaVersion"])
                .unwrap_or_else(|| "unknown".to_string());
            let document_status =
                string_from_json(&document, &["status"]).unwrap_or_else(|| "unknown".to_string());
            Ok(json!({
                "kind": kind,
                "expectedPath": path.display().to_string(),
                "path": existing_path_value(path),
                "artifactStatus": "present",
                "artifactSha256": file_sha256(path)?,
                "schemaVersion": schema_version,
                "documentStatus": document_status,
                "privateStateExposed": false
            }))
        })
        .collect()
}

fn repo_work_public_proof_bundle(overview: &Value) -> Result<Value> {
    let proof = overview
        .get("proofBundle")
        .ok_or_else(|| anyhow!("overview did not include proofBundle"))?;
    let artifact_rows = proof
        .get("artifactRows")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .map(repo_work_public_artifact_row)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let publication_rows = proof
        .get("publicationRows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    Ok(json!({
        "schemaVersion": "epiphany.repo_work_public_proof_bundle.v0",
        "bundleId": proof["bundleId"],
        "generatedAt": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "sourceProofGeneratedAt": proof["generatedAt"],
        "item": proof["item"],
        "branch": proof["branch"],
        "currentGate": proof["currentGate"],
        "blocker": proof["blocker"],
        "nextSafeMove": proof["nextSafeMove"],
        "changedPaths": proof["changedPaths"],
        "commitSha": proof["commitSha"],
        "soulVerdict": proof["soulVerdict"],
        "mindStateCommitReceiptId": proof["mindStateCommitReceiptId"],
        "bifrostPublicationReceiptId": proof["bifrostPublicationReceiptId"],
        "githubPublicationReceiptId": proof["githubPublicationReceiptId"],
        "upstreamMainSynced": proof["upstreamMainSynced"],
        "artifactRows": artifact_rows,
        "publicationRows": publication_rows,
        "tuiRows": proof["tuiRows"],
        "redaction": {
            "rawReceiptBodiesIncluded": false,
            "localReceiptPathsIncluded": false,
            "rawWorkerThoughtIncluded": false,
            "privateStateExposed": false
        },
        "authority": {
            "owner": "Eyes",
            "sightOnly": true,
            "publicationAuthorized": false,
            "mergeAuthorized": false,
            "serviceLifecycleAuthorized": false,
            "crossRepoMutationAuthorized": false,
            "privateStateExposed": false
        },
        "privateStateExposed": false
    }))
}

fn repo_work_public_artifact_row(row: &Value) -> Value {
    json!({
        "kind": row.get("kind").cloned().unwrap_or(Value::Null),
        "artifactStatus": row.get("artifactStatus").cloned().unwrap_or(Value::Null),
        "artifactSha256": row.get("artifactSha256").cloned().unwrap_or(Value::Null),
        "schemaVersion": row.get("schemaVersion").cloned().unwrap_or(Value::Null),
        "documentStatus": row.get("documentStatus").cloned().unwrap_or(Value::Null),
        "privateStateExposed": false
    })
}

fn repo_work_public_proof_tui_rows(
    bundle: &Value,
    output: &Path,
    sha256: &str,
    artifact_row_count: usize,
    publication_row_count: usize,
) -> Vec<String> {
    vec![format!(
        "PUBLIC-PROOF | item={} | gate={} | branch={} | commit={} | artifacts={} | publicationRows={} | upstreamMainSynced={} | proof={} | sha256={} | private=false",
        string_from_json(bundle, &["item"]).unwrap_or_else(|| "unknown".to_string()),
        string_from_json(bundle, &["currentGate"]).unwrap_or_else(|| "unknown".to_string()),
        string_from_json(bundle, &["branch"]).unwrap_or_else(|| "unknown".to_string()),
        string_from_json(bundle, &["commitSha"]).unwrap_or_else(|| "none".to_string()),
        artifact_row_count,
        publication_row_count,
        bundle
            .get("upstreamMainSynced")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        output.display(),
        sha256
    )]
}

fn repo_work_proof_bundle_tui_rows(
    item: &str,
    branch: &str,
    gate: &str,
    blocker: &str,
    closure_status: &str,
    publication_status: &str,
    sync_status: &str,
    present_artifact_count: usize,
) -> Vec<String> {
    vec![
        format!("PROOF | item={item} | gate={gate} | blocker={blocker} | private=false"),
        format!(
            "PROOF | branch={branch} | artifactsPresent={present_artifact_count} | closure={closure_status} | publication={publication_status} | sync={sync_status}"
        ),
    ]
}

fn repo_work_overview_gate(
    plan: Option<&Value>,
    run: Option<&Value>,
    adopt: Option<&Value>,
    execute: Option<&Value>,
    close: Option<&Value>,
) -> (&'static str, &'static str, &'static str) {
    if close
        .and_then(|receipt| receipt.get("status"))
        .and_then(Value::as_str)
        == Some("closed")
    {
        (
            "awaiting-publication",
            "bifrost-publication-missing",
            "Submit a Bifrost publication intent through the owning integration.",
        )
    } else if close
        .and_then(|receipt| receipt.get("status"))
        .and_then(Value::as_str)
        == Some("awaiting-modeling")
    {
        (
            "awaiting-modeling",
            "modeling-finding-missing",
            "Route the typed Modeling request and resume closure with its matching finding.",
        )
    } else if close.is_some() {
        (
            "verification-refused",
            "soul-verdict-not-passed",
            "Repair the Hands consequence or verification evidence before retrying Soul.",
        )
    } else if execute.is_some() {
        (
            "awaiting-closure",
            "soul-modeling-mind-closure-missing",
            "Run epiphany-work close before publication.",
        )
    } else if adopt.is_some() {
        (
            "ready-to-execute",
            "none",
            "Run epiphany-work execute --from-plan or pulse epiphany-work tick.",
        )
    } else if run.is_some() {
        (
            "ready-to-adopt",
            "none",
            "Run epiphany-work adopt --from-plan or pulse epiphany-work tick.",
        )
    } else if plan.is_some() {
        (
            "ready-to-run",
            "none",
            "Run epiphany-work run or pulse epiphany-work tick.",
        )
    } else {
        (
            "awaiting-plan",
            "plan-receipt-missing",
            "Run epiphany-work derive-plan or epiphany-work plan.",
        )
    }
}

fn load_repo_work_overview_queue_from_store(
    store: &Path,
    runtime_id: &str,
) -> Result<(
    Option<EpiphanyCultMeshRepoWorkOverviewEntry>,
    Vec<EpiphanyCultMeshRepoWorkOverviewEntry>,
)> {
    let latest = load_latest_epiphany_cultmesh_repo_work_overview(store, runtime_id.to_string())?;
    let mut overviews = load_epiphany_cultmesh_repo_work_overviews(store, runtime_id.to_string())?;
    if let Some(latest_overview) = latest.as_ref() {
        if !overviews
            .iter()
            .any(|overview| overview.overview_id == latest_overview.overview_id)
        {
            overviews.push(latest_overview.clone());
            overviews.sort_by(|a, b| {
                b.generated_at
                    .cmp(&a.generated_at)
                    .then_with(|| a.overview_id.cmp(&b.overview_id))
            });
        }
    }
    Ok((latest, overviews))
}

fn repo_work_gate_is_tick_actionable(gate: &str) -> bool {
    matches!(
        gate,
        "ready-to-run" | "ready-to-adopt" | "ready-to-execute" | "awaiting-closure"
    )
}

fn overview_workspace_matches(
    overview: &EpiphanyCultMeshRepoWorkOverviewEntry,
    workspace: &Path,
) -> bool {
    let overview_path = PathBuf::from(&overview.workspace);
    overview_path
        .canonicalize()
        .map(|path| path == workspace)
        .unwrap_or_else(|_| {
            normalize_path_text(&overview.workspace)
                == normalize_path_text(&workspace.display().to_string())
        })
}

fn normalize_path_text(value: &str) -> String {
    value.replace('\\', "/").to_ascii_lowercase()
}

fn repo_work_queue_selection_rows(
    overviews: &[EpiphanyCultMeshRepoWorkOverviewEntry],
    workspace: &Path,
) -> Vec<Value> {
    overviews
        .iter()
        .map(|overview| {
            let workspace_match = overview_workspace_matches(overview, workspace);
            let tick_actionable = repo_work_gate_is_tick_actionable(&overview.current_gate);
            json!({
                "overviewId": overview.overview_id,
                "item": overview.item,
                "branch": overview.branch,
                "gate": overview.current_gate,
                "blocker": overview.blocker,
                "actionable": tick_actionable,
                "workspaceMatch": workspace_match,
                "nextSafeMove": overview.next_safe_move,
                "privateStateExposed": overview.private_state_exposed,
            })
        })
        .collect()
}

fn latest_receipt_in(work_dir: &Path, prefix: &str) -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if work_dir.exists() {
        for entry in fs::read_dir(work_dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            let name = path.file_name().and_then(|name| name.to_str())?;
            if name.starts_with(prefix) && name.ends_with(".json") {
                let modified = entry.metadata().ok()?.modified().ok()?;
                candidates.push((modified, path));
            }
        }
    }
    candidates.sort_by_key(|(modified, _path)| *modified);
    candidates.pop().map(|(_modified, path)| path)
}

fn ensure_hands_review_allows(
    intent: &HandsActionIntent,
    review: &epiphany_core::HandsActionReview,
    operation: &str,
) -> Result<()> {
    if review.intent_id != intent.intent_id {
        return Err(anyhow!(
            "Hands review {} belongs to {}, not {}",
            review.review_id,
            review.intent_id,
            intent.intent_id
        ));
    }
    if review.decision != "approved" {
        return Err(anyhow!(
            "Hands review {} decision is {}, not approved",
            review.review_id,
            review.decision
        ));
    }
    if !review
        .allowed_operations
        .iter()
        .any(|allowed| allowed == operation)
    {
        return Err(anyhow!(
            "Hands review {} does not allow {operation}",
            review.review_id
        ));
    }
    Ok(())
}

fn validate_paths_within_gate(intent: &HandsActionIntent, paths: &[String]) -> Result<()> {
    let requested = normalize_paths(intent.requested_paths.clone());
    for path in paths {
        let normalized = normalize_path(path);
        if requested.iter().any(|allowed| {
            allowed == "."
                || normalized == *allowed
                || normalized.starts_with(&format!("{}/", allowed.trim_end_matches('/')))
        }) {
            continue;
        }
        return Err(anyhow!(
            "changed path {normalized:?} is outside approved Hands path scope {:?}",
            requested
        ));
    }
    Ok(())
}

fn normalize_paths(paths: Vec<String>) -> Vec<String> {
    paths
        .into_iter()
        .map(|path| normalize_path(&path))
        .collect()
}

fn normalize_path(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        ".".to_string()
    } else {
        trimmed.trim_start_matches("./").to_string()
    }
}

fn validate_plan_target_path(path: &str) -> Result<String> {
    let normalized = normalize_path(path);
    if normalized == "." || normalized.is_empty() {
        return Err(anyhow!("derived plan target path cannot be empty"));
    }
    if normalized.starts_with('/')
        || normalized.contains(':')
        || normalized
            .split('/')
            .any(|part| part == ".." || part.is_empty())
    {
        return Err(anyhow!(
            "derived plan target path {normalized:?} must be a clean repo-relative path"
        ));
    }
    Ok(normalized)
}

fn validate_markdown_target_path(path: &str) -> Result<String> {
    let normalized = validate_plan_target_path(path)?;
    if !normalized
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .ends_with(".md")
    {
        return Err(anyhow!(
            "managed section target path {normalized:?} must be a markdown file"
        ));
    }
    Ok(normalized)
}

fn validate_toml_target_path(path: &str) -> Result<String> {
    let normalized = validate_plan_target_path(path)?;
    if !normalized
        .rsplit('/')
        .next()
        .unwrap_or_default()
        .ends_with(".toml")
    {
        return Err(anyhow!(
            "task card target path {normalized:?} must be a TOML file"
        ));
    }
    Ok(normalized)
}

fn normalize_action_family(value: &str) -> Result<String> {
    let normalized = value.trim().to_ascii_lowercase().replace('_', "-");
    if normalized.is_empty() {
        return Err(anyhow!("derive-plan action family cannot be empty"));
    }
    Ok(normalized)
}

fn compact_line(text: &str) -> String {
    let compact = text
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .trim()
        .to_string();
    let mut chars = compact.chars();
    let truncated = chars.by_ref().take(240).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        compact
    }
}

fn compact_multiline(text: &str) -> Vec<String> {
    text.lines()
        .map(compact_line)
        .filter(|line| !line.is_empty())
        .take(40)
        .collect()
}

fn compact_join(values: &[String]) -> String {
    if values.is_empty() {
        "none".to_string()
    } else {
        values
            .iter()
            .map(|value| compact_line(value))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn sorted_normalized_paths(paths: Vec<String>) -> Vec<String> {
    let mut normalized = normalize_paths(paths)
        .into_iter()
        .filter(|path| path != ".")
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn powershell_single_quoted(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn powershell_append_lines_command(target_path: &str, lines: &[String]) -> String {
    let mut commands = vec![
        format!("$p = {}", powershell_single_quoted(target_path)),
        "$d = Split-Path -Parent $p".to_string(),
        "if ($d) { New-Item -ItemType Directory -Force -Path $d | Out-Null }".to_string(),
    ];
    commands.extend(lines.iter().map(|line| {
        format!(
            "Add-Content -LiteralPath $p -Value {}",
            powershell_single_quoted(line)
        )
    }));
    commands.join("; ")
}

fn powershell_set_lines_command(target_path: &str, lines: &[String]) -> String {
    let mut commands = vec![
        format!("$p = {}", powershell_single_quoted(target_path)),
        "$d = Split-Path -Parent $p".to_string(),
        "if ($d) { New-Item -ItemType Directory -Force -Path $d | Out-Null }".to_string(),
        "$lines = @()".to_string(),
    ];
    commands.extend(
        lines
            .iter()
            .map(|line| format!("$lines += {}", powershell_single_quoted(line))),
    );
    commands.push("Set-Content -LiteralPath $p -Value $lines".to_string());
    commands.join("; ")
}

fn toml_basic_string(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
    )
}

fn toml_array(values: &[String]) -> String {
    let entries = values
        .iter()
        .map(|value| toml_basic_string(&compact_line(value)))
        .collect::<Vec<_>>()
        .join(", ");
    format!("[{entries}]")
}

fn powershell_replace_managed_section_command(
    target_path: &str,
    start_marker: &str,
    end_marker: &str,
    lines: &[String],
) -> String {
    let section = lines.join("\n");
    [
        format!("$p = {}", powershell_single_quoted(target_path)),
        "$d = Split-Path -Parent $p".to_string(),
        "if ($d) { New-Item -ItemType Directory -Force -Path $d | Out-Null }".to_string(),
        format!("$start = {}", powershell_single_quoted(start_marker)),
        format!("$end = {}", powershell_single_quoted(end_marker)),
        format!("$section = {}", powershell_single_quoted(&section)),
        "if (Test-Path -LiteralPath $p) { $content = Get-Content -LiteralPath $p -Raw } else { $content = '' }".to_string(),
        "$pattern = '(?s)' + [regex]::Escape($start) + '.*?' + [regex]::Escape($end)"
            .to_string(),
        "if ($content -match $pattern) { $content = [regex]::Replace($content, $pattern, $section.Replace('$', '$$')) } elseif ($content.Trim().Length -gt 0) { $content = $content.TrimEnd() + [Environment]::NewLine + [Environment]::NewLine + $section } else { $content = $section }".to_string(),
        "Set-Content -LiteralPath $p -Value $content".to_string(),
    ]
    .join("; ")
}

fn normalize_path_for_receipt(path: &Path) -> String {
    path.display().to_string().replace('\\', "/")
}

fn git_output(workspace: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !output.status.success() {
        return Err(anyhow!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn git_add_paths(workspace: &Path, paths: &[String]) -> Result<()> {
    let mut command = Command::new("git");
    command.arg("-C").arg(workspace).arg("add").arg("--");
    for path in paths {
        command.arg(path);
    }
    let output = command
        .output()
        .with_context(|| format!("failed to run git add in {}", workspace.display()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "git add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn ensure_staged_changes(workspace: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["diff", "--cached", "--quiet"])
        .output()
        .with_context(|| format!("failed to inspect staged diff in {}", workspace.display()))?;
    if output.status.success() {
        Err(anyhow!(
            "execute staged no changed paths; refusing empty commit"
        ))
    } else {
        Ok(())
    }
}

fn git_commit(workspace: &Path, message: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(["commit", "-m", message])
        .output()
        .with_context(|| format!("failed to run git commit in {}", workspace.display()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "git commit failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn cargo_json(manifest_path: &Path, bin_name: &str, args: &[String]) -> Result<Value> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--bin")
        .arg(bin_name)
        .arg("--")
        .args(args)
        .output()
        .with_context(|| format!("failed to spawn cargo run --bin {bin_name}"))?;
    if !output.status.success() {
        return Err(anyhow!(
            "cargo run --bin {bin_name} failed:\n{}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("{bin_name} returned invalid JSON"))
}

fn path_from_json(value: &Value, path: &[&str]) -> Option<PathBuf> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    cursor.as_str().map(PathBuf::from)
}

fn string_from_json(value: &Value, path: &[&str]) -> Option<String> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    cursor.as_str().map(ToString::to_string)
}

fn bool_from_json(value: &Value, path: &[&str]) -> Option<bool> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    cursor.as_bool()
}

fn string_array_from_json(value: &Value, path: &[&str]) -> Vec<String> {
    let mut cursor = value;
    for segment in path {
        let Some(next) = cursor.get(*segment) else {
            return Vec::new();
        };
        cursor = next;
    }
    cursor
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn first_plan_action(plan: &Value) -> Option<&Value> {
    plan.get("actions")?.as_array()?.first()
}

fn string_from_value(value: &Value, field: &str) -> Option<String> {
    value.get(field)?.as_str().map(ToString::to_string)
}

fn string_array_field(value: &Value, field: &str) -> Vec<String> {
    value
        .get(field)
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(ToString::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn default_if_empty(values: Vec<String>, defaults: Vec<String>) -> Vec<String> {
    if values.is_empty() { defaults } else { values }
}

#[cfg(test)]
mod authority_tests {
    use super::*;
    use epiphany_core::REPO_WORK_MODELING_FINDING_SCHEMA_VERSION;
    use epiphany_core::RepoWorkModelingFinding;
    use epiphany_core::put_repo_work_modeling_finding;

    fn deployment_config_fixture() -> String {
        r#"
schema_version = "epiphany.repo_deployment_config.v0"
safe_action_family = "repo.deployment_config"
summary = "test summary"
private_state_exposed = false

[deployment]
enabled = false
owner = "Idunn"
trigger = "git-push-observed-by-idunn"
watched_ref = "refs/heads/main"
deployment_script_ref = "deploy/idunn-deploy.ps1"
deployment_script_hash_required = true
deployment_script_review_required = true
host_access_policy_ref_required = true
secret_values_embedded = false
rollback_plan_ref_required = true
aftercare_checks_required = true
idunn_receipt_required = true
aftercare_audit_required = true

[cultmesh]
local_verse = "gamecult-local"
capability_family = "gamecult.idunn.deployment"
intent_contract = "gamecult.idunn.deployment_intent.v0"
receipt_contract = "gamecult.idunn.deployment_receipt.v0"
aftercare_contract = "gamecult.idunn.deployment_aftercare_audit.v0"
daemon_owns_execution = true

[required_receipts]
mind_adoption = "epiphany.repo_work_mind_adoption_decision.v0"
soul_review = "epiphany.repo_work_closure_review.v0"
maintainer_review = "gamecult.maintainer.review_receipt.v0"
secret_policy = "epiphany.repo_secret_policy_request.v0"
idunn_deployment = "gamecult.idunn.deployment_receipt.v0"
aftercare_audit = "gamecult.idunn.deployment_aftercare_audit.v0"

[authority]
configuration_only = true
direct_deployment_authority = false
direct_ssh_authority = false
direct_git_push_authority = false
direct_service_lifecycle_authority = false
direct_hands_authority = false
publication_authorized = false
merge_authorized = false
cross_body_mutation_authorized = false
private_verse_rummaging = false
idunn_deployment_authority_required = true
"#
        .to_string()
    }

    #[test]
    fn deployment_config_uses_typed_toml_not_comment_resemblance() {
        let valid = parse_repo_deployment_config(&deployment_config_fixture()).unwrap();
        assert_eq!(valid.deployment.watched_ref, "refs/heads/main");
        assert!(!valid.authority.direct_git_push_authority);

        let counterfeit = deployment_config_fixture().replace(
            "direct_git_push_authority = false",
            "# direct_git_push_authority = false\ndirect_git_push_authority = true",
        );
        let parsed = parse_repo_deployment_config(&counterfeit).unwrap();
        assert!(parsed.authority.direct_git_push_authority);
        assert!(!parsed.has_authority_seals());

        let missing_typed_field = deployment_config_fixture().replace(
            "watched_ref = \"refs/heads/main\"\n",
            "# watched_ref = \"refs/heads/main\"\n",
        );
        assert!(parse_repo_deployment_config(&missing_typed_field).is_err());
    }

    #[test]
    fn deployment_closure_uses_the_typed_config_model() {
        let source = include_str!("epiphany-work.rs");
        let start = source
            .find("\"repo.deployment_config\" => {")
            .expect("deployment closure branch");
        let end = source[start..]
            .find("\n        _ => {")
            .map(|offset| start + offset)
            .expect("end of deployment closure branch");
        let branch = &source[start..end];
        assert!(branch.contains("parse_repo_deployment_config"));
        assert!(branch.contains("has_authority_seals"));
        assert!(!branch.contains("content.contains"));
    }

    fn secret_policy_fixture() -> String {
        r#"
schema_version = "epiphany.repo_secret_policy_request.v0"
safe_action_family = "repo.secret_policy_request"
summary = "test summary"
private_state_exposed = false

[request]
status = "awaiting-security-review"
routing_owner = "Self"
required_reviewers = ["Maintainer", "Soul", "Mind", "Bifrost"]
policy_admission_owner = "Mind"
requested_effect = "review-repo-secret-and-write-permission-policy"
requires_secret_inventory_without_values = true
requires_write_permission_scope = true
requires_public_private_export_boundary = true

[antecedents]
source_grounding_required = true
soul_review_required = true
mind_adoption_required = true
maintainer_review_required = true
bifrost_publication_review_required = true

[required_receipts]
source_grounding = "epiphany.eyes.evidence_packet"
soul_review = "epiphany.repo_work_closure_review.v0"
mind_adoption = "epiphany.repo_work_mind_adoption_decision.v0"
maintainer_review = "gamecult.maintainer.review_receipt.v0"
bifrost_publication_review = "gamecult.bifrost.publication_review_receipt.v0"

[security_packet]
requires_secret_locations_without_values = true
requires_credential_owner = true
requires_write_scope_matrix = true
requires_public_export_redaction_rules = true
requires_deployment_authority_owner = true
requires_incident_rollback_plan = true

[authority]
direct_secret_access_authority = false
secret_value_materialization = false
write_permission_authority = false
deployment_authority = false
publication_authorized = false
merge_authorized = false
service_lifecycle_authority = false
cross_body_mutation_authorized = false
private_verse_rummaging = false
maintainer_security_review_required = true
soul_security_verification_required = true
mind_policy_admission_required = true
bifrost_publication_review_required = true
"#
        .to_string()
    }

    #[test]
    fn secret_policy_closure_refuses_comment_authority_seals() {
        let valid = parse_repo_secret_policy_request(&secret_policy_fixture()).unwrap();
        assert!(valid.has_authority_seals());

        let counterfeit = secret_policy_fixture().replace(
            "direct_secret_access_authority = false",
            "# direct_secret_access_authority = false\ndirect_secret_access_authority = true",
        );
        let parsed = parse_repo_secret_policy_request(&counterfeit).unwrap();
        assert!(parsed.authority.direct_secret_access_authority);
        assert!(!parsed.has_authority_seals());

        let source = include_str!("epiphany-work.rs");
        let start = source
            .find("\"repo.secret_policy_request\" => {")
            .expect("secret policy closure branch");
        let end = source[start..]
            .find("\n        \"repo.dependency_policy_request\" => {")
            .map(|offset| start + offset)
            .expect("end of secret policy closure branch");
        let branch = &source[start..end];
        assert!(branch.contains("parse_repo_secret_policy_request"));
        assert!(!branch.contains("content.contains"));
    }

    fn dependency_policy_fixture() -> String {
        r#"
schema_version = "epiphany.repo_dependency_policy_request.v0"
safe_action_family = "repo.dependency_policy_request"
summary = "test summary"
private_state_exposed = false
[request]
status = "awaiting-dependency-policy-review"
routing_owner = "Self"
required_reviewers = ["Maintainer", "Soul", "Mind", "Bifrost"]
policy_admission_owner = "Mind"
requested_effect = "review-repo-dependency-and-supply-chain-policy"
requires_manifest_inventory = true
requires_lockfile_policy = true
requires_package_manager_command_review = true
requires_network_fetch_policy = true
requires_vulnerability_review = true
requires_license_review = true
requires_rollback_plan = true
[antecedents]
source_grounding_required = true
eyes_evidence_required = true
soul_review_required = true
mind_adoption_required = true
maintainer_review_required = true
bifrost_publication_review_required = true
[required_receipts]
source_grounding = "epiphany.eyes.evidence_packet"
soul_review = "epiphany.repo_work_closure_review.v0"
mind_adoption = "epiphany.repo_work_mind_adoption_decision.v0"
maintainer_review = "gamecult.maintainer.review_receipt.v0"
bifrost_publication_review = "gamecult.bifrost.publication_review_receipt.v0"
dependency_audit = "gamecult.supply_chain.dependency_audit_receipt.v0"
[dependency_packet]
requires_manifest_paths = true
requires_lockfile_paths = true
requires_package_manager_commands = true
requires_vulnerability_sources = true
requires_license_constraints = true
requires_vendored_code_policy = true
requires_update_cadence = true
requires_private_state_redaction_check = true
[authority]
direct_dependency_update_authority = false
direct_package_install_authority = false
direct_lockfile_mutation_authority = false
direct_network_fetch_authority = false
direct_ci_mutation_authority = false
direct_hands_authority = false
publication_authorized = false
merge_authorized = false
deployment_authority = false
service_lifecycle_authority = false
cross_body_mutation_authorized = false
private_verse_rummaging = false
maintainer_dependency_review_required = true
soul_dependency_verification_required = true
mind_policy_admission_required = true
bifrost_publication_review_required = true
supply_chain_audit_required = true
"#
        .to_string()
    }

    #[test]
    fn dependency_policy_closure_refuses_comment_authority_seals() {
        let valid = parse_repo_dependency_policy_request(&dependency_policy_fixture()).unwrap();
        assert!(valid.has_authority_seals());
        let counterfeit = dependency_policy_fixture().replace(
            "direct_package_install_authority = false",
            "# direct_package_install_authority = false\ndirect_package_install_authority = true",
        );
        let parsed = parse_repo_dependency_policy_request(&counterfeit).unwrap();
        assert!(parsed.authority.direct_package_install_authority);
        assert!(!parsed.has_authority_seals());

        let source = include_str!("epiphany-work.rs");
        let start = source
            .find("\"repo.dependency_policy_request\" => {")
            .expect("dependency policy closure branch");
        let end = source[start..]
            .find("\n        \"repo.deployment_request\" => {")
            .map(|offset| start + offset)
            .expect("end of dependency policy closure branch");
        let branch = &source[start..end];
        assert!(branch.contains("parse_repo_dependency_policy_request"));
        assert!(!branch.contains("content.contains"));
    }

    fn deployment_request_fixture() -> String {
        r#"
schema_version = "epiphany.repo_deployment_request.v0"
safe_action_family = "repo.deployment_request"
summary = "test summary"
private_state_exposed = false
[request]
status = "awaiting-idunn-review"
routing_owner = "Self"
required_reviewers = ["Maintainer", "Soul", "Mind", "Bifrost"]
execution_owner = "Idunn"
requested_effect = "review-repo-deployment-trigger-and-script"
deployment_trigger = "git-push-observed-by-idunn"
requires_explicit_deployment_policy = true
requires_idunn_receipt = true
requires_aftercare_audit = true
[antecedents]
source_grounding_required = true
mind_adoption_required = true
soul_review_required = true
maintainer_review_required = true
secret_policy_review_required = true
bifrost_publication_review_required = true
[required_receipts]
source_grounding = "epiphany.eyes.evidence_packet"
mind_adoption = "epiphany.repo_work_mind_adoption_decision.v0"
soul_review = "epiphany.repo_work_closure_review.v0"
maintainer_review = "gamecult.maintainer.review_receipt.v0"
secret_policy = "epiphany.repo_secret_policy_request.v0"
bifrost_publication_review = "gamecult.bifrost.publication_review_receipt.v0"
idunn_deployment = "gamecult.idunn.deployment_receipt.v0"
aftercare_audit = "gamecult.idunn.deployment_aftercare_audit.v0"
[deployment_packet]
requires_target_environment = true
requires_git_ref_or_branch = true
requires_deployment_script_ref = true
requires_script_hash_or_review_ref = true
requires_host_access_policy_ref = true
requires_secret_policy_ref = true
requires_rollback_plan = true
requires_aftercare_checks = true
[authority]
direct_deployment_authority = false
direct_ssh_authority = false
direct_git_push_authority = false
direct_service_lifecycle_authority = false
direct_hands_authority = false
publication_authorized = false
merge_authorized = false
cross_body_mutation_authorized = false
private_verse_rummaging = false
idunn_deployment_authority_required = true
"#
        .to_string()
    }

    #[test]
    fn deployment_request_closure_refuses_comment_authority_seals() {
        let valid = parse_repo_deployment_request(&deployment_request_fixture()).unwrap();
        assert!(valid.has_authority_seals());
        let counterfeit = deployment_request_fixture().replace(
            "direct_ssh_authority = false",
            "# direct_ssh_authority = false\ndirect_ssh_authority = true",
        );
        let parsed = parse_repo_deployment_request(&counterfeit).unwrap();
        assert!(parsed.authority.direct_ssh_authority);
        assert!(!parsed.has_authority_seals());

        let source = include_str!("epiphany-work.rs");
        let start = source
            .find("\"repo.deployment_request\" => {")
            .expect("deployment request closure branch");
        let end = source[start..]
            .find("\n        \"repo.deployment_config\" => {")
            .map(|offset| start + offset)
            .expect("end of deployment request closure branch");
        let branch = &source[start..end];
        assert!(branch.contains("parse_repo_deployment_request"));
        assert!(!branch.contains("content.contains"));
    }

    #[test]
    fn queue_selection_rows_preserve_typed_scheduler_state() {
        let overview = EpiphanyCultMeshRepoWorkOverviewEntry {
            schema_version: epiphany_core::EPIPHANY_CULTMESH_REPO_WORK_OVERVIEW_SCHEMA_VERSION
                .to_string(),
            runtime_id: "test".to_string(),
            verse_id: "gamecult-local".to_string(),
            overview_id: "overview-1".to_string(),
            generated_at: "2026-07-13T00:00:00Z".to_string(),
            workspace: "E:/test".to_string(),
            item: "item-1".to_string(),
            branch: "main".to_string(),
            current_gate: "ready-to-run".to_string(),
            blocker: "none".to_string(),
            next_safe_move: "run it".to_string(),
            changed_paths: Vec::new(),
            commit_sha: "none".to_string(),
            soul_verdict: "pending".to_string(),
            publication_status: "not-requested".to_string(),
            sync_status: "local".to_string(),
            receipt_refs: Vec::new(),
            tui_rows: vec!["misleading display text".to_string()],
            proof_bundle_ref: "none".to_string(),
            private_state_exposed: false,
            notes: Vec::new(),
        };
        let rows = repo_work_queue_selection_rows(&[overview], Path::new("E:/test"));
        assert!(rows[0].is_object());
        assert_eq!(rows[0]["item"], "item-1");
        assert_eq!(rows[0]["branch"], "main");
        assert_eq!(rows[0]["gate"], "ready-to-run");
        assert_eq!(rows[0]["actionable"], true);
        assert_ne!(rows[0], "misleading display text");
    }

    #[test]
    fn close_rejects_caller_authored_verification_and_modeling_authority() -> Result<()> {
        for flag in [
            "--closure-model-ref",
            "--closure-model-verdict",
            "--closure-model-finding",
            "--model-authored",
            "--modeling-summary",
            "--verification-command",
            "--require-source-grounding",
            "--verification-summary",
            "--state-revision",
        ] {
            let error = parse_close_args([flag.to_string(), "counterfeit".to_string()].into_iter())
                .expect_err("legacy Modeling echo must be refused");
            assert!(error.to_string().contains("unexpected close argument"));
        }
        Ok(())
    }

    #[test]
    fn scheduler_cannot_impersonate_modeling_or_mind() {
        let source = include_str!("epiphany-work.rs");
        let tick_start = source.find("fn run_tick").expect("run_tick");
        let tick_end = source[tick_start..]
            .find("#[cfg(test)]")
            .map(|offset| tick_start + offset)
            .unwrap_or(source.len());
        let tick = &source[tick_start..tick_end];
        assert!(tick.contains("run_verify(CloseArgs"));
        assert!(tick.contains("run_close(CloseArgs"));
        assert!(tick.contains("await-modeling"));
        assert!(tick.contains("typed Modeling request"));
        assert!(!tick.contains("put_repo_work_modeling_finding"));
        assert!(!tick.contains("RepoWorkModelingFinding {"));
        assert!(!tick.contains("commit_repo_work_map_admission"));
        assert!(tick.find("runtime_repo_work_modeling_finding") < tick.find("run_close(CloseArgs"));
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("production source before authority tests");
        assert!(!production.contains("put_repo_work_modeling_finding("));
        assert!(!production.contains(".spawn()"));
        assert!(production.contains("epiphany-daemon-supervisor"));
        assert!(production.contains("\"lifecycleOwner\": \"Idunn\""));
        assert!(
            production.find("preflight_command.output()")
                < production.find("open_runtime_spine_heartbeat_job(")
        );
        assert!(production.contains("schemaPreflightPassed"));
        assert!(!source.contains(&["RepoWork", "MapStore"].concat()));
        assert!(!source.contains(&["repo-work-map", ".msgpack"].concat()));
    }

    #[test]
    fn typed_closure_families_have_no_substring_authority() {
        let source = include_str!("epiphany-work.rs");
        for family in [
            "repo.tool_request",
            "repo.publication_request",
            "repo.sync_request",
            "repo.pr_request",
            "repo.maintainer_review_request",
            "repo.collaboration_policy",
            "repo.collaboration_topic",
            "repo.consensus_brief",
            "repo.interpreter_brief",
            "repo.objective_draft",
            "repo.adoption_request",
            "repo.scheduling_request",
            "repo.work_order",
            "repo.verification_request",
            "repo.credit_request",
            "repo.artifact_acceptance_request",
            "repo.metrics_request",
            "repo.readiness_review_request",
            "repo.doctrine_update_request",
            "repo.secret_policy_request",
            "repo.dependency_policy_request",
            "repo.deployment_config",
            "repo.deployment_request",
        ] {
            let marker = format!("\"{family}\" => {{");
            let start = source
                .find(&marker)
                .unwrap_or_else(|| panic!("missing {family}"));
            let remainder = &source[start + marker.len()..];
            let end = remainder.find("\n        \"").unwrap_or(remainder.len());
            let branch = &remainder[..end];
            assert!(
                !branch.contains("content.contains"),
                "{family} closure regained substring authority"
            );
        }
    }

    #[test]
    fn closure_has_no_substring_authority() {
        let source = include_str!("epiphany-work.rs");
        let start = source
            .find("fn closure_family_assertions(")
            .expect("closure start");
        let end = source[start..]
            .find("fn push_assertion(")
            .map(|offset| start + offset)
            .expect("closure end");
        assert!(
            !source[start..end].contains("content.contains"),
            "closure regained presentation substring authority"
        );
    }

    #[test]
    fn collaboration_policy_generator_owns_its_proposal_fields() {
        let receipt = json!({"feedback": {"candidateActionRefs": [], "publicDiscussionRefs": []}});
        let plan = derive_safe_plan_family(DeriveSafePlanInput {
            action_family: "repo-collaboration-policy",
            target_path: None,
            item: "policy-owner-test",
            summary: "proposal",
            source: "test",
            accept_receipt: &receipt,
            model_ref: Some("test-model"),
            model_authored: true,
        })
        .expect("collaboration policy plan derives");
        assert!(plan.command.contains("[policy]"));
        assert!(plan.command.contains("authoring_owner"));
        assert!(plan.command.contains("Imagination"));
        assert!(plan.command.contains("policy_admitted = false"));
        assert!(plan.command.contains("publication_owner"));
        assert!(plan.command.contains("Bifrost"));
        assert!(
            derive_safe_plan_family(DeriveSafePlanInput {
                action_family: "repo-manifest",
                target_path: None,
                item: "deleted-body-diorama",
                summary: "deleted",
                source: "test",
                accept_receipt: &receipt,
                model_ref: None,
                model_authored: false,
            })
            .is_err()
        );
    }

    #[test]
    fn doctrine_comment_cannot_counterfeit_hands_authority_seal() {
        let text = r#"
schema_version = "epiphany.repo_doctrine_update_request.v0"
safe_action_family = "repo.doctrine_update_request"
summary = "summary"
private_state_exposed = false
[request]
status = "awaiting-doctrine-review"
routing_owner = "Self"
required_reviewers = ["Maintainer", "Mind", "Soul"]
doctrine_admission_owner = "Mind"
mutation_owner = "Hands"
requested_effect = "review-repo-agent-doctrine-update"
doctrine_target = "AGENTS.md"
requires_source_grounding = true
requires_human_or_maintainer_review = true
[antecedents]
[required_receipts]
[doctrine_packet]
[authority]
direct_doctrine_mutation_authority = false
# direct_hands_authority = false
direct_hands_authority = true
"#;
        let request = parse_repo_doctrine_update_request(text).expect("fixture is typed TOML");
        assert!(!request.has_authority_seals());
    }

    #[test]
    fn modeling_finding_round_trips_and_refuses_private_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime.cc");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "modeling-test".to_string(),
                display_name: "Modeling Test".to_string(),
                created_at: "2026-07-12T00:00:00Z".to_string(),
            },
        )?;
        put_soul_verdict_receipt(
            &store,
            &SoulVerdictReceipt {
                schema_version: SOUL_VERDICT_RECEIPT_SCHEMA_VERSION.to_string(),
                receipt_id: "soul-1".to_string(),
                source_result_id: "result-1".to_string(),
                source_job_id: "job-1".to_string(),
                verdict: "passed".to_string(),
                summary: "verified".to_string(),
                evidence_ids: Vec::new(),
                risks: Vec::new(),
                emitted_at: "2026-07-12T00:00:00Z".to_string(),
                contract: "test".to_string(),
            },
        )?;
        let request = RepoWorkModelingRequest {
            schema_version: REPO_WORK_MODELING_REQUEST_SCHEMA_VERSION.to_string(),
            request_id: "modeling-request-1".to_string(),
            item: "item-1".to_string(),
            requester: "self".to_string(),
            soul_verdict_receipt_id: "soul-1".to_string(),
            commit_sha: "abc123".to_string(),
            changed_paths: vec!["README.md".to_string()],
            instruction: "Model verified consequence.".to_string(),
            requested_at: "2026-07-12T00:00:01Z".to_string(),
            private_state_exposed: false,
            contract: "test".to_string(),
        };
        let route = RepoWorkModelingRoute {
            schema_version: REPO_WORK_MODELING_ROUTE_SCHEMA_VERSION.to_string(),
            route_id: "modeling-route-1".to_string(),
            item: request.item.clone(),
            generation: 0,
            request_id: request.request_id.clone(),
            previous_finding_receipt_id: String::new(),
            authority_owner: "soul".to_string(),
            authority_witness_id: request.soul_verdict_receipt_id.clone(),
            updated_at: "2026-07-12T00:00:01Z".to_string(),
            private_state_exposed: false,
            contract: "test".to_string(),
        };
        commit_initial_repo_work_modeling_route(&store, &request, &route)?;
        assert_eq!(
            runtime_repo_work_modeling_route(&store, &route.route_id)?,
            Some(route.clone())
        );
        let mut finding = RepoWorkModelingFinding {
            schema_version: REPO_WORK_MODELING_FINDING_SCHEMA_VERSION.to_string(),
            receipt_id: "modeling-finding-1".to_string(),
            item: "item-1".to_string(),
            model_ref: "model-job-1".to_string(),
            soul_verdict_receipt_id: "soul-1".to_string(),
            verdict: "passed".to_string(),
            finding: "Verified consequence updates the repo map.".to_string(),
            summary: "Map updated.".to_string(),
            changed_paths: vec!["README.md".to_string()],
            commit_sha: "abc123".to_string(),
            emitted_at: "2026-07-12T00:00:01Z".to_string(),
            private_state_exposed: true,
            contract: "test".to_string(),
            request_id: request.request_id.clone(),
        };
        let mut unsolicited_finding = finding.clone();
        unsolicited_finding.receipt_id = "unsolicited-modeling-finding".to_string();
        unsolicited_finding.request_id = "missing-modeling-request".to_string();
        unsolicited_finding.private_state_exposed = false;
        assert!(put_repo_work_modeling_finding(&store, &unsolicited_finding).is_err());
        assert!(put_repo_work_modeling_finding(&store, &finding).is_err());
        finding.private_state_exposed = false;
        put_repo_work_modeling_finding(&store, &finding)?;
        let mut conflicting_finding = finding.clone();
        conflicting_finding.summary = "different meaning".to_string();
        assert!(put_repo_work_modeling_finding(&store, &conflicting_finding).is_err());
        assert_eq!(
            runtime_repo_work_modeling_finding(&store, &finding.receipt_id)?,
            Some(finding.clone())
        );

        let review = MindGatewayReview {
            schema_version: MIND_GATEWAY_REVIEW_SCHEMA_VERSION.to_string(),
            gateway_id: "mind-review-1".to_string(),
            source_kind: "repo_work_closure".to_string(),
            source_role_id: "modeling".to_string(),
            decision: MindGatewayDecision::Accept,
            allowed_effects: vec!["repoWork.map".to_string()],
            refused_effects: Vec::new(),
            reasons: vec!["typed Modeling finding reread".to_string()],
            contract: "test".to_string(),
        };
        let commit = mind_state_commit_receipt(
            "mind-commit-1".to_string(),
            &review,
            1,
            vec!["repoWork.map".to_string()],
            "2026-07-12T00:00:02Z".to_string(),
        );
        let mut map = RepoWorkMapEntry {
            schema_version: REPO_WORK_MAP_ENTRY_SCHEMA_VERSION.to_string(),
            map_entry_id: "repo-work-map-item-1".to_string(),
            admitted_at: "2026-07-12T00:00:02Z".to_string(),
            item: finding.item.clone(),
            branch: "test".to_string(),
            changed_paths: finding.changed_paths.clone(),
            commit_sha: finding.commit_sha.clone(),
            safe_action_family: "repo.status_section".to_string(),
            modeling_summary: "counterfeit".to_string(),
            modeling_finding_receipt_id: finding.receipt_id.clone(),
            soul_verdict_receipt_id: finding.soul_verdict_receipt_id.clone(),
            mind_gateway_review_id: review.gateway_id.clone(),
            mind_state_commit_receipt_id: commit.receipt_id.clone(),
            execute_receipt_path: "execute.json".to_string(),
            closure_review_path: "review.json".to_string(),
            closure_receipt_path: "close.json".to_string(),
            publication_gate: "Bifrost".to_string(),
            durable_state_admitted: true,
            private_state_exposed: false,
            modeling_route_id: route.route_id.clone(),
            modeling_generation: route.generation,
        };
        assert!(commit_repo_work_map_admission(&store, &map, &review, &commit).is_err());
        assert!(epiphany_core::runtime_mind_gateway_review(&store, &review.gateway_id)?.is_none());
        assert!(
            epiphany_core::runtime_mind_state_commit_receipt(&store, &commit.receipt_id)?.is_none()
        );
        assert!(runtime_repo_work_map_entry(&store, &map.map_entry_id)?.is_none());

        map.modeling_summary = finding.summary.clone();
        commit_repo_work_map_admission(&store, &map, &review, &commit)?;
        let admitted = runtime_repo_work_map_entry(&store, &map.map_entry_id)?.unwrap();
        assert_eq!(admitted, map);
        let mut retry = map.clone();
        retry.admitted_at = "2026-07-12T00:10:00Z".to_string();
        assert_eq!(
            commit_repo_work_map_admission(&store, &retry, &review, &commit)?,
            admitted
        );
        retry.safe_action_family = "repo.task_card".to_string();
        assert!(commit_repo_work_map_admission(&store, &retry, &review, &commit).is_err());
        assert_eq!(
            runtime_repo_work_map_entry(&store, &map.map_entry_id)?,
            Some(map)
        );
        Ok(())
    }

    #[test]
    fn modeling_route_generation_requires_mind_and_preserves_previous() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime.cc");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "modeling-route-test".to_string(),
                display_name: "Modeling Route Test".to_string(),
                created_at: "2026-07-12T00:00:00Z".to_string(),
            },
        )?;
        let soul = SoulVerdictReceipt {
            schema_version: SOUL_VERDICT_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "route-soul-1".to_string(),
            source_result_id: "result-1".to_string(),
            source_job_id: "job-1".to_string(),
            verdict: "passed".to_string(),
            summary: "verified".to_string(),
            evidence_ids: Vec::new(),
            risks: Vec::new(),
            emitted_at: "2026-07-12T00:00:00Z".to_string(),
            contract: "test".to_string(),
        };
        put_soul_verdict_receipt(&store, &soul)?;
        let request0 = RepoWorkModelingRequest {
            schema_version: REPO_WORK_MODELING_REQUEST_SCHEMA_VERSION.to_string(),
            request_id: "route-request-g0".to_string(),
            item: "route-item".to_string(),
            requester: "self".to_string(),
            soul_verdict_receipt_id: soul.receipt_id.clone(),
            commit_sha: "abc123".to_string(),
            changed_paths: vec!["README.md".to_string()],
            instruction: "Model consequence.".to_string(),
            requested_at: "2026-07-12T00:00:01Z".to_string(),
            private_state_exposed: false,
            contract: "test".to_string(),
        };
        let route0 = RepoWorkModelingRoute {
            schema_version: REPO_WORK_MODELING_ROUTE_SCHEMA_VERSION.to_string(),
            route_id: "route-item-current".to_string(),
            item: request0.item.clone(),
            generation: 0,
            request_id: request0.request_id.clone(),
            previous_finding_receipt_id: String::new(),
            authority_owner: "soul".to_string(),
            authority_witness_id: soul.receipt_id.clone(),
            updated_at: "2026-07-12T00:00:01Z".to_string(),
            private_state_exposed: false,
            contract: "test".to_string(),
        };
        commit_initial_repo_work_modeling_route(&store, &request0, &route0)?;
        let finding0 = RepoWorkModelingFinding {
            schema_version: REPO_WORK_MODELING_FINDING_SCHEMA_VERSION.to_string(),
            receipt_id: "route-request-g0-finding".to_string(),
            item: request0.item.clone(),
            model_ref: "model-1".to_string(),
            soul_verdict_receipt_id: soul.receipt_id.clone(),
            verdict: "needs-work".to_string(),
            finding: "Need the verified diff.".to_string(),
            summary: "Context incomplete.".to_string(),
            changed_paths: request0.changed_paths.clone(),
            commit_sha: request0.commit_sha.clone(),
            emitted_at: "2026-07-12T00:00:02Z".to_string(),
            private_state_exposed: false,
            contract: "test".to_string(),
            request_id: request0.request_id.clone(),
        };
        put_repo_work_modeling_finding(&store, &finding0)?;
        let stale_map_review = MindGatewayReview {
            schema_version: MIND_GATEWAY_REVIEW_SCHEMA_VERSION.to_string(),
            gateway_id: "stale-map-review".to_string(),
            source_kind: "repo_work_closure".to_string(),
            source_role_id: "modeling".to_string(),
            decision: MindGatewayDecision::Accept,
            allowed_effects: vec!["repoWork.map".to_string()],
            refused_effects: Vec::new(),
            reasons: vec!["attempt stale admission".to_string()],
            contract: "test".to_string(),
        };
        let stale_map_commit = mind_state_commit_receipt(
            "stale-map-commit".to_string(),
            &stale_map_review,
            1,
            vec!["repoWork.map".to_string()],
            "2026-07-12T00:00:02Z".to_string(),
        );
        let stale_map = RepoWorkMapEntry {
            schema_version: REPO_WORK_MAP_ENTRY_SCHEMA_VERSION.to_string(),
            map_entry_id: "stale-map".to_string(),
            admitted_at: "2026-07-12T00:00:02Z".to_string(),
            item: request0.item.clone(),
            branch: "test".to_string(),
            changed_paths: request0.changed_paths.clone(),
            commit_sha: request0.commit_sha.clone(),
            safe_action_family: "repo.status_section".to_string(),
            modeling_summary: finding0.summary.clone(),
            modeling_finding_receipt_id: finding0.receipt_id.clone(),
            soul_verdict_receipt_id: soul.receipt_id.clone(),
            mind_gateway_review_id: stale_map_review.gateway_id.clone(),
            mind_state_commit_receipt_id: stale_map_commit.receipt_id.clone(),
            execute_receipt_path: "execute.json".to_string(),
            closure_review_path: "review.json".to_string(),
            closure_receipt_path: "close.json".to_string(),
            publication_gate: "Bifrost".to_string(),
            durable_state_admitted: true,
            private_state_exposed: false,
            modeling_route_id: route0.route_id.clone(),
            modeling_generation: 0,
        };
        assert!(
            commit_repo_work_map_admission(
                &store,
                &stale_map,
                &stale_map_review,
                &stale_map_commit,
            )
            .is_err()
        );
        let review = MindGatewayReview {
            schema_version: MIND_GATEWAY_REVIEW_SCHEMA_VERSION.to_string(),
            gateway_id: "route-g1-mind-review".to_string(),
            source_kind: "repo_work_modeling_revision".to_string(),
            source_role_id: "mind".to_string(),
            decision: MindGatewayDecision::Accept,
            allowed_effects: vec!["repoWork.modelingRoute".to_string()],
            refused_effects: vec!["repoWork.map".to_string()],
            reasons: vec!["Reviewed missing context.".to_string()],
            contract: "test".to_string(),
        };
        let request1 = RepoWorkModelingRequest {
            request_id: "route-request-g1".to_string(),
            requester: "mind".to_string(),
            instruction: "Re-model with reviewed context.".to_string(),
            requested_at: "2026-07-12T00:00:03Z".to_string(),
            ..request0.clone()
        };
        let route1 = RepoWorkModelingRoute {
            generation: 1,
            request_id: request1.request_id.clone(),
            previous_finding_receipt_id: finding0.receipt_id.clone(),
            authority_owner: "mind".to_string(),
            authority_witness_id: review.gateway_id.clone(),
            updated_at: "2026-07-12T00:00:03Z".to_string(),
            contract: "test generation one".to_string(),
            ..route0.clone()
        };
        advance_repo_work_modeling_route(&store, &request1, &route1, &review)?;
        assert_eq!(
            runtime_repo_work_modeling_route(&store, &route0.route_id)?,
            Some(route1.clone())
        );
        assert_eq!(
            runtime_repo_work_modeling_request(&store, &request0.request_id)?,
            Some(request0)
        );
        assert_eq!(
            runtime_repo_work_modeling_finding(&store, &finding0.receipt_id)?,
            Some(finding0)
        );
        assert!(advance_repo_work_modeling_route(&store, &request1, &route1, &review).is_err());
        Ok(())
    }

    #[test]
    fn public_revision_mouth_routes_scheduler_to_generation_one() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let workspace = temp.path().join("repo");
        fs::create_dir_all(&workspace)?;
        let git = Command::new("git")
            .arg("init")
            .arg("--quiet")
            .arg(&workspace)
            .status()?;
        assert!(git.success());
        for args in [
            [
                "-C",
                workspace.to_str().unwrap(),
                "config",
                "user.name",
                "Epiphany Smoke",
            ],
            [
                "-C",
                workspace.to_str().unwrap(),
                "config",
                "user.email",
                "smoke@epiphany.invalid",
            ],
        ] {
            assert!(Command::new("git").args(args).status()?.success());
        }
        fs::write(workspace.join("README.md"), "# Process retry smoke\n")?;
        assert!(
            Command::new("git")
                .args(["-C", workspace.to_str().unwrap(), "add", "README.md"])
                .status()?
                .success()
        );
        assert!(
            Command::new("git")
                .args([
                    "-C",
                    workspace.to_str().unwrap(),
                    "commit",
                    "--quiet",
                    "-m",
                    "Seed process retry smoke",
                ])
                .status()?
                .success()
        );
        let commit_sha = git_output(&workspace, &["rev-parse", "HEAD"])?
            .trim()
            .to_string();
        let artifact_dir = workspace.join(".epiphany").join("work");
        fs::create_dir_all(&artifact_dir)?;
        let store = workspace.join("runtime.msgpack");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "process-retry-smoke".to_string(),
                display_name: "Process Retry Smoke".to_string(),
                created_at: "2026-07-12T00:00:00Z".to_string(),
            },
        )?;
        let soul = SoulVerdictReceipt {
            schema_version: SOUL_VERDICT_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "process-retry-soul".to_string(),
            source_result_id: "process-retry-result".to_string(),
            source_job_id: "process-retry-job".to_string(),
            verdict: "passed".to_string(),
            summary: "Verified consequence.".to_string(),
            evidence_ids: vec![commit_sha.clone()],
            risks: Vec::new(),
            emitted_at: "2026-07-12T00:00:00Z".to_string(),
            contract: "test".to_string(),
        };
        put_soul_verdict_receipt(&store, &soul)?;
        let request0 = RepoWorkModelingRequest {
            schema_version: REPO_WORK_MODELING_REQUEST_SCHEMA_VERSION.to_string(),
            request_id: "repo-work-close-process-retry-modeling-request-g0".to_string(),
            item: "process-retry".to_string(),
            requester: "self".to_string(),
            soul_verdict_receipt_id: soul.receipt_id.clone(),
            commit_sha,
            changed_paths: vec!["README.md".to_string()],
            instruction: "Model generation zero.".to_string(),
            requested_at: "2026-07-12T00:00:01Z".to_string(),
            private_state_exposed: false,
            contract: "test".to_string(),
        };
        let route0 = RepoWorkModelingRoute {
            schema_version: REPO_WORK_MODELING_ROUTE_SCHEMA_VERSION.to_string(),
            route_id: "repo-work-modeling-route-process-retry".to_string(),
            item: request0.item.clone(),
            generation: 0,
            request_id: request0.request_id.clone(),
            previous_finding_receipt_id: String::new(),
            authority_owner: "soul".to_string(),
            authority_witness_id: soul.receipt_id.clone(),
            updated_at: "2026-07-12T00:00:01Z".to_string(),
            private_state_exposed: false,
            contract: "test".to_string(),
        };
        commit_initial_repo_work_modeling_route(&store, &request0, &route0)?;
        let finding0 = RepoWorkModelingFinding {
            schema_version: REPO_WORK_MODELING_FINDING_SCHEMA_VERSION.to_string(),
            receipt_id: format!("{}-finding", request0.request_id),
            item: request0.item.clone(),
            model_ref: "runtime-generation-zero".to_string(),
            soul_verdict_receipt_id: soul.receipt_id.clone(),
            verdict: "needs-work".to_string(),
            finding: "Reviewed context is required.".to_string(),
            summary: "Generation zero incomplete.".to_string(),
            changed_paths: request0.changed_paths.clone(),
            commit_sha: request0.commit_sha.clone(),
            emitted_at: "2026-07-12T00:00:02Z".to_string(),
            private_state_exposed: false,
            contract: "test".to_string(),
            request_id: request0.request_id.clone(),
        };
        put_repo_work_modeling_finding(&store, &finding0)?;
        write_json(
            &artifact_dir.join("work-accept-process-retry.json"),
            &json!({
                "schemaVersion": "epiphany.repo_work_accept.v0",
                "status": "accepted",
                "item": "process-retry",
                "privateStateExposed": false
            }),
        )?;
        write_json(
            &artifact_dir.join("work-close-process-retry.json"),
            &json!({
                "schemaVersion": "epiphany.repo_work_closure_receipt.v0",
                "status": "awaiting-modeling",
                "item": "process-retry",
                "runtimeStore": store,
                "privateStateExposed": false
            }),
        )?;

        let revised = run_revise_modeling(ReviseModelingArgs {
            workspace: workspace.clone(),
            item: "process-retry".to_string(),
            runtime_store: Some(store.clone()),
            rationale: "The operator reviewed the missing context.".to_string(),
            review_ref: "review://process-retry/g1".to_string(),
        })?;
        assert_eq!(revised["generation"], 1);
        assert_eq!(
            revised["requestId"],
            "repo-work-close-process-retry-modeling-request-g1"
        );

        let tick = run_tick(TickArgs {
            workspace: workspace.clone(),
            epiphany_root: env::current_dir()?,
            item: Some("process-retry".to_string()),
            local_verse_store: None,
            artifact_dir: Some(artifact_dir),
            runtime_store: Some(store.clone()),
            cooldown_seconds: 0,
            active_timeout_seconds: 900,
            dry_run: true,
        })?;
        let tick_receipt = read_json(Path::new(tick["receiptPath"].as_str().unwrap()))?;
        assert_eq!(tick_receipt["action"], "launch-modeling");
        assert_eq!(tick_receipt["advancedResult"]["generation"], 1);
        assert_eq!(
            tick_receipt["advancedResult"]["requestId"],
            "repo-work-close-process-retry-modeling-request-g1"
        );
        assert_eq!(
            tick_receipt["advancedResult"]["jobId"],
            "repo-work-modeling-process-retry-g1"
        );
        assert_eq!(tick_receipt["advancedResult"]["preflightRequired"], true);
        assert_eq!(tick_receipt["advancedResult"]["lifecycleOwner"], "Idunn");
        assert_eq!(
            runtime_repo_work_modeling_route(&store, &route0.route_id)?
                .expect("current route")
                .generation,
            1
        );
        if env::var_os("EPIPHANY_RUN_LIVE_MODELING_LAUNCH").is_some() {
            let live_tick = run_tick(TickArgs {
                workspace: workspace.clone(),
                epiphany_root: env::current_dir()?,
                item: Some("process-retry".to_string()),
                local_verse_store: Some(workspace.join(".epiphany").join("local-verse.ccmp")),
                artifact_dir: Some(workspace.join(".epiphany").join("work")),
                runtime_store: Some(store.clone()),
                cooldown_seconds: 0,
                active_timeout_seconds: 900,
                dry_run: false,
            })?;
            let live_receipt = read_json(Path::new(live_tick["receiptPath"].as_str().unwrap()))?;
            assert_eq!(live_receipt["status"], "modeling-launched");
            assert_eq!(live_receipt["action"], "launch-modeling");
            assert_eq!(
                live_receipt["advancedResult"]["requestId"],
                tick_receipt["advancedResult"]["requestId"]
            );
            assert_eq!(
                live_receipt["advancedResult"]["jobId"],
                tick_receipt["advancedResult"]["jobId"]
            );
            assert_eq!(live_receipt["advancedResult"]["lifecycleOwner"], "Idunn");
            assert_eq!(
                live_receipt["advancedResult"]["schemaPreflightPassed"],
                true
            );
            for field in [
                "lifecycleReceiptId",
                "serviceId",
                "executableSha256",
                "schemaCatalogSha256",
                "preflightWitnessId",
            ] {
                assert!(
                    live_receipt["advancedResult"][field]
                        .as_str()
                        .is_some_and(|value| !value.is_empty()),
                    "live lifecycle projection omitted {field}"
                );
            }
            assert_eq!(
                live_receipt["advancedResult"]["requiredDocumentTypes"]
                    .as_array()
                    .map(Vec::len),
                Some(4)
            );
            let preserved = temp.keep();
            eprintln!(
                "preserved live generation-one fixture at {}",
                preserved.display()
            );
        }
        Ok(())
    }
}

fn adopted_action_item_from_plan(plan: &Value) -> Value {
    let action_item = plan
        .get("derivation")
        .and_then(|value| value.get("actionItemReceipt"));
    let plan_action = plan
        .get("actions")
        .and_then(Value::as_array)
        .and_then(|actions| actions.first());
    json!({
        "source": "plan.derivation.actionItemReceipt",
        "planId": plan.get("planId").cloned().unwrap_or(Value::Null),
        "receiptId": action_item
            .and_then(|value| value.get("receiptId"))
            .cloned()
            .unwrap_or(Value::Null),
        "receiptPath": action_item
            .and_then(|value| value.get("receiptPath"))
            .cloned()
            .unwrap_or(Value::Null),
        "schemaVersion": action_item
            .and_then(|value| value.get("schemaVersion"))
            .cloned()
            .unwrap_or(Value::Null),
        "status": action_item
            .and_then(|value| value.get("status"))
            .cloned()
            .unwrap_or(Value::Null),
        "modelAuthored": action_item
            .and_then(|value| value.get("modelAuthored"))
            .cloned()
            .unwrap_or(json!(false)),
        "safeActionFamily": action_item
            .and_then(|value| value.get("safeActionFamily"))
            .cloned()
            .unwrap_or(Value::Null),
        "requestedPaths": action_item
            .and_then(|value| value.get("requestedPaths"))
            .or_else(|| plan_action.and_then(|value| value.get("changedPaths")))
            .cloned()
            .unwrap_or(json!([])),
        "verificationAsks": action_item
            .and_then(|value| value.get("verificationAsks"))
            .or_else(|| plan_action.and_then(|value| value.get("verificationAsks")))
            .cloned()
            .unwrap_or(json!([])),
        "summary": plan.get("planSummary").cloned().unwrap_or(Value::Null),
        "planningFacets": action_item
            .and_then(|value| value.get("planningFacets"))
            .cloned()
            .unwrap_or(Value::Null),
        "adoptionEvidenceRefs": plan
            .get("adoptionEvidenceRefs")
            .cloned()
            .unwrap_or(json!([])),
        "handsCommandAuthority": false,
        "durableStateAuthority": false,
        "publicationAuthorized": false,
        "mergeAuthorized": false,
        "serviceLifecycleAuthority": false,
        "crossRepoMutation": false,
        "privateStateExposed": false
    })
}

fn repo_work_safe_family_is_recognized(safe_family: &str) -> bool {
    matches!(
        safe_family,
        "repo.append_worklog"
            | "repo.markdown_planning_note"
            | "repo.checklist_note"
            | "repo.markdown_managed_section"
            | "repo.status_section"
            | "repo.task_card"
            | "repo.tool_request"
            | "repo.collaboration_policy"
            | "repo.collaboration_topic"
            | "repo.consensus_brief"
            | "repo.interpreter_brief"
            | "repo.objective_draft"
            | "repo.adoption_request"
            | "repo.scheduling_request"
            | "repo.work_order"
            | "repo.verification_request"
            | "repo.publication_request"
            | "repo.sync_request"
            | "repo.maintainer_review_request"
            | "repo.pr_request"
            | "repo.credit_request"
            | "repo.artifact_acceptance_request"
            | "repo.metrics_request"
            | "repo.readiness_review_request"
            | "repo.doctrine_update_request"
            | "repo.secret_policy_request"
            | "repo.dependency_policy_request"
            | "repo.deployment_config"
            | "repo.deployment_request"
    )
}

fn compact_text(text: &str, limit: usize) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if compact.chars().count() <= limit {
        return compact;
    }
    let mut truncated = compact
        .chars()
        .take(limit.saturating_sub(3))
        .collect::<String>();
    truncated.push_str("...");
    truncated
}

fn read_json(path: &Path) -> Result<Value> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("failed to decode {}", path.display()))
}

fn file_sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path).with_context(|| format!("failed to hash {}", path.display()))?;
    let digest = Sha256::digest(&bytes);
    Ok(format!("{digest:x}"))
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn tick_receipt_path(artifact_dir: &Path, item_slug: &str) -> PathBuf {
    artifact_dir.join(format!("work-tick-{item_slug}.json"))
}

fn tick_active_receipt_path(artifact_dir: &Path, item_slug: &str) -> PathBuf {
    artifact_dir.join(format!("work-tick-active-{item_slug}.json"))
}

fn tick_last_completed_receipt_path(artifact_dir: &Path, item_slug: &str) -> PathBuf {
    artifact_dir.join(format!("work-tick-last-{item_slug}.json"))
}

fn scheduler_serve_receipt_path(artifact_dir: &Path, item_slug: &str) -> PathBuf {
    artifact_dir.join(format!("work-scheduler-serve-{item_slug}.json"))
}

fn parse_utc_rfc3339(value: &Value, field: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value.get(field)?.as_str()?)
        .ok()
        .map(|timestamp| timestamp.with_timezone(&Utc))
}

fn seconds_since(timestamp: DateTime<Utc>) -> i64 {
    Utc::now().signed_duration_since(timestamp).num_seconds()
}

fn ensure_git_repo(workspace: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .with_context(|| format!("failed to run git in {}", workspace.display()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!("{} is not a git repository", workspace.display()))
    }
}

fn take_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(take_string(args, name)?))
}

fn take_string(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("missing value for {name}"))
}

fn take_u64(args: &mut impl Iterator<Item = String>, name: &str) -> Result<u64> {
    take_string(args, name)?
        .parse::<u64>()
        .with_context(|| format!("invalid integer for {name}"))
}

fn sanitize(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if sanitized.is_empty() {
        "work-item".to_string()
    } else {
        sanitized
    }
}

fn print_usage() {
    eprintln!(
        "usage: epiphany-work <persona-intake|accept|derive-plan|plan|run|adopt|execute|close|overview|readiness|deployment-config-audit|deployment-execution-runbook|deployment-aftercare-audit|export-proof|tick|queue-run|serve> ...\n\
         persona-intake --workspace <repo> --item <id> --message <text> [--topic <topic>] [--store <local-verse.ccmp>] [--runtime-id <id>]\n\
         accept --workspace <repo> --from <persona|bifrost|persona-or-bifrost> --item <id> [--summary <text>] [--topic <topic>] [--store <local-verse.ccmp>] [--runtime-id <id>] [--online-receipt <path>] [--public-discussion-ref <ref>] [--candidate-action-ref <ref>]\n\
         derive-plan --workspace <repo> [--item <id>] [--accept-receipt <path>] [--action-family append-worklog|planning-note|checklist-note|section-note|repo-status-section|task-card|repo-tool-request|repo-collaboration-policy|repo-collaboration-topic|repo-consensus-brief|repo-interpreter-brief|repo-objective-draft|repo-adoption-request|repo-scheduling-request|repo-work-order|repo-verification-request|repo-publication-request|repo-sync-request|repo-maintainer-review-request|repo-pr-request|repo-credit-request|repo-artifact-acceptance-request|repo-metrics-request|repo-readiness-review-request|repo-doctrine-update-request|repo-secret-policy-request|repo-dependency-policy-request|repo-deployment-config|repo-deployment-request] [--target-path <path>] [--model-ref <ref>] [--model-authored] [--action-summary <text>] [--verification-ask <text>] [--stop-condition <text>] [--escalation-reason <text>] [--assumption <text>] [--constraint <text>] [--non-goal <text>] [--open-question <text>] [--decision-point <text>] [--evidence-need <text>]\n\
         plan --workspace <repo> [--item <id>] --objective <text> --plan-summary <text> --command <command> --changed-path <path> --commit-message <text> [--adoption-evidence-ref <ref>]\n\
         run --workspace <repo> [--item <id>] [--accept-receipt <path>] [--runtime-store <path>] [--requested-path <path>]\n\
         adopt --workspace <repo> [--item <id>] [--run-receipt <path>] [--from-plan <path>] [--plan-summary <text>] [--adoption-evidence-ref <ref>] [--mind-adoption-rationale <text>]\n\
         execute --workspace <repo> [--item <id>] [--from-plan <path>] [--command <command>] [--changed-path <path>] [--commit-message <text>]\n\
         close --workspace <repo> [--item <id>] [--execute-receipt <path>]\n\
         overview --workspace <repo> [--item <id>] [--accept-receipt <path>] [--no-write]\n\
         readiness --workspace <repo> [--item <id>] [--accept-receipt <path>] [--public-proof <path>] [--idunn-lifecycle-receipt <path>] [--deployment-aftercare-audit-receipt <path>|--deployment-aftercare-audit-receipt-ref <ref>] [--tool-directory-receipt <path>] [--no-write]\n\
         deployment-config-audit --workspace <repo> [--artifact-dir <path>] [--no-write]\n\
         deployment-execution-runbook --workspace <repo> [--artifact-dir <path>] [--remote origin] [--no-write]\n\
         deployment-aftercare-audit --workspace <repo> [--artifact-dir <path>] [--local-verse-store <path>] [--runtime-id <id>] [--runbook-receipt <path>] [--idunn-deployment-receipt-ref <ref>|--idunn-deployment-receipt <path>] [--aftercare-audit-receipt-ref <ref>|--aftercare-audit-receipt <path>] [--no-write]\n\
         export-proof --workspace <repo> [--item <id>] [--accept-receipt <path>] [--output <path>] [--local-verse-store <path>] [--runtime-id repo-swarm-local]\n\
         tick --workspace <repo> [--item <id>] [--local-verse-store <path>] [--runtime-store <path>] [--dry-run]\n\
         queue-run --workspace <repo> [--local-verse-store <path>] [--runtime-id repo-swarm-local] [--max-items 1] [--dry-run]"
    );
}
