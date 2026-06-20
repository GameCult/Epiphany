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
        .join(format!("repo-close-mind-adoption-guard-{stamp}"));
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
        "# Repo Close Mind Adoption Guard Smoke\n\nThis repository proves Soul refuses closure when the Mind adoption proof is tampered.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    git(["add", "README.md"], &repo)?;
    git(
        [
            "commit",
            "-m",
            "Seed repo close mind adoption guard smoke body",
        ],
        &repo,
    )?;
    git(
        ["switch", "-c", "epiphany/repo-close-mind-adoption-guard"],
        &repo,
    )?;

    let item = "repo-close-mind-adoption-guard";
    let target_path = "notes/epiphany-work/repo-close-mind-adoption-guard.md";
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
            "repo-close-mind-adoption-guard-smoke",
            "--topic",
            "repo-close-mind-adoption-guard",
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
            "repo-close-mind-adoption-guard-smoke",
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
            "Ask Soul to reject counterfeit Mind adoption proof before closure.",
            "--candidate-action-ref",
            "candidate-action://repo-close-mind-adoption-guard/tampered-adoption",
            "--public-discussion-ref",
            "epiphany-global/persona-collaboration/repo-close-mind-adoption-guard",
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
            "planning-note",
            "--model-ref",
            "repo-close-mind-adoption-guard-smoke-imagination-v1",
            "--model-authored",
        ],
        &root,
    )?;
    let run = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "run",
            "--workspace",
            path_str(&repo)?,
            "--epiphany-root",
            path_str(&root)?,
            "--item",
            item,
            "--path",
            target_path,
        ],
        &root,
    )?;
    let plan_path = value_at_path(&plan, &["receiptPath"])
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("plan output had no receiptPath"))?;
    let run_path = value_at_path(&run, &["receiptPath"])
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("run output had no receiptPath"))?;
    let adopt = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "adopt",
            "--workspace",
            path_str(&repo)?,
            "--epiphany-root",
            path_str(&root)?,
            "--item",
            item,
            "--run-receipt",
            run_path,
            "--from-plan",
            plan_path,
        ],
        &root,
    )?;
    let adopt_path = value_at_path(&adopt, &["receiptPath"])
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("adopt output had no receiptPath"))?;
    let execute = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "execute",
            "--workspace",
            path_str(&repo)?,
            "--epiphany-root",
            path_str(&root)?,
            "--item",
            item,
            "--adopt-receipt",
            adopt_path,
            "--from-plan",
            plan_path,
        ],
        &root,
    )?;
    let execute_path = value_at_path(&execute, &["receiptPath"])
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("execute output had no receiptPath"))?;

    let mut tampered_adopt = read_json(Path::new(adopt_path))?;
    tampered_adopt["mindAdoptionDecision"]["status"] = json!("tampered-after-adoption");
    tampered_adopt["mindAdoptionDecision"]["interpretation"]["classification"]["actionItemAccepted"] =
        json!(false);
    let tampered_adopt_path = smoke_dir.join("tampered-adopt.json");
    write_json(&tampered_adopt_path, &tampered_adopt)?;

    let mut tampered_execute = read_json(Path::new(execute_path))?;
    tampered_execute["adoptReceiptPath"] = json!(path_str(&tampered_adopt_path)?);
    let tampered_execute_path = smoke_dir.join("tampered-execute.json");
    write_json(&tampered_execute_path, &tampered_execute)?;

    let tampered_close = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "close",
            "--workspace",
            path_str(&repo)?,
            "--item",
            item,
            "--execute-receipt",
            path_str(&tampered_execute_path)?,
            "--verification-command",
            "git status --short",
        ],
        &root,
    )?;
    require_eq(&tampered_close, &["status"], "verification-failed")?;
    require_eq(&tampered_close, &["soul", "verdict"], "failed")?;
    require_eq(
        &tampered_close,
        &["closureReview", "mindAdoptionReview", "status"],
        "failed",
    )?;
    require_bool(
        &tampered_close,
        &["closureReview", "sourceGrounding", "mindAdoptionPassed"],
        false,
    )?;
    require_bool(
        &tampered_close,
        &["closureReview", "mindAdoptionReview", "privateStateExposed"],
        false,
    )?;
    require_assertion(
        &tampered_close,
        &["closureReview", "mindAdoptionReview", "assertions"],
        "decision-status-adopted",
        false,
    )?;
    require_assertion(
        &tampered_close,
        &["closureReview", "mindAdoptionReview", "assertions"],
        "action-item-accepted",
        false,
    )?;

    let close = cargo_json(
        &manifest,
        "epiphany-work",
        &[
            "close",
            "--workspace",
            path_str(&repo)?,
            "--item",
            item,
            "--execute-receipt",
            execute_path,
            "--verification-command",
            "git status --short",
        ],
        &root,
    )?;
    require_eq(&close, &["status"], "closed")?;
    require_eq(&close, &["soul", "verdict"], "passed")?;
    require_eq(
        &close,
        &["closureReview", "mindAdoptionReview", "status"],
        "passed",
    )?;
    require_bool(
        &close,
        &["closureReview", "sourceGrounding", "mindAdoptionPassed"],
        true,
    )?;
    require_bool(&close, &["privateStateExposed"], false)?;

    let summary = json!({
        "schemaVersion": "epiphany.repo_close_mind_adoption_guard_smoke.v0",
        "status": "ok",
        "smokeDir": smoke_dir,
        "repo": repo,
        "branch": git_output(["branch", "--show-current"], &repo)?,
        "item": item,
        "targetPath": target_path,
        "tamperedCloseStatus": tampered_close["status"],
        "tamperedSoulVerdict": tampered_close["soul"]["verdict"],
        "tamperedMindAdoptionReviewStatus": tampered_close["closureReview"]["mindAdoptionReview"]["status"],
        "closeStatus": close["status"],
        "soulVerdict": close["soul"]["verdict"],
        "mindAdoptionReviewStatus": close["closureReview"]["mindAdoptionReview"]["status"],
        "publicationAuthorized": false,
        "privateStateExposed": false
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
    if !output.status.success() {
        return Err(anyhow!(
            "git failed in {} with status {:?}\nstdout:\n{}\nstderr:\n{}",
            cwd.display(),
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn read_json(path: &Path) -> Result<Value> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("failed to decode {}", path.display()))
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
    let actual = value_at_path(value, path)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing string at {}", path.join(".")))?;
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to be {:?}, got {:?}",
            path.join("."),
            expected,
            actual
        ))
    }
}

fn require_bool(value: &Value, path: &[&str], expected: bool) -> Result<()> {
    let actual = value_at_path(value, path)
        .and_then(Value::as_bool)
        .ok_or_else(|| anyhow!("missing bool at {}", path.join(".")))?;
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to be {}, got {}",
            path.join("."),
            expected,
            actual
        ))
    }
}

fn require_assertion(
    value: &Value,
    assertions_path: &[&str],
    assertion_id: &str,
    expected: bool,
) -> Result<()> {
    let assertions = value_at_path(value, assertions_path)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("missing assertions at {}", assertions_path.join(".")))?;
    let assertion = assertions
        .iter()
        .find(|entry| entry.get("assertionId").and_then(Value::as_str) == Some(assertion_id))
        .ok_or_else(|| anyhow!("missing assertion {assertion_id}"))?;
    require_bool(assertion, &["passed"], expected)
}

fn value_at_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut cursor = value;
    for segment in path {
        cursor = cursor.get(*segment)?;
    }
    Some(cursor)
}

fn take_path<I>(args: &mut std::iter::Peekable<I>, name: &str) -> Result<PathBuf>
where
    I: Iterator<Item = String>,
{
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("missing value for {name}"))
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| anyhow!("path is not valid UTF-8: {}", path.display()))
}
