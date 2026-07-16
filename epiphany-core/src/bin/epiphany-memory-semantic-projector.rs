use anyhow::{Context, Result, anyhow};
use epiphany_core::{
    EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_SCHEMA_VERSION,
    EpiphanyCultMeshDaemonHeartbeatEventEntry, EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
    MemorySemanticIndexConfig, MemorySemanticProjectionInput, MemorySemanticProjectorPulseStatus,
    SemanticProjectorServiceBody, authenticate_epiphany_cultmesh_semantic_projector_launch,
    load_epiphany_cultmesh_daemon_service_lifecycle_receipt,
    publish_epiphany_cultmesh_semantic_projection_health,
    write_epiphany_cultmesh_daemon_heartbeat_event,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use std::time::Instant;
use uuid::Uuid;

const DAEMON_ID: &str = "epiphany-memory-semantic-projector";

fn main() -> Result<()> {
    let args = Args::parse()?;
    if args.requires_managed_launch {
        if args.startup_lifecycle_receipt_id.trim().is_empty() {
            return Err(anyhow!(
                "managed projector requires its startup launch receipt id"
            ));
        }
        let receipt = authenticate_managed_launch(&args)?;
        if receipt.process_id != Some(std::process::id()) {
            return Err(anyhow!(
                "managed projector process disagrees with its launch receipt"
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
                "managed projector executable disagrees with its launch receipt"
            ));
        }
    }
    let mut semantic_config = MemorySemanticIndexConfig::from_env();
    semantic_config.qdrant_url = args.qdrant_url.clone();
    semantic_config.ollama_base_url = args.ollama_base_url.clone();
    semantic_config.ollama_model = args.ollama_model.clone();
    let projector =
        SemanticProjectorServiceBody::new(&args.agent_store, &args.runtime_store, semantic_config)?;
    let provider_incarnation = projector.provider_incarnation().to_string();
    let mut cursor = None;
    let mut sequence = 0_u64;
    loop {
        sequence = sequence.saturating_add(1);
        let pulse = projector.pulse(cursor.as_deref());
        let outcome = pulse.outcome;
        let inputs = pulse.inputs;
        let source_faults = pulse.source_fault_count;
        cursor = outcome
            .selected_scope_id
            .clone()
            .or_else(|| outcome.inspections.last().map(|row| row.scope_id.clone()));

        let mut health_publication_faults = Vec::new();
        for input in &inputs {
            let store = source_store(&args, input)?;
            if let Err(error) = publish_epiphany_cultmesh_semantic_projection_health(
                &args.local_verse_store,
                args.runtime_id.clone(),
                store,
                input,
                &provider_incarnation,
            ) {
                health_publication_faults
                    .push(format!("{}: {error:#}", input.obligation().partition));
            }
        }
        let health_faults = health_publication_faults.len() as u32;
        let degraded = source_faults > 0
            || health_faults > 0
            || matches!(
                outcome.status,
                MemorySemanticProjectorPulseStatus::Contended
                    | MemorySemanticProjectorPulseStatus::Refused
            );
        let heartbeat = write_epiphany_cultmesh_daemon_heartbeat_event(
            &args.local_verse_store,
            args.runtime_id.clone(),
            EpiphanyCultMeshDaemonHeartbeatEventEntry {
                schema_version: EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_SCHEMA_VERSION.to_string(),
                heartbeat_id: Uuid::new_v4().to_string(),
                daemon_id: DAEMON_ID.to_string(),
                cluster_id: "local".to_string(),
                provider_incarnation: provider_incarnation.clone(),
                sequence,
                status: if degraded { "degraded" } else { "ready" }.to_string(),
                heartbeat_at: chrono::Utc::now().to_rfc3339(),
                private_state_exposed: false,
                startup_lifecycle_receipt_id: args.startup_lifecycle_receipt_id.clone(),
            },
        )?;
        println!(
            "{}",
            serde_json::to_string(&json!({
                "schemaVersion": "epiphany.memory_semantic_projector_pulse.v0",
                "pulseSequence": sequence,
                "providerIncarnation": provider_incarnation,
                "heartbeatId": heartbeat.heartbeat_id,
                "providerStatus": heartbeat.status,
                "pulseStatus": pulse_status(outcome.status),
                "inspectedSourceCount": outcome.inspections.len(),
                "sourceFaultCount": source_faults,
                "healthPublicationFaultCount": health_faults,
                "healthPublicationFaults": health_publication_faults,
                "selectedScopeId": outcome.selected_scope_id,
                "privateStateExposed": false,
                "authoritative": false
            }))?
        );
        if args.max_iterations != 0 && sequence >= args.max_iterations {
            break;
        }
        thread::sleep(Duration::from_secs(args.interval_seconds));
    }
    Ok(())
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
            return authenticate_epiphany_cultmesh_semantic_projector_launch(
                &args.local_verse_store,
                args.runtime_id.clone(),
                &args.startup_lifecycle_receipt_id,
            );
        }
        if Instant::now() >= deadline {
            return Err(anyhow!(
                "managed projector startup launch receipt was not persisted"
            ));
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn source_store<'a>(args: &'a Args, input: &MemorySemanticProjectionInput) -> Result<&'a PathBuf> {
    match input.obligation().partition.as_str() {
        "mind" => Ok(&args.agent_store),
        "modeling" => Ok(&args.runtime_store),
        other => Err(anyhow!(
            "unsupported semantic projection partition {other:?}"
        )),
    }
}

fn pulse_status(status: MemorySemanticProjectorPulseStatus) -> &'static str {
    match status {
        MemorySemanticProjectorPulseStatus::Idle => "idle",
        MemorySemanticProjectorPulseStatus::Executed => "executed",
        MemorySemanticProjectorPulseStatus::Contended => "contended",
        MemorySemanticProjectorPulseStatus::Refused => "refused",
        MemorySemanticProjectorPulseStatus::Busy => "busy",
    }
}

struct Args {
    agent_store: PathBuf,
    runtime_store: PathBuf,
    local_verse_store: PathBuf,
    runtime_id: String,
    interval_seconds: u64,
    max_iterations: u64,
    startup_lifecycle_receipt_id: String,
    qdrant_url: String,
    ollama_base_url: String,
    ollama_model: String,
    requires_managed_launch: bool,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut values = env::args().skip(1);
        let command = values.next().unwrap_or_else(|| "pulse".to_string());
        if !matches!(command.as_str(), "pulse" | "once" | "serve" | "daemon") {
            return Err(anyhow!("unknown command {command:?}; use pulse or serve"));
        }
        let mut agent_store = None;
        let mut runtime_store = None;
        let mut local_verse_store = None;
        let mut runtime_id = "epiphany-local".to_string();
        let mut interval_seconds = 60_u64;
        let mut max_iterations = if matches!(command.as_str(), "pulse" | "once") {
            1
        } else {
            0
        };
        let mut qdrant_url = None;
        let mut ollama_base_url = None;
        let mut ollama_model = "qwen3-embedding:0.6b".to_string();
        while let Some(flag) = values.next() {
            let mut value = || {
                values
                    .next()
                    .with_context(|| format!("missing value for {flag}"))
            };
            match flag.as_str() {
                "--agent-store" => agent_store = Some(PathBuf::from(value()?)),
                "--runtime-store" => runtime_store = Some(PathBuf::from(value()?)),
                "--local-verse-store" => local_verse_store = Some(PathBuf::from(value()?)),
                "--runtime-id" => runtime_id = value()?,
                "--interval-seconds" => interval_seconds = value()?.parse()?,
                "--max-iterations" => max_iterations = value()?.parse()?,
                "--qdrant-url" => qdrant_url = Some(value()?),
                "--ollama-base-url" => ollama_base_url = Some(value()?),
                "--ollama-model" => ollama_model = value()?,
                _ => return Err(anyhow!("unexpected argument {flag:?}")),
            }
        }
        if interval_seconds == 0 {
            return Err(anyhow!("--interval-seconds must be positive"));
        }
        Ok(Self {
            agent_store: agent_store.context("missing --agent-store")?,
            runtime_store: runtime_store.context("missing --runtime-store")?,
            local_verse_store: local_verse_store.context("missing --local-verse-store")?,
            runtime_id,
            interval_seconds,
            max_iterations,
            startup_lifecycle_receipt_id: env::var("EPIPHANY_STARTUP_LIFECYCLE_RECEIPT_ID")
                .unwrap_or_default(),
            qdrant_url: qdrant_url.context("missing --qdrant-url")?,
            ollama_base_url: ollama_base_url.context("missing --ollama-base-url")?,
            ollama_model,
            requires_managed_launch: matches!(command.as_str(), "serve" | "daemon"),
        })
    }
}
