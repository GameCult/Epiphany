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
    let smoke_dir = args.smoke_root.join(format!("repo-work-readiness-{stamp}"));
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
        "# Repo Work Readiness Smoke\n\nThis repository proves readiness sight without readiness approval.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    git(["add", "README.md"], &repo)?;
    git(
        ["commit", "-m", "Seed repo work readiness smoke body"],
        &repo,
    )?;
    git(["switch", "-c", "epiphany/repo-work-readiness"], &repo)?;

    let item = "repo-work-readiness";
    let local_verse = repo.join(".epiphany").join("local-verse.ccmp");
    let work_dir = repo.join(".epiphany").join("work");
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
            "repo-work-readiness-smoke",
            "--topic",
            "repo-work-readiness",
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
            "repo-work-readiness-smoke",
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
            "Prove repo-work readiness sight can name safe-family planning depth and missing publication and lifecycle gates without granting authority.",
            "--local-verse-store",
            path_str(&local_verse)?,
            "--runtime-id",
            "repo-work-readiness-smoke",
            "--candidate-action-ref",
            "candidate-action://repo-work-readiness/safe-family-planning",
            "--public-discussion-ref",
            "epiphany-global/persona-collaboration/repo-work-readiness",
        ],
        &root,
    )?;
    cargo_json(
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
            "repo-work-readiness-smoke-imagination-v1",
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
    cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "queue-run",
            "--workspace",
            path_str(&repo)?,
            "--local-verse-store",
            path_str(&local_verse)?,
            "--runtime-id",
            "repo-work-readiness-smoke",
            "--max-items",
            "1",
            "--dry-run",
        ],
        &root,
    )?;
    let public_proof = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "export-proof",
            "--workspace",
            path_str(&repo)?,
            "--item",
            item,
            "--local-verse-store",
            path_str(&local_verse)?,
            "--runtime-id",
            "repo-work-readiness-smoke",
        ],
        &root,
    )?;
    let close_receipt = repo
        .join(".epiphany")
        .join("work")
        .join(format!("work-close-{item}.json"));
    let publish = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "publish",
            "--workspace",
            path_str(&repo)?,
            "--epiphany-root",
            path_str(&root)?,
            "--item",
            item,
            "--artifact-dir",
            path_str(&work_dir)?,
            "--closure-receipt",
            path_str(&close_receipt)?,
            "--local-verse-store",
            path_str(&local_verse)?,
            "--change-summary",
            "Repo-work readiness smoke Bifrost publication proof.",
            "--justification",
            "Disposable Bifrost publication proof for the repo-work readiness smoke.",
            "--verification-receipt",
            "soul-verdict:repo-work-readiness-smoke",
            "--review-receipt",
            "mind-review:repo-work-readiness-smoke",
            "--author-agent",
            "epiphany.Hands",
            "--credit-subject",
            "epiphany.Hands",
            "--ledger-entry-id",
            "repo-work-readiness-smoke-ledger",
            "--pull-request-url",
            "https://example.invalid/GameCult/repo-work-readiness-smoke/pull/1",
            "--pull-request-number",
            "1",
            "--pull-request-title",
            "Repo work readiness smoke proof",
        ],
        &root,
    )?;
    git(["branch", "-f", "main", "HEAD"], &repo)?;
    let sync = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "sync",
            "--workspace",
            path_str(&repo)?,
            "--item",
            item,
            "--publish-receipt",
            publish
                .get("receiptPath")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("publish result missing receiptPath"))?,
            "--artifact-dir",
            path_str(&work_dir)?,
            "--upstream-ref",
            "main",
            "--merge-receipt",
            "maintainer-merge:repo-work-readiness-smoke",
        ],
        &root,
    )?;
    let deployment_aftercare_fixture = repo
        .join(".epiphany")
        .join("work")
        .join("deployment-aftercare-audit.json");
    write_json(
        &deployment_aftercare_fixture,
        &json!({
            "schemaVersion": "epiphany.repo_deployment_aftercare_audit.v0",
            "status": "complete",
            "deploymentComplete": true,
            "deploymentAuthority": false,
            "gitPushAuthority": false,
            "serviceLifecycleAuthority": false,
            "privateStateExposed": false
        }),
    )?;
    let idunn_lifecycle_fixture = repo
        .join(".epiphany")
        .join("work")
        .join("repo-work-service-audit.json");
    write_json(
        &idunn_lifecycle_fixture,
        &json!({
            "schemaVersion": "epiphany.repo_work_service_audit.v0",
            "status": "complete",
            "serviceId": "epiphany-repo-work-queue-runner",
            "schedulerId": "epiphany-repo-work-queue-runner",
            "receiptId": "repo-work-readiness-smoke-idunn-audit",
            "planStatus": "present",
            "runbookStatus": "present",
            "runbookArtifactStatus": "present",
            "runbookArtifactRef": "artifact://repo-work-readiness-smoke/repo-work-service-runbook.ps1",
            "runbookSha256": "readiness-smoke-fixture",
            "launchStatus": "ok",
            "launchExitCode": 0,
            "missingChecks": [],
            "failedChecks": [],
            "lifecycleOwner": "Idunn",
            "hostedBody": "repo-work",
            "mutatesServiceManager": false,
            "requiresElevatedAuthority": false,
            "privateStateExposed": false,
            "nextSafeMove": "continue repo-swarm MVP planner/interpreter hardening"
        }),
    )?;
    let tool_directory = cargo_json(
        &manifest,
        "epiphany-verse-query",
        &[
            "tool-directory",
            "--store",
            path_str(&local_verse)?,
            "--runtime-id",
            "repo-work-readiness-smoke",
        ],
        &root,
    )?;
    write_json(
        &repo
            .join(".epiphany")
            .join("work")
            .join("tool-directory.json"),
        &tool_directory,
    )?;
    let readiness = cargo_json(
        &manifest,
        "epiphany-work",
        &["readiness", "--workspace", path_str(&repo)?, "--item", item],
        &root,
    )?;

    require_eq(
        &readiness,
        &["schemaVersion"],
        "epiphany.repo_work_readiness.v0",
    )?;
    require_eq(&readiness, &["status"], "ready")?;
    require_u64(&readiness, &["missingRequiredCount"], 0)?;
    require_bool(&readiness, &["authority", "sightOnly"], true)?;
    require_bool(
        &readiness,
        &["authority", "readinessApprovalAuthorized"],
        false,
    )?;
    require_bool(&readiness, &["authority", "publicationAuthorized"], false)?;
    require_bool(&readiness, &["authority", "deploymentAuthority"], false)?;
    require_bool(
        &readiness,
        &["authority", "serviceLifecycleAuthority"],
        false,
    )?;
    require_bool(&readiness, &["authority", "handsActionAuthorized"], false)?;
    require_bool(&readiness, &["authority", "privateStateExposed"], false)?;
    require_eq(
        &readiness,
        &["verseProjection", "documentType"],
        "epiphany.cultmesh.repo_work_readiness",
    )?;
    require_eq(
        &readiness,
        &["verseProjection", "readinessId"],
        "repo-work-readiness-repo-work-readiness",
    )?;
    require_bool(
        &readiness,
        &["verseProjection", "privateStateExposed"],
        false,
    )?;
    require_row(&readiness, "repo-init", true)?;
    require_row(&readiness, "swarm-online", true)?;
    require_row(&readiness, "persona-intake", true)?;
    require_row(&readiness, "imagination-plan", true)?;
    require_row(&readiness, "self-queue-run", true)?;
    require_row(&readiness, "hands-branch-work", true)?;
    require_row(&readiness, "soul-closure", true)?;
    require_row(&readiness, "modeling-mind-admission", true)?;
    require_row(&readiness, "safe-family-planning", true)?;
    require_row_field_u64(
        &readiness,
        "safe-family-planning",
        "candidateNextSafeFamilyCount",
        21,
    )?;
    require_row(&readiness, "public-proof", true)?;
    require_row(&readiness, "bifrost-publication", true)?;
    require_row_field_u64(&readiness, "bifrost-publication", "creditReceiptCount", 1)?;
    require_row_field_u64(&readiness, "bifrost-publication", "changedPathCount", 1)?;
    require_row(&readiness, "upstream-main-sync", true)?;
    require_row_field_u64(&readiness, "upstream-main-sync", "mergeReceiptCount", 1)?;
    require_row_field_bool(&readiness, "upstream-main-sync", "ancestryProved", true)?;
    require_row_field_bool(
        &readiness,
        "upstream-main-sync",
        "publishReceiptMatches",
        true,
    )?;
    require_row(&readiness, "idunn-lifecycle", true)?;
    require_row(&readiness, "deployment-aftercare", true)?;
    require_row(&readiness, "tool-directory", true)?;
    require_row(&readiness, "private-state-redaction", true)?;

    let readiness_review = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "readiness-review",
            "--workspace",
            path_str(&repo)?,
            "--item",
            item,
            "--readiness-receipt",
            readiness
                .get("receiptPath")
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("readiness result missing receiptPath"))?,
            "--maintainer-review-receipt",
            "maintainer-review:repo-work-readiness-smoke",
            "--soul-review-receipt",
            "soul-review:repo-work-readiness-smoke",
            "--mind-review-receipt",
            "mind-review:repo-work-readiness-smoke",
            "--bifrost-review-receipt",
            "bifrost-review:repo-work-readiness-smoke",
            "--review-summary",
            "Smoke reviewers approve the ready readiness sight receipt without granting action authority.",
        ],
        &root,
    )?;
    require_eq(
        &readiness_review,
        &["schemaVersion"],
        "epiphany.repo_work_readiness_review.v0",
    )?;
    require_eq(&readiness_review, &["status"], "readiness-approved")?;
    require_u64(&readiness_review, &["missingRequiredCount"], 0)?;
    require_bool(
        &readiness_review,
        &["authority", "readinessApprovalAuthorized"],
        true,
    )?;
    require_bool(
        &readiness_review,
        &["authority", "durableStateCommitAuthorized"],
        false,
    )?;
    require_bool(
        &readiness_review,
        &["authority", "publicationAuthorized"],
        false,
    )?;
    require_bool(&readiness_review, &["authority", "mergeAuthorized"], false)?;
    require_bool(
        &readiness_review,
        &["authority", "serviceLifecycleAuthority"],
        false,
    )?;
    require_bool(
        &readiness_review,
        &["authority", "handsActionAuthorized"],
        false,
    )?;
    require_eq(
        &readiness_review,
        &["verseProjection", "documentType"],
        "epiphany.cultmesh.repo_work_readiness_review",
    )?;
    require_eq(
        &readiness_review,
        &["verseProjection", "reviewId"],
        "repo-work-readiness-review-repo-work-readiness",
    )?;
    require_bool(
        &readiness_review,
        &["verseProjection", "privateStateExposed"],
        false,
    )?;
    require_bool(&readiness_review, &["privateStateExposed"], false)?;

    let gjallar = cargo_json(
        &manifest,
        "epiphany-verse-query",
        &[
            "gjallar",
            "--store",
            path_str(&local_verse)?,
            "--runtime-id",
            "repo-work-readiness-smoke",
        ],
        &root,
    )?;
    require_u64(&gjallar, &["repoWorkReadinessCount"], 1)?;
    require_eq(
        &gjallar,
        &["latestRepoWorkReadiness"],
        "repo-work-readiness-repo-work-readiness",
    )?;
    require_bool(&gjallar, &["privateStateExposed"], false)?;
    require_tui_row_contains(
        &gjallar,
        &["repoWorkReadinessTuiRows"],
        "REPO-WORK-READINESS",
    )?;
    require_tui_row_contains(&gjallar, &["swarmActionTuiRows"], "repo-work-readiness")?;
    require_u64(&gjallar, &["repoWorkReadinessReviewCount"], 1)?;
    require_eq(
        &gjallar,
        &["latestRepoWorkReadinessReview"],
        "repo-work-readiness-review-repo-work-readiness",
    )?;
    require_tui_row_contains(
        &gjallar,
        &["repoWorkReadinessReviewTuiRows"],
        "REPO-WORK-READINESS-REVIEW",
    )?;
    require_tui_row_contains(
        &gjallar,
        &["swarmActionTuiRows"],
        "repo-work-readiness-review",
    )?;

    let summary = json!({
        "schemaVersion": "epiphany.repo_work_readiness_smoke.v0",
        "status": "ok",
        "smokeDir": smoke_dir,
        "repo": repo,
        "branch": git_output(["branch", "--show-current"], &repo)?,
        "item": item,
        "publicProofOutput": public_proof["outputPath"],
        "publishStatus": publish["status"],
        "syncStatus": sync["status"],
        "upstreamMainSynced": sync["authority"]["upstreamMainSynced"],
        "readinessStatus": readiness["status"],
        "readinessReviewStatus": readiness_review["status"],
        "readinessReviewReceiptPath": readiness_review["receiptPath"],
        "readinessReviewVerseProjection": readiness_review["verseProjection"],
        "missingRequiredCount": readiness["missingRequiredCount"],
        "readinessVerseProjection": readiness["verseProjection"],
        "gjallarRepoWorkReadinessCount": gjallar["repoWorkReadinessCount"],
        "gjallarLatestRepoWorkReadiness": gjallar["latestRepoWorkReadiness"],
        "gjallarRepoWorkReadinessReviewCount": gjallar["repoWorkReadinessReviewCount"],
        "gjallarLatestRepoWorkReadinessReview": gjallar["latestRepoWorkReadinessReview"],
        "sightOnly": readiness["authority"]["sightOnly"],
        "readinessApprovalAuthorized": readiness["authority"]["readinessApprovalAuthorized"],
        "publicationAuthorized": readiness["authority"]["publicationAuthorized"],
        "deploymentAuthority": readiness["authority"]["deploymentAuthority"],
        "serviceLifecycleAuthority": readiness["authority"]["serviceLifecycleAuthority"],
        "handsActionAuthorized": readiness["authority"]["handsActionAuthorized"],
        "privateStateExposed": readiness["privateStateExposed"],
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

fn require_u64(value: &Value, path: &[&str], expected: u64) -> Result<()> {
    let actual = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_u64)
        .unwrap_or(u64::MAX);
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
        .and_then(Value::as_bool)
        .unwrap_or(!expected);
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to be {expected:?}, got {actual:?}",
            path.join(".")
        ))
    }
}

fn require_row(value: &Value, kind: &str, expected_satisfied: bool) -> Result<()> {
    let rows = value
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("readiness output did not include rows"))?;
    let row = rows
        .iter()
        .find(|row| row.get("kind").and_then(Value::as_str) == Some(kind))
        .ok_or_else(|| anyhow!("missing readiness row {kind:?}"))?;
    let actual = row
        .get("satisfied")
        .and_then(Value::as_bool)
        .unwrap_or(!expected_satisfied);
    if actual == expected_satisfied {
        Ok(())
    } else {
        Err(anyhow!(
            "expected readiness row {kind:?} satisfied={expected_satisfied}, got {actual}"
        ))
    }
}

fn require_row_field_u64(value: &Value, kind: &str, field: &str, expected: u64) -> Result<()> {
    let rows = value
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("readiness output did not include rows"))?;
    let row = rows
        .iter()
        .find(|row| row.get("kind").and_then(Value::as_str) == Some(kind))
        .ok_or_else(|| anyhow!("missing readiness row {kind:?}"))?;
    let actual = row.get(field).and_then(Value::as_u64).unwrap_or(u64::MAX);
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected readiness row {kind:?} field {field:?} to be {expected}, got {actual}"
        ))
    }
}

fn require_row_field_bool(value: &Value, kind: &str, field: &str, expected: bool) -> Result<()> {
    let rows = value
        .get("rows")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("readiness output did not include rows"))?;
    let row = rows
        .iter()
        .find(|row| row.get("kind").and_then(Value::as_str) == Some(kind))
        .ok_or_else(|| anyhow!("missing readiness row {kind:?}"))?;
    let actual = row.get(field).and_then(Value::as_bool).unwrap_or(!expected);
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected readiness row {kind:?} field {field:?} to be {expected}, got {actual}"
        ))
    }
}

fn require_tui_row_contains(value: &Value, path: &[&str], needle: &str) -> Result<()> {
    let rows = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("{} did not contain TUI rows", path.join(".")))?;
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
