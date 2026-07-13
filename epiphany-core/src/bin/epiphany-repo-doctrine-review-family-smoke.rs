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
}

impl Args {
    fn parse() -> Result<Self> {
        let mut root = env::current_dir().context("failed to resolve current directory")?;
        let mut args = env::args().skip(1).peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--root" => root = take_path(&mut args, "--root")?,
                other => return Err(anyhow!("unexpected argument {other:?}")),
            }
        }
        Ok(Self { root })
    }
}

fn run_smoke(args: Args) -> Result<Value> {
    let root = args
        .root
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.root.display()))?;
    let smoke_root = root.join(".epiphany-smoke");
    let manifest = root.join("epiphany-core").join("Cargo.toml");
    if !manifest.exists() {
        return Err(anyhow!(
            "could not find epiphany-core manifest at {}",
            manifest.display()
        ));
    }
    fs::create_dir_all(&smoke_root)
        .with_context(|| format!("failed to create {}", smoke_root.display()))?;
    let stamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let smoke_dir = smoke_root.join(format!("repo-doctrine-update-request-family-{stamp}"));
    fs::create_dir(&smoke_dir)
        .with_context(|| format!("failed to claim fresh smoke dir {}", smoke_dir.display()))?;

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
        "# Repo Doctrine Update Request Family Smoke\n\nThis repository proves branch-local doctrine update request cargo without direct doctrine mutation authority.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    fs::write(
        repo.join("AGENTS.md"),
        "# Repo Agent Doctrine\n\nExisting doctrine stays untouched by the request artifact.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("AGENTS.md").display()))?;
    git(["add", "README.md", "AGENTS.md"], &repo)?;
    git(
        [
            "commit",
            "-m",
            "Seed repo doctrine update request smoke body",
        ],
        &repo,
    )?;
    git(
        [
            "switch",
            "-c",
            "epiphany/repo-doctrine-update-request-family",
        ],
        &repo,
    )?;

    let item = "repo-doctrine-update-request-family";
    let target_path = ".epiphany/doctrine-update-requests/repo-doctrine-update-request-family.toml";
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
            "repo-doctrine-update-request-family-smoke",
            "--topic",
            "repo-doctrine-update-request-family",
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
            "repo-doctrine-update-request-family-smoke",
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
            "Ask for a reviewed AGENTS.md doctrine update while preserving Mind, Soul, and maintainer gates.",
            "--candidate-action-ref",
            "candidate-action://repo-doctrine-update-request-family/agents-update",
            "--public-discussion-ref",
            "epiphany-global/persona-collaboration/repo-doctrine-update-request-family",
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
            "repo-doctrine-update-request",
            "--model-ref",
            "repo-doctrine-update-request-family-smoke-imagination-v1",
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

    let close = read_json(
        &repo
            .join(".epiphany")
            .join("work")
            .join(format!("work-close-{item}.json")),
    )?;
    let request_text = fs::read_to_string(repo.join(target_path))
        .with_context(|| format!("failed to read {}", repo.join(target_path).display()))?;
    let agents_text = fs::read_to_string(repo.join("AGENTS.md"))
        .with_context(|| format!("failed to read {}", repo.join("AGENTS.md").display()))?;

    require_eq(
        &plan,
        &["derivation", "safeActionFamily"],
        "repo.doctrine_update_request",
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
        "schema_version = \"epiphany.repo_doctrine_update_request.v0\"",
    )?;
    require_text(
        &request_text,
        "safe_action_family = \"repo.doctrine_update_request\"",
    )?;
    require_text(&request_text, "[request]")?;
    require_text(&request_text, "status = \"awaiting-doctrine-review\"")?;
    require_text(&request_text, "routing_owner = \"Self\"")?;
    require_text(
        &request_text,
        "required_reviewers = [\"Maintainer\", \"Mind\", \"Soul\"]",
    )?;
    require_text(&request_text, "doctrine_admission_owner = \"Mind\"")?;
    require_text(&request_text, "mutation_owner = \"Hands\"")?;
    require_text(
        &request_text,
        "requested_effect = \"review-repo-agent-doctrine-update\"",
    )?;
    require_text(&request_text, "doctrine_target = \"AGENTS.md\"")?;
    require_text(&request_text, "requires_source_grounding = true")?;
    require_text(&request_text, "requires_human_or_maintainer_review = true")?;
    require_text(&request_text, "[antecedents]")?;
    require_text(&request_text, "persona_or_human_feedback_required = true")?;
    require_text(&request_text, "imagination_plan_required = true")?;
    require_text(&request_text, "mind_adoption_required = true")?;
    require_text(&request_text, "soul_review_required = true")?;
    require_text(&request_text, "maintainer_review_required = true")?;
    require_text(&request_text, "[required_receipts]")?;
    require_text(
        &request_text,
        "imagination_plan = \"epiphany.repo_work_imagination_action_items_receipt.v0\"",
    )?;
    require_text(
        &request_text,
        "mind_adoption = \"epiphany.repo_work_mind_adoption_decision.v0\"",
    )?;
    require_text(
        &request_text,
        "soul_review = \"epiphany.repo_work_closure_review.v0\"",
    )?;
    require_text(
        &request_text,
        "maintainer_review = \"gamecult.maintainer.review_receipt.v0\"",
    )?;
    require_text(
        &request_text,
        "hands_commit = \"epiphany.hands.commit_receipt\"",
    )?;
    require_text(&request_text, "[doctrine_packet]")?;
    require_text(&request_text, "requires_current_doctrine_ref = true")?;
    require_text(&request_text, "requires_proposed_change_summary = true")?;
    require_text(&request_text, "requires_invariant_impact = true")?;
    require_text(&request_text, "requires_rehydration_impact = true")?;
    require_text(&request_text, "requires_rollback_plan = true")?;
    require_text(
        &request_text,
        "requires_private_state_redaction_check = true",
    )?;
    require_text(&request_text, "[authority]")?;
    require_text(&request_text, "direct_doctrine_mutation_authority = false")?;
    require_text(&request_text, "direct_hands_authority = false")?;
    require_text(&request_text, "direct_mind_state_commit = false")?;
    require_text(&request_text, "publication_authorized = false")?;
    require_text(&request_text, "merge_authorized = false")?;
    require_text(&request_text, "service_lifecycle_authority = false")?;
    require_text(&request_text, "cross_body_mutation_authorized = false")?;
    require_text(&request_text, "private_verse_rummaging = false")?;
    require_text(&request_text, "maintainer_review_required = true")?;
    require_text(&request_text, "mind_admission_required = true")?;
    require_text(&request_text, "hands_receipts_required = true")?;
    require_text(&request_text, "private_state_exposed = false")?;
    require_text(
        &agents_text,
        "Existing doctrine stays untouched by the request artifact.",
    )?;

    let summary = json!({
        "schemaVersion": "epiphany.repo_doctrine_update_request_family_smoke.v0",
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
        "awaitingDoctrineReview": true,
        "doctrineMutationAuthorized": false,
        "handsAuthorityGranted": false,
        "publicationAuthorized": false,
        "mergeAuthorized": false,
        "serviceLifecycleAuthority": false,
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
            "expected doctrine update request text to contain {needle:?}"
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
