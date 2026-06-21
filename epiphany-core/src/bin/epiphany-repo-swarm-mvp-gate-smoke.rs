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
    fresh_repo_summary: Option<PathBuf>,
    readiness_summary: Option<PathBuf>,
    bifrost_accounting_summary: Option<PathBuf>,
    deployment_handoff_summary: Option<PathBuf>,
    daemon_survival_summary: Option<PathBuf>,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut root = env::current_dir().context("failed to resolve current directory")?;
        let mut smoke_root = root.join(".epiphany-smoke");
        let mut fresh_repo_summary = None;
        let mut readiness_summary = None;
        let mut bifrost_accounting_summary = None;
        let mut deployment_handoff_summary = None;
        let mut daemon_survival_summary = None;
        let mut args = env::args().skip(1).peekable();
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--root" => root = take_path(&mut args, "--root")?,
                "--smoke-root" => smoke_root = take_path(&mut args, "--smoke-root")?,
                "--fresh-repo-summary" => {
                    fresh_repo_summary = Some(take_path(&mut args, "--fresh-repo-summary")?)
                }
                "--readiness-summary" => {
                    readiness_summary = Some(take_path(&mut args, "--readiness-summary")?)
                }
                "--bifrost-accounting-summary" => {
                    bifrost_accounting_summary =
                        Some(take_path(&mut args, "--bifrost-accounting-summary")?)
                }
                "--deployment-handoff-summary" => {
                    deployment_handoff_summary =
                        Some(take_path(&mut args, "--deployment-handoff-summary")?)
                }
                "--daemon-survival-summary" => {
                    daemon_survival_summary =
                        Some(take_path(&mut args, "--daemon-survival-summary")?)
                }
                other => return Err(anyhow!("unexpected argument {other:?}")),
            }
        }
        Ok(Self {
            root,
            smoke_root,
            fresh_repo_summary,
            readiness_summary,
            bifrost_accounting_summary,
            deployment_handoff_summary,
            daemon_survival_summary,
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
    let smoke_dir = args.smoke_root.join(format!("repo-swarm-mvp-gate-{stamp}"));
    if smoke_dir.exists() {
        fs::remove_dir_all(&smoke_dir)
            .with_context(|| format!("failed to clear {}", smoke_dir.display()))?;
    }
    fs::create_dir_all(&smoke_dir)
        .with_context(|| format!("failed to create {}", smoke_dir.display()))?;

    let fresh_repo_summary = args
        .fresh_repo_summary
        .unwrap_or_else(|| latest_summary(&args.smoke_root, "fresh-repo-mvp-"));
    let readiness_summary = args
        .readiness_summary
        .unwrap_or_else(|| latest_summary(&args.smoke_root, "repo-work-readiness-"));
    let bifrost_accounting_summary = args
        .bifrost_accounting_summary
        .unwrap_or_else(|| latest_summary(&args.smoke_root, "repo-bifrost-accounting-bundle-"));
    let deployment_handoff_summary = args
        .deployment_handoff_summary
        .unwrap_or_else(|| latest_summary(&args.smoke_root, "repo-deployment-config-family-"));
    let daemon_survival_summary = args
        .daemon_survival_summary
        .unwrap_or_else(|| latest_summary(&args.smoke_root, "daemon-survival-rehearsal-"));

    let fresh = read_json(&fresh_repo_summary)?;
    let readiness = read_json(&readiness_summary)?;
    let bifrost = read_json(&bifrost_accounting_summary)?;
    let deployment_handoff = read_json(&deployment_handoff_summary)?;
    let daemon_survival = read_json(&daemon_survival_summary)?;

    verify_fresh_repo(&fresh)?;
    verify_readiness(&readiness)?;
    verify_bifrost_accounting(&bifrost)?;
    verify_deployment_handoff(&deployment_handoff)?;
    verify_daemon_survival(&daemon_survival)?;

    let gate_rows = vec![
        green(
            "fresh-repo-body",
            "Hands",
            "epiphany-repo init + branch + commit receipt",
        ),
        green("swarm-online", "Self", "epiphany-swarm online"),
        green("persona-intake", "Persona", "persona-intake work item"),
        green(
            "imagination-plan",
            "Imagination",
            "model-authored repo-status-section plan",
        ),
        green(
            "self-scheduler-stop",
            "Self",
            "iteration-limit stop classification is explicit and nonmutating",
        ),
        green(
            "soul-closure",
            "Soul",
            "closed item with passed verdict and family assertions",
        ),
        green(
            "upstream-main-sync",
            "Hands",
            "publish + git push/fetch + sync reports upstream main synced",
        ),
        green(
            "public-proof",
            "Bifrost",
            "operator-safe public proof exported without private state",
        ),
        green(
            "readiness-review",
            "Soul",
            "readiness report and review receipt are present",
        ),
        green(
            "receipt-directory",
            "Bifrost",
            "readiness review is queryable through the receipt directory",
        ),
        green(
            "bifrost-accounting",
            "Bifrost",
            "readiness, artifact acceptance, and metrics accounting rows are closed",
        ),
        green(
            "idunn-deployment-handoff",
            "Idunn",
            "deployment config, operator runbook, and aftercare receipt readback are sealed",
        ),
        green(
            "long-running-daemon-proof",
            "Idunn",
            "bounded serve rehearsal wrote two scheduler pulses and sealed scheduler receipt",
        ),
        warning(
            "idunn-elevated-service",
            "Idunn",
            "service install/start/remote deployment execution remains explicit operator authority",
        ),
    ];
    let green_gate_count = gate_rows
        .iter()
        .filter(|row| row["status"] == "green")
        .count();
    let warning_gate_count = gate_rows
        .iter()
        .filter(|row| row["status"] == "warning")
        .count();
    let blocker_gate_count = gate_rows
        .iter()
        .filter(|row| row["status"] == "blocker")
        .count();
    let gate_tui_rows = gate_rows
        .iter()
        .map(|row| {
            format!(
                "MVP-GATE | {} | status={} | owner={} | evidence={}",
                row["gate"].as_str().unwrap_or("unknown"),
                row["status"].as_str().unwrap_or("unknown"),
                row["owner"].as_str().unwrap_or("unknown"),
                row["evidence"].as_str().unwrap_or("unknown")
            )
        })
        .collect::<Vec<_>>();

    let summary = json!({
        "schemaVersion": "epiphany.repo_swarm_mvp_gate_smoke.v0",
        "status": "mvp-demo-ready-with-known-operator-gates",
        "smokeDir": smoke_dir,
        "freshRepoSummary": fresh_repo_summary,
        "readinessSummary": readiness_summary,
        "bifrostAccountingSummary": bifrost_accounting_summary,
        "deploymentHandoffSummary": deployment_handoff_summary,
        "daemonSurvivalSummary": daemon_survival_summary,
        "freshRepoSmokeDir": fresh["smokeDir"],
        "readinessSmokeDir": readiness["smokeDir"],
        "bifrostAccountingSmokeDir": bifrost["smokeDir"],
        "deploymentHandoffSmokeDir": deployment_handoff["smokeDir"],
        "daemonSurvivalSmokeDir": daemon_survival["smokeDir"],
        "mvpReady": false,
        "demoReady": true,
        "greenGateCount": green_gate_count,
        "warningGateCount": warning_gate_count,
        "blockerGateCount": blocker_gate_count,
        "gateRows": gate_rows,
        "gateTuiRows": gate_tui_rows,
        "knownRemainingAuthorityGates": [
            "Idunn elevated service install/start/remote deployment execution"
        ],
        "privateStateExposed": false,
    });
    write_json(&smoke_dir.join("summary.json"), &summary)?;
    Ok(summary)
}

fn verify_fresh_repo(value: &Value) -> Result<()> {
    require_eq(
        value,
        &["schemaVersion"],
        "epiphany.repo_swarm_fresh_repo_mvp_smoke.v0",
    )?;
    require_eq(value, &["status"], "ok")?;
    require_eq(value, &["onlineStatus"], "attention")?;
    require_eq(
        value,
        &["personaIntakeStatus"],
        "accepted-for-imagination-consensus",
    )?;
    require_eq(value, &["planStatus"], "planned-for-self-adoption")?;
    require_eq(value, &["swarmRunStopCategory"], "iteration-limit")?;
    require_eq(value, &["swarmRunStopOwner"], "Self")?;
    require_eq(
        value,
        &["swarmRunStopGate"],
        "self.scheduler-iteration-limit",
    )?;
    require_eq(value, &["closeStatus"], "closed")?;
    require_eq(value, &["soulVerdict"], "passed")?;
    require_eq(value, &["familyAssertionsStatus"], "passed")?;
    require_eq(value, &["publishStatus"], "publication-receipts-recorded")?;
    require_eq(value, &["syncStatus"], "upstream-main-synced")?;
    require_bool(value, &["upstreamMainSynced"], true)?;
    require_eq(value, &["publicProofStatus"], "public-proof-exported")?;
    require_bool(value, &["privateStateExposed"], false)
}

fn verify_readiness(value: &Value) -> Result<()> {
    require_eq(
        value,
        &["schemaVersion"],
        "epiphany.repo_work_readiness_smoke.v0",
    )?;
    require_eq(value, &["status"], "ok")?;
    require_eq(value, &["readinessStatus"], "ready")?;
    require_eq(value, &["readinessReviewStatus"], "readiness-approved")?;
    require_eq(value, &["publishStatus"], "publication-receipts-recorded")?;
    require_eq(value, &["syncStatus"], "upstream-main-synced")?;
    require_bool(value, &["upstreamMainSynced"], true)?;
    require_bool(value, &["privateStateExposed"], false)?;
    require_bool(value, &["sightOnly"], true)?;
    require_bool(value, &["readinessApprovalAuthorized"], false)?;
    require_bool(value, &["publicationAuthorized"], false)?;
    require_bool(value, &["deploymentAuthority"], false)?;
    require_bool(value, &["serviceLifecycleAuthority"], false)?;
    require_bool(value, &["handsActionAuthorized"], false)?;
    require_row(
        value,
        &["bifrostReadinessReviewAccountingRow"],
        "repo-work-readiness-review",
        "closed",
        4,
        1,
    )
}

fn verify_bifrost_accounting(value: &Value) -> Result<()> {
    require_eq(
        value,
        &["schemaVersion"],
        "epiphany.repo_bifrost_accounting_bundle_smoke.v0",
    )?;
    require_eq(value, &["status"], "ok")?;
    require_eq(value, &["artifactAcceptanceCloseStatus"], "closed")?;
    require_eq(value, &["metricsCloseStatus"], "closed")?;
    require_bool(value, &["operatorSafeProofBundle"], true)?;
    require_bool(value, &["planningAuthorityOnly"], true)?;
    require_bool(value, &["privateStateExposed"], false)?;
    require_row(
        value,
        &["artifactAcceptanceClosedAccountingRow"],
        "artifact-acceptance-request",
        "closed",
        1,
        1,
    )?;
    require_row(
        value,
        &["metricsClosedAccountingRow"],
        "metrics-request",
        "closed",
        2,
        1,
    )
}

fn verify_daemon_survival(value: &Value) -> Result<()> {
    require_eq(
        value,
        &["schemaVersion"],
        "epiphany.daemon_survival_rehearsal_smoke.v0",
    )?;
    require_eq(value, &["status"], "ok")?;
    require_eq(value, &["policyStatus"], "written")?;
    require_eq(value, &["serveStatus"], "serveComplete")?;
    require_eq(value, &["schedulerReceiptStatus"], "tickComplete")?;
    require_bool(value, &["boundedProofMode"], true)?;
    require_bool(value, &["serviceManagerMutated"], false)?;
    require_bool(value, &["requiresElevatedAuthority"], false)?;
    require_bool(value, &["unattendedDaemonSurvivalRehearsed"], true)?;
    require_bool(value, &["privateStateExposed"], false)
}

fn verify_deployment_handoff(value: &Value) -> Result<()> {
    require_eq(
        value,
        &["schemaVersion"],
        "epiphany.repo_deployment_config_family_smoke.v0",
    )?;
    require_eq(value, &["status"], "ok")?;
    require_bool(value, &["deploymentConfigEnabled"], false)?;
    require_eq(
        value,
        &["deploymentConfigAuditStatus"],
        "ready-for-idunn-review",
    )?;
    require_bool(value, &["readyForIdunnReview"], true)?;
    require_eq(
        value,
        &["deploymentExecutionRunbookStatus"],
        "ready-for-operator-git-push",
    )?;
    require_eq(value, &["deploymentAftercareAuditStatus"], "complete")?;
    require_bool(value, &["deploymentComplete"], true)?;
    require_eq(value, &["idunnDeploymentReceiptSource"], "cultmesh")?;
    require_eq(value, &["idunnAftercareAuditReceiptSource"], "cultmesh")?;
    require_eq(value, &["deploymentTrigger"], "git-push-observed-by-idunn")?;
    require_eq(value, &["deploymentOwner"], "Idunn")?;
    require_bool(value, &["deploymentAuthority"], false)?;
    require_bool(value, &["privateStateExposed"], false)
}

fn green(gate: &str, owner: &str, evidence: &str) -> Value {
    json!({
        "gate": gate,
        "status": "green",
        "owner": owner,
        "evidence": evidence,
        "privateStateExposed": false,
    })
}

fn warning(gate: &str, owner: &str, evidence: &str) -> Value {
    json!({
        "gate": gate,
        "status": "warning",
        "owner": owner,
        "evidence": evidence,
        "privateStateExposed": false,
    })
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
    let actual_private = row
        .get("privateStateExposed")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    if actual_lane == lane
        && actual_status == status
        && actual_review_receipt_count == review_receipt_count
        && actual_public_artifact_count == public_artifact_count
        && !actual_private
    {
        Ok(())
    } else {
        Err(anyhow!(
            "accounting row {} mismatch: lane={actual_lane:?}, status={actual_status:?}, reviewReceiptCount={actual_review_receipt_count}, publicArtifactCount={actual_public_artifact_count}, privateStateExposed={actual_private}",
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
