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
    artifact_summary: Option<PathBuf>,
    metrics_summary: Option<PathBuf>,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut root = env::current_dir().context("failed to resolve current directory")?;
        let mut smoke_root = root.join(".epiphany-smoke");
        let mut artifact_summary = None;
        let mut metrics_summary = None;
        let mut args = env::args().skip(1).peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--root" => root = take_path(&mut args, "--root")?,
                "--smoke-root" => smoke_root = take_path(&mut args, "--smoke-root")?,
                "--artifact-summary" => {
                    artifact_summary = Some(take_path(&mut args, "--artifact-summary")?)
                }
                "--metrics-summary" => {
                    metrics_summary = Some(take_path(&mut args, "--metrics-summary")?)
                }
                other => return Err(anyhow!("unexpected argument {other:?}")),
            }
        }
        Ok(Self {
            root,
            smoke_root,
            artifact_summary,
            metrics_summary,
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
    let smoke_dir = args
        .smoke_root
        .join(format!("repo-bifrost-accounting-bundle-{stamp}"));
    if smoke_dir.exists() {
        fs::remove_dir_all(&smoke_dir)
            .with_context(|| format!("failed to clear {}", smoke_dir.display()))?;
    }
    fs::create_dir_all(&smoke_dir)
        .with_context(|| format!("failed to create {}", smoke_dir.display()))?;

    let artifact_summary = args.artifact_summary.unwrap_or_else(|| {
        latest_summary(&args.smoke_root, "repo-artifact-acceptance-request-family-")
    });
    let metrics_summary = args
        .metrics_summary
        .unwrap_or_else(|| latest_summary(&args.smoke_root, "repo-metrics-request-family-"));
    let artifact = read_json(&artifact_summary)?;
    let metrics = read_json(&metrics_summary)?;

    require_eq(
        &artifact,
        &["schemaVersion"],
        "epiphany.repo_artifact_acceptance_request_family_smoke.v0",
    )?;
    require_eq(&artifact, &["status"], "ok")?;
    require_eq(
        &artifact,
        &["safeActionFamily"],
        "repo.artifact_acceptance_request",
    )?;
    require_eq(&artifact, &["closeStatus"], "closed")?;
    require_eq(&artifact, &["soulVerdict"], "passed")?;
    require_eq(&artifact, &["familyAssertionsStatus"], "passed")?;
    require_bool(&artifact, &["privateStateExposed"], false)?;
    require_bool(&artifact, &["artifactAcceptanceAuthorized"], false)?;
    require_bool(&artifact, &["handsActionAuthorized"], false)?;
    require_bool(&artifact, &["upstreamSyncAuthorized"], false)?;
    require_eq(
        &artifact,
        &["bifrostArtifactAcceptanceReceiptId"],
        "bifrost-artifact-acceptance-repo-artifact-acceptance-request-family",
    )?;
    require_row(
        &artifact,
        &["bifrostArtifactAcceptanceOpenAccountingRow"],
        "artifact-acceptance-request",
        "open",
        3,
        1,
        None,
    )?;
    require_row(
        &artifact,
        &["bifrostArtifactAcceptanceClosedAccountingRow"],
        "artifact-acceptance-request",
        "closed",
        1,
        1,
        None,
    )?;

    require_eq(
        &metrics,
        &["schemaVersion"],
        "epiphany.repo_metrics_request_family_smoke.v0",
    )?;
    require_eq(&metrics, &["status"], "ok")?;
    require_eq(&metrics, &["safeActionFamily"], "repo.metrics_request")?;
    require_eq(&metrics, &["closeStatus"], "closed")?;
    require_eq(&metrics, &["soulVerdict"], "passed")?;
    require_eq(&metrics, &["familyAssertionsStatus"], "passed")?;
    require_bool(&metrics, &["privateStateExposed"], false)?;
    require_bool(&metrics, &["metricsLedgerAuthorized"], false)?;
    require_bool(&metrics, &["handsActionAuthorized"], false)?;
    require_bool(&metrics, &["upstreamSyncAuthorized"], false)?;
    require_eq(
        &metrics,
        &["bifrostMetricsReceiptId"],
        "bifrost-metrics-repo-metrics-request-family",
    )?;
    require_row(
        &metrics,
        &["bifrostMetricsOpenAccountingRow"],
        "metrics-request",
        "open",
        3,
        1,
        Some(0),
    )?;
    require_row(
        &metrics,
        &["bifrostMetricsClosedAccountingRow"],
        "metrics-request",
        "closed",
        2,
        1,
        Some(1),
    )?;

    let summary = json!({
        "schemaVersion": "epiphany.repo_bifrost_accounting_bundle_smoke.v0",
        "status": "ok",
        "smokeDir": smoke_dir,
        "artifactAcceptanceSummary": artifact_summary,
        "metricsSummary": metrics_summary,
        "artifactAcceptanceSmokeDir": artifact["smokeDir"],
        "metricsSmokeDir": metrics["smokeDir"],
        "artifactAcceptanceFamily": artifact["safeActionFamily"],
        "metricsFamily": metrics["safeActionFamily"],
        "artifactAcceptanceCloseStatus": artifact["closeStatus"],
        "metricsCloseStatus": metrics["closeStatus"],
        "artifactAcceptanceReceiptId": artifact["bifrostArtifactAcceptanceReceiptId"],
        "metricsReceiptId": metrics["bifrostMetricsReceiptId"],
        "artifactAcceptanceClosedAccountingRow": artifact["bifrostArtifactAcceptanceClosedAccountingRow"],
        "metricsClosedAccountingRow": metrics["bifrostMetricsClosedAccountingRow"],
        "operatorSafeProofBundle": true,
        "planningAuthorityOnly": true,
        "privateStateExposed": false,
    });
    write_json(&smoke_dir.join("summary.json"), &summary)?;
    Ok(summary)
}

fn write_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, serde_json::to_vec_pretty(value)?)
        .with_context(|| format!("failed to write {}", path.display()))
}

fn read_json(path: &Path) -> Result<Value> {
    let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_json::from_slice(&bytes).with_context(|| format!("failed to parse {}", path.display()))
}

fn latest_summary(smoke_root: &Path, parent_prefix: &str) -> PathBuf {
    let mut candidates = Vec::new();
    collect_summaries(smoke_root, parent_prefix, &mut candidates);
    candidates
        .into_iter()
        .max_by_key(|(modified, _)| *modified)
        .map(|(_, path)| path)
        .unwrap_or_else(|| smoke_root.join(parent_prefix).join("summary.json"))
}

fn collect_summaries(root: &Path, parent_prefix: &str, summaries: &mut Vec<(u128, PathBuf)>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_summaries(&path, parent_prefix, summaries);
        } else if path.file_name().and_then(|name| name.to_str()) == Some("summary.json")
            && path
                .parent()
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(parent_prefix))
        {
            let modified = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_millis())
                .unwrap_or(0);
            summaries.push((modified, path));
        }
    }
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

fn require_row(
    value: &Value,
    path: &[&str],
    lane: &str,
    status: &str,
    review_receipt_count: u64,
    public_artifact_count: u64,
    credit_receipt_count: Option<u64>,
) -> Result<()> {
    let row = path
        .iter()
        .try_fold(value, |current, key| current.get(*key))
        .ok_or_else(|| anyhow!("missing accounting row {}", path.join(".")))?;
    let actual_lane = row
        .get("lane")
        .and_then(Value::as_str)
        .unwrap_or("<missing>");
    let actual_status = row
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("<missing>");
    let actual_review_receipt_count = row
        .get("reviewReceiptCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let actual_public_artifact_count = row
        .get("publicArtifactCount")
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let actual_credit_receipt_count = row.get("creditReceiptCount").and_then(Value::as_u64);
    let actual_private = row
        .get("privateStateExposed")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let credit_matches = credit_receipt_count
        .is_none_or(|expected| actual_credit_receipt_count.unwrap_or(0) == expected);
    if actual_lane == lane
        && actual_status == status
        && actual_review_receipt_count == review_receipt_count
        && actual_public_artifact_count == public_artifact_count
        && credit_matches
        && !actual_private
    {
        Ok(())
    } else {
        Err(anyhow!(
            "accounting row {} mismatch: lane={actual_lane:?}, status={actual_status:?}, reviewReceiptCount={actual_review_receipt_count}, publicArtifactCount={actual_public_artifact_count}, creditReceiptCount={actual_credit_receipt_count:?}, privateStateExposed={actual_private}",
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
