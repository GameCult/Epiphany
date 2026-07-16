use anyhow::{Context, Result, anyhow};
use ed25519_dalek::SigningKey;
use epiphany_core::{
    WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_SCHEMA_VERSION,
    WorkspaceCoverageManagedProcessLaunchEntry, WorkspaceCoverageProjectorConfig,
    WorkspaceCoverageProjectorPulseStatus, WorkspaceCoverageProjectorServiceBody,
    WorkspaceCoverageProviderHeartbeatEntry,
    authenticate_workspace_coverage_managed_process_launch, capture_process_instance,
    load_workspace_coverage_managed_process_launch,
    load_workspace_coverage_managed_process_launch_with_digest, native_boot_identity,
    open_default_host_identity, read_workspace_coverage_process_bootstrap,
    sign_workspace_coverage_heartbeat, write_workspace_coverage_provider_heartbeat,
};
use serde_json::json;
use std::env;
use std::io;
use std::path::PathBuf;
use std::thread;
use std::time::{Duration, Instant};
use uuid::Uuid;
use zeroize::Zeroize;

fn main() -> Result<()> {
    let args = Args::parse()?;
    let mut bootstrap = read_workspace_coverage_process_bootstrap(io::stdin().lock())?;
    if bootstrap.launch_id.to_string() != args.managed_process_launch_id {
        return Err(anyhow!(
            "bootstrap launch id disagrees with launch environment"
        ));
    }
    let provider_key = SigningKey::from_bytes(&bootstrap.provider_signing_seed);
    bootstrap.provider_signing_seed.zeroize();
    let (launch, launch_digest) = authenticate_managed_launch(&args, &provider_key)?;

    let mut config = WorkspaceCoverageProjectorConfig::from_env();
    config.qdrant_url = args.qdrant_url.clone();
    config.ollama_base_url = args.ollama_base_url.clone();
    config.ollama_model = args.ollama_model.clone();
    let mut projector = WorkspaceCoverageProjectorServiceBody::new(
        &args.runtime_store,
        &args.runtime_id,
        config,
        &launch.provider_incarnation_id,
        &launch.launch_id,
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
        let mut heartbeat = WorkspaceCoverageProviderHeartbeatEntry {
            schema_version: WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_SCHEMA_VERSION.to_string(),
            heartbeat_id: Uuid::new_v4().to_string(),
            launch_id: launch.launch_id.clone(),
            launch_envelope_digest: launch_digest.clone(),
            service_id: launch.service_id.clone(),
            provider_daemon_id: launch.provider_daemon_id.clone(),
            runtime_id: launch.runtime_id.clone(),
            host_identity_id: launch.host_identity_id.clone(),
            host_identity_record_digest: launch.host_identity_record_digest.clone(),
            boot_identity: launch.boot_identity.clone(),
            process_id: launch.process_id,
            process_creation_token: launch.process_creation_token,
            process_executable_path: launch.process_executable_path.clone(),
            provider_incarnation_id: launch.provider_incarnation_id.clone(),
            provider_public_key: launch.provider_public_key.clone(),
            sequence,
            status: if degraded { "degraded" } else { "ready" }.to_string(),
            observed_at_utc: chrono::Utc::now().to_rfc3339(),
            provider_signature: Vec::new(),
            signature_algorithm: "ed25519".to_string(),
        };
        sign_workspace_coverage_heartbeat(&mut heartbeat, &provider_key)?;
        let heartbeat = write_workspace_coverage_provider_heartbeat(
            &args.local_verse_store,
            args.runtime_id.clone(),
            heartbeat,
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
    provider_key: &SigningKey,
) -> Result<(WorkspaceCoverageManagedProcessLaunchEntry, String)> {
    let host = open_default_host_identity()?;
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if load_workspace_coverage_managed_process_launch(
            &args.local_verse_store,
            args.runtime_id.clone(),
            &args.managed_process_launch_id,
        )?
        .is_some()
        {
            let launch = authenticate_workspace_coverage_managed_process_launch(
                &args.local_verse_store,
                args.runtime_id.clone(),
                &args.managed_process_launch_id,
                host.entry(),
            )?;
            let own = capture_process_instance(std::process::id())?;
            let boot = native_boot_identity()
                .ok_or_else(|| anyhow!("native boot identity is not provable"))?;
            if launch.boot_identity != boot
                || launch.process_id != own.process_id
                || launch.process_creation_token != own.creation_token
                || PathBuf::from(&launch.process_executable_path) != own.executable_path
                || launch.provider_public_key != provider_key.verifying_key().to_bytes()
            {
                return Err(anyhow!(
                    "managed launch disagrees with this exact provider process"
                ));
            }
            let (_, digest) = load_workspace_coverage_managed_process_launch_with_digest(
                &args.local_verse_store,
                args.runtime_id.clone(),
                &args.managed_process_launch_id,
            )?
            .ok_or_else(|| anyhow!("managed launch disappeared"))?;
            return Ok((launch, digest));
        }
        if Instant::now() >= deadline {
            return Err(anyhow!(
                "managed workspace coverage projector launch was not persisted"
            ));
        }
        thread::sleep(Duration::from_millis(20));
    }
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
    managed_process_launch_id: String,
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
        let managed_process_launch_id =
            env::var("EPIPHANY_WORKSPACE_COVERAGE_LAUNCH_ID").unwrap_or_default();
        if managed_process_launch_id.trim().is_empty() {
            return Err(anyhow!(
                "managed workspace coverage projector requires its managed process launch id"
            ));
        }
        Ok(Self {
            runtime_store: runtime_store.context("missing --runtime-store")?,
            local_verse_store: local_verse_store.context("missing --local-verse-store")?,
            runtime_id: runtime_id.context("missing --runtime-id")?,
            interval_seconds,
            managed_process_launch_id,
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
