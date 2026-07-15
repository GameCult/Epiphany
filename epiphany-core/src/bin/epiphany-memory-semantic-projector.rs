use anyhow::{Context, Result, anyhow};
use epiphany_core::{
    EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_SCHEMA_VERSION,
    EpiphanyCultMeshDaemonHeartbeatEventEntry, MemorySemanticIndexConfig,
    MemorySemanticProjectionInput, MemorySemanticProjectorPulseStatus,
    SemanticProjectorServiceBody, publish_epiphany_cultmesh_semantic_projection_health,
    write_epiphany_cultmesh_daemon_heartbeat_event,
};
use serde_json::json;
use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;
use uuid::Uuid;

const DAEMON_ID: &str = "epiphany-memory-semantic-projector";

fn main() -> Result<()> {
    let args = Args::parse()?;
    let projector = SemanticProjectorServiceBody::new(
        &args.agent_store,
        &args.runtime_store,
        MemorySemanticIndexConfig::from_env(),
    )?;
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

        let mut health_faults = 0_u32;
        for input in &inputs {
            let store = source_store(&args, input)?;
            if publish_epiphany_cultmesh_semantic_projection_health(
                &args.local_verse_store,
                args.runtime_id.clone(),
                store,
                input,
                &provider_incarnation,
            )
            .is_err()
            {
                health_faults += 1;
            }
        }
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
                "selectedScopeId": outcome.selected_scope_id,
                "privateStateExposed": false,
                "authoritative": false,
                "queryAdmission": false
            }))?
        );
        if args.max_iterations != 0 && sequence >= args.max_iterations {
            break;
        }
        thread::sleep(Duration::from_secs(args.interval_seconds));
    }
    Ok(())
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
        })
    }
}
