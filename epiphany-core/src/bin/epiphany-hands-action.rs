use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::SecondsFormat;
use epiphany_core::HandsActionIntent;
use epiphany_core::HandsActionReview;
use epiphany_core::hands_command_receipt_for_review;
use epiphany_core::hands_commit_receipt_for_review;
use epiphany_core::hands_patch_receipt_for_review;
use epiphany_core::put_hands_command_receipt;
use epiphany_core::put_hands_commit_receipt;
use epiphany_core::put_hands_patch_receipt;
use epiphany_core::relinquish_repo_frontier_hands_route;
use epiphany_core::runtime_hands_action_intent;
use epiphany_core::runtime_hands_action_review;
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;

const DEFAULT_STORE: &str = "state/runtime-spine.msgpack";

fn main() -> Result<()> {
    let args = Args::parse()?;
    let output = match args.command {
        Command::RecordPatch(command) => record_patch(&args.store, command)?,
        Command::RecordCommand(command) => record_command(&args.store, command)?,
        Command::RecordCommit(command) => record_commit(&args.store, command)?,
        Command::RecordPass(command) => record_pass(&args.store, command)?,
        Command::RecordRefusal(command) => record_refusal(&args.store, command)?,
    };
    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

#[derive(Debug)]
struct Args {
    store: PathBuf,
    command: Command,
}

#[derive(Debug)]
enum Command {
    RecordPatch(RecordPatchArgs),
    RecordCommand(RecordCommandArgs),
    RecordCommit(RecordCommitArgs),
    RecordPass(RecordPassArgs),
    RecordRefusal(RecordRefusalArgs),
}

#[derive(Debug, Clone)]
struct GateArgs {
    intent_id: Option<String>,
    review_id: Option<String>,
    gate_summary: Option<PathBuf>,
    receipt_id: Option<String>,
    summary: String,
}

#[derive(Debug)]
struct RecordPatchArgs {
    gate: GateArgs,
    changed_paths: Vec<String>,
}

#[derive(Debug)]
struct RecordCommandArgs {
    gate: GateArgs,
    command: String,
    exit_code: String,
    stdout_artifact: String,
    stderr_artifact: String,
}

#[derive(Debug)]
struct RecordCommitArgs {
    gate: GateArgs,
    commit_sha: String,
    branch: String,
    changed_paths: Vec<String>,
    validate_commit_sha: bool,
}

#[derive(Debug)]
struct RecordPassArgs {
    gate: GateArgs,
    command: String,
    exit_code: String,
    stdout_artifact: String,
    stderr_artifact: String,
    commit_sha: String,
    branch: String,
    changed_paths: Vec<String>,
    validate_commit_sha: bool,
}

#[derive(Debug)]
struct RecordRefusalArgs {
    gate: GateArgs,
    missing_required_paths: Vec<String>,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut tokens = env::args().skip(1).peekable();
        let mut store = PathBuf::from(DEFAULT_STORE);
        while matches!(tokens.peek().map(String::as_str), Some("--store")) {
            tokens.next();
            store = PathBuf::from(take_value(&mut tokens, "--store")?);
        }
        let command_name = tokens.next().ok_or_else(|| anyhow!("{}", usage()))?;
        let command = match command_name.as_str() {
            "record-patch" => Command::RecordPatch(parse_record_patch(tokens)?),
            "record-command" => Command::RecordCommand(parse_record_command(tokens)?),
            "record-commit" => Command::RecordCommit(parse_record_commit(tokens)?),
            "record-pass" => Command::RecordPass(parse_record_pass(tokens)?),
            "record-refusal" => Command::RecordRefusal(parse_record_refusal(tokens)?),
            _ => return Err(anyhow!("unknown command {command_name}\n{}", usage())),
        };
        Ok(Self { store, command })
    }
}

fn parse_record_refusal(tokens: impl Iterator<Item = String>) -> Result<RecordRefusalArgs> {
    let mut options = ParsedOptions::parse(tokens)?;
    let gate = options.take_gate()?;
    let missing_required_paths = options.take_many("--missing-required-path");
    options.finish()?;
    if missing_required_paths.is_empty() {
        return Err(anyhow!(
            "record-refusal requires at least one --missing-required-path"
        ));
    }
    Ok(RecordRefusalArgs {
        gate,
        missing_required_paths,
    })
}

fn parse_record_patch(tokens: impl Iterator<Item = String>) -> Result<RecordPatchArgs> {
    let mut options = ParsedOptions::parse(tokens)?;
    let gate = options.take_gate()?;
    let changed_paths = options.take_many("--changed-path");
    options.finish()?;
    if changed_paths.is_empty() {
        return Err(anyhow!("record-patch requires at least one --changed-path"));
    }
    Ok(RecordPatchArgs {
        gate,
        changed_paths,
    })
}

fn parse_record_command(tokens: impl Iterator<Item = String>) -> Result<RecordCommandArgs> {
    let mut options = ParsedOptions::parse(tokens)?;
    let gate = options.take_gate()?;
    let command = options.take_required("--command")?;
    let exit_code = options.take_required("--exit-code")?;
    let stdout_artifact = options.take_required("--stdout-artifact")?;
    let stderr_artifact = options.take_required("--stderr-artifact")?;
    options.finish()?;
    Ok(RecordCommandArgs {
        gate,
        command,
        exit_code,
        stdout_artifact,
        stderr_artifact,
    })
}

fn parse_record_commit(tokens: impl Iterator<Item = String>) -> Result<RecordCommitArgs> {
    let mut options = ParsedOptions::parse(tokens)?;
    let gate = options.take_gate()?;
    let commit_sha = options.take_required("--commit-sha")?;
    let branch = options.take_required("--branch")?;
    let changed_paths = options.take_many("--changed-path");
    options.finish()?;
    if changed_paths.is_empty() {
        return Err(anyhow!(
            "record-commit requires at least one --changed-path"
        ));
    }
    Ok(RecordCommitArgs {
        gate,
        commit_sha,
        branch,
        changed_paths,
        validate_commit_sha: true,
    })
}

fn parse_record_pass(tokens: impl Iterator<Item = String>) -> Result<RecordPassArgs> {
    let mut options = ParsedOptions::parse(tokens)?;
    let gate = options.take_gate()?;
    let command = options.take_required("--command")?;
    let exit_code = options.take_required("--exit-code")?;
    let stdout_artifact = options.take_required("--stdout-artifact")?;
    let stderr_artifact = options.take_required("--stderr-artifact")?;
    let commit_sha = options.take_required("--commit-sha")?;
    let branch = options.take_required("--branch")?;
    let changed_paths = options.take_many("--changed-path");
    options.finish()?;
    if changed_paths.is_empty() {
        return Err(anyhow!("record-pass requires at least one --changed-path"));
    }
    Ok(RecordPassArgs {
        gate,
        command,
        exit_code,
        stdout_artifact,
        stderr_artifact,
        commit_sha,
        branch,
        changed_paths,
        validate_commit_sha: true,
    })
}

#[derive(Debug, Default)]
struct ParsedOptions {
    values: Vec<(String, String)>,
}

impl ParsedOptions {
    fn parse(tokens: impl Iterator<Item = String>) -> Result<Self> {
        let mut values = Vec::new();
        let mut tokens = tokens.peekable();
        while let Some(name) = tokens.next() {
            if !name.starts_with("--") {
                return Err(anyhow!("unexpected positional argument {name}"));
            }
            let value = take_value(&mut tokens, &name)?;
            values.push((name, value));
        }
        Ok(Self { values })
    }

    fn take_gate(&mut self) -> Result<GateArgs> {
        let intent_id = self.take_optional("--intent-id");
        let review_id = self.take_optional("--review-id");
        let gate_summary = self.take_optional("--gate-summary").map(PathBuf::from);
        if gate_summary.is_none() && (intent_id.is_none() || review_id.is_none()) {
            return Err(anyhow!(
                "gate requires either --gate-summary or both --intent-id and --review-id"
            ));
        }
        if gate_summary.is_some() && (intent_id.is_some() || review_id.is_some()) {
            return Err(anyhow!(
                "use either --gate-summary or explicit --intent-id/--review-id, not both"
            ));
        }
        Ok(GateArgs {
            intent_id,
            review_id,
            gate_summary,
            receipt_id: self.take_optional("--receipt-id"),
            summary: self.take_required("--summary")?,
        })
    }

    fn take_required(&mut self, name: &str) -> Result<String> {
        self.take_optional(name)
            .ok_or_else(|| anyhow!("{name} is required"))
    }

    fn take_optional(&mut self, name: &str) -> Option<String> {
        let index = self
            .values
            .iter()
            .position(|(candidate, _)| candidate == name)?;
        Some(self.values.remove(index).1)
    }

    fn take_many(&mut self, name: &str) -> Vec<String> {
        let mut taken = Vec::new();
        let mut index = 0;
        while index < self.values.len() {
            if self.values[index].0 == name {
                taken.push(self.values.remove(index).1);
            } else {
                index += 1;
            }
        }
        taken
    }

    fn finish(self) -> Result<()> {
        if self.values.is_empty() {
            return Ok(());
        }
        let names: Vec<_> = self.values.into_iter().map(|(name, _)| name).collect();
        Err(anyhow!("unsupported arguments: {}", names.join(", ")))
    }
}

fn record_patch(store: &PathBuf, args: RecordPatchArgs) -> Result<serde_json::Value> {
    let (intent, review) = load_gate(store, &args.gate, "patch")?;
    validate_paths_within_gate(&intent, &args.changed_paths)?;
    let receipt_id = args
        .gate
        .receipt_id
        .unwrap_or_else(|| generated_receipt_id("hands-patch"));
    let receipt = hands_patch_receipt_for_review(
        receipt_id,
        &intent,
        &review,
        normalize_paths(args.changed_paths),
        args.gate.summary,
        now(),
    );
    put_hands_patch_receipt(store, &receipt)?;
    Ok(json!({
        "status": "ok",
        "type": "epiphany.hands.patch_receipt",
        "receiptId": receipt.receipt_id,
        "intentId": receipt.intent_id,
        "reviewId": receipt.review_id,
        "changedPaths": receipt.changed_paths,
        "store": store,
    }))
}

fn record_command(store: &PathBuf, args: RecordCommandArgs) -> Result<serde_json::Value> {
    let (intent, review) = load_gate(store, &args.gate, "command")?;
    let receipt_id = args
        .gate
        .receipt_id
        .unwrap_or_else(|| generated_receipt_id("hands-command"));
    let receipt = hands_command_receipt_for_review(
        receipt_id,
        &intent,
        &review,
        args.command,
        args.exit_code,
        normalize_path(&args.stdout_artifact),
        normalize_path(&args.stderr_artifact),
        args.gate.summary,
        now(),
    );
    put_hands_command_receipt(store, &receipt)?;
    Ok(json!({
        "status": "ok",
        "type": "epiphany.hands.command_receipt",
        "receiptId": receipt.receipt_id,
        "intentId": receipt.intent_id,
        "reviewId": receipt.review_id,
        "command": receipt.command,
        "exitCode": receipt.exit_code,
        "stdoutArtifact": receipt.stdout_artifact,
        "stderrArtifact": receipt.stderr_artifact,
        "store": store,
    }))
}

fn record_commit(store: &PathBuf, args: RecordCommitArgs) -> Result<serde_json::Value> {
    let (intent, review) = load_gate(store, &args.gate, "commit")?;
    validate_paths_within_gate(&intent, &args.changed_paths)?;
    let commit_sha = if args.validate_commit_sha {
        resolve_git_commit_sha(&args.commit_sha)?
    } else {
        args.commit_sha
    };
    let receipt_id = args
        .gate
        .receipt_id
        .unwrap_or_else(|| generated_receipt_id("hands-commit"));
    let receipt = hands_commit_receipt_for_review(
        receipt_id,
        &intent,
        &review,
        commit_sha,
        args.branch,
        normalize_paths(args.changed_paths),
        args.gate.summary,
        now(),
    );
    put_hands_commit_receipt(store, &receipt)?;
    Ok(json!({
        "status": "ok",
        "type": "epiphany.hands.commit_receipt",
        "receiptId": receipt.receipt_id,
        "intentId": receipt.intent_id,
        "reviewId": receipt.review_id,
        "commitSha": receipt.commit_sha,
        "branch": receipt.branch,
        "changedPaths": receipt.changed_paths,
        "store": store,
    }))
}

fn record_pass(store: &PathBuf, args: RecordPassArgs) -> Result<serde_json::Value> {
    let patch = record_patch(
        store,
        RecordPatchArgs {
            gate: pass_gate(&args.gate, "patch"),
            changed_paths: args.changed_paths.clone(),
        },
    )?;
    let command = record_command(
        store,
        RecordCommandArgs {
            gate: pass_gate(&args.gate, "command"),
            command: args.command,
            exit_code: args.exit_code,
            stdout_artifact: args.stdout_artifact,
            stderr_artifact: args.stderr_artifact,
        },
    )?;
    let commit = record_commit(
        store,
        RecordCommitArgs {
            gate: pass_gate(&args.gate, "commit"),
            commit_sha: args.commit_sha,
            branch: args.branch,
            changed_paths: args.changed_paths,
            validate_commit_sha: args.validate_commit_sha,
        },
    )?;
    Ok(json!({
        "status": "ok",
        "type": "epiphany.hands.action_pass_receipts",
        "patch": patch,
        "command": command,
        "commit": commit,
    }))
}

fn record_refusal(store: &PathBuf, args: RecordRefusalArgs) -> Result<serde_json::Value> {
    let resolved = resolve_gate(&args.gate)?;
    let receipt_id = args
        .gate
        .receipt_id
        .unwrap_or_else(|| generated_receipt_id("hands-refusal"));
    let receipt = relinquish_repo_frontier_hands_route(
        store,
        &resolved.intent_id,
        &resolved.review_id,
        &receipt_id,
        normalize_paths(args.missing_required_paths),
        args.gate.summary,
        now(),
    )?;
    Ok(json!({
        "status": "ok",
        "type": "epiphany.mind.repo_frontier_relinquishment_receipt",
        "receiptId": receipt.receipt_id,
        "handsRefusalReceiptId": receipt.hands_refusal_receipt_id,
        "routeId": receipt.route_id,
        "frontierItemId": receipt.frontier_item_id,
        "previousModelProjectionDigest": receipt.previous_model_projection_digest,
        "previousModelSourceDocuments": receipt.previous_model_source_documents,
        "admittedModelProjectionDigest": receipt.admitted_model_projection_digest,
        "admittedModelSourceDocuments": receipt.admitted_model_source_documents,
        "store": store,
    }))
}

fn pass_gate(gate: &GateArgs, operation: &str) -> GateArgs {
    let mut gate = gate.clone();
    gate.receipt_id = None;
    gate.summary = format!("{} ({operation})", gate.summary);
    gate
}

fn load_gate(
    store: &PathBuf,
    gate: &GateArgs,
    operation: &str,
) -> Result<(HandsActionIntent, HandsActionReview)> {
    let resolved = resolve_gate(gate)?;
    let intent = runtime_hands_action_intent(store, &resolved.intent_id)
        .with_context(|| format!("failed to load Hands intent {}", resolved.intent_id))?
        .ok_or_else(|| anyhow!("Hands intent {} was not found", resolved.intent_id))?;
    let review = runtime_hands_action_review(store, &resolved.review_id)
        .with_context(|| format!("failed to load Hands review {}", resolved.review_id))?
        .ok_or_else(|| anyhow!("Hands review {} was not found", resolved.review_id))?;
    if review.intent_id != intent.intent_id {
        return Err(anyhow!(
            "Hands review {} belongs to intent {}, not {}",
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
    Ok((intent, review))
}

#[derive(Debug)]
struct ResolvedGate {
    intent_id: String,
    review_id: String,
}

fn resolve_gate(gate: &GateArgs) -> Result<ResolvedGate> {
    if let Some(summary_path) = &gate.gate_summary {
        return resolve_gate_from_summary(summary_path);
    }
    Ok(ResolvedGate {
        intent_id: gate
            .intent_id
            .clone()
            .ok_or_else(|| anyhow!("--intent-id is required"))?,
        review_id: gate
            .review_id
            .clone()
            .ok_or_else(|| anyhow!("--review-id is required"))?,
    })
}

fn resolve_gate_from_summary(path: &PathBuf) -> Result<ResolvedGate> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("failed to read coordinator summary {}", path.display()))?;
    let value: serde_json::Value = serde_json::from_str(&text)
        .with_context(|| format!("failed to parse coordinator summary {}", path.display()))?;
    let gate = value
        .pointer("/finalAction/handsActionGate")
        .or_else(|| value.pointer("/handsActionGate"))
        .ok_or_else(|| {
            anyhow!(
                "{} does not contain finalAction.handsActionGate",
                path.display()
            )
        })?;
    let intent_id = gate
        .get("intentId")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("handsActionGate is missing intentId"))?;
    let review_id = gate
        .get("reviewId")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("handsActionGate is missing reviewId"))?;
    Ok(ResolvedGate {
        intent_id: intent_id.to_string(),
        review_id: review_id.to_string(),
    })
}

fn validate_paths_within_gate(intent: &HandsActionIntent, paths: &[String]) -> Result<()> {
    let requested = normalize_paths(intent.requested_paths.clone());
    if requested.iter().any(|path| path == ".") {
        return Ok(());
    }
    for path in normalize_paths(paths.to_vec()) {
        if !requested
            .iter()
            .any(|allowed| path == *allowed || path.starts_with(&format!("{allowed}/")))
        {
            return Err(anyhow!(
                "changed path {path} is outside Hands requested paths: {}",
                requested.join(", ")
            ));
        }
    }
    Ok(())
}

fn normalize_paths(paths: Vec<String>) -> Vec<String> {
    let mut normalized: Vec<_> = paths
        .into_iter()
        .map(|path| normalize_path(&path))
        .filter(|path| !path.is_empty())
        .collect();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn normalize_path(path: &str) -> String {
    path.trim().replace('\\', "/")
}

fn resolve_git_commit_sha(commit_sha: &str) -> Result<String> {
    let output = std::process::Command::new("git")
        .arg("rev-parse")
        .arg("--verify")
        .arg(format!("{commit_sha}^{{commit}}"))
        .output()
        .map_err(|err| anyhow!("failed to start git rev-parse for Hands commit receipt: {err}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(anyhow!(
            "Hands commit receipt references non-existent commit {commit_sha:?}: {stderr}"
        ));
    }
    let resolved = String::from_utf8(output.stdout)
        .map_err(|err| anyhow!("git rev-parse returned non-UTF8 commit sha: {err}"))?
        .trim()
        .to_string();
    if resolved.is_empty() {
        return Err(anyhow!(
            "git rev-parse returned an empty commit sha for {commit_sha:?}"
        ));
    }
    Ok(resolved)
}

fn generated_receipt_id(prefix: &str) -> String {
    format!("{prefix}-{}", uuid::Uuid::new_v4())
}

fn now() -> String {
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true)
}

fn take_value(tokens: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    tokens
        .next()
        .ok_or_else(|| anyhow!("{name} requires a value"))
}

fn usage() -> &'static str {
    "usage: epiphany-hands-action [--store path] <record-patch|record-command|record-commit|record-pass|record-refusal> (--gate-summary path | --intent-id id --review-id id) --summary text ..."
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_core::EpiphanyRepoModelSeed;
    use epiphany_core::EpiphanyRepoModelSeedDocuments;
    use epiphany_core::HANDS_ACTION_INTENT_SCHEMA_VERSION;
    use epiphany_core::RepoFrontierHandsAuthority;
    use epiphany_core::RuntimeSpineInitOptions;
    use epiphany_core::bind_repository_body;
    use epiphany_core::bind_runtime_to_swarm;
    use epiphany_core::hands_action_review_for_intent;
    use epiphany_core::initialize_keyed_repo_model;
    use epiphany_core::initialize_runtime_spine;
    use epiphany_core::observe_runtime_repository_body_basis;
    use epiphany_core::put_hands_action_intent;
    use epiphany_core::put_hands_action_review;
    use epiphany_core::put_repo_frontier_hands_authority;
    use epiphany_core::runtime_hands_command_receipt;
    use epiphany_core::runtime_hands_commit_receipt;
    use epiphany_core::runtime_hands_patch_receipt;
    use epiphany_core::runtime_latest_hands_receipt_chain_after;
    use epiphany_core::select_and_commit_repo_frontier_route;
    use epiphany_state_model::EpiphanyMemoryDomain;
    use epiphany_state_model::EpiphanyMemoryLifecycle;
    use epiphany_state_model::EpiphanyMemoryNode;
    use epiphany_state_model::EpiphanyMemoryNodeKind;
    use epiphany_state_model::EpiphanyMemoryProfile;
    use epiphany_state_model::RepoFrontierAdoptedPlan;
    use epiphany_state_model::RepoFrontierItem;
    use epiphany_state_model::RepoFrontierStatus;
    use std::process::Command as ProcessCommand;

    #[test]
    fn records_patch_command_and_commit_receipts_against_approved_gate() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime-spine.msgpack");
        seed_gate(&store, vec!["src".to_string()])?;

        record_patch(
            &store,
            RecordPatchArgs {
                gate: gate_args("hands-patch-test"),
                changed_paths: vec!["src/lib.rs".to_string()],
            },
        )?;
        record_command(
            &store,
            RecordCommandArgs {
                gate: gate_args("hands-command-test"),
                command: "cargo test".to_string(),
                exit_code: "0".to_string(),
                stdout_artifact: ".epiphany/stdout.log".to_string(),
                stderr_artifact: ".epiphany/stderr.log".to_string(),
            },
        )?;
        record_commit(
            &store,
            RecordCommitArgs {
                gate: gate_args("hands-commit-test"),
                commit_sha: "abc123".to_string(),
                branch: "codex/test".to_string(),
                changed_paths: vec!["src/lib.rs".to_string()],
                validate_commit_sha: false,
            },
        )?;

        assert!(runtime_hands_patch_receipt(&store, "hands-patch-test")?.is_some());
        assert!(runtime_hands_command_receipt(&store, "hands-command-test")?.is_some());
        assert!(runtime_hands_commit_receipt(&store, "hands-commit-test")?.is_some());
        Ok(())
    }

    #[test]
    fn record_pass_can_load_gate_from_coordinator_summary() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime-spine.msgpack");
        seed_gate(&store, vec!["src".to_string()])?;
        let summary_path = temp.path().join("coordinator-summary.json");
        fs::write(
            &summary_path,
            serde_json::to_string_pretty(&json!({
                "finalAction": {
                    "handsActionGate": {
                        "intentId": "hands-intent-test",
                        "reviewId": "hands-review-test"
                    }
                }
            }))?,
        )?;

        let result = record_pass(
            &store,
            RecordPassArgs {
                gate: GateArgs {
                    intent_id: None,
                    review_id: None,
                    gate_summary: Some(summary_path),
                    receipt_id: None,
                    summary: "implementation pass".to_string(),
                },
                command: "cargo test".to_string(),
                exit_code: "0".to_string(),
                stdout_artifact: ".epiphany/stdout.log".to_string(),
                stderr_artifact: ".epiphany/stderr.log".to_string(),
                commit_sha: "def456".to_string(),
                branch: "codex/test".to_string(),
                changed_paths: vec!["src/lib.rs".to_string()],
                validate_commit_sha: false,
            },
        )?;

        let patch_id = result
            .pointer("/patch/receiptId")
            .and_then(serde_json::Value::as_str)
            .expect("record-pass should emit patch receipt id");
        let command_id = result
            .pointer("/command/receiptId")
            .and_then(serde_json::Value::as_str)
            .expect("record-pass should emit command receipt id");
        let commit_id = result
            .pointer("/commit/receiptId")
            .and_then(serde_json::Value::as_str)
            .expect("record-pass should emit commit receipt id");
        assert!(runtime_hands_patch_receipt(&store, patch_id)?.is_some());
        assert!(runtime_hands_command_receipt(&store, command_id)?.is_some());
        assert!(runtime_hands_commit_receipt(&store, commit_id)?.is_some());

        let chain = runtime_latest_hands_receipt_chain_after(&store, "2026-06-02T00:00:02Z")?
            .expect("record-pass should produce a complete Hands receipt chain");
        assert_eq!(chain.patch_receipt_id, patch_id);
        assert_eq!(chain.command_receipt_id, command_id);
        assert_eq!(chain.commit_receipt_id, commit_id);
        assert_eq!(chain.intent_id, "hands-intent-test");
        assert_eq!(chain.review_id, "hands-review-test");
        assert_eq!(chain.runtime_job_id, "hands-job-test");
        assert_eq!(
            chain.substrate_gate_grant_receipt_id,
            "substrate-grant-test"
        );
        assert_eq!(chain.command, "cargo test");
        assert_eq!(chain.exit_code, "0");
        assert_eq!(chain.commit_sha, "def456");
        assert_eq!(chain.changed_paths, vec!["src/lib.rs".to_string()]);

        Ok(())
    }

    #[test]
    fn refuses_commit_receipt_for_unknown_git_sha() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime-spine.msgpack");
        seed_gate(&store, vec!["src".to_string()])?;

        let result = record_commit(
            &store,
            RecordCommitArgs {
                gate: gate_args("hands-commit-test"),
                commit_sha: "0000000000000000000000000000000000000000".to_string(),
                branch: "codex/test".to_string(),
                changed_paths: vec!["src/lib.rs".to_string()],
                validate_commit_sha: true,
            },
        );

        assert!(result.is_err());
        assert!(runtime_hands_commit_receipt(&store, "hands-commit-test")?.is_none());
        Ok(())
    }

    #[test]
    fn refuses_patch_receipt_outside_requested_paths() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime-spine.msgpack");
        seed_gate(&store, vec!["src".to_string()])?;

        let result = record_patch(
            &store,
            RecordPatchArgs {
                gate: gate_args("hands-patch-test"),
                changed_paths: vec!["notes/other.md".to_string()],
            },
        );

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn refuses_command_when_review_does_not_allow_operation() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("runtime-spine.msgpack");
        seed_gate_with_operations(&store, vec!["src".to_string()], vec!["patch".to_string()])?;

        let result = record_command(
            &store,
            RecordCommandArgs {
                gate: gate_args("hands-command-test"),
                command: "cargo test".to_string(),
                exit_code: "0".to_string(),
                stdout_artifact: ".epiphany/stdout.log".to_string(),
                stderr_artifact: ".epiphany/stderr.log".to_string(),
            },
        );

        assert!(result.is_err());
        Ok(())
    }

    fn seed_gate(store: &PathBuf, requested_paths: Vec<String>) -> Result<()> {
        seed_gate_with_operations(
            store,
            requested_paths,
            vec![
                "patch".to_string(),
                "command".to_string(),
                "commit".to_string(),
                "pr".to_string(),
            ],
        )
    }

    fn seed_gate_with_operations(
        store: &PathBuf,
        requested_paths: Vec<String>,
        operations: Vec<String>,
    ) -> Result<()> {
        initialize_runtime_spine(
            store,
            RuntimeSpineInitOptions {
                runtime_id: "epiphany-hands-action-test".to_string(),
                display_name: "Epiphany Hands Action Test".to_string(),
                created_at: "2026-06-02T00:00:00Z".to_string(),
            },
        )?;
        let fixture_root = store
            .parent()
            .ok_or_else(|| anyhow!("Hands test store has no fixture root"))?;
        let body_store = fixture_root.join("body.cc");
        let repo = fixture_root.join("workspace");
        fs::create_dir_all(repo.join("src"))?;
        fs::write(repo.join("src/lib.rs"), "pub fn fixture() {}\n")?;
        let git = ProcessCommand::new("git")
            .args(["init", "-q"])
            .current_dir(&repo)
            .status()?;
        if !git.success() {
            return Err(anyhow!("Hands test fixture could not initialize Git Body"));
        }
        bind_runtime_to_swarm(store, "hands-action-test-swarm", "2026-06-02T00:00:00.100Z")?;
        bind_repository_body(&repo, &body_store, store, "hands-action-test-workspace")?;
        let body = observe_runtime_repository_body_basis(store)?;
        let domain = EpiphanyMemoryDomain {
            id: "hands-action-domain".into(),
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            title: "Hands action fixture".into(),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        };
        let node = EpiphanyMemoryNode {
            id: "hands-action-node".into(),
            domain_id: domain.id.clone(),
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            kind: EpiphanyMemoryNodeKind::Module,
            title: "Hands action CLI".into(),
            claim: "Hands consequences require exact frontier authority.".into(),
            question: "Can the CLI record the adopted consequence?".into(),
            action_implication: "Exercise the exact typed route.".into(),
            source_hashes: vec!["anchor:missing".into()],
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        };
        let planned_action = if operations == ["patch"] {
            "patch"
        } else {
            "continueImplementation"
        };
        let frontier = RepoFrontierItem {
            id: "hands-action-frontier".into(),
            migration_body: "repo".into(),
            question: "Can Hands record the approved consequence?".into(),
            gap: "The CLI path needs its exact adopted route.".into(),
            target_claim_ids: vec![node.id.clone()],
            repository_scope: requested_paths.clone(),
            recommended_next_organ: "Hands".into(),
            adopted_plan: Some(RepoFrontierAdoptedPlan {
                planning_request_id: "hands-action-planning-request".into(),
                result_id: "hands-action-plan-result".into(),
                job_id: "hands-action-plan-job".into(),
                candidate_id: "hands-action-plan-candidate".into(),
                candidate_sha256:
                    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into(),
                safe_paths: requested_paths.clone(),
                action: planned_action.into(),
                command: "cargo test".into(),
                checks: vec!["cargo test".into()],
                stop_conditions: vec!["A check fails.".into()],
                rollback_steps: vec!["Revert the fixture change.".into()],
                commit_message: "Exercise exact Hands authority".into(),
            }),
            status: RepoFrontierStatus::Active,
            ..Default::default()
        };
        initialize_keyed_repo_model(
            store,
            &EpiphanyRepoModelSeed::new(
                "hands-action-model-seed",
                "hands-action-graph",
                "hands-action-test-swarm",
                "hands-action-test-workspace",
                body.body_binding_sha256,
                EpiphanyRepoModelSeedDocuments {
                    domains: vec![domain],
                    nodes: vec![node],
                    edges: Vec::new(),
                    summaries: Vec::new(),
                    frontier: vec![frontier],
                    lifecycle_receipts: Vec::new(),
                },
            )?,
            "2026-06-02T00:00:00.900Z",
        )?;
        let route = select_and_commit_repo_frontier_route(store, "2026-06-02T00:00:00.950Z")?;
        let grant = epiphany_core::substrate_gate_coordinator_implementation_grant(
            "substrate-grant-test".to_string(),
            "hands-job-test".to_string(),
            requested_paths.clone(),
            "2026-06-02T00:00:00Z".to_string(),
        );
        epiphany_core::put_substrate_gate_repo_access_grant_receipt(store, &grant)?;
        let requested_action = if operations == ["patch"] {
            "patch"
        } else {
            "continueImplementation"
        };
        let intent = HandsActionIntent {
            schema_version: HANDS_ACTION_INTENT_SCHEMA_VERSION.to_string(),
            intent_id: "hands-intent-test".to_string(),
            runtime_job_id: "hands-job-test".to_string(),
            binding_id: "implementation-worker".to_string(),
            role: "epiphany-hands".to_string(),
            authority_scope: "epiphany.role.implementation".to_string(),
            requested_action: requested_action.to_string(),
            requested_paths,
            substrate_gate_grant_receipt_id: "substrate-grant-test".to_string(),
            requested_at: "2026-06-02T00:00:01Z".to_string(),
            contract: "Test Hands intent.".to_string(),
            frontier_route_id: route.route_id.clone(),
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
        put_hands_action_intent(store, &intent)?;
        let review = hands_action_review_for_intent(
            "hands-review-test".to_string(),
            &intent,
            "approved".to_string(),
            operations,
            vec!["test gate".to_string()],
            "2026-06-02T00:00:02Z".to_string(),
        );
        put_hands_action_review(store, &review)?;
        put_repo_frontier_hands_authority(
            store,
            &RepoFrontierHandsAuthority {
                schema_version: epiphany_core::REPO_FRONTIER_HANDS_AUTHORITY_SCHEMA_VERSION.into(),
                authority_id: "repo-frontier-hands-authority-test".into(),
                route_id: route.route_id,
                model_projection_digest: route.model_projection_digest,
                model_source_documents: route.model_source_documents,
                frontier_item_id: route.frontier_item_id,
                frontier_item_hash: route.frontier_item_hash,
                hands_intent_id: intent.intent_id,
                hands_review_id: review.review_id,
                substrate_grant_receipt_id: grant.receipt_id,
                requested_paths: route.authorized_paths,
                granted_at: review.reviewed_at,
                contract: epiphany_core::REPO_FRONTIER_HANDS_AUTHORITY_CONTRACT.into(),
            },
        )?;
        Ok(())
    }

    fn gate_args(receipt_id: &str) -> GateArgs {
        GateArgs {
            intent_id: Some("hands-intent-test".to_string()),
            review_id: Some("hands-review-test".to_string()),
            gate_summary: None,
            receipt_id: Some(receipt_id.to_string()),
            summary: "test receipt".to_string(),
        }
    }
}
