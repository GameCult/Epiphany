use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::DateTime;
use chrono::Utc;
use epiphany_core::EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_REPO_WORK_OVERVIEW_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_REPO_WORK_PUBLIC_PROOF_SCHEMA_VERSION;
use epiphany_core::EpiphanyCultMeshRepoWorkOverviewEntry;
use epiphany_core::EpiphanyCultMeshRepoWorkPublicProofEntry;
use epiphany_core::HANDS_ACTION_INTENT_SCHEMA_VERSION;
use epiphany_core::HANDS_COMMAND_RECEIPT_TYPE;
use epiphany_core::HANDS_COMMIT_RECEIPT_TYPE;
use epiphany_core::HANDS_PATCH_RECEIPT_TYPE;
use epiphany_core::HANDS_PR_RECEIPT_TYPE;
use epiphany_core::HandsActionIntent;
use epiphany_core::MIND_GATEWAY_REVIEW_SCHEMA_VERSION;
use epiphany_core::MindGatewayDecision;
use epiphany_core::MindGatewayReview;
use epiphany_core::RuntimeSpineInitOptions;
use epiphany_core::SOUL_VERDICT_RECEIPT_SCHEMA_VERSION;
use epiphany_core::SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION;
use epiphany_core::SoulVerdictReceipt;
use epiphany_core::SubstrateGateRepoAccessGrantReceipt;
use epiphany_core::hands_action_review_for_intent;
use epiphany_core::hands_command_receipt_for_review;
use epiphany_core::hands_commit_receipt_for_review;
use epiphany_core::hands_patch_receipt_for_review;
use epiphany_core::hands_pr_receipt_for_review;
use epiphany_core::initialize_runtime_spine;
use epiphany_core::load_epiphany_cultmesh_repo_work_overviews;
use epiphany_core::load_epiphany_cultmesh_swarm_brake;
use epiphany_core::load_latest_epiphany_cultmesh_repo_work_overview;
use epiphany_core::mind_state_commit_receipt;
use epiphany_core::put_hands_action_intent;
use epiphany_core::put_hands_action_review;
use epiphany_core::put_hands_command_receipt;
use epiphany_core::put_hands_commit_receipt;
use epiphany_core::put_hands_patch_receipt;
use epiphany_core::put_hands_pr_receipt;
use epiphany_core::put_mind_gateway_review;
use epiphany_core::put_mind_state_commit_receipt;
use epiphany_core::put_soul_verdict_receipt;
use epiphany_core::put_substrate_gate_repo_access_grant_receipt;
use epiphany_core::runtime_hands_action_intent;
use epiphany_core::runtime_hands_action_review;
use epiphany_core::runtime_hands_commit_receipt;
use epiphany_core::runtime_latest_hands_receipt_chain_after;
use epiphany_core::write_epiphany_cultmesh_repo_work_overview;
use epiphany_core::write_epiphany_cultmesh_repo_work_public_proof;
use serde_json::Value;
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

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
        "close" | "closure" | "verify-close" => run_close(parse_close_args(args)?),
        "publish" => run_publish(parse_publish_args(args)?),
        "sync" | "sync-main" => run_sync(parse_sync_args(args)?),
        "overview" | "proof-bundle" | "status" => run_overview(parse_overview_args(args)?),
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
    verification_command: Option<String>,
    verification_summary: Option<String>,
    modeling_summary: Option<String>,
    closure_model_ref: Option<String>,
    closure_model_verdict: Option<String>,
    closure_model_finding: Option<String>,
    require_closure_model_verdict: bool,
    model_authored: bool,
    state_revision: u64,
}

#[derive(Clone, Debug)]
struct PublishArgs {
    workspace: PathBuf,
    epiphany_root: PathBuf,
    item: Option<String>,
    adopt_receipt: Option<PathBuf>,
    closure_receipt: Option<PathBuf>,
    runtime_store: Option<PathBuf>,
    local_verse_store: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    commit_receipt_id: Option<String>,
    hands_pr_receipt_id: Option<String>,
    target_branch: Option<String>,
    change_summary: String,
    justification: String,
    verification_receipts: Vec<String>,
    review_receipts: Vec<String>,
    author_agents: Vec<String>,
    credit_subjects: Vec<String>,
    credit_receipt_ids: Option<Vec<String>>,
    ledger_entry_id: String,
    pull_request_url: String,
    pull_request_number: String,
    pull_request_title: String,
    publication_status: String,
}

#[derive(Clone, Debug)]
struct SyncArgs {
    workspace: PathBuf,
    item: Option<String>,
    publish_receipt: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    upstream_ref: String,
    merge_receipts: Vec<String>,
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
    let mut verification_command = None;
    let mut verification_summary = None;
    let mut modeling_summary = None;
    let mut closure_model_ref = None;
    let mut closure_model_verdict = None;
    let mut closure_model_finding = None;
    let mut require_closure_model_verdict = false;
    let mut model_authored = false;
    let mut state_revision = 0_u64;

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
            "--verification-command" => {
                verification_command = Some(take_string(&mut args, "--verification-command")?);
            }
            "--verification-summary" => {
                verification_summary = Some(take_string(&mut args, "--verification-summary")?);
            }
            "--modeling-summary" => {
                modeling_summary = Some(take_string(&mut args, "--modeling-summary")?);
            }
            "--closure-model-ref" | "--model-ref" => {
                closure_model_ref = Some(take_string(&mut args, "--closure-model-ref")?);
            }
            "--closure-model-verdict" | "--model-verdict" => {
                closure_model_verdict = Some(take_string(&mut args, "--closure-model-verdict")?);
            }
            "--closure-model-finding" | "--model-finding" => {
                closure_model_finding = Some(take_string(&mut args, "--closure-model-finding")?);
            }
            "--require-closure-model-verdict" | "--require-model-verdict" => {
                require_closure_model_verdict = true;
            }
            "--model-authored" => model_authored = true,
            "--state-revision" => state_revision = take_u64(&mut args, "--state-revision")?,
            other => return Err(anyhow!("unexpected close argument {other:?}")),
        }
    }
    Ok(CloseArgs {
        workspace: workspace.context("missing --workspace")?,
        item,
        execute_receipt,
        runtime_store,
        artifact_dir,
        verification_command,
        verification_summary,
        modeling_summary,
        closure_model_ref,
        closure_model_verdict,
        closure_model_finding,
        require_closure_model_verdict,
        model_authored,
        state_revision,
    })
}

fn parse_publish_args(args: impl Iterator<Item = String>) -> Result<PublishArgs> {
    let mut workspace = None;
    let mut epiphany_root = None;
    let mut item = None;
    let mut adopt_receipt = None;
    let mut closure_receipt = None;
    let mut runtime_store = None;
    let mut local_verse_store = None;
    let mut artifact_dir = None;
    let mut commit_receipt_id = None;
    let mut hands_pr_receipt_id = None;
    let mut target_branch = None;
    let mut change_summary = None;
    let mut justification = None;
    let mut verification_receipts = Vec::new();
    let mut review_receipts = Vec::new();
    let mut author_agents = Vec::new();
    let mut credit_subjects = Vec::new();
    let mut credit_receipt_ids = Vec::new();
    let mut ledger_entry_id = None;
    let mut pull_request_url = None;
    let mut pull_request_number = None;
    let mut pull_request_title = None;
    let mut publication_status = None;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--epiphany-root" => epiphany_root = Some(take_path(&mut args, "--epiphany-root")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--adopt-receipt" => adopt_receipt = Some(take_path(&mut args, "--adopt-receipt")?),
            "--closure-receipt" | "--close-receipt" => {
                closure_receipt = Some(take_path(&mut args, "--closure-receipt")?);
            }
            "--runtime-store" => runtime_store = Some(take_path(&mut args, "--runtime-store")?),
            "--local-verse-store" | "--store" => {
                local_verse_store = Some(take_path(&mut args, "--local-verse-store")?);
            }
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--commit-receipt-id" => {
                commit_receipt_id = Some(take_string(&mut args, "--commit-receipt-id")?);
            }
            "--hands-pr-receipt-id" => {
                hands_pr_receipt_id = Some(take_string(&mut args, "--hands-pr-receipt-id")?);
            }
            "--target-branch" => target_branch = Some(take_string(&mut args, "--target-branch")?),
            "--change-summary" => {
                change_summary = Some(take_string(&mut args, "--change-summary")?)
            }
            "--justification" => justification = Some(take_string(&mut args, "--justification")?),
            "--verification-receipt" | "--soul-receipt" => {
                verification_receipts.push(take_string(&mut args, "--verification-receipt")?);
            }
            "--review-receipt" | "--mind-review-receipt" => {
                review_receipts.push(take_string(&mut args, "--review-receipt")?);
            }
            "--author-agent" => author_agents.push(take_string(&mut args, "--author-agent")?),
            "--credit-subject" => credit_subjects.push(take_string(&mut args, "--credit-subject")?),
            "--credit-receipt-id" => {
                credit_receipt_ids.push(take_string(&mut args, "--credit-receipt-id")?);
            }
            "--ledger-entry-id" => {
                ledger_entry_id = Some(take_string(&mut args, "--ledger-entry-id")?);
            }
            "--pull-request-url" | "--publication-url" => {
                pull_request_url = Some(take_string(&mut args, "--pull-request-url")?);
            }
            "--pull-request-number" => {
                pull_request_number = Some(take_string(&mut args, "--pull-request-number")?);
            }
            "--pull-request-title" => {
                pull_request_title = Some(take_string(&mut args, "--pull-request-title")?);
            }
            "--publication-status" => {
                publication_status = Some(take_string(&mut args, "--publication-status")?);
            }
            other => return Err(anyhow!("unexpected publish argument {other:?}")),
        }
    }
    if let Some(path) = closure_receipt.as_ref() {
        let closure = read_json(path)?;
        if closure.get("status").and_then(Value::as_str) != Some("closed") {
            return Err(anyhow!("closure receipt {} is not closed", path.display()));
        }
        if verification_receipts.is_empty() {
            if let Some(id) = string_from_json(&closure, &["soul", "verdictReceiptId"]) {
                verification_receipts.push(id);
            }
        }
        if review_receipts.is_empty() {
            if let Some(id) = string_from_json(&closure, &["mind", "stateCommitReceiptId"]) {
                review_receipts.push(id);
            }
        }
    }
    if verification_receipts.is_empty() {
        return Err(anyhow!(
            "publish requires at least one --verification-receipt from Soul"
        ));
    }
    if review_receipts.is_empty() {
        return Err(anyhow!(
            "publish requires at least one --review-receipt from Mind or maintainer review"
        ));
    }
    Ok(PublishArgs {
        workspace: workspace.context("missing --workspace")?,
        epiphany_root: epiphany_root
            .unwrap_or(env::current_dir().context("failed to resolve current directory")?),
        item,
        adopt_receipt,
        closure_receipt,
        runtime_store,
        local_verse_store,
        artifact_dir,
        commit_receipt_id,
        hands_pr_receipt_id,
        target_branch,
        change_summary: change_summary.context("missing --change-summary")?,
        justification: justification.context("missing --justification")?,
        verification_receipts,
        review_receipts,
        author_agents: if author_agents.is_empty() {
            vec!["epiphany.Hands".to_string()]
        } else {
            author_agents
        },
        credit_subjects: if credit_subjects.is_empty() {
            vec!["epiphany.Hands".to_string()]
        } else {
            credit_subjects
        },
        credit_receipt_ids: if credit_receipt_ids.is_empty() {
            None
        } else {
            Some(credit_receipt_ids)
        },
        ledger_entry_id: ledger_entry_id.context("missing --ledger-entry-id")?,
        pull_request_url: pull_request_url.context("missing --pull-request-url")?,
        pull_request_number: pull_request_number.unwrap_or_else(|| "unknown".to_string()),
        pull_request_title: pull_request_title.context("missing --pull-request-title")?,
        publication_status: publication_status
            .unwrap_or_else(|| "accepted-for-github-publication".to_string()),
    })
}

fn parse_sync_args(args: impl Iterator<Item = String>) -> Result<SyncArgs> {
    let mut workspace = None;
    let mut item = None;
    let mut publish_receipt = None;
    let mut artifact_dir = None;
    let mut upstream_ref = None;
    let mut merge_receipts = Vec::new();

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--publish-receipt" => {
                publish_receipt = Some(take_path(&mut args, "--publish-receipt")?);
            }
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--upstream-ref" => upstream_ref = Some(take_string(&mut args, "--upstream-ref")?),
            "--merge-receipt" | "--sync-receipt" => {
                merge_receipts.push(take_string(&mut args, "--merge-receipt")?);
            }
            other => return Err(anyhow!("unexpected sync argument {other:?}")),
        }
    }
    if merge_receipts.is_empty() {
        return Err(anyhow!(
            "sync requires at least one --merge-receipt or --sync-receipt from Bifrost or maintainer review"
        ));
    }
    Ok(SyncArgs {
        workspace: workspace.context("missing --workspace")?,
        item,
        publish_receipt,
        artifact_dir,
        upstream_ref: upstream_ref.unwrap_or_else(|| "origin/main".to_string()),
        merge_receipts,
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
    let public_ref = format!("eve://epiphany/persona#repo-intake/{item_slug}/{audit_id}");
    let candidate_ref = format!("candidate-action://{runtime_id}/{item_slug}/{audit_id}");
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
            "candidateActionRef": candidate_ref
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
        "bubblePath": receipt["persona"]["bubblePath"],
        "acceptReceiptPath": accept["receiptPath"],
        "feedback": accept["feedback"],
        "authority": receipt["authority"],
        "privateStateExposed": false,
        "nextSafeMove": receipt["nextSafeMove"],
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

    let substrate_grant = SubstrateGateRepoAccessGrantReceipt {
        schema_version: SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: substrate_grant_id.clone(),
        runtime_job_id: runtime_job_id.clone(),
        binding_id: "repo-work-runner".to_string(),
        role: "epiphany-hands".to_string(),
        authority_scope: "repo.branch_local_work".to_string(),
        granted_operations: vec!["read".to_string(), "snapshot".to_string()],
        granted_paths: requested_paths.clone(),
        granted_at: now.clone(),
        contract: "Substrate Gate grants read/snapshot access for work-run planning only; mutation awaits an approved Hands review.".to_string(),
    };
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
        "repo-manifest" | "body-manifest" | "epiphany-manifest" => {
            derive_repo_manifest_plan(input, &action_family)
        }
        "repo-tool-capabilities" | "tool-capabilities" | "capability-manifest" => {
            derive_repo_tool_capabilities_plan(input, &action_family)
        }
        "repo-collaboration-topic" | "collaboration-topic" | "eve-collaboration" => {
            derive_repo_collaboration_topic_plan(input, &action_family)
        }
        other => Err(anyhow!(
            "unsupported derive-plan action family {other:?}; supported families are append-worklog, planning-note, checklist-note, section-note, repo-status-section, task-card, repo-manifest, repo-tool-capabilities, and repo-collaboration-topic"
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

fn derive_repo_manifest_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let target_path = validate_toml_target_path(input.target_path.unwrap_or("epiphany.toml"))?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let body_domain = format!("repo:{item_slug}");
    let private_verse_id = format!("epiphany.repo.{item_slug}.private");
    let local_verse_id = "gamecult-local".to_string();
    let public_verse_id = "epiphany-global".to_string();
    let eve_surface_id = format!("eve://epiphany/repo/{item_slug}");
    let lines = vec![
        "# Epiphany repo Body manifest.".to_string(),
        "# Branch-local public routing cargo; not private state, publication, merge, service, or cross-repo authority.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_body_manifest.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.body_manifest")
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
        "[body]".to_string(),
        format!("domain = {}", toml_basic_string(&body_domain)),
        "authority_owner = \"Epiphany Self\"".to_string(),
        "hosted_by_daemon = false".to_string(),
        "branch_local_only = true".to_string(),
        String::new(),
        "[verses]".to_string(),
        format!("private = {}", toml_basic_string(&private_verse_id)),
        format!("local = {}", toml_basic_string(&local_verse_id)),
        format!("public = {}", toml_basic_string(&public_verse_id)),
        "private_state_may_leave_repo = false".to_string(),
        String::new(),
        "[eve]".to_string(),
        format!("surface = {}", toml_basic_string(&eve_surface_id)),
        "agent_friendly_tui = true".to_string(),
        "public_discussion_allowed = true".to_string(),
        String::new(),
        "[capabilities]".to_string(),
        "advertised = [\"repo-work-overview\", \"repo-work-public-proof\"]".to_string(),
        "requires_receipts = true".to_string(),
        "arbitrary_shell_authority = false".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the repo manifest path changed and contains the accepted pressure summary.\","
            .to_string(),
        "  \"Soul verifies the manifest names body, Verse, and Eve routing ids without private-state exposure.\","
            .to_string(),
        "  \"Soul verifies no paths outside the declared manifest changed.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove epiphany.toml if the repo Body manifest was misderived.\"]"
            .to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.body_manifest".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a repo Body manifest from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!("Add Epiphany repo manifest for work item {}", input.item),
        verification_asks: vec![
            "Soul verifies the repo Body manifest path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the manifest publishes body, Verse, and Eve routing ids while sealing private state.".to_string(),
            "Soul verifies no paths outside the declared manifest changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated repo Body manifest if the accepted pressure was misinterpreted.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.body_manifest"),
    })
}

fn derive_repo_tool_capabilities_plan(
    input: DeriveSafePlanInput<'_>,
    action_family: &str,
) -> Result<DerivedSafePlan> {
    let item_slug = sanitize(input.item);
    let target_path = validate_toml_target_path(
        input
            .target_path
            .unwrap_or(".epiphany/repo-tool-capabilities.toml"),
    )?;
    let candidate_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "candidateActionRefs"]);
    let public_refs =
        string_array_from_json(input.accept_receipt, &["feedback", "publicDiscussionRefs"]);
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let tool_surface_id = format!("epiphany.repo.{item_slug}.tool-capabilities");
    let intent_contract = "epiphany.cultmesh.daemon_tool_invocation_intent.v0";
    let receipt_contract = "epiphany.cultmesh.daemon_tool_invocation_receipt.v0";
    let lines = vec![
        "# Epiphany repo tool capability manifest.".to_string(),
        "# Branch-local capability discovery cargo; the hosting daemon owns execution.".to_string(),
        format!(
            "schema_version = {}",
            toml_basic_string("epiphany.repo_tool_capabilities.v0")
        ),
        format!("item = {}", toml_basic_string(input.item)),
        format!("created_at = {}", toml_basic_string(&now)),
        format!("source = {}", toml_basic_string(input.source)),
        format!("summary = {}", toml_basic_string(&compact_line(input.summary))),
        format!(
            "safe_action_family = {}",
            toml_basic_string("repo.tool_capabilities")
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
        "[tool_directory]".to_string(),
        format!("surface_id = {}", toml_basic_string(&tool_surface_id)),
        "verse = \"gamecult-local\"".to_string(),
        "odin_discoverable = true".to_string(),
        "available_to_authorized_agents = true".to_string(),
        "agent_friendly_tui = true".to_string(),
        String::new(),
        "[contracts]".to_string(),
        format!(
            "intent = {}",
            toml_basic_string(intent_contract)
        ),
        format!(
            "receipt = {}",
            toml_basic_string(receipt_contract)
        ),
        "requires_receipts = true".to_string(),
        "host_daemon_owns_execution = true".to_string(),
        "idunn_owns_lifecycle = true".to_string(),
        String::new(),
        "[expected_capabilities]".to_string(),
        "ids = [".to_string(),
        "  \"repo-work-overview\"," .to_string(),
        "  \"repo-work-queue-run\"," .to_string(),
        "  \"repo-work-public-proof\"," .to_string(),
        "  \"bifrost-public-proof\"".to_string(),
        "]".to_string(),
        String::new(),
        "[authority]".to_string(),
        "arbitrary_shell_authority = false".to_string(),
        "deployment_authority = false".to_string(),
        "service_start_stop_authority = false".to_string(),
        "private_verse_rummaging = false".to_string(),
        "tool_invocation_requires_host_receipt = true".to_string(),
        String::new(),
        "[verification]".to_string(),
        "asks = [".to_string(),
        "  \"Soul verifies the capability manifest path changed and contains the accepted pressure summary.\"," .to_string(),
        "  \"Soul verifies the manifest names tool intent and receipt contracts without granting execution authority.\"," .to_string(),
        "  \"Soul verifies no paths outside the declared tool capability manifest changed.\"".to_string(),
        "]".to_string(),
        String::new(),
        "[rollback]".to_string(),
        "hints = [\"Remove the repo tool capability manifest if the accepted pressure was misderived.\"]".to_string(),
        String::new(),
    ];
    let command = powershell_set_lines_command(&target_path, &lines);
    Ok(DerivedSafePlan {
        safe_action_family: "repo.tool_capabilities".to_string(),
        target_path,
        plan_summary: format!(
            "Imagination derived a repo tool capability manifest from accepted {} pressure.",
            input.source
        ),
        command,
        commit_message: format!(
            "Add repo tool capability manifest for work item {}",
            input.item
        ),
        verification_asks: vec![
            "Soul verifies the repo tool capability manifest path changed and contains the accepted pressure summary.".to_string(),
            "Soul verifies the manifest advertises typed tool intent/receipt contracts while leaving execution with host daemons.".to_string(),
            "Soul verifies no paths outside the declared capability manifest changed.".to_string(),
        ],
        rollback_hints: vec![
            "Remove the generated repo tool capability manifest if the accepted pressure was misinterpreted.".to_string(),
        ],
        derivation: plan_derivation_receipt(input, action_family, "repo.tool_capabilities"),
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
        format!("public_room = {}", toml_basic_string(&public_room)),
        format!("eve_surface = {}", toml_basic_string(&eve_surface)),
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

fn closure_family_assertions(
    workspace: &Path,
    commit_sha: &str,
    execute_receipt: &Value,
    item: &str,
) -> Result<(Value, bool)> {
    let Some(plan_receipt_path) = path_from_json(execute_receipt, &["planReceiptPath"]) else {
        return Ok((
            json!({
                "status": "skipped",
                "reason": "execute receipt has no planReceiptPath",
                "assertions": []
            }),
            true,
        ));
    };
    let plan_receipt = read_json_if_exists(&plan_receipt_path)?.unwrap_or(Value::Null);
    if plan_receipt.is_null() {
        return Ok((
            json!({
                "status": "skipped",
                "reason": "plan receipt path is missing",
                "planReceiptPath": plan_receipt_path,
                "assertions": []
            }),
            true,
        ));
    }
    let safe_family = string_from_json(&plan_receipt, &["derivation", "safeActionFamily"])
        .unwrap_or_else(|| "manual-or-unknown".to_string());
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
    let item_slug = sanitize(item);
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
        "repo.append_worklog" => {
            push_assertion(
                &mut assertions,
                "worklog-summary-present",
                content.contains(&compact_summary),
                "Committed worklog contains the accepted pressure summary.".to_string(),
            );
        }
        "repo.markdown_planning_note" => {
            push_assertion(
                &mut assertions,
                "planning-summary-present",
                content.contains(&compact_summary),
                "Committed planning note contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "planning-authority-seal-present",
                content.contains("Authority seal"),
                "Committed planning note carries an authority seal.".to_string(),
            );
        }
        "repo.checklist_note" => {
            push_assertion(
                &mut assertions,
                "checklist-summary-present",
                content.contains(&compact_summary),
                "Committed checklist contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "checklist-items-present",
                content.contains("- [ ]"),
                "Committed checklist carries branch-local checklist items.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "checklist-authority-present",
                content.contains("Authority"),
                "Committed checklist carries an authority section.".to_string(),
            );
        }
        "repo.markdown_managed_section" => {
            let start_marker = format!("<!-- epiphany-section:{item_slug}:start -->");
            let end_marker = format!("<!-- epiphany-section:{item_slug}:end -->");
            push_assertion(
                &mut assertions,
                "managed-section-start-marker",
                content.contains(&start_marker),
                "Committed managed section contains its start marker.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "managed-section-end-marker",
                content.contains(&end_marker),
                "Committed managed section contains its end marker.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "managed-section-summary-present",
                content.contains(&compact_summary),
                "Committed managed section contains the accepted pressure summary.".to_string(),
            );
        }
        "repo.status_section" => {
            let start_marker = format!("<!-- epiphany-status:{item_slug}:start -->");
            let end_marker = format!("<!-- epiphany-status:{item_slug}:end -->");
            push_assertion(
                &mut assertions,
                "status-section-start-marker",
                content.contains(&start_marker),
                "Committed repo status section contains its start marker.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "status-section-end-marker",
                content.contains(&end_marker),
                "Committed repo status section contains its end marker.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "status-section-summary-present",
                content.contains(&compact_summary),
                "Committed repo status section contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "status-section-private-seal",
                content.contains("Private state exposed: false"),
                "Committed repo status section preserves the private-state seal.".to_string(),
            );
        }
        "repo.task_card" => {
            push_assertion(
                &mut assertions,
                "task-card-schema-present",
                content.contains("schema_version = \"epiphany.repo_work_task_card.v0\""),
                "Committed task card carries the task-card schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "task-card-family-present",
                content.contains("safe_action_family = \"repo.task_card\""),
                "Committed task card carries the task-card safe family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "task-card-summary-present",
                content.contains(&compact_summary),
                "Committed task card contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "task-card-private-seal",
                content.contains("private_state_exposed = false"),
                "Committed task card preserves the private-state seal.".to_string(),
            );
        }
        "repo.body_manifest" => {
            push_assertion(
                &mut assertions,
                "repo-manifest-schema-present",
                content.contains("schema_version = \"epiphany.repo_body_manifest.v0\""),
                "Committed repo manifest carries the Body manifest schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "repo-manifest-family-present",
                content.contains("safe_action_family = \"repo.body_manifest\""),
                "Committed repo manifest carries the Body manifest safe family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "repo-manifest-summary-present",
                content.contains(&compact_summary),
                "Committed repo manifest contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "repo-manifest-body-domain",
                content.contains("[body]") && content.contains("domain = \"repo:"),
                "Committed repo manifest names the repo Body domain.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "repo-manifest-verses-present",
                content.contains("[verses]")
                    && content.contains("private = \"epiphany.repo.")
                    && content.contains("local = \"gamecult-local\"")
                    && content.contains("public = \"epiphany-global\""),
                "Committed repo manifest names private, local, and public Verse ids.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "repo-manifest-eve-surface",
                content.contains("[eve]") && content.contains("surface = \"eve://epiphany/repo/"),
                "Committed repo manifest names the Eve surface.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "repo-manifest-private-seal",
                content.contains("private_state_exposed = false")
                    && content.contains("private_state_may_leave_repo = false"),
                "Committed repo manifest preserves private-state seals.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "repo-manifest-no-arbitrary-shell",
                content.contains("arbitrary_shell_authority = false")
                    && content.contains("requires_receipts = true"),
                "Committed repo manifest keeps arbitrary shell authority sealed behind receipts."
                    .to_string(),
            );
        }
        "repo.tool_capabilities" => {
            push_assertion(
                &mut assertions,
                "tool-capabilities-schema-present",
                content.contains("schema_version = \"epiphany.repo_tool_capabilities.v0\""),
                "Committed tool capability manifest carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "tool-capabilities-family-present",
                content.contains("safe_action_family = \"repo.tool_capabilities\""),
                "Committed tool capability manifest carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "tool-capabilities-summary-present",
                content.contains(&compact_summary),
                "Committed tool capability manifest contains the accepted pressure summary."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "tool-directory-present",
                content.contains("[tool_directory]")
                    && content.contains("verse = \"gamecult-local\"")
                    && content.contains("odin_discoverable = true")
                    && content.contains("available_to_authorized_agents = true"),
                "Committed tool capability manifest exposes local CultMesh/Odin discovery."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "tool-contracts-present",
                content.contains("[contracts]")
                    && content.contains(
                        "intent = \"epiphany.cultmesh.daemon_tool_invocation_intent.v0\"",
                    )
                    && content.contains(
                        "receipt = \"epiphany.cultmesh.daemon_tool_invocation_receipt.v0\"",
                    )
                    && content.contains("requires_receipts = true")
                    && content.contains("host_daemon_owns_execution = true"),
                "Committed tool capability manifest names typed invocation contracts and host execution ownership.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "tool-capabilities-ids-present",
                content.contains("[expected_capabilities]")
                    && content.contains("\"repo-work-overview\"")
                    && content.contains("\"repo-work-queue-run\"")
                    && content.contains("\"repo-work-public-proof\"")
                    && content.contains("\"bifrost-public-proof\""),
                "Committed tool capability manifest lists expected repo-swarm capability ids."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "tool-capabilities-authority-seals",
                content.contains("[authority]")
                    && content.contains("arbitrary_shell_authority = false")
                    && content.contains("deployment_authority = false")
                    && content.contains("service_start_stop_authority = false")
                    && content.contains("private_verse_rummaging = false")
                    && content.contains("tool_invocation_requires_host_receipt = true"),
                "Committed tool capability manifest denies shell/deploy/service/private-rummaging authority and requires host receipts.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "tool-capabilities-private-seal",
                content.contains("private_state_exposed = false"),
                "Committed tool capability manifest preserves the private-state seal.".to_string(),
            );
        }
        "repo.collaboration_topic" => {
            push_assertion(
                &mut assertions,
                "collaboration-topic-schema-present",
                content.contains("schema_version = \"epiphany.repo_collaboration_topic.v0\""),
                "Committed collaboration topic carries the schema version.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-topic-family-present",
                content.contains("safe_action_family = \"repo.collaboration_topic\""),
                "Committed collaboration topic carries the safe action family.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-topic-summary-present",
                content.contains(&compact_summary),
                "Committed collaboration topic contains the accepted pressure summary.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-topic-public-surface",
                content.contains("[topic]")
                    && content.contains("public_room = \"epiphany-global/persona-collaboration/")
                    && content.contains("eve_surface = \"eve://epiphany/repo/")
                    && content.contains("persona_discussion_allowed = true")
                    && content.contains("human_discussion_allowed = true"),
                "Committed collaboration topic names public discussion and Eve surfaces."
                    .to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-topic-imagination-route",
                content.contains("[imagination]")
                    && content.contains("consensus_route = \"imagination://repo/")
                    && content.contains("consensus_required_before_action = true")
                    && content.contains("candidate_actions_are_non_authoritative = true")
                    && content.contains("mind_adoption_required = true"),
                "Committed collaboration topic routes feedback to Imagination consensus before adoption.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-topic-authority-seals",
                content.contains("[authority]")
                    && content.contains("adoption_authorized = false")
                    && content.contains("hands_action_authorized = false")
                    && content.contains("publication_authorized = false")
                    && content.contains("cross_body_mutation_authorized = false")
                    && content.contains("private_verse_rummaging = false"),
                "Committed collaboration topic denies action, publication, cross-body, and private-rummaging authority.".to_string(),
            );
            push_assertion(
                &mut assertions,
                "collaboration-topic-private-seal",
                content.contains("private_state_exposed = false"),
                "Committed collaboration topic preserves the private-state seal.".to_string(),
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

fn closure_model_review(
    model_authored: bool,
    model_ref: Option<&str>,
    verdict: Option<&str>,
    finding: Option<&str>,
    required: bool,
) -> Result<(Value, bool)> {
    let normalized_verdict = verdict
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty());
    if let Some(verdict) = normalized_verdict.as_deref() {
        match verdict {
            "passed" | "failed" | "needs-work" | "blocked" => {}
            other => {
                return Err(anyhow!(
                    "unsupported closure model verdict {other:?}; expected passed, failed, needs-work, or blocked"
                ));
            }
        }
    }

    let gate_enforced = required || normalized_verdict.is_some();
    let passed = match normalized_verdict.as_deref() {
        Some("passed") => true,
        Some("failed" | "needs-work" | "blocked") => false,
        Some(_) => unreachable!(),
        None => !required,
    };
    let status = match normalized_verdict.as_deref() {
        Some("passed") => "passed",
        Some("failed" | "needs-work" | "blocked") => "failed",
        Some(_) => unreachable!(),
        None if required => "missing-required-verdict",
        None if model_authored || model_ref.is_some() => "provenance-only",
        None => "deterministic-fallback",
    };
    let reason = match status {
        "passed" => "model-authored closure verdict passed",
        "failed" => "model-authored closure verdict refused closure",
        "missing-required-verdict" => {
            "closure required a model-authored verdict but none was supplied"
        }
        "provenance-only" => "model provenance was recorded without making it a hard closure gate",
        _ => "deterministic closure checks are the active gate",
    };

    Ok((
        json!({
            "schemaVersion": "epiphany.repo_work_model_closure_review.v0",
            "status": status,
            "passed": passed,
            "required": required,
            "gateEnforced": gate_enforced,
            "modelAuthored": model_authored || model_ref.is_some(),
            "modelRef": model_ref,
            "verdict": normalized_verdict,
            "finding": finding.map(compact_line),
            "reason": reason,
            "reviewedInputs": [
                "Hands commit receipt matched execute receipt",
                "Verification command result",
                "Declared versus actual git changed paths",
                "Safe-family committed-content assertions",
                "Authority seal and private-state exposure flag"
            ],
            "operatorAuthoredShellDetails": false,
            "privateStateExposed": false
        }),
        passed,
    ))
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
    model_ref: Option<&'a str>,
    model_authored: bool,
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

    let item = run_receipt
        .get("item")
        .and_then(Value::as_str)
        .unwrap_or("work-item")
        .to_string();
    let item_slug = sanitize(&item);
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
        "handsActionGate": adoption_receipt["handsActionGate"],
        "authority": adoption_receipt["authority"],
        "privateStateExposed": false,
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
        "nextSafeMove": "Route Soul verification and Mind review before publication; use epiphany-work publish only after review receipts exist."
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

fn run_close(args: CloseArgs) -> Result<Value> {
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
    let verification_command = args
        .verification_command
        .unwrap_or_else(|| format!("git show --stat --oneline {commit_sha}"));
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
    let (family_assertions, family_assertions_passed) =
        closure_family_assertions(&workspace, &commit_sha, &execute_receipt, &item)?;
    let closure_model_authored = args.model_authored || args.closure_model_ref.is_some();
    let (model_closure_review, model_closure_passed) = closure_model_review(
        closure_model_authored,
        args.closure_model_ref.as_deref(),
        args.closure_model_verdict.as_deref(),
        args.closure_model_finding.as_deref(),
        args.require_closure_model_verdict,
    )?;
    let closure_review_id = format!("repo-work-close-{item_slug}-closure-review");
    let closure_review_path = artifact_dir.join(format!("work-close-{item_slug}-review.json"));
    let closure_review = json!({
        "schemaVersion": "epiphany.repo_work_closure_review.v0",
        "createdAt": now,
        "workspace": workspace,
        "receiptId": closure_review_id,
        "item": item,
        "owner": "Soul+Modeling",
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
            "familyAssertionsPassed": family_assertions_passed,
            "modelClosurePassed": model_closure_passed
        },
        "familyAssertions": family_assertions,
        "modelingReview": {
            "modelAuthored": closure_model_authored,
            "modelRef": args.closure_model_ref.clone(),
            "deterministicFallback": !closure_model_authored,
            "operatorAuthoredShellDetails": false,
            "summary": args.modeling_summary.clone(),
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
        "nextSafeMove": "Soul may pass closure only when verification succeeds, actual git changed paths match the Hands-declared path scope, safe-family assertions pass for known Imagination families, and any supplied or required model-authored closure verdict passes."
    });
    write_json(&closure_review_path, &closure_review)?;
    let verification_passed = verification.status.success()
        && path_scope_matched
        && family_assertions_passed
        && model_closure_passed;
    let soul_verdict_id = format!("repo-work-close-{item_slug}-soul-verdict");
    let soul_summary = args.verification_summary.unwrap_or_else(|| {
        if verification_passed {
            format!("Soul verified branch-local commit {commit_sha} for repo work item {item}.")
        } else if !path_scope_matched {
            format!(
                "Soul verification failed for branch-local commit {commit_sha}: actual changed paths did not match declared Hands path scope."
            )
        } else if !family_assertions_passed {
            format!(
                "Soul verification failed for branch-local commit {commit_sha}: safe-family assertions did not pass."
            )
        } else if !model_closure_passed {
            format!(
                "Soul verification failed for branch-local commit {commit_sha}: model-authored closure review did not pass."
            )
        } else {
            format!("Soul verification failed for branch-local commit {commit_sha}.")
        }
    });
    let mut evidence_ids = vec![
        patch_receipt_id.clone(),
        command_receipt_id.clone(),
        commit_receipt_id.clone(),
        normalize_path_for_receipt(&stdout_artifact),
        normalize_path_for_receipt(&stderr_artifact),
        normalize_path_for_receipt(&closure_review_path),
    ];
    evidence_ids.extend(declared_changed_paths.clone());
    let soul_verdict = SoulVerdictReceipt {
        schema_version: SOUL_VERDICT_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: soul_verdict_id.clone(),
        source_result_id: format!("repo-work-execute-{item_slug}"),
        source_job_id: format!("repo-work-close-{item_slug}"),
        verdict: if verification_passed {
            "passed".to_string()
        } else {
            "failed".to_string()
        },
        summary: soul_summary.clone(),
        evidence_ids: evidence_ids.clone(),
        risks: if verification_passed {
            Vec::new()
        } else if !path_scope_matched {
            vec![
                "Closure refused because actual git changed paths differ from Hands-declared scope."
                    .to_string(),
            ]
        } else if !family_assertions_passed {
            vec![
                "Closure refused because the committed content did not satisfy known safe-family assertions."
                    .to_string(),
            ]
        } else if !model_closure_passed {
            vec![
                "Closure refused because the model-authored closure review was missing or non-passing."
                    .to_string(),
            ]
        } else {
            vec!["Closure verification command failed; publication remains blocked.".to_string()]
        },
        emitted_at: now.clone(),
        contract: "Soul verdict for repo-work closure; verifies the Hands patch/command/commit consequence before Modeling/Mind closure and Bifrost publication.".to_string(),
    };
    put_soul_verdict_receipt(&runtime_store, &soul_verdict)?;

    let closure_receipt_path = artifact_dir.join(format!("work-close-{item_slug}.json"));
    if !verification_passed {
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

    let modeling_summary = args.modeling_summary.unwrap_or_else(|| {
        format!(
            "Modeling records repo work item {item} changed [{}] at commit {commit_sha}; scheduler should stop implementation until publication review.",
            declared_changed_paths.join(", ")
        )
    });
    let gateway_id = format!("repo-work-close-{item_slug}-mind-review");
    let mind_review = MindGatewayReview {
        schema_version: MIND_GATEWAY_REVIEW_SCHEMA_VERSION.to_string(),
        gateway_id: gateway_id.clone(),
        source_kind: "repo_work_closure".to_string(),
        source_role_id: "modeling".to_string(),
        decision: MindGatewayDecision::Accept,
        allowed_effects: vec![
            "repoWorkClosure".to_string(),
            "modelingSummary".to_string(),
            "publicationGate".to_string(),
        ],
        refused_effects: Vec::new(),
        reasons: vec![
            "Soul passed the branch-local Hands consequence.".to_string(),
            "Modeling summary is source-grounded in execute receipt, closure review, and commit proof.".to_string(),
            "Mind admits closure metadata only; Bifrost still gates publication and merge.".to_string(),
        ],
        contract: "Mind review for repo-work closure; admits the verified Modeling summary and publication gate without granting merge or service authority.".to_string(),
    };
    put_mind_gateway_review(&runtime_store, &mind_review)?;
    let mind_commit_id = format!("repo-work-close-{item_slug}-mind-commit");
    let mind_commit = mind_state_commit_receipt(
        mind_commit_id.clone(),
        &mind_review,
        args.state_revision,
        vec![
            "repoWork.closure".to_string(),
            "repoWork.modelingSummary".to_string(),
        ],
        Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    );
    put_mind_state_commit_receipt(&runtime_store, &mind_commit)?;

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
            "summary": modeling_summary,
            "changedPaths": execute_receipt["changedPaths"],
            "commitSha": execute_receipt["handsReceipts"]["commitSha"],
            "source": "epiphany.repo_work_closure_review.v0",
            "modelAuthored": closure_model_authored
        },
        "mind": {
            "gatewayReviewId": mind_review.gateway_id,
            "stateCommitReceiptId": mind_commit.receipt_id,
            "stateRevision": mind_commit.state_revision,
            "changedFields": mind_commit.changed_fields
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
        "nextSafeMove": "Use epiphany-work publish --closure-receipt <receipt> with Bifrost/GitHub refs; merge remains gated by maintainers and sync receipts."
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

fn run_publish(args: PublishArgs) -> Result<Value> {
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
    let adopt_receipt_path =
        resolve_adopt_receipt(&workspace, args.item.as_deref(), args.adopt_receipt)?;
    let adopt_receipt = read_json(&adopt_receipt_path)?;
    let run_receipt_path = path_from_json(&adopt_receipt, &["runReceiptPath"])
        .ok_or_else(|| anyhow!("adopt receipt has no runReceiptPath"))?;
    let run_receipt = read_json(&run_receipt_path)?;
    let online_receipt_path = path_from_json(&run_receipt, &["onlineReceiptPath"])
        .ok_or_else(|| anyhow!("run receipt has no onlineReceiptPath"))?;
    let online_receipt = read_json(&online_receipt_path)?;
    let runtime_id = string_from_json(&adopt_receipt, &["runtimeId"])
        .or_else(|| string_from_json(&run_receipt, &["runtimeId"]))
        .or_else(|| string_from_json(&online_receipt, &["runtimeId"]))
        .unwrap_or_else(|| "repo-swarm-local".to_string());
    let runtime_store = args.runtime_store.unwrap_or_else(|| {
        path_from_json(&adopt_receipt, &["runtimeStore"]).unwrap_or_else(|| {
            workspace
                .join(".epiphany")
                .join("state")
                .join("runtime-spine.msgpack")
        })
    });
    let local_verse_store = args.local_verse_store.unwrap_or_else(|| {
        path_from_json(&online_receipt, &["localVerseStore"])
            .unwrap_or_else(|| workspace.join(".epiphany").join("local-verse.ccmp"))
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

    let commit_receipt = if let Some(commit_receipt_id) = args.commit_receipt_id.clone() {
        runtime_hands_commit_receipt(&runtime_store, &commit_receipt_id)?
            .ok_or_else(|| anyhow!("Hands commit receipt {commit_receipt_id} was not found"))?
    } else {
        let lower_bound = adopt_receipt
            .get("createdAt")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("adopt receipt has no createdAt timestamp"))?;
        let chain = runtime_latest_hands_receipt_chain_after(&runtime_store, lower_bound)?
            .ok_or_else(|| anyhow!("no complete Hands patch/command/commit chain after adoption; pass --commit-receipt-id after recording branch-local work"))?;
        runtime_hands_commit_receipt(&runtime_store, &chain.commit_receipt_id)?.ok_or_else(
            || {
                anyhow!(
                    "Hands commit receipt {} was not found",
                    chain.commit_receipt_id
                )
            },
        )?
    };
    if commit_receipt.intent_id != intent.intent_id || commit_receipt.review_id != review.review_id
    {
        return Err(anyhow!(
            "Hands commit receipt {} belongs to intent {}/review {}, not {}/{}",
            commit_receipt.receipt_id,
            commit_receipt.intent_id,
            commit_receipt.review_id,
            intent.intent_id,
            review.review_id
        ));
    }

    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let mut publication_review = review.clone();
    if !publication_review
        .allowed_operations
        .iter()
        .any(|operation| operation == "pr")
    {
        publication_review.allowed_operations.push("pr".to_string());
    }
    if !publication_review
        .required_receipts
        .iter()
        .any(|receipt| receipt == HANDS_PR_RECEIPT_TYPE)
    {
        publication_review
            .required_receipts
            .push(HANDS_PR_RECEIPT_TYPE.to_string());
    }
    publication_review.reviewed_at = now.clone();
    publication_review.reasons.push(format!(
        "Bifrost publication routing approved after verification refs [{}] and review refs [{}].",
        args.verification_receipts.join(", "),
        args.review_receipts.join(", ")
    ));
    put_hands_action_review(&runtime_store, &publication_review)?;

    let item = adopt_receipt
        .get("item")
        .and_then(Value::as_str)
        .unwrap_or("work-item")
        .to_string();
    let item_slug = sanitize(&item);
    let hands_pr_receipt_id = args
        .hands_pr_receipt_id
        .clone()
        .unwrap_or_else(|| format!("repo-work-publish-{item_slug}-hands-pr"));
    let bifrost_intent_id = format!("repo-work-publish-{item_slug}-bifrost-intent");
    let bifrost_publication_receipt_id =
        format!("repo-work-publish-{item_slug}-bifrost-publication");
    let github_publication_receipt_id = format!("repo-work-publish-{item_slug}-github");
    let target_branch = args
        .target_branch
        .clone()
        .unwrap_or_else(|| commit_receipt.branch.clone());
    let target_repository = format!("repo:{}", workspace.display());
    let body_domain = target_repository.clone();

    let mut verse_args = vec![
        "bifrost-publication".to_string(),
        "--store".to_string(),
        local_verse_store.display().to_string(),
        "--runtime-id".to_string(),
        runtime_id.clone(),
        "--intent-id".to_string(),
        bifrost_intent_id.clone(),
        "--receipt-id".to_string(),
        bifrost_publication_receipt_id.clone(),
        "--github-receipt-id".to_string(),
        github_publication_receipt_id.clone(),
        "--target-repository".to_string(),
        target_repository.clone(),
        "--target-branch".to_string(),
        target_branch.clone(),
        "--change-summary".to_string(),
        args.change_summary.clone(),
        "--justification".to_string(),
        args.justification.clone(),
        "--ledger-entry-id".to_string(),
        args.ledger_entry_id.clone(),
        "--hands-pr-receipt-id".to_string(),
        hands_pr_receipt_id.clone(),
        "--publication-url".to_string(),
        args.pull_request_url.clone(),
        "--pull-request-number".to_string(),
        args.pull_request_number.clone(),
        "--commit-sha".to_string(),
        commit_receipt.commit_sha.clone(),
        "--source-cluster-id".to_string(),
        "epiphany.cluster.hands".to_string(),
        "--source-agent-id".to_string(),
        "epiphany.Hands".to_string(),
        "--body-domain".to_string(),
        body_domain,
        "--receipt-status".to_string(),
        args.publication_status.clone(),
    ];
    for path in &commit_receipt.changed_paths {
        verse_args.extend(["--changed-path".to_string(), path.clone()]);
    }
    for receipt in &args.verification_receipts {
        verse_args.extend(["--verification-receipt".to_string(), receipt.clone()]);
    }
    for receipt in &args.review_receipts {
        verse_args.extend(["--review-receipt".to_string(), receipt.clone()]);
    }
    for author in &args.author_agents {
        verse_args.extend(["--author-agent".to_string(), author.clone()]);
    }
    for subject in &args.credit_subjects {
        verse_args.extend(["--credit-subject".to_string(), subject.clone()]);
    }
    if let Some(credit_receipts) = &args.credit_receipt_ids {
        for receipt in credit_receipts {
            verse_args.extend(["--credit-receipt".to_string(), receipt.clone()]);
        }
    }
    let bifrost = cargo_json(&manifest_path, "epiphany-verse-query", &verse_args)?;

    let pr_receipt = hands_pr_receipt_for_review(
        hands_pr_receipt_id.clone(),
        &intent,
        &publication_review,
        &commit_receipt,
        args.pull_request_url.clone(),
        args.pull_request_number.clone(),
        args.pull_request_title.clone(),
        bifrost_publication_receipt_id.clone(),
        format!("Published repo work item {item} through Bifrost receipts."),
        now.clone(),
    );
    put_hands_pr_receipt(&runtime_store, &pr_receipt)?;

    let publish_receipt = json!({
        "schemaVersion": "epiphany.repo_work_publish_receipt.v0",
        "createdAt": now,
        "workspace": workspace,
        "runtimeId": runtime_id,
        "runtimeStore": runtime_store,
        "localVerseStore": local_verse_store,
        "adoptReceiptPath": adopt_receipt_path,
        "runReceiptPath": run_receipt_path,
        "onlineReceiptPath": online_receipt_path,
        "closureReceiptPath": args.closure_receipt,
        "item": item,
        "status": "publication-receipts-recorded",
        "targetRepository": target_repository,
        "targetBranch": target_branch,
        "changedPaths": commit_receipt.changed_paths,
        "handsActionGate": {
            "intentId": intent.intent_id,
            "reviewId": publication_review.review_id,
            "decision": publication_review.decision,
            "allowedOperations": publication_review.allowed_operations,
            "requiredReceipts": publication_review.required_receipts
        },
        "handsReceipts": {
            "commitReceiptId": commit_receipt.receipt_id,
            "commitSha": commit_receipt.commit_sha,
            "branch": commit_receipt.branch,
            "prReceiptId": pr_receipt.receipt_id,
            "pullRequestUrl": pr_receipt.pull_request_url,
            "pullRequestNumber": pr_receipt.pull_request_number,
            "pullRequestTitle": pr_receipt.pull_request_title
        },
        "bifrost": {
            "intentId": bifrost["intentId"],
            "publicationReceiptId": bifrost["publicationReceiptId"],
            "githubPublicationReceiptId": bifrost["githubPublicationReceiptId"],
            "ledgerEntryId": bifrost["ledgerEntryId"],
            "creditReceiptIds": bifrost["creditReceiptIds"],
            "pullRequestUrl": bifrost["pullRequestUrl"]
        },
        "verificationReceipts": args.verification_receipts,
        "reviewReceipts": args.review_receipts,
        "authority": {
            "publicationAuthorized": true,
            "upstreamMainSynced": false,
            "mergeAuthorized": false,
            "mergeGate": "maintainer_or_bifrost_merge_receipt",
            "privateStateExposed": false
        },
        "nextSafeMove": "Maintain the PR/publication receipt chain; do not claim upstream main is synced until a merge/sync receipt exists."
    });
    let receipt_path = artifact_dir.join(format!("work-publish-{item_slug}.json"));
    write_json(&receipt_path, &publish_receipt)?;
    Ok(json!({
        "schemaVersion": "epiphany.repo_work_publish.v0",
        "status": publish_receipt["status"],
        "workspace": publish_receipt["workspace"],
        "runtimeId": publish_receipt["runtimeId"],
        "runtimeStore": publish_receipt["runtimeStore"],
        "localVerseStore": publish_receipt["localVerseStore"],
        "receiptPath": receipt_path,
        "item": publish_receipt["item"],
        "handsReceipts": publish_receipt["handsReceipts"],
        "bifrost": publish_receipt["bifrost"],
        "authority": publish_receipt["authority"],
        "privateStateExposed": false,
        "nextSafeMove": publish_receipt["nextSafeMove"],
    }))
}

fn run_sync(args: SyncArgs) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    ensure_git_repo(&workspace)?;
    let publish_receipt_path =
        resolve_publish_receipt(&workspace, args.item.as_deref(), args.publish_receipt)?;
    let publish_receipt = read_json(&publish_receipt_path)?;
    let artifact_dir = args
        .artifact_dir
        .unwrap_or_else(|| workspace.join(".epiphany").join("work"));
    fs::create_dir_all(&artifact_dir)
        .with_context(|| format!("failed to create {}", artifact_dir.display()))?;

    let item = publish_receipt
        .get("item")
        .and_then(Value::as_str)
        .unwrap_or("work-item")
        .to_string();
    let item_slug = sanitize(&item);
    let published_commit_sha = publish_receipt
        .get("handsReceipts")
        .and_then(|hands| hands.get("commitSha"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("publish receipt has no handsReceipts.commitSha"))?
        .to_string();
    let commit_receipt_id = publish_receipt
        .get("handsReceipts")
        .and_then(|hands| hands.get("commitReceiptId"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let pr_receipt_id = publish_receipt
        .get("handsReceipts")
        .and_then(|hands| hands.get("prReceiptId"))
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let upstream_commit_sha =
        git_output(&workspace, &["rev-parse", "--verify", &args.upstream_ref])
            .with_context(|| format!("failed to resolve upstream ref {}", args.upstream_ref))?;
    let canonical_published_commit = git_output(
        &workspace,
        &["rev-parse", "--verify", &published_commit_sha],
    )
    .with_context(|| format!("failed to resolve published commit {published_commit_sha}"))?;
    let upstream_main_synced = git_status_success(
        &workspace,
        &[
            "merge-base",
            "--is-ancestor",
            &canonical_published_commit,
            &args.upstream_ref,
        ],
    )?;
    let status = if upstream_main_synced {
        "upstream-main-synced"
    } else {
        "upstream-main-not-synced"
    };
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let sync_receipt = json!({
        "schemaVersion": "epiphany.repo_work_sync_receipt.v0",
        "createdAt": now,
        "workspace": workspace,
        "publishReceiptPath": publish_receipt_path,
        "item": item,
        "status": status,
        "upstreamRef": args.upstream_ref,
        "upstreamCommitSha": upstream_commit_sha,
        "publishedCommitSha": canonical_published_commit,
        "mergeReceipts": args.merge_receipts,
        "handsReceipts": {
            "commitReceiptId": commit_receipt_id,
            "prReceiptId": pr_receipt_id
        },
        "bifrost": publish_receipt["bifrost"],
        "authority": {
            "publicationAuthorized": true,
            "upstreamMainSynced": upstream_main_synced,
            "mergeAuthorized": upstream_main_synced,
            "mergeAuthorityReceipts": args.merge_receipts,
            "privateStateExposed": false
        },
        "nextSafeMove": if upstream_main_synced {
            "Update durable map/Mind receipts and proof bundle; upstream main now contains the published work."
        } else {
            "Wait for maintainer/Bifrost merge, then rerun epiphany-work sync against upstream main."
        }
    });
    let receipt_path = artifact_dir.join(format!("work-sync-{item_slug}.json"));
    write_json(&receipt_path, &sync_receipt)?;
    Ok(json!({
        "schemaVersion": "epiphany.repo_work_sync.v0",
        "status": sync_receipt["status"],
        "workspace": sync_receipt["workspace"],
        "receiptPath": receipt_path,
        "item": sync_receipt["item"],
        "upstreamRef": sync_receipt["upstreamRef"],
        "upstreamCommitSha": sync_receipt["upstreamCommitSha"],
        "publishedCommitSha": sync_receipt["publishedCommitSha"],
        "authority": sync_receipt["authority"],
        "privateStateExposed": false,
        "nextSafeMove": sync_receipt["nextSafeMove"],
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
    let publish_receipt_path = work_receipt_path(&workspace, "publish", &item);
    let sync_receipt_path = work_receipt_path(&workspace, "sync", &item);
    let overview_receipt_path = artifact_dir.join(format!("work-overview-{item_slug}.json"));

    let plan_receipt = read_json_if_exists(&plan_receipt_path)?;
    let run_receipt = read_json_if_exists(&run_receipt_path)?;
    let adopt_receipt = read_json_if_exists(&adopt_receipt_path)?;
    let execute_receipt = read_json_if_exists(&execute_receipt_path)?;
    let close_receipt = read_json_if_exists(&close_receipt_path)?;
    let publish_receipt = read_json_if_exists(&publish_receipt_path)?;
    let sync_receipt = read_json_if_exists(&sync_receipt_path)?;

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
    let publication_status = publish_receipt
        .as_ref()
        .and_then(|receipt| receipt.get("publicationStatus").and_then(Value::as_str))
        .or_else(|| {
            publish_receipt
                .as_ref()
                .and_then(|receipt| receipt.get("status").and_then(Value::as_str))
        })
        .unwrap_or("missing");
    let sync_status = sync_receipt
        .as_ref()
        .and_then(|receipt| receipt.get("status").and_then(Value::as_str))
        .unwrap_or("missing");
    let (gate, blocker, next_safe_move) = repo_work_overview_gate(
        plan_receipt.as_ref(),
        run_receipt.as_ref(),
        adopt_receipt.as_ref(),
        execute_receipt.as_ref(),
        close_receipt.as_ref(),
        publish_receipt.as_ref(),
        sync_receipt.as_ref(),
    );
    let tui_rows = vec![
        format!("item {item}"),
        format!("branch {branch}"),
        format!("gate {gate}"),
        format!("blocker {blocker}"),
        format!("next {next_safe_move}"),
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
        ("publish", &publish_receipt_path),
        ("sync", &sync_receipt_path),
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
        &publish_receipt_path,
        &sync_receipt_path,
    );
    let proof_artifacts = repo_work_proof_artifact_rows(&[
        ("accept", &accept_receipt_path),
        ("plan", &plan_receipt_path),
        ("run", &run_receipt_path),
        ("adopt", &adopt_receipt_path),
        ("execute", &execute_receipt_path),
        ("close-review", &close_review_receipt_path),
        ("close", &close_receipt_path),
        ("publish", &publish_receipt_path),
        ("sync", &sync_receipt_path),
    ])?;
    let proof_publication_rows =
        repo_work_proof_publication_rows(publish_receipt.as_ref(), sync_receipt.as_ref());
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
        "publishReceiptPath": existing_path_value(&publish_receipt_path),
        "syncReceiptPath": existing_path_value(&sync_receipt_path),
        "changedPaths": changed_paths,
        "commitSha": commit_sha,
        "soulVerdict": soul_verdict,
        "mindStateCommitReceiptId": close_receipt.as_ref().and_then(|receipt| string_from_json(receipt, &["mind", "stateCommitReceiptId"])),
        "bifrostPublicationReceiptId": publish_receipt.as_ref().and_then(|receipt| string_from_json(receipt, &["bifrost", "publicationReceiptId"])),
        "githubPublicationReceiptId": publish_receipt.as_ref().and_then(|receipt| string_from_json(receipt, &["github", "publicationReceiptId"])),
        "upstreamMainSynced": sync_receipt.as_ref().and_then(sync_receipt_upstream_main_synced).unwrap_or(false),
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
        "rows": rows,
        "authority": {
            "owner": "Eyes/Gjallar",
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
                    "Gjallar/Eve may project these rows, but they do not own scheduling, publication, merge, service lifecycle, or cross-repo mutation.".to_string(),
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
        "rows": receipt["rows"],
        "verseProjection": verse_projection,
        "authority": receipt["authority"],
        "privateStateExposed": false
    }))
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
            "Gjallar/Odin may discover this proof, but Bifrost still owns publication, labor ledger, credit, and public consequence.".to_string(),
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
        "selectedRows": receipt["selectedRows"],
        "outputs": receipt["outputs"],
        "authority": receipt["authority"],
        "privateStateExposed": false,
        "nextSafeMove": receipt["nextSafeMove"],
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
    let publish_receipt_path = work_receipt_path(&workspace, "publish", &item);
    let sync_receipt_path = work_receipt_path(&workspace, "sync", &item);

    let before_receipts = repo_work_receipt_state(
        &accept_receipt_path,
        &plan_receipt_path,
        &run_receipt_path,
        &adopt_receipt_path,
        &execute_receipt_path,
        &close_receipt_path,
        &publish_receipt_path,
        &sync_receipt_path,
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
                        &publish_receipt_path,
                        &sync_receipt_path,
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
                &publish_receipt_path,
                &sync_receipt_path,
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
                    &publish_receipt_path,
                    &sync_receipt_path,
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

    if sync_receipt_path.exists() {
        action = "none".to_string();
        status = "noop".to_string();
        reason = "work item already has an upstream sync receipt".to_string();
        next_safe_move = "No branch-local scheduler action remains for this item.".to_string();
    } else if publish_receipt_path.exists() {
        action = "none".to_string();
        status = "noop".to_string();
        reason =
            "publication receipt exists; scheduler stops before merge/sync authority".to_string();
        next_safe_move =
            "Wait for maintainer/Bifrost merge receipt, then run epiphany-work sync.".to_string();
    } else if close_receipt_path.exists() {
        action = "none".to_string();
        status = "noop".to_string();
        reason = "Soul/Modeling/Mind closure is recorded; scheduler stops before Bifrost publication authority".to_string();
        next_safe_move =
            "Route Bifrost/GitHub publication through epiphany-work publish --closure-receipt."
                .to_string();
    } else if execute_receipt_path.exists() {
        if args.dry_run {
            action = "close-from-execute".to_string();
            status = "would-advance".to_string();
            reason =
                "branch-local execution receipt exists and Soul/Modeling/Mind closure is missing"
                    .to_string();
            next_safe_move =
                "Rerun without --dry-run to close the executed branch-local work before publication."
                    .to_string();
        } else {
            action = "close-from-execute".to_string();
            advanced_result = run_close(CloseArgs {
                workspace: workspace.clone(),
                item: Some(item.clone()),
                execute_receipt: Some(execute_receipt_path.clone()),
                runtime_store: args.runtime_store.clone(),
                artifact_dir: Some(artifact_dir.clone()),
                verification_command: None,
                verification_summary: Some(format!(
                    "Scheduler pulse closed executed repo work item {item}."
                )),
                modeling_summary: Some(format!(
                    "Modeling records scheduler-closed repo work item {item}; publication remains gated by Bifrost."
                )),
                closure_model_ref: None,
                closure_model_verdict: None,
                closure_model_finding: None,
                require_closure_model_verdict: false,
                model_authored: false,
                state_revision: 0,
            })?;
            status = "advanced".to_string();
            reason = "closed executed branch-local work through Soul, Modeling, and Mind receipts"
                .to_string();
            next_safe_move =
                "Route Bifrost/GitHub publication through epiphany-work publish --closure-receipt."
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
        &publish_receipt_path,
        &sync_receipt_path,
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

fn resolve_publish_receipt(
    workspace: &Path,
    item: Option<&str>,
    explicit: Option<PathBuf>,
) -> Result<PathBuf> {
    if let Some(path) = explicit {
        return Ok(path);
    }
    let work_dir = workspace.join(".epiphany").join("work");
    if let Some(item) = item {
        return Ok(work_dir.join(format!("work-publish-{}.json", sanitize(item))));
    }
    latest_receipt_in(&work_dir, "work-publish-").ok_or_else(|| {
        anyhow!(
            "no work publish receipt found; run epiphany-work publish first or pass --publish-receipt"
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
    publish: &Path,
    sync: &Path,
) -> Value {
    json!({
        "accept": receipt_path_state(accept),
        "plan": receipt_path_state(plan),
        "run": receipt_path_state(run),
        "adopt": receipt_path_state(adopt),
        "execute": receipt_path_state(execute),
        "close": receipt_path_state(close),
        "publish": receipt_path_state(publish),
        "sync": receipt_path_state(sync),
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

fn repo_work_proof_publication_rows(publish: Option<&Value>, sync: Option<&Value>) -> Vec<Value> {
    let mut rows = Vec::new();
    if let Some(receipt) = publish {
        rows.push(json!({
            "kind": "bifrost",
            "status": receipt.get("status").and_then(Value::as_str).unwrap_or("unknown"),
            "intentId": string_from_json(receipt, &["bifrost", "intentId"]),
            "publicationReceiptId": string_from_json(receipt, &["bifrost", "publicationReceiptId"]),
            "githubPublicationReceiptId": string_from_json(receipt, &["bifrost", "githubPublicationReceiptId"]),
            "ledgerEntryId": string_from_json(receipt, &["bifrost", "ledgerEntryId"]),
            "creditReceiptIds": receipt.get("bifrost").and_then(|bifrost| bifrost.get("creditReceiptIds")).cloned().unwrap_or(Value::Array(Vec::new())),
            "pullRequestUrl": string_from_json(receipt, &["bifrost", "pullRequestUrl"]),
            "privateStateExposed": false
        }));
        rows.push(json!({
            "kind": "github",
            "status": receipt.get("status").and_then(Value::as_str).unwrap_or("unknown"),
            "commitReceiptId": string_from_json(receipt, &["handsReceipts", "commitReceiptId"]),
            "commitSha": string_from_json(receipt, &["handsReceipts", "commitSha"]),
            "prReceiptId": string_from_json(receipt, &["handsReceipts", "prReceiptId"]),
            "pullRequestUrl": string_from_json(receipt, &["handsReceipts", "pullRequestUrl"]),
            "pullRequestNumber": receipt
                .get("handsReceipts")
                .and_then(|hands| hands.get("pullRequestNumber"))
                .cloned()
                .unwrap_or(Value::Null),
            "pullRequestTitle": string_from_json(receipt, &["handsReceipts", "pullRequestTitle"]),
            "privateStateExposed": false
        }));
    }
    if let Some(receipt) = sync {
        rows.push(json!({
            "kind": "upstream-main",
            "status": receipt.get("status").and_then(Value::as_str).unwrap_or("unknown"),
            "upstreamRef": string_from_json(receipt, &["upstreamRef"]),
            "publishedCommitSha": string_from_json(receipt, &["publishedCommitSha"]),
            "upstreamCommitSha": string_from_json(receipt, &["upstreamCommitSha"]),
            "upstreamMainSynced": sync_receipt_upstream_main_synced(receipt).unwrap_or(false),
            "privateStateExposed": false
        }));
    }
    rows
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
            "owner": "Eyes/Gjallar",
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

fn sync_receipt_upstream_main_synced(receipt: &Value) -> Option<bool> {
    receipt
        .get("upstreamMainSynced")
        .and_then(Value::as_bool)
        .or_else(|| {
            receipt
                .get("authority")
                .and_then(|authority| authority.get("upstreamMainSynced"))
                .and_then(Value::as_bool)
        })
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
    publish: Option<&Value>,
    sync: Option<&Value>,
) -> (&'static str, &'static str, &'static str) {
    if sync.is_some() {
        (
            "complete-or-awaiting-new-work",
            "none",
            "No local branch-work action remains for this item.",
        )
    } else if publish.is_some() {
        (
            "awaiting-upstream-sync",
            "merge-or-sync-receipt-missing",
            "After maintainer/Bifrost merge, run epiphany-work sync.",
        )
    } else if close.is_some() {
        (
            "awaiting-publication",
            "bifrost-publication-missing",
            "Run epiphany-work publish --closure-receipt when publication is authorized.",
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
) -> Vec<String> {
    overviews
        .iter()
        .map(|overview| {
            let workspace_match = overview_workspace_matches(overview, workspace);
            let tick_actionable = repo_work_gate_is_tick_actionable(&overview.current_gate);
            format!(
                "QUEUE-RUN | item={} | gate={} | blocker={} | actionable={} | workspaceMatch={} | next={} | private={}",
                overview.item,
                overview.current_gate,
                overview.blocker,
                tick_actionable,
                workspace_match,
                overview.next_safe_move,
                overview.private_state_exposed
            )
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

fn git_status_success(workspace: &Path, args: &[&str]) -> Result<bool> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    Ok(output.status.success())
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
        "usage: epiphany-work <persona-intake|accept|derive-plan|plan|run|adopt|execute|close|publish|sync|overview|export-proof|tick|queue-run|serve> ...\n\
         persona-intake --workspace <repo> --item <id> --message <text> [--topic <topic>] [--store <local-verse.ccmp>] [--runtime-id <id>]\n\
         accept --workspace <repo> --from <persona|bifrost|persona-or-bifrost> --item <id> [--summary <text>] [--topic <topic>] [--store <local-verse.ccmp>] [--runtime-id <id>] [--online-receipt <path>] [--public-discussion-ref <ref>] [--candidate-action-ref <ref>]\n\
         derive-plan --workspace <repo> [--item <id>] [--accept-receipt <path>] [--action-family append-worklog|planning-note|checklist-note|section-note|repo-status-section|task-card|repo-manifest|repo-tool-capabilities|repo-collaboration-topic] [--target-path <path>] [--model-ref <ref>] [--model-authored] [--action-summary <text>] [--verification-ask <text>] [--stop-condition <text>] [--escalation-reason <text>]\n\
         plan --workspace <repo> [--item <id>] --objective <text> --plan-summary <text> --command <command> --changed-path <path> --commit-message <text> [--adoption-evidence-ref <ref>]\n\
         run --workspace <repo> [--item <id>] [--accept-receipt <path>] [--runtime-store <path>] [--requested-path <path>]\n\
         adopt --workspace <repo> [--item <id>] [--run-receipt <path>] [--from-plan <path>] [--plan-summary <text>] [--adoption-evidence-ref <ref>]\n\
         execute --workspace <repo> [--item <id>] [--from-plan <path>] [--command <command>] [--changed-path <path>] [--commit-message <text>]\n\
         close --workspace <repo> [--item <id>] [--execute-receipt <path>] [--verification-command <command>] [--closure-model-ref <ref>] [--model-authored] [--closure-model-verdict passed|failed|needs-work|blocked] [--closure-model-finding <text>] [--require-closure-model-verdict]\n\
         publish --workspace <repo> [--item <id>] --change-summary <text> --justification <text> --verification-receipt <ref> --review-receipt <ref> --ledger-entry-id <id> --pull-request-url <url> --pull-request-title <text>\n\
         sync --workspace <repo> [--item <id>] [--publish-receipt <path>] [--upstream-ref origin/main] --merge-receipt <ref>\n\
         overview --workspace <repo> [--item <id>] [--accept-receipt <path>] [--no-write]\n\
         export-proof --workspace <repo> [--item <id>] [--accept-receipt <path>] [--output <path>] [--local-verse-store <path>] [--runtime-id repo-swarm-local]\n\
         tick --workspace <repo> [--item <id>] [--local-verse-store <path>] [--runtime-store <path>] [--dry-run]\n\
         queue-run --workspace <repo> [--local-verse-store <path>] [--runtime-id repo-swarm-local] [--max-items 1] [--dry-run]"
    );
}
