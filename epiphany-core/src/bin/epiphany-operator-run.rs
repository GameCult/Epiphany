use anyhow::Context;
use anyhow::Result;
use chrono::Utc;
use epiphany_core::EPIPHANY_CULTMESH_INTERNAL_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION;
use epiphany_core::EpiphanyCultMeshOperatorRunIntentEntry;
use epiphany_core::EpiphanyCultMeshOperatorRunReceiptEntry;
use epiphany_core::epiphany_cultmesh_coordinator_run_receipt_from_summary_json;
use epiphany_core::epiphany_cultmesh_hands_action_gate_from_summary_json;
use epiphany_core::epiphany_cultmesh_role_review_event_from_summary_json;
use epiphany_core::load_epiphany_cultmesh_operator_run_intent;
use epiphany_core::load_latest_epiphany_cultmesh_operator_run_intent;
use epiphany_core::load_latest_epiphany_cultmesh_operator_run_receipt;
use epiphany_core::write_epiphany_cultmesh_coordinator_run_receipt;
use epiphany_core::write_epiphany_cultmesh_hands_action_gate;
use epiphany_core::write_epiphany_cultmesh_operator_run_intent;
use epiphany_core::write_epiphany_cultmesh_operator_run_receipt;
use epiphany_core::write_epiphany_cultmesh_role_review_event;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args = Args::parse()?;
    match args.command.as_str() {
        "intent" => {
            let intent = EpiphanyCultMeshOperatorRunIntentEntry {
                schema_version: EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION.to_string(),
                runtime_id: args.runtime_id.clone(),
                verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
                run_id: args.run_id.clone(),
                requested_at_utc: Utc::now().to_rfc3339(),
                mode: args.mode.clone(),
                root: args.root.clone(),
                workspace: args.workspace.clone(),
                thread_id: args.thread_id.clone(),
                codex_home: args.codex_home.clone(),
                target_dir: args.target_dir.clone(),
                max_steps: args.max_steps,
                timeout_seconds: args.timeout_seconds,
                auto_review: args.auto_review,
                no_ephemeral: args.no_ephemeral,
                artifact_root: args.artifact_root.clone(),
                dogfood_root: args.dogfood_root.clone(),
            };
            let written = write_epiphany_cultmesh_operator_run_intent(&args.store, intent)?;
            print_json(json!({"status": "written", "store": args.store, "intent": written}))?;
        }
        "receipt" => {
            let intent = load_epiphany_cultmesh_operator_run_intent(
                &args.store,
                &args.runtime_id,
                &args.run_id,
            )?
            .context("operator-run receipt requires a persisted intent for its run id")?;
            if intent.mode != args.mode {
                anyhow::bail!(
                    "operator-run receipt mode does not match its intent: run={} requested mode={} intent mode={}",
                    args.run_id,
                    args.mode,
                    intent.mode
                );
            }
            let result_path = canonical_file(&args.result_path, "--result-path")?;
            let artifact_root = canonical_directory(&args.artifact_root, "--artifact-root")?;
            if !result_path.starts_with(&artifact_root) {
                anyhow::bail!(
                    "operator-run result {} is outside artifact root {}",
                    result_path.display(),
                    artifact_root.display()
                );
            }
            let result_source = fs::read_to_string(&result_path).with_context(|| {
                format!("failed to read result artifact {}", result_path.display())
            })?;
            let _: Value = serde_json::from_str(result_source.trim_start_matches('\u{feff}'))
                .with_context(|| {
                    format!(
                        "operator-run result is not valid JSON: {}",
                        result_path.display()
                    )
                })?;
            let requested_at = chrono::DateTime::parse_from_rfc3339(&intent.requested_at_utc)
                .context("operator-run intent has invalid requested_at_utc")?
                .with_timezone(&Utc);
            let modified_at: chrono::DateTime<Utc> = fs::metadata(&result_path)?
                .modified()
                .context("operator-run result has no modification time")?
                .into();
            if modified_at < requested_at {
                anyhow::bail!(
                    "operator-run result predates its intent: result={} intent={}",
                    modified_at.to_rfc3339(),
                    requested_at.to_rfc3339()
                );
            }
            let mut artifact_refs = Vec::new();
            push_non_empty(&mut artifact_refs, &args.result_path);
            push_non_empty(&mut artifact_refs, &args.artifact_root);
            push_non_empty(&mut artifact_refs, &args.dogfood_root);
            let receipt = EpiphanyCultMeshOperatorRunReceiptEntry {
                schema_version: EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION.to_string(),
                runtime_id: args.runtime_id.clone(),
                verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
                run_id: args.run_id.clone(),
                completed_at_utc: Utc::now().to_rfc3339(),
                mode: args.mode.clone(),
                status: "completed".to_string(),
                result_path: args.result_path.clone(),
                artifact_root: args.artifact_root.clone(),
                dogfood_root: args.dogfood_root.clone(),
                operator_snapshot_store: args.operator_snapshot_store.clone(),
                operator_snapshot_id: args.operator_snapshot_id.clone(),
                artifact_refs,
                notes: vec![
                    "Receipt records local operator-run completion; referenced artifacts remain evidence, not owner state.".to_string(),
                    "Codex app-server output is compatibility projection until the run path is native end to end.".to_string(),
                ],
            };
            let written = write_epiphany_cultmesh_operator_run_receipt(&args.store, receipt)?;
            print_json(json!({"status": "written", "store": args.store, "receipt": written}))?;
        }
        "latest" => {
            let intent =
                load_latest_epiphany_cultmesh_operator_run_intent(&args.store, &args.runtime_id)?;
            let receipt =
                load_latest_epiphany_cultmesh_operator_run_receipt(&args.store, &args.runtime_id)?;
            print_json(json!({
                "status": if intent.is_some() || receipt.is_some() { "ready" } else { "missing" },
                "store": args.store,
                "intent": intent,
                "receipt": receipt,
            }))?;
        }
        "coordinator-receipt" => {
            let summary_path = args
                .coordinator_summary
                .clone()
                .context("coordinator-receipt requires --coordinator-summary")?;
            let summary_source = fs::read_to_string(&summary_path)
                .with_context(|| format!("failed to read {}", summary_path.display()))?;
            let summary_json: Value =
                serde_json::from_str(summary_source.trim_start_matches('\u{feff}'))
                    .with_context(|| format!("failed to parse {}", summary_path.display()))?;
            let receipt_id = if args.coordinator_receipt_id.trim().is_empty() {
                format!("coordinator-cultmesh-{}", args.run_id)
            } else {
                args.coordinator_receipt_id.clone()
            };
            let written = write_epiphany_cultmesh_coordinator_run_receipt(
                &args.store,
                epiphany_cultmesh_coordinator_run_receipt_from_summary_json(
                    args.runtime_id.clone(),
                    receipt_id,
                    Utc::now().to_rfc3339(),
                    args.artifact_root.clone(),
                    &summary_json,
                )?,
            )?;
            let hands_action_gate = epiphany_cultmesh_hands_action_gate_from_summary_json(
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
                summary_path.display().to_string(),
                &summary_json,
            )?
            .map(|gate| write_epiphany_cultmesh_hands_action_gate(&args.store, gate))
            .transpose()?;
            let role_review_event = epiphany_cultmesh_role_review_event_from_summary_json(
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
                summary_path.display().to_string(),
                &summary_json,
            )?
            .map(|event| write_epiphany_cultmesh_role_review_event(&args.store, event))
            .transpose()?;
            print_json(json!({
                "status": "written",
                "store": args.store,
                "coordinatorReceipt": written,
                "handsActionGate": hands_action_gate,
                "roleReviewEvent": role_review_event,
            }))?;
        }
        "smoke" => {
            let intent = EpiphanyCultMeshOperatorRunIntentEntry {
                schema_version: EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION.to_string(),
                runtime_id: args.runtime_id.clone(),
                verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
                run_id: args.run_id.clone(),
                requested_at_utc: "2026-05-27T00:00:00Z".to_string(),
                mode: "status".to_string(),
                root: "E:\\Projects\\EpiphanyAgent".to_string(),
                workspace: "E:\\Projects\\EpiphanyAgent".to_string(),
                thread_id: String::new(),
                codex_home: "C:\\Users\\Meta\\.codex".to_string(),
                target_dir: "C:\\Users\\Meta\\.cargo-target-codex".to_string(),
                max_steps: 4,
                timeout_seconds: 240,
                auto_review: false,
                no_ephemeral: false,
                artifact_root: ".epiphany-run/local-smoke".to_string(),
                dogfood_root: ".epiphany-dogfood/local-smoke".to_string(),
            };
            let receipt = EpiphanyCultMeshOperatorRunReceiptEntry {
                schema_version: EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION.to_string(),
                runtime_id: args.runtime_id.clone(),
                verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
                run_id: args.run_id.clone(),
                completed_at_utc: "2026-05-27T00:00:01Z".to_string(),
                mode: "status".to_string(),
                status: "completed".to_string(),
                result_path: ".epiphany-run/local-smoke/status.json".to_string(),
                artifact_root: ".epiphany-run/local-smoke".to_string(),
                dogfood_root: ".epiphany-dogfood/local-smoke".to_string(),
                operator_snapshot_store: ".epiphany-run/cultmesh/operator-snapshots.ccmp"
                    .to_string(),
                operator_snapshot_id: "local-smoke-status".to_string(),
                artifact_refs: vec![".epiphany-run/local-smoke/status.json".to_string()],
                notes: vec!["smoke receipt".to_string()],
            };
            write_epiphany_cultmesh_operator_run_intent(&args.store, intent.clone())?;
            write_epiphany_cultmesh_operator_run_receipt(&args.store, receipt.clone())?;
            let latest_intent =
                load_latest_epiphany_cultmesh_operator_run_intent(&args.store, &args.runtime_id)?;
            let latest_receipt =
                load_latest_epiphany_cultmesh_operator_run_receipt(&args.store, &args.runtime_id)?;
            if latest_intent != Some(intent) || latest_receipt != Some(receipt) {
                anyhow::bail!("operator run intent/receipt did not round-trip through CultMesh");
            }
            print_json(json!({
                "status": "ok",
                "store": args.store,
                "runtimeId": args.runtime_id,
                "runId": args.run_id,
            }))?;
        }
        other => anyhow::bail!("unknown command {other:?}; use intent, receipt, latest, or smoke"),
    }
    Ok(())
}

struct Args {
    command: String,
    store: PathBuf,
    runtime_id: String,
    run_id: String,
    mode: String,
    root: String,
    workspace: String,
    thread_id: String,
    codex_home: String,
    target_dir: String,
    max_steps: u32,
    timeout_seconds: u32,
    auto_review: bool,
    no_ephemeral: bool,
    artifact_root: String,
    dogfood_root: String,
    result_path: String,
    operator_snapshot_store: String,
    operator_snapshot_id: String,
    coordinator_summary: Option<PathBuf>,
    coordinator_receipt_id: String,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut values = env::args().skip(1);
        let command = values.next().unwrap_or_else(|| "latest".to_string());
        let mut args = Args {
            command,
            store: PathBuf::from(".epiphany-run/cultmesh/operator-runs.ccmp"),
            runtime_id: "epiphany-local".to_string(),
            run_id: format!("run-{}", Utc::now().timestamp()),
            mode: "status".to_string(),
            root: String::new(),
            workspace: String::new(),
            thread_id: String::new(),
            codex_home: String::new(),
            target_dir: String::new(),
            max_steps: 4,
            timeout_seconds: 240,
            auto_review: false,
            no_ephemeral: false,
            artifact_root: String::new(),
            dogfood_root: String::new(),
            result_path: String::new(),
            operator_snapshot_store: String::new(),
            operator_snapshot_id: String::new(),
            coordinator_summary: None,
            coordinator_receipt_id: String::new(),
        };

        while let Some(arg) = values.next() {
            match arg.as_str() {
                "--store" => args.store = PathBuf::from(next(&mut values, "--store")?),
                "--runtime-id" => args.runtime_id = next(&mut values, "--runtime-id")?,
                "--run-id" => args.run_id = next(&mut values, "--run-id")?,
                "--mode" => args.mode = next(&mut values, "--mode")?,
                "--root" => args.root = next(&mut values, "--root")?,
                "--workspace" => args.workspace = next(&mut values, "--workspace")?,
                "--thread-id" => args.thread_id = next(&mut values, "--thread-id")?,
                "--codex-home" => args.codex_home = next(&mut values, "--codex-home")?,
                "--target-dir" => args.target_dir = next(&mut values, "--target-dir")?,
                "--max-steps" => args.max_steps = parse_u32(&mut values, "--max-steps")?,
                "--timeout-seconds" => {
                    args.timeout_seconds = parse_u32(&mut values, "--timeout-seconds")?
                }
                "--auto-review" => args.auto_review = parse_bool(&mut values, "--auto-review")?,
                "--no-ephemeral" => args.no_ephemeral = parse_bool(&mut values, "--no-ephemeral")?,
                "--artifact-root" => args.artifact_root = next(&mut values, "--artifact-root")?,
                "--dogfood-root" => args.dogfood_root = next(&mut values, "--dogfood-root")?,
                "--result-path" => args.result_path = next(&mut values, "--result-path")?,
                "--operator-snapshot-store" => {
                    args.operator_snapshot_store = next(&mut values, "--operator-snapshot-store")?
                }
                "--operator-snapshot-id" => {
                    args.operator_snapshot_id = next(&mut values, "--operator-snapshot-id")?
                }
                "--coordinator-summary" => {
                    args.coordinator_summary =
                        Some(PathBuf::from(next(&mut values, "--coordinator-summary")?))
                }
                "--coordinator-receipt-id" => {
                    args.coordinator_receipt_id = next(&mut values, "--coordinator-receipt-id")?
                }
                _ => anyhow::bail!("unknown argument {arg:?}"),
            }
        }

        if let Some(parent) = args.store.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(args)
    }
}

fn next(values: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    values
        .next()
        .with_context(|| format!("missing {name} value"))
}

fn parse_u32(values: &mut impl Iterator<Item = String>, name: &str) -> Result<u32> {
    next(values, name)?
        .parse::<u32>()
        .with_context(|| format!("{name} must be a non-negative integer"))
}

fn parse_bool(values: &mut impl Iterator<Item = String>, name: &str) -> Result<bool> {
    match next(values, name)?.as_str() {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        _ => anyhow::bail!("{name} must be true/false"),
    }
}

fn push_non_empty(items: &mut Vec<String>, value: &str) {
    if !value.trim().is_empty() {
        items.push(value.to_string());
    }
}

fn canonical_file(value: &str, name: &str) -> Result<PathBuf> {
    if value.trim().is_empty() {
        anyhow::bail!("operator-run receipt requires {name}");
    }
    let path =
        fs::canonicalize(value).with_context(|| format!("{name} does not exist: {value}"))?;
    if !path.is_file() {
        anyhow::bail!("{name} is not a file: {}", path.display());
    }
    Ok(path)
}

fn canonical_directory(value: &str, name: &str) -> Result<PathBuf> {
    if value.trim().is_empty() {
        anyhow::bail!("operator-run receipt requires {name}");
    }
    let path =
        fs::canonicalize(value).with_context(|| format!("{name} does not exist: {value}"))?;
    if !path.is_dir() {
        anyhow::bail!("{name} is not a directory: {}", path.display());
    }
    Ok(path)
}

fn print_json(value: Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
