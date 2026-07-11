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
        .join(format!("repo-readiness-review-request-family-{stamp}"));
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
        "# Repo Readiness Review Request Family Smoke\n\nThis repository proves branch-local MVP readiness review request cargo without publication, merge, deployment, service, or state authority.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    git(["add", "README.md"], &repo)?;
    git(
        [
            "commit",
            "-m",
            "Seed repo readiness review request smoke body",
        ],
        &repo,
    )?;
    git(
        [
            "switch",
            "-c",
            "epiphany/repo-readiness-review-request-family",
        ],
        &repo,
    )?;

    let item = "repo-readiness-review-request-family";
    let target_path = ".epiphany/readiness-reviews/repo-readiness-review-request-family.toml";
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
            "repo-readiness-review-request-family-smoke",
            "--topic",
            "repo-readiness-review-request-family",
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
            "repo-readiness-review-request-family-smoke",
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
            "Ask Maintainer, Soul, Mind, and Bifrost to review a redacted repo-swarm MVP proof bundle without granting publication or deployment authority.",
            "--candidate-action-ref",
            "candidate-action://repo-readiness-review-request-family/readiness-packet",
            "--candidate-action-ref",
            "candidate-action://repo-readiness-review-request-family/proof-bundle-review",
            "--public-discussion-ref",
            "epiphany-global/persona-collaboration/repo-readiness-review-request-family",
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
            "repo-readiness-review-request",
            "--model-ref",
            "repo-readiness-review-request-family-smoke-imagination-v1",
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

    require_eq(
        &plan,
        &["derivation", "safeActionFamily"],
        "repo.readiness_review_request",
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
        "schema_version = \"epiphany.repo_readiness_review_request.v0\"",
    )?;
    require_text(
        &request_text,
        "safe_action_family = \"repo.readiness_review_request\"",
    )?;
    require_text(&request_text, "[request]")?;
    require_text(&request_text, "status = \"awaiting-mvp-readiness-review\"")?;
    require_text(
        &request_text,
        "requested_owner = \"Maintainer/Soul/Mind/Bifrost\"",
    )?;
    require_text(
        &request_text,
        "requested_effect = \"review-redacted-repo-swarm-mvp-proof-bundle\"",
    )?;
    require_text(
        &request_text,
        "readiness_scope = \"fresh repo init, online swarm, Persona intake, Imagination planning, Self queue-run, Hands branch work, Soul closure, Modeling map update, Mind admission, Bifrost public proof, upstream-main sync, Idunn lifecycle, global tool directory, and private-state redaction\"",
    )?;
    require_text(
        &request_text,
        "review_is_advisory_until_maintainer_or_bifrost_acceptance = true",
    )?;
    require_text(&request_text, "[antecedents]")?;
    require_text(&request_text, "repo_init_required = true")?;
    require_text(&request_text, "swarm_online_required = true")?;
    require_text(&request_text, "persona_intake_required = true")?;
    require_text(&request_text, "imagination_plan_required = true")?;
    require_text(&request_text, "self_queue_run_required = true")?;
    require_text(&request_text, "hands_commit_required = true")?;
    require_text(&request_text, "soul_closure_required = true")?;
    require_text(&request_text, "modeling_map_update_required = true")?;
    require_text(&request_text, "mind_admission_required = true")?;
    require_text(&request_text, "public_proof_required = true")?;
    require_text(&request_text, "bifrost_publication_required = true")?;
    require_text(&request_text, "upstream_main_sync_required = true")?;
    require_text(&request_text, "idunn_lifecycle_readiness_required = true")?;
    require_text(&request_text, "tool_directory_readiness_required = true")?;
    require_text(&request_text, "private_state_redaction_required = true")?;
    require_text(&request_text, "[required_receipts]")?;
    require_text(
        &request_text,
        "repo_init = \"epiphany.repo_swarm_init_receipt.v0\"",
    )?;
    require_text(
        &request_text,
        "swarm_online = \"epiphany.repo_swarm_online_receipt.v0\"",
    )?;
    require_text(
        &request_text,
        "persona_speech_audit = \"epiphany.persona_speech_audit.v0\"",
    )?;
    require_text(
        &request_text,
        "imagination_action_items = \"epiphany.repo_work_imagination_action_items_receipt.v0\"",
    )?;
    require_text(
        &request_text,
        "queue_run = \"epiphany.repo_work_queue_run_receipt.v0\"",
    )?;
    require_text(
        &request_text,
        "hands_commit = \"epiphany.hands.commit_receipt\"",
    )?;
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
        "modeling_map = \"epiphany.repo_work_map_entry.v0\"",
    )?;
    require_text(
        &request_text,
        "bifrost_publication = \"gamecult.bifrost.public_proof_publication_receipt.v0\"",
    )?;
    require_text(
        &request_text,
        "upstream_sync = \"epiphany.repo_work_upstream_sync_receipt.v0\"",
    )?;
    require_text(
        &request_text,
        "idunn_lifecycle = \"epiphany.cultmesh.daemon_service_lifecycle_receipt.v0\"",
    )?;
    require_text(
        &request_text,
        "tool_directory = \"epiphany.cultmesh.daemon_tool_directory_readback.v0\"",
    )?;
    require_text(&request_text, "[readiness_packet]")?;
    require_text(&request_text, "requires_proof_bundle_ref = true")?;
    require_text(&request_text, "requires_changed_path_list = true")?;
    require_text(&request_text, "requires_branch_name = true")?;
    require_text(&request_text, "requires_upstream_main_ref = true")?;
    require_text(&request_text, "requires_public_proof_ref = true")?;
    require_text(&request_text, "requires_bifrost_ledger_ref = true")?;
    require_text(&request_text, "requires_idunn_lifecycle_ref = true")?;
    require_text(&request_text, "requires_tool_directory_ref = true")?;
    require_text(&request_text, "requires_redaction_report = true")?;
    require_text(&request_text, "requires_reviewer_identity = true")?;
    require_text(
        &request_text,
        "allowed_verdicts = [\"ready\", \"ready-with-caveats\", \"not-ready\", \"needs-human-review\"]",
    )?;
    require_text(&request_text, "readiness_approval_authorized = false")?;
    require_text(&request_text, "durable_state_commit_authorized = false")?;
    require_text(&request_text, "publication_authorized = false")?;
    require_text(&request_text, "bifrost_publication_authorized = false")?;
    require_text(&request_text, "github_pr_authorized = false")?;
    require_text(&request_text, "merge_authorized = false")?;
    require_text(&request_text, "upstream_sync_authorized = false")?;
    require_text(&request_text, "deployment_authority = false")?;
    require_text(&request_text, "service_lifecycle_authority = false")?;
    require_text(&request_text, "hands_action_authorized = false")?;
    require_text(&request_text, "cross_body_mutation_authorized = false")?;
    require_text(&request_text, "private_verse_rummaging = false")?;
    require_text(
        &request_text,
        "maintainer_soul_mind_or_bifrost_review_required = true",
    )?;
    require_text(&request_text, "private_state_exposed = false")?;

    let summary = json!({
        "schemaVersion": "epiphany.repo_readiness_review_request_family_smoke.v0",
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
        "awaitingMvpReadinessReview": true,
        "repoInitRequired": true,
        "swarmOnlineRequired": true,
        "personaIntakeRequired": true,
        "imaginationPlanRequired": true,
        "selfQueueRunRequired": true,
        "handsCommitRequired": true,
        "soulClosureRequired": true,
        "modelingMapUpdateRequired": true,
        "mindAdmissionRequired": true,
        "bifrostPublicationRequired": true,
        "upstreamMainSyncRequired": true,
        "idunnLifecycleRequired": true,
        "toolDirectoryRequired": true,
        "readinessApprovalAuthorized": false,
        "durableStateCommitAuthorized": false,
        "githubPrAuthorized": false,
        "mergeAuthorized": false,
        "publicationAuthorized": false,
        "deploymentAuthority": false,
        "serviceLifecycleAuthority": false,
        "upstreamSyncAuthorized": false,
        "handsActionAuthorized": false,
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
            "expected readiness review request text to contain {needle:?}"
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
