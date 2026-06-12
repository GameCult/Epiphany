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
use epiphany_core::runtime_hands_action_intent;
use epiphany_core::runtime_hands_action_review;
use serde_json::json;
use std::env;
use std::path::PathBuf;

const DEFAULT_STORE: &str = "state/runtime-spine.msgpack";

fn main() -> Result<()> {
    let args = Args::parse()?;
    let output = match args.command {
        Command::RecordPatch(command) => record_patch(&args.store, command)?,
        Command::RecordCommand(command) => record_command(&args.store, command)?,
        Command::RecordCommit(command) => record_commit(&args.store, command)?,
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
}

#[derive(Debug)]
struct GateArgs {
    intent_id: String,
    review_id: String,
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
            _ => return Err(anyhow!("unknown command {command_name}\n{}", usage())),
        };
        Ok(Self { store, command })
    }
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
        Ok(GateArgs {
            intent_id: self.take_required("--intent-id")?,
            review_id: self.take_required("--review-id")?,
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
    let receipt_id = args
        .gate
        .receipt_id
        .unwrap_or_else(|| generated_receipt_id("hands-commit"));
    let receipt = hands_commit_receipt_for_review(
        receipt_id,
        &intent,
        &review,
        args.commit_sha,
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

fn load_gate(
    store: &PathBuf,
    gate: &GateArgs,
    operation: &str,
) -> Result<(HandsActionIntent, HandsActionReview)> {
    let intent = runtime_hands_action_intent(store, &gate.intent_id)
        .with_context(|| format!("failed to load Hands intent {}", gate.intent_id))?
        .ok_or_else(|| anyhow!("Hands intent {} was not found", gate.intent_id))?;
    let review = runtime_hands_action_review(store, &gate.review_id)
        .with_context(|| format!("failed to load Hands review {}", gate.review_id))?
        .ok_or_else(|| anyhow!("Hands review {} was not found", gate.review_id))?;
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
    "usage: epiphany-hands-action [--store path] <record-patch|record-command|record-commit> --intent-id id --review-id id --summary text ..."
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_core::HANDS_ACTION_INTENT_SCHEMA_VERSION;
    use epiphany_core::RuntimeSpineInitOptions;
    use epiphany_core::hands_action_review_for_intent;
    use epiphany_core::initialize_runtime_spine;
    use epiphany_core::put_hands_action_intent;
    use epiphany_core::put_hands_action_review;
    use epiphany_core::runtime_hands_command_receipt;
    use epiphany_core::runtime_hands_commit_receipt;
    use epiphany_core::runtime_hands_patch_receipt;

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
            },
        )?;

        assert!(runtime_hands_patch_receipt(&store, "hands-patch-test")?.is_some());
        assert!(runtime_hands_command_receipt(&store, "hands-command-test")?.is_some());
        assert!(runtime_hands_commit_receipt(&store, "hands-commit-test")?.is_some());
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
        seed_gate_with_operations(&store, vec![".".to_string()], vec!["patch".to_string()])?;

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
        let intent = HandsActionIntent {
            schema_version: HANDS_ACTION_INTENT_SCHEMA_VERSION.to_string(),
            intent_id: "hands-intent-test".to_string(),
            runtime_job_id: "hands-job-test".to_string(),
            binding_id: "implementation-worker".to_string(),
            role: "epiphany-hands".to_string(),
            authority_scope: "epiphany.role.implementation".to_string(),
            requested_action: "continueImplementation".to_string(),
            requested_paths,
            substrate_gate_grant_receipt_id: "substrate-grant-test".to_string(),
            requested_at: "2026-06-02T00:00:01Z".to_string(),
            contract: "Test Hands intent.".to_string(),
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
        Ok(())
    }

    fn gate_args(receipt_id: &str) -> GateArgs {
        GateArgs {
            intent_id: "hands-intent-test".to_string(),
            review_id: "hands-review-test".to_string(),
            receipt_id: Some(receipt_id.to_string()),
            summary: "test receipt".to_string(),
        }
    }
}
