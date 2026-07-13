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
    let smoke_dir = smoke_root.join(format!("repo-consensus-brief-family-{stamp}"));
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
        "# Repo Consensus Brief Family Smoke\n\nThis repository proves branch-local Imagination consensus briefs.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    git(["add", "README.md"], &repo)?;
    git(
        ["commit", "-m", "Seed repo consensus brief smoke body"],
        &repo,
    )?;
    git(
        ["switch", "-c", "epiphany/repo-consensus-brief-family"],
        &repo,
    )?;

    let item = "repo-consensus-brief-family";
    let target_path = ".epiphany/consensus-briefs/repo-consensus-brief-family.toml";
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
            "repo-consensus-brief-family-smoke",
            "--topic",
            "repo-consensus-brief-family",
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
            "repo-consensus-brief-family-smoke",
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
            "Summarize public Persona collaboration into an Imagination consensus brief before any candidate action is adopted.",
            "--candidate-action-ref",
            "candidate-action://repo-consensus-brief-family/draft-task-card",
            "--candidate-action-ref",
            "candidate-action://repo-consensus-brief-family/request-review",
            "--public-discussion-ref",
            "epiphany-global/persona-collaboration/repo-consensus-brief-family",
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
            "repo-consensus-brief",
            "--model-ref",
            "repo-consensus-brief-family-smoke-imagination-v1",
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
    let overview = cargo_json(
        &manifest,
        "epiphany-work",
        &["overview", "--workspace", path_str(&repo)?, "--item", item],
        &root,
    )?;
    let brief_text = fs::read_to_string(repo.join(target_path))
        .with_context(|| format!("failed to read {}", repo.join(target_path).display()))?;

    require_eq(
        &plan,
        &["derivation", "safeActionFamily"],
        "repo.consensus_brief",
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
        "schema_version = \"epiphany.repo_consensus_brief.v0\"",
    )?;
    require_text(&brief_text, "safe_action_family = \"repo.consensus_brief\"")?;
    require_text(&brief_text, "[consensus]")?;
    require_text(&brief_text, "status = \"draft\"")?;
    require_text(&brief_text, "converged = false")?;
    require_text(&brief_text, "conflicts_remaining = true")?;
    require_text(&brief_text, "requires_human_or_persona_review = true")?;
    require_text(
        &brief_text,
        "recommended_next_safe_family = \"repo.task_card\"",
    )?;
    require_text(&brief_text, "[imagination]")?;
    require_text(&brief_text, "role = \"consensus-discovery\"")?;
    require_text(&brief_text, "candidate_actions_non_authoritative = true")?;
    require_text(&brief_text, "may_emit_action_items_receipt = true")?;
    require_text(&brief_text, "must_not_read_private_verses = true")?;
    require_text(&brief_text, "[inputs]")?;
    require_text(
        &brief_text,
        "epiphany-global/persona-collaboration/repo-consensus-brief-family",
    )?;
    require_text(
        &brief_text,
        "candidate-action://repo-consensus-brief-family/draft-task-card",
    )?;
    require_text(&brief_text, "objective_adoption_authorized = false")?;
    require_text(&brief_text, "hands_action_authorized = false")?;
    require_text(&brief_text, "cross_body_mutation_authorized = false")?;
    require_text(&brief_text, "mind_adoption_required = true")?;
    require_text(&brief_text, "bifrost_publication_required = true")?;
    require_text(&brief_text, "private_state_exposed = false")?;
    require_eq(
        &overview,
        &["intakeConsensus", "schemaVersion"],
        "epiphany.repo_work_intake_consensus_readback.v0",
    )?;
    require_eq(
        &overview,
        &["intakeConsensus", "owner"],
        "Persona->Imagination",
    )?;
    require_eq(
        &overview,
        &["intakeConsensus", "requestedConsensusRoute"],
        "imagination.consensus_discovery",
    )?;
    require_eq(
        &overview,
        &["intakeConsensus", "planSafeActionFamily"],
        "repo.consensus_brief",
    )?;
    require_bool(&overview, &["intakeConsensus", "planModelAuthored"], true)?;
    require_bool(
        &overview,
        &["intakeConsensus", "handsAuthorityGranted"],
        false,
    )?;
    require_bool(
        &overview,
        &["intakeConsensus", "durableStateAdmitted"],
        false,
    )?;
    require_bool(
        &overview,
        &["intakeConsensus", "publicationAuthorized"],
        false,
    )?;
    require_bool(
        &overview,
        &["intakeConsensus", "privateStateExposed"],
        false,
    )?;
    require_u64(
        &overview,
        &["intakeConsensus", "publicDiscussionRefCount"],
        1,
    )?;
    require_u64(
        &overview,
        &["intakeConsensus", "candidateActionRefCount"],
        2,
    )?;
    require_eq(
        &overview,
        &["proofBundle", "intakeConsensus", "planSafeActionFamily"],
        "repo.consensus_brief",
    )?;
    require_array_text_contains(&overview, &["verseProjection", "tuiRows"], "CONSENSUS |")?;
    require_array_text_contains(
        &overview,
        &["verseProjection", "tuiRows"],
        "route=imagination.consensus_discovery",
    )?;

    let summary = json!({
        "schemaVersion": "epiphany.repo_work_consensus_brief_family_smoke.v0",
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
        "briefIsDraft": true,
        "intakeConsensusSchema": overview["intakeConsensus"]["schemaVersion"],
        "intakeConsensusRoute": overview["intakeConsensus"]["requestedConsensusRoute"],
        "intakeConsensusCandidateActionRefCount": overview["intakeConsensus"]["candidateActionRefCount"],
        "intakeConsensusPlanSafeActionFamily": overview["intakeConsensus"]["planSafeActionFamily"],
        "candidateActionsNonAuthoritative": true,
        "mindAdoptionRequired": true,
        "bifrostPublicationRequired": true,
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

fn require_u64(value: &Value, path: &[&str], expected: u64) -> Result<()> {
    let actual = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_u64);
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

fn require_array_text_contains(value: &Value, path: &[&str], needle: &str) -> Result<()> {
    let values = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    if values
        .iter()
        .filter_map(Value::as_str)
        .any(|row| row.contains(needle))
    {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to contain {needle:?}, got {:?}",
            path.join("."),
            values
        ))
    }
}

fn require_text(haystack: &str, needle: &str) -> Result<()> {
    if haystack.contains(needle) {
        Ok(())
    } else {
        Err(anyhow!(
            "expected consensus brief text to contain {needle:?}"
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
