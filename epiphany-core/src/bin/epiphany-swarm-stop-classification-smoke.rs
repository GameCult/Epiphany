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
        .join(format!("swarm-stop-classification-{stamp}"));
    if smoke_dir.exists() {
        fs::remove_dir_all(&smoke_dir)
            .with_context(|| format!("failed to clear {}", smoke_dir.display()))?;
    }
    fs::create_dir_all(&smoke_dir)
        .with_context(|| format!("failed to create {}", smoke_dir.display()))?;

    let origin = smoke_dir.join("origin.git");
    git(["init", "--bare", path_str(&origin)?], &smoke_dir)?;

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
        "# Swarm Stop Classification Smoke\n\nThis repository proves typed repo-swarm run stop classification.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    git(["add", "README.md"], &repo)?;
    git(
        ["commit", "-m", "Seed swarm stop classification smoke"],
        &repo,
    )?;
    git(["branch", "-M", "main"], &repo)?;
    git(["remote", "add", "origin", path_str(&origin)?], &repo)?;
    git(["push", "-u", "origin", "main"], &repo)?;

    let runtime_id = "swarm-stop-classification-smoke";
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
            runtime_id,
            "--topic",
            "swarm-stop-classification",
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
            runtime_id,
            "--local-verse-store",
            path_str(&local_verse)?,
        ],
        &root,
    )?;

    let empty_run = cargo_json(
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
            "1",
            "--max-items",
            "1",
            "--cooldown-seconds",
            "0",
        ],
        &root,
    )?;
    require_eq(
        &empty_run,
        &["stopClassification", "schemaVersion"],
        "epiphany.repo_swarm_run_stop_classification.v0",
    )?;
    require_eq(
        &empty_run,
        &["stopClassification", "category"],
        "queue-empty",
    )?;
    require_eq(&empty_run, &["stopClassification", "owner"], "Gjallar")?;
    require_eq(
        &empty_run,
        &["stopClassification", "authorityGate"],
        "repo.work.overview",
    )?;
    require_eq(
        &empty_run,
        &["stopClassification", "blocker"],
        "no-repo-work-rows",
    )?;
    require_bool(&empty_run, &["stopClassification", "mutatesState"], false)?;
    require_bool(
        &empty_run,
        &["stopClassification", "requiresElevatedAuthority"],
        false,
    )?;
    require_bool(
        &empty_run,
        &["stopClassification", "privateStateExposed"],
        false,
    )?;

    let item = "gate-proof";
    cargo_json(
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
            "Prove a dry-run swarm pulse can classify a ready-to-run queue row without mutation.",
            "--topic",
            "swarm-stop-classification",
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
            "repo-status-section",
            "--model-ref",
            "swarm-stop-classification-smoke-imagination-v1",
            "--model-authored",
        ],
        &root,
    )?;
    cargo_json(
        &manifest,
        "epiphany-work",
        &["overview", "--workspace", path_str(&repo)?, "--item", item],
        &root,
    )?;
    let dry_run = cargo_json(
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
            "1",
            "--max-items",
            "1",
            "--cooldown-seconds",
            "0",
            "--dry-run",
        ],
        &root,
    )?;
    require_eq(
        &dry_run,
        &["stopClassification", "schemaVersion"],
        "epiphany.repo_swarm_run_stop_classification.v0",
    )?;
    require_eq(
        &dry_run,
        &["stopClassification", "category"],
        "dry-run-preview",
    )?;
    require_eq(&dry_run, &["stopClassification", "owner"], "Self")?;
    require_eq(
        &dry_run,
        &["stopClassification", "authorityGate"],
        "ready-to-run",
    )?;
    require_eq(&dry_run, &["stopClassification", "selectedItem"], item)?;
    require_bool(&dry_run, &["stopClassification", "mutatesState"], false)?;
    require_bool(
        &dry_run,
        &["stopClassification", "requiresElevatedAuthority"],
        false,
    )?;
    require_bool(
        &dry_run,
        &["stopClassification", "privateStateExposed"],
        false,
    )?;

    git(
        [
            "checkout",
            "-B",
            "epiphany/swarm-stop-classification/gate-proof",
        ],
        &repo,
    )?;
    let publication_pipeline = cargo_json(
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
    require_eq(
        &publication_pipeline,
        &["stopClassification", "category"],
        "iteration-limit",
    )?;
    let publication_gate_run = cargo_json(
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
            "1",
            "--max-items",
            "1",
            "--cooldown-seconds",
            "0",
        ],
        &root,
    )?;
    require_eq(
        &publication_gate_run,
        &["stopClassification", "category"],
        "authority-gated",
    )?;
    require_eq(
        &publication_gate_run,
        &["stopClassification", "owner"],
        "Bifrost",
    )?;
    require_eq(
        &publication_gate_run,
        &["stopClassification", "authorityGate"],
        "awaiting-publication",
    )?;
    require_eq(
        &publication_gate_run,
        &["stopClassification", "blocker"],
        "bifrost-publication-missing",
    )?;
    require_bool(
        &publication_gate_run,
        &["stopClassification", "mutatesState"],
        false,
    )?;
    require_bool(
        &publication_gate_run,
        &["stopClassification", "requiresElevatedAuthority"],
        false,
    )?;
    require_bool(
        &publication_gate_run,
        &["stopClassification", "privateStateExposed"],
        false,
    )?;

    let closure_receipt = repo
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
            "--closure-receipt",
            path_str(&closure_receipt)?,
            "--local-verse-store",
            path_str(&local_verse)?,
            "--change-summary",
            "Swarm stop classification publication gate proof.",
            "--justification",
            "Disposable Bifrost proof for gated repo-swarm stop classification.",
            "--ledger-entry-id",
            "swarm-stop-classification-publication-ledger",
            "--pull-request-url",
            "https://example.invalid/GameCult/swarm-stop-classification/pull/1",
            "--pull-request-number",
            "1",
            "--pull-request-title",
            "Swarm stop classification publication gate proof",
        ],
        &root,
    )?;
    require_eq(&publish, &["status"], "publication-receipts-recorded")?;
    cargo_json(
        &manifest,
        "epiphany-work",
        &["overview", "--workspace", path_str(&repo)?, "--item", item],
        &root,
    )?;
    let sync_gate_run = cargo_json(
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
            "1",
            "--max-items",
            "1",
            "--cooldown-seconds",
            "0",
        ],
        &root,
    )?;
    require_eq(
        &sync_gate_run,
        &["stopClassification", "category"],
        "authority-gated",
    )?;
    require_eq(
        &sync_gate_run,
        &["stopClassification", "owner"],
        "Bifrost/GitHub",
    )?;
    require_eq(
        &sync_gate_run,
        &["stopClassification", "authorityGate"],
        "awaiting-upstream-sync",
    )?;
    require_eq(
        &sync_gate_run,
        &["stopClassification", "blocker"],
        "merge-or-sync-receipt-missing",
    )?;
    require_bool(
        &sync_gate_run,
        &["stopClassification", "mutatesState"],
        false,
    )?;
    require_bool(
        &sync_gate_run,
        &["stopClassification", "requiresElevatedAuthority"],
        false,
    )?;
    require_bool(
        &sync_gate_run,
        &["stopClassification", "privateStateExposed"],
        false,
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
            "maintainer-merge:swarm-stop-classification-smoke",
        ],
        &root,
    )?;
    require_eq(&sync, &["status"], "upstream-main-synced")?;

    let summary = json!({
        "schemaVersion": "epiphany.repo_swarm_stop_classification_smoke.v0",
        "status": "ok",
        "smokeDir": smoke_dir,
        "repo": repo,
        "origin": origin,
        "emptyStopCategory": empty_run["stopClassification"]["category"],
        "emptyStopOwner": empty_run["stopClassification"]["owner"],
        "emptyStopGate": empty_run["stopClassification"]["authorityGate"],
        "dryRunStopCategory": dry_run["stopClassification"]["category"],
        "dryRunStopOwner": dry_run["stopClassification"]["owner"],
        "dryRunStopGate": dry_run["stopClassification"]["authorityGate"],
        "dryRunSelectedItem": dry_run["stopClassification"]["selectedItem"],
        "publicationStopCategory": publication_gate_run["stopClassification"]["category"],
        "publicationStopOwner": publication_gate_run["stopClassification"]["owner"],
        "publicationStopGate": publication_gate_run["stopClassification"]["authorityGate"],
        "publicationStopBlocker": publication_gate_run["stopClassification"]["blocker"],
        "syncStopCategory": sync_gate_run["stopClassification"]["category"],
        "syncStopOwner": sync_gate_run["stopClassification"]["owner"],
        "syncStopGate": sync_gate_run["stopClassification"]["authorityGate"],
        "syncStopBlocker": sync_gate_run["stopClassification"]["blocker"],
        "syncStatus": sync["status"],
        "upstreamMainSynced": sync["authority"]["upstreamMainSynced"],
        "mutatesState": false,
        "requiresElevatedAuthority": false,
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
