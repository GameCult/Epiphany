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
        let mut args = env::args().skip(1);
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
    let smoke_dir = smoke_root.join(format!("repo-mind-safe-family-guard-{stamp}"));
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
        "# Repo Mind Safe Family Guard Smoke\n\nThis repository proves Mind refuses alien safe-family labels before Hands authority.\n",
    )
    .with_context(|| format!("failed to seed {}", repo.join("README.md").display()))?;
    git(["add", "README.md"], &repo)?;
    git(
        [
            "commit",
            "-m",
            "Seed repo mind safe family guard smoke body",
        ],
        &repo,
    )?;
    git(
        ["switch", "-c", "epiphany/repo-mind-safe-family-guard"],
        &repo,
    )?;

    let item = "repo-mind-safe-family-guard";
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
            "repo-mind-safe-family-guard-smoke",
            "--topic",
            "repo-mind-safe-family-guard",
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
            "repo-mind-safe-family-guard-smoke",
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
            "Ask Mind to reject a tampered safe-family label before Hands receives authority.",
            "--candidate-action-ref",
            "candidate-action://repo-mind-safe-family-guard/tampered-family",
            "--public-discussion-ref",
            "epiphany-global/persona-collaboration/repo-mind-safe-family-guard",
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
            "repo-mind-safe-family-guard-smoke-imagination-v1",
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
            "notes/epiphany-work/repo-mind-safe-family-guard.md",
        ],
        &root,
    )?;

    let plan_path = value_at_path(&plan, &["receiptPath"])
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("plan output had no receiptPath"))?;
    let mut tampered_plan = read_json(Path::new(plan_path))?;
    tampered_plan["derivation"]["actionItemReceipt"]["safeActionFamily"] =
        json!("idunn.service_lifecycle");
    let tampered_path = smoke_dir.join("tampered-plan.json");
    write_json(&tampered_path, &tampered_plan)?;

    let adopt_failure = cargo_json_expect_failure(
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
            value_at_path(&run, &["receiptPath"])
                .and_then(Value::as_str)
                .ok_or_else(|| anyhow!("run output had no receiptPath"))?,
            "--from-plan",
            path_str(&tampered_path)?,
        ],
        &root,
    )?;
    if !adopt_failure.contains("unsupported repo-work safe family") {
        return Err(anyhow!(
            "adopt failure did not name unsupported safe family: {adopt_failure}"
        ));
    }

    let refusal_path = repo
        .join(".epiphany")
        .join("work")
        .join("work-mind-adopt-repo-mind-safe-family-guard.json");
    let refusal = read_json(&refusal_path)?;
    require_eq(
        &refusal,
        &["schemaVersion"],
        "epiphany.repo_work_mind_adoption_decision.v0",
    )?;
    require_eq(&refusal, &["status"], "refused-unsupported-safe-family")?;
    require_eq(
        &refusal,
        &["interpretation", "inputSummary", "safeActionFamily"],
        "idunn.service_lifecycle",
    )?;
    require_bool(
        &refusal,
        &["interpretation", "classification", "actionItemAccepted"],
        false,
    )?;
    require_bool(
        &refusal,
        &["interpretation", "classification", "safeFamilyRecognized"],
        false,
    )?;
    require_text_field(
        &refusal,
        &["interpretation", "refusalReasons", "0"],
        "Unsupported repo-work safe action family idunn.service_lifecycle",
    )?;
    require_bool(&refusal, &["authority", "handsAuthorityGranted"], false)?;
    require_bool(&refusal, &["authority", "durableStateAdmitted"], false)?;
    require_bool(&refusal, &["authority", "serviceLifecycleAuthority"], false)?;
    require_bool(&refusal, &["authority", "privateStateExposed"], false)?;
    require_eq(
        &refusal,
        &["authority", "nextGate"],
        "imagination.replan_with_allowed_safe_family",
    )?;
    require_bool(&refusal, &["privateStateExposed"], false)?;
    let adoption_receipt_path = repo
        .join(".epiphany")
        .join("work")
        .join("work-adopt-repo-mind-safe-family-guard.json");
    if adoption_receipt_path.exists() {
        return Err(anyhow!(
            "adoption receipt was written despite unsupported safe-family refusal: {}",
            adoption_receipt_path.display()
        ));
    }

    Ok(json!({
        "schemaVersion": "epiphany.repo_work_mind_safe_family_guard_smoke.v0",
        "status": "ok",
        "smokeDir": smoke_dir,
        "repo": repo,
        "item": item,
        "tamperedSafeActionFamily": "idunn.service_lifecycle",
        "mindDecisionStatus": refusal["status"],
        "actionItemAccepted": false,
        "safeFamilyRecognized": false,
        "handsAuthorityGranted": false,
        "durableStateAdmitted": false,
        "serviceLifecycleAuthority": false,
        "adoptionReceiptWritten": false,
        "privateStateExposed": false
    }))
}

fn cargo_json(manifest_path: &Path, bin_name: &str, args: &[&str], cwd: &Path) -> Result<Value> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--bin")
        .arg(bin_name)
        .arg("--")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to spawn cargo run --bin {bin_name}"))?;
    if !output.status.success() {
        return Err(anyhow!(
            "cargo run --bin {bin_name} failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    serde_json::from_slice(&output.stdout)
        .with_context(|| format!("{bin_name} returned invalid JSON"))
}

fn cargo_json_expect_failure(
    manifest_path: &Path,
    bin_name: &str,
    args: &[&str],
    cwd: &Path,
) -> Result<String> {
    let output = Command::new("cargo")
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(manifest_path)
        .arg("--bin")
        .arg(bin_name)
        .arg("--")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to spawn cargo run --bin {bin_name}"))?;
    if output.status.success() {
        return Err(anyhow!("{bin_name} unexpectedly succeeded"));
    }
    Ok(format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

fn git<const N: usize>(args: [&str; N], cwd: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .with_context(|| format!("failed to spawn git in {}", cwd.display()))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "git failed in {}:\n{}",
            cwd.display(),
            String::from_utf8_lossy(&output.stderr)
        ))
    }
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

fn require_text_field(value: &Value, path: &[&str], needle: &str) -> Result<()> {
    let actual = value_at_path(value, path)
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow!("missing string at {}", path.join(".")))?;
    if actual.contains(needle) {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to contain {:?}, got {:?}",
            path.join("."),
            needle,
            actual
        ))
    }
}

fn value_at_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    let mut cursor = value;
    for segment in path {
        if let Ok(index) = segment.parse::<usize>() {
            cursor = cursor.as_array()?.get(index)?;
        } else {
            cursor = cursor.get(*segment)?;
        }
    }
    Some(cursor)
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str()
        .ok_or_else(|| anyhow!("path is not valid UTF-8: {}", path.display()))
}

fn take_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(
        args.next()
            .ok_or_else(|| anyhow!("missing value for {name}"))?,
    ))
}
