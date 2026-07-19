use anyhow::{Result, anyhow};
use epiphany_core::{
    import_bifrost_persona_feedback_deliveries, validate_bifrost_persona_feedback_source,
    validate_persona_feedback_store_separation,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::env;
use std::path::PathBuf;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct IngressProjection {
    schema_version: &'static str,
    status: &'static str,
    runtime_id: String,
    repository: String,
    persona_id: String,
    applicable_delivery_count: usize,
    present_pressure_count: usize,
    authority: &'static str,
    private_state_exposed: bool,
}

fn import_projection_status(present_pressure_count: usize) -> &'static str {
    if present_pressure_count == 0 {
        "no-applicable-deliveries"
    } else {
        "pressure-present"
    }
}

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
    let (status, applicable_delivery_count, present_pressure_count) = match command.as_str() {
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
            (
                import_projection_status(admitted.len()),
                admitted.len(),
                admitted.len(),
            )
        }
        _ => return Err(anyhow!("unknown command {command:?}; use status or import")),
    };
    println!(
        "{}",
        serde_json::to_string_pretty(&IngressProjection {
            schema_version: "epiphany.persona_feedback.ingress_projection.v1",
            status,
            runtime_id,
            repository,
            persona_id,
            applicable_delivery_count,
            present_pressure_count,
            authority: "resident-pressure-only",
            private_state_exposed: false,
        })?
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_projection_does_not_call_absence_admission() {
        assert_eq!(import_projection_status(0), "no-applicable-deliveries");
        assert_eq!(import_projection_status(1), "pressure-present");
        let encoded = serde_json::to_value(IngressProjection {
            schema_version: "epiphany.persona_feedback.ingress_projection.v1",
            status: import_projection_status(0),
            runtime_id: "epiphany-yggdrasil".into(),
            repository: "GameCult/Alien".into(),
            persona_id: "epiphany".into(),
            applicable_delivery_count: 0,
            present_pressure_count: 0,
            authority: "resident-pressure-only",
            private_state_exposed: false,
        })
        .expect("projection serializes");
        assert_eq!(encoded["status"], "no-applicable-deliveries");
        assert!(encoded.get("admittedCount").is_none());
    }
}
