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
        .join(format!("repo-planning-brief-family-{stamp}"));
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
        "# Repo Planning Brief Family Smoke\n\nThis repository proves branch-local Imagination safe-family planning briefs.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    git(["add", "README.md"], &repo)?;
    git(
        ["commit", "-m", "Seed repo planning brief smoke body"],
        &repo,
    )?;
    git(
        ["switch", "-c", "epiphany/repo-planning-brief-family"],
        &repo,
    )?;

    let item = "repo-planning-brief-family";
    let target_path = ".epiphany/planning-briefs/repo-planning-brief-family.toml";
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
            "repo-planning-brief-family-smoke",
            "--topic",
            "repo-planning-brief-family",
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
            "repo-planning-brief-family-smoke",
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
            "Turn a rough Persona request into a safe-family planning brief before any work item receives Hands authority.",
            "--candidate-action-ref",
            "candidate-action://repo-planning-brief-family/decompose-safe-families",
            "--candidate-action-ref",
            "candidate-action://repo-planning-brief-family/request-mind-review",
            "--public-discussion-ref",
            "epiphany-global/persona-collaboration/repo-planning-brief-family",
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
            "repo-planning-brief",
            "--model-ref",
            "repo-planning-brief-family-smoke-imagination-v1",
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
        "repo.planning_brief",
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
    require_eq(
        &close,
        &[
            "closureReview",
            "familyAssertions",
            "safeFamilyPlanning",
            "schemaVersion",
        ],
        "epiphany.repo_work_safe_family_planning_readback.v0",
    )?;
    require_eq(
        &close,
        &[
            "closureReview",
            "familyAssertions",
            "safeFamilyPlanning",
            "sourceSafeFamily",
        ],
        "repo.planning_brief",
    )?;
    require_bool(
        &close,
        &[
            "closureReview",
            "familyAssertions",
            "safeFamilyPlanning",
            "allExpectedCandidateFamiliesPresent",
        ],
        true,
    )?;
    require_bool(
        &close,
        &[
            "closureReview",
            "familyAssertions",
            "safeFamilyPlanning",
            "allPlanningRequirementsPresent",
        ],
        true,
    )?;
    require_bool(
        &close,
        &[
            "closureReview",
            "familyAssertions",
            "safeFamilyPlanning",
            "allRequiredGatesPresent",
        ],
        true,
    )?;
    require_bool(
        &close,
        &[
            "closureReview",
            "familyAssertions",
            "safeFamilyPlanning",
            "authorityDenied",
        ],
        true,
    )?;
    require_bool(
        &close,
        &[
            "closureReview",
            "familyAssertions",
            "safeFamilyPlanning",
            "privateStateExposed",
        ],
        false,
    )?;
    require_u64(
        &close,
        &[
            "closureReview",
            "familyAssertions",
            "safeFamilyPlanning",
            "candidateNextSafeFamilyCount",
        ],
        21,
    )?;
    require_bool(
        &close,
        &[
            "closureReview",
            "familyAssertions",
            "safeFamilyPlanning",
            "allMatrixGroupsComplete",
        ],
        true,
    )?;
    require_bool(
        &close,
        &[
            "closureReview",
            "familyAssertions",
            "safeFamilyPlanning",
            "matrixControlsPresent",
        ],
        true,
    )?;
    require_bool(
        &close,
        &[
            "closureReview",
            "familyAssertions",
            "safeFamilyPlanning",
            "allClosureProofsPresent",
        ],
        true,
    )?;
    require_bool(&close, &["privateStateExposed"], false)?;
    require_text(
        &brief_text,
        "schema_version = \"epiphany.repo_planning_brief.v0\"",
    )?;
    require_text(&brief_text, "safe_action_family = \"repo.planning_brief\"")?;
    require_text(&brief_text, "[planning_brief]")?;
    require_text(&brief_text, "status = \"draft\"")?;
    require_text(&brief_text, "owner = \"Imagination\"")?;
    require_text(&brief_text, "candidate_actions_non_authoritative = true")?;
    require_text(&brief_text, "requires_mind_interpretation = true")?;
    require_text(&brief_text, "requires_soul_evidence_needs = true")?;
    require_text(&brief_text, "[decomposition]")?;
    require_text(&brief_text, "\"repo.consensus_brief\"")?;
    require_text(&brief_text, "\"repo.interpreter_brief\"")?;
    require_text(&brief_text, "\"repo.objective_draft\"")?;
    require_text(&brief_text, "\"repo.adoption_request\"")?;
    require_text(&brief_text, "\"repo.scheduling_request\"")?;
    require_text(&brief_text, "\"repo.task_card\"")?;
    require_text(&brief_text, "\"repo.work_order\"")?;
    require_text(&brief_text, "\"repo.verification_request\"")?;
    require_text(&brief_text, "\"repo.maintainer_review_request\"")?;
    require_text(&brief_text, "\"repo.artifact_acceptance_request\"")?;
    require_text(&brief_text, "\"repo.publication_request\"")?;
    require_text(&brief_text, "\"repo.sync_request\"")?;
    require_text(&brief_text, "\"repo.pr_request\"")?;
    require_text(&brief_text, "\"repo.credit_request\"")?;
    require_text(&brief_text, "\"repo.metrics_request\"")?;
    require_text(&brief_text, "\"repo.readiness_review_request\"")?;
    require_text(&brief_text, "\"repo.doctrine_update_request\"")?;
    require_text(&brief_text, "\"repo.secret_policy_request\"")?;
    require_text(&brief_text, "\"repo.dependency_policy_request\"")?;
    require_text(&brief_text, "\"repo.deployment_config\"")?;
    require_text(&brief_text, "\"repo.deployment_request\"")?;
    require_text(
        &brief_text,
        "candidate_items_must_name_requested_paths = true",
    )?;
    require_text(
        &brief_text,
        "candidate_items_must_name_verification_asks = true",
    )?;
    require_text(
        &brief_text,
        "candidate_items_must_name_evidence_needs = true",
    )?;
    require_text(&brief_text, "candidate_items_must_name_owner = true")?;
    require_text(
        &brief_text,
        "candidate_items_must_name_authority_denials = true",
    )?;
    require_text(
        &brief_text,
        "candidate_items_must_name_closure_proofs = true",
    )?;
    require_text(&brief_text, "[safe_family_matrix]")?;
    require_text(&brief_text, "preparation = [")?;
    require_text(&brief_text, "adoption_and_queue = [")?;
    require_text(&brief_text, "execution_and_review = [")?;
    require_text(&brief_text, "publication_and_accounting = [")?;
    require_text(&brief_text, "policy_and_deployment = [")?;
    require_text(&brief_text, "matrix_is_planning_only = true")?;
    require_text(&brief_text, "families_may_not_inherit_authority = true")?;
    require_text(
        &brief_text,
        "family_choice_requires_mind_or_self_review = true",
    )?;
    require_text(&brief_text, "[closure_proofs]")?;
    require_text(&brief_text, "soul_family_assertions_required = true")?;
    require_text(
        &brief_text,
        "modeling_map_update_required_after_verified_consequence = true",
    )?;
    require_text(&brief_text, "mind_gateway_review_required = true")?;
    require_text(&brief_text, "mind_state_commit_required = true")?;
    require_text(
        &brief_text,
        "bifrost_publication_gate_required_for_upstream = true",
    )?;
    require_text(
        &brief_text,
        "upstream_main_sync_required_after_publication = true",
    )?;
    require_text(&brief_text, "private_state_redaction_required = true")?;
    require_text(&brief_text, "[gates]")?;
    require_text(&brief_text, "mind_interpreter_required = true")?;
    require_text(&brief_text, "self_queue_selection_required = true")?;
    require_text(&brief_text, "substrate_gate_required_before_hands = true")?;
    require_text(&brief_text, "hands_receipts_required_before_soul = true")?;
    require_text(
        &brief_text,
        "soul_verdict_required_before_mind_map_admission = true",
    )?;
    require_text(&brief_text, "bifrost_required_before_publication = true")?;
    require_text(
        &brief_text,
        "idunn_required_before_deployment_execution = true",
    )?;
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
        "schemaVersion": "epiphany.repo_work_planning_brief_family_smoke.v0",
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
        "safeFamilyPlanningSchema": close["closureReview"]["familyAssertions"]["safeFamilyPlanning"]["schemaVersion"],
        "candidateNextSafeFamilyCount": close["closureReview"]["familyAssertions"]["safeFamilyPlanning"]["candidateNextSafeFamilyCount"],
        "allExpectedCandidateFamiliesPresent": close["closureReview"]["familyAssertions"]["safeFamilyPlanning"]["allExpectedCandidateFamiliesPresent"],
        "allMatrixGroupsComplete": close["closureReview"]["familyAssertions"]["safeFamilyPlanning"]["allMatrixGroupsComplete"],
        "matrixControlsPresent": close["closureReview"]["familyAssertions"]["safeFamilyPlanning"]["matrixControlsPresent"],
        "allClosureProofsPresent": close["closureReview"]["familyAssertions"]["safeFamilyPlanning"]["allClosureProofsPresent"],
        "allRequiredGatesPresent": close["closureReview"]["familyAssertions"]["safeFamilyPlanning"]["allRequiredGatesPresent"],
        "authorityDenied": close["closureReview"]["familyAssertions"]["safeFamilyPlanning"]["authorityDenied"],
        "briefIsDraft": true,
        "candidateActionsNonAuthoritative": true,
        "mindInterpreterRequired": true,
        "soulEvidenceNeedsRequired": true,
        "idunnDeploymentExecutionRequired": true,
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

fn require_text(haystack: &str, needle: &str) -> Result<()> {
    if haystack.contains(needle) {
        Ok(())
    } else {
        Err(anyhow!(
            "expected planning brief text to contain {needle:?}"
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
