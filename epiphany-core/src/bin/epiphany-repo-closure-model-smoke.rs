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
    let smoke_dir = args.smoke_root.join(format!("closure-model-gate-{stamp}"));
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
        "# Closure Model Gate Smoke\n\nThis repository proves model-authored closure verdicts.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    git(["add", "README.md"], &repo)?;
    git(["commit", "-m", "Seed closure model smoke repo"], &repo)?;
    git(["switch", "-c", "epiphany/closure-model-gate"], &repo)?;

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
            "closure-model-gate-smoke",
            "--topic",
            "closure-model-gate",
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
            "closure-model-gate-smoke",
            "--local-verse-store",
            path_str(&local_verse)?,
        ],
        &root,
    )?;

    let good = drive_item(
        &manifest,
        &root,
        &repo,
        "closure-model-pass",
        "Prove a model-authored closure verdict can explicitly pass after deterministic family checks.",
        "passed",
        "Modeling reviewed the verified status section, path scope, family assertions, and authority seal.",
    )?;
    let bad = drive_item(
        &manifest,
        &root,
        &repo,
        "closure-model-block",
        "Prove a model-authored closure verdict can block publication even when deterministic family checks pass.",
        "needs-work",
        "Modeling found the status text too thin and requires another branch-local pass before publication.",
    )?;

    require_eq(&good.close, &["status"], "closed")?;
    require_eq(
        &good.close,
        &["closureReview", "modelingReview", "closureReview", "status"],
        "passed",
    )?;
    require_bool(
        &good.close,
        &[
            "closureReview",
            "modelingReview",
            "closureReview",
            "gateEnforced",
        ],
        true,
    )?;
    require_eq(&bad.close, &["status"], "verification-failed")?;
    require_eq(&bad.close, &["soul", "verdict"], "failed")?;
    require_eq(
        &bad.close,
        &["closureReview", "modelingReview", "closureReview", "status"],
        "failed",
    )?;
    require_bool(
        &bad.close,
        &["closureReview", "modelingReview", "closureReview", "passed"],
        false,
    )?;
    require_eq(
        &bad.close,
        &["closureReview", "familyAssertions", "status"],
        "passed",
    )?;
    require_bool(
        &bad.close,
        &["closureReview", "sourceGrounding", "pathScopeMatched"],
        true,
    )?;
    require_bool(&good.close, &["privateStateExposed"], false)?;
    require_bool(&bad.close, &["privateStateExposed"], false)?;

    let summary = json!({
        "schemaVersion": "epiphany.repo_work_closure_model_gate_smoke.v0",
        "status": "ok",
        "smokeDir": smoke_dir,
        "repo": repo,
        "branch": git_output(["branch", "--show-current"], &repo)?,
        "passItem": good.item,
        "passCloseStatus": good.close["status"],
        "passModelStatus": good.close["closureReview"]["modelingReview"]["closureReview"]["status"],
        "passModelGateEnforced": good.close["closureReview"]["modelingReview"]["closureReview"]["gateEnforced"],
        "blockItem": bad.item,
        "blockCloseStatus": bad.close["status"],
        "blockSoulVerdict": bad.close["soul"]["verdict"],
        "blockModelStatus": bad.close["closureReview"]["modelingReview"]["closureReview"]["status"],
        "blockFamilyAssertionsStatus": bad.close["closureReview"]["familyAssertions"]["status"],
        "blockPathScopeMatched": bad.close["closureReview"]["sourceGrounding"]["pathScopeMatched"],
        "privateStateExposed": false,
    });
    write_json(&smoke_dir.join("summary.json"), &summary)?;
    Ok(summary)
}

struct ItemProof {
    item: String,
    close: Value,
}

fn drive_item(
    manifest: &Path,
    root: &Path,
    repo: &Path,
    item: &str,
    summary: &str,
    model_verdict: &str,
    model_finding: &str,
) -> Result<ItemProof> {
    cargo_json(
        manifest,
        "epiphany-work",
        &[
            "accept",
            "--workspace",
            path_str(repo)?,
            "--from",
            "persona",
            "--item",
            item,
            "--summary",
            summary,
        ],
        root,
    )?;
    cargo_json(
        manifest,
        "epiphany-work",
        &[
            "derive-plan",
            "--workspace",
            path_str(repo)?,
            "--item",
            item,
            "--action-family",
            "repo-status-section",
            "--model-ref",
            "closure-model-gate-smoke-imagination-v1",
            "--model-authored",
        ],
        root,
    )?;
    for _ in 0..3 {
        cargo_json(
            manifest,
            "epiphany-work",
            &[
                "tick",
                "--workspace",
                path_str(repo)?,
                "--epiphany-root",
                path_str(root)?,
                "--item",
                item,
                "--cooldown-seconds",
                "0",
            ],
            root,
        )?;
    }
    let close = cargo_json(
        manifest,
        "epiphany-work",
        &[
            "close",
            "--workspace",
            path_str(repo)?,
            "--item",
            item,
            "--closure-model-ref",
            "closure-model-gate-smoke-soul-v1",
            "--model-authored",
            "--closure-model-verdict",
            model_verdict,
            "--closure-model-finding",
            model_finding,
        ],
        root,
    )?;
    Ok(ItemProof {
        item: item.to_string(),
        close,
    })
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
