use std::{env, path::PathBuf};

use anyhow::{Result, anyhow};
use epiphany_core::{
    enroll_persona_discord_request_identity, export_persona_discord_request_anchor_candidate,
};

fn main() -> Result<()> {
    let mut mouth_identity_store = None;
    let mut mouth_request_anchor = None;
    let mut runtime_id = None;
    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| anyhow!("{arg} requires a value"))?;
        match arg.as_str() {
            "--mouth-identity-store" => mouth_identity_store = Some(PathBuf::from(value)),
            "--mouth-request-anchor" => mouth_request_anchor = Some(PathBuf::from(value)),
            "--runtime-id" => runtime_id = Some(value),
            _ => return Err(anyhow!("unknown argument {arg}")),
        }
    }
    let identity =
        mouth_identity_store.ok_or_else(|| anyhow!("--mouth-identity-store is required"))?;
    let anchor =
        mouth_request_anchor.ok_or_else(|| anyhow!("--mouth-request-anchor is required"))?;
    let runtime = runtime_id.ok_or_else(|| anyhow!("--runtime-id is required"))?;
    enroll_persona_discord_request_identity(&identity)?;
    export_persona_discord_request_anchor_candidate(&identity, &anchor, &runtime)?;
    println!(
        "{}",
        serde_json::json!({
            "schemaVersion": "epiphany.persona_mouth_identity_enrollment.v0",
            "status": "ready",
            "runtimeId": runtime,
            "privateStateExposed": false
        })
    );
    Ok(())
}
