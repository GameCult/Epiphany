use anyhow::{Result, anyhow};
use epiphany_core::{
    default_host_identity_store_path, enroll_default_host_identity, enroll_host_identity_at,
    export_host_identity_trust_anchor, export_raw_host_identity_trust_anchor,
    open_default_host_identity, open_host_identity_at,
};
use serde_json::json;

fn main() -> Result<()> {
    let command = std::env::args().nth(1).ok_or_else(|| {
        anyhow!("usage: epiphany-host-identity <enroll|status|enroll-trust-anchor|export-trust-anchor|enroll-raw-trust-anchor|export-raw-trust-anchor> [output]")
    })?;
    let values = std::env::args().skip(2).collect::<Vec<_>>();
    let store_index = values.iter().position(|value| value == "--store");
    let explicit_store = match store_index {
        Some(index) if index + 2 == values.len() => {
            Some(std::path::PathBuf::from(&values[index + 1]))
        }
        Some(_) => return Err(anyhow!("--store must be the final option with one path")),
        None => None,
    };
    let positional = &values[..store_index.unwrap_or(values.len())];
    if matches!(
        command.as_str(),
        "enroll-trust-anchor"
            | "export-trust-anchor"
            | "enroll-raw-trust-anchor"
            | "export-raw-trust-anchor"
    ) {
        let output = positional
            .first()
            .filter(|_| positional.len() == 1)
            .ok_or_else(|| anyhow!("{command} requires one output path"))?;
        let signer = if matches!(
            command.as_str(),
            "enroll-trust-anchor" | "enroll-raw-trust-anchor"
        ) {
            match explicit_store.as_deref() {
                Some(store) => enroll_host_identity_at(store)?,
                None => enroll_default_host_identity()?,
            }
        } else {
            match explicit_store.as_deref() {
                Some(store) => open_host_identity_at(store)?,
                None => open_default_host_identity()?,
            }
        };
        let anchor = if command.ends_with("raw-trust-anchor") {
            export_raw_host_identity_trust_anchor(&signer, std::path::Path::new(output))?
        } else {
            export_host_identity_trust_anchor(&signer, std::path::Path::new(output))?
        };
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
    if !positional.is_empty() {
        return Err(anyhow!("unexpected host identity positional argument"));
    }
    let signer = match command.as_str() {
        "enroll" => match explicit_store.as_deref() {
            Some(store) => enroll_host_identity_at(store)?,
            None => enroll_default_host_identity()?,
        },
        "status" => match explicit_store.as_deref() {
            Some(store) => open_host_identity_at(store)?,
            None => open_default_host_identity()?,
        },
        _ => {
            return Err(anyhow!(
                "usage: epiphany-host-identity <enroll|status|enroll-trust-anchor|export-trust-anchor|enroll-raw-trust-anchor|export-raw-trust-anchor> [output]"
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
            "store": explicit_store.unwrap_or(default_host_identity_store_path()?),
            "privateStateExposed": false
        }))?
    );
    Ok(())
}
