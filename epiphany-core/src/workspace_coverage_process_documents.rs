use crate::cultmesh_integration::validate_workspace_coverage_projector_managed_service_policy;
use crate::{
    EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID,
    EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID, EpiphanyCultMeshManagedServicePolicyEntry,
    HOST_IDENTITY_KEY, HOST_IDENTITY_TYPE, HostIdentitySignature, HostIdentitySigner,
    HostIncarnationIdentityEntry, ProcessInstanceIdentity, open_epiphany_cultmesh_node,
    verify_host_identity_signature,
};
use anyhow::{Context, Result, anyhow, bail};
use chrono::DateTime;
use cultcache_rs::{DatabaseEntry, SingleFileMessagePackBackingStore};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::Path;

pub const WORKSPACE_COVERAGE_PROCESS_LAUNCH_TYPE: &str =
    "epiphany.workspace_coverage.managed_process_launch";
pub const WORKSPACE_COVERAGE_PROCESS_LAUNCH_SCHEMA_VERSION: &str =
    "epiphany.workspace_coverage.managed_process_launch.v0";
pub const WORKSPACE_COVERAGE_PROCESS_LAUNCH_LATEST_KEY: &str =
    "epiphany-local/workspace-coverage/managed-process-launch/latest";
pub const WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_TYPE: &str =
    "epiphany.workspace_coverage.provider_heartbeat";
pub const WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_SCHEMA_VERSION: &str =
    "epiphany.workspace_coverage.provider_heartbeat.v0";
pub const WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_LATEST_KEY: &str =
    "epiphany-local/workspace-coverage/provider-heartbeat/latest/";

const HOST_LAUNCH_PURPOSE: &str = "epiphany.workspace-coverage.managed-process-launch.v0";
const PROVIDER_HEARTBEAT_DOMAIN: &[u8] =
    b"epiphany.workspace-coverage.provider-heartbeat.signature.v0\0";

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.workspace_coverage.managed_process_launch",
    schema = "WorkspaceCoverageManagedProcessLaunchEntry"
)]
pub struct WorkspaceCoverageManagedProcessLaunchEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub launch_id: String,
    #[cultcache(key = 2)]
    pub service_id: String,
    #[cultcache(key = 3)]
    pub provider_daemon_id: String,
    #[cultcache(key = 4)]
    pub runtime_id: String,
    #[cultcache(key = 5)]
    pub policy_id: String,
    #[cultcache(key = 6)]
    pub policy_envelope_digest: String,
    #[cultcache(key = 7)]
    pub command: String,
    #[cultcache(key = 8)]
    pub args: Vec<String>,
    #[cultcache(key = 9)]
    pub cwd: Option<String>,
    #[cultcache(key = 10)]
    pub launched_at_utc: String,
    #[cultcache(key = 11)]
    pub host_identity_id: String,
    #[cultcache(key = 12)]
    pub host_public_key: Vec<u8>,
    #[cultcache(key = 13)]
    pub host_assurance: String,
    #[cultcache(key = 14)]
    pub host_identity_record_digest: String,
    #[cultcache(key = 15)]
    pub boot_identity: String,
    #[cultcache(key = 16)]
    pub process_id: u32,
    #[cultcache(key = 17)]
    pub process_creation_token: u64,
    #[cultcache(key = 18)]
    pub process_created_at_rfc3339: Option<String>,
    #[cultcache(key = 19)]
    pub process_executable_path: String,
    #[cultcache(key = 20)]
    pub executable_sha256: String,
    #[cultcache(key = 21)]
    pub provider_incarnation_id: String,
    #[cultcache(key = 22)]
    pub provider_public_key: Vec<u8>,
    #[cultcache(key = 23)]
    pub host_signature: Vec<u8>,
    #[cultcache(key = 24)]
    pub supervisor_id: String,
    #[cultcache(key = 25)]
    pub identity_captured_at_utc: String,
    #[cultcache(key = 26)]
    pub signature_algorithm: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.workspace_coverage.provider_heartbeat",
    schema = "WorkspaceCoverageProviderHeartbeatEntry"
)]
pub struct WorkspaceCoverageProviderHeartbeatEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub heartbeat_id: String,
    #[cultcache(key = 2)]
    pub launch_id: String,
    #[cultcache(key = 3)]
    pub launch_envelope_digest: String,
    #[cultcache(key = 4)]
    pub service_id: String,
    #[cultcache(key = 5)]
    pub provider_daemon_id: String,
    #[cultcache(key = 6)]
    pub runtime_id: String,
    #[cultcache(key = 7)]
    pub host_identity_id: String,
    #[cultcache(key = 8)]
    pub host_identity_record_digest: String,
    #[cultcache(key = 9)]
    pub boot_identity: String,
    #[cultcache(key = 10)]
    pub process_id: u32,
    #[cultcache(key = 11)]
    pub process_creation_token: u64,
    #[cultcache(key = 12)]
    pub process_executable_path: String,
    #[cultcache(key = 13)]
    pub provider_incarnation_id: String,
    #[cultcache(key = 14)]
    pub provider_public_key: Vec<u8>,
    #[cultcache(key = 15)]
    pub sequence: u64,
    #[cultcache(key = 16)]
    pub status: String,
    #[cultcache(key = 17)]
    pub observed_at_utc: String,
    #[cultcache(key = 18)]
    pub provider_signature: Vec<u8>,
    #[cultcache(key = 19)]
    pub signature_algorithm: String,
}

#[derive(Serialize)]
struct LaunchStatement<'a> {
    schema_version: &'a str,
    launch_id: &'a str,
    service_id: &'a str,
    provider_daemon_id: &'a str,
    runtime_id: &'a str,
    policy_id: &'a str,
    policy_envelope_digest: &'a str,
    command: &'a str,
    args: &'a [String],
    cwd: &'a Option<String>,
    launched_at_utc: &'a str,
    host_identity_id: &'a str,
    host_public_key: &'a [u8],
    host_assurance: &'a str,
    host_identity_record_digest: &'a str,
    boot_identity: &'a str,
    process_id: u32,
    process_creation_token: u64,
    process_created_at_rfc3339: &'a Option<String>,
    process_executable_path: &'a str,
    executable_sha256: &'a str,
    provider_incarnation_id: &'a str,
    provider_public_key: &'a [u8],
    supervisor_id: &'a str,
    identity_captured_at_utc: &'a str,
    signature_algorithm: &'a str,
}

#[derive(Serialize)]
struct HeartbeatStatement<'a> {
    schema_version: &'a str,
    heartbeat_id: &'a str,
    launch_id: &'a str,
    launch_envelope_digest: &'a str,
    service_id: &'a str,
    provider_daemon_id: &'a str,
    runtime_id: &'a str,
    host_identity_id: &'a str,
    host_identity_record_digest: &'a str,
    boot_identity: &'a str,
    process_id: u32,
    process_creation_token: u64,
    process_executable_path: &'a str,
    provider_incarnation_id: &'a str,
    provider_public_key: &'a [u8],
    sequence: u64,
    status: &'a str,
    observed_at_utc: &'a str,
    signature_algorithm: &'a str,
}

pub fn workspace_coverage_host_identity_record_digest(
    entry: &HostIncarnationIdentityEntry,
) -> Result<String> {
    let payload = rmp_serde::to_vec(entry)?;
    let mut digest = Sha256::new();
    digest.update(HOST_IDENTITY_TYPE.as_bytes());
    digest.update([0]);
    digest.update(HOST_IDENTITY_KEY.as_bytes());
    digest.update([0]);
    digest.update(payload);
    Ok(format!("sha256-{:x}", digest.finalize()))
}

pub fn workspace_coverage_launch_statement(
    entry: &WorkspaceCoverageManagedProcessLaunchEntry,
) -> Result<Vec<u8>> {
    Ok(rmp_serde::to_vec_named(&LaunchStatement {
        schema_version: &entry.schema_version,
        launch_id: &entry.launch_id,
        service_id: &entry.service_id,
        provider_daemon_id: &entry.provider_daemon_id,
        runtime_id: &entry.runtime_id,
        policy_id: &entry.policy_id,
        policy_envelope_digest: &entry.policy_envelope_digest,
        command: &entry.command,
        args: &entry.args,
        cwd: &entry.cwd,
        launched_at_utc: &entry.launched_at_utc,
        host_identity_id: &entry.host_identity_id,
        host_public_key: &entry.host_public_key,
        host_assurance: &entry.host_assurance,
        host_identity_record_digest: &entry.host_identity_record_digest,
        boot_identity: &entry.boot_identity,
        process_id: entry.process_id,
        process_creation_token: entry.process_creation_token,
        process_created_at_rfc3339: &entry.process_created_at_rfc3339,
        process_executable_path: &entry.process_executable_path,
        executable_sha256: &entry.executable_sha256,
        provider_incarnation_id: &entry.provider_incarnation_id,
        provider_public_key: &entry.provider_public_key,
        supervisor_id: &entry.supervisor_id,
        identity_captured_at_utc: &entry.identity_captured_at_utc,
        signature_algorithm: &entry.signature_algorithm,
    })?)
}

pub fn sign_workspace_coverage_launch(
    entry: &mut WorkspaceCoverageManagedProcessLaunchEntry,
    signer: &HostIdentitySigner,
) -> Result<()> {
    entry.host_signature.clear();
    let proof = signer.sign(
        HOST_LAUNCH_PURPOSE,
        &workspace_coverage_launch_statement(entry)?,
    )?;
    entry.host_signature = proof.signature;
    Ok(())
}

pub fn workspace_coverage_heartbeat_statement(
    entry: &WorkspaceCoverageProviderHeartbeatEntry,
) -> Result<Vec<u8>> {
    Ok(rmp_serde::to_vec_named(&HeartbeatStatement {
        schema_version: &entry.schema_version,
        heartbeat_id: &entry.heartbeat_id,
        launch_id: &entry.launch_id,
        launch_envelope_digest: &entry.launch_envelope_digest,
        service_id: &entry.service_id,
        provider_daemon_id: &entry.provider_daemon_id,
        runtime_id: &entry.runtime_id,
        host_identity_id: &entry.host_identity_id,
        host_identity_record_digest: &entry.host_identity_record_digest,
        boot_identity: &entry.boot_identity,
        process_id: entry.process_id,
        process_creation_token: entry.process_creation_token,
        process_executable_path: &entry.process_executable_path,
        provider_incarnation_id: &entry.provider_incarnation_id,
        provider_public_key: &entry.provider_public_key,
        sequence: entry.sequence,
        status: &entry.status,
        observed_at_utc: &entry.observed_at_utc,
        signature_algorithm: &entry.signature_algorithm,
    })?)
}

pub fn sign_workspace_coverage_heartbeat(
    entry: &mut WorkspaceCoverageProviderHeartbeatEntry,
    signing_key: &SigningKey,
) -> Result<()> {
    if signing_key.verifying_key().to_bytes().as_slice() != entry.provider_public_key.as_slice() {
        bail!("provider signing key disagrees with heartbeat public key");
    }
    entry.provider_signature.clear();
    let statement = workspace_coverage_heartbeat_statement(entry)?;
    entry.provider_signature = signing_key
        .sign(&provider_message(&statement))
        .to_bytes()
        .to_vec();
    Ok(())
}

pub fn process_identity_from_workspace_coverage_launch(
    entry: &WorkspaceCoverageManagedProcessLaunchEntry,
) -> ProcessInstanceIdentity {
    ProcessInstanceIdentity {
        process_id: entry.process_id,
        creation_token: entry.process_creation_token,
        created_at_rfc3339: entry.process_created_at_rfc3339.clone(),
        executable_path: entry.process_executable_path.clone().into(),
    }
}

pub fn write_workspace_coverage_managed_process_launch(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    entry: WorkspaceCoverageManagedProcessLaunchEntry,
    host_identity: &HostIncarnationIdentityEntry,
) -> Result<WorkspaceCoverageManagedProcessLaunchEntry> {
    validate_launch(&entry, host_identity)?;
    let store_path = store_path.as_ref();
    let runtime_id = runtime_id.into();
    if runtime_id != entry.runtime_id {
        bail!("workspace coverage launch runtime argument disagrees with signed runtime id");
    }
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let policy_key = managed_policy_key();
    let policy_envelope = node
        .cache()
        .get_envelope::<EpiphanyCultMeshManagedServicePolicyEntry>(&policy_key)?
        .ok_or_else(|| anyhow!("workspace coverage managed policy is absent"))?;
    let policy: EpiphanyCultMeshManagedServicePolicyEntry =
        rmp_serde::from_slice(&policy_envelope.payload)?;
    validate_workspace_coverage_projector_managed_service_policy(&policy)?;
    if entry.policy_id != policy.policy_id
        || entry.policy_envelope_digest != envelope_digest(&policy_envelope)
        || entry.command != policy.command
        || entry.args != policy.args
        || entry.cwd != policy.cwd
    {
        bail!("workspace coverage launch disagrees with current managed policy");
    }
    let identity_key = launch_key(&entry.launch_id);
    if let Some(existing) = node.get::<WorkspaceCoverageManagedProcessLaunchEntry>(&identity_key)? {
        if existing == entry {
            return Ok(existing);
        }
        bail!("workspace coverage launch identity collision");
    }
    let mut expected = vec![policy_envelope.clone()];
    let mut replacements = vec![
        policy_envelope,
        node.cache().prepare_entry(&identity_key, &entry)?.0,
    ];
    if let Some(latest) = node
        .cache()
        .get_envelope::<WorkspaceCoverageManagedProcessLaunchEntry>(
            WORKSPACE_COVERAGE_PROCESS_LAUNCH_LATEST_KEY,
        )?
    {
        expected.push(latest);
    }
    replacements.push(
        node.cache()
            .prepare_entry(WORKSPACE_COVERAGE_PROCESS_LAUNCH_LATEST_KEY, &entry)?
            .0,
    );
    if !SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&expected, replacements)?
    {
        bail!("workspace coverage launch lost exact policy/latest compare-and-swap");
    }
    Ok(entry)
}

pub fn write_workspace_coverage_provider_heartbeat(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    entry: WorkspaceCoverageProviderHeartbeatEntry,
) -> Result<WorkspaceCoverageProviderHeartbeatEntry> {
    validate_heartbeat_shape(&entry)?;
    let store_path = store_path.as_ref();
    let runtime_id = runtime_id.into();
    if runtime_id != entry.runtime_id {
        bail!("workspace coverage heartbeat runtime argument disagrees with signed runtime id");
    }
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let launch_key = launch_key(&entry.launch_id);
    let launch_envelope = node
        .cache()
        .get_envelope::<WorkspaceCoverageManagedProcessLaunchEntry>(&launch_key)?
        .ok_or_else(|| anyhow!("workspace coverage heartbeat launch is absent"))?;
    let launch: WorkspaceCoverageManagedProcessLaunchEntry =
        rmp_serde::from_slice(&launch_envelope.payload)?;
    authenticate_heartbeat_against_launch(&entry, &launch, &envelope_digest(&launch_envelope))?;
    let identity_key = heartbeat_key(&entry.heartbeat_id);
    if let Some(existing) = node.get::<WorkspaceCoverageProviderHeartbeatEntry>(&identity_key)? {
        if existing == entry {
            return Ok(existing);
        }
        bail!("workspace coverage heartbeat identity collision");
    }
    let mut expected = vec![launch_envelope.clone()];
    let mut replacements = vec![
        launch_envelope,
        node.cache().prepare_entry(&identity_key, &entry)?.0,
    ];
    let latest_key = heartbeat_latest_key(&entry.launch_id);
    let latest = node
        .cache()
        .get_envelope::<WorkspaceCoverageProviderHeartbeatEntry>(&latest_key)?;
    match latest.as_ref() {
        Some(envelope) => {
            let prior: WorkspaceCoverageProviderHeartbeatEntry =
                rmp_serde::from_slice(&envelope.payload)?;
            if entry.sequence
                != prior
                    .sequence
                    .checked_add(1)
                    .ok_or_else(|| anyhow!("workspace coverage heartbeat sequence exhausted"))?
            {
                bail!(
                    "workspace coverage heartbeat sequence must exactly advance latest launch sequence"
                );
            }
            let prior_time = DateTime::parse_from_rfc3339(&prior.observed_at_utc)?;
            let current_time = DateTime::parse_from_rfc3339(&entry.observed_at_utc)?;
            if current_time <= prior_time {
                bail!("workspace coverage heartbeat time must strictly advance");
            }
            expected.push(envelope.clone());
        }
        None if entry.sequence != 1 => {
            bail!("first workspace coverage heartbeat must have sequence one")
        }
        None => {}
    }
    replacements.push(node.cache().prepare_entry(&latest_key, &entry)?.0);
    if !SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&expected, replacements)?
    {
        bail!("workspace coverage heartbeat lost exact launch/latest compare-and-swap");
    }
    Ok(entry)
}

pub fn load_workspace_coverage_managed_process_launch(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    launch_id: &str,
) -> Result<Option<WorkspaceCoverageManagedProcessLaunchEntry>> {
    require_nonempty("launch id", launch_id)?;
    open_epiphany_cultmesh_node(store_path, runtime_id)?.get(&launch_key(launch_id))
}

pub fn load_workspace_coverage_managed_process_launch_with_digest(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    launch_id: &str,
) -> Result<Option<(WorkspaceCoverageManagedProcessLaunchEntry, String)>> {
    require_nonempty("launch id", launch_id)?;
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let Some(envelope) = node
        .cache()
        .get_envelope::<WorkspaceCoverageManagedProcessLaunchEntry>(&launch_key(launch_id))?
    else {
        return Ok(None);
    };
    let digest = envelope_digest(&envelope);
    Ok(Some((rmp_serde::from_slice(&envelope.payload)?, digest)))
}

pub fn load_latest_workspace_coverage_managed_process_launch(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<WorkspaceCoverageManagedProcessLaunchEntry>> {
    open_epiphany_cultmesh_node(store_path, runtime_id)?
        .get(WORKSPACE_COVERAGE_PROCESS_LAUNCH_LATEST_KEY)
}

pub fn load_workspace_coverage_provider_heartbeat(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    heartbeat_id: &str,
) -> Result<Option<WorkspaceCoverageProviderHeartbeatEntry>> {
    require_nonempty("heartbeat id", heartbeat_id)?;
    open_epiphany_cultmesh_node(store_path, runtime_id)?.get(&heartbeat_key(heartbeat_id))
}

pub fn load_latest_workspace_coverage_provider_heartbeat(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    launch_id: &str,
) -> Result<Option<WorkspaceCoverageProviderHeartbeatEntry>> {
    require_nonempty("launch id", launch_id)?;
    open_epiphany_cultmesh_node(store_path, runtime_id)?.get(&heartbeat_latest_key(launch_id))
}

pub fn authenticate_workspace_coverage_managed_process_launch(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    launch_id: &str,
    host_identity: &HostIncarnationIdentityEntry,
) -> Result<WorkspaceCoverageManagedProcessLaunchEntry> {
    let runtime_id = runtime_id.into();
    let entry = load_workspace_coverage_managed_process_launch(
        store_path.as_ref(),
        runtime_id.clone(),
        launch_id,
    )?
    .ok_or_else(|| anyhow!("workspace coverage managed process launch is absent"))?;
    if runtime_id != entry.runtime_id {
        bail!("workspace coverage launch runtime argument disagrees with signed runtime id");
    }
    validate_launch(&entry, host_identity)?;
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let envelope = node
        .cache()
        .get_envelope::<EpiphanyCultMeshManagedServicePolicyEntry>(&managed_policy_key())?
        .ok_or_else(|| anyhow!("workspace coverage managed policy is absent"))?;
    let policy: EpiphanyCultMeshManagedServicePolicyEntry =
        rmp_serde::from_slice(&envelope.payload)?;
    validate_workspace_coverage_projector_managed_service_policy(&policy)?;
    if entry.policy_id != policy.policy_id
        || entry.policy_envelope_digest != envelope_digest(&envelope)
        || entry.command != policy.command
        || entry.args != policy.args
        || entry.cwd != policy.cwd
    {
        bail!("workspace coverage launch disagrees with current managed policy");
    }
    Ok(entry)
}

pub fn authenticate_workspace_coverage_provider_heartbeat(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    heartbeat_id: &str,
    host_identity: &HostIncarnationIdentityEntry,
) -> Result<WorkspaceCoverageProviderHeartbeatEntry> {
    let runtime_id = runtime_id.into();
    let heartbeat = load_workspace_coverage_provider_heartbeat(
        store_path.as_ref(),
        runtime_id.clone(),
        heartbeat_id,
    )?
    .ok_or_else(|| anyhow!("workspace coverage provider heartbeat is absent"))?;
    if runtime_id != heartbeat.runtime_id {
        bail!("workspace coverage heartbeat runtime argument disagrees with signed runtime id");
    }
    let launch = authenticate_workspace_coverage_managed_process_launch(
        store_path.as_ref(),
        runtime_id.clone(),
        &heartbeat.launch_id,
        host_identity,
    )?;
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let envelope = node
        .cache()
        .get_envelope::<WorkspaceCoverageManagedProcessLaunchEntry>(&launch_key(
            &heartbeat.launch_id,
        ))?
        .ok_or_else(|| anyhow!("workspace coverage launch envelope disappeared"))?;
    authenticate_heartbeat_against_launch(&heartbeat, &launch, &envelope_digest(&envelope))?;
    Ok(heartbeat)
}

fn validate_launch(
    entry: &WorkspaceCoverageManagedProcessLaunchEntry,
    host: &HostIncarnationIdentityEntry,
) -> Result<()> {
    if entry.schema_version != WORKSPACE_COVERAGE_PROCESS_LAUNCH_SCHEMA_VERSION
        || entry.service_id != EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID
        || entry.provider_daemon_id != EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID
        || entry.supervisor_id != "epiphany-daemon-supervisor"
    {
        bail!("workspace coverage launch violates its reserved schema or authority");
    }
    for (name, value) in [
        ("launch id", entry.launch_id.as_str()),
        ("runtime id", entry.runtime_id.as_str()),
        ("policy id", entry.policy_id.as_str()),
        ("policy digest", entry.policy_envelope_digest.as_str()),
        ("command", entry.command.as_str()),
        ("launch time", entry.launched_at_utc.as_str()),
        ("boot identity", entry.boot_identity.as_str()),
        (
            "process executable path",
            entry.process_executable_path.as_str(),
        ),
        ("executable digest", entry.executable_sha256.as_str()),
        (
            "provider incarnation id",
            entry.provider_incarnation_id.as_str(),
        ),
        ("supervisor id", entry.supervisor_id.as_str()),
    ] {
        require_nonempty(name, value)?;
    }
    DateTime::parse_from_rfc3339(&entry.launched_at_utc)
        .context("workspace coverage launch time must be RFC3339")?;
    let identity_captured_at = DateTime::parse_from_rfc3339(&entry.identity_captured_at_utc)
        .context("process identity capture time must be RFC3339")?;
    if identity_captured_at < DateTime::parse_from_rfc3339(&entry.launched_at_utc)? {
        bail!("process identity cannot be captured before launch");
    }
    if let Some(created) = &entry.process_created_at_rfc3339 {
        DateTime::parse_from_rfc3339(created).context("process creation time must be RFC3339")?;
    }
    uuid::Uuid::parse_str(&entry.launch_id).context("launch id must be UUID")?;
    uuid::Uuid::parse_str(&entry.provider_incarnation_id)
        .context("provider incarnation id must be UUID")?;
    validate_digest("policy", &entry.policy_envelope_digest)?;
    validate_digest("host identity record", &entry.host_identity_record_digest)?;
    validate_digest("executable", &entry.executable_sha256)?;
    validate_absolute_path(&entry.process_executable_path)?;
    if entry.signature_algorithm != "ed25519"
        || entry.process_id == 0
        || entry.process_creation_token == 0
        || entry.provider_public_key.len() != 32
        || entry.host_signature.len() != 64
    {
        bail!("workspace coverage launch has invalid process identity or signature material");
    }
    if entry.host_identity_id != host.identity_id
        || entry.host_public_key != host.public_key
        || entry.host_assurance != host.assurance
        || entry.host_identity_record_digest
            != workspace_coverage_host_identity_record_digest(host)?
    {
        bail!("workspace coverage launch disagrees with enrolled host public record");
    }
    verify_host_identity_signature(
        host,
        HOST_LAUNCH_PURPOSE,
        &workspace_coverage_launch_statement(entry)?,
        &HostIdentitySignature {
            identity_id: entry.host_identity_id.clone(),
            signature: entry.host_signature.clone(),
        },
    )?;
    Ok(())
}

fn validate_heartbeat_shape(entry: &WorkspaceCoverageProviderHeartbeatEntry) -> Result<()> {
    if entry.schema_version != WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_SCHEMA_VERSION
        || entry.service_id != EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID
        || entry.provider_daemon_id != EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID
    {
        bail!("workspace coverage heartbeat violates its reserved schema or authority");
    }
    for (name, value) in [
        ("heartbeat id", entry.heartbeat_id.as_str()),
        ("launch id", entry.launch_id.as_str()),
        ("launch digest", entry.launch_envelope_digest.as_str()),
        ("runtime id", entry.runtime_id.as_str()),
        ("host identity id", entry.host_identity_id.as_str()),
        (
            "host identity record digest",
            entry.host_identity_record_digest.as_str(),
        ),
        ("boot identity", entry.boot_identity.as_str()),
        (
            "process executable path",
            entry.process_executable_path.as_str(),
        ),
        (
            "provider incarnation id",
            entry.provider_incarnation_id.as_str(),
        ),
        ("status", entry.status.as_str()),
        ("observation time", entry.observed_at_utc.as_str()),
    ] {
        require_nonempty(name, value)?;
    }
    DateTime::parse_from_rfc3339(&entry.observed_at_utc)
        .context("workspace coverage heartbeat time must be RFC3339")?;
    uuid::Uuid::parse_str(&entry.heartbeat_id).context("heartbeat id must be UUID")?;
    uuid::Uuid::parse_str(&entry.launch_id).context("heartbeat launch id must be UUID")?;
    uuid::Uuid::parse_str(&entry.provider_incarnation_id)
        .context("heartbeat provider incarnation id must be UUID")?;
    validate_digest("launch", &entry.launch_envelope_digest)?;
    validate_digest("host identity record", &entry.host_identity_record_digest)?;
    validate_absolute_path(&entry.process_executable_path)?;
    if !matches!(entry.status.as_str(), "ready" | "degraded" | "stopping") {
        bail!("workspace coverage heartbeat status is not authoritative");
    }
    if entry.signature_algorithm != "ed25519"
        || entry.sequence == 0
        || entry.process_id == 0
        || entry.process_creation_token == 0
        || entry.provider_public_key.len() != 32
        || entry.provider_signature.len() != 64
    {
        bail!("workspace coverage heartbeat has invalid process identity or signature material");
    }
    Ok(())
}

fn authenticate_heartbeat_against_launch(
    heartbeat: &WorkspaceCoverageProviderHeartbeatEntry,
    launch: &WorkspaceCoverageManagedProcessLaunchEntry,
    launch_digest: &str,
) -> Result<()> {
    validate_heartbeat_shape(heartbeat)?;
    let observed = DateTime::parse_from_rfc3339(&heartbeat.observed_at_utc)?;
    let launched = DateTime::parse_from_rfc3339(&launch.launched_at_utc)?;
    let captured = DateTime::parse_from_rfc3339(&launch.identity_captured_at_utc)?;
    if observed < launched || observed < captured {
        bail!("workspace coverage heartbeat predates its launch identity");
    }
    if heartbeat.launch_envelope_digest != launch_digest
        || heartbeat.launch_id != launch.launch_id
        || heartbeat.service_id != launch.service_id
        || heartbeat.provider_daemon_id != launch.provider_daemon_id
        || heartbeat.runtime_id != launch.runtime_id
        || heartbeat.host_identity_id != launch.host_identity_id
        || heartbeat.host_identity_record_digest != launch.host_identity_record_digest
        || heartbeat.boot_identity != launch.boot_identity
        || heartbeat.process_id != launch.process_id
        || heartbeat.process_creation_token != launch.process_creation_token
        || heartbeat.process_executable_path != launch.process_executable_path
        || heartbeat.provider_incarnation_id != launch.provider_incarnation_id
        || heartbeat.provider_public_key != launch.provider_public_key
    {
        bail!("workspace coverage heartbeat disagrees with its exact launch identity");
    }
    let key: [u8; 32] = heartbeat
        .provider_public_key
        .as_slice()
        .try_into()
        .map_err(|_| anyhow!("provider public key has invalid length"))?;
    let sig = Signature::from_slice(&heartbeat.provider_signature)
        .map_err(|_| anyhow!("provider signature has invalid length"))?;
    VerifyingKey::from_bytes(&key)?
        .verify(
            &provider_message(&workspace_coverage_heartbeat_statement(heartbeat)?),
            &sig,
        )
        .map_err(|_| anyhow!("workspace coverage heartbeat provider signature verification failed"))
}

fn provider_message(statement: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(PROVIDER_HEARTBEAT_DOMAIN.len() + 8 + statement.len());
    out.extend_from_slice(PROVIDER_HEARTBEAT_DOMAIN);
    out.extend_from_slice(&(statement.len() as u64).to_be_bytes());
    out.extend_from_slice(statement);
    out
}
fn require_nonempty(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("workspace coverage {name} is required");
    }
    Ok(())
}
fn validate_digest(name: &str, value: &str) -> Result<()> {
    if value.len() != 71
        || !value.starts_with("sha256-")
        || !value[7..].bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        bail!("workspace coverage {name} digest must be sha256 plus 64 hex digits");
    }
    Ok(())
}
fn validate_absolute_path(value: &str) -> Result<()> {
    let path = Path::new(value);
    if !path.is_absolute() {
        bail!("workspace coverage process executable path must be absolute");
    }
    Ok(())
}
fn launch_key(id: &str) -> String {
    format!("epiphany-local/workspace-coverage/managed-process-launch/{id}")
}
fn heartbeat_key(id: &str) -> String {
    format!("epiphany-local/workspace-coverage/provider-heartbeat/{id}")
}
fn heartbeat_latest_key(launch_id: &str) -> String {
    format!("{WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_LATEST_KEY}{launch_id}")
}
fn managed_policy_key() -> String {
    format!(
        "epiphany-local/managed-service-policy/{}",
        EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID
    )
}
fn envelope_digest(envelope: &cultcache_rs::CultCacheEnvelope) -> String {
    let mut digest = Sha256::new();
    digest.update(envelope.r#type.as_bytes());
    digest.update([0]);
    digest.update(envelope.key.as_bytes());
    digest.update([0]);
    digest.update(&envelope.payload);
    format!("sha256-{:x}", digest.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION, enroll_host_identity_at};
    use rand_core::{OsRng, RngCore};
    use uuid::Uuid;

    fn provider_key() -> SigningKey {
        let mut seed = [0_u8; 32];
        OsRng.fill_bytes(&mut seed);
        SigningKey::from_bytes(&seed)
    }

    fn policy() -> Result<EpiphanyCultMeshManagedServicePolicyEntry> {
        let command = std::env::current_exe()?
            .with_file_name(if cfg!(windows) {
                "epiphany-workspace-coverage-projector.exe"
            } else {
                "epiphany-workspace-coverage-projector"
            })
            .display()
            .to_string();
        Ok(EpiphanyCultMeshManagedServicePolicyEntry {
            schema_version: EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION.to_string(),
            policy_id: "managed-service-policy-epiphany-workspace-coverage-projector-service"
                .to_string(),
            service_id: EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID.to_string(),
            owner_daemon_id: "epiphany-daemon-supervisor".to_string(),
            command,
            args: vec![
                "serve",
                "--runtime-store",
                "runtime.ccmp",
                "--local-verse-store",
                "verse.ccmp",
                "--runtime-id",
                "local",
                "--interval-seconds",
                "30",
                "--qdrant-url",
                "http://127.0.0.1:6333",
                "--ollama-base-url",
                "http://127.0.0.1:11434",
                "--ollama-model",
                "embedding:model",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            cwd: None,
            enabled: true,
            restart_mode: "always".to_string(),
            cooldown_seconds: 5,
            backoff_multiplier: 2,
            stdout_artifact: "stdout.log".to_string(),
            stderr_artifact: "stderr.log".to_string(),
            updated_at_utc: chrono::Utc::now().to_rfc3339(),
            private_state_exposed: false,
            notes: Vec::new(),
        })
    }

    fn launch(
        policy: &EpiphanyCultMeshManagedServicePolicyEntry,
        policy_digest: String,
        host: &HostIdentitySigner,
        provider: &SigningKey,
    ) -> Result<WorkspaceCoverageManagedProcessLaunchEntry> {
        let now = chrono::Utc::now().to_rfc3339();
        let mut entry = WorkspaceCoverageManagedProcessLaunchEntry {
            schema_version: WORKSPACE_COVERAGE_PROCESS_LAUNCH_SCHEMA_VERSION.to_string(),
            launch_id: Uuid::new_v4().to_string(),
            service_id: EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID.to_string(),
            provider_daemon_id: EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID.to_string(),
            runtime_id: "local".to_string(),
            policy_id: policy.policy_id.clone(),
            policy_envelope_digest: policy_digest,
            command: policy.command.clone(),
            args: policy.args.clone(),
            cwd: policy.cwd.clone(),
            launched_at_utc: now.clone(),
            host_identity_id: host.entry().identity_id.clone(),
            host_public_key: host.entry().public_key.clone(),
            host_assurance: host.entry().assurance.clone(),
            host_identity_record_digest: workspace_coverage_host_identity_record_digest(
                host.entry(),
            )?,
            boot_identity: "test-boot-incarnation".to_string(),
            process_id: std::process::id(),
            process_creation_token: 42,
            process_created_at_rfc3339: None,
            process_executable_path: std::fs::canonicalize(std::env::current_exe()?)?
                .display()
                .to_string(),
            executable_sha256: format!("sha256-{}", "1".repeat(64)),
            provider_incarnation_id: Uuid::new_v4().to_string(),
            provider_public_key: provider.verifying_key().to_bytes().to_vec(),
            host_signature: Vec::new(),
            supervisor_id: "epiphany-daemon-supervisor".to_string(),
            identity_captured_at_utc: now,
            signature_algorithm: "ed25519".to_string(),
        };
        sign_workspace_coverage_launch(&mut entry, host)?;
        Ok(entry)
    }

    fn heartbeat(
        launch: &WorkspaceCoverageManagedProcessLaunchEntry,
        launch_digest: String,
        provider: &SigningKey,
        sequence: u64,
    ) -> Result<WorkspaceCoverageProviderHeartbeatEntry> {
        let mut entry = WorkspaceCoverageProviderHeartbeatEntry {
            schema_version: WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_SCHEMA_VERSION.to_string(),
            heartbeat_id: Uuid::new_v4().to_string(),
            launch_id: launch.launch_id.clone(),
            launch_envelope_digest: launch_digest,
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
            status: "ready".to_string(),
            observed_at_utc: chrono::Utc::now().to_rfc3339(),
            provider_signature: Vec::new(),
            signature_algorithm: "ed25519".to_string(),
        };
        sign_workspace_coverage_heartbeat(&mut entry, provider)?;
        Ok(entry)
    }

    #[test]
    fn signed_launch_and_per_launch_heartbeat_chain_is_exact() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("verse.ccmp");
        let host_store = temp.path().join("host.ccmp");
        let host = enroll_host_identity_at(&host_store)?;
        let policy = policy()?;
        let mut node = open_epiphany_cultmesh_node(&store, "local")?;
        node.put(managed_policy_key(), &policy)?;
        let policy_envelope = node
            .cache()
            .get_envelope::<EpiphanyCultMeshManagedServicePolicyEntry>(&managed_policy_key())?
            .context("test policy envelope absent")?;
        let provider = provider_key();
        let launch = launch(&policy, envelope_digest(&policy_envelope), &host, &provider)?;
        write_workspace_coverage_managed_process_launch(
            &store,
            "local",
            launch.clone(),
            host.entry(),
        )?;
        let launch_envelope = open_epiphany_cultmesh_node(&store, "local")?
            .cache()
            .get_envelope::<WorkspaceCoverageManagedProcessLaunchEntry>(&launch_key(
                &launch.launch_id,
            ))?
            .context("test launch envelope absent")?;
        let first = heartbeat(&launch, envelope_digest(&launch_envelope), &provider, 1)?;
        write_workspace_coverage_provider_heartbeat(&store, "local", first.clone())?;
        assert_eq!(
            load_latest_workspace_coverage_provider_heartbeat(&store, "local", &launch.launch_id)?,
            Some(first.clone())
        );

        let mut forged = first.clone();
        forged.heartbeat_id = Uuid::new_v4().to_string();
        forged.status = "degraded".to_string();
        assert!(write_workspace_coverage_provider_heartbeat(&store, "local", forged).is_err());

        let gap = heartbeat(&launch, envelope_digest(&launch_envelope), &provider, 3)?;
        assert!(write_workspace_coverage_provider_heartbeat(&store, "local", gap).is_err());

        let mut collision = launch.clone();
        collision.executable_sha256 = format!("sha256-{}", "2".repeat(64));
        sign_workspace_coverage_launch(&mut collision, &host)?;
        assert!(
            write_workspace_coverage_managed_process_launch(
                &store,
                "local",
                collision,
                host.entry()
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn reserved_documents_refuse_wrong_runtime_tuple_and_status() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let host = enroll_host_identity_at(&temp.path().join("host.ccmp"))?;
        let policy = policy()?;
        let provider = provider_key();
        let mut launch = launch(
            &policy,
            format!("sha256-{}", "3".repeat(64)),
            &host,
            &provider,
        )?;
        assert!(validate_launch(&launch, host.entry()).is_ok());
        launch.supervisor_id = "some-other-writer".to_string();
        sign_workspace_coverage_launch(&mut launch, &host)?;
        assert!(validate_launch(&launch, host.entry()).is_err());

        let mut heartbeat = heartbeat(&launch, format!("sha256-{}", "4".repeat(64)), &provider, 1)?;
        heartbeat.status = "healthy-ish".to_string();
        sign_workspace_coverage_heartbeat(&mut heartbeat, &provider)?;
        assert!(validate_heartbeat_shape(&heartbeat).is_err());
        Ok(())
    }
}
