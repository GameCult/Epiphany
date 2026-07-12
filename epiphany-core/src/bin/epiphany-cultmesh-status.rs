use anyhow::Context;
use anyhow::Result;
use chrono::Utc;
use epiphany_core::default_epiphany_cultmesh_operator_status;
use epiphany_core::load_epiphany_cultmesh_operator_status;
use epiphany_core::write_epiphany_cultmesh_operator_status;
use serde_json::json;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args = Args::parse()?;
    match args.command.as_str() {
        "write" => {
            let status = default_epiphany_cultmesh_operator_status(
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            );
            let written = write_epiphany_cultmesh_operator_status(&args.store, status)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "written",
                    "store": args.store,
                    "document": written,
                }))?
            );
        }
        "read" => {
            let loaded = load_epiphany_cultmesh_operator_status(&args.store, &args.runtime_id)?;
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": if loaded.is_some() { "ready" } else { "missing" },
                    "store": args.store,
                    "document": loaded,
                }))?
            );
        }
        "smoke" => {
            let status = default_epiphany_cultmesh_operator_status(
                args.runtime_id.clone(),
                "2026-05-27T00:00:00Z",
            );
            write_epiphany_cultmesh_operator_status(&args.store, status.clone())?;
            let loaded = load_epiphany_cultmesh_operator_status(&args.store, &args.runtime_id)?;
            if loaded != Some(status) {
                anyhow::bail!("operator status did not round-trip through CultMesh");
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": args.runtime_id,
                }))?
            );
        }
        other => anyhow::bail!("unknown command {other:?}; use write, read, or smoke"),
    }
    Ok(())
}

struct Args {
    command: String,
    store: PathBuf,
    runtime_id: String,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut values = env::args().skip(1);
        let command = values.next().unwrap_or_else(|| "read".to_string());
        let mut store = PathBuf::from(".epiphany-run/cultmesh/operator-status.ccmp");
        let mut store_explicit = false;
        let mut runtime_id = "epiphany-local".to_string();

        while let Some(arg) = values.next() {
            match arg.as_str() {
                "--store" => {
                    store = PathBuf::from(values.next().context("missing --store value")?);
                    store_explicit = true;
                }
                "--runtime-id" => {
                    runtime_id = values.next().context("missing --runtime-id value")?;
                }
                _ => anyhow::bail!("unknown argument {arg:?}"),
            }
        }

        if command == "smoke" {
            if store_explicit {
                anyhow::bail!(
                    "operator-status smoke accepts no store override and writes only beneath .epiphany-smoke"
                );
            }
            store = PathBuf::from(".epiphany-smoke/cultmesh/operator-status.ccmp");
        }

        Ok(Self {
            command,
            store,
            runtime_id,
        })
    }
}
