use crate::cultmesh_integration::validate_workspace_coverage_projector_managed_service_policy;
use crate::{
    EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID,
    EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID, EpiphanyCultMeshManagedServicePolicyEntry,
    HOST_IDENTITY_KEY, HOST_IDENTITY_TYPE, HostIdentitySignature, HostIdentitySigner,
    HostIncarnationIdentityEntry, ProcessInstanceIdentity, ProcessInstanceObservation,
    native_boot_identity, observe_process_instance, open_epiphany_cultmesh_node,
    verify_host_identity_signature,
};
use anyhow::{Context, Result, anyhow, bail};
use chrono::DateTime;
use cultcache_rs::{
    CacheBackingStore, CultCacheEnvelope, DatabaseEntry, SingleFileMessagePackBackingStore,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::Path;

pub const WORKSPACE_COVERAGE_PROCESS_LAUNCH_TYPE: &str =
    "epiphany.workspace_coverage.managed_process_launch";
pub const WORKSPACE_COVERAGE_PROCESS_LAUNCH_SCHEMA_VERSION: &str =
    "epiphany.workspace_coverage.managed_process_launch.v1";
pub const WORKSPACE_COVERAGE_PROCESS_LAUNCH_LATEST_KEY: &str =
    "epiphany-local/workspace-coverage/managed-process-launch/latest";
pub const WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_TYPE: &str =
    "epiphany.workspace_coverage.provider_heartbeat";
pub const WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_SCHEMA_VERSION: &str =
    "epiphany.workspace_coverage.provider_heartbeat.v0";
pub const WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_LATEST_KEY: &str =
    "epiphany-local/workspace-coverage/provider-heartbeat/latest/";
pub const WORKSPACE_COVERAGE_PROCESS_TERMINATION_TYPE: &str =
    "epiphany.workspace_coverage.process_termination_observation";
pub const WORKSPACE_COVERAGE_PROCESS_TERMINATION_SCHEMA_VERSION: &str =
    "epiphany.workspace_coverage.process_termination_observation.v1";
pub(crate) const WORKSPACE_COVERAGE_PROCESS_EVIDENCE_HEAD_SCHEMA_VERSION: &str =
    "epiphany.workspace_coverage.process_evidence_head.v0";

const HOST_LAUNCH_PURPOSE: &str = "epiphany.workspace-coverage.managed-process-launch.v0";
const PROVIDER_HEARTBEAT_DOMAIN: &[u8] =
    b"epiphany.workspace-coverage.provider-heartbeat.signature.v0\0";
const HOST_TERMINATION_PURPOSE: &str =
    "epiphany.workspace-coverage.process-termination-observation.v0";
const WORKSPACE_COVERAGE_TERMINATION_OBSERVER: &str = "epiphany-daemon-supervisor";
const COVERAGE_ADVANCEMENT_SIGHT_DOMAIN: &[u8] =
    b"epiphany.workspace-coverage.advancement-sight.signature.v0\0";
const COVERAGE_TERMINAL_SIGHT_DOMAIN: &[u8] =
    b"epiphany.workspace-coverage.terminal-sight.signature.v0\0";
const COVERAGE_CLAIM_SIGHT_DOMAIN: &[u8] =
    b"epiphany.workspace-coverage.claim-sight.signature.v0\0";
const HOST_RECOVERY_DIRECTIVE_PURPOSE: &str =
    "epiphany.workspace-coverage.host-recovery-directive.v0";

pub const WORKSPACE_COVERAGE_ADVANCEMENT_SIGHT_TYPE: &str =
    "epiphany.workspace_coverage.advancement_sight";
pub const WORKSPACE_COVERAGE_ADVANCEMENT_SIGHT_SCHEMA_VERSION: &str =
    "epiphany.workspace_coverage.advancement_sight.v0";
pub const WORKSPACE_COVERAGE_TERMINAL_SIGHT_TYPE: &str =
    "epiphany.workspace_coverage.terminal_sight";
pub const WORKSPACE_COVERAGE_TERMINAL_SIGHT_SCHEMA_VERSION: &str =
    "epiphany.workspace_coverage.terminal_sight.v0";
pub const WORKSPACE_COVERAGE_CLAIM_SIGHT_TYPE: &str = "epiphany.workspace_coverage.claim_sight";
pub const WORKSPACE_COVERAGE_CLAIM_SIGHT_SCHEMA_VERSION: &str =
    "epiphany.workspace_coverage.claim_sight.v0";
pub const WORKSPACE_COVERAGE_RECOVERY_DIRECTIVE_TYPE: &str =
    "epiphany.workspace_coverage.recovery_directive";
pub const WORKSPACE_COVERAGE_RECOVERY_DIRECTIVE_SCHEMA_VERSION: &str =
    "epiphany.workspace_coverage.recovery_directive.v0";

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.workspace_coverage.claim_sight",
    schema = "WorkspaceCoverageClaimSightEntry"
)]
pub struct WorkspaceCoverageClaimSightEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub workspace_id: String,
    #[cultcache(key = 3)]
    pub launch_id: String,
    #[cultcache(key = 4)]
    pub launch_envelope_digest: String,
    #[cultcache(key = 5)]
    pub provider_incarnation_id: String,
    #[cultcache(key = 6)]
    pub provider_public_key: Vec<u8>,
    #[cultcache(key = 7)]
    pub coverage_store_binding_id: String,
    #[cultcache(key = 8)]
    pub coverage_store_binding_envelope_digest: String,
    #[cultcache(key = 9)]
    pub coverage_store_file_identity: String,
    #[cultcache(key = 10)]
    pub runtime_coverage_route_envelope_digest: String,
    #[cultcache(key = 11)]
    pub body_binding_sha256: String,
    #[cultcache(key = 12)]
    pub body_observation_id: String,
    #[cultcache(key = 13)]
    pub body_generation: u64,
    #[cultcache(key = 14)]
    pub manifest_root_sha256: String,
    #[cultcache(key = 15)]
    pub claim_id: String,
    #[cultcache(key = 16)]
    pub claim_epoch: u64,
    #[cultcache(key = 17)]
    pub plan_id: String,
    #[cultcache(key = 18)]
    pub attempt_id: String,
    #[cultcache(key = 19)]
    pub attempt_envelope_digest: String,
    #[cultcache(key = 20)]
    pub recovery_receipt_id: Option<String>,
    #[cultcache(key = 21)]
    pub recovery_receipt_digest: Option<String>,
    #[cultcache(key = 22)]
    pub observed_at_utc: String,
    #[cultcache(key = 23)]
    pub provider_signature: Vec<u8>,
    #[cultcache(key = 24)]
    pub signature_algorithm: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.workspace_coverage.recovery_directive",
    schema = "WorkspaceCoverageRecoveryDirectiveEntry"
)]
pub struct WorkspaceCoverageRecoveryDirectiveEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub directive_id: String,
    #[cultcache(key = 2)]
    pub runtime_id: String,
    #[cultcache(key = 3)]
    pub old_claim_id: String,
    #[cultcache(key = 4)]
    pub old_launch_id: String,
    #[cultcache(key = 5)]
    pub replacement_launch_id: String,
    #[cultcache(key = 6)]
    pub replacement_ready_heartbeat_id: String,
    #[cultcache(key = 7)]
    pub termination_id: String,
    #[cultcache(key = 8)]
    pub claim_sight_envelope_digest: String,
    #[cultcache(key = 9)]
    pub old_claim_epoch: u64,
    #[cultcache(key = 10)]
    pub old_plan_id: String,
    #[cultcache(key = 11)]
    pub termination_envelope_digest: String,
    #[cultcache(key = 12)]
    pub replacement_launch_envelope_digest: String,
    #[cultcache(key = 13)]
    pub replacement_ready_heartbeat_envelope_digest: String,
    #[cultcache(key = 14)]
    pub workspace_id: String,
    #[cultcache(key = 15)]
    pub body_binding_sha256: String,
    #[cultcache(key = 16)]
    pub body_observation_id: String,
    #[cultcache(key = 17)]
    pub body_generation: u64,
    #[cultcache(key = 18)]
    pub manifest_root_sha256: String,
    #[cultcache(key = 19)]
    pub coverage_store_binding_id: String,
    #[cultcache(key = 20)]
    pub coverage_store_binding_envelope_digest: String,
    #[cultcache(key = 21)]
    pub coverage_store_file_identity: String,
    #[cultcache(key = 22)]
    pub runtime_coverage_route_envelope_digest: String,
    #[cultcache(key = 23)]
    pub issued_at_utc: String,
    #[cultcache(key = 24)]
    pub host_identity_id: String,
    #[cultcache(key = 25)]
    pub host_identity_record_digest: String,
    #[cultcache(key = 26)]
    pub host_signature: Vec<u8>,
    #[cultcache(key = 27)]
    pub signature_algorithm: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.workspace_coverage.advancement_sight",
    schema = "WorkspaceCoverageAdvancementSightEntry"
)]
pub struct WorkspaceCoverageAdvancementSightEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub workspace_id: String,
    #[cultcache(key = 3)]
    pub launch_id: String,
    #[cultcache(key = 4)]
    pub launch_envelope_digest: String,
    #[cultcache(key = 5)]
    pub provider_incarnation_id: String,
    #[cultcache(key = 6)]
    pub provider_public_key: Vec<u8>,
    #[cultcache(key = 7)]
    pub coverage_store_binding_id: String,
    #[cultcache(key = 8)]
    pub coverage_store_binding_envelope_digest: String,
    #[cultcache(key = 9)]
    pub coverage_store_file_identity: String,
    #[cultcache(key = 10)]
    pub runtime_coverage_route_envelope_digest: String,
    #[cultcache(key = 11)]
    pub body_binding_sha256: String,
    #[cultcache(key = 12)]
    pub body_observation_id: String,
    #[cultcache(key = 13)]
    pub body_generation: u64,
    #[cultcache(key = 14)]
    pub manifest_root_sha256: String,
    #[cultcache(key = 15)]
    pub claim_id: String,
    #[cultcache(key = 16)]
    pub claim_epoch: u64,
    #[cultcache(key = 17)]
    pub plan_id: String,
    #[cultcache(key = 18)]
    pub progress_id: String,
    #[cultcache(key = 19)]
    pub progress_envelope_digest: String,
    #[cultcache(key = 20)]
    pub checkpoint_id: String,
    #[cultcache(key = 21)]
    pub checkpoint_envelope_digest: String,
    #[cultcache(key = 22)]
    pub sequence: u64,
    #[cultcache(key = 23)]
    pub status: String,
    #[cultcache(key = 24)]
    pub completed_units: u64,
    #[cultcache(key = 25)]
    pub total_units: u64,
    #[cultcache(key = 26)]
    pub last_advanced_at_utc: String,
    #[cultcache(key = 27)]
    pub observed_at_utc: String,
    #[cultcache(key = 28)]
    pub provider_signature: Vec<u8>,
    #[cultcache(key = 29)]
    pub signature_algorithm: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.workspace_coverage.terminal_sight",
    schema = "WorkspaceCoverageTerminalSightEntry"
)]
pub struct WorkspaceCoverageTerminalSightEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub workspace_id: String,
    #[cultcache(key = 3)]
    pub launch_id: String,
    #[cultcache(key = 4)]
    pub launch_envelope_digest: String,
    #[cultcache(key = 5)]
    pub provider_incarnation_id: String,
    #[cultcache(key = 6)]
    pub provider_public_key: Vec<u8>,
    #[cultcache(key = 7)]
    pub coverage_store_binding_id: String,
    #[cultcache(key = 8)]
    pub coverage_store_binding_envelope_digest: String,
    #[cultcache(key = 9)]
    pub coverage_store_file_identity: String,
    #[cultcache(key = 10)]
    pub runtime_coverage_route_envelope_digest: String,
    #[cultcache(key = 11)]
    pub body_binding_sha256: String,
    #[cultcache(key = 12)]
    pub body_observation_id: String,
    #[cultcache(key = 13)]
    pub body_generation: u64,
    #[cultcache(key = 14)]
    pub manifest_root_sha256: String,
    #[cultcache(key = 15)]
    pub claim_id: String,
    #[cultcache(key = 16)]
    pub claim_epoch: u64,
    #[cultcache(key = 17)]
    pub plan_id: String,
    #[cultcache(key = 18)]
    pub receipt_id: String,
    #[cultcache(key = 19)]
    pub receipt_envelope_digest: String,
    #[cultcache(key = 20)]
    pub head_id: String,
    #[cultcache(key = 21)]
    pub head_envelope_digest: String,
    #[cultcache(key = 22)]
    pub sequence: u64,
    #[cultcache(key = 23)]
    pub status: String,
    #[cultcache(key = 24)]
    pub observed_at_utc: String,
    #[cultcache(key = 25)]
    pub provider_signature: Vec<u8>,
    #[cultcache(key = 26)]
    pub signature_algorithm: String,
}

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
    #[cultcache(key = 27, default)]
    pub replaces_launch_id: Option<String>,
    #[cultcache(key = 28, default)]
    pub replaces_termination_id: Option<String>,
    #[cultcache(key = 29, default)]
    pub replaces_termination_envelope_digest: Option<String>,
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

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.workspace_coverage.process_termination_observation",
    schema = "WorkspaceCoverageProcessTerminationObservationEntry"
)]
pub struct WorkspaceCoverageProcessTerminationObservationEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub termination_id: String,
    #[cultcache(key = 2)]
    pub launch_id: String,
    #[cultcache(key = 3)]
    pub launch_envelope_digest: String,
    #[cultcache(key = 4)]
    pub heartbeat_id: Option<String>,
    #[cultcache(key = 5, default)]
    pub heartbeat_envelope_digest: Option<String>,
    #[cultcache(key = 6)]
    pub policy_id: String,
    #[cultcache(key = 7)]
    pub policy_envelope_digest: String,
    #[cultcache(key = 8)]
    pub runtime_id: String,
    #[cultcache(key = 9)]
    pub host_identity_id: String,
    #[cultcache(key = 10)]
    pub host_identity_record_digest: String,
    #[cultcache(key = 11)]
    pub expected_boot_identity: String,
    #[cultcache(key = 12)]
    pub expected_process_id: u32,
    #[cultcache(key = 13)]
    pub expected_process_creation_token: u64,
    #[cultcache(key = 14)]
    pub expected_process_executable_path: String,
    #[cultcache(key = 15)]
    pub observed_boot_identity: String,
    #[cultcache(key = 16)]
    pub outcome: String,
    #[cultcache(key = 17)]
    pub exit_code: Option<u32>,
    #[cultcache(key = 18)]
    pub replacement_process_id: Option<u32>,
    #[cultcache(key = 19)]
    pub replacement_process_creation_token: Option<u64>,
    #[cultcache(key = 20)]
    pub replacement_process_created_at_rfc3339: Option<String>,
    #[cultcache(key = 21)]
    pub replacement_process_executable_path: Option<String>,
    #[cultcache(key = 22)]
    pub observed_at_utc: String,
    #[cultcache(key = 23)]
    pub observer_id: String,
    #[cultcache(key = 24)]
    pub host_signature: Vec<u8>,
    #[cultcache(key = 25)]
    pub signature_algorithm: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.workspace_coverage.process_evidence_head",
    schema = "WorkspaceCoverageProcessEvidenceHead"
)]
pub(crate) struct WorkspaceCoverageProcessEvidenceHead {
    #[cultcache(key = 0)]
    schema_version: String,
    #[cultcache(key = 1)]
    launch_id: String,
    #[cultcache(key = 2)]
    generation: u64,
    #[cultcache(key = 3)]
    state: String,
    #[cultcache(key = 4, default)]
    heartbeat_id: Option<String>,
    #[cultcache(key = 5, default)]
    termination_id: Option<String>,
}

trait WorkspaceCoverageProcessObservationSource {
    fn boot_identity(&self) -> Option<String>;
    fn observe(&self, expected: &ProcessInstanceIdentity) -> ProcessInstanceObservation;
}

struct NativeWorkspaceCoverageProcessObservationSource;
impl WorkspaceCoverageProcessObservationSource for NativeWorkspaceCoverageProcessObservationSource {
    fn boot_identity(&self) -> Option<String> {
        native_boot_identity()
    }
    fn observe(&self, expected: &ProcessInstanceIdentity) -> ProcessInstanceObservation {
        observe_process_instance(expected)
    }
}

/// A typed, non-mutating observation of the exact process incarnation named by
/// a workspace-coverage launch.  Reconciliation must branch on this value;
/// errors and uncertain observations are never evidence of termination.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkspaceCoverageProcessLifecycleObservation {
    ExactAlive,
    BootSuperseded { observed_boot_identity: String },
    ExactExited { exit_code: Option<u32> },
    Missing,
    Replaced { observed: ProcessInstanceIdentity },
    Inaccessible,
    Indeterminate { reason: String },
}

fn observe_workspace_coverage_process_with_source(
    launch: &WorkspaceCoverageManagedProcessLaunchEntry,
    source: &dyn WorkspaceCoverageProcessObservationSource,
) -> WorkspaceCoverageProcessLifecycleObservation {
    let Some(observed_boot_identity) = source.boot_identity() else {
        return WorkspaceCoverageProcessLifecycleObservation::Indeterminate {
            reason: "current boot identity is unavailable".to_string(),
        };
    };
    if observed_boot_identity != launch.boot_identity {
        return WorkspaceCoverageProcessLifecycleObservation::BootSuperseded {
            observed_boot_identity,
        };
    }
    match source.observe(&process_identity_from_workspace_coverage_launch(launch)) {
        ProcessInstanceObservation::ExactAlive => {
            WorkspaceCoverageProcessLifecycleObservation::ExactAlive
        }
        ProcessInstanceObservation::ExactExited { exit_code } => {
            WorkspaceCoverageProcessLifecycleObservation::ExactExited { exit_code }
        }
        ProcessInstanceObservation::Missing => {
            WorkspaceCoverageProcessLifecycleObservation::Missing
        }
        ProcessInstanceObservation::Replaced { observed } => {
            WorkspaceCoverageProcessLifecycleObservation::Replaced { observed }
        }
        ProcessInstanceObservation::Inaccessible => {
            WorkspaceCoverageProcessLifecycleObservation::Inaccessible
        }
        ProcessInstanceObservation::Indeterminate { reason } => {
            WorkspaceCoverageProcessLifecycleObservation::Indeterminate { reason }
        }
    }
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
    replaces_launch_id: &'a Option<String>,
    replaces_termination_id: &'a Option<String>,
    replaces_termination_envelope_digest: &'a Option<String>,
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

#[derive(Serialize)]
struct TerminationStatement<'a> {
    schema_version: &'a str,
    termination_id: &'a str,
    launch_id: &'a str,
    launch_envelope_digest: &'a str,
    heartbeat_id: &'a Option<String>,
    heartbeat_envelope_digest: &'a Option<String>,
    policy_id: &'a str,
    policy_envelope_digest: &'a str,
    runtime_id: &'a str,
    host_identity_id: &'a str,
    host_identity_record_digest: &'a str,
    expected_boot_identity: &'a str,
    expected_process_id: u32,
    expected_process_creation_token: u64,
    expected_process_executable_path: &'a str,
    observed_boot_identity: &'a str,
    outcome: &'a str,
    exit_code: Option<u32>,
    replacement_process_id: Option<u32>,
    replacement_process_creation_token: Option<u64>,
    replacement_process_created_at_rfc3339: &'a Option<String>,
    replacement_process_executable_path: &'a Option<String>,
    observed_at_utc: &'a str,
    observer_id: &'a str,
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
        replaces_launch_id: &entry.replaces_launch_id,
        replaces_termination_id: &entry.replaces_termination_id,
        replaces_termination_envelope_digest: &entry.replaces_termination_envelope_digest,
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

pub fn workspace_coverage_termination_statement(
    entry: &WorkspaceCoverageProcessTerminationObservationEntry,
) -> Result<Vec<u8>> {
    Ok(rmp_serde::to_vec_named(&TerminationStatement {
        schema_version: &entry.schema_version,
        termination_id: &entry.termination_id,
        launch_id: &entry.launch_id,
        launch_envelope_digest: &entry.launch_envelope_digest,
        heartbeat_id: &entry.heartbeat_id,
        heartbeat_envelope_digest: &entry.heartbeat_envelope_digest,
        policy_id: &entry.policy_id,
        policy_envelope_digest: &entry.policy_envelope_digest,
        runtime_id: &entry.runtime_id,
        host_identity_id: &entry.host_identity_id,
        host_identity_record_digest: &entry.host_identity_record_digest,
        expected_boot_identity: &entry.expected_boot_identity,
        expected_process_id: entry.expected_process_id,
        expected_process_creation_token: entry.expected_process_creation_token,
        expected_process_executable_path: &entry.expected_process_executable_path,
        observed_boot_identity: &entry.observed_boot_identity,
        outcome: &entry.outcome,
        exit_code: entry.exit_code,
        replacement_process_id: entry.replacement_process_id,
        replacement_process_creation_token: entry.replacement_process_creation_token,
        replacement_process_created_at_rfc3339: &entry.replacement_process_created_at_rfc3339,
        replacement_process_executable_path: &entry.replacement_process_executable_path,
        observed_at_utc: &entry.observed_at_utc,
        observer_id: &entry.observer_id,
        signature_algorithm: &entry.signature_algorithm,
    })?)
}

pub fn sign_workspace_coverage_termination(
    entry: &mut WorkspaceCoverageProcessTerminationObservationEntry,
    signer: &HostIdentitySigner,
) -> Result<()> {
    entry.host_signature.clear();
    let proof = signer.sign(
        HOST_TERMINATION_PURPOSE,
        &workspace_coverage_termination_statement(entry)?,
    )?;
    entry.host_signature = proof.signature;
    Ok(())
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
    let evidence_head_key = process_evidence_head_key(&entry.launch_id);
    if node
        .cache()
        .get_envelope::<WorkspaceCoverageProcessEvidenceHead>(&evidence_head_key)?
        .is_some()
    {
        bail!("workspace coverage launch evidence head identity collision");
    }
    let evidence_head = WorkspaceCoverageProcessEvidenceHead {
        schema_version: WORKSPACE_COVERAGE_PROCESS_EVIDENCE_HEAD_SCHEMA_VERSION.into(),
        launch_id: entry.launch_id.clone(),
        generation: 1,
        state: "launched".into(),
        heartbeat_id: None,
        termination_id: None,
    };
    let mut expected = vec![policy_envelope.clone()];
    let mut replacements = vec![
        policy_envelope,
        node.cache().prepare_entry(&identity_key, &entry)?.0,
        node.cache()
            .prepare_entry(&evidence_head_key, &evidence_head)?
            .0,
    ];
    if let (Some(old_launch_id), Some(termination_id), Some(termination_digest)) = (
        entry.replaces_launch_id.as_deref(),
        entry.replaces_termination_id.as_deref(),
        entry.replaces_termination_envelope_digest.as_deref(),
    ) {
        let termination_envelope = node
            .cache()
            .get_envelope::<WorkspaceCoverageProcessTerminationObservationEntry>(&termination_key(
                old_launch_id,
            ))?
            .ok_or_else(|| anyhow!("workspace coverage replacement termination is absent"))?;
        let termination: WorkspaceCoverageProcessTerminationObservationEntry =
            rmp_serde::from_slice(&termination_envelope.payload)?;
        authenticate_workspace_coverage_process_termination_observation(
            store_path,
            entry.runtime_id.clone(),
            old_launch_id,
            host_identity,
        )?;
        if termination.termination_id != termination_id
            || envelope_digest(&termination_envelope) != termination_digest
        {
            bail!("workspace coverage replacement launch disagrees with exact termination");
        }
        let replacement_key =
            format!("epiphany-local/workspace-coverage/replacement-for/{old_launch_id}");
        if node
            .cache()
            .get_envelope::<WorkspaceCoverageManagedProcessLaunchEntry>(&replacement_key)?
            .is_some()
        {
            bail!("workspace coverage termination already has a replacement launch");
        }
        expected.push(termination_envelope.clone());
        replacements.push(termination_envelope);
        replacements.push(node.cache().prepare_entry(&replacement_key, &entry)?.0);
    }
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
    let latest_key = heartbeat_latest_key(&entry.launch_id);
    let evidence_key = heartbeat_evidence_key(&entry.launch_id, &entry.heartbeat_id);
    let latest = node
        .cache()
        .get_envelope::<WorkspaceCoverageProviderHeartbeatEntry>(&latest_key)?;
    if let Some(existing) = latest.as_ref() {
        let existing: WorkspaceCoverageProviderHeartbeatEntry =
            rmp_serde::from_slice(&existing.payload)?;
        if existing == entry {
            return Ok(existing);
        }
        if existing.heartbeat_id == entry.heartbeat_id {
            bail!("workspace coverage heartbeat identity collision");
        }
    }
    let evidence_head_key = process_evidence_head_key(&entry.launch_id);
    let evidence_head_envelope = node
        .cache()
        .get_envelope::<WorkspaceCoverageProcessEvidenceHead>(&evidence_head_key)?
        .ok_or_else(|| anyhow!("workspace coverage heartbeat process evidence head is absent"))?;
    let evidence_head: WorkspaceCoverageProcessEvidenceHead =
        rmp_serde::from_slice(&evidence_head_envelope.payload)?;
    validate_process_evidence_head(&evidence_head, &entry.launch_id)?;
    if evidence_head.state == "terminated" {
        bail!("workspace coverage heartbeat cannot advance a terminated process");
    }
    let next_evidence_head = WorkspaceCoverageProcessEvidenceHead {
        schema_version: WORKSPACE_COVERAGE_PROCESS_EVIDENCE_HEAD_SCHEMA_VERSION.into(),
        launch_id: entry.launch_id.clone(),
        generation: evidence_head
            .generation
            .checked_add(1)
            .ok_or_else(|| anyhow!("workspace coverage process evidence generation exhausted"))?,
        state: "heartbeat".into(),
        heartbeat_id: Some(entry.heartbeat_id.clone()),
        termination_id: None,
    };
    let mut expected = vec![launch_envelope.clone(), evidence_head_envelope.clone()];
    let mut replacements = vec![
        launch_envelope,
        node.cache()
            .prepare_entry(&evidence_head_key, &next_evidence_head)?
            .0,
    ];
    if node
        .cache()
        .get_envelope::<WorkspaceCoverageProviderHeartbeatEntry>(&evidence_key)?
        .is_some()
    {
        bail!("workspace coverage heartbeat immutable evidence identity collision");
    }
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
    replacements.push(node.cache().prepare_entry(&evidence_key, &entry)?.0);
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
    let runtime_id = runtime_id.into();
    let Some(envelope) = latest_heartbeat_envelope_by_id(store_path, &runtime_id, heartbeat_id)?
    else {
        return Ok(None);
    };
    Ok(Some(rmp_serde::from_slice(&envelope.payload)?))
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

pub(crate) fn authenticate_workspace_coverage_managed_process_launch_with_envelope_digest(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    launch_id: &str,
    host_identity: &HostIncarnationIdentityEntry,
) -> Result<(WorkspaceCoverageManagedProcessLaunchEntry, String)> {
    let runtime_id = runtime_id.into();
    let launch = authenticate_workspace_coverage_managed_process_launch(
        store_path.as_ref(),
        runtime_id.clone(),
        launch_id,
        host_identity,
    )?;
    let envelope = open_epiphany_cultmesh_node(store_path, runtime_id)?
        .cache()
        .get_envelope::<WorkspaceCoverageManagedProcessLaunchEntry>(&launch_key(launch_id))?
        .ok_or_else(|| anyhow!("workspace coverage launch envelope disappeared"))?;
    Ok((launch, envelope_digest(&envelope)))
}

pub fn observe_workspace_coverage_managed_process(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    launch_id: &str,
    host_identity: &HostIncarnationIdentityEntry,
) -> Result<WorkspaceCoverageProcessLifecycleObservation> {
    let store_path = store_path.as_ref();
    let runtime_id = runtime_id.into();
    let launch = authenticate_workspace_coverage_managed_process_launch(
        store_path,
        runtime_id.clone(),
        launch_id,
        host_identity,
    )?;
    let policy_envelope = open_epiphany_cultmesh_node(store_path, runtime_id)?
        .cache()
        .get_envelope::<EpiphanyCultMeshManagedServicePolicyEntry>(&managed_policy_key())?
        .ok_or_else(|| anyhow!("workspace coverage managed policy is absent"))?;
    let policy: EpiphanyCultMeshManagedServicePolicyEntry =
        rmp_serde::from_slice(&policy_envelope.payload)?;
    validate_workspace_coverage_projector_managed_service_policy(&policy)?;
    if launch.policy_id != policy.policy_id
        || launch.policy_envelope_digest != envelope_digest(&policy_envelope)
        || launch.command != policy.command
        || launch.args != policy.args
        || launch.cwd != policy.cwd
    {
        bail!("workspace coverage observed launch disagrees with current managed policy");
    }
    Ok(observe_workspace_coverage_process_with_source(
        &launch,
        &NativeWorkspaceCoverageProcessObservationSource,
    ))
}

pub(crate) fn authenticate_workspace_coverage_provider_heartbeat_with_envelope_digest(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    heartbeat_id: &str,
    host_identity: &HostIncarnationIdentityEntry,
) -> Result<(WorkspaceCoverageProviderHeartbeatEntry, String)> {
    let runtime_id = runtime_id.into();
    let heartbeat = authenticate_workspace_coverage_provider_heartbeat(
        store_path.as_ref(),
        runtime_id.clone(),
        heartbeat_id,
        host_identity,
    )?;
    let envelope = latest_heartbeat_envelope_by_id(store_path, &runtime_id, heartbeat_id)?
        .ok_or_else(|| anyhow!("workspace coverage heartbeat envelope disappeared"))?;
    Ok((heartbeat, envelope_digest(&envelope)))
}

pub fn write_workspace_coverage_process_termination_observation(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    launch_id: &str,
    host: &HostIdentitySigner,
) -> Result<WorkspaceCoverageProcessTerminationObservationEntry> {
    write_workspace_coverage_process_termination_observation_with_source(
        store_path,
        runtime_id,
        launch_id,
        host,
        &NativeWorkspaceCoverageProcessObservationSource,
    )
}

fn write_workspace_coverage_process_termination_observation_with_source(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    launch_id: &str,
    host: &HostIdentitySigner,
    source: &dyn WorkspaceCoverageProcessObservationSource,
) -> Result<WorkspaceCoverageProcessTerminationObservationEntry> {
    require_nonempty("launch id", launch_id)?;
    let store_path = store_path.as_ref();
    let runtime_id = runtime_id.into();
    let node = open_epiphany_cultmesh_node(store_path, runtime_id.clone())?;

    let policy_envelope = node
        .cache()
        .get_envelope::<EpiphanyCultMeshManagedServicePolicyEntry>(&managed_policy_key())?
        .ok_or_else(|| anyhow!("workspace coverage managed policy is absent"))?;
    let policy: EpiphanyCultMeshManagedServicePolicyEntry =
        rmp_serde::from_slice(&policy_envelope.payload)?;
    validate_workspace_coverage_projector_managed_service_policy(&policy)?;

    let launch_envelope = node
        .cache()
        .get_envelope::<WorkspaceCoverageManagedProcessLaunchEntry>(&launch_key(launch_id))?
        .ok_or_else(|| anyhow!("workspace coverage managed process launch is absent"))?;
    let launch: WorkspaceCoverageManagedProcessLaunchEntry =
        rmp_serde::from_slice(&launch_envelope.payload)?;
    validate_launch(&launch, host.entry())?;
    if launch.runtime_id != runtime_id
        || launch.policy_id != policy.policy_id
        || launch.policy_envelope_digest != envelope_digest(&policy_envelope)
        || launch.command != policy.command
        || launch.args != policy.args
        || launch.cwd != policy.cwd
    {
        bail!("workspace coverage termination launch disagrees with current managed policy");
    }

    let evidence_head_key = process_evidence_head_key(launch_id);
    let evidence_head_envelope = node
        .cache()
        .get_envelope::<WorkspaceCoverageProcessEvidenceHead>(&evidence_head_key)?
        .ok_or_else(|| anyhow!("workspace coverage termination process evidence head is absent"))?;
    let evidence_head: WorkspaceCoverageProcessEvidenceHead =
        rmp_serde::from_slice(&evidence_head_envelope.payload)?;
    validate_process_evidence_head(&evidence_head, launch_id)?;
    if evidence_head.state == "terminated" {
        bail!("workspace coverage process already has terminal evidence");
    }
    let heartbeat_evidence = if let Some(heartbeat_id) = evidence_head.heartbeat_id.as_deref() {
        let envelope = node
            .cache()
            .get_envelope::<WorkspaceCoverageProviderHeartbeatEntry>(&heartbeat_latest_key(
                launch_id,
            ))?
            .ok_or_else(|| anyhow!("workspace coverage evidence head heartbeat is absent"))?;
        let heartbeat: WorkspaceCoverageProviderHeartbeatEntry =
            rmp_serde::from_slice(&envelope.payload)?;
        authenticate_heartbeat_against_launch(
            &heartbeat,
            &launch,
            &envelope_digest(&launch_envelope),
        )?;
        if heartbeat.heartbeat_id != heartbeat_id {
            bail!("workspace coverage evidence head disagrees with latest heartbeat");
        }
        Some((envelope, heartbeat))
    } else {
        None
    };

    let observation = observe_workspace_coverage_process_with_source(&launch, source);
    let (observed_boot_identity, outcome, exit_code, replacement) = match observation {
        WorkspaceCoverageProcessLifecycleObservation::BootSuperseded {
            observed_boot_identity,
        } => (observed_boot_identity, "boot_superseded", None, None),
        WorkspaceCoverageProcessLifecycleObservation::ExactExited { exit_code } => (
            launch.boot_identity.clone(),
            "exact_exited",
            exit_code,
            None,
        ),
        WorkspaceCoverageProcessLifecycleObservation::Missing => {
            (launch.boot_identity.clone(), "process_missing", None, None)
        }
        WorkspaceCoverageProcessLifecycleObservation::Replaced { observed } => (
            launch.boot_identity.clone(),
            "process_replaced",
            None,
            Some(observed),
        ),
        WorkspaceCoverageProcessLifecycleObservation::ExactAlive => {
            bail!("exact workspace coverage process instance is still alive")
        }
        WorkspaceCoverageProcessLifecycleObservation::Inaccessible => {
            bail!("workspace coverage process observation is inaccessible")
        }
        WorkspaceCoverageProcessLifecycleObservation::Indeterminate { reason } => {
            bail!("workspace coverage process termination is indeterminate: {reason}")
        }
    };
    let observed_at_utc = chrono::Utc::now().to_rfc3339();
    let mut entry = WorkspaceCoverageProcessTerminationObservationEntry {
        schema_version: WORKSPACE_COVERAGE_PROCESS_TERMINATION_SCHEMA_VERSION.to_string(),
        termination_id: launch.launch_id.clone(),
        launch_id: launch.launch_id.clone(),
        launch_envelope_digest: envelope_digest(&launch_envelope),
        heartbeat_id: heartbeat_evidence
            .as_ref()
            .map(|(_, heartbeat)| heartbeat.heartbeat_id.clone()),
        heartbeat_envelope_digest: heartbeat_evidence
            .as_ref()
            .map(|(envelope, _)| envelope_digest(envelope)),
        policy_id: policy.policy_id.clone(),
        policy_envelope_digest: envelope_digest(&policy_envelope),
        runtime_id,
        host_identity_id: host.entry().identity_id.clone(),
        host_identity_record_digest: workspace_coverage_host_identity_record_digest(host.entry())?,
        expected_boot_identity: launch.boot_identity.clone(),
        expected_process_id: launch.process_id,
        expected_process_creation_token: launch.process_creation_token,
        expected_process_executable_path: launch.process_executable_path.clone(),
        observed_boot_identity,
        outcome: outcome.to_string(),
        exit_code,
        replacement_process_id: replacement.as_ref().map(|value| value.process_id),
        replacement_process_creation_token: replacement.as_ref().map(|value| value.creation_token),
        replacement_process_created_at_rfc3339: replacement
            .as_ref()
            .and_then(|value| value.created_at_rfc3339.clone()),
        replacement_process_executable_path: replacement
            .as_ref()
            .map(|value| value.executable_path.display().to_string()),
        observed_at_utc,
        observer_id: WORKSPACE_COVERAGE_TERMINATION_OBSERVER.to_string(),
        host_signature: Vec::new(),
        signature_algorithm: "ed25519".to_string(),
    };
    sign_workspace_coverage_termination(&mut entry, host)?;
    validate_termination(&entry, host.entry())?;

    let key = termination_key(launch_id);
    let terminal_head = WorkspaceCoverageProcessEvidenceHead {
        schema_version: WORKSPACE_COVERAGE_PROCESS_EVIDENCE_HEAD_SCHEMA_VERSION.into(),
        launch_id: launch_id.into(),
        generation: evidence_head
            .generation
            .checked_add(1)
            .ok_or_else(|| anyhow!("workspace coverage process evidence generation exhausted"))?,
        state: "terminated".into(),
        heartbeat_id: evidence_head.heartbeat_id.clone(),
        termination_id: Some(entry.termination_id.clone()),
    };
    let replacement = node.cache().prepare_entry(&key, &entry)?.0;
    let mut expected = vec![
        policy_envelope.clone(),
        launch_envelope.clone(),
        evidence_head_envelope,
    ];
    let mut replacements = vec![
        policy_envelope,
        launch_envelope,
        node.cache()
            .prepare_entry(&evidence_head_key, &terminal_head)?
            .0,
        replacement,
    ];
    if let Some((heartbeat_envelope, _)) = heartbeat_evidence {
        expected.push(heartbeat_envelope.clone());
        replacements.push(heartbeat_envelope);
    }
    if !SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&expected, replacements)?
    {
        bail!("workspace coverage termination lost exact policy/launch/heartbeat CAS or collided");
    }
    Ok(entry)
}

pub fn load_workspace_coverage_process_termination_observation(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    launch_id: &str,
) -> Result<Option<WorkspaceCoverageProcessTerminationObservationEntry>> {
    require_nonempty("launch id", launch_id)?;
    open_epiphany_cultmesh_node(store_path, runtime_id)?.get(&termination_key(launch_id))
}

pub fn authenticate_workspace_coverage_process_termination_observation(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    launch_id: &str,
    host: &HostIncarnationIdentityEntry,
) -> Result<WorkspaceCoverageProcessTerminationObservationEntry> {
    let store_path = store_path.as_ref();
    let runtime_id = runtime_id.into();
    let entry = load_workspace_coverage_process_termination_observation(
        store_path,
        runtime_id.clone(),
        launch_id,
    )?
    .ok_or_else(|| anyhow!("workspace coverage process termination observation is absent"))?;
    validate_termination(&entry, host)?;
    if entry.runtime_id != runtime_id || entry.launch_id != launch_id {
        bail!("workspace coverage termination request disagrees with signed identity");
    }
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let policy_envelope = node
        .cache()
        .get_envelope::<EpiphanyCultMeshManagedServicePolicyEntry>(&managed_policy_key())?
        .ok_or_else(|| anyhow!("workspace coverage termination policy evidence is absent"))?;
    let policy: EpiphanyCultMeshManagedServicePolicyEntry =
        rmp_serde::from_slice(&policy_envelope.payload)?;
    validate_workspace_coverage_projector_managed_service_policy(&policy)?;
    let launch_envelope = node
        .cache()
        .get_envelope::<WorkspaceCoverageManagedProcessLaunchEntry>(&launch_key(launch_id))?
        .ok_or_else(|| anyhow!("workspace coverage termination launch evidence is absent"))?;
    let launch: WorkspaceCoverageManagedProcessLaunchEntry =
        rmp_serde::from_slice(&launch_envelope.payload)?;
    validate_launch(&launch, host)?;
    let evidence_head: WorkspaceCoverageProcessEvidenceHead = node
        .get(&process_evidence_head_key(launch_id))?
        .ok_or_else(|| anyhow!("workspace coverage termination process evidence head is absent"))?;
    validate_process_evidence_head(&evidence_head, launch_id)?;
    if evidence_head.state != "terminated"
        || evidence_head.termination_id.as_deref() != Some(entry.termination_id.as_str())
        || evidence_head.heartbeat_id != entry.heartbeat_id
    {
        bail!("workspace coverage termination disagrees with process evidence head");
    }
    let heartbeat_evidence = match (
        entry.heartbeat_id.as_deref(),
        entry.heartbeat_envelope_digest.as_deref(),
    ) {
        (Some(heartbeat_id), Some(expected_digest)) => {
            let envelope = node
                .cache()
                .get_envelope::<WorkspaceCoverageProviderHeartbeatEntry>(&heartbeat_latest_key(
                    launch_id,
                ))?
                .ok_or_else(|| {
                    anyhow!("workspace coverage termination heartbeat evidence is absent")
                })?;
            let heartbeat: WorkspaceCoverageProviderHeartbeatEntry =
                rmp_serde::from_slice(&envelope.payload)?;
            authenticate_heartbeat_against_launch(
                &heartbeat,
                &launch,
                &envelope_digest(&launch_envelope),
            )?;
            if heartbeat.heartbeat_id != heartbeat_id
                || envelope_digest(&envelope) != expected_digest
            {
                bail!("workspace coverage termination heartbeat evidence disagrees");
            }
            Some(heartbeat)
        }
        (None, None) => None,
        _ => bail!("workspace coverage termination has partial heartbeat evidence"),
    };
    if entry.policy_id != policy.policy_id
        || entry.policy_envelope_digest != envelope_digest(&policy_envelope)
        || launch.policy_id != policy.policy_id
        || launch.policy_envelope_digest != entry.policy_envelope_digest
        || entry.launch_envelope_digest != envelope_digest(&launch_envelope)
        || entry.host_identity_id != launch.host_identity_id
        || entry.host_identity_record_digest != launch.host_identity_record_digest
        || entry.expected_boot_identity != launch.boot_identity
        || entry.expected_process_id != launch.process_id
        || entry.expected_process_creation_token != launch.process_creation_token
        || entry.expected_process_executable_path != launch.process_executable_path
    {
        bail!("workspace coverage termination evidence chain disagrees with its exact sources");
    }
    if heartbeat_evidence
        .as_ref()
        .map(|heartbeat| heartbeat.heartbeat_id.as_str())
        != entry.heartbeat_id.as_deref()
    {
        bail!("workspace coverage termination heartbeat identity disagrees");
    }
    Ok(entry)
}

pub fn authenticate_workspace_coverage_termination_with_envelope_digest(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    launch_id: &str,
    host: &HostIncarnationIdentityEntry,
) -> Result<(WorkspaceCoverageProcessTerminationObservationEntry, String)> {
    let store_path = store_path.as_ref();
    let runtime_id = runtime_id.into();
    let entry = authenticate_workspace_coverage_process_termination_observation(
        store_path,
        runtime_id.clone(),
        launch_id,
        host,
    )?;
    let envelope = open_epiphany_cultmesh_node(store_path, runtime_id)?
        .cache()
        .get_envelope::<WorkspaceCoverageProcessTerminationObservationEntry>(&termination_key(
            launch_id,
        ))?
        .ok_or_else(|| anyhow!("workspace coverage termination envelope disappeared"))?;
    Ok((entry, envelope_digest(&envelope)))
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
    match (
        entry.replaces_launch_id.as_deref(),
        entry.replaces_termination_id.as_deref(),
        entry.replaces_termination_envelope_digest.as_deref(),
    ) {
        (None, None, None) => {}
        (Some(launch_id), Some(termination_id), Some(digest))
            if !launch_id.trim().is_empty() && !termination_id.trim().is_empty() =>
        {
            validate_digest("replacement termination", digest)?;
        }
        _ => bail!("workspace coverage replacement launch has a partial causal edge"),
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

fn validate_termination(
    entry: &WorkspaceCoverageProcessTerminationObservationEntry,
    host: &HostIncarnationIdentityEntry,
) -> Result<()> {
    if entry.schema_version != WORKSPACE_COVERAGE_PROCESS_TERMINATION_SCHEMA_VERSION
        || entry.observer_id != WORKSPACE_COVERAGE_TERMINATION_OBSERVER
        || entry.termination_id != entry.launch_id
        || entry.runtime_id.trim().is_empty()
    {
        bail!("workspace coverage termination violates its reserved authority");
    }
    uuid::Uuid::parse_str(&entry.launch_id).context("termination launch id must be UUID")?;
    match (
        entry.heartbeat_id.as_deref(),
        entry.heartbeat_envelope_digest.as_deref(),
    ) {
        (Some(id), Some(digest)) => {
            uuid::Uuid::parse_str(id).context("termination heartbeat id must be UUID")?;
            validate_digest("heartbeat", digest)?;
        }
        (None, None) => {}
        _ => bail!("termination heartbeat evidence is partial"),
    }
    DateTime::parse_from_rfc3339(&entry.observed_at_utc)
        .context("termination observation time must be RFC3339")?;
    for (label, digest) in [
        ("launch", &entry.launch_envelope_digest),
        ("policy", &entry.policy_envelope_digest),
        ("host identity record", &entry.host_identity_record_digest),
    ] {
        validate_digest(label, digest)?;
    }
    for (label, value) in [
        ("policy id", entry.policy_id.as_str()),
        ("host identity id", entry.host_identity_id.as_str()),
        (
            "expected boot identity",
            entry.expected_boot_identity.as_str(),
        ),
        (
            "observed boot identity",
            entry.observed_boot_identity.as_str(),
        ),
    ] {
        require_nonempty(label, value)?;
    }
    validate_absolute_path(&entry.expected_process_executable_path)?;
    if entry.expected_process_id == 0 || entry.expected_process_creation_token == 0 {
        bail!("termination expected process identity is invalid");
    }
    match entry.outcome.as_str() {
        "exact_exited" => {
            if replacement_fields_present(entry) {
                bail!("exact exit must not carry a replacement process identity");
            }
        }
        "process_missing" => {
            if entry.exit_code.is_some() || replacement_fields_present(entry) {
                bail!("missing process must not carry exit or replacement material");
            }
        }
        "process_replaced" => {
            if entry.exit_code.is_some()
                || entry.replacement_process_id.unwrap_or(0) == 0
                || entry.replacement_process_creation_token.unwrap_or(0) == 0
                || entry.replacement_process_executable_path.is_none()
            {
                bail!("replaced process requires one complete replacement identity");
            }
            validate_absolute_path(
                entry
                    .replacement_process_executable_path
                    .as_deref()
                    .unwrap_or_default(),
            )?;
            if entry.replacement_process_id == Some(entry.expected_process_id)
                && entry.replacement_process_creation_token
                    == Some(entry.expected_process_creation_token)
                && entry.replacement_process_executable_path.as_deref()
                    == Some(entry.expected_process_executable_path.as_str())
            {
                bail!("replacement process identity must differ from the terminated instance");
            }
        }
        "boot_superseded" => {
            if entry.observed_boot_identity == entry.expected_boot_identity
                || entry.exit_code.is_some()
                || replacement_fields_present(entry)
            {
                bail!("boot supersession requires two distinct boot identities only");
            }
        }
        _ => bail!("workspace coverage termination outcome is not authoritative"),
    }
    if entry.outcome != "boot_superseded"
        && entry.observed_boot_identity != entry.expected_boot_identity
    {
        bail!("same-boot process outcome disagrees with the expected boot identity");
    }
    if entry.signature_algorithm != "ed25519"
        || entry.host_signature.len() != 64
        || entry.host_identity_id != host.identity_id
        || entry.host_identity_record_digest
            != workspace_coverage_host_identity_record_digest(host)?
    {
        bail!("workspace coverage termination host signature material is invalid");
    }
    verify_host_identity_signature(
        host,
        HOST_TERMINATION_PURPOSE,
        &workspace_coverage_termination_statement(entry)?,
        &HostIdentitySignature {
            identity_id: entry.host_identity_id.clone(),
            signature: entry.host_signature.clone(),
        },
    )
}

fn replacement_fields_present(entry: &WorkspaceCoverageProcessTerminationObservationEntry) -> bool {
    entry.replacement_process_id.is_some()
        || entry.replacement_process_creation_token.is_some()
        || entry.replacement_process_created_at_rfc3339.is_some()
        || entry.replacement_process_executable_path.is_some()
}

fn require_nonempty(name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        bail!("workspace coverage {name} is required");
    }
    Ok(())
}

fn claim_sight_key(runtime_id: &str, body_generation: u64, body_observation_id: &str) -> String {
    format!(
        "epiphany-local/workspace-coverage/claim-sight/{runtime_id}/{body_generation}/{body_observation_id}"
    )
}

fn validate_claim_sight_shape(
    entry: &WorkspaceCoverageClaimSightEntry,
    signed: bool,
) -> Result<()> {
    if entry.schema_version != WORKSPACE_COVERAGE_CLAIM_SIGHT_SCHEMA_VERSION
        || entry.claim_epoch == 0
        || entry.signature_algorithm != "ed25519"
        || entry.provider_public_key.len() != 32
        || signed != (entry.provider_signature.len() == 64)
    {
        bail!("workspace coverage claim sight violates its reserved schema or signature shape");
    }
    for (name, value) in [
        ("runtime id", entry.runtime_id.as_str()),
        ("workspace id", entry.workspace_id.as_str()),
        ("launch id", entry.launch_id.as_str()),
        ("claim id", entry.claim_id.as_str()),
        ("attempt id", entry.attempt_id.as_str()),
        ("plan id", entry.plan_id.as_str()),
        ("Body observation id", entry.body_observation_id.as_str()),
    ] {
        require_nonempty(name, value)?;
    }
    uuid::Uuid::parse_str(&entry.launch_id).context("claim sight launch id must be UUID")?;
    uuid::Uuid::parse_str(&entry.claim_id).context("claim sight claim id must be UUID")?;
    uuid::Uuid::parse_str(&entry.attempt_id).context("claim sight attempt id must be UUID")?;
    DateTime::parse_from_rfc3339(&entry.observed_at_utc)
        .context("claim sight observation time must be RFC3339")?;
    Ok(())
}

pub(crate) fn publish_workspace_coverage_claim_sight(
    local_verse_store: &Path,
    runtime_store: &Path,
    coverage: &crate::WorkspaceCoverageAuthority,
    acquisition: &crate::workspace_coverage_projector::WorkspaceCoverageAcquisition,
    runtime_id: &str,
    launch_id: &str,
    trusted_host: &HostIncarnationIdentityEntry,
    provider: &SigningKey,
    observed_at: chrono::DateTime<chrono::Utc>,
) -> Result<WorkspaceCoverageClaimSightEntry> {
    if acquisition.claim.status != "running"
        || acquisition.claim.managed_process_launch_id != launch_id
        || acquisition.claim.attempt_id != acquisition.attempt.attempt_id
    {
        bail!("claim sight requires the projector's exact running claim and attempt");
    }
    let opening = coverage.store.pull_all()?;
    let attempt_env = opening
        .iter()
        .find(|env| {
            env.r#type == "gamecult.epiphany.workspace_coverage_projection_attempt"
                && env.key == acquisition.attempt.attempt_id
        })
        .ok_or_else(|| anyhow!("claim sight attempt envelope is absent"))?;
    let (launch, launch_digest) =
        authenticate_workspace_coverage_managed_process_launch_with_envelope_digest(
            local_verse_store,
            runtime_id,
            launch_id,
            trusted_host,
        )?;
    if launch.provider_public_key.as_slice() != provider.verifying_key().to_bytes()
        || launch.provider_incarnation_id != acquisition.claim.executor_incarnation
    {
        bail!("claim sight provider disagrees with its authenticated launch");
    }
    let basis = crate::load_current_runtime_repository_body_basis(runtime_store)?;
    let mut entry = WorkspaceCoverageClaimSightEntry {
        schema_version: WORKSPACE_COVERAGE_CLAIM_SIGHT_SCHEMA_VERSION.into(),
        runtime_id: runtime_id.into(),
        workspace_id: basis.workspace_id,
        launch_id: launch_id.into(),
        launch_envelope_digest: launch_digest,
        provider_incarnation_id: launch.provider_incarnation_id,
        provider_public_key: launch.provider_public_key,
        coverage_store_binding_id: coverage.store_binding.binding_id.clone(),
        coverage_store_binding_envelope_digest: coverage.store_binding_envelope_sha256.clone(),
        coverage_store_file_identity: coverage.store_binding.store_file_identity.clone(),
        runtime_coverage_route_envelope_digest: coverage
            .runtime_coverage_route_envelope_sha256
            .clone(),
        body_binding_sha256: basis.body_binding_sha256,
        body_observation_id: acquisition.claim.body_observation_id.clone(),
        body_generation: acquisition.claim.body_generation,
        manifest_root_sha256: acquisition.claim.manifest_root_sha256.clone(),
        claim_id: acquisition.claim.claim_id.clone(),
        claim_epoch: acquisition.claim.claim_epoch,
        plan_id: acquisition.claim.plan_id.clone(),
        attempt_id: acquisition.attempt.attempt_id.clone(),
        attempt_envelope_digest: envelope_digest(attempt_env),
        recovery_receipt_id: acquisition.claim.recovery_receipt_id.clone(),
        recovery_receipt_digest: acquisition.claim.recovery_receipt_digest.clone(),
        observed_at_utc: observed_at.to_rfc3339(),
        provider_signature: Vec::new(),
        signature_algorithm: "ed25519".into(),
    };
    validate_workspace_coverage_sight_closing_authority(
        runtime_store,
        coverage,
        &entry.workspace_id,
        &entry.body_binding_sha256,
        &entry.body_observation_id,
        entry.body_generation,
        &entry.manifest_root_sha256,
    )?;
    validate_claim_sight_shape(&entry, false)?;
    entry.provider_signature = provider
        .sign(&signed_sight_statement(
            COVERAGE_CLAIM_SIGHT_DOMAIN,
            &entry,
        )?)
        .to_bytes()
        .to_vec();
    validate_claim_sight_shape(&entry, true)?;
    let backing = SingleFileMessagePackBackingStore::new(local_verse_store);
    let node = open_epiphany_cultmesh_node(local_verse_store, runtime_id.to_string())?;
    let key = claim_sight_key(
        runtime_id,
        entry.body_generation,
        &entry.body_observation_id,
    );
    let opening = backing.pull_all()?;
    if let Some(prior_env) = opening
        .iter()
        .find(|env| env.r#type == WORKSPACE_COVERAGE_CLAIM_SIGHT_TYPE && env.key == key)
    {
        let prior: WorkspaceCoverageClaimSightEntry = rmp_serde::from_slice(&prior_env.payload)?;
        authenticate_workspace_coverage_claim_sight_entry(&prior)?;
        if entry.claim_epoch < prior.claim_epoch
            || (entry.claim_epoch == prior.claim_epoch && entry.claim_id != prior.claim_id)
        {
            bail!("workspace coverage claim sight cannot regress or fork its epoch");
        }
    }
    replace_latest_sight(
        node.cache(),
        &backing,
        WORKSPACE_COVERAGE_CLAIM_SIGHT_TYPE,
        &key,
        &entry,
        &opening,
    )?;
    Ok(entry)
}

fn authenticate_workspace_coverage_claim_sight_entry(
    entry: &WorkspaceCoverageClaimSightEntry,
) -> Result<()> {
    validate_claim_sight_shape(entry, true)?;
    let key: [u8; 32] = entry
        .provider_public_key
        .clone()
        .try_into()
        .map_err(|_| anyhow!("claim sight provider key length is invalid"))?;
    let signature = Signature::from_slice(&entry.provider_signature)?;
    let mut unsigned = entry.clone();
    unsigned.provider_signature.clear();
    VerifyingKey::from_bytes(&key)?
        .verify(
            &signed_sight_statement(COVERAGE_CLAIM_SIGHT_DOMAIN, &unsigned)?,
            &signature,
        )
        .map_err(|_| anyhow!("workspace coverage claim sight signature verification failed"))?;
    Ok(())
}

pub fn authenticate_current_workspace_coverage_claim_sight(
    local_verse_store: impl AsRef<Path>,
    runtime_store: impl AsRef<Path>,
    runtime_id: &str,
    trusted_host: &HostIncarnationIdentityEntry,
) -> Result<Option<WorkspaceCoverageClaimSightEntry>> {
    let runtime_store = runtime_store.as_ref();
    let basis = crate::load_current_runtime_repository_body_basis(runtime_store)?;
    let route = crate::runtime_workspace_coverage_store_binding(runtime_store)?
        .ok_or_else(|| anyhow!("runtime has no workspace coverage route"))?;
    let runtime_opening = SingleFileMessagePackBackingStore::new(runtime_store).pull_all()?;
    let route_env = runtime_opening
        .iter()
        .find(|env| {
            env.r#type == crate::RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_TYPE
                && env.key == crate::RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_KEY
        })
        .ok_or_else(|| anyhow!("runtime workspace coverage route envelope is absent"))?;
    let node = open_epiphany_cultmesh_node(local_verse_store.as_ref(), runtime_id.to_string())?;
    let Some(env) = node
        .cache()
        .get_envelope::<WorkspaceCoverageClaimSightEntry>(&claim_sight_key(
            runtime_id,
            basis.generation,
            &basis.observation_id,
        ))?
    else {
        return Ok(None);
    };
    if env.r#type != WORKSPACE_COVERAGE_CLAIM_SIGHT_TYPE {
        bail!("claim sight key has alien type");
    }
    let entry: WorkspaceCoverageClaimSightEntry = rmp_serde::from_slice(&env.payload)?;
    authenticate_workspace_coverage_claim_sight_entry(&entry)?;
    let (launch, digest) =
        authenticate_workspace_coverage_managed_process_launch_with_envelope_digest(
            local_verse_store.as_ref(),
            runtime_id,
            &entry.launch_id,
            trusted_host,
        )?;
    if entry.runtime_id != runtime_id
        || entry.launch_envelope_digest != digest
        || entry.provider_incarnation_id != launch.provider_incarnation_id
        || entry.provider_public_key != launch.provider_public_key
        || entry.workspace_id != basis.workspace_id
        || entry.body_binding_sha256 != basis.body_binding_sha256
        || entry.body_observation_id != basis.observation_id
        || entry.body_generation != basis.generation
        || entry.manifest_root_sha256 != basis.manifest_root_sha256
        || entry.coverage_store_binding_id != route.binding_id
        || entry.coverage_store_file_identity != route.store_file_identity
        || entry.runtime_coverage_route_envelope_digest
            != format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(route_env)?))
    {
        bail!("claim sight disagrees with authenticated launch");
    }
    Ok(Some(entry))
}

fn recovery_directive_key(runtime_id: &str, replacement_launch_id: &str) -> String {
    format!(
        "epiphany-local/workspace-coverage/recovery-directive/{runtime_id}/{replacement_launch_id}"
    )
}

fn recovery_directive_statement(
    entry: &WorkspaceCoverageRecoveryDirectiveEntry,
) -> Result<Vec<u8>> {
    let mut unsigned = entry.clone();
    unsigned.host_signature.clear();
    Ok(rmp_serde::to_vec_named(&unsigned)?)
}

pub fn write_workspace_coverage_recovery_directive(
    local_verse_store: impl AsRef<Path>,
    runtime_store: impl AsRef<Path>,
    runtime_id: &str,
    claim_sight: &WorkspaceCoverageClaimSightEntry,
    replacement_launch_id: &str,
    replacement_ready_heartbeat_id: &str,
    host: &HostIdentitySigner,
) -> Result<WorkspaceCoverageRecoveryDirectiveEntry> {
    let local_verse_store = local_verse_store.as_ref();
    let runtime_store = runtime_store.as_ref();
    let authenticated = authenticate_current_workspace_coverage_claim_sight(
        local_verse_store,
        runtime_store,
        runtime_id,
        host.entry(),
    )?
    .ok_or_else(|| anyhow!("workspace coverage recovery claim sight is absent"))?;
    if &authenticated != claim_sight {
        bail!("workspace coverage recovery claim sight moved");
    }
    let node = open_epiphany_cultmesh_node(local_verse_store, runtime_id.to_string())?;
    let directive_key = recovery_directive_key(runtime_id, replacement_launch_id);
    if node
        .cache()
        .get_envelope::<WorkspaceCoverageRecoveryDirectiveEntry>(&directive_key)?
        .is_some()
    {
        let existing = authenticate_workspace_coverage_recovery_directive(
            local_verse_store,
            runtime_store,
            runtime_id,
            replacement_launch_id,
            host.entry(),
        )?
        .ok_or_else(|| anyhow!("existing recovery directive belongs to a stale Body"))?;
        if existing.old_claim_id != claim_sight.claim_id
            || existing.old_claim_epoch != claim_sight.claim_epoch
            || existing.old_plan_id != claim_sight.plan_id
            || existing.old_launch_id != claim_sight.launch_id
            || existing.workspace_id != claim_sight.workspace_id
            || existing.body_binding_sha256 != claim_sight.body_binding_sha256
            || existing.body_observation_id != claim_sight.body_observation_id
            || existing.body_generation != claim_sight.body_generation
            || existing.manifest_root_sha256 != claim_sight.manifest_root_sha256
            || existing.coverage_store_binding_id != claim_sight.coverage_store_binding_id
            || existing.coverage_store_binding_envelope_digest
                != claim_sight.coverage_store_binding_envelope_digest
            || existing.coverage_store_file_identity != claim_sight.coverage_store_file_identity
            || existing.runtime_coverage_route_envelope_digest
                != claim_sight.runtime_coverage_route_envelope_digest
        {
            bail!(
                "recovery directive replacement identity already belongs to another claim lineage"
            );
        }
        return Ok(existing);
    }
    let claim_env = node
        .cache()
        .get_envelope::<WorkspaceCoverageClaimSightEntry>(&claim_sight_key(
            runtime_id,
            claim_sight.body_generation,
            &claim_sight.body_observation_id,
        ))?
        .ok_or_else(|| anyhow!("workspace coverage recovery claim sight envelope is absent"))?;
    let (termination, termination_digest) =
        authenticate_workspace_coverage_termination_with_envelope_digest(
            local_verse_store,
            runtime_id.to_string(),
            &claim_sight.launch_id,
            host.entry(),
        )?;
    let (replacement, replacement_digest) =
        authenticate_workspace_coverage_managed_process_launch_with_envelope_digest(
            local_verse_store,
            runtime_id,
            replacement_launch_id,
            host.entry(),
        )?;
    let (ready, ready_digest) =
        authenticate_workspace_coverage_provider_heartbeat_with_envelope_digest(
            local_verse_store,
            runtime_id,
            replacement_ready_heartbeat_id,
            host.entry(),
        )?;
    if replacement.replaces_launch_id.as_deref() != Some(claim_sight.launch_id.as_str())
        || replacement.replaces_termination_id.as_deref()
            != Some(termination.termination_id.as_str())
        || ready.launch_id != replacement.launch_id
        || ready.status != "ready"
    {
        bail!("workspace coverage recovery directive evidence is not an exact ready replacement");
    }
    let mut entry = WorkspaceCoverageRecoveryDirectiveEntry {
        schema_version: WORKSPACE_COVERAGE_RECOVERY_DIRECTIVE_SCHEMA_VERSION.into(),
        directive_id: uuid::Uuid::new_v4().to_string(),
        runtime_id: runtime_id.into(),
        old_claim_id: claim_sight.claim_id.clone(),
        old_launch_id: claim_sight.launch_id.clone(),
        replacement_launch_id: replacement.launch_id,
        replacement_ready_heartbeat_id: ready.heartbeat_id,
        termination_id: termination.termination_id,
        claim_sight_envelope_digest: envelope_digest(&claim_env),
        old_claim_epoch: claim_sight.claim_epoch,
        old_plan_id: claim_sight.plan_id.clone(),
        termination_envelope_digest: termination_digest,
        replacement_launch_envelope_digest: replacement_digest,
        replacement_ready_heartbeat_envelope_digest: ready_digest,
        workspace_id: claim_sight.workspace_id.clone(),
        body_binding_sha256: claim_sight.body_binding_sha256.clone(),
        body_observation_id: claim_sight.body_observation_id.clone(),
        body_generation: claim_sight.body_generation,
        manifest_root_sha256: claim_sight.manifest_root_sha256.clone(),
        coverage_store_binding_id: claim_sight.coverage_store_binding_id.clone(),
        coverage_store_binding_envelope_digest: claim_sight
            .coverage_store_binding_envelope_digest
            .clone(),
        coverage_store_file_identity: claim_sight.coverage_store_file_identity.clone(),
        runtime_coverage_route_envelope_digest: claim_sight
            .runtime_coverage_route_envelope_digest
            .clone(),
        issued_at_utc: chrono::Utc::now().to_rfc3339(),
        host_identity_id: host.entry().identity_id.clone(),
        host_identity_record_digest: workspace_coverage_host_identity_record_digest(host.entry())?,
        host_signature: Vec::new(),
        signature_algorithm: "ed25519".into(),
    };
    entry.host_signature = host
        .sign(
            HOST_RECOVERY_DIRECTIVE_PURPOSE,
            &recovery_directive_statement(&entry)?,
        )?
        .signature;
    let backing = SingleFileMessagePackBackingStore::new(local_verse_store);
    let opening = backing.pull_all()?;
    replace_latest_sight(
        node.cache(),
        &backing,
        WORKSPACE_COVERAGE_RECOVERY_DIRECTIVE_TYPE,
        &directive_key,
        &entry,
        &opening,
    )?;
    Ok(entry)
}

pub fn authenticate_workspace_coverage_recovery_directive(
    local_verse_store: impl AsRef<Path>,
    runtime_store: impl AsRef<Path>,
    runtime_id: &str,
    replacement_launch_id: &str,
    trusted_host: &HostIncarnationIdentityEntry,
) -> Result<Option<WorkspaceCoverageRecoveryDirectiveEntry>> {
    let node = open_epiphany_cultmesh_node(local_verse_store.as_ref(), runtime_id.to_string())?;
    let Some(env) = node
        .cache()
        .get_envelope::<WorkspaceCoverageRecoveryDirectiveEntry>(&recovery_directive_key(
            runtime_id,
            replacement_launch_id,
        ))?
    else {
        return Ok(None);
    };
    let entry: WorkspaceCoverageRecoveryDirectiveEntry = rmp_serde::from_slice(&env.payload)?;
    if entry.schema_version != WORKSPACE_COVERAGE_RECOVERY_DIRECTIVE_SCHEMA_VERSION
        || entry.runtime_id != runtime_id
        || entry.replacement_launch_id != replacement_launch_id
        || entry.host_identity_id != trusted_host.identity_id
        || entry.host_identity_record_digest
            != workspace_coverage_host_identity_record_digest(trusted_host)?
        || entry.signature_algorithm != "ed25519"
    {
        bail!("workspace coverage recovery directive identity is invalid");
    }
    verify_host_identity_signature(
        trusted_host,
        HOST_RECOVERY_DIRECTIVE_PURPOSE,
        &recovery_directive_statement(&entry)?,
        &HostIdentitySignature {
            identity_id: entry.host_identity_id.clone(),
            signature: entry.host_signature.clone(),
        },
    )?;
    let basis = crate::load_current_runtime_repository_body_basis(runtime_store.as_ref())?;
    if entry.body_observation_id != basis.observation_id
        || entry.body_generation != basis.generation
        || entry.manifest_root_sha256 != basis.manifest_root_sha256
        || entry.body_binding_sha256 != basis.body_binding_sha256
        || entry.workspace_id != basis.workspace_id
    {
        // A directive is authority only for its exact Body generation. Keeping
        // the immutable document is useful evidence; treating it as current
        // pressure after Body advances would strand the replacement process.
        return Ok(None);
    }
    let claim_node =
        open_epiphany_cultmesh_node(local_verse_store.as_ref(), runtime_id.to_string())?;
    let claim_env = claim_node
        .cache()
        .get_envelope::<WorkspaceCoverageClaimSightEntry>(&claim_sight_key(
            runtime_id,
            basis.generation,
            &basis.observation_id,
        ))?
        .ok_or_else(|| anyhow!("recovery directive claim sight is absent"))?;
    let claim: WorkspaceCoverageClaimSightEntry = rmp_serde::from_slice(&claim_env.payload)?;
    authenticate_workspace_coverage_claim_sight_entry(&claim)?;
    let (termination, termination_digest) =
        authenticate_workspace_coverage_termination_with_envelope_digest(
            local_verse_store.as_ref(),
            runtime_id.to_string(),
            &entry.old_launch_id,
            trusted_host,
        )?;
    let (_, replacement_digest) =
        authenticate_workspace_coverage_managed_process_launch_with_envelope_digest(
            local_verse_store.as_ref(),
            runtime_id,
            &entry.replacement_launch_id,
            trusted_host,
        )?;
    let (_, ready_digest) =
        authenticate_workspace_coverage_provider_heartbeat_with_envelope_digest(
            local_verse_store.as_ref(),
            runtime_id,
            &entry.replacement_ready_heartbeat_id,
            trusted_host,
        )?;
    if entry.claim_sight_envelope_digest != envelope_digest(&claim_env)
        || entry.old_claim_id != claim.claim_id
        || entry.old_claim_epoch != claim.claim_epoch
        || entry.old_plan_id != claim.plan_id
        || entry.old_launch_id != claim.launch_id
        || entry.termination_id != termination.termination_id
        || entry.termination_envelope_digest != termination_digest
        || entry.replacement_launch_envelope_digest != replacement_digest
        || entry.replacement_ready_heartbeat_envelope_digest != ready_digest
        || entry.workspace_id != claim.workspace_id
        || entry.body_binding_sha256 != claim.body_binding_sha256
        || entry.body_observation_id != claim.body_observation_id
        || entry.body_generation != claim.body_generation
        || entry.manifest_root_sha256 != claim.manifest_root_sha256
        || entry.coverage_store_binding_id != claim.coverage_store_binding_id
        || entry.coverage_store_binding_envelope_digest
            != claim.coverage_store_binding_envelope_digest
        || entry.coverage_store_file_identity != claim.coverage_store_file_identity
        || entry.runtime_coverage_route_envelope_digest
            != claim.runtime_coverage_route_envelope_digest
    {
        bail!("workspace coverage recovery directive evidence moved or was substituted");
    }
    Ok(Some(entry))
}

fn advancement_sight_key(launch_id: &str, claim_id: &str) -> String {
    format!("epiphany-local/workspace-coverage/advancement-sight/{launch_id}/{claim_id}")
}

fn terminal_sight_key(runtime_id: &str, body_generation: u64, body_observation_id: &str) -> String {
    format!(
        "epiphany-local/workspace-coverage/terminal-sight/{runtime_id}/{body_generation}/{body_observation_id}"
    )
}

fn signed_sight_statement<T: Serialize + Clone>(domain: &[u8], unsigned: &T) -> Result<Vec<u8>> {
    let mut statement = domain.to_vec();
    statement.extend(rmp_serde::to_vec_named(unsigned)?);
    Ok(statement)
}

pub(crate) fn sign_workspace_coverage_advancement_sight(
    entry: &mut WorkspaceCoverageAdvancementSightEntry,
    provider: &SigningKey,
) -> Result<()> {
    entry.provider_signature.clear();
    validate_advancement_sight_shape(entry, false)?;
    entry.provider_signature = provider
        .sign(&signed_sight_statement(
            COVERAGE_ADVANCEMENT_SIGHT_DOMAIN,
            entry,
        )?)
        .to_bytes()
        .to_vec();
    Ok(())
}

pub(crate) fn sign_workspace_coverage_terminal_sight(
    entry: &mut WorkspaceCoverageTerminalSightEntry,
    provider: &SigningKey,
) -> Result<()> {
    entry.provider_signature.clear();
    validate_terminal_sight_shape(entry, false)?;
    entry.provider_signature = provider
        .sign(&signed_sight_statement(
            COVERAGE_TERMINAL_SIGHT_DOMAIN,
            entry,
        )?)
        .to_bytes()
        .to_vec();
    Ok(())
}

pub(crate) fn write_workspace_coverage_advancement_sight(
    local_verse_store: impl AsRef<Path>,
    runtime_id: &str,
    entry: WorkspaceCoverageAdvancementSightEntry,
) -> Result<WorkspaceCoverageAdvancementSightEntry> {
    validate_advancement_sight_shape(&entry, true)?;
    if entry.runtime_id != runtime_id {
        bail!("workspace coverage advancement sight runtime argument disagrees");
    }
    let backing = SingleFileMessagePackBackingStore::new(local_verse_store.as_ref());
    let node = open_epiphany_cultmesh_node(local_verse_store.as_ref(), runtime_id.to_string())?;
    let key = advancement_sight_key(&entry.launch_id, &entry.claim_id);
    let opening = backing.pull_all()?;
    if let Some(prior) = opening.iter().find(|envelope| {
        envelope.r#type == WORKSPACE_COVERAGE_ADVANCEMENT_SIGHT_TYPE && envelope.key == key
    }) {
        let prior: WorkspaceCoverageAdvancementSightEntry = rmp_serde::from_slice(&prior.payload)?;
        validate_advancement_sight_shape(&prior, true)?;
        let mut unsigned = prior.clone();
        unsigned.provider_signature.clear();
        verify_sight_signature(
            COVERAGE_ADVANCEMENT_SIGHT_DOMAIN,
            &unsigned,
            &prior.provider_public_key,
            &prior.provider_signature,
        )?;
        if entry.sequence <= prior.sequence
            || entry.completed_units < prior.completed_units
            || entry.total_units != prior.total_units
            || entry.body_observation_id != prior.body_observation_id
            || entry.body_generation != prior.body_generation
            || entry.plan_id != prior.plan_id
            || entry.launch_envelope_digest != prior.launch_envelope_digest
            || entry.provider_incarnation_id != prior.provider_incarnation_id
            || entry.provider_public_key != prior.provider_public_key
            || entry.coverage_store_binding_id != prior.coverage_store_binding_id
            || entry.coverage_store_binding_envelope_digest
                != prior.coverage_store_binding_envelope_digest
            || entry.coverage_store_file_identity != prior.coverage_store_file_identity
            || entry.runtime_coverage_route_envelope_digest
                != prior.runtime_coverage_route_envelope_digest
            || entry.body_binding_sha256 != prior.body_binding_sha256
            || entry.manifest_root_sha256 != prior.manifest_root_sha256
        {
            bail!("workspace coverage advancement sight cannot regress or substitute its basis");
        }
    }
    replace_latest_sight(
        node.cache(),
        &backing,
        WORKSPACE_COVERAGE_ADVANCEMENT_SIGHT_TYPE,
        &key,
        &entry,
        &opening,
    )?;
    Ok(entry)
}

pub(crate) fn write_workspace_coverage_terminal_sight(
    local_verse_store: impl AsRef<Path>,
    runtime_id: &str,
    entry: WorkspaceCoverageTerminalSightEntry,
) -> Result<WorkspaceCoverageTerminalSightEntry> {
    validate_terminal_sight_shape(&entry, true)?;
    if entry.runtime_id != runtime_id {
        bail!("workspace coverage terminal sight runtime argument disagrees");
    }
    let backing = SingleFileMessagePackBackingStore::new(local_verse_store.as_ref());
    let node = open_epiphany_cultmesh_node(local_verse_store.as_ref(), runtime_id.to_string())?;
    let key = terminal_sight_key(
        runtime_id,
        entry.body_generation,
        &entry.body_observation_id,
    );
    let opening = backing.pull_all()?;
    if let Some(prior) = opening.iter().find(|envelope| {
        envelope.r#type == WORKSPACE_COVERAGE_TERMINAL_SIGHT_TYPE && envelope.key == key
    }) {
        let prior: WorkspaceCoverageTerminalSightEntry = rmp_serde::from_slice(&prior.payload)?;
        validate_terminal_sight_shape(&prior, true)?;
        let mut unsigned = prior.clone();
        unsigned.provider_signature.clear();
        verify_sight_signature(
            COVERAGE_TERMINAL_SIGHT_DOMAIN,
            &unsigned,
            &prior.provider_public_key,
            &prior.provider_signature,
        )?;
        if prior == entry {
            return Ok(entry);
        }
        bail!("workspace coverage terminal sight is immutable for its exact Body basis");
    }
    replace_latest_sight(
        node.cache(),
        &backing,
        WORKSPACE_COVERAGE_TERMINAL_SIGHT_TYPE,
        &key,
        &entry,
        &opening,
    )?;
    Ok(entry)
}

/// Publishes operator sight only after the owned coverage store proves a
/// current durable checkpoint/progress join. The returned document grants no
/// authority over that store.
#[allow(clippy::too_many_arguments)]
pub(crate) fn publish_workspace_coverage_advancement_sight(
    local_verse_store: &Path,
    runtime_store: &Path,
    coverage: &crate::WorkspaceCoverageAuthority,
    runtime_id: &str,
    launch_id: &str,
    trusted_host: &HostIncarnationIdentityEntry,
    provider: &SigningKey,
    observed_at: chrono::DateTime<chrono::Utc>,
    no_advance_lease: chrono::Duration,
) -> Result<WorkspaceCoverageAdvancementSightEntry> {
    let advancing = crate::authenticate_current_workspace_coverage_advancement(
        Path::new(&coverage.runtime_body_route.body_store_path),
        &coverage.store,
        local_verse_store,
        runtime_id,
        launch_id,
        trusted_host,
        observed_at,
        no_advance_lease,
    )?
    .ok_or_else(|| anyhow!("owned workspace coverage has no current advancing authority"))?;
    let opening = coverage.store.pull_all()?;
    let progress_env = opening
        .iter()
        .find(|env| {
            env.r#type == crate::WORKSPACE_COVERAGE_PROJECTION_PROGRESS_TYPE && {
                rmp_serde::from_slice::<crate::WorkspaceCoverageProjectionProgressEntry>(
                    &env.payload,
                )
                .is_ok_and(|progress| progress.progress_id == advancing.progress_id)
            }
        })
        .ok_or_else(|| anyhow!("advancement sight progress envelope is absent"))?;
    let progress: crate::WorkspaceCoverageProjectionProgressEntry =
        rmp_serde::from_slice(&progress_env.payload)?;
    let checkpoint_env = opening.iter().find(|env| env.r#type == crate::workspace_coverage_projection_batch_checkpoint::WORKSPACE_COVERAGE_PROJECTION_BATCH_CHECKPOINT_TYPE && {
        rmp_serde::from_slice::<crate::workspace_coverage_projection_batch_checkpoint::WorkspaceCoverageProjectionBatchCheckpointEntry>(&env.payload)
            .is_ok_and(|checkpoint| checkpoint.checkpoint_id == advancing.checkpoint_id)
    }).ok_or_else(|| anyhow!("advancement sight checkpoint envelope is absent"))?;
    let (launch, launch_digest) =
        authenticate_workspace_coverage_managed_process_launch_with_envelope_digest(
            local_verse_store,
            runtime_id,
            launch_id,
            trusted_host,
        )?;
    let basis = crate::load_current_runtime_repository_body_basis(runtime_store)?;
    let mut entry = WorkspaceCoverageAdvancementSightEntry {
        schema_version: WORKSPACE_COVERAGE_ADVANCEMENT_SIGHT_SCHEMA_VERSION.into(),
        runtime_id: runtime_id.into(),
        workspace_id: basis.workspace_id,
        launch_id: launch_id.into(),
        launch_envelope_digest: launch_digest,
        provider_incarnation_id: launch.provider_incarnation_id,
        provider_public_key: launch.provider_public_key,
        coverage_store_binding_id: coverage.store_binding.binding_id.clone(),
        coverage_store_binding_envelope_digest: coverage.store_binding_envelope_sha256.clone(),
        coverage_store_file_identity: coverage.store_binding.store_file_identity.clone(),
        runtime_coverage_route_envelope_digest: coverage
            .runtime_coverage_route_envelope_sha256
            .clone(),
        body_binding_sha256: basis.body_binding_sha256,
        body_observation_id: basis.observation_id,
        body_generation: basis.generation,
        manifest_root_sha256: basis.manifest_root_sha256,
        claim_id: advancing.claim_id,
        claim_epoch: advancing.claim_epoch,
        plan_id: advancing.plan_id,
        progress_id: advancing.progress_id,
        progress_envelope_digest: envelope_digest(progress_env),
        checkpoint_id: advancing.checkpoint_id,
        checkpoint_envelope_digest: envelope_digest(checkpoint_env),
        sequence: progress.sequence,
        status: "warming".into(),
        completed_units: advancing.completed_units,
        total_units: advancing.total_units,
        last_advanced_at_utc: advancing.last_advanced_at_utc,
        observed_at_utc: observed_at.to_rfc3339(),
        provider_signature: Vec::new(),
        signature_algorithm: "ed25519".into(),
    };
    validate_workspace_coverage_sight_closing_authority(
        runtime_store,
        coverage,
        &entry.workspace_id,
        &entry.body_binding_sha256,
        &entry.body_observation_id,
        entry.body_generation,
        &entry.manifest_root_sha256,
    )?;
    sign_workspace_coverage_advancement_sight(&mut entry, provider)?;
    write_workspace_coverage_advancement_sight(local_verse_store, runtime_id, entry)
}

pub(crate) fn publish_workspace_coverage_terminal_sight(
    local_verse_store: &Path,
    runtime_store: &Path,
    coverage: &crate::WorkspaceCoverageAuthority,
    runtime_id: &str,
    launch_id: &str,
    trusted_host: &HostIncarnationIdentityEntry,
    provider: &SigningKey,
    observed_at: chrono::DateTime<chrono::Utc>,
) -> Result<WorkspaceCoverageTerminalSightEntry> {
    let terminal = crate::workspace_coverage_projector::authenticate_current_workspace_coverage_terminal_authority_with_store(
        runtime_store, &coverage.store,
    )?.ok_or_else(|| anyhow!("owned workspace coverage has no current terminal authority"))?;
    if terminal.managed_process_launch_id != launch_id {
        bail!("terminal sight launch disagrees with terminal coverage authority");
    }
    if let Some(existing) = authenticate_current_workspace_coverage_terminal_sight(
        runtime_store,
        local_verse_store,
        runtime_id,
        trusted_host,
    )? {
        if existing.launch_id == launch_id
            && existing.claim_id == terminal.claim_id
            && existing.claim_epoch == terminal.claim_epoch
            && existing.plan_id == terminal.plan_id
            && existing.receipt_id == terminal.receipt_id
        {
            return Ok(existing);
        }
        bail!("current terminal sight disagrees with owned terminal authority");
    }
    let opening = coverage.store.pull_all()?;
    let receipt_env = opening
        .iter()
        .find(|env| {
            env.r#type == "gamecult.epiphany.workspace_coverage_receipt" && {
                rmp_serde::from_slice::<
                        crate::workspace_retrieval_coverage::WorkspaceCoverageReceipt,
                    >(&env.payload)
                    .is_ok_and(|receipt| receipt.receipt_id == terminal.receipt_id)
            }
        })
        .ok_or_else(|| anyhow!("terminal sight receipt envelope is absent"))?;
    let head_env = opening
        .iter()
        .find(|env| {
            env.r#type == "gamecult.epiphany.workspace_coverage_head" && env.key == "current"
        })
        .ok_or_else(|| anyhow!("terminal sight head envelope is absent"))?;
    let (launch, launch_digest) =
        authenticate_workspace_coverage_managed_process_launch_with_envelope_digest(
            local_verse_store,
            runtime_id,
            launch_id,
            trusted_host,
        )?;
    let basis = crate::load_current_runtime_repository_body_basis(runtime_store)?;
    let mut entry = WorkspaceCoverageTerminalSightEntry {
        schema_version: WORKSPACE_COVERAGE_TERMINAL_SIGHT_SCHEMA_VERSION.into(),
        runtime_id: runtime_id.into(),
        workspace_id: basis.workspace_id,
        launch_id: launch_id.into(),
        launch_envelope_digest: launch_digest,
        provider_incarnation_id: launch.provider_incarnation_id,
        provider_public_key: launch.provider_public_key,
        coverage_store_binding_id: coverage.store_binding.binding_id.clone(),
        coverage_store_binding_envelope_digest: coverage.store_binding_envelope_sha256.clone(),
        coverage_store_file_identity: coverage.store_binding.store_file_identity.clone(),
        runtime_coverage_route_envelope_digest: coverage
            .runtime_coverage_route_envelope_sha256
            .clone(),
        body_binding_sha256: basis.body_binding_sha256,
        body_observation_id: terminal.body_observation_id,
        body_generation: terminal.body_generation,
        manifest_root_sha256: basis.manifest_root_sha256,
        claim_id: terminal.claim_id,
        claim_epoch: terminal.claim_epoch,
        plan_id: terminal.plan_id,
        receipt_id: terminal.receipt_id,
        receipt_envelope_digest: envelope_digest(receipt_env),
        head_id: "current".into(),
        head_envelope_digest: envelope_digest(head_env),
        sequence: terminal.claim_epoch,
        status: "succeeded".into(),
        observed_at_utc: observed_at.to_rfc3339(),
        provider_signature: Vec::new(),
        signature_algorithm: "ed25519".into(),
    };
    validate_workspace_coverage_sight_closing_authority(
        runtime_store,
        coverage,
        &entry.workspace_id,
        &entry.body_binding_sha256,
        &entry.body_observation_id,
        entry.body_generation,
        &entry.manifest_root_sha256,
    )?;
    sign_workspace_coverage_terminal_sight(&mut entry, provider)?;
    write_workspace_coverage_terminal_sight(local_verse_store, runtime_id, entry)
}

pub(crate) fn validate_workspace_coverage_sight_closing_authority(
    runtime_store: &Path,
    coverage: &crate::WorkspaceCoverageAuthority,
    expected_workspace_id: &str,
    expected_body_binding_sha256: &str,
    expected_body_observation_id: &str,
    expected_body_generation: u64,
    expected_manifest_root_sha256: &str,
) -> Result<()> {
    let closing_basis = crate::load_current_runtime_repository_body_basis(runtime_store)?;
    let closing_route = crate::runtime_workspace_coverage_store_binding(runtime_store)?
        .ok_or_else(|| anyhow!("runtime lost workspace coverage route before sight publication"))?;
    if closing_basis.workspace_id != expected_workspace_id
        || closing_basis.body_binding_sha256 != expected_body_binding_sha256
        || closing_basis.observation_id != expected_body_observation_id
        || closing_basis.generation != expected_body_generation
        || closing_basis.manifest_root_sha256 != expected_manifest_root_sha256
        || closing_route != coverage.runtime_coverage_route
    {
        bail!("Repository Body or coverage route moved before sight publication");
    }
    Ok(())
}

fn replace_latest_sight<T: DatabaseEntry>(
    cache: &cultcache_rs::CultCache,
    backing: &SingleFileMessagePackBackingStore,
    ty: &str,
    key: &str,
    entry: &T,
    opening: &[CultCacheEnvelope],
) -> Result<()> {
    let expected = opening
        .iter()
        .filter(|envelope| envelope.r#type == ty && envelope.key == key)
        .cloned()
        .collect::<Vec<_>>();
    if expected.len() > 1 {
        bail!("workspace coverage sight latest identity is duplicated");
    }
    let replacement = cache.prepare_entry(key, entry)?.0;
    if !backing.compare_and_swap_batch(&expected, vec![replacement])? {
        bail!("workspace coverage sight lost exact latest compare-and-swap");
    }
    Ok(())
}

pub fn authenticate_current_workspace_coverage_advancement_sight(
    runtime_store: impl AsRef<Path>,
    local_verse_store: impl AsRef<Path>,
    runtime_id: &str,
    launch_id: &str,
    host: &HostIncarnationIdentityEntry,
) -> Result<Option<WorkspaceCoverageAdvancementSightEntry>> {
    let basis = crate::load_current_runtime_repository_body_basis(runtime_store.as_ref())?;
    let local = SingleFileMessagePackBackingStore::new(local_verse_store.as_ref()).pull_all()?;
    let prefix = format!("epiphany-local/workspace-coverage/advancement-sight/{launch_id}/");
    let mut matches = Vec::new();
    for envelope in local.iter().filter(|envelope| {
        envelope.r#type == WORKSPACE_COVERAGE_ADVANCEMENT_SIGHT_TYPE
            && envelope.key.starts_with(&prefix)
    }) {
        let entry: WorkspaceCoverageAdvancementSightEntry =
            rmp_serde::from_slice(&envelope.payload)?;
        validate_advancement_sight_shape(&entry, true)?;
        if envelope.key != advancement_sight_key(&entry.launch_id, &entry.claim_id) {
            bail!("workspace coverage advancement sight key disagrees with signed identity");
        }
        if entry.body_observation_id == basis.observation_id
            && entry.body_generation == basis.generation
        {
            matches.push((envelope, entry));
        }
    }
    if matches.len() > 1 {
        bail!("workspace coverage launch has multiple current advancement sights");
    }
    let Some((_, entry)) = matches.first() else {
        return Ok(None);
    };
    let entry = entry.clone();
    authenticate_advancement_sight(
        &entry,
        runtime_store.as_ref(),
        local_verse_store.as_ref(),
        runtime_id,
        launch_id,
        host,
    )?;
    Ok(Some(entry))
}

pub fn authenticate_current_workspace_coverage_terminal_sight(
    runtime_store: impl AsRef<Path>,
    local_verse_store: impl AsRef<Path>,
    runtime_id: &str,
    host: &HostIncarnationIdentityEntry,
) -> Result<Option<WorkspaceCoverageTerminalSightEntry>> {
    let basis = crate::load_current_runtime_repository_body_basis(runtime_store.as_ref())?;
    let key = terminal_sight_key(runtime_id, basis.generation, &basis.observation_id);
    let local = SingleFileMessagePackBackingStore::new(local_verse_store.as_ref()).pull_all()?;
    let matches = local
        .iter()
        .filter(|envelope| {
            envelope.r#type == WORKSPACE_COVERAGE_TERMINAL_SIGHT_TYPE && envelope.key == key
        })
        .collect::<Vec<_>>();
    if matches.len() > 1 {
        bail!("workspace coverage terminal sight identity is duplicated");
    }
    let Some(envelope) = matches.first() else {
        return Ok(None);
    };
    let entry: WorkspaceCoverageTerminalSightEntry = rmp_serde::from_slice(&envelope.payload)?;
    authenticate_terminal_sight(
        &entry,
        runtime_store.as_ref(),
        local_verse_store.as_ref(),
        runtime_id,
        host,
    )?;
    Ok(Some(entry))
}

fn authenticate_advancement_sight(
    entry: &WorkspaceCoverageAdvancementSightEntry,
    runtime_store: &Path,
    local_verse_store: &Path,
    runtime_id: &str,
    launch_id: &str,
    host: &HostIncarnationIdentityEntry,
) -> Result<()> {
    validate_advancement_sight_shape(entry, true)?;
    if entry.runtime_id != runtime_id || entry.launch_id != launch_id {
        bail!("workspace coverage advancement sight identity disagrees");
    }
    authenticate_sight_common(
        runtime_store,
        local_verse_store,
        runtime_id,
        host,
        &entry.launch_id,
        &entry.launch_envelope_digest,
        &entry.provider_incarnation_id,
        &entry.provider_public_key,
        &entry.workspace_id,
        &entry.coverage_store_binding_id,
        &entry.coverage_store_binding_envelope_digest,
        &entry.coverage_store_file_identity,
        &entry.runtime_coverage_route_envelope_digest,
        &entry.body_binding_sha256,
        &entry.body_observation_id,
        entry.body_generation,
        &entry.manifest_root_sha256,
    )?;
    let mut unsigned = entry.clone();
    unsigned.provider_signature.clear();
    verify_sight_signature(
        COVERAGE_ADVANCEMENT_SIGHT_DOMAIN,
        &unsigned,
        &entry.provider_public_key,
        &entry.provider_signature,
    )
}

fn authenticate_terminal_sight(
    entry: &WorkspaceCoverageTerminalSightEntry,
    runtime_store: &Path,
    local_verse_store: &Path,
    runtime_id: &str,
    host: &HostIncarnationIdentityEntry,
) -> Result<()> {
    validate_terminal_sight_shape(entry, true)?;
    if entry.runtime_id != runtime_id {
        bail!("workspace coverage terminal sight identity disagrees");
    }
    authenticate_sight_common(
        runtime_store,
        local_verse_store,
        runtime_id,
        host,
        &entry.launch_id,
        &entry.launch_envelope_digest,
        &entry.provider_incarnation_id,
        &entry.provider_public_key,
        &entry.workspace_id,
        &entry.coverage_store_binding_id,
        &entry.coverage_store_binding_envelope_digest,
        &entry.coverage_store_file_identity,
        &entry.runtime_coverage_route_envelope_digest,
        &entry.body_binding_sha256,
        &entry.body_observation_id,
        entry.body_generation,
        &entry.manifest_root_sha256,
    )?;
    let mut unsigned = entry.clone();
    unsigned.provider_signature.clear();
    verify_sight_signature(
        COVERAGE_TERMINAL_SIGHT_DOMAIN,
        &unsigned,
        &entry.provider_public_key,
        &entry.provider_signature,
    )
}

#[allow(clippy::too_many_arguments)]
fn authenticate_sight_common(
    runtime_store: &Path,
    local_verse_store: &Path,
    runtime_id: &str,
    host: &HostIncarnationIdentityEntry,
    launch_id: &str,
    launch_digest: &str,
    provider_incarnation_id: &str,
    provider_public_key: &[u8],
    workspace_id: &str,
    binding_id: &str,
    binding_envelope_digest: &str,
    store_file_identity: &str,
    runtime_route_digest: &str,
    body_binding_sha256: &str,
    body_observation_id: &str,
    body_generation: u64,
    manifest_root_sha256: &str,
) -> Result<()> {
    let (launch, exact_launch_digest) =
        authenticate_workspace_coverage_managed_process_launch_with_envelope_digest(
            local_verse_store,
            runtime_id,
            launch_id,
            host,
        )?;
    if exact_launch_digest != launch_digest
        || launch.provider_incarnation_id != provider_incarnation_id
        || launch.provider_public_key != provider_public_key
    {
        bail!("workspace coverage sight disagrees with exact managed launch");
    }
    let basis = crate::load_current_runtime_repository_body_basis(runtime_store)?;
    if basis.runtime_id != runtime_id
        || basis.workspace_id != workspace_id
        || basis.body_binding_sha256 != body_binding_sha256
        || basis.observation_id != body_observation_id
        || basis.generation != body_generation
        || basis.manifest_root_sha256 != manifest_root_sha256
    {
        bail!("workspace coverage sight disagrees with current repository Body basis");
    }
    let route = crate::runtime_workspace_coverage_store_binding(runtime_store)?
        .ok_or_else(|| anyhow!("runtime has no workspace coverage store route"))?;
    if route.binding_id != binding_id
        || route.store_file_identity != store_file_identity
        || route.runtime_id != runtime_id
        || route.workspace_id != workspace_id
        || route.body_binding_sha256 != body_binding_sha256
    {
        bail!("workspace coverage sight disagrees with current coverage route");
    }
    let opening = SingleFileMessagePackBackingStore::new(runtime_store).pull_all()?;
    let route_env = opening
        .iter()
        .find(|env| {
            env.r#type == crate::RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_TYPE
                && env.key == crate::RUNTIME_WORKSPACE_COVERAGE_STORE_BINDING_KEY
        })
        .ok_or_else(|| anyhow!("runtime workspace coverage route envelope is absent"))?;
    let exact_route_digest = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(route_env)?));
    if exact_route_digest != runtime_route_digest {
        bail!("workspace coverage sight route envelope was substituted");
    }
    let body_route = crate::runtime_repository_body_store_binding(runtime_store)?
        .ok_or_else(|| anyhow!("runtime has no repository Body route"))?;
    let body_opening =
        SingleFileMessagePackBackingStore::new(&body_route.body_store_path).pull_all()?;
    let body_binding_env = body_opening
        .iter()
        .find(|env| env.r#type == crate::BODY_BINDING_TYPE && env.key == crate::BODY_BINDING_KEY)
        .ok_or_else(|| anyhow!("repository Body binding envelope is absent"))?;
    let body_binding: crate::RepositoryBodyBinding =
        rmp_serde::from_slice(&body_binding_env.payload)?;
    let local_binding = crate::WorkspaceCoverageStoreBinding {
        schema_version: crate::WORKSPACE_COVERAGE_STORE_BINDING_SCHEMA_VERSION.into(),
        binding_id: route.binding_id.clone(),
        runtime_id: route.runtime_id.clone(),
        swarm_id: route.swarm_id.clone(),
        workspace_id: route.workspace_id.clone(),
        store_file_identity: route.store_file_identity.clone(),
        body_binding_sha256: route.body_binding_sha256.clone(),
        repository_source_identity_sha256: body_binding.source_identity_sha256,
        projection_scope: "workspace_coverage".into(),
        storage_backend: "cultcache_redb_v0".into(),
        created_at_utc: route.created_at_utc.clone(),
    };
    let reconstructed = CultCacheEnvelope {
        key: crate::WORKSPACE_COVERAGE_STORE_BINDING_KEY.into(),
        r#type: crate::WORKSPACE_COVERAGE_STORE_BINDING_TYPE.into(),
        payload: rmp_serde::to_vec(&local_binding)?,
        stored_at: route.created_at_utc,
        schema_id: Some(crate::WORKSPACE_COVERAGE_STORE_BINDING_TYPE.into()),
    };
    let exact_binding_digest = format!(
        "{:x}",
        Sha256::digest(rmp_serde::to_vec_named(&reconstructed)?)
    );
    if exact_binding_digest != binding_envelope_digest {
        bail!("workspace coverage sight binding envelope was substituted");
    }
    Ok(())
}

fn verify_sight_signature<T: Serialize + Clone>(
    domain: &[u8],
    entry: &T,
    public_key: &[u8],
    signature: &[u8],
) -> Result<()> {
    let verifying = VerifyingKey::from_bytes(
        public_key
            .try_into()
            .map_err(|_| anyhow!("workspace coverage sight public key must be 32 bytes"))?,
    )?;
    let signature = Signature::from_slice(signature)?;
    verifying.verify(&signed_sight_statement(domain, entry)?, &signature)?;
    Ok(())
}

fn validate_advancement_sight_shape(
    entry: &WorkspaceCoverageAdvancementSightEntry,
    signed: bool,
) -> Result<()> {
    if entry.schema_version != WORKSPACE_COVERAGE_ADVANCEMENT_SIGHT_SCHEMA_VERSION
        || entry.status != "warming"
        || entry.sequence == 0
        || entry.claim_epoch == 0
        || entry.body_generation == 0
        || entry.completed_units == 0
        || entry.completed_units > entry.total_units
    {
        bail!("workspace coverage advancement sight shape is invalid");
    }
    validate_sight_strings(&[
        &entry.runtime_id,
        &entry.workspace_id,
        &entry.launch_id,
        &entry.launch_envelope_digest,
        &entry.provider_incarnation_id,
        &entry.coverage_store_binding_id,
        &entry.coverage_store_binding_envelope_digest,
        &entry.coverage_store_file_identity,
        &entry.runtime_coverage_route_envelope_digest,
        &entry.body_binding_sha256,
        &entry.body_observation_id,
        &entry.manifest_root_sha256,
        &entry.claim_id,
        &entry.plan_id,
        &entry.progress_id,
        &entry.progress_envelope_digest,
        &entry.checkpoint_id,
        &entry.checkpoint_envelope_digest,
        &entry.last_advanced_at_utc,
        &entry.observed_at_utc,
    ])?;
    DateTime::parse_from_rfc3339(&entry.last_advanced_at_utc)?;
    DateTime::parse_from_rfc3339(&entry.observed_at_utc)?;
    if entry.signature_algorithm != "ed25519"
        || entry.provider_public_key.len() != 32
        || (signed && entry.provider_signature.len() != 64)
        || (!signed && !entry.provider_signature.is_empty())
    {
        bail!("workspace coverage advancement sight signature shape is invalid");
    }
    Ok(())
}

fn validate_terminal_sight_shape(
    entry: &WorkspaceCoverageTerminalSightEntry,
    signed: bool,
) -> Result<()> {
    if entry.schema_version != WORKSPACE_COVERAGE_TERMINAL_SIGHT_SCHEMA_VERSION
        || entry.status != "succeeded"
        || entry.sequence == 0
        || entry.claim_epoch == 0
        || entry.body_generation == 0
        || entry.head_id != "current"
    {
        bail!("workspace coverage terminal sight shape is invalid");
    }
    validate_sight_strings(&[
        &entry.runtime_id,
        &entry.workspace_id,
        &entry.launch_id,
        &entry.launch_envelope_digest,
        &entry.provider_incarnation_id,
        &entry.coverage_store_binding_id,
        &entry.coverage_store_binding_envelope_digest,
        &entry.coverage_store_file_identity,
        &entry.runtime_coverage_route_envelope_digest,
        &entry.body_binding_sha256,
        &entry.body_observation_id,
        &entry.manifest_root_sha256,
        &entry.claim_id,
        &entry.plan_id,
        &entry.receipt_id,
        &entry.receipt_envelope_digest,
        &entry.head_envelope_digest,
        &entry.observed_at_utc,
    ])?;
    DateTime::parse_from_rfc3339(&entry.observed_at_utc)?;
    if entry.signature_algorithm != "ed25519"
        || entry.provider_public_key.len() != 32
        || (signed && entry.provider_signature.len() != 64)
        || (!signed && !entry.provider_signature.is_empty())
    {
        bail!("workspace coverage terminal sight signature shape is invalid");
    }
    Ok(())
}

fn validate_sight_strings(values: &[&str]) -> Result<()> {
    if values.iter().any(|value| value.trim().is_empty()) {
        bail!("workspace coverage sight has an empty identity or digest");
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
fn heartbeat_latest_key(launch_id: &str) -> String {
    format!("{WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_LATEST_KEY}{launch_id}")
}
fn heartbeat_evidence_key(launch_id: &str, heartbeat_id: &str) -> String {
    format!("epiphany-local/workspace-coverage/provider-heartbeat/{launch_id}/{heartbeat_id}")
}

fn latest_heartbeat_envelope_by_id(
    store_path: impl AsRef<Path>,
    runtime_id: &str,
    heartbeat_id: &str,
) -> Result<Option<CultCacheEnvelope>> {
    let _node = open_epiphany_cultmesh_node(store_path.as_ref(), runtime_id.to_string())?;
    let opening = SingleFileMessagePackBackingStore::new(store_path.as_ref()).pull_all()?;
    let matches = opening
        .iter()
        .filter(|envelope| {
            envelope.r#type == WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_TYPE
                && rmp_serde::from_slice::<WorkspaceCoverageProviderHeartbeatEntry>(
                    &envelope.payload,
                )
                .is_ok_and(|heartbeat| {
                    heartbeat.runtime_id == runtime_id && heartbeat.heartbeat_id == heartbeat_id
                })
        })
        .collect::<Vec<_>>();
    let immutable = matches
        .into_iter()
        .filter(|envelope| {
            !envelope
                .key
                .starts_with(WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_LATEST_KEY)
        })
        .collect::<Vec<_>>();
    if immutable.len() != 1 {
        bail!("workspace coverage heartbeat immutable evidence is absent or duplicated");
    }
    Ok(Some(immutable[0].clone()))
}
fn termination_key(launch_id: &str) -> String {
    format!("epiphany-local/workspace-coverage/process-termination/{launch_id}")
}
fn process_evidence_head_key(launch_id: &str) -> String {
    format!("epiphany-local/workspace-coverage/process-evidence-head/{launch_id}")
}

fn validate_process_evidence_head(
    head: &WorkspaceCoverageProcessEvidenceHead,
    launch_id: &str,
) -> Result<()> {
    if head.schema_version != WORKSPACE_COVERAGE_PROCESS_EVIDENCE_HEAD_SCHEMA_VERSION
        || head.launch_id != launch_id
        || head.generation == 0
    {
        bail!("workspace coverage process evidence head is invalid");
    }
    match head.state.as_str() {
        "launched"
            if head.generation == 1
                && head.heartbeat_id.is_none()
                && head.termination_id.is_none() => {}
        "heartbeat" if head.heartbeat_id.is_some() && head.termination_id.is_none() => {}
        "terminated" if head.termination_id.is_some() => {}
        _ => bail!("workspace coverage process evidence head state is incoherent"),
    }
    Ok(())
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
    use crate::{
        EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION, enroll_host_identity_at,
        write_epiphany_cultmesh_workspace_coverage_projector_service_policy,
    };
    use cultcache_rs::CacheBackingStore;
    use rand_core::{OsRng, RngCore};
    use std::process::Command;
    use uuid::Uuid;

    struct FakeObservation {
        boot: Option<String>,
        process: ProcessInstanceObservation,
    }
    impl WorkspaceCoverageProcessObservationSource for FakeObservation {
        fn boot_identity(&self) -> Option<String> {
            self.boot.clone()
        }
        fn observe(&self, _expected: &ProcessInstanceIdentity) -> ProcessInstanceObservation {
            self.process.clone()
        }
    }

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
                "--heartbeat-interval-seconds",
                "10",
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
            replaces_launch_id: None,
            replaces_termination_id: None,
            replaces_termination_envelope_digest: None,
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

    fn persisted_chain(
        root: &Path,
    ) -> Result<(
        std::path::PathBuf,
        HostIdentitySigner,
        WorkspaceCoverageManagedProcessLaunchEntry,
        SigningKey,
    )> {
        let store = root.join("verse.ccmp");
        let host = enroll_host_identity_at(&root.join("host.ccmp"))?;
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
        let pulse = heartbeat(&launch, envelope_digest(&launch_envelope), &provider, 1)?;
        write_workspace_coverage_provider_heartbeat(&store, "local", pulse)?;
        Ok((store, host, launch, provider))
    }

    fn advancement_sight(
        provider: &SigningKey,
        sequence: u64,
    ) -> Result<WorkspaceCoverageAdvancementSightEntry> {
        let mut entry = WorkspaceCoverageAdvancementSightEntry {
            schema_version: WORKSPACE_COVERAGE_ADVANCEMENT_SIGHT_SCHEMA_VERSION.into(),
            runtime_id: "local".into(),
            workspace_id: "workspace".into(),
            launch_id: Uuid::new_v4().to_string(),
            launch_envelope_digest: "launch-digest".into(),
            provider_incarnation_id: Uuid::new_v4().to_string(),
            provider_public_key: provider.verifying_key().to_bytes().to_vec(),
            coverage_store_binding_id: "binding".into(),
            coverage_store_binding_envelope_digest: "binding-envelope".into(),
            coverage_store_file_identity: "file-identity".into(),
            runtime_coverage_route_envelope_digest: "route-envelope".into(),
            body_binding_sha256: "body-binding".into(),
            body_observation_id: "observation".into(),
            body_generation: 7,
            manifest_root_sha256: "manifest".into(),
            claim_id: "claim".into(),
            claim_epoch: 3,
            plan_id: "plan".into(),
            progress_id: Uuid::new_v4().to_string(),
            progress_envelope_digest: "progress-envelope".into(),
            checkpoint_id: Uuid::new_v4().to_string(),
            checkpoint_envelope_digest: "checkpoint-envelope".into(),
            sequence,
            status: "warming".into(),
            completed_units: sequence,
            total_units: 10,
            last_advanced_at_utc: chrono::Utc::now().to_rfc3339(),
            observed_at_utc: chrono::Utc::now().to_rfc3339(),
            provider_signature: Vec::new(),
            signature_algorithm: "ed25519".into(),
        };
        sign_workspace_coverage_advancement_sight(&mut entry, provider)?;
        Ok(entry)
    }

    #[test]
    fn advancement_sight_signature_refuses_substitution_and_storage_is_latest_only() -> Result<()> {
        let provider = provider_key();
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("verse.ccmp");
        let first = advancement_sight(&provider, 1)?;
        write_workspace_coverage_advancement_sight(&store, "local", first.clone())?;
        let mut second = advancement_sight(&provider, 2)?;
        second.launch_id = first.launch_id.clone();
        second.provider_incarnation_id = first.provider_incarnation_id.clone();
        second.claim_id = first.claim_id.clone();
        sign_workspace_coverage_advancement_sight(&mut second, &provider)?;
        write_workspace_coverage_advancement_sight(&store, "local", second.clone())?;
        let sights = SingleFileMessagePackBackingStore::new(&store)
            .pull_all()?
            .into_iter()
            .filter(|entry| entry.r#type == WORKSPACE_COVERAGE_ADVANCEMENT_SIGHT_TYPE)
            .collect::<Vec<_>>();
        assert_eq!(
            sights.len(),
            1,
            "advancement sight must replace its exact latest projection"
        );
        assert_eq!(
            rmp_serde::from_slice::<WorkspaceCoverageAdvancementSightEntry>(&sights[0].payload)?,
            second
        );

        let mut regression = first.clone();
        regression.launch_id = second.launch_id.clone();
        regression.provider_incarnation_id = second.provider_incarnation_id.clone();
        regression.claim_id = second.claim_id.clone();
        sign_workspace_coverage_advancement_sight(&mut regression, &provider)?;
        assert!(write_workspace_coverage_advancement_sight(&store, "local", regression).is_err());

        let mut substituted = second.clone();
        substituted.checkpoint_envelope_digest = "alien-checkpoint".into();
        let mut unsigned = substituted.clone();
        unsigned.provider_signature.clear();
        assert!(
            verify_sight_signature(
                COVERAGE_ADVANCEMENT_SIGHT_DOMAIN,
                &unsigned,
                &substituted.provider_public_key,
                &substituted.provider_signature,
            )
            .is_err()
        );
        Ok(())
    }

    #[test]
    fn first_checkpoint_survives_owned_store_reopen_and_resumes_to_terminal_without_body_copies()
    -> Result<()> {
        let temp = tempfile::tempdir()?;
        let verse = temp.path().join("verse.ccmp");
        let host = enroll_host_identity_at(&temp.path().join("host.ccmp"))?;
        let policy = policy()?;
        let mut node = open_epiphany_cultmesh_node(&verse, "local")?;
        node.put(managed_policy_key(), &policy)?;
        let policy_envelope = node
            .cache()
            .get_envelope::<EpiphanyCultMeshManagedServicePolicyEntry>(&managed_policy_key())?
            .context("test policy envelope absent")?;
        let provider = provider_key();
        let launch = launch(&policy, envelope_digest(&policy_envelope), &host, &provider)?;
        write_workspace_coverage_managed_process_launch(
            &verse,
            "local",
            launch.clone(),
            host.entry(),
        )?;

        let repo = tempfile::tempdir()?;
        let initialized = std::process::Command::new("git")
            .arg("init")
            .current_dir(repo.path())
            .output()?;
        if !initialized.status.success() {
            bail!("resume fixture failed to initialize repository");
        }
        std::fs::write(repo.path().join("source.rs"), "fn resumed() {}\n")?;
        let runtime = temp.path().join("runtime.cc");
        let agents = temp.path().join("agents.cc");
        let body_store = temp.path().join("body.cc");
        let coverage_path = std::fs::canonicalize(temp.path())?.join("coverage.redb");
        crate::initialize_runtime_spine(
            &runtime,
            crate::RuntimeSpineInitOptions {
                runtime_id: "local".into(),
                display_name: "checkpoint resume".into(),
                created_at: "2026-07-17T00:00:00Z".into(),
            },
        )?;
        crate::ensure_agent_memory_swarm_identity(&agents, "resume-swarm")?;
        crate::bind_runtime_to_agent_memory_swarm(&runtime, &agents, "2026-07-17T00:00:01Z")?;
        crate::bind_repository_body(repo.path(), &body_store, &runtime, "resume-workspace")?;
        crate::bind_runtime_workspace_coverage_store(
            &runtime,
            &coverage_path,
            "2026-07-17T00:00:02Z",
        )?;
        let basis = crate::observe_runtime_repository_body_basis(&runtime)?;
        let session = crate::RepositoryBodyReadSession::open(&runtime, &basis)?;
        let prepared = crate::workspace_coverage_projector::prepare_workspace_coverage_projection(
            &session,
            "test-provider",
            "test-model",
            3,
        )?;
        let authority = crate::open_workspace_coverage_authority(&runtime)?;
        let acquisition =
            match crate::workspace_coverage_projector::acquire_workspace_coverage_projection(
                &prepared,
                &authority.store,
                EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID,
                &launch.provider_incarnation_id,
                &launch.launch_id,
            )? {
                crate::workspace_coverage_projector::WorkspaceCoverageAcquireResult::Acquired(
                    value,
                ) => value,
                _ => bail!("resume fixture did not acquire projection"),
            };
        let body_before = std::fs::read(&body_store)?;
        crate::workspace_coverage_projection_progress::publish_workspace_coverage_progress_genesis(
            &body_store,
            &authority.store,
            &verse,
            "local",
            host.entry(),
            &provider,
            30_000,
        )?;
        let vector = vec![0.25_f32; 3];
        let mut point_bindings = Vec::new();
        let mut vector_bindings = Vec::new();
        let mut stored = Vec::new();
        for point in &acquisition.plan.planned_points {
            let payload = crate::workspace_coverage_projector::payload_for(
                &acquisition.obligation,
                &acquisition.plan,
                point,
            );
            point_bindings.push(crate::WorkspaceCoveragePointBinding {
                point_id: point.point_id.clone(),
                payload_sha256: format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&payload)?)),
            });
            vector_bindings.push(crate::WorkspaceCoverageVectorBinding {
                point_id: point.point_id.clone(),
                vector_sha256: format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&vector)?)),
            });
            stored.push(crate::semantic_backend::SemanticStoredPoint {
                id: point.point_id.clone(),
                payload: Some(payload),
                vector: Some(vector.clone()),
            });
        }
        crate::workspace_coverage_projection_batch_checkpoint::admit_observed_workspace_coverage_batch(
            &body_store,
            &authority.store,
            &verse,
            "local",
            host.entry(),
            &provider,
            crate::workspace_coverage_projection_batch_checkpoint::ObservedWorkspaceCoverageBatchInput {
                claim_id: acquisition.claim.claim_id.clone(),
                attempt_id: acquisition.attempt.attempt_id.clone(),
                plan_id: acquisition.plan.plan_id.clone(),
                first_plan_ordinal: 0,
                point_bindings,
                vector_bindings,
            },
        )?;
        let claim_id = acquisition.claim.claim_id.clone();
        let claim_epoch = acquisition.claim.claim_epoch;
        drop(acquisition);
        drop(authority);

        let reopened = crate::open_workspace_coverage_authority(&runtime)?;
        let chain = crate::workspace_coverage_projection_batch_checkpoint::load_authenticated_checkpoint_chain(
            &body_store,
            &reopened.store,
            &verse,
            host.entry(),
            &claim_id,
            claim_epoch,
        )?;
        assert_eq!(chain.len(), 1);
        assert!(
            crate::authenticate_current_workspace_coverage_advancement(
                &body_store,
                &reopened.store,
                &verse,
                "local",
                &launch.launch_id,
                host.entry(),
                chrono::Utc::now(),
                chrono::Duration::minutes(5),
            )?
            .is_some()
        );
        let collection = crate::workspace_coverage_execution_collection(
            &prepared.plan.plan_id,
            &claim_id,
            claim_epoch,
        )?;
        let obligation = prepared.obligation.clone();
        let plan = prepared.plan.clone();
        let observed = crate::workspace_coverage_projector::observe_final_projection_against_authenticated_checkpoint_chain(
            &obligation,
            &plan,
            &collection,
            stored,
            &chain,
        )?;
        let resumed =
            match crate::workspace_coverage_projector::acquire_workspace_coverage_projection(
                &prepared,
                &reopened.store,
                EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID,
                &launch.provider_incarnation_id,
                &launch.launch_id,
            )? {
                crate::workspace_coverage_projector::WorkspaceCoverageAcquireResult::Acquired(
                    value,
                ) => value,
                _ => bail!("reopened exact executor did not resume claim"),
            };
        crate::workspace_coverage_projector::commit_workspace_coverage_success(&resumed, observed)?;
        assert_eq!(std::fs::read(&body_store)?, body_before);
        assert!(reopened.store.pull_all()?.iter().all(|entry| !matches!(
            entry.r#type.as_str(),
            crate::BODY_BINDING_TYPE
                | crate::BODY_HEAD_TYPE
                | crate::BODY_OBSERVATION_TYPE
                | crate::BODY_MANIFEST_TYPE
        )));
        Ok(())
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
        let mut second = heartbeat(&launch, envelope_digest(&launch_envelope), &provider, 2)?;
        second.observed_at_utc = (DateTime::parse_from_rfc3339(&first.observed_at_utc)?
            + chrono::Duration::seconds(1))
        .to_rfc3339();
        sign_workspace_coverage_heartbeat(&mut second, &provider)?;
        write_workspace_coverage_provider_heartbeat(&store, "local", second.clone())?;
        assert_eq!(
            load_workspace_coverage_provider_heartbeat(&store, "local", &second.heartbeat_id)?,
            Some(second)
        );
        assert_eq!(
            load_workspace_coverage_provider_heartbeat(&store, "local", &first.heartbeat_id)?,
            Some(first.clone())
        );
        assert_eq!(
            SingleFileMessagePackBackingStore::new(&store)
                .pull_all()?
                .into_iter()
                .filter(|entry| entry.r#type == WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_TYPE)
                .count(),
            3,
            "latest is a derived head over immutable heartbeat evidence"
        );

        let mut forged = first.clone();
        forged.heartbeat_id = Uuid::new_v4().to_string();
        forged.status = "degraded".to_string();
        assert!(write_workspace_coverage_provider_heartbeat(&store, "local", forged).is_err());

        let gap = heartbeat(&launch, envelope_digest(&launch_envelope), &provider, 4)?;
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

    #[test]
    fn termination_observation_accepts_only_exact_native_proofs_and_is_immutable() -> Result<()> {
        let cases = vec![
            (
                "exact_exited",
                ProcessInstanceObservation::ExactExited { exit_code: Some(7) },
            ),
            ("process_missing", ProcessInstanceObservation::Missing),
            (
                "process_replaced",
                ProcessInstanceObservation::Replaced {
                    observed: ProcessInstanceIdentity {
                        process_id: 777,
                        creation_token: 888,
                        created_at_rfc3339: None,
                        executable_path: std::fs::canonicalize(std::env::current_exe()?)?,
                    },
                },
            ),
        ];
        for (expected_outcome, process) in cases {
            let temp = tempfile::tempdir()?;
            let (store, host, launch, _provider) = persisted_chain(temp.path())?;
            let source = FakeObservation {
                boot: Some(launch.boot_identity.clone()),
                process,
            };
            let proof = write_workspace_coverage_process_termination_observation_with_source(
                &store,
                "local",
                &launch.launch_id,
                &host,
                &source,
            )?;
            assert_eq!(proof.outcome, expected_outcome);
            assert_eq!(
                authenticate_workspace_coverage_process_termination_observation(
                    &store,
                    "local",
                    &launch.launch_id,
                    host.entry(),
                )?,
                proof
            );
            assert!(
                write_workspace_coverage_process_termination_observation_with_source(
                    &store,
                    "local",
                    &launch.launch_id,
                    &host,
                    &source,
                )
                .expect_err("termination key is immutable")
                .to_string()
                .contains("already has terminal evidence")
            );
            let mut advanced_policy = policy()?;
            advanced_policy.updated_at_utc = "2026-07-16T23:59:59Z".to_string();
            write_epiphany_cultmesh_workspace_coverage_projector_service_policy(
                &store,
                "local",
                advanced_policy,
            )?;
            assert!(
                authenticate_workspace_coverage_process_termination_observation(
                    &store,
                    "local",
                    &launch.launch_id,
                    host.entry(),
                )
                .expect_err("moved policy source invalidates exact termination chain")
                .to_string()
                .contains("disagrees")
            );
        }

        let temp = tempfile::tempdir()?;
        let (store, host, launch, _provider) = persisted_chain(temp.path())?;
        let source = FakeObservation {
            boot: Some("proved-new-boot".to_string()),
            process: ProcessInstanceObservation::ExactAlive,
        };
        let proof = write_workspace_coverage_process_termination_observation_with_source(
            &store,
            "local",
            &launch.launch_id,
            &host,
            &source,
        )?;
        assert_eq!(proof.outcome, "boot_superseded");
        assert_ne!(proof.expected_boot_identity, proof.observed_boot_identity);
        Ok(())
    }

    #[test]
    fn termination_observation_refuses_missing_boot_alive_and_uncertain_processes() -> Result<()> {
        let cases = vec![
            FakeObservation {
                boot: None,
                process: ProcessInstanceObservation::Missing,
            },
            FakeObservation {
                boot: Some("test-boot-incarnation".into()),
                process: ProcessInstanceObservation::ExactAlive,
            },
            FakeObservation {
                boot: Some("test-boot-incarnation".into()),
                process: ProcessInstanceObservation::Inaccessible,
            },
            FakeObservation {
                boot: Some("test-boot-incarnation".into()),
                process: ProcessInstanceObservation::Indeterminate {
                    reason: "host lied".into(),
                },
            },
        ];
        for source in cases {
            let temp = tempfile::tempdir()?;
            let (store, host, launch, _provider) = persisted_chain(temp.path())?;
            assert!(
                write_workspace_coverage_process_termination_observation_with_source(
                    &store,
                    "local",
                    &launch.launch_id,
                    &host,
                    &source,
                )
                .is_err()
            );
            assert!(
                load_workspace_coverage_process_termination_observation(
                    &store,
                    "local",
                    &launch.launch_id,
                )?
                .is_none()
            );
        }
        Ok(())
    }

    #[test]
    fn typed_process_observation_never_writes_termination_state() -> Result<()> {
        let cases = vec![
            (
                ProcessInstanceObservation::ExactAlive,
                WorkspaceCoverageProcessLifecycleObservation::ExactAlive,
            ),
            (
                ProcessInstanceObservation::Missing,
                WorkspaceCoverageProcessLifecycleObservation::Missing,
            ),
            (
                ProcessInstanceObservation::Inaccessible,
                WorkspaceCoverageProcessLifecycleObservation::Inaccessible,
            ),
            (
                ProcessInstanceObservation::Indeterminate {
                    reason: "ambiguous kernel sight".into(),
                },
                WorkspaceCoverageProcessLifecycleObservation::Indeterminate {
                    reason: "ambiguous kernel sight".into(),
                },
            ),
        ];
        for (process, expected) in cases {
            let temp = tempfile::tempdir()?;
            let (store, _host, launch, _provider) = persisted_chain(temp.path())?;
            let observed = observe_workspace_coverage_process_with_source(
                &launch,
                &FakeObservation {
                    boot: Some(launch.boot_identity.clone()),
                    process,
                },
            );
            assert_eq!(observed, expected);
            assert!(
                load_workspace_coverage_process_termination_observation(
                    &store,
                    "local",
                    &launch.launch_id,
                )?
                .is_none()
            );
        }
        Ok(())
    }

    #[test]
    fn public_process_observation_refuses_stale_launch_policy_substitution() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let (store, host, launch, _provider) = persisted_chain(temp.path())?;
        let mut moved_policy = policy()?;
        moved_policy.updated_at_utc = "2026-07-17T00:00:01Z".into();
        write_epiphany_cultmesh_workspace_coverage_projector_service_policy(
            &store,
            "local",
            moved_policy,
        )?;
        assert!(
            observe_workspace_coverage_managed_process(
                &store,
                "local",
                &launch.launch_id,
                host.entry(),
            )
            .expect_err("stale launch must not be observed as current authority")
            .to_string()
            .contains("disagrees with current managed policy")
        );
        assert!(
            load_workspace_coverage_process_termination_observation(
                &store,
                "local",
                &launch.launch_id,
            )?
            .is_none()
        );
        Ok(())
    }

    #[test]
    fn termination_before_first_heartbeat_seals_the_process_evidence_head() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("verse.ccmp");
        let host = enroll_host_identity_at(&temp.path().join("host.ccmp"))?;
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
        let source = FakeObservation {
            boot: Some(launch.boot_identity.clone()),
            process: ProcessInstanceObservation::Missing,
        };
        let termination = write_workspace_coverage_process_termination_observation_with_source(
            &store,
            "local",
            &launch.launch_id,
            &host,
            &source,
        )?;
        assert!(termination.heartbeat_id.is_none());
        assert!(termination.heartbeat_envelope_digest.is_none());
        authenticate_workspace_coverage_process_termination_observation(
            &store,
            "local",
            &launch.launch_id,
            host.entry(),
        )?;

        let launch_envelope = open_epiphany_cultmesh_node(&store, "local")?
            .cache()
            .get_envelope::<WorkspaceCoverageManagedProcessLaunchEntry>(&launch_key(
                &launch.launch_id,
            ))?
            .context("test launch envelope absent")?;
        let pulse = heartbeat(&launch, envelope_digest(&launch_envelope), &provider, 1)?;
        assert!(
            write_workspace_coverage_provider_heartbeat(&store, "local", pulse).is_err(),
            "a late heartbeat must not resurrect a terminated launch"
        );
        Ok(())
    }

    #[test]
    fn body_recovery_is_one_exact_evidence_fenced_transaction() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let (verse, host, old_launch, old_provider) = persisted_chain(temp.path())?;

        let repo = temp.path().join("repo");
        std::fs::create_dir_all(&repo)?;
        Command::new("git")
            .args(["init"])
            .current_dir(&repo)
            .output()?;
        std::fs::write(repo.join("source.rs"), "fn awake() {}\n")?;
        Command::new("git")
            .args(["add", "."])
            .current_dir(&repo)
            .output()?;
        let runtime = temp.path().join("runtime.ccmp");
        let agents = temp.path().join("agents.ccmp");
        let body_store = temp.path().join("body.ccmp");
        crate::initialize_runtime_spine(
            &runtime,
            crate::RuntimeSpineInitOptions {
                runtime_id: "local".into(),
                display_name: "recovery-test".into(),
                created_at: "2026-07-16T00:00:00Z".into(),
            },
        )?;
        crate::ensure_agent_memory_swarm_identity(&agents, "recovery-swarm")?;
        crate::bind_runtime_to_agent_memory_swarm(&runtime, &agents, "2026-07-16T00:00:01Z")?;
        crate::bind_repository_body(&repo, &body_store, &runtime, "recovery-workspace")?;
        let coverage_store_path = std::fs::canonicalize(temp.path())?.join("coverage.redb");
        crate::bind_runtime_workspace_coverage_store(
            &runtime,
            &coverage_store_path,
            "2026-07-16T00:00:02Z",
        )?;
        let coverage = crate::open_workspace_coverage_authority(&runtime)?;
        let basis = crate::observe_runtime_repository_body_basis(&runtime)?;
        let session = crate::RepositoryBodyReadSession::open(&runtime, &basis)?;
        let prepared = crate::workspace_coverage_projector::prepare_workspace_coverage_projection(
            &session,
            "test-provider",
            "test-model",
            3,
        )?;
        let acquired =
            match crate::workspace_coverage_projector::acquire_workspace_coverage_projection(
                &prepared,
                &coverage.store,
                EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID,
                &old_launch.provider_incarnation_id,
                &old_launch.launch_id,
            )? {
                crate::workspace_coverage_projector::WorkspaceCoverageAcquireResult::Acquired(
                    value,
                ) => value,
                _ => bail!("fixture did not acquire old claim"),
            };
        publish_workspace_coverage_claim_sight(
            &verse,
            &runtime,
            &coverage,
            &acquired,
            "local",
            &old_launch.launch_id,
            host.entry(),
            &old_provider,
            chrono::Utc::now(),
        )?;
        assert!(
            authenticate_current_workspace_coverage_claim_sight(
                &verse,
                &runtime,
                "local",
                host.entry(),
            )?
            .is_some()
        );

        let source = FakeObservation {
            boot: Some(old_launch.boot_identity.clone()),
            process: ProcessInstanceObservation::Missing,
        };
        let termination = write_workspace_coverage_process_termination_observation_with_source(
            &verse,
            "local",
            &old_launch.launch_id,
            &host,
            &source,
        )?;
        let (_, termination_digest) =
            authenticate_workspace_coverage_termination_with_envelope_digest(
                &verse,
                "local",
                &old_launch.launch_id,
                host.entry(),
            )?;
        let old_claim_id = acquired.claim.claim_id.clone();
        let old_claim_epoch = acquired.claim.claim_epoch;
        drop(acquired);
        drop(coverage);

        let node = open_epiphany_cultmesh_node(&verse, "local")?;
        let policy_envelope = node
            .cache()
            .get_envelope::<EpiphanyCultMeshManagedServicePolicyEntry>(&managed_policy_key())?
            .context("policy missing")?;
        let policy: EpiphanyCultMeshManagedServicePolicyEntry =
            rmp_serde::from_slice(&policy_envelope.payload)?;
        let body_before_refusal = std::fs::read(&body_store)?;
        let out_of_order_provider = provider_key();
        let mut out_of_order = launch(
            &policy,
            envelope_digest(&policy_envelope),
            &host,
            &out_of_order_provider,
        )?;
        out_of_order.launched_at_utc = termination.observed_at_utc.clone();
        out_of_order.identity_captured_at_utc = termination.observed_at_utc.clone();
        sign_workspace_coverage_launch(&mut out_of_order, &host)?;
        write_workspace_coverage_managed_process_launch(
            &verse,
            "local",
            out_of_order.clone(),
            host.entry(),
        )?;
        let out_of_order_envelope = open_epiphany_cultmesh_node(&verse, "local")?
            .cache()
            .get_envelope::<WorkspaceCoverageManagedProcessLaunchEntry>(&launch_key(
                &out_of_order.launch_id,
            ))?
            .context("out-of-order launch missing")?;
        let out_of_order_ready = heartbeat(
            &out_of_order,
            envelope_digest(&out_of_order_envelope),
            &out_of_order_provider,
            1,
        )?;
        write_workspace_coverage_provider_heartbeat(&verse, "local", out_of_order_ready.clone())?;
        assert!(
            crate::workspace_coverage_projector::recover_workspace_coverage_projection_with_authority(
                &crate::open_workspace_coverage_authority(&runtime)?,
                &runtime,
                &verse,
                "local",
                host.entry(),
                &old_launch.launch_id,
                &out_of_order.launch_id,
                &out_of_order_ready.heartbeat_id,
                &old_claim_id,
            )
            .is_err(),
            "unbound replacement must be refused"
        );
        assert_eq!(std::fs::read(&body_store)?, body_before_refusal);

        let replacement_provider = provider_key();
        let mut replacement = launch(
            &policy,
            envelope_digest(&policy_envelope),
            &host,
            &replacement_provider,
        )?;
        replacement.replaces_launch_id = Some(old_launch.launch_id.clone());
        replacement.replaces_termination_id = Some(termination.termination_id.clone());
        replacement.replaces_termination_envelope_digest = Some(termination_digest);
        sign_workspace_coverage_launch(&mut replacement, &host)?;
        write_workspace_coverage_managed_process_launch(
            &verse,
            "local",
            replacement.clone(),
            host.entry(),
        )?;
        let competing_provider = provider_key();
        let mut competing = launch(
            &policy,
            envelope_digest(&policy_envelope),
            &host,
            &competing_provider,
        )?;
        competing.replaces_launch_id = Some(old_launch.launch_id.clone());
        competing.replaces_termination_id = Some(termination.termination_id.clone());
        competing.replaces_termination_envelope_digest =
            replacement.replaces_termination_envelope_digest.clone();
        sign_workspace_coverage_launch(&mut competing, &host)?;
        assert!(
            write_workspace_coverage_managed_process_launch(
                &verse,
                "local",
                competing,
                host.entry(),
            )
            .is_err(),
            "one termination must authorize at most one replacement launch"
        );
        let replacement_envelope = open_epiphany_cultmesh_node(&verse, "local")?
            .cache()
            .get_envelope::<WorkspaceCoverageManagedProcessLaunchEntry>(&launch_key(
                &replacement.launch_id,
            ))?
            .context("replacement launch missing")?;
        let initial_ready = heartbeat(
            &replacement,
            envelope_digest(&replacement_envelope),
            &replacement_provider,
            1,
        )?;
        write_workspace_coverage_provider_heartbeat(&verse, "local", initial_ready)?;

        let body_before_refusal = std::fs::read(&body_store)?;
        let mut degraded = heartbeat(
            &replacement,
            envelope_digest(&replacement_envelope),
            &replacement_provider,
            2,
        )?;
        degraded.status = "degraded".into();
        sign_workspace_coverage_heartbeat(&mut degraded, &replacement_provider)?;
        write_workspace_coverage_provider_heartbeat(&verse, "local", degraded.clone())?;
        assert!(
            crate::workspace_coverage_projector::recover_workspace_coverage_projection_with_authority(
                &crate::open_workspace_coverage_authority(&runtime)?,
                &runtime,
                &verse,
                "local",
                host.entry(),
                &old_launch.launch_id,
                &replacement.launch_id,
                &degraded.heartbeat_id,
                &old_claim_id,
            )
            .is_err(),
            "degraded replacement must not inherit Body authority"
        );
        assert_eq!(std::fs::read(&body_store)?, body_before_refusal);

        let ready = heartbeat(
            &replacement,
            envelope_digest(&replacement_envelope),
            &replacement_provider,
            3,
        )?;
        write_workspace_coverage_provider_heartbeat(&verse, "local", ready.clone())?;
        let later_ready = heartbeat(
            &replacement,
            envelope_digest(&replacement_envelope),
            &replacement_provider,
            4,
        )?;
        write_workspace_coverage_provider_heartbeat(&verse, "local", later_ready)?;
        let directive = write_workspace_coverage_recovery_directive(
            &verse,
            &runtime,
            "local",
            &authenticate_current_workspace_coverage_claim_sight(
                &verse,
                &runtime,
                "local",
                host.entry(),
            )?
            .context("current claim sight missing before recovery directive")?,
            &replacement.launch_id,
            &ready.heartbeat_id,
            &host,
        )?;
        assert!(
            authenticate_workspace_coverage_recovery_directive(
                &verse,
                &runtime,
                "local",
                &replacement.launch_id,
                host.entry(),
            )?
            .is_some()
        );
        let mut conflicting_lineage = authenticate_current_workspace_coverage_claim_sight(
            &verse,
            &runtime,
            "local",
            host.entry(),
        )?
        .context("claim sight missing for conflict check")?;
        conflicting_lineage.claim_id = uuid::Uuid::new_v4().to_string();
        assert!(
            write_workspace_coverage_recovery_directive(
                &verse,
                &runtime,
                "local",
                &conflicting_lineage,
                &replacement.launch_id,
                &ready.heartbeat_id,
                &host,
            )
            .is_err(),
            "a replacement directive must refuse conflicting claim lineage"
        );

        let recovered = crate::workspace_coverage_projector::recover_workspace_coverage_projection_with_authority(
            &crate::open_workspace_coverage_authority(&runtime)?,
            &runtime,
            &verse,
            "local",
            host.entry(),
            &old_launch.launch_id,
            &replacement.launch_id,
            &ready.heartbeat_id,
            &old_claim_id,
        )?;
        assert_eq!(recovered.claim_epoch, old_claim_epoch + 1);
        assert_eq!(recovered.managed_process_launch_id, replacement.launch_id);
        assert_eq!(
            recovered.executor_incarnation,
            replacement.provider_incarnation_id
        );
        let reissued = write_workspace_coverage_recovery_directive(
            &verse,
            &runtime,
            "local",
            &authenticate_current_workspace_coverage_claim_sight(
                &verse,
                &runtime,
                "local",
                host.entry(),
            )?
            .context("old claim sight must survive the CAS-to-sight crash boundary")?,
            &replacement.launch_id,
            &ready.heartbeat_id,
            &host,
        )?;
        assert_eq!(
            reissued, directive,
            "directive reissue must be byte-identical"
        );
        let coverage = crate::open_workspace_coverage_authority(&runtime)?;
        let resumed =
            match crate::workspace_coverage_projector::acquire_workspace_coverage_projection(
                &prepared,
                &coverage.store,
                EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID,
                &replacement.provider_incarnation_id,
                &replacement.launch_id,
            )? {
                crate::workspace_coverage_projector::WorkspaceCoverageAcquireResult::Acquired(
                    resumed,
                ) => resumed,
                _ => bail!("exact replacement incarnation did not resume its recovered claim"),
            };
        assert_eq!(resumed.claim.claim_id, recovered.claim_id);
        publish_workspace_coverage_claim_sight(
            &verse,
            &runtime,
            &coverage,
            &resumed,
            "local",
            &replacement.launch_id,
            host.entry(),
            &replacement_provider,
            chrono::Utc::now(),
        )?;
        assert_eq!(
            authenticate_current_workspace_coverage_claim_sight(
                &verse,
                &runtime,
                "local",
                host.entry(),
            )?
            .context("successor sight missing after recovery resume")?
            .claim_id,
            recovered.claim_id,
        );
        drop(resumed);
        assert!(matches!(
            crate::workspace_coverage_projector::acquire_workspace_coverage_projection(
                &prepared,
                &coverage.store,
                EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID,
                "wrong-incarnation",
                &replacement.launch_id,
            )?,
            crate::workspace_coverage_projector::WorkspaceCoverageAcquireResult::Contended
        ));
        let opening = coverage.store.pull_all()?;
        let old_history: crate::workspace_coverage_projector::WorkspaceCoverageProjectionClaim =
            rmp_serde::from_slice(
                &opening
                    .iter()
                    .find(|entry| {
                        entry.r#type == "gamecult.epiphany.workspace_coverage_projection_claim"
                            && entry.key == format!("history/{old_claim_id}")
                    })
                    .context("failed claim history missing")?
                    .payload,
            )?;
        assert_eq!(old_history.status, "failed");
        assert!(old_history.termination_evidence_digest.is_some());
        drop(coverage);
        let recovery_id = recovered.recovery_receipt_id.as_str();
        let recovery_digest = recovered.recovery_receipt_digest.as_str();
        crate::workspace_coverage_projector::authenticate_workspace_coverage_recovery_receipt(
            &runtime,
            &verse,
            "local",
            host.entry(),
            recovery_id,
            recovery_digest,
        )?;
        assert_eq!(
            old_history.recovery_receipt_id.as_deref(),
            Some(recovery_id)
        );
        assert_eq!(
            old_history.recovery_receipt_digest.as_deref(),
            Some(recovery_digest)
        );
        let replayed =
            crate::workspace_coverage_projector::recover_workspace_coverage_projection_with_authority(
                &crate::open_workspace_coverage_authority(&runtime)?,
                &runtime,
                &verse,
                "local",
                host.entry(),
                &old_launch.launch_id,
                &replacement.launch_id,
                &ready.heartbeat_id,
                &old_claim_id,
            )?;
        assert_eq!(
            replayed, recovered,
            "exact crash-boundary replay must be idempotent"
        );
        std::fs::write(repo.join("after-recovery.rs"), "fn newer_body() {}\n")?;
        Command::new("git")
            .args(["add", "."])
            .current_dir(&repo)
            .output()?;
        let newer_basis = crate::observe_runtime_repository_body_basis(&runtime)?;
        assert!(newer_basis.generation > basis.generation);
        assert!(
            authenticate_current_workspace_coverage_claim_sight(
                &verse,
                &runtime,
                "local",
                host.entry(),
            )?
            .is_none(),
            "a prior-Body claim sight must not become the current recovery target"
        );
        assert!(
            authenticate_workspace_coverage_recovery_directive(
                &verse,
                &runtime,
                "local",
                &directive.replacement_launch_id,
                host.entry(),
            )?
            .is_none(),
            "a prior-Body directive must become inert after Body advances"
        );
        Ok(())
    }
}
