use std::{env, net::SocketAddr, path::PathBuf, time::Duration};

use anyhow::{Result, anyhow};
use epiphany_core::{
    enroll_persona_discord_request_identity, exchange_persona_discord_delivery_rudp,
    export_persona_discord_request_anchor_candidate, insert_persona_discord_delivery_request,
    load_persona_discord_receipt_anchor, open_persona_discord_request_identity,
    sign_persona_discord_delivery_request,
};

fn main() -> Result<()> {
    let values = args()?;
    let request_store = path(&values, "--request-store")?;
    let receipt_store = path(&values, "--receipt-store")?;
    let identity_store = path(&values, "--identity-store")?;
    let request_anchor = path(&values, "--request-anchor")?;
    let receipt_anchor = path(&values, "--receipt-anchor")?;
    let endpoint: SocketAddr = value(&values, "--endpoint")?.parse()?;
    enroll_persona_discord_request_identity(&identity_store)?;
    export_persona_discord_request_anchor_candidate(
        &identity_store,
        &request_anchor,
        "epiphany-starfire",
    )?;
    let signer = open_persona_discord_request_identity(&identity_store)?;
    let issued = chrono::Utc::now();
    let request = sign_persona_discord_delivery_request(
        &signer,
        "persona-discord:rudp-cross-language:0".into(),
        "effect-rudp-cross-language".into(),
        "epiphany-starfire".into(),
        "epiphany.Persona".into(),
        "123".into(),
        "456".into(),
        "Cross-language Persona nerve smoke.".into(),
        issued.to_rfc3339(),
        (issued + chrono::Duration::seconds(90)).to_rfc3339(),
    )?;
    insert_persona_discord_delivery_request(&request_store, &request)?;
    let anchor = load_persona_discord_receipt_anchor(&receipt_anchor)?;
    let receipt = exchange_persona_discord_delivery_rudp(
        endpoint,
        "epiphany-starfire",
        &request_store,
        &receipt_store,
        &request,
        &anchor,
        Duration::from_secs(5),
    )?;
    println!(
        "{}",
        serde_json::to_string(&serde_json::json!({
            "requestId": receipt.request_id,
            "status": receipt.status,
            "messageId": receipt.message_id,
            "providerIdentityId": receipt.provider_identity_id,
        }))?
    );
    Ok(())
}

fn args() -> Result<std::collections::BTreeMap<String, String>> {
    let mut values = std::collections::BTreeMap::new();
    let mut args = env::args().skip(1);
    while let Some(key) = args.next() {
        values.insert(
            key,
            args.next()
                .ok_or_else(|| anyhow!("argument requires value"))?,
        );
    }
    Ok(values)
}
fn value(values: &std::collections::BTreeMap<String, String>, key: &str) -> Result<String> {
    values
        .get(key)
        .cloned()
        .ok_or_else(|| anyhow!("{key} is required"))
}
fn path(values: &std::collections::BTreeMap<String, String>, key: &str) -> Result<PathBuf> {
    Ok(value(values, key)?.into())
}
