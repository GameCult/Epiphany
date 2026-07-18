use anyhow::{Result, anyhow};
use epiphany_core::{
    import_bifrost_persona_feedback_deliveries, validate_bifrost_persona_feedback_source,
    validate_persona_feedback_store_separation,
};
use serde_json::json;
use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let mut args = env::args().skip(1);
    let command = args.next().ok_or_else(|| anyhow!("use status or import"))?;
    let mut values = BTreeMap::new();
    while let Some(flag) = args.next() {
        let value = args
            .next()
            .ok_or_else(|| anyhow!("missing value for {flag}"))?;
        values.insert(flag, value);
    }
    let required = |name: &str| -> Result<String> {
        values
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow!("missing {name}"))
    };
    let source_store = PathBuf::from(required("--source-store")?);
    let trust_anchor = PathBuf::from(required("--trust-anchor")?);
    let runtime_id = required("--runtime-id")?;
    let repository = required("--repository")?;
    let persona_id = required("--persona-id")?;
    let (status, delivery_count, admitted_count) = match command.as_str() {
        "status" => (
            "ready",
            validate_bifrost_persona_feedback_source(
                &source_store,
                &trust_anchor,
                &runtime_id,
                &repository,
                &persona_id,
            )?,
            0,
        ),
        "import" => {
            let feedback_store = PathBuf::from(required("--persona-feedback-store")?);
            let resident_store = PathBuf::from(required("--resident-store")?);
            let runtime_store = PathBuf::from(required("--runtime-store")?);
            let mind_store = PathBuf::from(required("--mind-store")?);
            let release_store = PathBuf::from(required("--release-store")?);
            validate_persona_feedback_store_separation(
                &source_store,
                &feedback_store,
                &[
                    &trust_anchor,
                    &resident_store,
                    &runtime_store,
                    &mind_store,
                    &release_store,
                ],
            )?;
            let admitted = import_bifrost_persona_feedback_deliveries(
                &source_store,
                &feedback_store,
                &trust_anchor,
                &runtime_id,
                &repository,
                &persona_id,
            )?;
            ("admitted", admitted.len(), admitted.len())
        }
        _ => return Err(anyhow!("unknown command {command:?}; use status or import")),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schemaVersion": "epiphany.persona_feedback.ingress_projection.v0",
            "status": status,
            "runtimeId": runtime_id,
            "repository": repository,
            "personaId": persona_id,
            "deliveryCount": delivery_count,
            "admittedCount": admitted_count,
            "authority": "resident-pressure-only",
            "privateStateExposed": false,
        }))?
    );
    Ok(())
}
