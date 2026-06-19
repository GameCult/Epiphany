use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::Utc;
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
        "usage: epiphany-work accept --workspace <repo> --from <persona|bifrost|persona-or-bifrost> --item <id> [--summary <text>] [--topic <topic>] [--store <local-verse.ccmp>] [--runtime-id <id>] [--online-receipt <path>] [--public-discussion-ref <ref>] [--candidate-action-ref <ref>]"
    );
}
