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
use epiphany_core::hands_pr_receipt_for_review;
use epiphany_core::initialize_runtime_spine;
use epiphany_core::put_hands_action_intent;
use epiphany_core::put_hands_action_review;
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
        "run" => run_work(parse_run_args(args)?),
        "adopt" | "promote" => run_adopt(parse_adopt_args(args)?),
        "publish" => run_publish(parse_publish_args(args)?),
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
struct AdoptArgs {
    workspace: PathBuf,
    epiphany_root: PathBuf,
    item: Option<String>,
    run_receipt: Option<PathBuf>,
    runtime_store: Option<PathBuf>,
    artifact_dir: Option<PathBuf>,
    plan_summary: String,
    adoption_evidence_refs: Vec<String>,
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

fn parse_adopt_args(args: impl Iterator<Item = String>) -> Result<AdoptArgs> {
    let mut workspace = None;
    let mut epiphany_root = None;
    let mut item = None;
    let mut run_receipt = None;
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
            "--runtime-store" => runtime_store = Some(take_path(&mut args, "--runtime-store")?),
            "--artifact-dir" => artifact_dir = Some(take_path(&mut args, "--artifact-dir")?),
            "--plan-summary" => plan_summary = Some(take_string(&mut args, "--plan-summary")?),
            "--adoption-evidence-ref" | "--evidence-ref" => {
                adoption_evidence_refs.push(take_string(&mut args, "--adoption-evidence-ref")?);
            }
            other => return Err(anyhow!("unexpected adopt argument {other:?}")),
        }
    }
    if adoption_evidence_refs.is_empty() {
        return Err(anyhow!(
            "adopt requires at least one --adoption-evidence-ref proving Imagination/Self/Mind adoption"
        ));
    }
    Ok(AdoptArgs {
        workspace: workspace.context("missing --workspace")?,
        epiphany_root: epiphany_root
            .unwrap_or(env::current_dir().context("failed to resolve current directory")?),
        item,
        run_receipt,
        runtime_store,
        artifact_dir,
        plan_summary: plan_summary.context("missing --plan-summary")?,
        adoption_evidence_refs,
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
            format!("Adopted plan: {}", args.plan_summary),
            format!(
                "Adoption evidence refs: {}",
                args.adoption_evidence_refs.join(", ")
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
        "item": item,
        "status": "approved-for-branch-local-hands-action",
        "planSummary": args.plan_summary,
        "adoptionEvidenceRefs": args.adoption_evidence_refs,
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
        "usage: epiphany-work <accept|run|adopt|publish> ...\n\
         accept --workspace <repo> --from <persona|bifrost|persona-or-bifrost> --item <id> [--summary <text>] [--topic <topic>] [--store <local-verse.ccmp>] [--runtime-id <id>] [--online-receipt <path>] [--public-discussion-ref <ref>] [--candidate-action-ref <ref>]\n\
         run --workspace <repo> [--item <id>] [--accept-receipt <path>] [--runtime-store <path>] [--requested-path <path>]\n\
         adopt --workspace <repo> [--item <id>] [--run-receipt <path>] --plan-summary <text> --adoption-evidence-ref <ref>\n\
         publish --workspace <repo> [--item <id>] --change-summary <text> --justification <text> --verification-receipt <ref> --review-receipt <ref> --ledger-entry-id <id> --pull-request-url <url> --pull-request-title <text>"
    );
}
