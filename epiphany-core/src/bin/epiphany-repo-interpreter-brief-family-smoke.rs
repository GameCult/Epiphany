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
        .join(format!("repo-interpreter-brief-family-{stamp}"));
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
        "# Repo Interpreter Brief Family Smoke\n\nThis repository proves branch-local Mind-owned interpretation briefs.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    git(["add", "README.md"], &repo)?;
    git(
        ["commit", "-m", "Seed repo interpreter brief smoke body"],
        &repo,
    )?;
    git(
        ["switch", "-c", "epiphany/repo-interpreter-brief-family"],
        &repo,
    )?;

    let item = "repo-interpreter-brief-family";
    let target_path = ".epiphany/interpreter-briefs/repo-interpreter-brief-family.toml";
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
            "repo-interpreter-brief-family-smoke",
            "--topic",
            "repo-interpreter-brief-family",
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
            "repo-interpreter-brief-family-smoke",
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
            "Turn rough Persona and Imagination consensus into a Mind-owned interpreter brief before any work item receives Hands authority.",
            "--candidate-action-ref",
            "candidate-action://repo-interpreter-brief-family/interpret-action-semantics",
            "--candidate-action-ref",
            "candidate-action://repo-interpreter-brief-family/request-soul-grounding",
            "--public-discussion-ref",
            "epiphany-global/persona-collaboration/repo-interpreter-brief-family",
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
            "repo-interpreter-brief",
            "--model-ref",
            "repo-interpreter-brief-family-smoke-imagination-v1",
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
    let brief_text = fs::read_to_string(repo.join(target_path))
        .with_context(|| format!("failed to read {}", repo.join(target_path).display()))?;

    require_eq(
        &plan,
        &["derivation", "safeActionFamily"],
        "repo.interpreter_brief",
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
        &brief_text,
        "schema_version = \"epiphany.repo_interpreter_brief.v0\"",
    )?;
    require_text(
        &brief_text,
        "safe_action_family = \"repo.interpreter_brief\"",
    )?;
    require_text(&brief_text, "[interpreter]")?;
    require_text(&brief_text, "status = \"draft\"")?;
    require_text(&brief_text, "owner = \"Mind\"")?;
    require_text(&brief_text, "source = \"Imagination\"")?;
    require_text(
        &brief_text,
        "purpose = \"public-pressure-to-action-semantics\"",
    )?;
    require_text(&brief_text, "requires_consensus_readback = true")?;
    require_text(&brief_text, "requires_safe_family_choice = true")?;
    require_text(&brief_text, "requires_requested_paths = true")?;
    require_text(&brief_text, "requires_verification_asks = true")?;
    require_text(&brief_text, "requires_evidence_needs = true")?;
    require_text(&brief_text, "candidate_actions_non_authoritative = true")?;
    require_text(&brief_text, "[semantic_checks]")?;
    require_text(&brief_text, "intent_summary_required = true")?;
    require_text(&brief_text, "scope_boundary_required = true")?;
    require_text(&brief_text, "requested_paths_required = true")?;
    require_text(&brief_text, "verification_required = true")?;
    require_text(&brief_text, "evidence_required = true")?;
    require_text(&brief_text, "rollback_required = true")?;
    require_text(&brief_text, "non_goals_required = true")?;
    require_text(&brief_text, "open_questions_required = true")?;
    require_text(&brief_text, "consensus_alignment_required = true")?;
    require_text(&brief_text, "[allowed_outputs]")?;
    require_text(&brief_text, "\"repo.consensus_brief\"")?;
    require_text(&brief_text, "\"repo.objective_draft\"")?;
    require_text(&brief_text, "\"repo.adoption_request\"")?;
    require_text(&brief_text, "\"repo.work_order\"")?;
    require_text(&brief_text, "\"repo.verification_request\"")?;
    require_text(&brief_text, "\"repo.publication_request\"")?;
    require_text(&brief_text, "may_request_replanning = true")?;
    require_text(&brief_text, "may_request_more_consensus = true")?;
    require_text(&brief_text, "may_adopt_objective = false")?;
    require_text(&brief_text, "may_schedule_work = false")?;
    require_text(&brief_text, "may_touch_substrate = false")?;
    require_text(&brief_text, "may_publish = false")?;
    require_text(&brief_text, "may_deploy = false")?;
    require_text(&brief_text, "[required_gates]")?;
    require_text(&brief_text, "imagination_consensus_required = true")?;
    require_text(&brief_text, "mind_review_required = true")?;
    require_text(&brief_text, "soul_source_grounding_required = true")?;
    require_text(&brief_text, "bifrost_publication_review_required = true")?;
    require_text(
        &brief_text,
        "hands_receipt_required_before_state_change = true",
    )?;
    require_text(
        &brief_text,
        "substrate_receipt_required_before_mutation = true",
    )?;
    require_text(
        &brief_text,
        "idunn_receipt_required_before_deployment = true",
    )?;
    require_text(&brief_text, "direct_state_commit_authorized = false")?;
    require_text(&brief_text, "objective_adoption_authorized = false")?;
    require_text(&brief_text, "self_scheduling_authorized = false")?;
    require_text(&brief_text, "substrate_access_authorized = false")?;
    require_text(&brief_text, "hands_action_authorized = false")?;
    require_text(&brief_text, "shell_command_authorized = false")?;
    require_text(&brief_text, "commit_authorized = false")?;
    require_text(&brief_text, "publication_authorized = false")?;
    require_text(&brief_text, "deployment_execution_authority = false")?;
    require_text(&brief_text, "cross_body_mutation_authorized = false")?;
    require_text(&brief_text, "private_worker_transcripts_allowed = false")?;
    require_text(&brief_text, "raw_result_payloads_allowed = false")?;
    require_text(&brief_text, "private_state_exposed = false")?;

    let summary = json!({
        "schemaVersion": "epiphany.repo_work_interpreter_brief_family_smoke.v0",
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
        "interpreterIsDraft": true,
        "mindOwnsInterpretation": true,
        "candidateActionsNonAuthoritative": true,
        "directStateCommitAuthorized": false,
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
            "expected interpreter brief text to contain {needle:?}"
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
