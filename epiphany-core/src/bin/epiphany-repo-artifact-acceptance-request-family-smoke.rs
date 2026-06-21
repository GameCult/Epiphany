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
    let args = Args::parse()?;
    let result = run_smoke(args)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[derive(Clone, Debug)]
struct Args {
    root: PathBuf,
    smoke_root: PathBuf,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut root = env::current_dir().context("failed to resolve current directory")?;
        let mut smoke_root = root.join(".epiphany-smoke");
        let mut args = env::args().skip(1).peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--root" => root = take_path(&mut args, "--root")?,
                "--smoke-root" => smoke_root = take_path(&mut args, "--smoke-root")?,
                other => return Err(anyhow!("unexpected argument {other:?}")),
            }
        }
        Ok(Self { root, smoke_root })
    }
}

fn run_smoke(args: Args) -> Result<Value> {
    let root = args
        .root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.root.display()))?;
    let manifest = root.join("epiphany-core").join("Cargo.toml");
    if !manifest.exists() {
        return Err(anyhow!(
            "could not find epiphany-core manifest at {}",
            manifest.display()
        ));
    }
    fs::create_dir_all(&args.smoke_root)
        .with_context(|| format!("failed to create {}", args.smoke_root.display()))?;
    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let smoke_dir = args
        .smoke_root
        .join(format!("repo-artifact-acceptance-request-family-{stamp}"));
    if smoke_dir.exists() {
        fs::remove_dir_all(&smoke_dir)
            .with_context(|| format!("failed to clear {}", smoke_dir.display()))?;
    }
    fs::create_dir_all(&smoke_dir)
        .with_context(|| format!("failed to create {}", smoke_dir.display()))?;

    let repo = smoke_dir.join("repo-body");
    fs::create_dir_all(&repo).with_context(|| format!("failed to create {}", repo.display()))?;
    git(["init"], &repo)?;
    git(
        ["config", "user.email", "epiphany-smoke@example.invalid"],
        &repo,
    )?;
    git(["config", "user.name", "Epiphany Smoke"], &repo)?;
    fs::write(
        repo.join("README.md"),
        "# Repo Artifact Acceptance Request Family Smoke\n\nThis repository proves branch-local artifact acceptance request cargo without acceptance authority.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    git(["add", "README.md"], &repo)?;
    git(
        [
            "commit",
            "-m",
            "Seed repo artifact acceptance request smoke body",
        ],
        &repo,
    )?;
    git(
        [
            "switch",
            "-c",
            "epiphany/repo-artifact-acceptance-request-family",
        ],
        &repo,
    )?;

    let item = "repo-artifact-acceptance-request-family";
    let target_path =
        ".epiphany/artifact-acceptance-requests/repo-artifact-acceptance-request-family.toml";
    let local_verse = repo.join(".epiphany").join("local-verse.ccmp");
    cargo_json(
        &manifest,
        "epiphany-repo",
        &[
            "init",
            "--workspace",
            path_str(&repo)?,
            "--epiphany-root",
            path_str(&root)?,
            "--swarm-id",
            "repo-artifact-acceptance-request-family-smoke",
            "--topic",
            "repo-artifact-acceptance-request-family",
        ],
        &root,
    )?;
    cargo_json(
        &manifest,
        "epiphany-swarm",
        &[
            "online",
            "--workspace",
            path_str(&repo)?,
            "--epiphany-root",
            path_str(&root)?,
            "--runtime-id",
            "repo-artifact-acceptance-request-family-smoke",
            "--local-verse-store",
            path_str(&local_verse)?,
        ],
        &root,
    )?;
    cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "accept",
            "--workspace",
            path_str(&repo)?,
            "--from",
            "persona",
            "--item",
            item,
            "--summary",
            "Ask a maintainer and Bifrost to record artifact acceptance for reviewed branch work without granting acceptance authority.",
            "--candidate-action-ref",
            "candidate-action://repo-artifact-acceptance-request-family/artifact-packet",
            "--candidate-action-ref",
            "candidate-action://repo-artifact-acceptance-request-family/acceptance-review",
            "--public-discussion-ref",
            "epiphany-global/persona-collaboration/repo-artifact-acceptance-request-family",
        ],
        &root,
    )?;
    let plan = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "derive-plan",
            "--workspace",
            path_str(&repo)?,
            "--item",
            item,
            "--action-family",
            "repo-artifact-acceptance-request",
            "--model-ref",
            "repo-artifact-acceptance-request-family-smoke-imagination-v1",
            "--model-authored",
        ],
        &root,
    )?;
    for _ in 0..4 {
        cargo_json(
            &manifest,
            "epiphany-work",
            &[
                "tick",
                "--workspace",
                path_str(&repo)?,
                "--epiphany-root",
                path_str(&root)?,
                "--item",
                item,
                "--cooldown-seconds",
                "0",
            ],
            &root,
        )?;
    }
    let close_path = repo
        .join(".epiphany")
        .join("work")
        .join(format!("work-close-{item}.json"));
    let close = read_json(&close_path)?;
    let request_text = fs::read_to_string(repo.join(target_path))
        .with_context(|| format!("failed to read {}", repo.join(target_path).display()))?;
    let open_bifrost_ledger = cargo_json(
        &manifest,
        "epiphany-verse-query",
        &[
            "bifrost-ledger",
            "--store",
            path_str(&local_verse)?,
            "--runtime-id",
            "repo-artifact-acceptance-request-family-smoke",
        ],
        &root,
    )?;
    let artifact_acceptance_receipt = cargo_json(
        &manifest,
        "epiphany-verse-query",
        &[
            "bifrost-artifact-acceptance",
            "--store",
            path_str(&local_verse)?,
            "--runtime-id",
            "repo-artifact-acceptance-request-family-smoke",
            "--artifact-ref",
            "artifact://repo-artifact-acceptance-request-family/final",
            "--ledger-entry-id",
            "bifrost-ledger:repo-artifact-acceptance-request-family",
            "--review-receipt",
            "maintainer-review:repo-artifact-acceptance-request-family",
            "--public-proof-ref",
            "public-proof:repo-artifact-acceptance-request-family",
        ],
        &root,
    )?;
    let closed_bifrost_ledger = cargo_json(
        &manifest,
        "epiphany-verse-query",
        &[
            "bifrost-ledger",
            "--store",
            path_str(&local_verse)?,
            "--runtime-id",
            "repo-artifact-acceptance-request-family-smoke",
        ],
        &root,
    )?;

    require_eq(
        &plan,
        &["derivation", "safeActionFamily"],
        "repo.artifact_acceptance_request",
    )?;
    require_eq(&close, &["status"], "closed")?;
    require_eq(&close, &["soul", "verdict"], "passed")?;
    require_eq(
        &close,
        &["closureReview", "familyAssertions", "status"],
        "passed",
    )?;
    require_bool(
        &close,
        &["closureReview", "sourceGrounding", "pathScopeMatched"],
        true,
    )?;
    require_bool(&close, &["privateStateExposed"], false)?;
    require_text(
        &request_text,
        "schema_version = \"epiphany.repo_artifact_acceptance_request.v0\"",
    )?;
    require_text(
        &request_text,
        "safe_action_family = \"repo.artifact_acceptance_request\"",
    )?;
    require_text(&request_text, "[request]")?;
    require_text(
        &request_text,
        "status = \"awaiting-artifact-acceptance-review\"",
    )?;
    require_text(&request_text, "requested_owner = \"Maintainer/Bifrost\"")?;
    require_text(
        &request_text,
        "requested_effect = \"record-accepted-artifact-for-reviewed-branch-work\"",
    )?;
    require_text(&request_text, "verification_request_ref = ")?;
    require_text(&request_text, "maintainer_review_request_ref = ")?;
    require_text(&request_text, "publication_request_ref = ")?;
    require_text(
        &request_text,
        "acceptance_scope = \"changed paths, Hands commit, Soul closure, maintainer review, public proof, and accepted artifact readback\"",
    )?;
    require_text(&request_text, "[antecedents]")?;
    require_text(&request_text, "closure_review_required = true")?;
    require_text(&request_text, "soul_verdict_required = true")?;
    require_text(&request_text, "mind_commit_required = true")?;
    require_text(&request_text, "public_proof_required = true")?;
    require_text(&request_text, "maintainer_review_required = true")?;
    require_text(&request_text, "hands_commit_required = true")?;
    require_text(&request_text, "[required_receipts]")?;
    require_text(
        &request_text,
        "closure_review = \"epiphany.repo_work_closure_review.v0\"",
    )?;
    require_text(
        &request_text,
        "soul_verdict = \"epiphany.soul.verification_verdict\"",
    )?;
    require_text(
        &request_text,
        "mind_commit = \"epiphany.mind.state_commit_receipt\"",
    )?;
    require_text(
        &request_text,
        "public_proof = \"epiphany.repo_work_public_proof_bundle.v0\"",
    )?;
    require_text(
        &request_text,
        "maintainer_review = \"gamecult.maintainer.review_receipt.v0\"",
    )?;
    require_text(
        &request_text,
        "hands_commit = \"epiphany.hands.commit_receipt\"",
    )?;
    require_text(
        &request_text,
        "accepted_artifact = \"gamecult.artifact.acceptance_receipt.v0\"",
    )?;
    require_text(&request_text, "[artifact_packet]")?;
    require_text(&request_text, "requires_artifact_ref = true")?;
    require_text(&request_text, "requires_commit_sha = true")?;
    require_text(&request_text, "requires_changed_path_list = true")?;
    require_text(&request_text, "requires_review_verdict = true")?;
    require_text(&request_text, "requires_public_proof_ref = true")?;
    require_text(&request_text, "requires_acceptance_rationale = true")?;
    require_text(
        &request_text,
        "requires_private_state_redaction_check = true",
    )?;
    require_text(&request_text, "artifact_acceptance_authorized = false")?;
    require_text(&request_text, "credit_ledger_authorized = false")?;
    require_text(&request_text, "github_pr_authorized = false")?;
    require_text(&request_text, "merge_authorized = false")?;
    require_text(&request_text, "publication_authorized = false")?;
    require_text(&request_text, "upstream_sync_authorized = false")?;
    require_text(&request_text, "hands_action_authorized = false")?;
    require_text(&request_text, "cross_body_mutation_authorized = false")?;
    require_text(
        &request_text,
        "maintainer_or_bifrost_acceptance_authority_required = true",
    )?;
    require_text(&request_text, "private_state_exposed = false")?;
    require_bifrost_accounting_row(
        &open_bifrost_ledger,
        "artifact-acceptance-request",
        "open",
        3,
        1,
    )?;
    require_tui_row_contains(
        &open_bifrost_ledger,
        &["accountingTuiRows"],
        "BIFROST-ACCOUNTING | artifact-acceptance-request",
    )?;
    require_tui_row_contains(&open_bifrost_ledger, &["accountingTuiRows"], "status=open")?;
    require_tui_row_contains(
        &open_bifrost_ledger,
        &["accountingTuiRows"],
        "request=present",
    )?;
    require_bool(&open_bifrost_ledger, &["privateStateExposed"], false)?;
    require_eq(
        &artifact_acceptance_receipt,
        &["receiptId"],
        "bifrost-artifact-acceptance-repo-artifact-acceptance-request-family",
    )?;
    require_eq(&artifact_acceptance_receipt, &["status"], "ok")?;
    require_bool(
        &artifact_acceptance_receipt,
        &["privateStateExposed"],
        false,
    )?;
    require_bifrost_accounting_row(
        &closed_bifrost_ledger,
        "artifact-acceptance-request",
        "closed",
        1,
        1,
    )?;
    require_tui_row_contains(
        &closed_bifrost_ledger,
        &["accountingTuiRows"],
        "acceptance=present",
    )?;
    require_tui_row_contains(
        &closed_bifrost_ledger,
        &["accountingTuiRows"],
        "receipt=bifrost-artifact-acceptance-repo-artifact-acceptance-request-family",
    )?;
    require_bool(&closed_bifrost_ledger, &["privateStateExposed"], false)?;

    let summary = json!({
        "schemaVersion": "epiphany.repo_artifact_acceptance_request_family_smoke.v0",
        "status": "ok",
        "smokeDir": smoke_dir,
        "repo": repo,
        "branch": git_output(["branch", "--show-current"], &repo)?,
        "item": item,
        "safeActionFamily": plan["derivation"]["safeActionFamily"],
        "targetPath": target_path,
        "closeStatus": close["status"],
        "soulVerdict": close["soul"]["verdict"],
        "familyAssertionsStatus": close["closureReview"]["familyAssertions"]["status"],
        "pathScopeMatched": close["closureReview"]["sourceGrounding"]["pathScopeMatched"],
        "awaitingArtifactAcceptanceReview": true,
        "artifactAcceptanceAuthorized": false,
        "creditLedgerAuthorized": false,
        "githubPrAuthorized": false,
        "mergeAuthorized": false,
        "publicationAuthorized": false,
        "upstreamSyncAuthorized": false,
        "handsActionAuthorized": false,
        "bifrostLedgerAccountingRowCount": closed_bifrost_ledger["accountingRowCount"],
        "bifrostArtifactAcceptanceReceiptId": artifact_acceptance_receipt["receiptId"],
        "bifrostArtifactAcceptanceOpenAccountingRow": bifrost_accounting_row_value(&open_bifrost_ledger, "artifact-acceptance-request")?,
        "bifrostArtifactAcceptanceClosedAccountingRow": bifrost_accounting_row_value(&closed_bifrost_ledger, "artifact-acceptance-request")?,
        "privateStateExposed": false,
    });
    write_json(&smoke_dir.join("summary.json"), &summary)?;
    Ok(summary)
}

fn cargo_json(manifest: &Path, bin: &str, args: &[&str], cwd: &Path) -> Result<Value> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(manifest)
        .arg("--bin")
        .arg(bin)
        .arg("--")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to run cargo bin {bin}"))?;
    if !output.status.success() {
        return Err(anyhow!(
            "{bin} failed with status {:?}\nstdout:\n{}\nstderr:\n{}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "{bin} did not emit JSON on stdout:\n{}",
            String::from_utf8_lossy(&output.stdout)
        )
    })
}

fn git<const N: usize>(args: [&str; N], cwd: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to run git in {}", cwd.display()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "git failed in {} with status {:?}\nstdout:\n{}\nstderr:\n{}",
            cwd.display(),
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn git_output<const N: usize>(args: [&str; N], cwd: &Path) -> Result<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to run git in {}", cwd.display()))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(anyhow!(
            "git failed in {} with status {:?}\nstdout:\n{}\nstderr:\n{}",
            cwd.display(),
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
}

fn read_json(path: &Path) -> Result<Value> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("failed to parse {}", path.display()))
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn require_eq(value: &Value, path: &[&str], expected: &str) -> Result<()> {
    let actual = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_str)
        .unwrap_or("<missing>");
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to be {expected:?}, got {actual:?}",
            path.join(".")
        ))
    }
}

fn require_bool(value: &Value, path: &[&str], expected: bool) -> Result<()> {
    let actual = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_bool);
    if actual == Some(expected) {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to be {expected}, got {:?}",
            path.join("."),
            actual
        ))
    }
}

fn require_text(haystack: &str, needle: &str) -> Result<()> {
    if haystack.contains(needle) {
        Ok(())
    } else {
        Err(anyhow!(
            "expected artifact acceptance request text to contain {needle:?}"
        ))
    }
}

fn require_tui_row_contains(value: &Value, path: &[&str], needle: &str) -> Result<()> {
    let rows = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("missing TUI row array {}", path.join(".")))?;
    if rows
        .iter()
        .filter_map(Value::as_str)
        .any(|row| row.contains(needle))
    {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to contain row fragment {needle:?}",
            path.join(".")
        ))
    }
}

fn bifrost_accounting_row_value<'a>(value: &'a Value, lane: &str) -> Result<&'a Value> {
    value
        .get("accountingRows")
        .and_then(Value::as_array)
        .and_then(|rows| {
            rows.iter()
                .find(|row| row.get("lane").and_then(Value::as_str) == Some(lane))
        })
        .ok_or_else(|| anyhow!("missing Bifrost accounting row {lane:?}"))
}

fn require_bifrost_accounting_row(
    value: &Value,
    lane: &str,
    status: &str,
    review_receipt_count: u64,
    public_artifact_count: u64,
) -> Result<()> {
    let row = bifrost_accounting_row_value(value, lane)?;
    let actual_status = row
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("<missing>");
    let actual_review_receipt_count = row
        .get("reviewReceiptCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let actual_public_artifact_count = row
        .get("publicArtifactCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let actual_private = row
        .get("privateStateExposed")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    if actual_status == status
        && actual_review_receipt_count == review_receipt_count
        && actual_public_artifact_count == public_artifact_count
        && !actual_private
    {
        Ok(())
    } else {
        Err(anyhow!(
            "Bifrost accounting row {lane:?} mismatch: status={actual_status:?}, reviewReceiptCount={actual_review_receipt_count}, publicArtifactCount={actual_public_artifact_count}, privateStateExposed={actual_private}"
        ))
    }
}

fn take_path(
    args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    flag: &str,
) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .with_context(|| format!("missing value for {flag}"))
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| anyhow!("path is not valid UTF-8: {}", path.display()))
}
