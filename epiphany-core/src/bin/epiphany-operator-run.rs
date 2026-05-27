use anyhow::Context;
use anyhow::Result;
use chrono::Utc;
use epiphany_core::EPIPHANY_CULTMESH_INTERNAL_VERSE_ID;
use epiphany_core::EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION;
use epiphany_core::EpiphanyCultMeshOperatorRunIntentEntry;
use epiphany_core::EpiphanyCultMeshOperatorRunReceiptEntry;
use epiphany_core::load_latest_epiphany_cultmesh_operator_run_intent;
use epiphany_core::load_latest_epiphany_cultmesh_operator_run_receipt;
use epiphany_core::write_epiphany_cultmesh_operator_run_intent;
use epiphany_core::write_epiphany_cultmesh_operator_run_receipt;
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
                status: args.status.clone(),
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
    status: String,
    operator_snapshot_store: String,
    operator_snapshot_id: String,
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
            status: "completed".to_string(),
            operator_snapshot_store: String::new(),
            operator_snapshot_id: String::new(),
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
                "--status" => args.status = next(&mut values, "--status")?,
                "--operator-snapshot-store" => {
                    args.operator_snapshot_store = next(&mut values, "--operator-snapshot-store")?
                }
                "--operator-snapshot-id" => {
                    args.operator_snapshot_id = next(&mut values, "--operator-snapshot-id")?
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

fn print_json(value: Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
