use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::Utc;
use epiphany_core::HANDS_ACTION_INTENT_SCHEMA_VERSION;
use epiphany_core::HANDS_COMMAND_RECEIPT_TYPE;
use epiphany_core::HANDS_COMMIT_RECEIPT_TYPE;
use epiphany_core::HANDS_PATCH_RECEIPT_TYPE;
use epiphany_core::HANDS_PR_RECEIPT_TYPE;
use epiphany_core::HandsActionIntent;
use epiphany_core::RuntimeSpineInitOptions;
use epiphany_core::SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION;
use epiphany_core::SubstrateGateRepoAccessGrantReceipt;
use epiphany_core::hands_action_review_for_intent;
use epiphany_core::hands_command_receipt_for_review;
use epiphany_core::hands_commit_receipt_for_review;
use epiphany_core::hands_patch_receipt_for_review;
use epiphany_core::hands_pr_receipt_for_review;
use epiphany_core::initialize_runtime_spine;
use epiphany_core::put_hands_action_intent;
use epiphany_core::put_hands_action_review;
use epiphany_core::put_hands_command_receipt;
use epiphany_core::put_hands_commit_receipt;
use epiphany_core::put_hands_patch_receipt;
use epiphany_core::put_hands_pr_receipt;
use epiphany_core::put_substrate_gate_repo_access_grant_receipt;
use epiphany_core::runtime_hands_action_intent;
use epiphany_core::runtime_hands_action_review;
use epiphany_core::runtime_hands_commit_receipt;
use epiphany_core::runtime_latest_hands_receipt_chain_after;
use serde_json::Value;
use serde_json::json;
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
        "plan" => run_plan(parse_plan_args(args)?),
        "run" => run_work(parse_run_args(args)?),
        "adopt" | "promote" => run_adopt(parse_adopt_args(args)?),
        "execute" | "exec" => run_execute(parse_execute_args(args)?),
        "publish" => run_publish(parse_publish_args(args)?),
        "sync" | "sync-main" => run_sync(parse_sync_args(args)?),
        "tick" | "pulse" | "schedule" => run_tick(parse_tick_args(args)?),
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
struct PublishArgs {
    workspace: PathBuf,
    epiphany_root: PathBuf,
    item: Option<String>,
    adopt_receipt: Option<PathBuf>,
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
struct TickArgs {
    workspace: PathBuf,
    epiphany_root: PathBuf,
    item: Option<String>,
    artifact_dir: Option<PathBuf>,
    runtime_store: Option<PathBuf>,
    dry_run: bool,
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

fn parse_publish_args(args: impl Iterator<Item = String>) -> Result<PublishArgs> {
    let mut workspace = None;
    let mut epiphany_root = None;
    let mut item = None;
    let mut adopt_receipt = None;
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

fn parse_tick_args(args: impl Iterator<Item = String>) -> Result<TickArgs> {
    let mut workspace = None;
    let mut epiphany_root = None;
    let mut item = None;
    let mut artifact_dir = None;
    let mut runtime_store = None;
    let mut dry_run = false;

    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--workspace" => workspace = Some(take_path(&mut args, "--workspace")?),
            "--epiphany-root" => epiphany_root = Some(take_path(&mut args, "--epiphany-root")?),
            "--item" => item = Some(take_string(&mut args, "--item")?),
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--runtime-store" => runtime_store = Some(take_path(&mut args, "--runtime-store")?),
            "--dry-run" | "--no-execute" => dry_run = true,
            other => return Err(anyhow!("unexpected tick argument {other:?}")),
        }
    }
    Ok(TickArgs {
        workspace: workspace.context("missing --workspace")?,
        epiphany_root: epiphany_root
            .unwrap_or(env::current_dir().context("failed to resolve current directory")?),
        item,
        artifact_dir,
        runtime_store,
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

    let item = accept_receipt
        .get("item")
        .and_then(Value::as_str)
        .unwrap_or("work-item")
        .to_string();
    let item_slug = sanitize(&item);
    let normalized_paths = normalize_paths(args.changed_paths.clone());
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let plan_id = format!("repo-work-plan-{item_slug}");
    let action_id = format!("{plan_id}-action-1");
    let plan_receipt = json!({
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
        "objective": args.objective,
        "planSummary": args.plan_summary,
        "adoptionEvidenceRefs": args.adoption_evidence_refs,
        "actions": [{
            "actionId": action_id,
            "kind": "repo.branch_local_command",
            "command": args.command,
            "changedPaths": normalized_paths,
            "commitMessage": args.commit_message,
            "verificationAsks": args.verification_asks,
            "stopConditions": args.stop_conditions,
            "rollbackHints": args.rollback_hints
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

    let plan_receipt_path = work_receipt_path(&workspace, "plan", &item);
    let run_receipt_path = work_receipt_path(&workspace, "run", &item);
    let adopt_receipt_path = work_receipt_path(&workspace, "adopt", &item);
    let execute_receipt_path = work_receipt_path(&workspace, "execute", &item);
    let publish_receipt_path = work_receipt_path(&workspace, "publish", &item);
    let sync_receipt_path = work_receipt_path(&workspace, "sync", &item);

    let before_receipts = repo_work_receipt_state(
        &accept_receipt_path,
        &plan_receipt_path,
        &run_receipt_path,
        &adopt_receipt_path,
        &execute_receipt_path,
        &publish_receipt_path,
        &sync_receipt_path,
    );

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
        reason = "publication receipt exists; scheduler stops before merge/sync authority".to_string();
        next_safe_move =
            "Wait for maintainer/Bifrost merge receipt, then run epiphany-work sync.".to_string();
    } else if execute_receipt_path.exists() {
        action = "none".to_string();
        status = "noop".to_string();
        reason =
            "branch-local execution is recorded; scheduler stops before Soul/Mind/Bifrost gates"
                .to_string();
        next_safe_move =
            "Route Soul verification and Mind review before epiphany-work publish.".to_string();
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
                "Write an Imagination/Self action plan before adopting Hands authority.".to_string();
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
                "Run another scheduler pulse to execute the approved branch-local plan.".to_string();
        }
    } else if plan_receipt_path.exists() {
        let plan_receipt = read_json(&plan_receipt_path)?;
        let requested_paths = first_plan_action(&plan_receipt)
            .map(|action| string_array_field(action, "changedPaths"))
            .unwrap_or_default();
        if requested_paths.is_empty() {
            status = "blocked".to_string();
            reason = "plan receipt has no changedPaths for the Substrate Gate".to_string();
            next_safe_move = "Repair the plan receipt or write a new plan with changed paths."
                .to_string();
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
            reason = "opened queued Substrate Gate and Hands run packet from plan paths"
                .to_string();
            next_safe_move =
                "Run another scheduler pulse to adopt the plan into Hands authority.".to_string();
        }
    } else {
        status = "blocked".to_string();
        reason = "accepted work item has no matching action plan receipt".to_string();
        next_safe_move =
            "Create an Imagination/Self plan receipt before scheduler can advance work.".to_string();
    }

    let after_receipts = repo_work_receipt_state(
        &accept_receipt_path,
        &plan_receipt_path,
        &run_receipt_path,
        &adopt_receipt_path,
        &execute_receipt_path,
        &publish_receipt_path,
        &sync_receipt_path,
    );
    let now = Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    let receipt = json!({
        "schemaVersion": "epiphany.repo_work_scheduler_tick_receipt.v0",
        "createdAt": now,
        "workspace": workspace,
        "item": item,
        "scheduler": {
            "owner": "Self",
            "pulseKind": "repo-work-local",
            "oneStepOnly": true,
            "dryRun": args.dry_run
        },
        "status": status,
        "action": action,
        "reason": reason,
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
    let receipt_path = artifact_dir.join(format!("work-tick-{item_slug}.json"));
    write_json(&receipt_path, &receipt)?;
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
    publish: &Path,
    sync: &Path,
) -> Value {
    json!({
        "accept": receipt_path_state(accept),
        "plan": receipt_path_state(plan),
        "run": receipt_path_state(run),
        "adopt": receipt_path_state(adopt),
        "execute": receipt_path_state(execute),
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

fn read_json(path: &Path) -> Result<Value> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("failed to decode {}", path.display()))
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
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
        "usage: epiphany-work <accept|plan|run|adopt|execute|publish|sync|tick> ...\n\
         accept --workspace <repo> --from <persona|bifrost|persona-or-bifrost> --item <id> [--summary <text>] [--topic <topic>] [--store <local-verse.ccmp>] [--runtime-id <id>] [--online-receipt <path>] [--public-discussion-ref <ref>] [--candidate-action-ref <ref>]\n\
         plan --workspace <repo> [--item <id>] --objective <text> --plan-summary <text> --command <command> --changed-path <path> --commit-message <text> [--adoption-evidence-ref <ref>]\n\
         run --workspace <repo> [--item <id>] [--accept-receipt <path>] [--runtime-store <path>] [--requested-path <path>]\n\
         adopt --workspace <repo> [--item <id>] [--run-receipt <path>] [--from-plan <path>] [--plan-summary <text>] [--adoption-evidence-ref <ref>]\n\
         execute --workspace <repo> [--item <id>] [--from-plan <path>] [--command <command>] [--changed-path <path>] [--commit-message <text>]\n\
         publish --workspace <repo> [--item <id>] --change-summary <text> --justification <text> --verification-receipt <ref> --review-receipt <ref> --ledger-entry-id <id> --pull-request-url <url> --pull-request-title <text>\n\
         sync --workspace <repo> [--item <id>] [--publish-receipt <path>] [--upstream-ref origin/main] --merge-receipt <ref>\n\
         tick --workspace <repo> [--item <id>] [--runtime-store <path>] [--dry-run]"
    );
}
