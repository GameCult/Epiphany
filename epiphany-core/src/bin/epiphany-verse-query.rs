use anyhow::Context;
use anyhow::Result;
use chrono::Utc;
use epiphany_core::import_gjallar_daemon_affordances;
use epiphany_core::query_epiphany_local_verse_context;
use epiphany_core::seed_epiphany_local_verse_context;
use serde_json::json;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args = Args::parse()?;
    match args.command.as_str() {
        "seed" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                Utc::now().to_rfc3339(),
            )?;
            if let Some(gjallar_store) = args.gjallar_affordance_store.as_ref() {
                import_gjallar_daemon_affordances(
                    &args.store,
                    gjallar_store,
                    args.runtime_id.clone(),
                )?;
            }
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id)?;
            println!("{}", serde_json::to_string_pretty(&context)?);
        }
        "query" => {
            if let Some(gjallar_store) = args.gjallar_affordance_store.as_ref() {
                import_gjallar_daemon_affordances(
                    &args.store,
                    gjallar_store,
                    args.runtime_id.clone(),
                )?;
            }
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id)?;
            println!("{}", serde_json::to_string_pretty(&context)?);
        }
        "smoke" => {
            seed_epiphany_local_verse_context(
                &args.store,
                args.runtime_id.clone(),
                "2026-06-02T00:00:00Z",
            )?;
            if let Some(gjallar_store) = args.gjallar_affordance_store.as_ref() {
                import_gjallar_daemon_affordances(
                    &args.store,
                    gjallar_store,
                    args.runtime_id.clone(),
                )?;
            }
            let context = query_epiphany_local_verse_context(&args.store, args.runtime_id)?;
            if context.verse_policies.len() != 3 {
                anyhow::bail!("local Verse query smoke expected three Verse policies");
            }
            if !context.verse_policies.iter().any(|policy| {
                policy.verse_id == "gamecult-local" && policy.yggdrasil_tunnel_allowed
            }) {
                anyhow::bail!("local Verse query smoke lost Yggdrasil tunnel policy");
            }
            if context.contract_summaries.len() < 6 {
                anyhow::bail!("local Verse query smoke expected organ contract summaries");
            }
            if args.gjallar_affordance_store.is_some() && context.daemon_affordances.is_empty() {
                anyhow::bail!("local Verse query smoke expected imported daemon affordances");
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "status": "ok",
                    "store": args.store,
                    "runtimeId": context.runtime_id,
                    "verses": context.verse_policies.len(),
                    "globalRooms": context.global_room_policies.len(),
                    "contracts": context.contract_summaries.len(),
                    "daemonAffordances": context.daemon_affordances.len(),
                }))?
            );
        }
        other => anyhow::bail!("unknown command {other:?}; use seed, query, or smoke"),
    }
    Ok(())
}

struct Args {
    command: String,
    store: PathBuf,
    runtime_id: String,
    gjallar_affordance_store: Option<PathBuf>,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut values = env::args().skip(1);
        let command = values.next().unwrap_or_else(|| "query".to_string());
        let mut store = PathBuf::from(".epiphany-run/cultmesh/epiphany-local.ccmp");
        let mut runtime_id = "epiphany-local".to_string();
        let mut gjallar_affordance_store = None;

        while let Some(arg) = values.next() {
            match arg.as_str() {
                "--store" => {
                    store = PathBuf::from(values.next().context("missing --store value")?);
                }
                "--runtime-id" => {
                    runtime_id = values.next().context("missing --runtime-id value")?;
                }
                "--gjallar-affordance-store" => {
                    gjallar_affordance_store = Some(PathBuf::from(
                        values
                            .next()
                            .context("missing --gjallar-affordance-store value")?,
                    ));
                }
                _ => anyhow::bail!("unknown argument {arg:?}"),
            }
        }

        if let Some(parent) = store.parent() {
            std::fs::create_dir_all(parent)?;
        }

        Ok(Self {
            command,
            store,
            runtime_id,
            gjallar_affordance_store,
        })
    }
}
