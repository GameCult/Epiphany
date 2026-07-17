use anyhow::{Result, anyhow};
use epiphany_core::{
    default_host_identity_store_path, enroll_default_host_identity,
    export_host_identity_trust_anchor, open_default_host_identity,
};
use serde_json::json;

fn main() -> Result<()> {
    let command = std::env::args().nth(1).ok_or_else(|| {
        anyhow!("usage: epiphany-host-identity <enroll|status|enroll-trust-anchor|export-trust-anchor> [output]")
    })?;
    if matches!(
        command.as_str(),
        "enroll-trust-anchor" | "export-trust-anchor"
    ) {
        let output = std::env::args()
            .nth(2)
            .ok_or_else(|| anyhow!("{command} requires an output path"))?;
        let signer = if command == "enroll-trust-anchor" {
            enroll_default_host_identity()?
        } else {
            open_default_host_identity()?
        };
        let anchor = export_host_identity_trust_anchor(&signer, std::path::Path::new(&output))?;
        println!(
            "{}",
            serde_json::to_string_pretty(&json!({
                "schemaVersion": anchor.schema_version,
                "identityId": anchor.identity_id,
                "output": output,
                "privateStateExposed": false
            }))?
        );
        return Ok(());
    }
    let signer = match command.as_str() {
        "enroll" => enroll_default_host_identity()?,
        "status" => open_default_host_identity()?,
        _ => {
            return Err(anyhow!(
                "usage: epiphany-host-identity <enroll|status|enroll-trust-anchor|export-trust-anchor> [output]"
            ));
        }
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
