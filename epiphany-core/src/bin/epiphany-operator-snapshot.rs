use anyhow::Context;
use anyhow::Result;
use chrono::Utc;
use epiphany_core::epiphany_cultmesh_operator_snapshot_from_status_json;
use epiphany_core::load_latest_epiphany_cultmesh_operator_snapshot;
use epiphany_core::write_epiphany_cultmesh_operator_snapshot;
use serde_json::Value;
use serde_json::json;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args = Args::parse()?;
    match args.command.as_str() {
        "from-status" => {
            let input = args.input.context("from-status requires --input")?;
            let source = fs::read_to_string(&input)
                .with_context(|| format!("failed to read {}", input.display()))?;
            let status_json: Value = serde_json::from_str(&source)
                .with_context(|| format!("failed to parse {}", input.display()))?;
            let snapshot = epiphany_cultmesh_operator_snapshot_from_status_json(
                args.runtime_id.clone(),
                args.snapshot_id.clone(),
                Utc::now().to_rfc3339(),
                args.source_mode.clone(),
                input.to_string_lossy(),
                &status_json,
            )?;
            let written = write_epiphany_cultmesh_operator_snapshot(&args.store, snapshot)?;
            print_json(json!({
                "status": "written",
                "store": args.store,
                "snapshot": written,
            }))?;
        }
        "latest" => {
            let latest =
                load_latest_epiphany_cultmesh_operator_snapshot(&args.store, &args.runtime_id)?;
            print_json(json!({
                "status": if latest.is_some() { "ready" } else { "missing" },
                "store": args.store,
                "snapshot": latest,
            }))?;
        }
        "smoke" => {
            let status_json = json!({
                "threadId": "thread-smoke",
                "scene": {
                    "scene": {
                        "stateStatus": "missing",
                        "availableActions": ["crrc", "roles"]
                    }
                },
                "pressure": {
                    "pressure": {
                        "level": "low"
                    }
                },
                "reorient": {
                    "decision": {
                        "action": "regather",
                        "nextAction": "Regather source context."
                    }
                },
                "crrc": {
                    "recommendation": {
                        "action": "regatherManually"
                    }
                },
                "coordinator": {
                    "action": "wait"
                },
                "rawResult": {
                    "sealed": true
                }
            });
            let snapshot = epiphany_cultmesh_operator_snapshot_from_status_json(
                args.runtime_id.clone(),
                args.snapshot_id.clone(),
                "2026-05-27T00:00:00Z",
                "status",
                ".epiphany-smoke/operator-status.json",
                &status_json,
            )?;
            write_epiphany_cultmesh_operator_snapshot(&args.store, snapshot.clone())?;
            let latest =
                load_latest_epiphany_cultmesh_operator_snapshot(&args.store, &args.runtime_id)?;
            if latest != Some(snapshot) {
                anyhow::bail!("operator snapshot did not round-trip through CultMesh");
            }
            print_json(json!({
                "status": "ok",
                "store": args.store,
                "runtimeId": args.runtime_id,
                "snapshotId": args.snapshot_id,
            }))?;
        }
        other => anyhow::bail!("unknown command {other:?}; use from-status, latest, or smoke"),
    }
    Ok(())
}

struct Args {
    command: String,
    store: PathBuf,
    runtime_id: String,
    snapshot_id: String,
    source_mode: String,
    input: Option<PathBuf>,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut values = env::args().skip(1);
        let command = values.next().unwrap_or_else(|| "latest".to_string());
        let mut store = PathBuf::from(".epiphany-run/cultmesh/operator-snapshots.ccmp");
        let mut runtime_id = "epiphany-local".to_string();
        let mut snapshot_id = format!("snapshot-{}", Utc::now().timestamp());
        let mut source_mode = "status".to_string();
        let mut input = None;

        while let Some(arg) = values.next() {
            match arg.as_str() {
                "--store" => {
                    store = PathBuf::from(values.next().context("missing --store value")?);
                }
                "--runtime-id" => {
                    runtime_id = values.next().context("missing --runtime-id value")?;
                }
                "--snapshot-id" => {
                    snapshot_id = values.next().context("missing --snapshot-id value")?;
                }
                "--source-mode" => {
                    source_mode = values.next().context("missing --source-mode value")?;
                }
                "--input" => {
                    input = Some(PathBuf::from(
                        values.next().context("missing --input value")?,
                    ));
                }
                _ => anyhow::bail!("unknown argument {arg:?}"),
            }
        }

        if let Some(parent) = store.parent() {
            fs::create_dir_all(parent)?;
        }

        Ok(Self {
            command,
            store,
            runtime_id,
            snapshot_id,
            source_mode,
            input,
        })
    }
}

fn print_json(value: Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(&value)?);
    Ok(())
}
