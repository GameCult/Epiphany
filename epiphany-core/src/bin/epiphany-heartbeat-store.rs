use anyhow::Result;
use anyhow::anyhow;
use epiphany_core::load_heartbeat_state_entry;
use epiphany_core::migrate_heartbeat_json_to_cultcache;
use epiphany_core::write_heartbeat_json_projection;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return usage();
    };
    let mut json_path: Option<PathBuf> = None;
    let mut store_path: Option<PathBuf> = None;
    let mut projection_path: Option<PathBuf> = None;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json_path = Some(next_path(&mut args, "--json")?),
            "--store" => store_path = Some(next_path(&mut args, "--store")?),
            "--projection" => projection_path = Some(next_path(&mut args, "--projection")?),
            _ => return Err(anyhow!("unknown argument {arg:?}")),
        }
    }

    match command.as_str() {
        "migrate-json" => {
            let json_path = json_path.ok_or_else(|| anyhow!("migrate-json requires --json"))?;
            let store_path = store_path.ok_or_else(|| anyhow!("migrate-json requires --store"))?;
            let state = migrate_heartbeat_json_to_cultcache(&json_path, &store_path)?;
            if let Some(projection_path) = projection_path {
                write_heartbeat_json_projection(&store_path, projection_path)?;
            }
            println!(
                "{}",
                serde_json::json!({
                    "ok": true,
                    "command": "migrate-json",
                    "json": json_path,
                    "store": store_path,
                    "schemaVersion": state.schema_version,
                    "participants": state.participants.len(),
                    "history": state.history.len(),
                })
            );
        }
        "project" => {
            let store_path = store_path.ok_or_else(|| anyhow!("project requires --store"))?;
            let projection_path =
                projection_path.ok_or_else(|| anyhow!("project requires --projection"))?;
            let state = write_heartbeat_json_projection(&store_path, &projection_path)?;
            println!(
                "{}",
                serde_json::json!({
                    "ok": true,
                    "command": "project",
                    "store": store_path,
                    "projection": projection_path,
                    "schemaVersion": state.schema_version,
                    "participants": state.participants.len(),
                    "history": state.history.len(),
                })
            );
        }
        "status" => {
            let store_path = store_path.ok_or_else(|| anyhow!("status requires --store"))?;
            let state = load_heartbeat_state_entry(&store_path)?;
            println!(
                "{}",
                serde_json::json!({
                    "ok": true,
                    "command": "status",
                    "store": store_path,
                    "present": state.is_some(),
                    "schemaVersion": state.as_ref().map(|value| value.schema_version.as_str()),
                    "participants": state.as_ref().map(|value| value.participants.len()),
                    "history": state.as_ref().map(|value| value.history.len()),
                })
            );
        }
        _ => return usage(),
    }

    Ok(())
}

fn next_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    args.next()
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("{name} requires a value"))
}

fn usage() -> Result<()> {
    Err(anyhow!(
        "usage: epiphany-heartbeat-store migrate-json --json <path> --store <path> [--projection <path>]\n       epiphany-heartbeat-store project --store <path> --projection <path>\n       epiphany-heartbeat-store status --store <path>"
    ))
}
