use anyhow::{Context, Result};
use epiphany_core::{
    epiphany_cultmesh_eve_connection_receipt_for_intent,
    load_latest_epiphany_cultmesh_eve_connection_intent,
    write_epiphany_cultmesh_eve_connection_receipt,
};
use serde_json::json;
use std::{env, path::PathBuf};

fn main() -> Result<()> {
    let args = Args::parse()?;
    let intent =
        load_latest_epiphany_cultmesh_eve_connection_intent(&args.store, args.runtime_id.clone())?
            .context("Eve provider found no pending connection intent")?;
    if intent.target_cluster_id != args.provider_cluster_id {
        anyhow::bail!(
            "Eve provider {} cannot answer intent owned by {}",
            args.provider_cluster_id,
            intent.target_cluster_id
        );
    }
    if let Some(expected_intent_id) = args.intent_id.as_deref() {
        if intent.intent_id != expected_intent_id {
            anyhow::bail!(
                "latest Eve connection intent {:?} does not match requested {:?}",
                intent.intent_id,
                expected_intent_id
            );
        }
    }
    let receipt =
        epiphany_cultmesh_eve_connection_receipt_for_intent(args.receipt_id, &intent, args.status);
    let written = write_epiphany_cultmesh_eve_connection_receipt(
        &args.store,
        args.runtime_id.clone(),
        receipt,
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schemaVersion": "epiphany.eve_provider_connection_receipt.v0",
            "status": written.status,
            "runtimeId": args.runtime_id,
            "providerClusterId": args.provider_cluster_id,
            "intentId": written.intent_id,
            "receiptId": written.receipt_id,
            "feedbackRoute": written.feedback_route,
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

struct Args {
    store: PathBuf,
    runtime_id: String,
    provider_cluster_id: String,
    intent_id: Option<String>,
    receipt_id: String,
    status: String,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut values = env::args().skip(1);
        let command = values.next().unwrap_or_else(|| "accept".to_string());
        if command != "accept" {
            anyhow::bail!("unknown command {command:?}; use accept");
        }
        let mut store = PathBuf::from(".epiphany-run/cultmesh/local-verse.ccmp");
        let mut runtime_id = "epiphany-local".to_string();
        let mut provider_cluster_id = None;
        let mut intent_id = None;
        let mut receipt_id = None;
        let mut status = "accepted".to_string();
        while let Some(arg) = values.next() {
            match arg.as_str() {
                "--store" => store = PathBuf::from(values.next().context("missing --store")?),
                "--runtime-id" => runtime_id = values.next().context("missing --runtime-id")?,
                "--provider-cluster-id" => {
                    provider_cluster_id =
                        Some(values.next().context("missing --provider-cluster-id")?)
                }
                "--intent-id" => intent_id = Some(values.next().context("missing --intent-id")?),
                "--receipt-id" => receipt_id = Some(values.next().context("missing --receipt-id")?),
                "--status" => status = values.next().context("missing --status")?,
                _ => anyhow::bail!("unknown argument {arg:?}"),
            }
        }
        Ok(Self {
            store,
            runtime_id,
            provider_cluster_id: provider_cluster_id.context("missing --provider-cluster-id")?,
            intent_id,
            receipt_id: receipt_id.context("missing --receipt-id")?,
            status,
        })
    }
}
