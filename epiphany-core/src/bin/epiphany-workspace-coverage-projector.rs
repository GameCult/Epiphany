use anyhow::{Context, Result, anyhow};
use epiphany_core::{
    EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_SCHEMA_VERSION,
    EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID, EpiphanyCultMeshDaemonHeartbeatEventEntry,
    EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry, WorkspaceCoverageProjectorConfig,
    WorkspaceCoverageProjectorPulseStatus, WorkspaceCoverageProjectorServiceBody,
    authenticate_epiphany_cultmesh_workspace_coverage_projector_launch,
    load_epiphany_cultmesh_daemon_service_lifecycle_receipt,
    write_epiphany_cultmesh_daemon_heartbeat_event,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;

fn main() -> Result<()> {
    let args = Args::parse()?;
    let receipt = authenticate_managed_launch(&args)?;
    authenticate_process_body(&receipt)?;

    let provider_incarnation = Uuid::new_v4().to_string();
    let mut config = WorkspaceCoverageProjectorConfig::from_env();
    config.qdrant_url = args.qdrant_url.clone();
    config.ollama_base_url = args.ollama_base_url.clone();
    config.ollama_model = args.ollama_model.clone();
    let mut projector = WorkspaceCoverageProjectorServiceBody::new(
        &args.runtime_store,
        &args.runtime_id,
        config,
        &provider_incarnation,
        &args.startup_lifecycle_receipt_id,
    )?;

    let mut sequence = 0_u64;
    loop {
        sequence = sequence
            .checked_add(1)
            .ok_or_else(|| anyhow!("workspace coverage heartbeat sequence exhausted"))?;
        let pulse = projector.pulse();
        // Contention describes canonical claim ownership, not provider health.
        // Publishing it as degraded would make a live successor unable to
        // establish the ready heartbeat required to fence an abandoned owner.
        let degraded = matches!(pulse.status, WorkspaceCoverageProjectorPulseStatus::Refused);
        let heartbeat = write_epiphany_cultmesh_daemon_heartbeat_event(
            &args.local_verse_store,
            args.runtime_id.clone(),
            EpiphanyCultMeshDaemonHeartbeatEventEntry {
                schema_version: EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_SCHEMA_VERSION.to_string(),
                heartbeat_id: Uuid::new_v4().to_string(),
                daemon_id: EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID.to_string(),
                cluster_id: "local".to_string(),
                provider_incarnation: projector.provider_incarnation().to_string(),
                sequence,
                status: if degraded { "degraded" } else { "ready" }.to_string(),
                heartbeat_at: chrono::Utc::now().to_rfc3339(),
                private_state_exposed: false,
                startup_lifecycle_receipt_id: projector.startup_lifecycle_receipt_id().to_string(),
            },
        )?;
        println!(
            "{}",
            serde_json::to_string(&json!({
                "schemaVersion": "epiphany.workspace_coverage_projector_pulse.v0",
                "pulseSequence": sequence,
                "providerIncarnation": projector.provider_incarnation(),
                "heartbeatId": heartbeat.heartbeat_id,
                "providerStatus": heartbeat.status,
                "pulseStatus": pulse_status(pulse.status),
                "bodyObservationId": pulse.body_observation_id,
                "bodyGeneration": pulse.body_generation,
                "planId": pulse.plan_id,
                "receiptId": pulse.receipt_id,
                "fault": pulse.fault,
                "privateStateExposed": false,
                "authoritative": false
            }))?
        );
        thread::sleep(Duration::from_secs(args.interval_seconds));
    }
}

fn authenticate_managed_launch(
    args: &Args,
) -> Result<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if load_epiphany_cultmesh_daemon_service_lifecycle_receipt(
            &args.local_verse_store,
            args.runtime_id.clone(),
            &args.startup_lifecycle_receipt_id,
        )?
        .is_some()
        {
            return authenticate_epiphany_cultmesh_workspace_coverage_projector_launch(
                &args.local_verse_store,
                args.runtime_id.clone(),
                &args.startup_lifecycle_receipt_id,
            );
        }
        if Instant::now() >= deadline {
            return Err(anyhow!(
                "managed workspace coverage projector startup launch receipt was not persisted"
            ));
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn authenticate_process_body(
    receipt: &EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
) -> Result<()> {
    if receipt.process_id != Some(std::process::id()) {
        return Err(anyhow!(
            "managed workspace coverage projector process disagrees with its launch receipt"
        ));
    }
    let executable = env::current_exe().context("failed to resolve projector executable")?;
    let executable_sha256 = format!(
        "sha256-{:x}",
        Sha256::digest(fs::read(&executable).with_context(|| {
            format!(
                "failed to fingerprint projector executable {}",
                executable.display()
            )
        })?)
    );
    if receipt.executable_sha256 != executable_sha256 {
        return Err(anyhow!(
            "managed workspace coverage projector executable disagrees with its launch receipt"
        ));
    }
    Ok(())
}

fn pulse_status(status: WorkspaceCoverageProjectorPulseStatus) -> &'static str {
    match status {
        WorkspaceCoverageProjectorPulseStatus::Idle => "idle",
        WorkspaceCoverageProjectorPulseStatus::Executed => "executed",
        WorkspaceCoverageProjectorPulseStatus::Contended => "contended",
        WorkspaceCoverageProjectorPulseStatus::Refused => "refused",
    }
}

struct Args {
    runtime_store: PathBuf,
    local_verse_store: PathBuf,
    runtime_id: String,
    interval_seconds: u64,
    startup_lifecycle_receipt_id: String,
    qdrant_url: String,
    ollama_base_url: String,
    ollama_model: String,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut values = env::args().skip(1);
        let command = values.next().context("missing command; use serve")?;
        if command != "serve" {
            return Err(anyhow!("unknown command {command:?}; use serve"));
        }
        let mut runtime_store = None;
        let mut local_verse_store = None;
        let mut runtime_id = None;
        let mut interval_seconds = None;
        let mut qdrant_url = None;
        let mut ollama_base_url = None;
        let mut ollama_model = None;
        while let Some(flag) = values.next() {
            let mut value = || {
                values
                    .next()
                    .with_context(|| format!("missing value for {flag}"))
            };
            match flag.as_str() {
                "--runtime-store" => runtime_store = Some(PathBuf::from(value()?)),
                "--local-verse-store" => local_verse_store = Some(PathBuf::from(value()?)),
                "--runtime-id" => runtime_id = Some(value()?),
                "--interval-seconds" => interval_seconds = Some(value()?.parse::<u64>()?),
                "--qdrant-url" => qdrant_url = Some(value()?),
                "--ollama-base-url" => ollama_base_url = Some(value()?),
                "--ollama-model" => ollama_model = Some(value()?),
                _ => return Err(anyhow!("unexpected argument {flag:?}")),
            }
        }
        let interval_seconds = interval_seconds.context("missing --interval-seconds")?;
        if interval_seconds == 0 {
            return Err(anyhow!("--interval-seconds must be positive"));
        }
        let startup_lifecycle_receipt_id =
            env::var("EPIPHANY_STARTUP_LIFECYCLE_RECEIPT_ID").unwrap_or_default();
        if startup_lifecycle_receipt_id.trim().is_empty() {
            return Err(anyhow!(
                "managed workspace coverage projector requires its startup launch receipt id"
            ));
        }
        Ok(Self {
            runtime_store: runtime_store.context("missing --runtime-store")?,
            local_verse_store: local_verse_store.context("missing --local-verse-store")?,
            runtime_id: runtime_id.context("missing --runtime-id")?,
            interval_seconds,
            startup_lifecycle_receipt_id,
            qdrant_url: required_nonempty(qdrant_url, "--qdrant-url")?,
            ollama_base_url: required_nonempty(ollama_base_url, "--ollama-base-url")?,
            ollama_model: required_nonempty(ollama_model, "--ollama-model")?,
        })
    }
}

fn required_nonempty(value: Option<String>, flag: &str) -> Result<String> {
    let value = value.with_context(|| format!("missing {flag}"))?;
    if value.trim().is_empty() {
        return Err(anyhow!("{flag} must not be empty"));
    }
    Ok(value)
}
