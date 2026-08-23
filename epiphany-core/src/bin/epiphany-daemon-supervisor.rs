use anyhow::Context;
use anyhow::Result;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use cultnet_rs::{
    GameCultProviderHealthIdentity, ServiceIdentitySigner, enroll_service_identity_at,
    export_service_identity_trust_anchor, open_service_identity_at,
};
use ed25519_dalek::SigningKey;
use epiphany_core::EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION;
use epiphany_core::EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION;
use epiphany_core::EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry;
use epiphany_core::EpiphanyCultMeshManagedServicePolicyEntry;
use epiphany_core::EpiphanyCultMeshSwarmBrakeEntry;
use epiphany_core::EpiphanyProcessObservation as ProcessObservation;
use epiphany_core::authenticate_epiphany_cultmesh_semantic_projector_launch;
use epiphany_core::authenticate_resident_provider;
use epiphany_core::idunn_recover_memory_semantic_projection_from_cultmesh;
use epiphany_core::load_current_epiphany_cultmesh_daemon_service_lifecycle_receipt_for_service;
use epiphany_core::load_epiphany_cultmesh_managed_service_policies;
use epiphany_core::load_epiphany_cultmesh_managed_service_policy;
use epiphany_core::load_epiphany_cultmesh_managed_service_policy_with_digest;
use epiphany_core::load_epiphany_cultmesh_swarm_brake;
use epiphany_core::load_epiphany_packaged_release;
use epiphany_core::load_latest_epiphany_cultmesh_daemon_heartbeat;
use epiphany_core::observe_native_process as observe_process;
use epiphany_core::runtime_modeling_semantic_projection_input;
use epiphany_core::write_epiphany_cultmesh_daemon_service_lifecycle_receipt;
use epiphany_core::write_epiphany_cultmesh_semantic_projector_service_policy;
use epiphany_core::write_epiphany_cultmesh_workspace_coverage_projector_service_policy;
use epiphany_core::{
    EpiphanyAggregateRuntimeHealthInput, derive_epiphany_aggregate_runtime_health,
    publish_idunn_daemon_health_rudp, sign_epiphany_runtime_health,
};
use epiphany_core::{EpiphanyPackagedReleaseEntry, authenticate_epiphany_packaged_release};
use epiphany_core::{
    ProcessInstanceIdentity, ProcessInstanceObservation,
    WORKSPACE_COVERAGE_PROCESS_LAUNCH_SCHEMA_VERSION, WorkspaceCoverageManagedProcessLaunchEntry,
    WorkspaceCoverageProcessBootstrap, WorkspaceCoverageProcessLifecycleObservation,
    authenticate_recovery_workspace_coverage_claim_sight,
    authenticate_workspace_coverage_managed_process_launch,
    authenticate_workspace_coverage_provider_heartbeat,
    authenticate_workspace_coverage_replacement_lineage,
    authenticate_workspace_coverage_termination_with_envelope_digest, capture_process_instance,
    load_latest_workspace_coverage_managed_process_launch,
    load_latest_workspace_coverage_provider_heartbeat,
    load_workspace_coverage_process_termination_observation, native_boot_identity,
    observe_historical_workspace_coverage_managed_process, observe_process_instance,
    observe_workspace_coverage_managed_process, open_default_host_identity,
    process_identity_from_workspace_coverage_launch, sign_workspace_coverage_launch,
    workspace_coverage_host_identity_record_digest,
    write_workspace_coverage_managed_process_launch, write_workspace_coverage_process_bootstrap,
    write_workspace_coverage_process_termination_observation,
    write_workspace_coverage_recovery_directive,
};
use epiphany_core::{
    authenticate_current_workspace_coverage_advancement_sight,
    authenticate_current_workspace_coverage_terminal_sight,
};
use epiphany_core::{
    epiphany_packaged_release_binary_path, epiphany_packaged_release_witness_sha256,
};
use rand_core::{OsRng, RngCore};
use serde_json::json;
use sha2::Digest;
use sha2::Sha256;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::process::Child;
use std::process::Command;
use std::process::Stdio;
use std::thread;
use std::time::Duration as StdDuration;
use uuid::Uuid;
use zeroize::Zeroize;

const SEMANTIC_PROJECTOR_SERVICE_ID: &str = "epiphany-memory-semantic-projector-service";
const SEMANTIC_PROJECTOR_EXECUTOR_ID: &str = "epiphany-memory-semantic-projector";
const WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID: &str =
    "epiphany-workspace-coverage-projector-service";
const WORKSPACE_COVERAGE_PROJECTOR_EXECUTOR_ID: &str = "epiphany-workspace-coverage-projector";
const AGGREGATE_HEARTBEAT_FRESH_SECONDS: i64 = 180;
const WORKSPACE_PROGRESS_NO_ADVANCE_LEASE_SECONDS: i64 = 300;

enum ManagedServiceLineage {
    Current,
    Pending,
    Stale(String),
}

#[cfg(windows)]
use std::os::windows::process::CommandExt;

fn main() -> Result<()> {
    let args = Args::parse()?;
    let fatal_log = args.fatal_log.clone();
    let result = dispatch(args);
    if let Err(error) = &result
        && let Some(path) = fatal_log
    {
        let _ = append_fatal_log(&path, error);
    }
    result
}

fn dispatch(args: Args) -> Result<()> {
    match args.command.as_str() {
        "managed-service-serve" => managed_service_serve(args),
        "provider-health-identity-enroll" => provider_health_identity_enroll(args),
        "provider-health-identity-export" => provider_health_identity_export(args),
        "semantic-projector-service-policy" => semantic_projector_service_policy(args),
        "workspace-coverage-projector-service-policy" => {
            workspace_coverage_projector_service_policy(args)
        }
        "semantic-recover" => semantic_recover(args),
        other => anyhow::bail!(
            "unknown command {other:?}; use managed-service-serve, provider-health-identity-enroll/export, semantic-projector-service-policy, workspace-coverage-projector-service-policy, or semantic-recover"
        ),
    }
}

fn append_fatal_log(path: &Path, error: &anyhow::Error) -> Result<()> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{} {}", Utc::now().to_rfc3339(), error)?;
    Ok(())
}

fn provider_health_identity_enroll(args: Args) -> Result<()> {
    let path = args.idunn_provider_health_identity_store.context(
        "provider-health identity enrollment requires --idunn-provider-health-identity-store",
    )?;
    let signer = enroll_service_identity_at::<GameCultProviderHealthIdentity>(&path)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schemaVersion": "epiphany.provider_health_identity_enrollment.v2",
            "identityId": signer.entry().identity_id,
            "publicKeyHex": lowercase_hex(&signer.entry().public_key),
            "privateStateExposed": false,
        }))?
    );
    Ok(())
}

fn provider_health_identity_export(args: Args) -> Result<()> {
    let private = args.idunn_provider_health_identity_store.context(
        "provider-health identity export requires --idunn-provider-health-identity-store",
    )?;
    let public = args.idunn_provider_health_public_anchor.context(
        "provider-health identity export requires --idunn-provider-health-public-anchor",
    )?;
    if private == public {
        anyhow::bail!("provider-health private identity and public anchor paths must differ");
    }
    let signer = open_service_identity_at::<GameCultProviderHealthIdentity>(&private)?;
    export_service_identity_trust_anchor(&signer, &public)?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schemaVersion": "epiphany.provider_health_identity_export.v2",
            "identityId": signer.entry().identity_id,
            "publicKeyHex": lowercase_hex(&signer.entry().public_key),
            "publicAnchor": public,
            "privateStateExposed": false,
        }))?
    );
    Ok(())
}

fn lowercase_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn pinned_packaged_release(
    args: &Args,
    require_digest: bool,
) -> Result<(EpiphanyPackagedReleaseEntry, String)> {
    let release_id = args
        .release_id
        .as_deref()
        .context("managed-service deployment requires --release-id")?;
    let witness = load_epiphany_packaged_release(&args.store, &args.runtime_id, release_id)?
        .context("pinned packaged release is absent")?;
    let digest = match args.release_witness_sha256.as_deref() {
        Some(expected) => expected.to_string(),
        None if require_digest => {
            anyhow::bail!("managed-service runtime requires --release-witness-sha256")
        }
        None => epiphany_packaged_release_witness_sha256(&witness)?,
    };
    let authenticated =
        authenticate_epiphany_packaged_release(&args.store, &args.runtime_id, release_id, &digest)?;
    Ok((authenticated, digest))
}

fn required<'a>(value: &'a Option<String>, flag: &str) -> Result<&'a str> {
    value.as_deref().with_context(|| format!("missing {flag}"))
}

fn semantic_recover(args: Args) -> Result<()> {
    let store = args
        .runtime_store
        .as_ref()
        .context("semantic-recover requires --runtime-store")?;
    let input = runtime_modeling_semantic_projection_input(store)?;
    let (authorization, claim) = idunn_recover_memory_semantic_projection_from_cultmesh(
        &args.store,
        args.runtime_id.clone(),
        store,
        &input,
        required(&args.expected_claim_id, "--expected-claim-id")?,
        SEMANTIC_PROJECTOR_EXECUTOR_ID,
        required(&args.receipt_id, "--receipt-id")?,
        required(&args.provider_heartbeat_id, "--provider-heartbeat-id")?,
        &Utc::now().to_rfc3339(),
    )?;
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "authorizationId": authorization.authorization_id,
            "epoch": claim.epoch,
            "executorId": claim.executor_id,
            "privateStateExposed": false
        }))?
    );
    Ok(())
}

fn managed_service_serve(args: Args) -> Result<()> {
    let (release, _) = pinned_packaged_release(&args, true)?;
    let expected_supervisor = fs::canonicalize(epiphany_packaged_release_binary_path(
        &release,
        "supervisor",
    )?)?;
    if fs::canonicalize(env::current_exe()?)? != expected_supervisor {
        anyhow::bail!("managed-service reconciler executable is not the pinned release supervisor");
    }
    let health_signer = args
        .idunn_provider_health_identity_store
        .as_deref()
        .map(open_service_identity_at::<GameCultProviderHealthIdentity>)
        .transpose()?;
    let health_publisher_incarnation = Uuid::new_v4().to_string();
    let mut iteration = 0_u64;
    loop {
        let (release, release_witness_sha256) = pinned_packaged_release(&args, true)?;
        iteration = iteration
            .checked_add(1)
            .context("aggregate runtime health sequence exhausted")?;
        let policies =
            load_epiphany_cultmesh_managed_service_policies(&args.store, args.runtime_id.clone())?;
        let brake = load_epiphany_cultmesh_swarm_brake(&args.store, args.runtime_id.clone())?;
        let lifecycle_braked = swarm_brake_blocks_service_lifecycle_entry(brake.as_ref());
        if !lifecycle_braked {
            for policy in &policies {
                let mut service_args = args.clone();
                service_args.service_id = policy.service_id.clone();
                managed_service_reconcile(service_args)?;
            }
        }
        publish_managed_service_iteration_health(
            &args,
            &release,
            &release_witness_sha256,
            &policies,
            lifecycle_braked,
            &health_publisher_incarnation,
            iteration,
            health_signer.as_ref(),
        );
        if args.max_iterations != 0 && iteration >= args.max_iterations {
            break;
        }
        std::thread::sleep(std::time::Duration::from_secs(
            args.loop_interval_seconds.max(0) as u64,
        ));
    }
    Ok(())
}

const REQUIRED_MANAGED_HEALTH_SERVICES: [&str; 2] = [
    SEMANTIC_PROJECTOR_SERVICE_ID,
    WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID,
];

fn required_managed_health_services(lifecycle_braked: bool) -> &'static [&'static str] {
    if lifecycle_braked {
        &[]
    } else {
        &REQUIRED_MANAGED_HEALTH_SERVICES
    }
}

fn publish_managed_service_iteration_health(
    args: &Args,
    release: &EpiphanyPackagedReleaseEntry,
    authenticated_release_witness_sha256: &str,
    policies: &[EpiphanyCultMeshManagedServicePolicyEntry],
    lifecycle_braked: bool,
    publisher_incarnation_id: &str,
    publisher_sequence: u64,
    health_signer: Option<&ServiceIdentitySigner<GameCultProviderHealthIdentity>>,
) {
    let (Some(endpoint), Some(daemon_id), Some(health_contract), Some(health_signer)) = (
        args.idunn_rudp_health,
        args.idunn_daemon.as_deref(),
        args.idunn_health_contract.as_deref(),
        health_signer,
    ) else {
        return;
    };
    let required = required_managed_health_services(lifecycle_braked);
    let mut expected = required.len();
    let mut terminal_current = 0_usize;
    let mut warming = 0_usize;
    let mut contradictions = Vec::new();
    if let Some(resident_store) = args.resident_self_store.as_deref() {
        expected += 1;
        let provider = authenticate_resident_provider(
            release,
            authenticated_release_witness_sha256,
            resident_store,
            args.resident_provider_stale_seconds.saturating_mul(1000),
        );
        terminal_current += provider.terminal_current;
        warming += provider.warming;
        contradictions.extend(provider.contradictions);
    }
    for &service_id in required {
        let Some(policy) = policies
            .iter()
            .find(|policy| policy.service_id == service_id)
        else {
            continue;
        };
        if !policy.enabled {
            continue;
        }
        match managed_service_lineage(args, release, policy) {
            Ok(ManagedServiceLineage::Current)
                if policy.service_id == WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID =>
            {
                let runtime_store = match args.runtime_store.as_deref() {
                    Some(store) => store,
                    None => {
                        contradictions
                            .push("workspace health lacks the bound runtime store".to_string());
                        continue;
                    }
                };
                let host = match open_default_host_identity() {
                    Ok(host) => host,
                    Err(error) => {
                        contradictions.push(format!(
                            "workspace health cannot authenticate host: {error:#}"
                        ));
                        continue;
                    }
                };
                match authenticate_current_workspace_coverage_terminal_sight(
                    runtime_store,
                    &args.store,
                    &args.runtime_id,
                    host.entry(),
                ) {
                    Ok(Some(_authority)) => {
                        terminal_current += 1;
                    }
                    Ok(None) => {
                        let launch = match load_latest_workspace_coverage_managed_process_launch(
                            &args.store,
                            args.runtime_id.clone(),
                        ) {
                            Ok(Some(launch)) => launch,
                            Ok(None) => continue,
                            Err(error) => {
                                contradictions.push(format!(
                                    "workspace health cannot load launch: {error:#}"
                                ));
                                continue;
                            }
                        };
                        match authenticate_current_workspace_coverage_advancement_sight(
                            runtime_store,
                            &args.store,
                            &args.runtime_id,
                            &launch.launch_id,
                            host.entry(),
                        ) {
                            Ok(Some(authority)) => {
                                let advanced =
                                    DateTime::parse_from_rfc3339(&authority.last_advanced_at_utc)
                                        .map(|time| time.with_timezone(&Utc));
                                match advanced {
                                    Ok(advanced) if Utc::now().signed_duration_since(advanced) >= Duration::zero()
                                        && Utc::now().signed_duration_since(advanced) <= Duration::seconds(WORKSPACE_PROGRESS_NO_ADVANCE_LEASE_SECONDS) => {
                                            warming += 1;
                                        }
                                    Ok(_) => contradictions.push("workspace advancement sight exceeded the supervisor no-advance lease".into()),
                                    Err(error) => contradictions.push(format!("workspace advancement sight time is invalid: {error:#}")),
                                }
                            }
                            Ok(None) => {}
                            Err(error) => contradictions.push(format!(
                                "workspace advancement authority is invalid: {error:#}"
                            )),
                        }
                    }
                    Err(error) => contradictions.push(format!(
                        "workspace terminal authority is invalid: {error:#}"
                    )),
                }
            }
            Ok(ManagedServiceLineage::Current) => terminal_current += 1,
            Ok(ManagedServiceLineage::Pending) => {}
            Ok(ManagedServiceLineage::Stale(reason)) => {
                contradictions.push(format!("{}: {reason}", policy.service_id))
            }
            Err(error) => contradictions.push(format!("{}: {error:#}", policy.service_id)),
        }
    }
    let health = derive_epiphany_aggregate_runtime_health(EpiphanyAggregateRuntimeHealthInput {
        daemon_id: daemon_id.to_string(),
        health_contract: health_contract.to_string(),
        observed_at: Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        release_authenticated: true,
        expected_service_count: expected,
        terminal_current_service_count: terminal_current,
        warming_service_count: warming,
        contradictions,
    });
    match health.and_then(|health| {
        let witness_sha256 = epiphany_packaged_release_witness_sha256(release)?;
        if witness_sha256 != authenticated_release_witness_sha256 {
            anyhow::bail!("aggregate health release witness changed after authentication");
        }
        let signed = sign_epiphany_runtime_health(
            health,
            &args.runtime_id,
            &release.release_id,
            &witness_sha256,
            &release.source_commit_sha,
            args.idunn_deployment_request_id
                .as_deref()
                .context("aggregate health requires --idunn-deployment-request-id")?,
            publisher_incarnation_id,
            publisher_sequence,
            health_signer,
        )?;
        publish_idunn_daemon_health_rudp(endpoint, &args.runtime_id, &signed)
    }) {
        Ok(()) => {}
        Err(error) => eprintln!("Epiphany could not publish aggregate Idunn health: {error:#}"),
    }
}

fn managed_service_lineage(
    args: &Args,
    release: &EpiphanyPackagedReleaseEntry,
    policy: &EpiphanyCultMeshManagedServicePolicyEntry,
) -> Result<ManagedServiceLineage> {
    let (_, policy_digest) = load_epiphany_cultmesh_managed_service_policy_with_digest(
        &args.store,
        args.runtime_id.clone(),
        &policy.service_id,
    )?
    .with_context(|| {
        format!(
            "managed service policy disappeared for {}",
            policy.service_id
        )
    })?;
    if policy.service_id == WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID {
        let Some(launch) = load_latest_workspace_coverage_managed_process_launch(
            &args.store,
            args.runtime_id.clone(),
        )?
        else {
            return Ok(ManagedServiceLineage::Pending);
        };
        let host = open_default_host_identity()?;
        let launch = authenticate_workspace_coverage_managed_process_launch(
            &args.store,
            args.runtime_id.clone(),
            &launch.launch_id,
            host.entry(),
        )?;
        if launch.policy_envelope_digest != policy_digest {
            return Ok(ManagedServiceLineage::Stale(
                "workspace child launch disagrees with current policy digest".into(),
            ));
        }
        let expected = fs::canonicalize(epiphany_packaged_release_binary_path(
            release,
            "workspace-coverage-projector",
        )?)?;
        if fs::canonicalize(&launch.process_executable_path)? != expected {
            return Ok(ManagedServiceLineage::Stale(
                "workspace child executable is outside current packaged release".into(),
            ));
        }
        let Some(heartbeat) = load_latest_workspace_coverage_provider_heartbeat(
            &args.store,
            args.runtime_id.clone(),
            &launch.launch_id,
        )?
        else {
            return Ok(ManagedServiceLineage::Pending);
        };
        let heartbeat = authenticate_workspace_coverage_provider_heartbeat(
            &args.store,
            args.runtime_id.clone(),
            &heartbeat.heartbeat_id,
            host.entry(),
        )?;
        if heartbeat.status != "ready" || !timestamp_is_fresh(&heartbeat.observed_at_utc)? {
            return Ok(ManagedServiceLineage::Pending);
        }
        let identity = ProcessInstanceIdentity {
            process_id: launch.process_id,
            creation_token: launch.process_creation_token,
            created_at_rfc3339: launch.process_created_at_rfc3339,
            executable_path: PathBuf::from(launch.process_executable_path),
        };
        return Ok(
            if observe_process_instance(&identity) == ProcessInstanceObservation::ExactAlive {
                ManagedServiceLineage::Current
            } else {
                ManagedServiceLineage::Pending
            },
        );
    }
    let Some(receipt) =
        load_current_epiphany_cultmesh_daemon_service_lifecycle_receipt_for_service(
            &args.store,
            args.runtime_id.clone(),
            &policy.service_id,
        )?
    else {
        return Ok(ManagedServiceLineage::Pending);
    };
    if receipt.managed_policy_id != policy.policy_id
        || receipt.managed_policy_digest != policy_digest
        || receipt.command != policy.command
        || receipt.args != policy.args
        || receipt.cwd != policy.cwd
    {
        lifecycle_process_identity(&receipt)?;
        return Ok(ManagedServiceLineage::Stale(
            "alive child lineage disagrees with current managed policy".into(),
        ));
    }
    if policy.service_id == SEMANTIC_PROJECTOR_SERVICE_ID {
        authenticate_epiphany_cultmesh_semantic_projector_launch(
            &args.store,
            args.runtime_id.clone(),
            &receipt.receipt_id,
        )?;
        let expected = fs::canonicalize(epiphany_packaged_release_binary_path(
            release,
            "semantic-projector",
        )?)?;
        if fs::canonicalize(&receipt.command)? != expected {
            lifecycle_process_identity(&receipt)?;
            return Ok(ManagedServiceLineage::Stale(
                "semantic child executable is outside current packaged release".into(),
            ));
        }
        let heartbeat = load_latest_epiphany_cultmesh_daemon_heartbeat(
            &args.store,
            args.runtime_id.clone(),
            &receipt.provider_daemon_id,
        )?;
        let launch_completed = receipt
            .completed_at_utc
            .as_deref()
            .context("semantic launch receipt has no spawn completion time")?;
        if !semantic_heartbeat_is_ready(
            heartbeat.as_ref(),
            &receipt.receipt_id,
            launch_completed,
            Utc::now(),
        )? {
            return Ok(ManagedServiceLineage::Pending);
        }
    }
    let identity = lifecycle_process_identity(&receipt)?;
    Ok(
        if observe_process_instance(&identity) == ProcessInstanceObservation::ExactAlive {
            ManagedServiceLineage::Current
        } else {
            ManagedServiceLineage::Pending
        },
    )
}

fn lifecycle_process_identity(
    receipt: &EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
) -> Result<ProcessInstanceIdentity> {
    process_identity_from_parts(
        receipt.process_id,
        receipt.process_creation_token,
        receipt.process_created_at_rfc3339.clone(),
        &receipt.process_executable_path,
    )
}

fn process_identity_from_parts(
    process_id: Option<u32>,
    creation_token: u64,
    created_at_rfc3339: Option<String>,
    executable_path: &str,
) -> Result<ProcessInstanceIdentity> {
    if creation_token == 0 || executable_path.trim().is_empty() {
        anyhow::bail!("launch receipt has no authenticated process-instance identity");
    }
    Ok(ProcessInstanceIdentity {
        process_id: process_id.context("launch receipt has no process id")?,
        creation_token,
        created_at_rfc3339,
        executable_path: PathBuf::from(executable_path),
    })
}

fn replacement_process_identity(
    lineage: &ManagedServiceLineage,
    receipt: &EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
) -> Result<Option<ProcessInstanceIdentity>> {
    replacement_identity_from_parts(
        lineage,
        receipt.process_id,
        receipt.process_creation_token,
        receipt.process_created_at_rfc3339.clone(),
        &receipt.process_executable_path,
    )
}

fn replacement_identity_from_parts(
    lineage: &ManagedServiceLineage,
    process_id: Option<u32>,
    creation_token: u64,
    created_at_rfc3339: Option<String>,
    executable_path: &str,
) -> Result<Option<ProcessInstanceIdentity>> {
    match lineage {
        ManagedServiceLineage::Stale(_) => process_identity_from_parts(
            process_id,
            creation_token,
            created_at_rfc3339,
            executable_path,
        )
        .map(Some),
        ManagedServiceLineage::Current | ManagedServiceLineage::Pending => Ok(None),
    }
}

fn timestamp_is_fresh(value: &str) -> Result<bool> {
    timestamp_is_fresh_at(value, Utc::now())
}

fn semantic_heartbeat_is_ready(
    heartbeat: Option<&epiphany_core::EpiphanyCultMeshDaemonHeartbeatEventEntry>,
    lifecycle_receipt_id: &str,
    launch_completed_at: &str,
    now: DateTime<Utc>,
) -> Result<bool> {
    let Some(heartbeat) = heartbeat else {
        return Ok(false);
    };
    Ok(heartbeat.status == "ready"
        && heartbeat.startup_lifecycle_receipt_id == lifecycle_receipt_id
        && DateTime::parse_from_rfc3339(&heartbeat.heartbeat_at)?
            > DateTime::parse_from_rfc3339(launch_completed_at)?
        && timestamp_is_fresh_at(&heartbeat.heartbeat_at, now)?)
}

fn timestamp_is_fresh_at(value: &str, now: DateTime<Utc>) -> Result<bool> {
    let observed = DateTime::parse_from_rfc3339(value)?.with_timezone(&Utc);
    let age = now.signed_duration_since(observed);
    Ok(age >= Duration::zero() && age <= Duration::seconds(AGGREGATE_HEARTBEAT_FRESH_SECONDS))
}

#[derive(Clone)]
struct CoverageReplacementEvidence {
    old_launch_id: String,
    termination_id: String,
    termination_envelope_digest: String,
}

fn spawn_managed_service(
    args: &Args,
    command_path: &Path,
    environment: &[(&str, String)],
    piped_stdin: bool,
) -> Result<(Child, Vec<String>, DateTime<Utc>)> {
    let brake = load_epiphany_cultmesh_swarm_brake(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle_entry(brake.as_ref())?;
    let started_at = Utc::now();
    let service_args = args.service_args.clone();
    let mut command = Command::new(command_path);
    command.args(&service_args);
    for (key, value) in environment {
        command.env(key, value);
    }
    if piped_stdin {
        command.stdin(Stdio::piped());
    } else {
        command.stdin(Stdio::null());
    }
    if let Some(stdout_path) = &args.stdout_artifact {
        if let Some(parent) = stdout_path.parent() {
            fs::create_dir_all(parent)?;
        }
        command.stdout(Stdio::from(fs::File::create(stdout_path)?));
    } else {
        command.stdout(Stdio::null());
    }
    if let Some(stderr_path) = &args.stderr_artifact {
        if let Some(parent) = stderr_path.parent() {
            fs::create_dir_all(parent)?;
        }
        command.stderr(Stdio::from(fs::File::create(stderr_path)?));
    } else {
        command.stderr(Stdio::null());
    }
    if let Some(cwd) = &args.cwd {
        command.current_dir(cwd);
    }
    #[cfg(windows)]
    {
        command.creation_flags(0x08000000);
    }
    let child = command
        .spawn()
        .with_context(|| format!("failed to launch service {}", command_path.display()))?;
    Ok((child, service_args, started_at))
}

fn launch_semantic_projector(args: Args) -> Result<()> {
    let (policy, policy_digest) = load_epiphany_cultmesh_managed_service_policy_with_digest(
        &args.store,
        args.runtime_id.clone(),
        SEMANTIC_PROJECTOR_SERVICE_ID,
    )?
    .context("semantic projector managed policy is absent")?;
    let command_path = service_command_path(&args)?;
    if command_path != PathBuf::from(&policy.command)
        || args.service_args != policy.args
        || args.cwd.as_ref().map(|path| path.display().to_string()) != policy.cwd
    {
        anyhow::bail!("semantic projector launch must use the exact current managed policy");
    }
    let receipt_id = Uuid::new_v4().to_string();
    let executable_sha256 = local_file_sha256(&command_path.display().to_string())
        .map(|digest| format!("sha256-{digest}"))
        .context("semantic projector executable cannot be fingerprinted")?;
    let environment = [("EPIPHANY_STARTUP_LIFECYCLE_RECEIPT_ID", receipt_id.clone())];
    let (mut child, service_args, started_at) =
        spawn_managed_service(&args, &command_path, &environment, false)?;
    let persist_result = (|| {
        let identity = capture_process_instance(child.id())
            .context("failed to capture exact semantic projector process identity")?;
        if identity.executable_path != command_path.canonicalize()? {
            anyhow::bail!("spawned semantic projector executable disagrees with policy command");
        }
        let mut receipt = service_lifecycle_receipt(
            &args,
            "launch",
            "launched",
            command_path.display().to_string(),
            service_args,
            Some(child.id()),
            None,
            started_at,
            Some(Utc::now()),
            args.stdout_artifact
                .as_ref()
                .map(|path| path.display().to_string()),
        );
        receipt.receipt_id = receipt_id.clone();
        receipt.managed_policy_id = policy.policy_id;
        receipt.managed_policy_digest = policy_digest;
        receipt.provider_daemon_id = SEMANTIC_PROJECTOR_EXECUTOR_ID.to_string();
        receipt.startup_correlation_id = receipt_id;
        receipt.executable_sha256 = executable_sha256;
        receipt.process_creation_token = identity.creation_token;
        receipt.process_created_at_rfc3339 = identity.created_at_rfc3339;
        receipt.process_executable_path = identity.executable_path.display().to_string();
        write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
            &args.store,
            args.runtime_id.clone(),
            receipt,
        )
    })();
    match persist_result {
        Ok(_) => Ok(()),
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            Err(error).context("failed to establish authenticated semantic launch")
        }
    }
}

fn launch_workspace_coverage_projector(
    args: Args,
    replacement: Option<CoverageReplacementEvidence>,
) -> Result<WorkspaceCoverageManagedProcessLaunchEntry> {
    let (policy, policy_digest) = load_epiphany_cultmesh_managed_service_policy_with_digest(
        &args.store,
        args.runtime_id.clone(),
        WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID,
    )?
    .context("workspace coverage managed policy is absent")?;
    let command_path = service_command_path(&args)?;
    if command_path != PathBuf::from(&policy.command)
        || args.service_args != policy.args
        || args.cwd.as_ref().map(|path| path.display().to_string()) != policy.cwd
    {
        anyhow::bail!("workspace coverage launch must use the exact current managed policy");
    }
    let host = open_default_host_identity()
        .context("workspace coverage launch requires an enrolled host identity")?;
    let boot = native_boot_identity()
        .context("workspace coverage launch requires a proven native boot identity")?;
    let launch_id = Uuid::new_v4();
    let incarnation_id = Uuid::new_v4();
    let mut seed = [0_u8; 32];
    OsRng.fill_bytes(&mut seed);
    let provider_key = SigningKey::from_bytes(&seed);
    let environment = [(
        "EPIPHANY_WORKSPACE_COVERAGE_LAUNCH_ID",
        launch_id.to_string(),
    )];
    let (mut child, _, started_at) =
        spawn_managed_service(&args, &command_path, &environment, true)?;
    let persist_result = (|| -> Result<WorkspaceCoverageManagedProcessLaunchEntry> {
        let process = capture_process_instance(child.id())
            .context("failed to capture exact workspace coverage process identity")?;
        let canonical_executable = command_path
            .canonicalize()
            .context("failed to canonicalize workspace coverage executable")?;
        if process.executable_path != canonical_executable {
            anyhow::bail!(
                "spawned workspace coverage process executable disagrees with policy command"
            );
        }
        let executable_sha256 = local_file_sha256(&canonical_executable.display().to_string())
            .map(|digest| format!("sha256-{digest}"))
            .context("workspace coverage executable cannot be fingerprinted")?;
        let mut bootstrap = WorkspaceCoverageProcessBootstrap {
            launch_id,
            provider_signing_seed: seed,
        };
        let mut stdin = child
            .stdin
            .take()
            .context("workspace coverage child stdin is unavailable")?;
        write_workspace_coverage_process_bootstrap(&mut stdin, &bootstrap)?;
        drop(stdin);
        bootstrap.provider_signing_seed.zeroize();
        seed.zeroize();
        let mut launch = WorkspaceCoverageManagedProcessLaunchEntry {
            schema_version: WORKSPACE_COVERAGE_PROCESS_LAUNCH_SCHEMA_VERSION.to_string(),
            launch_id: launch_id.to_string(),
            service_id: WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID.to_string(),
            provider_daemon_id: WORKSPACE_COVERAGE_PROJECTOR_EXECUTOR_ID.to_string(),
            runtime_id: args.runtime_id.clone(),
            policy_id: policy.policy_id,
            policy_envelope_digest: policy_digest,
            command: policy.command,
            args: policy.args,
            cwd: policy.cwd,
            launched_at_utc: started_at.to_rfc3339(),
            host_identity_id: host.entry().identity_id.clone(),
            host_public_key: host.entry().public_key.clone(),
            host_assurance: host.entry().assurance.clone(),
            host_identity_record_digest: workspace_coverage_host_identity_record_digest(
                host.entry(),
            )?,
            boot_identity: boot,
            process_id: process.process_id,
            process_creation_token: process.creation_token,
            process_created_at_rfc3339: process.created_at_rfc3339,
            process_executable_path: process.executable_path.display().to_string(),
            executable_sha256,
            provider_incarnation_id: incarnation_id.to_string(),
            provider_public_key: provider_key.verifying_key().to_bytes().to_vec(),
            host_signature: Vec::new(),
            supervisor_id: "epiphany-daemon-supervisor".to_string(),
            identity_captured_at_utc: Utc::now().to_rfc3339(),
            signature_algorithm: "ed25519".to_string(),
            replaces_launch_id: replacement
                .as_ref()
                .map(|evidence| evidence.old_launch_id.clone()),
            replaces_termination_id: replacement
                .as_ref()
                .map(|evidence| evidence.termination_id.clone()),
            replaces_termination_envelope_digest: replacement
                .as_ref()
                .map(|evidence| evidence.termination_envelope_digest.clone()),
        };
        sign_workspace_coverage_launch(&mut launch, &host)?;
        write_workspace_coverage_managed_process_launch(
            &args.store,
            args.runtime_id.clone(),
            launch,
            host.entry(),
        )
    })();
    seed.zeroize();
    let written = match persist_result {
        Ok(written) => written,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            return Err(error)
                .context("failed to establish authenticated workspace coverage launch");
        }
    };
    Ok(written)
}

fn build_managed_service_policy(args: &Args) -> Result<EpiphanyCultMeshManagedServicePolicyEntry> {
    let command = service_command_path(&args)?;
    let stdout_artifact = args.stdout_artifact.clone().unwrap_or_else(|| {
        PathBuf::from(format!(
            ".epiphany-run/services/{}.stdout.log",
            args.service_id
        ))
    });
    let stderr_artifact = args.stderr_artifact.clone().unwrap_or_else(|| {
        PathBuf::from(format!(
            ".epiphany-run/services/{}.stderr.log",
            args.service_id
        ))
    });
    Ok(EpiphanyCultMeshManagedServicePolicyEntry {
        schema_version: EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION.to_string(),
        policy_id: format!("managed-service-policy-{}", sanitize_id(&args.service_id)),
        service_id: args.service_id.clone(),
        command: command.display().to_string(),
        args: args.service_args.clone(),
        cwd: args.cwd.as_ref().map(|path| path.display().to_string()),
        enabled: !args.disabled,
        cooldown_seconds: args.cooldown_seconds,
        stdout_artifact: stdout_artifact.display().to_string(),
        stderr_artifact: stderr_artifact.display().to_string(),
        private_state_exposed: false,
    })
}

fn report_managed_service_policy(
    written: &EpiphanyCultMeshManagedServicePolicyEntry,
) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schemaVersion": written.schema_version,
            "status": "written",
            "policyId": written.policy_id,
            "serviceId": written.service_id,
            "enabled": written.enabled,
            "command": written.command,
            "args": written.args,
            "stdoutArtifact": written.stdout_artifact,
            "stderrArtifact": written.stderr_artifact,
            "privateStateExposed": written.private_state_exposed,
        }))?
    );
    Ok(())
}

fn bind_managed_policy(args: &mut Args, policy: &EpiphanyCultMeshManagedServicePolicyEntry) {
    args.service_command = Some(PathBuf::from(&policy.command));
    args.service_args = policy.args.clone();
    args.cwd = policy.cwd.as_ref().map(PathBuf::from);
    args.stdout_artifact = Some(PathBuf::from(&policy.stdout_artifact));
    args.stderr_artifact = Some(PathBuf::from(&policy.stderr_artifact));
}

fn semantic_projector_service_policy(mut args: Args) -> Result<()> {
    let runtime_store = args
        .runtime_store
        .as_ref()
        .context("semantic-projector-service-policy requires --runtime-store")?;
    let qdrant_url = args
        .qdrant_url
        .as_deref()
        .context("semantic-projector-service-policy requires --qdrant-url")?;
    let ollama_base_url = args
        .ollama_base_url
        .as_deref()
        .context("semantic-projector-service-policy requires --ollama-base-url")?;
    args.service_id = SEMANTIC_PROJECTOR_SERVICE_ID.to_string();
    args.service_command = Some(packaged_role_command_path(&args, "semantic-projector")?);
    args.service_args = semantic_projector_service_args(
        runtime_store,
        &args.store,
        &args.runtime_id,
        args.loop_interval_seconds,
        qdrant_url,
        ollama_base_url,
        &args.ollama_model,
    );
    if args.max_iterations != 0 {
        anyhow::bail!("semantic projector managed service must not have a finite iteration limit");
    }
    let policy = build_managed_service_policy(&args)?;
    let written = write_epiphany_cultmesh_semantic_projector_service_policy(
        &args.store,
        args.runtime_id,
        policy,
    )?;
    report_managed_service_policy(&written)
}

fn workspace_coverage_projector_service_policy(mut args: Args) -> Result<()> {
    let runtime_store = args
        .runtime_store
        .as_ref()
        .context("workspace-coverage-projector-service-policy requires --runtime-store")?;
    let qdrant_url = args
        .qdrant_url
        .as_deref()
        .context("workspace-coverage-projector-service-policy requires --qdrant-url")?;
    let ollama_base_url = args
        .ollama_base_url
        .as_deref()
        .context("workspace-coverage-projector-service-policy requires --ollama-base-url")?;
    args.service_id = WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID.to_string();
    args.service_command = Some(packaged_role_command_path(
        &args,
        "workspace-coverage-projector",
    )?);
    args.service_args = workspace_coverage_projector_service_args(
        runtime_store,
        &args.store,
        &args.runtime_id,
        args.loop_interval_seconds,
        qdrant_url,
        ollama_base_url,
        &args.ollama_model,
    );
    if args.max_iterations != 0 {
        anyhow::bail!(
            "workspace coverage projector managed service must not have a finite iteration limit"
        );
    }
    let policy = build_managed_service_policy(&args)?;
    let written = write_epiphany_cultmesh_workspace_coverage_projector_service_policy(
        &args.store,
        args.runtime_id,
        policy,
    )?;
    report_managed_service_policy(&written)
}

fn packaged_role_command_path(args: &Args, role: &str) -> Result<PathBuf> {
    let (release, _) = pinned_packaged_release(args, true)?;
    epiphany_packaged_release_binary_path(&release, role)
}

fn workspace_coverage_projector_service_args(
    runtime_store: &Path,
    local_verse_store: &Path,
    runtime_id: &str,
    interval_seconds: i64,
    qdrant_url: &str,
    ollama_base_url: &str,
    ollama_model: &str,
) -> Vec<String> {
    vec![
        "serve".to_string(),
        "--runtime-store".to_string(),
        runtime_store.display().to_string(),
        "--local-verse-store".to_string(),
        local_verse_store.display().to_string(),
        "--runtime-id".to_string(),
        runtime_id.to_string(),
        "--interval-seconds".to_string(),
        interval_seconds.to_string(),
        "--heartbeat-interval-seconds".to_string(),
        "10".to_string(),
        "--qdrant-url".to_string(),
        qdrant_url.to_string(),
        "--ollama-base-url".to_string(),
        ollama_base_url.to_string(),
        "--ollama-model".to_string(),
        ollama_model.to_string(),
    ]
}

fn semantic_projector_service_args(
    runtime_store: &Path,
    local_verse_store: &Path,
    runtime_id: &str,
    interval_seconds: i64,
    qdrant_url: &str,
    ollama_base_url: &str,
    ollama_model: &str,
) -> Vec<String> {
    vec![
        "serve".to_string(),
        "--runtime-store".to_string(),
        runtime_store.display().to_string(),
        "--local-verse-store".to_string(),
        local_verse_store.display().to_string(),
        "--runtime-id".to_string(),
        runtime_id.to_string(),
        "--interval-seconds".to_string(),
        interval_seconds.to_string(),
        "--qdrant-url".to_string(),
        qdrant_url.to_string(),
        "--ollama-base-url".to_string(),
        ollama_base_url.to_string(),
        "--ollama-model".to_string(),
        ollama_model.to_string(),
    ]
}

fn managed_service_reconcile(mut args: Args) -> Result<()> {
    let brake = load_epiphany_cultmesh_swarm_brake(&args.store, args.runtime_id.clone())?;
    assert_swarm_brake_allows_service_lifecycle_entry(brake.as_ref())?;
    let pinned_release = pinned_packaged_release(&args, true)?.0;
    let policy = load_epiphany_cultmesh_managed_service_policy(
        &args.store,
        args.runtime_id.clone(),
        &args.service_id,
    )?
    .with_context(|| format!("managed service policy missing for {}", args.service_id))?;
    let expected_role = match policy.service_id.as_str() {
        SEMANTIC_PROJECTOR_SERVICE_ID => "semantic-projector",
        WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID => "workspace-coverage-projector",
        other => anyhow::bail!("unsupported managed service policy {other}"),
    };
    if fs::canonicalize(&policy.command)?
        != fs::canonicalize(epiphany_packaged_release_binary_path(
            &pinned_release,
            expected_role,
        )?)?
    {
        anyhow::bail!(
            "reserved managed-service policy command is outside pinned release role {expected_role}"
        );
    }
    if policy.service_id == WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID {
        return reconcile_workspace_coverage_projector(args, policy);
    }
    let current = load_current_epiphany_cultmesh_daemon_service_lifecycle_receipt_for_service(
        &args.store,
        args.runtime_id.clone(),
        &args.service_id,
    )?;
    let mut observation = current
        .as_ref()
        .and_then(|receipt| receipt.process_id)
        .map(observe_process)
        .transpose()?
        .unwrap_or(ProcessObservation::Missing);
    if observation == ProcessObservation::Alive {
        let lineage =
            managed_service_lineage(&args, &pinned_release, &policy).with_context(|| {
                format!("failed to authenticate {} child lineage", policy.service_id)
            })?;
        if let Some(identity) = replacement_process_identity(
            &lineage,
            current
                .as_ref()
                .context("alive managed service has no lifecycle receipt")?,
        )? {
            terminate_native_process_instance(&identity)?;
            observation = ProcessObservation::Missing;
        }
    }
    if !policy.enabled {
        return Ok(());
    }
    if observation == ProcessObservation::Alive {
        return Ok(());
    }
    if !args.force && policy.cooldown_seconds > 0 {
        if let Some(started) = current
            .as_ref()
            .and_then(|receipt| DateTime::parse_from_rfc3339(&receipt.started_at_utc).ok())
        {
            let elapsed = Utc::now().signed_duration_since(started.with_timezone(&Utc));
            if elapsed < Duration::seconds(policy.cooldown_seconds) {
                return Ok(());
            }
        }
    }
    bind_managed_policy(&mut args, &policy);
    launch_semantic_projector(args)
}

fn terminate_native_process_instance(identity: &ProcessInstanceIdentity) -> Result<()> {
    if observe_process_instance(identity) != ProcessInstanceObservation::ExactAlive {
        anyhow::bail!("stale managed-service process identity drifted before termination");
    }
    let process_id = identity.process_id;
    #[cfg(windows)]
    let status = Command::new("taskkill")
        .args(["/PID", &process_id.to_string(), "/T", "/F"])
        .status()?;
    #[cfg(not(windows))]
    let status = Command::new("kill")
        .args(["-TERM", &process_id.to_string()])
        .status()?;
    if !status.success() {
        anyhow::bail!("failed to terminate stale managed-service process {process_id}");
    }
    Ok(())
}

fn reconcile_workspace_coverage_projector(
    mut args: Args,
    policy: EpiphanyCultMeshManagedServicePolicyEntry,
) -> Result<()> {
    if !policy.enabled {
        return Ok(());
    }
    let runtime_store = args.runtime_store.clone().context(
        "workspace coverage reconciliation requires --runtime-store for Body/route sight",
    )?;
    let host = open_default_host_identity()
        .context("workspace coverage reconciliation requires an enrolled host identity")?;
    let latest = load_latest_workspace_coverage_managed_process_launch(
        &args.store,
        args.runtime_id.clone(),
    )?;
    let Some(latest) = latest else {
        bind_managed_policy(&mut args, &policy);
        launch_workspace_coverage_projector(args, None)?;
        return Ok(());
    };
    let current_policy_matches_latest = latest.policy_id == policy.policy_id
        && latest.command == policy.command
        && latest.args == policy.args
        && latest.cwd == policy.cwd
        && load_epiphany_cultmesh_managed_service_policy_with_digest(
            &args.store,
            args.runtime_id.clone(),
            &policy.service_id,
        )?
        .is_some_and(|(_, digest)| digest == latest.policy_envelope_digest);
    if !current_policy_matches_latest {
        let observation = observe_historical_workspace_coverage_managed_process(
            &args.store,
            args.runtime_id.clone(),
            &latest.launch_id,
            host.entry(),
        )?;
        if observation == WorkspaceCoverageProcessLifecycleObservation::ExactAlive {
            terminate_native_process_instance(&process_identity_from_workspace_coverage_launch(
                &latest,
            ))?;
            return Ok(());
        }
        if matches!(
            observation,
            WorkspaceCoverageProcessLifecycleObservation::Inaccessible
                | WorkspaceCoverageProcessLifecycleObservation::Indeterminate { .. }
        ) {
            return Ok(());
        }
        if load_workspace_coverage_process_termination_observation(
            &args.store,
            args.runtime_id.clone(),
            &latest.launch_id,
        )?
        .is_none()
        {
            write_workspace_coverage_process_termination_observation(
                &args.store,
                args.runtime_id.clone(),
                &latest.launch_id,
                &host,
            )?;
        }
        let (termination, termination_digest) =
            authenticate_workspace_coverage_termination_with_envelope_digest(
                &args.store,
                args.runtime_id.clone(),
                &latest.launch_id,
                host.entry(),
            )?;
        bind_managed_policy(&mut args, &policy);
        launch_workspace_coverage_projector(
            args,
            Some(CoverageReplacementEvidence {
                old_launch_id: latest.launch_id,
                termination_id: termination.termination_id,
                termination_envelope_digest: termination_digest,
            }),
        )?;
        return Ok(());
    }
    let terminal_sight = authenticate_current_workspace_coverage_terminal_sight(
        &runtime_store,
        &args.store,
        &args.runtime_id,
        host.entry(),
    )?;
    if let Some(terminal) = terminal_sight {
        if terminal.launch_id != latest.launch_id {
            authenticate_workspace_coverage_replacement_lineage(
                &args.store,
                &args.runtime_id,
                &terminal.launch_id,
                &latest.launch_id,
                host.entry(),
            )?;
        }
        let observation = observe_workspace_coverage_managed_process(
            &args.store,
            args.runtime_id.clone(),
            &latest.launch_id,
            host.entry(),
        )?;
        match observation {
            WorkspaceCoverageProcessLifecycleObservation::ExactAlive => return Ok(()),
            WorkspaceCoverageProcessLifecycleObservation::Inaccessible
            | WorkspaceCoverageProcessLifecycleObservation::Indeterminate { .. } => return Ok(()),
            WorkspaceCoverageProcessLifecycleObservation::BootSuperseded { .. }
            | WorkspaceCoverageProcessLifecycleObservation::ExactExited { .. }
            | WorkspaceCoverageProcessLifecycleObservation::Missing
            | WorkspaceCoverageProcessLifecycleObservation::Replaced { .. } => {}
        }
        if load_workspace_coverage_process_termination_observation(
            &args.store,
            args.runtime_id.clone(),
            &latest.launch_id,
        )?
        .is_none()
        {
            write_workspace_coverage_process_termination_observation(
                &args.store,
                args.runtime_id.clone(),
                &latest.launch_id,
                &host,
            )?;
        }
        let (termination, termination_digest) =
            authenticate_workspace_coverage_termination_with_envelope_digest(
                &args.store,
                args.runtime_id.clone(),
                &latest.launch_id,
                host.entry(),
            )?;
        bind_managed_policy(&mut args, &policy);
        launch_workspace_coverage_projector(
            args,
            Some(CoverageReplacementEvidence {
                old_launch_id: latest.launch_id,
                termination_id: termination.termination_id,
                termination_envelope_digest: termination_digest,
            }),
        )?;
        return Ok(());
    }
    let target = authenticate_recovery_workspace_coverage_claim_sight(
        &args.store,
        &runtime_store,
        &args.runtime_id,
        host.entry(),
    )?;
    if target
        .as_ref()
        .is_some_and(|claim| claim.launch_id != latest.launch_id)
    {
        let old_target = target
            .clone()
            .context("workspace coverage mismatch lost its authenticated claim target")?;
        let replacement = authenticate_workspace_coverage_managed_process_launch(
            &args.store,
            args.runtime_id.clone(),
            &latest.launch_id,
            host.entry(),
        )?;
        authenticate_workspace_coverage_replacement_lineage(
            &args.store,
            &args.runtime_id,
            &old_target.launch_id,
            &latest.launch_id,
            host.entry(),
        )?;
        return match observe_workspace_coverage_managed_process(
            &args.store,
            args.runtime_id.clone(),
            &replacement.launch_id,
            host.entry(),
        )? {
            WorkspaceCoverageProcessLifecycleObservation::ExactAlive => {
                finish_workspace_coverage_recovery(
                    &args,
                    &runtime_store,
                    host.entry(),
                    &host,
                    old_target,
                    replacement,
                )
            }
            _ => Ok(()),
        };
    }
    let observation = observe_workspace_coverage_managed_process(
        &args.store,
        args.runtime_id.clone(),
        &latest.launch_id,
        host.entry(),
    )?;
    match observation {
        WorkspaceCoverageProcessLifecycleObservation::ExactAlive => return Ok(()),
        WorkspaceCoverageProcessLifecycleObservation::Inaccessible
        | WorkspaceCoverageProcessLifecycleObservation::Indeterminate { .. } => return Ok(()),
        WorkspaceCoverageProcessLifecycleObservation::BootSuperseded { .. }
        | WorkspaceCoverageProcessLifecycleObservation::ExactExited { .. }
        | WorkspaceCoverageProcessLifecycleObservation::Missing
        | WorkspaceCoverageProcessLifecycleObservation::Replaced { .. } => {}
    }

    if load_workspace_coverage_process_termination_observation(
        &args.store,
        args.runtime_id.clone(),
        &latest.launch_id,
    )?
    .is_none()
    {
        write_workspace_coverage_process_termination_observation(
            &args.store,
            args.runtime_id.clone(),
            &latest.launch_id,
            &host,
        )?;
    }
    let (termination, termination_digest) =
        authenticate_workspace_coverage_termination_with_envelope_digest(
            &args.store,
            args.runtime_id.clone(),
            &latest.launch_id,
            host.entry(),
        )?;
    // Claim acquisition may race the first reconciliation observation.  The
    // terminal proof is the cut: authenticate Body sight again after it, and
    // use only this post-terminal target for replacement/recovery authority.
    let post_terminal_target = authenticate_recovery_workspace_coverage_claim_sight(
        &args.store,
        &runtime_store,
        &args.runtime_id,
        host.entry(),
    )?;
    if post_terminal_target
        .as_ref()
        .is_some_and(|claim| claim.launch_id != latest.launch_id)
    {
        anyhow::bail!("post-terminal claim sight does not name the terminated managed launch");
    }
    let replacement_evidence = CoverageReplacementEvidence {
        old_launch_id: latest.launch_id.clone(),
        termination_id: termination.termination_id.clone(),
        termination_envelope_digest: termination_digest,
    };
    let existing_replacement = load_latest_workspace_coverage_managed_process_launch(
        &args.store,
        args.runtime_id.clone(),
    )?
    .filter(|launch| {
        launch.replaces_launch_id.as_deref() == Some(latest.launch_id.as_str())
            && launch.replaces_termination_id.as_deref()
                == Some(termination.termination_id.as_str())
    });
    let replacement = if let Some(existing) = existing_replacement {
        existing
    } else {
        bind_managed_policy(&mut args, &policy);
        launch_workspace_coverage_projector(args.clone(), Some(replacement_evidence))?
    };

    let Some(target) = post_terminal_target else {
        return Ok(());
    };

    finish_workspace_coverage_recovery(
        &args,
        &runtime_store,
        host.entry(),
        &host,
        target,
        replacement,
    )
}

fn finish_workspace_coverage_recovery(
    args: &Args,
    runtime_store: &Path,
    host_identity: &epiphany_core::HostIncarnationIdentityEntry,
    host: &epiphany_core::HostIdentitySigner,
    target: epiphany_core::WorkspaceCoverageClaimSightEntry,
    replacement: WorkspaceCoverageManagedProcessLaunchEntry,
) -> Result<()> {
    let ready = (0..100).find_map(|_| {
        let observed = load_latest_workspace_coverage_provider_heartbeat(
            &args.store,
            args.runtime_id.clone(),
            &replacement.launch_id,
        )
        .ok()
        .flatten();
        if let Some(heartbeat) = observed
            && heartbeat.status == "ready"
            && timestamp_is_fresh(&heartbeat.observed_at_utc).unwrap_or(false)
            && authenticate_workspace_coverage_provider_heartbeat(
                &args.store,
                args.runtime_id.clone(),
                &heartbeat.heartbeat_id,
                host_identity,
            )
            .is_ok()
        {
            return Some(heartbeat);
        }
        thread::sleep(StdDuration::from_millis(100));
        None
    });
    let Some(ready) = ready else {
        anyhow::bail!("workspace coverage replacement did not publish signed readiness in time");
    };
    let actuation_observation = observe_workspace_coverage_managed_process(
        &args.store,
        args.runtime_id.clone(),
        &replacement.launch_id,
        host_identity,
    )?;
    match actuation_observation {
        WorkspaceCoverageProcessLifecycleObservation::ExactAlive => {
            write_workspace_coverage_recovery_directive(
                &args.store,
                runtime_store,
                &args.runtime_id,
                &target,
                &replacement.launch_id,
                &ready.heartbeat_id,
                host,
            )?;
        }
        WorkspaceCoverageProcessLifecycleObservation::Inaccessible
        | WorkspaceCoverageProcessLifecycleObservation::Indeterminate { .. }
        | WorkspaceCoverageProcessLifecycleObservation::BootSuperseded { .. }
        | WorkspaceCoverageProcessLifecycleObservation::ExactExited { .. }
        | WorkspaceCoverageProcessLifecycleObservation::Missing
        | WorkspaceCoverageProcessLifecycleObservation::Replaced { .. } => {}
    }
    Ok(())
}

fn local_file_sha256(path: &str) -> Option<String> {
    if path.trim().is_empty() || path == "none" {
        return None;
    }
    let path = PathBuf::from(path);
    if !path.is_file() {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    let digest = Sha256::digest(&bytes);
    Some(format!("{digest:x}"))
}
fn service_command_path(args: &Args) -> Result<PathBuf> {
    args.service_command
        .clone()
        .context("managed service command was not derived from its packaged role")
}

fn service_lifecycle_receipt(
    args: &Args,
    action: &str,
    status: &str,
    command: String,
    service_args: Vec<String>,
    process_id: Option<u32>,
    exit_code: Option<i32>,
    started_at: DateTime<Utc>,
    completed_at: Option<DateTime<Utc>>,
    operator_artifact_ref: Option<String>,
) -> EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry {
    let receipt_id = format!(
        "daemon-service-lifecycle-receipt-{}-{}-{}-{}",
        sanitize_id(&args.service_id),
        sanitize_id(action),
        started_at.timestamp_millis(),
        Uuid::new_v4()
    );
    EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry {
        schema_version: EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION
            .to_string(),
        receipt_id,
        service_id: args.service_id.clone(),
        scheduler_id: "epiphany-daemon-supervisor".to_string(),
        runtime_id: args.runtime_id.clone(),
        daemon_selector: "*".to_string(),
        action: action.to_string(),
        status: status.to_string(),
        command,
        args: service_args,
        cwd: args.cwd.as_ref().map(|path| path.display().to_string()),
        process_id,
        exit_code,
        started_at_utc: started_at.to_rfc3339(),
        completed_at_utc: completed_at.map(|instant| instant.to_rfc3339()),
        operator_artifact_ref: operator_artifact_ref.unwrap_or_else(|| {
            format!(
                "service://{}/{}",
                sanitize_id(&args.service_id),
                sanitize_id(action)
            )
        }),
        private_state_exposed: false,
        notes: vec!["Idunn observed this managed-service lifecycle consequence.".to_string()],
        executable_sha256: String::new(),
        schema_catalog_sha256: String::new(),
        preflight_witness_id: String::new(),
        required_document_types: Vec::new(),
        schema_preflight_passed: false,
        managed_policy_id: String::new(),
        managed_policy_digest: String::new(),
        provider_daemon_id: String::new(),
        startup_correlation_id: String::new(),
        process_creation_token: 0,
        process_created_at_rfc3339: None,
        process_executable_path: String::new(),
    }
}

fn assert_swarm_brake_allows_service_lifecycle_entry(
    brake: Option<&EpiphanyCultMeshSwarmBrakeEntry>,
) -> Result<()> {
    if let Some(brake) = brake
        && swarm_brake_blocks_service_lifecycle_entry(Some(brake))
    {
        anyhow::bail!(
            "local Verse swarm brake engaged; refusing daemon supervisor service lifecycle action; scope={}; protected={}; reason={}",
            brake.scope,
            brake.protected_surfaces.join(","),
            brake.reason
        );
    }
    Ok(())
}

fn swarm_brake_blocks_service_lifecycle_entry(
    brake: Option<&EpiphanyCultMeshSwarmBrakeEntry>,
) -> bool {
    brake.is_some_and(|brake| {
        brake.status == "engaged"
            && matches!(brake.scope.as_str(), "swarm" | "all")
            && (brake.protected_surfaces.is_empty()
                || brake.protected_surfaces.iter().any(|surface| {
                    surface == "daemon.lifecycle_poke" || surface == "daemon.*" || surface == "*"
                }))
    })
}

fn sanitize_id(raw: &str) -> String {
    raw.chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch } else { '-' })
        .collect()
}

#[cfg(test)]
mod service_lifecycle_brake_authority_tests {
    use super::*;

    fn brake(protected_surfaces: &[&str]) -> EpiphanyCultMeshSwarmBrakeEntry {
        EpiphanyCultMeshSwarmBrakeEntry {
            schema_version: "epiphany.cultmesh.swarm_brake.v0".into(),
            brake_id: "brake-test".into(),
            status: "engaged".into(),
            scope: "all".into(),
            reason: "sleep".into(),
            operator_agent_id: "operator".into(),
            affected_clusters: vec!["local".into()],
            protected_surfaces: protected_surfaces.iter().map(|v| (*v).into()).collect(),
            created_at_utc: "2026-07-18T00:00:00Z".into(),
            expires_at_utc: None,
            private_state_exposed: false,
            notes: Vec::new(),
            runtime_id: "runtime".into(),
        }
    }

    #[test]
    fn cognitive_scheduler_brake_does_not_claim_service_physiology() {
        let cognitive = brake(&[
            "heartbeat.scheduler",
            "coordinator.run",
            "persona.public_speech",
            "daemon.tool_invocation",
        ]);
        assert!(assert_swarm_brake_allows_service_lifecycle_entry(Some(&cognitive)).is_ok());
        assert!(!swarm_brake_blocks_service_lifecycle_entry(Some(
            &cognitive
        )));
        assert_eq!(required_managed_health_services(false).len(), 2);
        for surface in ["daemon.lifecycle_poke", "daemon.*", "*"] {
            let lifecycle = brake(&[surface]);
            assert!(assert_swarm_brake_allows_service_lifecycle_entry(Some(&lifecycle)).is_err());
            assert!(swarm_brake_blocks_service_lifecycle_entry(Some(&lifecycle)));
            assert!(required_managed_health_services(true).is_empty());
        }
    }
}

#[cfg(test)]
mod supervisor_invariant_tests {
    use super::*;

    #[test]
    fn idunn_health_configuration_is_explicit_and_all_or_none() {
        let endpoint: SocketAddr = "127.0.0.1:17870".parse().unwrap();
        let identity_store = Path::new("provider-health.cc");
        assert!(validate_idunn_health_options(None, None, None, None, None).is_ok());
        assert!(
            validate_idunn_health_options(
                Some(&endpoint),
                Some("yggdrasil-epiphany"),
                Some("epiphany.cultnet-rudp-runtime-health"),
                Some("deploy-request-test"),
                Some(identity_store),
            )
            .is_ok()
        );
        assert!(validate_idunn_health_options(Some(&endpoint), None, None, None, None).is_err());
        assert!(validate_idunn_health_options(None, Some("epiphany"), None, None, None).is_err());
        assert!(
            validate_idunn_health_options(
                Some(&endpoint),
                Some("yggdrasil-epiphany"),
                Some("epiphany.cultnet-rudp-runtime-health"),
                Some("deploy-request-test"),
                None,
            )
            .is_err()
        );
    }

    #[test]
    fn stale_child_replacement_fails_closed_and_revalidates_process_identity() {
        assert!(process_identity_from_parts(Some(7), 0, None, "projector").is_err());
        assert!(process_identity_from_parts(Some(7), 9, None, "").is_err());
        let identity = process_identity_from_parts(Some(7), 9, None, "projector").unwrap();
        assert_eq!(identity.process_id, 7);
        assert!(
            replacement_identity_from_parts(&ManagedServiceLineage::Pending, Some(7), 0, None, "",)
                .unwrap()
                .is_none()
        );
        assert!(
            replacement_identity_from_parts(
                &ManagedServiceLineage::Stale("policy mismatch".into()),
                Some(7),
                0,
                None,
                "",
            )
            .is_err()
        );

        let mut live = capture_process_instance(std::process::id()).unwrap();
        live.creation_token = live.creation_token.saturating_add(1);
        assert!(matches!(
            observe_process_instance(&live),
            ProcessInstanceObservation::Replaced { .. }
        ));
    }

    #[test]
    fn aggregate_heartbeat_freshness_has_a_bounded_authority_window() {
        let now = DateTime::parse_from_rfc3339("2026-07-16T12:03:00Z")
            .unwrap()
            .with_timezone(&Utc);
        assert!(timestamp_is_fresh_at("2026-07-16T12:00:00Z", now).unwrap());
        assert!(!timestamp_is_fresh_at("2026-07-16T11:59:59Z", now).unwrap());
        assert!(!timestamp_is_fresh_at("2026-07-16T12:03:01Z", now).unwrap());

        let heartbeat = epiphany_core::EpiphanyCultMeshDaemonHeartbeatEventEntry {
            schema_version: "epiphany.cultmesh.daemon_heartbeat_event.v0".into(),
            heartbeat_id: "heartbeat".into(),
            daemon_id: SEMANTIC_PROJECTOR_EXECUTOR_ID.into(),
            cluster_id: "local".into(),
            provider_incarnation: "provider".into(),
            sequence: 1,
            status: "ready".into(),
            heartbeat_at: "2026-07-16T12:02:00Z".into(),
            private_state_exposed: false,
            startup_lifecycle_receipt_id: "receipt".into(),
        };
        assert!(
            !semantic_heartbeat_is_ready(None, "receipt", "2026-07-16T12:00:00Z", now).unwrap()
        );
        assert!(
            semantic_heartbeat_is_ready(Some(&heartbeat), "receipt", "2026-07-16T12:00:00Z", now,)
                .unwrap()
        );
        let mut alien = heartbeat.clone();
        alien.startup_lifecycle_receipt_id = "other".into();
        assert!(
            !semantic_heartbeat_is_ready(Some(&alien), "receipt", "2026-07-16T12:00:00Z", now,)
                .unwrap()
        );
    }
}

#[derive(Clone)]
struct Args {
    command: String,
    store: PathBuf,
    runtime_id: String,
    cwd: Option<PathBuf>,
    force: bool,
    disabled: bool,
    cooldown_seconds: i64,
    service_id: String,
    service_command: Option<PathBuf>,
    service_args: Vec<String>,
    loop_interval_seconds: i64,
    max_iterations: u64,
    receipt_id: Option<String>,
    stdout_artifact: Option<PathBuf>,
    stderr_artifact: Option<PathBuf>,
    runtime_store: Option<PathBuf>,
    expected_claim_id: Option<String>,
    provider_heartbeat_id: Option<String>,
    qdrant_url: Option<String>,
    ollama_base_url: Option<String>,
    ollama_model: String,
    fatal_log: Option<PathBuf>,
    release_id: Option<String>,
    release_witness_sha256: Option<String>,
    idunn_rudp_health: Option<SocketAddr>,
    idunn_daemon: Option<String>,
    idunn_health_contract: Option<String>,
    idunn_deployment_request_id: Option<String>,
    idunn_provider_health_identity_store: Option<PathBuf>,
    idunn_provider_health_public_anchor: Option<PathBuf>,
    resident_self_store: Option<PathBuf>,
    resident_provider_stale_seconds: u64,
}

impl Args {
    fn parse() -> Result<Self> {
        let mut values = env::args().skip(1);
        let command = values.next().context("missing supervisor command")?;
        let mut store = PathBuf::from(".epiphany-run/cultmesh/local-verse.ccmp");
        let mut store_explicit = false;
        let mut runtime_id = "epiphany-local".to_string();
        let mut cwd = None;
        let mut force = false;
        let mut disabled = false;
        let mut cooldown_seconds = 0_i64;
        let service_id = String::new();
        let service_command = None;
        let service_args = Vec::new();
        let mut loop_interval_seconds = 60_i64;
        let mut max_iterations = 0_u64;
        let mut receipt_id = None;
        let mut stdout_artifact = None;
        let mut stderr_artifact = None;
        let mut runtime_store = None;
        let mut expected_claim_id = None;
        let mut provider_heartbeat_id = None;
        let mut qdrant_url = None;
        let mut ollama_base_url = None;
        let mut ollama_model = "qwen3-embedding:0.6b".to_string();
        let mut fatal_log = None;
        let mut release_id = None;
        let mut release_witness_sha256 = None;
        let mut idunn_rudp_health = None;
        let mut idunn_daemon = None;
        let mut idunn_health_contract = None;
        let mut idunn_deployment_request_id = None;
        let mut idunn_provider_health_identity_store = None;
        let mut idunn_provider_health_public_anchor = None;
        let mut resident_self_store = None;
        let mut resident_provider_stale_seconds = 180_u64;

        while let Some(arg) = values.next() {
            match arg.as_str() {
                "--store" => {
                    store = PathBuf::from(values.next().context("missing --store value")?);
                    store_explicit = true;
                }
                "--runtime-id" => {
                    runtime_id = values.next().context("missing --runtime-id value")?
                }
                "--cwd" => cwd = Some(PathBuf::from(values.next().context("missing --cwd value")?)),
                "--force" => force = true,
                "--disabled" => disabled = true,
                "--cooldown-seconds" => {
                    cooldown_seconds = values
                        .next()
                        .context("missing --cooldown-seconds value")?
                        .parse()?;
                }
                "--loop-interval-seconds" | "--serve-interval-seconds" => {
                    loop_interval_seconds = values
                        .next()
                        .context("missing --loop-interval-seconds value")?
                        .parse()?;
                }
                "--max-iterations" => {
                    max_iterations = values
                        .next()
                        .context("missing --max-iterations value")?
                        .parse()?;
                }
                "--receipt-id" => {
                    receipt_id = Some(values.next().context("missing --receipt-id value")?)
                }
                "--stdout-artifact" => {
                    stdout_artifact = Some(PathBuf::from(
                        values.next().context("missing --stdout-artifact value")?,
                    ));
                }
                "--stderr-artifact" => {
                    stderr_artifact = Some(PathBuf::from(
                        values.next().context("missing --stderr-artifact value")?,
                    ));
                }
                "--runtime-store" => {
                    runtime_store = Some(PathBuf::from(
                        values.next().context("missing --runtime-store value")?,
                    ))
                }
                "--expected-claim-id" => {
                    expected_claim_id =
                        Some(values.next().context("missing --expected-claim-id value")?)
                }
                "--provider-heartbeat-id" => {
                    provider_heartbeat_id = Some(
                        values
                            .next()
                            .context("missing --provider-heartbeat-id value")?,
                    )
                }
                "--qdrant-url" => {
                    qdrant_url = Some(values.next().context("missing --qdrant-url value")?)
                }
                "--ollama-base-url" => {
                    ollama_base_url =
                        Some(values.next().context("missing --ollama-base-url value")?)
                }
                "--ollama-model" => {
                    ollama_model = values.next().context("missing --ollama-model value")?
                }
                "--fatal-log" => {
                    fatal_log = Some(PathBuf::from(
                        values.next().context("missing --fatal-log value")?,
                    ));
                }
                "--release-id" => {
                    release_id = Some(values.next().context("missing --release-id value")?);
                }
                "--release-witness-sha256" => {
                    release_witness_sha256 = Some(
                        values
                            .next()
                            .context("missing --release-witness-sha256 value")?,
                    );
                }
                "--idunn-rudp-health" => {
                    idunn_rudp_health = Some(
                        values
                            .next()
                            .context("missing --idunn-rudp-health value")?
                            .parse()?,
                    );
                }
                "--idunn-daemon" => {
                    idunn_daemon = Some(values.next().context("missing --idunn-daemon value")?);
                }
                "--idunn-health-contract" => {
                    idunn_health_contract = Some(
                        values
                            .next()
                            .context("missing --idunn-health-contract value")?,
                    );
                }
                "--idunn-deployment-request-id" => {
                    idunn_deployment_request_id = Some(
                        values
                            .next()
                            .context("missing --idunn-deployment-request-id value")?,
                    );
                }
                "--idunn-provider-health-identity-store" => {
                    idunn_provider_health_identity_store =
                        Some(PathBuf::from(values.next().context(
                            "missing --idunn-provider-health-identity-store value",
                        )?));
                }
                "--idunn-provider-health-public-anchor" => {
                    idunn_provider_health_public_anchor =
                        Some(PathBuf::from(values.next().context(
                            "missing --idunn-provider-health-public-anchor value",
                        )?));
                }
                "--resident-self-store" => {
                    resident_self_store = Some(PathBuf::from(
                        values
                            .next()
                            .context("missing --resident-self-store value")?,
                    ));
                }
                "--resident-provider-stale-seconds" => {
                    resident_provider_stale_seconds = values
                        .next()
                        .context("missing --resident-provider-stale-seconds value")?
                        .parse()?;
                    if resident_provider_stale_seconds == 0 {
                        anyhow::bail!("--resident-provider-stale-seconds must be positive");
                    }
                }
                other => anyhow::bail!("unknown argument {other:?}"),
            }
        }

        if !store_explicit && command.contains("smoke") {
            store = PathBuf::from(format!(
                ".epiphany-smoke/daemon-supervisor-{}/local-verse.ccmp",
                sanitize_id(&command)
            ));
        }

        if loop_interval_seconds < 0 {
            anyhow::bail!("--loop-interval-seconds must be non-negative");
        }
        if matches!(
            command.as_str(),
            "provider-health-identity-enroll" | "provider-health-identity-export"
        ) {
            if [
                idunn_rudp_health.is_some(),
                idunn_daemon.is_some(),
                idunn_health_contract.is_some(),
                idunn_deployment_request_id.is_some(),
            ]
            .into_iter()
            .any(|present| present)
            {
                anyhow::bail!("provider-health identity commands cannot publish health");
            }
            if idunn_provider_health_identity_store.is_none() {
                anyhow::bail!(
                    "provider-health identity command requires --idunn-provider-health-identity-store"
                );
            }
            if command == "provider-health-identity-enroll"
                && idunn_provider_health_public_anchor.is_some()
            {
                anyhow::bail!(
                    "provider-health enrollment cannot write a public anchor; use provider-health-identity-export"
                );
            }
        } else {
            validate_idunn_health_options(
                idunn_rudp_health.as_ref(),
                idunn_daemon.as_deref(),
                idunn_health_contract.as_deref(),
                idunn_deployment_request_id.as_deref(),
                idunn_provider_health_identity_store.as_deref(),
            )?;
            if idunn_provider_health_public_anchor.is_some() {
                anyhow::bail!(
                    "--idunn-provider-health-public-anchor belongs only to provider-health-identity-export"
                );
            }
        }
        Ok(Self {
            command,
            store,
            runtime_id,
            cwd,
            force,
            disabled,
            cooldown_seconds,
            service_id,
            service_command,
            service_args,
            loop_interval_seconds,
            max_iterations,
            receipt_id,
            stdout_artifact,
            stderr_artifact,
            runtime_store,
            expected_claim_id,
            provider_heartbeat_id,
            qdrant_url,
            ollama_base_url,
            ollama_model,
            fatal_log,
            release_id,
            release_witness_sha256,
            idunn_rudp_health,
            idunn_daemon,
            idunn_health_contract,
            idunn_deployment_request_id,
            idunn_provider_health_identity_store,
            idunn_provider_health_public_anchor,
            resident_self_store,
            resident_provider_stale_seconds,
        })
    }
}

fn validate_idunn_health_options(
    endpoint: Option<&SocketAddr>,
    daemon_id: Option<&str>,
    health_contract: Option<&str>,
    deployment_request_id: Option<&str>,
    identity_store: Option<&Path>,
) -> Result<()> {
    let fields = [
        endpoint.is_some(),
        daemon_id.is_some(),
        health_contract.is_some(),
        deployment_request_id.is_some(),
        identity_store.is_some(),
    ];
    if fields.into_iter().any(|present| present) && !fields.into_iter().all(|present| present) {
        anyhow::bail!(
            "--idunn-rudp-health, --idunn-daemon, --idunn-health-contract, --idunn-deployment-request-id, and --idunn-provider-health-identity-store are all-or-none"
        );
    }
    for (label, value) in [
        ("--idunn-daemon", daemon_id),
        ("--idunn-health-contract", health_contract),
        ("--idunn-deployment-request-id", deployment_request_id),
    ] {
        if value.is_some_and(|value| value.trim().is_empty()) {
            anyhow::bail!("{label} cannot be empty");
        }
    }
    Ok(())
}
