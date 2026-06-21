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

#[derive(Debug)]
struct Args {
    workspace: PathBuf,
    item: String,
    expected_path: String,
    expected_family: String,
    expected_gate: String,
    expected_blocker: String,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut workspace = env::current_dir().context("failed to resolve current directory")?;
        let mut item = None;
        let mut expected_path = "README.md".to_string();
        let mut expected_family = "repo.status_section".to_string();
        let mut expected_gate = "awaiting-publication".to_string();
        let mut expected_blocker = "bifrost-publication-missing".to_string();
        let mut args = env::args().skip(1).peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--workspace" => workspace = take_path(&mut args, "--workspace")?,
                "--item" => item = Some(take_value(&mut args, "--item")?),
                "--expected-path" => expected_path = take_value(&mut args, "--expected-path")?,
                "--expected-family" => {
                    expected_family = take_value(&mut args, "--expected-family")?
                }
                "--expected-gate" => expected_gate = take_value(&mut args, "--expected-gate")?,
                "--expected-blocker" => {
                    expected_blocker = take_value(&mut args, "--expected-blocker")?
                }
                other => return Err(anyhow!("unexpected argument {other:?}")),
            }
        }
        let item = item.ok_or_else(|| anyhow!("--item is required"))?;
        Ok(Self {
            workspace,
            item,
            expected_path,
            expected_family,
            expected_gate,
            expected_blocker,
        })
    }
}

fn run_smoke(args: Args) -> Result<Value> {
    let workspace = args
        .workspace
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", args.workspace.display()))?;
    let work_dir = workspace.join(".epiphany").join("work");
    let close_path = work_dir.join(format!("work-close-{}.json", args.item));
    let overview_path = work_dir.join(format!("work-overview-{}.json", args.item));
    let close = read_json(&close_path)?;
    let overview = read_json(&overview_path)?;

    require_str(
        &close,
        &["schemaVersion"],
        "epiphany.repo_work_closure_receipt.v0",
    )?;
    require_str(&close, &["status"], "closed")?;
    require_str(&close, &["item"], &args.item)?;
    require_bool(&close, &["privateStateExposed"], false)?;
    require_str(&close, &["soul", "verdict"], "passed")?;
    require_bool(&close, &["authority", "branchLocalOnly"], true)?;
    require_bool(&close, &["authority", "publicationAuthorized"], false)?;
    require_bool(&close, &["authority", "mergeAuthorized"], false)?;
    require_bool(&close, &["authority", "serviceLifecycleAuthorized"], false)?;
    require_bool(&close, &["authority", "crossRepoMutationAuthorized"], false)?;
    require_bool(&close, &["authority", "privateStateExposed"], false)?;
    require_str(
        &close,
        &["mind", "repoMapEntry", "safeActionFamily"],
        &args.expected_family,
    )?;
    require_str(
        &close,
        &["mind", "repoMapEntry", "publicationGate"],
        "Bifrost",
    )?;
    let commit_sha = require_present_str(&close, &["handsReceipts", "commitSha"])?;
    require_single_path(
        &close,
        &["mind", "repoMapEntry", "changedPaths"],
        &args.expected_path,
    )?;
    require_single_path(
        &close,
        &["closureReview", "sourceGrounding", "actualChangedPaths"],
        &args.expected_path,
    )?;
    require_bool(
        &close,
        &["closureReview", "sourceGrounding", "pathScopeMatched"],
        true,
    )?;
    require_bool(
        &close,
        &["closureReview", "sourceGrounding", "familyAssertionsPassed"],
        true,
    )?;
    require_bool(
        &close,
        &["closureReview", "sourceGrounding", "mindAdoptionPassed"],
        true,
    )?;

    let overview_schema = require_one_of_str(
        &overview,
        &["schemaVersion"],
        &[
            "epiphany.repo_work_overview.v0",
            "epiphany.repo_work_overview_receipt.v0",
        ],
    )?;
    if overview_schema == "epiphany.repo_work_overview.v0" {
        require_str(&overview, &["status"], "overview-ready")?;
    }
    require_str(&overview, &["item"], &args.item)?;
    require_str(&overview, &["currentGate"], &args.expected_gate)?;
    require_str(&overview, &["blocker"], &args.expected_blocker)?;
    require_bool(&overview, &["privateStateExposed"], false)?;
    require_str(&overview, &["proofBundle", "soulVerdict"], "passed")?;
    require_single_path(
        &overview,
        &["proofBundle", "changedPaths"],
        &args.expected_path,
    )?;

    let git_paths = git_changed_paths(&workspace, commit_sha)?;
    if git_paths != vec![args.expected_path.clone()] {
        return Err(anyhow!(
            "expected git commit {commit_sha} to change only {:?}, got {:?}",
            args.expected_path,
            git_paths
        ));
    }

    Ok(json!({
        "schemaVersion": "epiphany.repo_livefire_closure_smoke.v0",
        "status": "ok",
        "workspace": workspace,
        "item": args.item,
        "branch": string_at(&close, &["mind", "repoMapEntry", "branch"]),
        "commitSha": commit_sha,
        "safeActionFamily": args.expected_family,
        "changedPaths": git_paths,
        "currentGate": args.expected_gate,
        "blocker": args.expected_blocker,
        "publicationGate": "Bifrost",
        "soulVerdict": "passed",
        "mindStateCommitReceiptId": string_at(&close, &["mind", "stateCommitReceiptId"]),
        "closeReceipt": close_path,
        "overviewReceipt": overview_path,
        "verifiedAt": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        "privateStateExposed": false,
    }))
}

fn read_json(path: &Path) -> Result<Value> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("failed to parse {}", path.display()))
}

fn take_value(
    args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    name: &str,
) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("missing value for {name}"))
}

fn take_path(
    args: &mut std::iter::Peekable<impl Iterator<Item = String>>,
    name: &str,
) -> Result<PathBuf> {
    Ok(PathBuf::from(take_value(args, name)?))
}

fn value_at<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
    path.iter()
        .try_fold(value, |current, key| current.get(*key))
}

fn string_at(value: &Value, path: &[&str]) -> String {
    value_at(value, path)
        .and_then(Value::as_str)
        .unwrap_or("<missing>")
        .to_string()
}

fn require_present_str<'a>(value: &'a Value, path: &[&str]) -> Result<&'a str> {
    value_at(value, path)
        .and_then(Value::as_str)
        .filter(|actual| !actual.is_empty())
        .ok_or_else(|| anyhow!("expected {} to be a non-empty string", path.join(".")))
}

fn require_str(value: &Value, path: &[&str], expected: &str) -> Result<()> {
    let actual = string_at(value, path);
    if actual == expected {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to be {expected:?}, got {actual:?}",
            path.join(".")
        ))
    }
}

fn require_one_of_str<'a>(value: &Value, path: &[&str], expected: &'a [&str]) -> Result<&'a str> {
    let actual = string_at(value, path);
    if let Some(matched) = expected.iter().find(|candidate| **candidate == actual) {
        Ok(*matched)
    } else {
        Err(anyhow!(
            "expected {} to be one of {:?}, got {actual:?}",
            path.join("."),
            expected
        ))
    }
}

fn require_bool(value: &Value, path: &[&str], expected: bool) -> Result<()> {
    let actual = value_at(value, path).and_then(Value::as_bool);
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

fn require_single_path(value: &Value, path: &[&str], expected: &str) -> Result<()> {
    let paths = value_at(value, path)
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("expected {} to be an array", path.join(".")))?
        .iter()
        .map(|entry| entry.as_str().unwrap_or("<non-string>").to_string())
        .collect::<Vec<_>>();
    if paths == vec![expected.to_string()] {
        Ok(())
    } else {
        Err(anyhow!(
            "expected {} to contain only {expected:?}, got {:?}",
            path.join("."),
            paths
        ))
    }
}

fn git_changed_paths(workspace: &Path, commit_sha: &str) -> Result<Vec<String>> {
    let output = Command::new("git")
        .arg("-C")
        .arg(workspace)
        .arg("show")
        .arg("--format=")
        .arg("--name-only")
        .arg(commit_sha)
        .output()
        .with_context(|| format!("failed to run git show in {}", workspace.display()))?;
    if !output.status.success() {
        return Err(anyhow!(
            "git show failed with exit {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(|line| line.replace('\\', "/"))
        .collect())
}
