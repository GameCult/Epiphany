use anyhow::{Result, anyhow};
use epiphany_core::{
    default_host_identity_store_path, enroll_default_host_identity, open_default_host_identity,
};
use serde_json::json;

fn main() -> Result<()> {
    let command = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow!("usage: epiphany-host-identity <enroll|status>"))?;
    let signer = match command.as_str() {
        "enroll" => enroll_default_host_identity()?,
        "status" => open_default_host_identity()?,
        _ => return Err(anyhow!("usage: epiphany-host-identity <enroll|status>")),
    };
    let entry = signer.entry();
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schemaVersion": entry.schema_version,
            "identityId": entry.identity_id,
            "assurance": entry.assurance,
            "protectorKind": entry.protector_kind,
            "createdAt": entry.created_at,
            "store": default_host_identity_store_path()?,
            "privateStateExposed": false
        }))?
    );
    Ok(())
}
