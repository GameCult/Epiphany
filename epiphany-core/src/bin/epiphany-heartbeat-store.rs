use anyhow::Result;
use anyhow::anyhow;
use epiphany_core::HeartbeatCompleteOptions;
use epiphany_core::HeartbeatTickOptions;
use epiphany_core::complete_heartbeat_store;
use epiphany_core::heartbeat_status_projection;
use epiphany_core::initialize_heartbeat_store;
use epiphany_core::load_heartbeat_state_entry;
use epiphany_core::migrate_heartbeat_json_to_cultcache;
use epiphany_core::tick_heartbeat_store;
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
    let mut artifact_dir: Option<PathBuf> = None;
    let mut target_heartbeat_rate = 1.0_f64;
    let mut coordinator_action: Option<String> = None;
    let mut target_role: Option<String> = None;
    let mut urgency = 0.75_f64;
    let mut schedule_id = "epiphany-heartbeat".to_string();
    let mut source_scene_ref = "epiphany/coordinator".to_string();
    let mut defer_completion = false;
    let mut role: Option<String> = None;
    let mut action_id: Option<String> = None;
    let mut limit = 8_usize;

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--json" => json_path = Some(next_path(&mut args, "--json")?),
            "--store" => store_path = Some(next_path(&mut args, "--store")?),
            "--projection" => projection_path = Some(next_path(&mut args, "--projection")?),
            "--artifact-dir" => artifact_dir = Some(next_path(&mut args, "--artifact-dir")?),
            "--target-heartbeat-rate" => {
                target_heartbeat_rate = next_value(&mut args, "--target-heartbeat-rate")?.parse()?
            }
            "--coordinator-action" => {
                coordinator_action = Some(next_value(&mut args, "--coordinator-action")?)
            }
            "--target-role" => target_role = Some(next_value(&mut args, "--target-role")?),
            "--urgency" => urgency = next_value(&mut args, "--urgency")?.parse()?,
            "--schedule-id" => schedule_id = next_value(&mut args, "--schedule-id")?,
            "--source-scene-ref" => source_scene_ref = next_value(&mut args, "--source-scene-ref")?,
            "--defer-completion" => defer_completion = true,
            "--role" => role = Some(next_value(&mut args, "--role")?),
            "--action-id" => action_id = Some(next_value(&mut args, "--action-id")?),
            "--limit" => limit = next_value(&mut args, "--limit")?.parse()?,
            _ => return Err(anyhow!("unknown argument {arg:?}")),
        }
    }

    match command.as_str() {
        "init" => {
            let store_path = store_path.ok_or_else(|| anyhow!("init requires --store"))?;
            let state = initialize_heartbeat_store(&store_path, target_heartbeat_rate)?;
            println!(
                "{}",
                serde_json::json!({
                    "ok": true,
                    "command": "init",
                    "storeFile": store_path,
                    "schemaVersion": state.schema_version,
                    "participants": state.participants.len(),
                    "history": state.history.len(),
                })
            );
        }
        "tick" => {
            let store_path = store_path.ok_or_else(|| anyhow!("tick requires --store"))?;
            let artifact_dir =
                artifact_dir.ok_or_else(|| anyhow!("tick requires --artifact-dir"))?;
            let result = tick_heartbeat_store(
                &store_path,
                &artifact_dir,
                HeartbeatTickOptions {
                    target_heartbeat_rate,
                    coordinator_action,
                    target_role,
                    urgency,
                    schedule_id,
                    source_scene_ref,
                    defer_completion,
                },
            )?;
            println!("{}", result);
        }
        "complete" => {
            let store_path = store_path.ok_or_else(|| anyhow!("complete requires --store"))?;
            let artifact_dir =
                artifact_dir.ok_or_else(|| anyhow!("complete requires --artifact-dir"))?;
            let role = role.ok_or_else(|| anyhow!("complete requires --role"))?;
            let result = complete_heartbeat_store(
                &store_path,
                &artifact_dir,
                HeartbeatCompleteOptions { role, action_id },
            )?;
            println!("{}", result);
        }
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
            if let Some(artifact_dir) = artifact_dir {
                println!(
                    "{}",
                    heartbeat_status_projection(
                        &store_path,
                        artifact_dir,
                        target_heartbeat_rate,
                        limit
                    )?
                );
            } else {
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
        }
        _ => return usage(),
    }

    Ok(())
}

fn next_path(args: &mut impl Iterator<Item = String>, name: &str) -> Result<PathBuf> {
    Ok(PathBuf::from(next_value(args, name)?))
}

fn next_value(args: &mut impl Iterator<Item = String>, name: &str) -> Result<String> {
    args.next()
        .ok_or_else(|| anyhow!("{name} requires a value"))
}

fn usage() -> Result<()> {
    Err(anyhow!(
        "usage: epiphany-heartbeat-store init --store <path>\n       epiphany-heartbeat-store tick --store <path> --artifact-dir <path> [--coordinator-action <action>] [--defer-completion]\n       epiphany-heartbeat-store complete --store <path> --artifact-dir <path> --role <role> [--action-id <id>]\n       epiphany-heartbeat-store status --store <path> [--artifact-dir <path>]\n       epiphany-heartbeat-store migrate-json --json <path> --store <path> [--projection <path>]\n       epiphany-heartbeat-store project --store <path> --projection <path>"
    ))
}
