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
    keep: bool,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut root = env::current_dir().context("failed to resolve current directory")?;
        let mut smoke_root = root.join(".epiphany-smoke");
        let mut keep = false;
        let mut args = env::args().skip(1).peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--root" => root = take_path(&mut args, "--root")?,
                "--smoke-root" => smoke_root = take_path(&mut args, "--smoke-root")?,
                "--keep" => keep = true,
                other => return Err(anyhow!("unexpected argument {other:?}")),
            }
        }
        Ok(Self {
            root,
            smoke_root,
            keep,
        })
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
    let smoke_dir = args.smoke_root.join(format!("fresh-repo-mvp-{stamp}"));
    if smoke_dir.exists() {
        fs::remove_dir_all(&smoke_dir)
            .with_context(|| format!("failed to clear {}", smoke_dir.display()))?;
    }
    fs::create_dir_all(&smoke_dir)
        .with_context(|| format!("failed to create {}", smoke_dir.display()))?;

    let origin = smoke_dir.join("origin.git");
    let repo = smoke_dir.join("repo-body");
    git(["init", "--bare", path_str(&origin)?], &root)?;
    git(["clone", path_str(&origin)?, path_str(&repo)?], &root)?;
    git(
        ["config", "user.email", "epiphany-smoke@example.invalid"],
        &repo,
    )?;
    git(["config", "user.name", "Epiphany Smoke"], &repo)?;
    git(["switch", "-c", "main"], &repo)?;
    fs::write(
        repo.join("README.md"),
        "# Fresh Repo MVP Smoke\n\nThis repository starts empty enough for Epiphany to prove birth.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    git(["add", "README.md"], &repo)?;
    git(["commit", "-m", "Seed fresh repo body"], &repo)?;
    git(["push", "-u", "origin", "main"], &repo)?;

    let item = "fresh-repo-mvp";
    let runtime_id = "repo-swarm-fresh-mvp";
    let local_verse = repo.join(".epiphany").join("local-verse.ccmp");
    let init = cargo_json(
        &manifest,
        "epiphany-repo",
        &[
            "init",
            "--workspace",
            path_str(&repo)?,
            "--epiphany-root",
            path_str(&root)?,
            "--swarm-id",
            "fresh-repo-smoke",
            "--topic",
            "mvp-proof",
            "--switch-branch",
        ],
        &root,
    )?;
    let online = cargo_json(
        &manifest,
        "epiphany-swarm",
        &[
            "online",
            "--workspace",
            path_str(&repo)?,
            "--epiphany-root",
            path_str(&root)?,
            "--runtime-id",
            runtime_id,
            "--local-verse-store",
            path_str(&local_verse)?,
        ],
        &root,
    )?;
    let intake = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "persona-intake",
            "--workspace",
            path_str(&repo)?,
            "--epiphany-root",
            path_str(&root)?,
            "--runtime-id",
            runtime_id,
            "--item",
            item,
            "--message",
            "Please prove this fresh repository can carry an Epiphany branch-local status update from Persona pressure to verified proof.",
            "--topic",
            "fresh-repo-mvp",
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
            "repo-status-section",
            "--model-ref",
            "fresh-repo-mvp-smoke-imagination-v1",
            "--model-authored",
        ],
        &root,
    )?;
    let pre_run_overview = cargo_json(
        &manifest,
        "epiphany-work",
        &["overview", "--workspace", path_str(&repo)?, "--item", item],
        &root,
    )?;
    let swarm_run = cargo_json(
        &manifest,
        "epiphany-swarm",
        &[
            "run",
            "--workspace",
            path_str(&repo)?,
            "--epiphany-root",
            path_str(&root)?,
            "--runtime-id",
            runtime_id,
            "--local-verse-store",
            path_str(&local_verse)?,
            "--max-iterations",
            "4",
            "--max-items",
            "1",
            "--cooldown-seconds",
            "0",
        ],
        &root,
    )?;
    let close_receipt = repo
        .join(".epiphany")
        .join("work")
        .join(format!("work-close-{item}.json"));
    let close = read_json(&close_receipt)?;
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
            "--closure-receipt",
            path_str(&close_receipt)?,
            "--local-verse-store",
            path_str(&local_verse)?,
            "--change-summary",
            "Fresh-repo MVP smoke branch-local status proof.",
            "--justification",
            "Disposable maintainer review for the native fresh-repo MVP smoke.",
            "--ledger-entry-id",
            "fresh-repo-mvp-smoke-ledger",
            "--pull-request-url",
            "https://example.invalid/GameCult/fresh-repo-mvp-smoke/pull/1",
            "--pull-request-number",
            "1",
            "--pull-request-title",
            "Fresh repo MVP smoke proof",
        ],
        &root,
    )?;

    git(["push", "origin", "HEAD:main"], &repo)?;
    git(["fetch", "origin", "main"], &repo)?;
    let publish_receipt = repo
        .join(".epiphany")
        .join("work")
        .join(format!("work-publish-{item}.json"));
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
            path_str(&publish_receipt)?,
            "--upstream-ref",
            "origin/main",
            "--merge-receipt",
            "maintainer-merge:fresh-repo-mvp-smoke",
        ],
        &root,
    )?;
    let overview = cargo_json(
        &manifest,
        "epiphany-work",
        &["overview", "--workspace", path_str(&repo)?, "--item", item],
        &root,
    )?;
    let export = cargo_json(
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
            runtime_id,
        ],
        &root,
    )?;

    require_eq(&close, &["status"], "closed")?;
    require_eq(&close, &["soul", "verdict"], "passed")?;
    require_eq(
        &close,
        &["closureReview", "familyAssertions", "status"],
        "passed",
    )?;
    require_eq(
        &swarm_run,
        &["stopClassification", "schemaVersion"],
        "epiphany.repo_swarm_run_stop_classification.v0",
    )?;
    require_eq(
        &swarm_run,
        &["stopClassification", "category"],
        "iteration-limit",
    )?;
    require_eq(&swarm_run, &["stopClassification", "owner"], "Self")?;
    require_eq(
        &swarm_run,
        &["stopClassification", "authorityGate"],
        "self.scheduler-iteration-limit",
    )?;
    require_bool(&swarm_run, &["stopClassification", "mutatesState"], false)?;
    require_bool(
        &swarm_run,
        &["stopClassification", "requiresElevatedAuthority"],
        false,
    )?;
    require_bool(
        &swarm_run,
        &["stopClassification", "privateStateExposed"],
        false,
    )?;
    require_eq(&publish, &["status"], "publication-receipts-recorded")?;
    require_eq(&sync, &["status"], "upstream-main-synced")?;
    require_eq(&export, &["status"], "public-proof-exported")?;
    require_non_empty(&intake, &["memoryRecallStatus"])?;
    require_non_empty(&intake, &["memoryRecallCacheStatus"])?;
    require_non_empty(&intake, &["weksaLoweringReceiptId"])?;
    require_bool(&intake, &["privateStateExposed"], false)?;
    require_bool(&overview, &["privateStateExposed"], false)?;
    require_bool(&export, &["privateStateExposed"], false)?;
    let close_private = close
        .pointer("/authority/privateStateExposed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if close_private {
        return Err(anyhow!("close receipt exposed private state"));
    }
    let commit_sha = close
        .pointer("/handsReceipts/commitSha")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_string();
    let final_summary = json!({
        "schemaVersion": "epiphany.repo_swarm_fresh_repo_mvp_smoke.v0",
        "status": "ok",
        "smokeDir": smoke_dir,
        "repo": repo,
        "origin": origin,
        "item": item,
        "branch": git_output(["branch", "--show-current"], &repo)?,
        "commitSha": commit_sha,
        "initStatus": init["status"],
        "onlineStatus": online["status"],
        "personaIntakeStatus": intake["status"],
        "personaMemoryRecallStatus": intake["memoryRecallStatus"],
        "personaMemoryRecallCacheStatus": intake["memoryRecallCacheStatus"],
        "personaMemoryRecallHitCount": intake["memoryRecallHitCount"],
        "weksaLoweringReceiptId": intake["weksaLoweringReceiptId"],
        "planStatus": plan["status"],
        "preRunOverviewGate": pre_run_overview["gate"],
        "preRunOverviewBlocker": pre_run_overview["blocker"],
        "swarmRunStatus": swarm_run["status"],
        "swarmRunStopCategory": swarm_run["stopClassification"]["category"],
        "swarmRunStopOwner": swarm_run["stopClassification"]["owner"],
        "swarmRunStopGate": swarm_run["stopClassification"]["authorityGate"],
        "closeStatus": close["status"],
        "soulVerdict": close["soul"]["verdict"],
        "familyAssertionsStatus": close["closureReview"]["familyAssertions"]["status"],
        "publishStatus": publish["status"],
        "syncStatus": sync["status"],
        "upstreamMainSynced": sync["authority"]["upstreamMainSynced"],
        "overviewGate": overview["gate"],
        "overviewBlocker": overview["blocker"],
        "publicProofStatus": export["status"],
        "publicProofArtifact": export["outputPath"],
        "privateStateExposed": false,
        "kept": args.keep
    });
    let summary_path = smoke_dir.join("summary.json");
    write_json(&summary_path, &final_summary)?;
    Ok(final_summary)
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

fn require_non_empty(value: &Value, path: &[&str]) -> Result<()> {
    let actual = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .and_then(Value::as_str)
        .unwrap_or_default();
    if actual.trim().is_empty() {
        Err(anyhow!("expected {} to be non-empty", path.join(".")))
    } else {
        Ok(())
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
