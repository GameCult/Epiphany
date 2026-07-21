use std::{env, path::PathBuf};

use anyhow::{Result, anyhow};
use epiphany_core::{
    enroll_persona_discord_request_identity, export_persona_discord_request_anchor_candidate,
    insert_persona_discord_delivery_request, open_persona_discord_request_identity,
    sign_persona_discord_delivery_request,
};

fn main() -> Result<()> {
    let mut values = std::collections::BTreeMap::new();
    let mut args = env::args().skip(1);
    while let Some(key) = args.next() {
        values.insert(
            key.clone(),
            args.next()
                .ok_or_else(|| anyhow!("{key} requires a value"))?,
        );
    }
    let value = |key: &str| {
        values
            .get(key)
            .cloned()
            .ok_or_else(|| anyhow!("{key} is required"))
    };
    let store = PathBuf::from(value("--request-store")?);
    let identity = PathBuf::from(value("--identity-store")?);
    let anchor = PathBuf::from(value("--request-anchor")?);
    enroll_persona_discord_request_identity(&identity)?;
    export_persona_discord_request_anchor_candidate(&identity, &anchor, "epiphany-yggdrasil")?;
    let signer = open_persona_discord_request_identity(&identity)?;
    let issued = chrono::Utc::now();
    let request = sign_persona_discord_delivery_request(
        &signer,
        "rust-crossing-smoke-1".into(),
        "effect-rust-crossing-smoke-1".into(),
        "epiphany-yggdrasil".into(),
        "epiphany.Persona".into(),
        "123".into(),
        "".into(),
        "Rust-authored crossing smoke".into(),
        issued.to_rfc3339(),
        (issued + chrono::Duration::seconds(120)).to_rfc3339(),
    )?;
    insert_persona_discord_delivery_request(&store, &request)?;
    println!("{}", request.request_id);
    Ok(())
}
