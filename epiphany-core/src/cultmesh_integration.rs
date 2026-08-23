use crate::default_continuity_cultnet_contracts;
use crate::default_eyes_cultnet_contracts;
use crate::default_hands_cultnet_contracts;
use crate::default_mind_cultnet_contracts;
use crate::default_soul_cultnet_contracts;
use crate::default_substrate_gate_cultnet_contracts;
use crate::packaged_release::{EpiphanyPackagedReleaseEntry, EpiphanyPackagedReleaseHead};
use crate::workspace_coverage_process_documents::{
    WORKSPACE_COVERAGE_ADVANCEMENT_SIGHT_SCHEMA_VERSION,
    WORKSPACE_COVERAGE_CLAIM_SIGHT_SCHEMA_VERSION,
    WORKSPACE_COVERAGE_PROCESS_EVIDENCE_HEAD_SCHEMA_VERSION,
    WORKSPACE_COVERAGE_PROCESS_LAUNCH_SCHEMA_VERSION,
    WORKSPACE_COVERAGE_PROCESS_TERMINATION_SCHEMA_VERSION,
    WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_SCHEMA_VERSION,
    WORKSPACE_COVERAGE_RECOVERY_DIRECTIVE_SCHEMA_VERSION,
    WORKSPACE_COVERAGE_TERMINAL_SIGHT_SCHEMA_VERSION, WorkspaceCoverageAdvancementSightEntry,
    WorkspaceCoverageClaimSightEntry, WorkspaceCoverageManagedProcessLaunchEntry,
    WorkspaceCoverageProcessEvidenceHead, WorkspaceCoverageProcessTerminationObservationEntry,
    WorkspaceCoverageProviderHeartbeatEntry, WorkspaceCoverageRecoveryDirectiveEntry,
    WorkspaceCoverageTerminalSightEntry,
};
use crate::workspace_coverage_projection_progress::{
    WORKSPACE_COVERAGE_PROJECTION_PROGRESS_SCHEMA_VERSION, WorkspaceCoverageProjectionProgressEntry,
};
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::DateTime;
use chrono::Utc;
use cultcache_rs::CacheBackingStore;
use cultcache_rs::DatabaseEntry;
use cultcache_rs::SingleFileMessagePackBackingStore;
use cultmesh_rs::CultMesh;
use cultmesh_rs::CultMeshNode;
use cultmesh_rs::CultMeshNodeOptions;
use cultmesh_rs::cultmesh_documents;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use std::path::Path;
use uuid::Uuid;

pub const EPIPHANY_CULTMESH_CLUSTER_TOPOLOGY_TYPE: &str = "epiphany.cultmesh.cluster_topology";
pub const EPIPHANY_CULTMESH_CLUSTER_TOPOLOGY_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.cluster_topology.v0";
pub const EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_TYPE: &str =
    "epiphany.cultmesh.daemon_heartbeat_event";
pub const EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.daemon_heartbeat_event.v1";
pub const EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_TYPE: &str =
    "epiphany.cultmesh.daemon_service_lifecycle_receipt";
pub const EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.daemon_service_lifecycle_receipt.v2";
pub const EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_TYPE: &str =
    "epiphany.cultmesh.managed_service_policy";
pub const EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.managed_service_policy.v1";
const EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID: &str = "epiphany-memory-semantic-projector-service";
pub const EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID: &str =
    "epiphany-workspace-coverage-projector-service";
pub const EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID: &str =
    "epiphany-workspace-coverage-projector";
pub const EPIPHANY_CULTMESH_SWARM_BRAKE_TYPE: &str = "epiphany.cultmesh.swarm_brake";
pub const EPIPHANY_CULTMESH_SWARM_BRAKE_SCHEMA_VERSION: &str = "epiphany.cultmesh.swarm_brake.v0";
pub const EPIPHANY_CULTMESH_SWARM_BRAKE_KEY: &str = "epiphany-local/swarm-brake";
pub const EPIPHANY_CULTMESH_INTERNAL_VERSE_ID: &str = "epiphany-internal";
pub const EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID: &str = "gamecult-local";
pub const EPIPHANY_CULTMESH_GLOBAL_VERSE_ID: &str = "epiphany-global";
pub const EPIPHANY_CULTMESH_INTERNAL_TIER: &str = "internal";
pub const EPIPHANY_CULTMESH_LOCAL_AREA_TIER: &str = "local-area";
pub const EPIPHANY_CULTMESH_GLOBAL_TIER: &str = "global";
pub const EPIPHANY_CULTMESH_SEMANTIC_PROJECTION_HEALTH_TYPE: &str =
    "epiphany.cultmesh.semantic_projection_health";
pub const EPIPHANY_CULTMESH_SEMANTIC_PROJECTION_HEALTH_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.semantic_projection_health.v0";

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.semantic_projection_health",
    schema = "EpiphanyCultMeshSemanticProjectionHealthEntry"
)]
pub struct EpiphanyCultMeshSemanticProjectionHealthEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub verse_id: String,
    #[cultcache(key = 2)]
    pub verse_tier: String,
    #[cultcache(key = 3)]
    pub swarm_id: String,
    #[cultcache(key = 4)]
    pub partition: String,
    #[cultcache(key = 5)]
    pub obligation_id: String,
    #[cultcache(key = 6)]
    pub source_generation: u64,
    #[cultcache(key = 7)]
    pub canonical_model_hash: String,
    #[cultcache(key = 8)]
    pub canonical_content_set_hash: String,
    #[cultcache(key = 9)]
    pub status: String,
    #[cultcache(key = 10)]
    pub receipt_id: Option<String>,
    #[cultcache(key = 11)]
    pub indexed_document_count: Option<u32>,
    #[cultcache(key = 12)]
    pub vector_dimensions: Option<u32>,
    #[cultcache(key = 13)]
    pub observed_at: String,
    #[cultcache(key = 14)]
    pub private_state_exposed: bool,
    #[cultcache(key = 15)]
    pub provider_id: String,
    #[cultcache(key = 16)]
    pub provider_incarnation: String,
    #[cultcache(key = 17)]
    pub observed_source_at: String,
    #[cultcache(key = 18)]
    pub authoritative: bool,
    #[cultcache(key = 19)]
    pub query_eligible_display_only: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyVersePolicy {
    pub verse_id: String,
    pub tier: String,
    pub purpose: String,
    pub transport_scope: String,
    pub trust_boundary: String,
    pub private_state_allowed: bool,
    pub untrusted_ingress_allowed: bool,
    pub yggdrasil_tunnel_allowed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyGlobalRoomPolicy {
    pub room_id: String,
    pub verse_id: String,
    pub topic: String,
    pub purpose: String,
    pub posting_policy: String,
    pub threaded: bool,
    pub persona_posting_allowed: bool,
    pub untrusted_ingress_allowed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.cluster_topology",
    schema = "EpiphanyCultMeshClusterTopologyEntry"
)]
pub struct EpiphanyCultMeshClusterTopologyEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub cluster_id: String,
    #[cultcache(key = 2)]
    pub role_id: String,
    #[cultcache(key = 3)]
    pub display_name: String,
    #[cultcache(key = 4)]
    pub private_verse_id: String,
    #[cultcache(key = 5)]
    pub body_domain: String,
    #[cultcache(key = 6)]
    pub body_kind: String,
    #[cultcache(key = 7)]
    pub daemon_id: String,
    #[cultcache(key = 8)]
    pub daemon_surface_id: String,
    #[cultcache(key = 9)]
    pub eve_surface_id: String,
    #[cultcache(key = 10)]
    pub public_persona_discussion_allowed: bool,
    #[cultcache(key = 11)]
    #[cultcache(key = 12)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.daemon_heartbeat_event",
    schema = "EpiphanyCultMeshDaemonHeartbeatEventEntry"
)]
pub struct EpiphanyCultMeshDaemonHeartbeatEventEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub heartbeat_id: String,
    #[cultcache(key = 2)]
    pub daemon_id: String,
    #[cultcache(key = 3)]
    pub cluster_id: String,
    #[cultcache(key = 4)]
    pub provider_incarnation: String,
    #[cultcache(key = 5)]
    pub sequence: u64,
    #[cultcache(key = 6)]
    pub status: String,
    #[cultcache(key = 7)]
    pub heartbeat_at: String,
    #[cultcache(key = 8)]
    pub private_state_exposed: bool,
    #[cultcache(key = 9, default)]
    pub startup_lifecycle_receipt_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.daemon_service_lifecycle_receipt",
    schema = "EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry"
)]
pub struct EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub service_id: String,
    #[cultcache(key = 3)]
    pub scheduler_id: String,
    #[cultcache(key = 4)]
    pub runtime_id: String,
    #[cultcache(key = 5)]
    pub daemon_selector: String,
    #[cultcache(key = 6)]
    pub action: String,
    #[cultcache(key = 7)]
    pub status: String,
    #[cultcache(key = 8)]
    pub command: String,
    #[cultcache(key = 9)]
    pub args: Vec<String>,
    #[cultcache(key = 10)]
    pub cwd: Option<String>,
    #[cultcache(key = 11)]
    pub process_id: Option<u32>,
    #[cultcache(key = 12)]
    pub exit_code: Option<i32>,
    #[cultcache(key = 13)]
    pub started_at_utc: String,
    #[cultcache(key = 14)]
    pub completed_at_utc: Option<String>,
    #[cultcache(key = 15)]
    pub operator_artifact_ref: String,
    #[cultcache(key = 16)]
    pub private_state_exposed: bool,
    #[cultcache(key = 17)]
    pub notes: Vec<String>,
    #[cultcache(key = 18, default)]
    pub executable_sha256: String,
    #[cultcache(key = 19, default)]
    pub preflight_witness_id: String,
    #[cultcache(key = 20, default)]
    pub required_document_types: Vec<String>,
    #[cultcache(key = 21, default)]
    pub schema_preflight_passed: bool,
    #[cultcache(key = 22, default)]
    pub schema_catalog_sha256: String,
    #[cultcache(key = 23, default)]
    pub managed_policy_id: String,
    #[cultcache(key = 24, default)]
    pub managed_policy_digest: String,
    #[cultcache(key = 25, default)]
    pub provider_daemon_id: String,
    #[cultcache(key = 26, default)]
    pub startup_correlation_id: String,
    #[cultcache(key = 27, default)]
    pub process_creation_token: u64,
    #[cultcache(key = 28, default)]
    pub process_created_at_rfc3339: Option<String>,
    #[cultcache(key = 29, default)]
    pub process_executable_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.managed_service_policy",
    schema = "EpiphanyCultMeshManagedServicePolicyEntry"
)]
pub struct EpiphanyCultMeshManagedServicePolicyEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub policy_id: String,
    #[cultcache(key = 2)]
    pub service_id: String,
    #[cultcache(key = 4)]
    pub command: String,
    #[cultcache(key = 5)]
    pub args: Vec<String>,
    #[cultcache(key = 6)]
    pub cwd: Option<String>,
    #[cultcache(key = 7)]
    pub enabled: bool,
    #[cultcache(key = 9)]
    pub cooldown_seconds: i64,
    #[cultcache(key = 11)]
    pub stdout_artifact: String,
    #[cultcache(key = 12)]
    pub stderr_artifact: String,
    #[cultcache(key = 15)]
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.swarm_brake",
    schema = "EpiphanyCultMeshSwarmBrakeEntry"
)]
pub struct EpiphanyCultMeshSwarmBrakeEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub brake_id: String,
    #[cultcache(key = 2)]
    pub status: String,
    #[cultcache(key = 3)]
    pub scope: String,
    #[cultcache(key = 4)]
    pub reason: String,
    #[cultcache(key = 5)]
    pub operator_agent_id: String,
    #[cultcache(key = 6)]
    pub affected_clusters: Vec<String>,
    #[cultcache(key = 7)]
    pub protected_surfaces: Vec<String>,
    #[cultcache(key = 8)]
    pub created_at_utc: String,
    #[cultcache(key = 9)]
    pub expires_at_utc: Option<String>,
    #[cultcache(key = 10)]
    pub private_state_exposed: bool,
    #[cultcache(key = 11)]
    pub notes: Vec<String>,
    #[cultcache(key = 12, default)]
    pub runtime_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyLocalVerseContext {
    pub schema_version: String,
    pub runtime_id: String,
    pub store_path: String,
    pub summary: String,
    pub odin_scope: String,
    pub yggdrasil_scope: String,
    pub prompt_assembly_note: String,
    pub verse_policies: Vec<EpiphanyVersePolicy>,
    pub global_room_policies: Vec<EpiphanyGlobalRoomPolicy>,
    pub cluster_topology: Vec<EpiphanyCultMeshClusterTopologyEntry>,
    pub swarm_brake: Option<EpiphanyCultMeshSwarmBrakeEntry>,
    pub contract_summaries: Vec<EpiphanyLocalVerseContractSummary>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyLocalVerseContractSummary {
    pub contract_id: String,
    pub verse_id: String,
    pub authority: String,
    pub document_type: String,
    pub operations: Vec<String>,
    pub receipt_document_types: Vec<String>,
}

cultmesh_documents!(EpiphanyCultMeshDocuments {
    EpiphanyCultMeshClusterTopologyEntry => EPIPHANY_CULTMESH_CLUSTER_TOPOLOGY_SCHEMA_VERSION,
    EpiphanyCultMeshDaemonHeartbeatEventEntry => EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_SCHEMA_VERSION,
    EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry => EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshManagedServicePolicyEntry => EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION,
    WorkspaceCoverageManagedProcessLaunchEntry => WORKSPACE_COVERAGE_PROCESS_LAUNCH_SCHEMA_VERSION,
    WorkspaceCoverageProcessEvidenceHead => WORKSPACE_COVERAGE_PROCESS_EVIDENCE_HEAD_SCHEMA_VERSION,
    WorkspaceCoverageProviderHeartbeatEntry => WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_SCHEMA_VERSION,
    WorkspaceCoverageProcessTerminationObservationEntry => WORKSPACE_COVERAGE_PROCESS_TERMINATION_SCHEMA_VERSION,
    WorkspaceCoverageProjectionProgressEntry => WORKSPACE_COVERAGE_PROJECTION_PROGRESS_SCHEMA_VERSION,
    WorkspaceCoverageAdvancementSightEntry => WORKSPACE_COVERAGE_ADVANCEMENT_SIGHT_SCHEMA_VERSION,
    WorkspaceCoverageClaimSightEntry => WORKSPACE_COVERAGE_CLAIM_SIGHT_SCHEMA_VERSION,
    WorkspaceCoverageRecoveryDirectiveEntry => WORKSPACE_COVERAGE_RECOVERY_DIRECTIVE_SCHEMA_VERSION,
    WorkspaceCoverageTerminalSightEntry => WORKSPACE_COVERAGE_TERMINAL_SIGHT_SCHEMA_VERSION,
    EpiphanyPackagedReleaseEntry => crate::packaged_release::EPIPHANY_PACKAGED_RELEASE_SCHEMA_VERSION,
    EpiphanyPackagedReleaseHead => crate::packaged_release::EPIPHANY_PACKAGED_RELEASE_HEAD_SCHEMA_VERSION,
    EpiphanyCultMeshSwarmBrakeEntry => EPIPHANY_CULTMESH_SWARM_BRAKE_SCHEMA_VERSION,
    crate::persona_feedback_admission::LocalAdmittedPersonaFeedback => crate::persona_feedback_admission::LOCAL_PERSONA_FEEDBACK_SCHEMA_VERSION,
    EpiphanyCultMeshSemanticProjectionHealthEntry => EPIPHANY_CULTMESH_SEMANTIC_PROJECTION_HEALTH_SCHEMA_VERSION,
});

pub fn open_epiphany_cultmesh_node(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<CultMeshNode> {
    CultMesh::create_node(
        store_path,
        EpiphanyCultMeshDocuments,
        CultMeshNodeOptions {
            runtime_id: runtime_id.into(),
            ..CultMeshNodeOptions::default()
        },
    )
}

fn semantic_projection_health_scope_key(swarm_id: &str, partition: &str) -> String {
    use sha2::{Digest, Sha256};
    format!(
        "gamecult-local/semantic-projection-health/{:x}",
        Sha256::digest(format!("{swarm_id}|{partition}").as_bytes())
    )
}

/// Publishes operator sight derived from authenticated canonical projection state.
///
/// This mirror is deliberately powerless: it neither creates work nor participates
/// in semantic-query admission. Callers must retain the canonical source store and
/// sealed input for either operation.
pub fn publish_epiphany_cultmesh_semantic_projection_health(
    verse_store: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    canonical_store: impl AsRef<Path>,
    input: &crate::MemorySemanticProjectionInput,
    provider_incarnation: &str,
) -> Result<EpiphanyCultMeshSemanticProjectionHealthEntry> {
    let runtime_id = runtime_id.into();
    if !bounded_opaque_health_id(&runtime_id) || !bounded_opaque_health_id(provider_incarnation) {
        return Err(anyhow!(
            "semantic projection health provider identity is required"
        ));
    }
    let observation = crate::observe_memory_semantic_projection(canonical_store, input)?;
    let keyed_modeling_basis = input.obligation().partition == "modeling"
        && input.obligation().source_commit_id.starts_with("sha256:");
    let entry = EpiphanyCultMeshSemanticProjectionHealthEntry {
        schema_version: EPIPHANY_CULTMESH_SEMANTIC_PROJECTION_HEALTH_SCHEMA_VERSION.to_string(),
        verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
        verse_tier: EPIPHANY_CULTMESH_LOCAL_AREA_TIER.to_string(),
        swarm_id: observation.swarm_id,
        partition: observation.partition,
        obligation_id: observation.obligation_id,
        source_generation: observation.source_generation,
        canonical_model_hash: observation.canonical_model_hash,
        canonical_content_set_hash: observation.canonical_content_set_hash,
        status: observation.status,
        receipt_id: observation.receipt_id,
        indexed_document_count: observation.indexed_document_count,
        vector_dimensions: observation.vector_dimensions,
        observed_at: Utc::now().to_rfc3339(),
        private_state_exposed: false,
        provider_id: runtime_id.clone(),
        provider_incarnation: provider_incarnation.to_string(),
        observed_source_at: observation.observed_source_at,
        authoritative: false,
        query_eligible_display_only: observation.query_eligible_display_only,
    };
    validate_semantic_projection_health(&entry)?;

    let scope_key = semantic_projection_health_scope_key(&entry.swarm_id, &entry.partition);
    let latest_key = format!("{scope_key}/latest");
    use sha2::{Digest, Sha256};
    let event_key = format!(
        "{scope_key}/event-{:x}",
        Sha256::digest(
            format!(
                "{}|{}|{}|{}",
                entry.obligation_id,
                entry.status,
                entry.receipt_id.as_deref().unwrap_or("none"),
                entry.observed_at
            )
            .as_bytes()
        )
    );
    let node = open_epiphany_cultmesh_node(&verse_store, runtime_id)?;
    let backing = SingleFileMessagePackBackingStore::new(verse_store.as_ref());
    for _ in 0..8 {
        let opening = backing.pull_all()?;
        let latest_envelope = opening.iter().find(|row| {
            row.r#type == EPIPHANY_CULTMESH_SEMANTIC_PROJECTION_HEALTH_TYPE && row.key == latest_key
        });
        let latest = latest_envelope
            .map(|row| {
                rmp_serde::from_slice::<EpiphanyCultMeshSemanticProjectionHealthEntry>(&row.payload)
            })
            .transpose()?;
        if let Some(latest) = &latest {
            validate_semantic_projection_health(latest)?;
            if !keyed_modeling_basis {
                if latest.source_generation == entry.source_generation
                    && latest.obligation_id != entry.obligation_id
                {
                    return Err(anyhow!(
                        "semantic projection health generation has conflicting canonical obligations"
                    ));
                }
                if latest.source_generation > entry.source_generation {
                    return Ok(latest.clone());
                }
            }
        }
        let event = node.cache().prepare_entry(&event_key, &entry)?.0;
        let latest_replacement = node.cache().prepare_entry(&latest_key, &entry)?.0;
        let expected = latest_envelope.cloned().into_iter().collect::<Vec<_>>();
        if backing.compare_and_swap_batch(&expected, vec![event, latest_replacement])? {
            return Ok(entry);
        }
    }
    Err(anyhow!(
        "semantic projection health latest advanced during publication"
    ))
}

pub fn load_epiphany_cultmesh_semantic_projection_health(
    verse_store: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshSemanticProjectionHealthEntry>> {
    let node = open_epiphany_cultmesh_node(verse_store, runtime_id)?;
    let mut rows = node
        .get_all_with_keys::<EpiphanyCultMeshSemanticProjectionHealthEntry>()?
        .into_iter()
        .filter(|(key, _)| key.ends_with("/latest"))
        .map(|(key, row)| {
            let expected_key = format!(
                "{}/latest",
                semantic_projection_health_scope_key(&row.swarm_id, &row.partition)
            );
            if key != expected_key {
                return Err(anyhow!(
                    "semantic projection health latest key does not match its declared scope"
                ));
            }
            Ok(row)
        })
        .collect::<Result<Vec<_>>>()?;
    for row in &rows {
        validate_semantic_projection_health(row)?;
    }
    rows.sort_by(|left, right| {
        left.swarm_id
            .cmp(&right.swarm_id)
            .then(left.partition.cmp(&right.partition))
    });
    Ok(rows)
}

fn validate_semantic_projection_health(
    row: &EpiphanyCultMeshSemanticProjectionHealthEntry,
) -> Result<()> {
    if row.schema_version != EPIPHANY_CULTMESH_SEMANTIC_PROJECTION_HEALTH_SCHEMA_VERSION
        || row.verse_id != EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID
        || row.verse_tier != EPIPHANY_CULTMESH_LOCAL_AREA_TIER
        || row.swarm_id.trim().is_empty()
        || row.partition != "modeling"
        || row.obligation_id.trim().is_empty()
        || row.canonical_model_hash.trim().is_empty()
        || row.canonical_content_set_hash.trim().is_empty()
        || !matches!(row.status.as_str(), "pending" | "failed" | "ready")
        || DateTime::parse_from_rfc3339(&row.observed_at).is_err()
        || DateTime::parse_from_rfc3339(&row.observed_source_at).is_err()
        || row.private_state_exposed
        || !bounded_opaque_health_id(&row.provider_id)
        || !bounded_opaque_health_id(&row.provider_incarnation)
        || row.authoritative
    {
        return Err(anyhow!("semantic projection health mirror is invalid"));
    }
    let has_receipt = row.receipt_id.is_some()
        && row.indexed_document_count.is_some()
        && row.vector_dimensions.is_some();
    if (row.query_eligible_display_only && row.status != "ready")
        || has_receipt != (row.status == "ready")
    {
        return Err(anyhow!(
            "semantic projection health evidence shape is invalid"
        ));
    }
    Ok(())
}

fn bounded_opaque_health_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-' | b':'))
}

pub fn write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    receipt: EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
) -> Result<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry> {
    validate_daemon_service_lifecycle_receipt(&receipt)?;
    let store_path = store_path.as_ref();
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let receipt_key = epiphany_cultmesh_daemon_service_lifecycle_receipt_key(&receipt.receipt_id);
    if let Some(existing) =
        node.get::<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry>(&receipt_key)?
    {
        if existing == receipt {
            return Ok(existing);
        }
        return Err(anyhow!(
            "daemon service lifecycle receipt identity collision for {:?}",
            receipt.receipt_id
        ));
    }
    let (receipt_envelope, written) = node.cache().prepare_entry(receipt_key, &receipt)?;
    let mut expected = Vec::new();
    let mut replacements = vec![receipt_envelope];
    if written.service_id == EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID && written.action == "launch" {
        let reserved_name = "semantic projector";
        let policy_key = epiphany_cultmesh_managed_service_policy_key(&written.service_id);
        let policy_envelope = node
            .cache()
            .get_envelope::<EpiphanyCultMeshManagedServicePolicyEntry>(&policy_key)?
            .ok_or_else(|| anyhow!("reserved {reserved_name} managed policy is absent"))?;
        let mut digest = Sha256::new();
        digest.update(policy_envelope.r#type.as_bytes());
        digest.update([0]);
        digest.update(policy_envelope.key.as_bytes());
        digest.update([0]);
        digest.update(&policy_envelope.payload);
        if written.managed_policy_digest != format!("sha256-{:x}", digest.finalize()) {
            return Err(anyhow!(
                "reserved {reserved_name} launch receipt has stale managed policy digest"
            ));
        }
        expected.push(policy_envelope.clone());
        replacements.push(policy_envelope);
    }
    let service_current_key =
        epiphany_cultmesh_daemon_service_lifecycle_receipt_current_key(&written.service_id);
    if let Some(envelope) = node
        .cache()
        .get_envelope::<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry>(&service_current_key)?
    {
        expected.push(envelope);
    }
    replacements.push(node.cache().prepare_entry(&service_current_key, &written)?.0);
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    if backing.compare_and_swap_batch(&expected, replacements)? {
        return Ok(written);
    }
    let reloaded = open_epiphany_cultmesh_node(store_path, "lifecycle-cas-readback")?;
    match reloaded.get::<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry>(
        &epiphany_cultmesh_daemon_service_lifecycle_receipt_key(&written.receipt_id),
    )? {
        Some(existing) if existing == written => Ok(existing),
        Some(_) => Err(anyhow!(
            "daemon service lifecycle receipt identity collision for {:?}",
            written.receipt_id
        )),
        None => Err(anyhow!(
            "daemon service lifecycle receipt CAS lost concurrent current-state race"
        )),
    }
}

pub fn load_epiphany_cultmesh_daemon_service_lifecycle_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    receipt_id: &str,
) -> Result<Option<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(&epiphany_cultmesh_daemon_service_lifecycle_receipt_key(
        receipt_id,
    ))
}

pub fn load_epiphany_cultmesh_managed_service_policy_with_digest(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    service_id: &str,
) -> Result<Option<(EpiphanyCultMeshManagedServicePolicyEntry, String)>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let key = epiphany_cultmesh_managed_service_policy_key(service_id);
    let Some(policy) = node.get::<EpiphanyCultMeshManagedServicePolicyEntry>(&key)? else {
        return Ok(None);
    };
    validate_managed_service_policy(&policy)?;
    let digest =
        cultmesh_envelope_digest::<EpiphanyCultMeshManagedServicePolicyEntry>(&node, &key)?;
    Ok(Some((policy, digest)))
}

pub fn authenticate_epiphany_cultmesh_semantic_projector_launch(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    receipt_id: &str,
) -> Result<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry> {
    let runtime_id = runtime_id.into();
    let receipt = load_epiphany_cultmesh_daemon_service_lifecycle_receipt(
        store_path.as_ref(),
        runtime_id.clone(),
        receipt_id,
    )?
    .ok_or_else(|| anyhow!("semantic projector startup launch receipt is absent"))?;
    validate_semantic_projector_launch_receipt(&receipt)?;
    let (policy, digest) = load_epiphany_cultmesh_managed_service_policy_with_digest(
        store_path,
        runtime_id,
        EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID,
    )?
    .ok_or_else(|| anyhow!("semantic projector managed policy is absent"))?;
    validate_semantic_projector_managed_service_policy(&policy)?;
    if receipt.managed_policy_id != policy.policy_id
        || receipt.managed_policy_digest != digest
        || receipt.command != policy.command
        || receipt.args != policy.args
        || receipt.cwd != policy.cwd
    {
        return Err(anyhow!(
            "semantic projector startup launch receipt disagrees with current managed policy"
        ));
    }
    Ok(receipt)
}

pub fn load_current_epiphany_cultmesh_daemon_service_lifecycle_receipt_for_service(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    service_id: &str,
) -> Result<Option<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(&epiphany_cultmesh_daemon_service_lifecycle_receipt_current_key(service_id))
}

pub fn write_epiphany_cultmesh_semantic_projector_service_policy(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    policy: EpiphanyCultMeshManagedServicePolicyEntry,
) -> Result<EpiphanyCultMeshManagedServicePolicyEntry> {
    validate_managed_service_policy(&policy)?;
    if policy.service_id != EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID {
        return Err(anyhow!(
            "semantic projector policy writer requires its reserved service id"
        ));
    }
    write_validated_managed_service_policy(store_path, runtime_id, policy)
}

pub fn write_epiphany_cultmesh_workspace_coverage_projector_service_policy(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    policy: EpiphanyCultMeshManagedServicePolicyEntry,
) -> Result<EpiphanyCultMeshManagedServicePolicyEntry> {
    validate_managed_service_policy(&policy)?;
    if policy.service_id != EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID {
        return Err(anyhow!(
            "workspace coverage projector policy writer requires its reserved service id"
        ));
    }
    write_validated_managed_service_policy(store_path, runtime_id, policy)
}

fn write_validated_managed_service_policy(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    policy: EpiphanyCultMeshManagedServicePolicyEntry,
) -> Result<EpiphanyCultMeshManagedServicePolicyEntry> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let key = epiphany_cultmesh_managed_service_policy_key(&policy.service_id);
    let written = node.put(key.as_str(), &policy)?;
    node.flush()?;
    Ok(written)
}

pub fn load_epiphany_cultmesh_managed_service_policy(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    service_id: &str,
) -> Result<Option<EpiphanyCultMeshManagedServicePolicyEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let policy = node.get(&epiphany_cultmesh_managed_service_policy_key(service_id))?;
    policy
        .map(|policy| {
            validate_managed_service_policy(&policy)?;
            Ok(policy)
        })
        .transpose()
}

pub fn load_epiphany_cultmesh_managed_service_policies(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshManagedServicePolicyEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node
        .get_all_with_keys::<EpiphanyCultMeshManagedServicePolicyEntry>()?
        .into_iter()
        .map(|(_, policy)| {
            validate_managed_service_policy(&policy)?;
            Ok(policy)
        })
        .collect()
}

pub fn default_epiphany_cultmesh_swarm_brake(
    generated_at_utc: impl Into<String>,
) -> EpiphanyCultMeshSwarmBrakeEntry {
    EpiphanyCultMeshSwarmBrakeEntry {
        schema_version: EPIPHANY_CULTMESH_SWARM_BRAKE_SCHEMA_VERSION.to_string(),
        brake_id: EPIPHANY_CANONICAL_SWARM_BRAKE_ID.to_string(),
        status: "released".to_string(),
        scope: "swarm".to_string(),
        reason: "No swarm brake is engaged; unattended automation still requires typed scheduler, cooldown, recovery, and operator receipt gates.".to_string(),
        operator_agent_id: EPIPHANY_CANONICAL_SWARM_BRAKE_OWNER.to_string(),
        affected_clusters: epiphany_cultmesh_cluster_topology()
            .into_iter()
            .map(|cluster| cluster.cluster_id)
            .collect(),
        protected_surfaces: vec![
            "heartbeat.scheduler".to_string(),
            "coordinator.run".to_string(),
            "persona.public_speech".to_string(),
            "daemon.tool_invocation".to_string(),
            "daemon.lifecycle_poke".to_string(),
            "atlas.publish".to_string(),
            "atlas.project".to_string(),
            "atlas.impact_ingress".to_string(),
        ],
        created_at_utc: generated_at_utc.into(),
        expires_at_utc: None,
        private_state_exposed: false,
        notes: vec![
            "The swarm brake is the operator-safe pause surface for live-fire readiness.".to_string(),
            "It may stop scheduling and daemon pokes, but it must not expose worker thoughts or private Verse state.".to_string(),
            "Engaged brakes require a scoped reason so silence cannot masquerade as consent.".to_string(),
        ],
        runtime_id: String::new(),
    }
}

/// The single runtime brake identity shared by deployment, resident readiness,
/// and local typed control surfaces. A caller identity is provenance, not
/// brake ownership, and must not be substituted into either constant.
pub const EPIPHANY_CANONICAL_SWARM_BRAKE_ID: &str = "epiphany/swarm-brake";
pub const EPIPHANY_CANONICAL_SWARM_BRAKE_OWNER: &str = "epiphany.swarm-brake";

pub fn canonical_epiphany_swarm_brake_protected_surfaces() -> Vec<String> {
    vec![
        "heartbeat.scheduler".to_string(),
        "coordinator.run".to_string(),
        "persona.public_speech".to_string(),
        "daemon.tool_invocation".to_string(),
        "daemon.lifecycle_poke".to_string(),
        "atlas.publish".to_string(),
        "atlas.project".to_string(),
        "atlas.impact_ingress".to_string(),
    ]
}

pub fn engage_epiphany_cultmesh_swarm_brake(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    reason: impl Into<String>,
    actor_id: impl Into<String>,
    created_at_utc: impl Into<String>,
    allow_engaged_adoption: bool,
) -> Result<EpiphanyCultMeshSwarmBrakeEntry> {
    let runtime_id = runtime_id.into();
    let actor_id = actor_id.into();
    if actor_id.trim().is_empty() {
        return Err(anyhow!("swarm brake engagement requires an actor identity"));
    }
    if allow_engaged_adoption && actor_id != "Idunn" {
        return Err(anyhow!(
            "only Idunn may adopt an already-engaged legacy brake"
        ));
    }
    if let Some(current) = load_epiphany_cultmesh_swarm_brake(&store_path, runtime_id.clone())? {
        let foreign = current.brake_id != EPIPHANY_CANONICAL_SWARM_BRAKE_ID
            || current.operator_agent_id != EPIPHANY_CANONICAL_SWARM_BRAKE_OWNER;
        if current.status == "engaged" && foreign && !allow_engaged_adoption {
            return Err(anyhow!(
                "refusing to replace engaged foreign swarm brake {} owned by {}",
                current.brake_id,
                current.operator_agent_id
            ));
        }
    }
    let mut brake = default_epiphany_cultmesh_swarm_brake(created_at_utc);
    brake.status = "engaged".to_string();
    brake.scope = "all".to_string();
    brake.reason = reason.into();
    brake.affected_clusters = vec![runtime_id.clone()];
    brake.protected_surfaces = canonical_epiphany_swarm_brake_protected_surfaces();
    brake.notes = vec![format!("Explicit brake engagement by {actor_id}.")];
    write_epiphany_cultmesh_swarm_brake(store_path, runtime_id, brake)
}

pub fn release_epiphany_cultmesh_swarm_brake(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    reason: impl Into<String>,
    actor_id: impl Into<String>,
    created_at_utc: impl Into<String>,
) -> Result<EpiphanyCultMeshSwarmBrakeEntry> {
    let runtime_id = runtime_id.into();
    let actor_id = actor_id.into();
    if actor_id.trim().is_empty() {
        return Err(anyhow!("swarm brake release requires an actor identity"));
    }
    let mut brake = load_epiphany_cultmesh_swarm_brake(&store_path, runtime_id.clone())?
        .ok_or_else(|| anyhow!("refusing to release an absent swarm brake"))?;
    if brake.brake_id != EPIPHANY_CANONICAL_SWARM_BRAKE_ID
        || brake.operator_agent_id != EPIPHANY_CANONICAL_SWARM_BRAKE_OWNER
    {
        return Err(anyhow!(
            "refusing to release foreign swarm brake {} owned by {}",
            brake.brake_id,
            brake.operator_agent_id
        ));
    }
    brake.status = "released".to_string();
    brake.reason = reason.into();
    brake.created_at_utc = created_at_utc.into();
    brake.expires_at_utc = None;
    brake.notes = vec![format!("Explicit brake release by {actor_id}.")];
    write_epiphany_cultmesh_swarm_brake(store_path, runtime_id, brake)
}

pub fn write_epiphany_cultmesh_swarm_brake(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    mut brake: EpiphanyCultMeshSwarmBrakeEntry,
) -> Result<EpiphanyCultMeshSwarmBrakeEntry> {
    let runtime_id = runtime_id.into();
    brake.runtime_id = runtime_id.clone();
    validate_swarm_brake(&brake)?;
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let written = node.put(EPIPHANY_CULTMESH_SWARM_BRAKE_KEY, &brake)?;
    node.flush()?;
    Ok(written)
}

pub fn load_epiphany_cultmesh_swarm_brake(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshSwarmBrakeEntry>> {
    let runtime_id = runtime_id.into();
    let node = open_epiphany_cultmesh_node(store_path, runtime_id.clone())?;
    Ok(node
        .get::<EpiphanyCultMeshSwarmBrakeEntry>(EPIPHANY_CULTMESH_SWARM_BRAKE_KEY)?
        .filter(|brake| brake.runtime_id.is_empty() || brake.runtime_id == runtime_id))
}

fn validate_swarm_brake(brake: &EpiphanyCultMeshSwarmBrakeEntry) -> Result<()> {
    if brake.private_state_exposed {
        return Err(anyhow!("swarm brake must not expose private state"));
    }
    if brake.brake_id.trim().is_empty() || brake.scope.trim().is_empty() {
        return Err(anyhow!("swarm brake requires brake id and scope"));
    }
    if brake.runtime_id.trim().is_empty() {
        return Err(anyhow!("swarm brake requires its owning runtime id"));
    }
    if brake.created_at_utc.trim().is_empty() {
        return Err(anyhow!("swarm brake requires a creation timestamp"));
    }
    if !matches!(brake.status.as_str(), "released" | "engaged") {
        return Err(anyhow!("swarm brake status must be released or engaged"));
    }
    if brake.status == "engaged" {
        if brake.reason.trim().is_empty() || brake.operator_agent_id.trim().is_empty() {
            return Err(anyhow!(
                "engaged swarm brake requires operator id and reason"
            ));
        }
        if brake.affected_clusters.is_empty() && brake.protected_surfaces.is_empty() {
            return Err(anyhow!(
                "engaged swarm brake requires affected clusters or protected surfaces"
            ));
        }
    }
    Ok(())
}

fn validate_managed_service_policy(
    policy: &EpiphanyCultMeshManagedServicePolicyEntry,
) -> Result<()> {
    if policy.schema_version != EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION {
        return Err(anyhow!("historical managed service policy schema is not writable"));
    }
    if policy.private_state_exposed {
        return Err(anyhow!(
            "managed service policies must not expose private state"
        ));
    }
    for (label, value) in [
        ("policy id", policy.policy_id.as_str()),
        ("service id", policy.service_id.as_str()),
        ("command", policy.command.as_str()),
        ("stdout artifact", policy.stdout_artifact.as_str()),
        ("stderr artifact", policy.stderr_artifact.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("managed service policy missing {label}"));
        }
    }
    if policy.cooldown_seconds < 0 {
        return Err(anyhow!(
            "managed service policy requires a non-negative cooldown"
        ));
    }
    match policy.service_id.as_str() {
        EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID => {
            validate_semantic_projector_managed_service_policy(policy)
        }
        EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID => {
            validate_workspace_coverage_projector_managed_service_policy(policy)
        }
        _ => Err(anyhow!("unregistered managed service policy is not writable")),
    }
}

fn validate_semantic_projector_managed_service_policy(
    policy: &EpiphanyCultMeshManagedServicePolicyEntry,
) -> Result<()> {
    let expected_binary = if cfg!(windows) {
        "epiphany-memory-semantic-projector.exe"
    } else {
        "epiphany-memory-semantic-projector"
    };
    if Path::new(&policy.command)
        .file_name()
        .and_then(|name| name.to_str())
        != Some(expected_binary)
    {
        return Err(anyhow!(
            "reserved semantic projector policy requires the packaged projector executable"
        ));
    }
    if policy.policy_id != "managed-service-policy-epiphany-memory-semantic-projector-service"
        || policy.args.len() != 15
        || policy.args[0] != "serve"
        || policy.args[1] != "--runtime-store"
        || policy.args[2].trim().is_empty()
        || policy.args[3] != "--local-verse-store"
        || policy.args[4].trim().is_empty()
        || policy.args[5] != "--runtime-id"
        || policy.args[6].trim().is_empty()
        || policy.args[7] != "--interval-seconds"
        || policy.args[8].trim().is_empty()
        || policy.args[8]
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .is_none()
        || policy.args[9] != "--qdrant-url"
        || policy.args[10].trim().is_empty()
        || policy.args[11] != "--ollama-base-url"
        || policy.args[12].trim().is_empty()
        || policy.args[13] != "--ollama-model"
        || policy.args[14].trim().is_empty()
    {
        return Err(anyhow!(
            "reserved semantic projector policy must bind one packaged process to the canonical Modeling store"
        ));
    }
    Ok(())
}

pub(crate) fn validate_workspace_coverage_projector_managed_service_policy(
    policy: &EpiphanyCultMeshManagedServicePolicyEntry,
) -> Result<()> {
    let expected_binary = if cfg!(windows) {
        "epiphany-workspace-coverage-projector.exe"
    } else {
        "epiphany-workspace-coverage-projector"
    };
    let expected_command = std::env::current_exe()
        .context("cannot resolve current executable for packaged projector policy")?
        .with_file_name(expected_binary);
    if Path::new(&policy.command) != expected_command {
        return Err(anyhow!(
            "reserved workspace coverage projector policy requires the packaged projector executable"
        ));
    }
    if policy.policy_id != "managed-service-policy-epiphany-workspace-coverage-projector-service"
        || policy.service_id != EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID
        || !policy.enabled
        || policy.args.len() != 17
        || policy.args[0] != "serve"
        || policy.args[1] != "--runtime-store"
        || policy.args[2].trim().is_empty()
        || policy.args[3] != "--local-verse-store"
        || policy.args[4].trim().is_empty()
        || policy.args[5] != "--runtime-id"
        || policy.args[6].trim().is_empty()
        || policy.args[7] != "--interval-seconds"
        || policy.args[8]
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .is_none()
        || policy.args[9] != "--heartbeat-interval-seconds"
        || policy.args[10]
            .parse::<u64>()
            .ok()
            .filter(|value| *value > 0)
            .is_none()
        || policy.args[11] != "--qdrant-url"
        || policy.args[12].trim().is_empty()
        || policy.args[13] != "--ollama-base-url"
        || policy.args[14].trim().is_empty()
        || policy.args[15] != "--ollama-model"
        || policy.args[16].trim().is_empty()
    {
        return Err(anyhow!(
            "reserved workspace coverage projector policy must bind one packaged process to its authenticated runtime Body route"
        ));
    }
    Ok(())
}

fn validate_daemon_service_lifecycle_receipt(
    receipt: &EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
) -> Result<()> {
    if receipt.service_id == EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID {
        return Err(anyhow!(
            "workspace coverage process authority belongs to its specialized managed process documents"
        ));
    }
    if receipt.private_state_exposed {
        return Err(anyhow!(
            "daemon service lifecycle receipts must not expose private state"
        ));
    }
    for (label, value) in [
        ("receipt id", receipt.receipt_id.as_str()),
        ("service id", receipt.service_id.as_str()),
        ("scheduler id", receipt.scheduler_id.as_str()),
        ("runtime id", receipt.runtime_id.as_str()),
        ("daemon selector", receipt.daemon_selector.as_str()),
        ("action", receipt.action.as_str()),
        ("status", receipt.status.as_str()),
        ("command", receipt.command.as_str()),
        ("started at", receipt.started_at_utc.as_str()),
        (
            "operator artifact ref",
            receipt.operator_artifact_ref.as_str(),
        ),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("daemon service lifecycle receipt missing {label}"));
        }
    }
    let started_at = DateTime::parse_from_rfc3339(&receipt.started_at_utc).map_err(|error| {
        anyhow!("daemon service lifecycle receipt has invalid started at: {error}")
    })?;
    if let Some(completed_at) = receipt.completed_at_utc.as_deref() {
        let completed_at = DateTime::parse_from_rfc3339(completed_at).map_err(|error| {
            anyhow!("daemon service lifecycle receipt has invalid completed at: {error}")
        })?;
        if completed_at < started_at {
            return Err(anyhow!(
                "daemon service lifecycle receipt completed before it started"
            ));
        }
    }
    if !receipt.required_document_types.is_empty()
        && (!receipt.schema_preflight_passed
            || receipt.executable_sha256.trim().is_empty()
            || receipt.schema_catalog_sha256.trim().is_empty()
            || receipt.preflight_witness_id.trim().is_empty())
    {
        return Err(anyhow!(
            "typed daemon service lifecycle receipt requires passing schema preflight, executable fingerprint, and preflight witness"
        ));
    }
    if receipt.service_id == EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID && receipt.action == "launch" {
        validate_semantic_projector_launch_receipt(receipt)?;
    }
    Ok(())
}

fn validate_semantic_projector_launch_receipt(
    receipt: &EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
) -> Result<()> {
    if receipt.schema_version != EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION
        || receipt.service_id != EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID
        || receipt.action != "launch"
        || receipt.status != "launched"
        || receipt.process_id.is_none()
        || receipt.process_creation_token == 0
        || receipt.process_executable_path.trim().is_empty()
        || receipt.exit_code.is_some()
        || receipt.completed_at_utc.is_none()
        || !receipt.executable_sha256.starts_with("sha256-")
        || receipt.managed_policy_id.trim().is_empty()
        || !receipt.managed_policy_digest.starts_with("sha256-")
        || receipt.provider_daemon_id != "epiphany-memory-semantic-projector"
        || receipt.startup_correlation_id != receipt.receipt_id
        || Uuid::parse_str(&receipt.receipt_id).is_err()
    {
        return Err(anyhow!(
            "reserved semantic projector launch receipt must bind completed spawn to exact managed policy and provider identity"
        ));
    }
    Ok(())
}

pub fn seed_epiphany_local_verse_context(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    generated_at_utc: impl Into<String>,
    body_domain: impl Into<String>,
) -> Result<()> {
    let store_path = store_path.as_ref();
    let runtime_id = runtime_id.into();
    let generated_at_utc = generated_at_utc.into();
    let body_domain = body_domain.into();
    if !body_domain.starts_with("repo:") || body_domain.trim() == "repo:" {
        anyhow::bail!("local Verse repository topology requires an explicit repo: Body domain");
    }
    write_epiphany_cultmesh_cluster_topology(store_path, runtime_id.clone(), body_domain)?;
    {
        let node = open_epiphany_cultmesh_node(store_path, runtime_id.clone())?;
        if node
            .get::<EpiphanyCultMeshSwarmBrakeEntry>(EPIPHANY_CULTMESH_SWARM_BRAKE_KEY)?
            .is_none()
        {
            write_epiphany_cultmesh_swarm_brake(
                store_path,
                runtime_id.clone(),
                default_epiphany_cultmesh_swarm_brake(generated_at_utc.clone()),
            )?;
        }
    }
    Ok(())
}

pub fn query_epiphany_local_verse_context(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<EpiphanyLocalVerseContext> {
    let store_path = store_path.as_ref();
    let runtime_id = runtime_id.into();
    if !store_path.exists() {
        anyhow::bail!(
            "local Verse store does not exist at {}",
            store_path.display()
        );
    }
    let node = open_epiphany_cultmesh_node(store_path, runtime_id.clone())?;
    let verse_policies = epiphany_verse_policies();
    let global_room_policies = epiphany_global_room_policies();

    let mut cluster_topology = Vec::new();
    for cluster in epiphany_cultmesh_cluster_topology() {
        if let Some(loaded) =
            node.get::<EpiphanyCultMeshClusterTopologyEntry>(&cluster.cluster_id)?
        {
            cluster_topology.push(loaded);
        }
    }

    let mut contract_summaries = Vec::new();
    contract_summaries.extend(
        default_mind_cultnet_contracts()
            .into_iter()
            .map(IntoLocalVerseContractSummary::into_local_verse_summary),
    );
    contract_summaries.extend(
        default_substrate_gate_cultnet_contracts()
            .into_iter()
            .map(IntoLocalVerseContractSummary::into_local_verse_summary),
    );
    contract_summaries.extend(
        default_eyes_cultnet_contracts()
            .into_iter()
            .map(IntoLocalVerseContractSummary::into_local_verse_summary),
    );
    contract_summaries.extend(
        default_hands_cultnet_contracts()
            .into_iter()
            .map(IntoLocalVerseContractSummary::into_local_verse_summary),
    );
    contract_summaries.extend(
        default_soul_cultnet_contracts()
            .into_iter()
            .map(IntoLocalVerseContractSummary::into_local_verse_summary),
    );
    contract_summaries.extend(
        default_continuity_cultnet_contracts()
            .into_iter()
            .map(IntoLocalVerseContractSummary::into_local_verse_summary),
    );
    Ok(EpiphanyLocalVerseContext {
        schema_version: "epiphany.local_verse_context.v0".to_string(),
        runtime_id: runtime_id.clone(),
        store_path: store_path.display().to_string(),
        summary: "Local Verse query context for compact Epiphany prompt assembly and operator inspection.".to_string(),
        odin_scope: "Odin is the all-seer coordinator of Verse discovery: it may know every Verse's advertised public/operator-safe surface, but it must not bypass Verse trust boundaries or Mind adoption gates.".to_string(),
        yggdrasil_scope: "Yggdrasil is the hosting spine for important trusted GameCult Verses such as Bifrost; local-area writes require explicit trusted tunnel/lease policy and never carry private internal state.".to_string(),
        prompt_assembly_note: "Prompt assembly should query this compact typed bundle plus semantic memory context cuts; Verse context is injected dynamically as bounded context, not as durable truth.".to_string(),
        verse_policies,
        global_room_policies,
        cluster_topology,
        swarm_brake: node.get(EPIPHANY_CULTMESH_SWARM_BRAKE_KEY)?,
        contract_summaries,
    })
}

trait IntoLocalVerseContractSummary {
    fn into_local_verse_summary(self) -> EpiphanyLocalVerseContractSummary;
}

macro_rules! impl_local_verse_contract_summary {
    ($ty:ty) => {
        impl IntoLocalVerseContractSummary for $ty {
            fn into_local_verse_summary(self) -> EpiphanyLocalVerseContractSummary {
                EpiphanyLocalVerseContractSummary {
                    contract_id: self.contract_id,
                    verse_id: self.verse_id,
                    authority: self.authority,
                    document_type: self.document_type,
                    operations: self.operations,
                    receipt_document_types: self.receipt_document_types,
                }
            }
        }
    };
}

impl_local_verse_contract_summary!(crate::MindCultNetContract);
impl_local_verse_contract_summary!(crate::SubstrateGateCultNetContract);
impl_local_verse_contract_summary!(crate::EyesCultNetContract);
impl_local_verse_contract_summary!(crate::HandsCultNetContract);
impl_local_verse_contract_summary!(crate::SoulCultNetContract);
impl_local_verse_contract_summary!(crate::ContinuityCultNetContract);

fn epiphany_cultmesh_daemon_service_lifecycle_receipt_key(receipt_id: &str) -> String {
    format!("epiphany-local/daemon-service-lifecycle-receipt/{receipt_id}")
}

fn epiphany_cultmesh_daemon_service_lifecycle_receipt_current_key(service_id: &str) -> String {
    format!("epiphany-local/daemon-service-lifecycle-receipt/current/{service_id}")
}

fn epiphany_cultmesh_managed_service_policy_key(service_id: &str) -> String {
    format!("epiphany-local/managed-service-policy/{service_id}")
}

pub fn epiphany_verse_policies() -> Vec<EpiphanyVersePolicy> {
    vec![
        EpiphanyVersePolicy {
            verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
            tier: EPIPHANY_CULTMESH_INTERNAL_TIER.to_string(),
            purpose: "Sub-agent typed state: heartbeat, organ-state records, runtime-spine jobs, private receipts, and other Epiphany-owned organs.".to_string(),
            transport_scope: "single-host or trusted localhost mesh".to_string(),
            trust_boundary: "private Epiphany instance boundary".to_string(),
            private_state_allowed: true,
            untrusted_ingress_allowed: false,
            yggdrasil_tunnel_allowed: false,
        },
        EpiphanyVersePolicy {
            verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
            tier: EPIPHANY_CULTMESH_LOCAL_AREA_TIER.to_string(),
            purpose: "Trusted GameCult local-area sharing across projects, including operator-approved tunnels to services on Yggdrasil.".to_string(),
            transport_scope: "LAN plus explicit GameCult tunnel endpoints".to_string(),
            trust_boundary: "trusted GameCult project/runtime boundary".to_string(),
            private_state_allowed: false,
            untrusted_ingress_allowed: false,
            yggdrasil_tunnel_allowed: true,
        },
        EpiphanyVersePolicy {
            verse_id: EPIPHANY_CULTMESH_GLOBAL_VERSE_ID.to_string(),
            tier: EPIPHANY_CULTMESH_GLOBAL_TIER.to_string(),
            purpose: "Untrusted public surfaces: public dreams, questions, hypotheses, invitations, lineage, ingress receipts, and adoption receipts.".to_string(),
            transport_scope: "public internet".to_string(),
            trust_boundary: "untrusted public boundary".to_string(),
            private_state_allowed: false,
            untrusted_ingress_allowed: true,
            yggdrasil_tunnel_allowed: false,
        },
    ]
}

pub fn epiphany_global_room_policies() -> Vec<EpiphanyGlobalRoomPolicy> {
    [
        (
            "dreams",
            "Dreams",
            "Public dreams, symbolic fragments, imaginative pressure, and unfinished possible worlds.",
        ),
        (
            "architecture",
            "Architecture",
            "System design, ownership maps, protocol boundaries, and rejected machine shapes.",
        ),
        (
            "research",
            "Research",
            "Prior art, papers, source-grounded findings, and scout reports.",
        ),
        (
            "Personas",
            "Personas",
            "Public Persona identity, voice, social surface, and community-facing presence.",
        ),
        (
            "gamecult",
            "GameCult",
            "GameCult project coordination, public receipts, and cross-project questions.",
        ),
        (
            "governance",
            "Governance",
            "Public proposals and governance-adjacent discussion before any Bifrost adoption.",
        ),
    ]
    .into_iter()
    .map(|(slug, topic, purpose)| EpiphanyGlobalRoomPolicy {
        room_id: format!("epiphany-global/{slug}"),
        verse_id: EPIPHANY_CULTMESH_GLOBAL_VERSE_ID.to_string(),
        topic: topic.to_string(),
        purpose: purpose.to_string(),
        posting_policy:
            "Personas may post public, non-private, citation/provenance-bearing thread roots and replies; local adoption still requires review."
                .to_string(),
        threaded: true,
        persona_posting_allowed: true,
        untrusted_ingress_allowed: true,
    })
    .collect()
}

pub fn epiphany_cultmesh_cluster_topology() -> Vec<EpiphanyCultMeshClusterTopologyEntry> {
    epiphany_cultmesh_cluster_topology_for_body("repo:unbound")
}

pub fn epiphany_cultmesh_cluster_topology_for_body(
    body_domain: impl Into<String>,
) -> Vec<EpiphanyCultMeshClusterTopologyEntry> {
    let body_domain = body_domain.into();
    [
        ("self", "coordinator", "Self", false),
        ("hands", "implementation", "Hands", false),
        ("persona", "Persona", "Persona", true),
        ("imagination", "imagination", "Imagination", false),
        ("eyes", "research", "Eyes", false),
        ("modeling", "modeling", "Modeling", false),
        ("soul", "verification", "Soul", false),
    ]
    .into_iter()
    .map(
        |(cluster_slug, role_id, display_name, public_persona_discussion_allowed)| {
            let cluster_id = format!("epiphany.cluster.{cluster_slug}");
            EpiphanyCultMeshClusterTopologyEntry {
                schema_version: EPIPHANY_CULTMESH_CLUSTER_TOPOLOGY_SCHEMA_VERSION.to_string(),
                cluster_id: cluster_id.clone(),
                role_id: role_id.to_string(),
                display_name: display_name.to_string(),
                private_verse_id: format!("{cluster_id}.private"),
                body_domain: body_domain.clone(),
                body_kind: "repository".to_string(),
                daemon_id: format!("epiphany-daemon-{cluster_slug}"),
                daemon_surface_id: format!("epiphany-daemon-{cluster_slug}/local"),
                eve_surface_id: format!("eve://epiphany/{cluster_slug}"),
                public_persona_discussion_allowed,
                notes: vec![
                    format!(
                        "CultMesh advertises this cluster topology as {EPIPHANY_CULTMESH_CLUSTER_TOPOLOGY_TYPE}."
                    ),
                    "Private Verse carries cluster-local typed state and is not public collaboration weather.".to_string(),
                    "Odin may advertise compact metadata and Eve connection hints, but not private state payloads.".to_string(),
                    "The body domain names the substrate this cluster serves; Substrate Gate still governs repo access.".to_string(),
                ],
            }
        },
    )
    .collect()
}

pub fn write_epiphany_cultmesh_cluster_topology(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    body_domain: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshClusterTopologyEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for cluster in epiphany_cultmesh_cluster_topology_for_body(body_domain) {
        written.push(node.put(cluster.cluster_id.clone(), &cluster)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn load_epiphany_cultmesh_cluster_topology(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshClusterTopologyEntry>> {
    let store_path = store_path.as_ref();
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut topology = Vec::new();
    for cluster in epiphany_cultmesh_cluster_topology() {
        if let Some(loaded) =
            node.get::<EpiphanyCultMeshClusterTopologyEntry>(&cluster.cluster_id)?
        {
            topology.push(loaded);
        }
    }
    Ok(topology)
}

fn epiphany_cultmesh_daemon_heartbeat_event_key(heartbeat_id: &str) -> String {
    format!("epiphany-local/daemon-heartbeat/event/{heartbeat_id}")
}

fn epiphany_cultmesh_daemon_heartbeat_latest_key(daemon_id: &str) -> String {
    format!("epiphany-local/daemon-heartbeat/{daemon_id}/latest")
}

pub fn write_epiphany_cultmesh_daemon_heartbeat_event(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    event: EpiphanyCultMeshDaemonHeartbeatEventEntry,
) -> Result<EpiphanyCultMeshDaemonHeartbeatEventEntry> {
    validate_daemon_heartbeat_event(&event)?;
    let store_path = store_path.as_ref();
    let runtime_id = runtime_id.into();
    let event_key = epiphany_cultmesh_daemon_heartbeat_event_key(&event.heartbeat_id);
    let latest_key = epiphany_cultmesh_daemon_heartbeat_latest_key(&event.daemon_id);
    let backing = SingleFileMessagePackBackingStore::new(store_path);

    for _ in 0..8 {
        let node = open_epiphany_cultmesh_node(store_path, runtime_id.clone())?;
        if let Some(existing) = node.get::<EpiphanyCultMeshDaemonHeartbeatEventEntry>(&event_key)? {
            return if existing == event {
                Ok(existing)
            } else {
                Err(anyhow!(
                    "immutable daemon heartbeat identity collision for {:?}",
                    event.heartbeat_id
                ))
            };
        }
        let latest = node.get::<EpiphanyCultMeshDaemonHeartbeatEventEntry>(&latest_key)?;
        let advances_latest = match latest.as_ref() {
            Some(current) => daemon_heartbeat_advances(current, &event)?,
            None => true,
        };
        let event_envelope = node.cache().prepare_entry(&event_key, &event)?.0;
        let mut replacements = vec![event_envelope];
        let mut expected = Vec::new();
        if advances_latest {
            if let Some(envelope) = node
                .cache()
                .get_envelope::<EpiphanyCultMeshDaemonHeartbeatEventEntry>(&latest_key)?
            {
                expected.push(envelope);
            }
            replacements.push(node.cache().prepare_entry(&latest_key, &event)?.0);
        }
        if backing.compare_and_swap_batch(&expected, replacements)? {
            return Ok(event);
        }
    }
    Err(anyhow!(
        "daemon heartbeat latest advanced during publication"
    ))
}

pub fn load_epiphany_cultmesh_daemon_heartbeat_event(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    heartbeat_id: &str,
) -> Result<Option<EpiphanyCultMeshDaemonHeartbeatEventEntry>> {
    validate_heartbeat_identifier("heartbeat", heartbeat_id)?;
    open_epiphany_cultmesh_node(store_path, runtime_id)?
        .get(&epiphany_cultmesh_daemon_heartbeat_event_key(heartbeat_id))
}

pub fn load_latest_epiphany_cultmesh_daemon_heartbeat(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    daemon_id: &str,
) -> Result<Option<EpiphanyCultMeshDaemonHeartbeatEventEntry>> {
    validate_heartbeat_identifier("daemon", daemon_id)?;
    open_epiphany_cultmesh_node(store_path, runtime_id)?
        .get(&epiphany_cultmesh_daemon_heartbeat_latest_key(daemon_id))
}

#[allow(clippy::too_many_arguments)]
pub fn idunn_recover_memory_semantic_projection_from_cultmesh(
    verse_store: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    canonical_store: impl AsRef<Path>,
    input: &crate::MemorySemanticProjectionInput,
    expected_claim_id: &str,
    replacement_executor_id: &str,
    launch_lifecycle_receipt_id: &str,
    provider_heartbeat_id: &str,
    recovered_at: &str,
) -> Result<(
    crate::MemorySemanticProjectorRecoveryAuthorization,
    crate::MemorySemanticProjectionClaim,
)> {
    let verse_store = verse_store.as_ref();
    let runtime_id = runtime_id.into();
    let canonical_store = canonical_store.as_ref();
    crate::observe_memory_semantic_projection(canonical_store, input)?;

    let receipt = authenticate_epiphany_cultmesh_semantic_projector_launch(
        verse_store,
        runtime_id.clone(),
        launch_lifecycle_receipt_id,
    )?;
    let (policy, policy_digest) = load_epiphany_cultmesh_managed_service_policy_with_digest(
        verse_store,
        runtime_id.clone(),
        EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID,
    )?
    .ok_or_else(|| anyhow!("Idunn recovery managed service policy is absent"))?;
    let node = open_epiphany_cultmesh_node(verse_store, runtime_id)?;
    let receipt_key =
        epiphany_cultmesh_daemon_service_lifecycle_receipt_key(launch_lifecycle_receipt_id);

    let heartbeat_key = epiphany_cultmesh_daemon_heartbeat_event_key(provider_heartbeat_id);
    let heartbeat = node
        .get::<EpiphanyCultMeshDaemonHeartbeatEventEntry>(&heartbeat_key)?
        .ok_or_else(|| anyhow!("Idunn recovery provider heartbeat is absent"))?;
    validate_daemon_heartbeat_event(&heartbeat)?;
    if heartbeat.heartbeat_id != provider_heartbeat_id
        || heartbeat.daemon_id != receipt.provider_daemon_id
        || heartbeat.cluster_id != "local"
        || heartbeat.status != "ready"
        || heartbeat.startup_lifecycle_receipt_id != receipt.receipt_id
    {
        return Err(anyhow!("Idunn recovery provider heartbeat disagrees"));
    }
    let receipt_completed_at = receipt
        .completed_at_utc
        .as_deref()
        .ok_or_else(|| anyhow!("Idunn recovery launch receipt is not completed"))?;
    let receipt_completed = DateTime::parse_from_rfc3339(receipt_completed_at)?;
    let heartbeat_at = DateTime::parse_from_rfc3339(&heartbeat.heartbeat_at)?;
    if heartbeat_at <= receipt_completed {
        return Err(anyhow!(
            "Idunn recovery heartbeat must follow lifecycle completion"
        ));
    }

    let receipt_digest = cultmesh_envelope_digest::<
        EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
    >(&node, &receipt_key)?;
    let heartbeat_digest = cultmesh_envelope_digest::<EpiphanyCultMeshDaemonHeartbeatEventEntry>(
        &node,
        &heartbeat_key,
    )?;
    let evidence =
        crate::memory_graph::semantic_projector::idunn_semantic_recovery_evidence_from_cultmesh(
            canonical_store,
            input,
            expected_claim_id,
            &format!("idunn-{}", Uuid::new_v4()),
            &policy.policy_id,
            &policy_digest,
            &receipt.receipt_id,
            &receipt_digest,
            receipt_completed_at,
            &heartbeat.heartbeat_id,
            &heartbeat_digest,
            &heartbeat.provider_incarnation,
            &heartbeat.heartbeat_at,
            &heartbeat.startup_lifecycle_receipt_id,
        )?;
    crate::memory_graph::semantic_projector::idunn_recover_memory_semantic_projection(
        canonical_store,
        input,
        expected_claim_id,
        replacement_executor_id,
        &heartbeat.provider_incarnation,
        &evidence,
        recovered_at,
    )
}

fn cultmesh_envelope_digest<T: DatabaseEntry>(node: &CultMeshNode, key: &str) -> Result<String> {
    let envelope = node
        .cache()
        .get_envelope::<T>(key)?
        .ok_or_else(|| anyhow!("authenticated CultMesh evidence envelope disappeared"))?;
    let mut digest = Sha256::new();
    digest.update(envelope.r#type.as_bytes());
    digest.update([0]);
    digest.update(envelope.key.as_bytes());
    digest.update([0]);
    digest.update(&envelope.payload);
    Ok(format!("sha256-{:x}", digest.finalize()))
}

fn daemon_heartbeat_advances(
    current: &EpiphanyCultMeshDaemonHeartbeatEventEntry,
    candidate: &EpiphanyCultMeshDaemonHeartbeatEventEntry,
) -> Result<bool> {
    validate_daemon_heartbeat_event(current)?;
    let current_time = DateTime::parse_from_rfc3339(&current.heartbeat_at)?;
    let candidate_time = DateTime::parse_from_rfc3339(&candidate.heartbeat_at)?;
    if current.provider_incarnation == candidate.provider_incarnation {
        if candidate.sequence > current.sequence && candidate_time < current_time {
            return Err(anyhow!(
                "daemon heartbeat time regressed within provider incarnation"
            ));
        }
        if candidate.sequence <= current.sequence {
            return Ok(false);
        }
    }
    Ok(
        (candidate_time, candidate.sequence, &candidate.heartbeat_id)
            > (current_time, current.sequence, &current.heartbeat_id),
    )
}

fn validate_daemon_heartbeat_event(
    event: &EpiphanyCultMeshDaemonHeartbeatEventEntry,
) -> Result<()> {
    if event.schema_version != EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_SCHEMA_VERSION {
        return Err(anyhow!("unsupported daemon heartbeat schema"));
    }
    if event.daemon_id == EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID {
        return Err(anyhow!(
            "workspace coverage provider authority belongs to its specialized signed heartbeat"
        ));
    }
    validate_heartbeat_identifier("heartbeat", &event.heartbeat_id)?;
    validate_heartbeat_identifier("daemon", &event.daemon_id)?;
    validate_heartbeat_identifier("cluster", &event.cluster_id)?;
    validate_heartbeat_identifier("provider incarnation", &event.provider_incarnation)?;
    if !event.startup_lifecycle_receipt_id.is_empty() {
        validate_heartbeat_identifier(
            "startup lifecycle receipt",
            &event.startup_lifecycle_receipt_id,
        )?;
    }
    if event.sequence == 0 {
        return Err(anyhow!("daemon heartbeat sequence must be positive"));
    }
    if !matches!(event.status.as_str(), "ready" | "degraded" | "stopping") {
        return Err(anyhow!("invalid daemon heartbeat status"));
    }
    DateTime::parse_from_rfc3339(&event.heartbeat_at)
        .context("daemon heartbeat requires RFC3339 heartbeat_at")?;
    if event.private_state_exposed {
        return Err(anyhow!("daemon heartbeat must not expose private state"));
    }
    Ok(())
}

fn validate_heartbeat_identifier(label: &str, value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(anyhow!(
            "daemon heartbeat requires a bounded opaque {label} id"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cultcache_rs::CacheBackingStore;
    use pretty_assertions::assert_eq;

    #[test]
    fn reserved_semantic_projector_policy_requires_specialized_exact_writer() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("verse.ccmp");
        let binary = if cfg!(windows) {
            "C:\\epiphany-memory-semantic-projector.exe"
        } else {
            "/tmp/epiphany-memory-semantic-projector"
        };
        let exact = EpiphanyCultMeshManagedServicePolicyEntry {
            schema_version: EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION.to_string(),
            policy_id: "managed-service-policy-epiphany-memory-semantic-projector-service".into(),
            service_id: EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID.into(),
            command: binary.into(),
            args: vec![
                "serve",
                "--runtime-store",
                "modeling.ccmp",
                "--local-verse-store",
                "verse.ccmp",
                "--runtime-id",
                "local",
                "--interval-seconds",
                "60",
                "--qdrant-url",
                "http://127.0.0.1:16333",
                "--ollama-base-url",
                "http://10.77.0.1:11435",
                "--ollama-model",
                "qwen3-embedding:0.6b",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            cwd: None,
            enabled: true,
            cooldown_seconds: 0,
            stdout_artifact: "projector.stdout.log".into(),
            stderr_artifact: "projector.stderr.log".into(),
            private_state_exposed: false,
        };
        let historical_store = temp.path().join("historical-verse.ccmp");
        let mut historical = exact.clone();
        historical.schema_version = "epiphany.cultmesh.managed_service_policy.v0".into();
        let mut node = open_epiphany_cultmesh_node(&historical_store, "local")?;
        node.put(
            epiphany_cultmesh_managed_service_policy_key(&historical.service_id),
            &historical,
        )?;
        node.flush()?;
        assert!(
            load_epiphany_cultmesh_managed_service_policy(
                &historical_store,
                "local",
                EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID,
            )
            .expect_err("historical writable policy must refuse at the load boundary")
            .to_string()
            .contains("historical managed service policy schema")
        );
        let mut forged = exact.clone();
        forged.command = "arbitrary.exe".into();
        assert!(
            write_epiphany_cultmesh_semantic_projector_service_policy(&store, "local", forged)
                .is_err()
        );
        assert!(
            write_epiphany_cultmesh_semantic_projector_service_policy(
                &store,
                "local",
                exact.clone(),
            )
            .is_ok()
        );
        let (_, policy_digest) = load_epiphany_cultmesh_managed_service_policy_with_digest(
            &store,
            "local",
            EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID,
        )?
        .context("missing exact policy")?;
        let receipt = EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry {
            schema_version: EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION
                .into(),
            receipt_id: "9f63fa72-a2e1-4ca5-9c1a-9292b7798891".into(),
            service_id: EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID.into(),
            scheduler_id: "epiphany-daemon-supervisor".into(),
            runtime_id: "local".into(),
            daemon_selector: "epiphany-daemon-supervisor".into(),
            action: "launch".into(),
            status: "launched".into(),
            command: exact.command.clone(),
            args: exact.args.clone(),
            cwd: exact.cwd.clone(),
            process_id: Some(4242),
            exit_code: None,
            started_at_utc: "2026-07-15T12:00:00Z".into(),
            completed_at_utc: Some("2026-07-15T12:00:01Z".into()),
            operator_artifact_ref: "service://semantic-projector/launch".into(),
            private_state_exposed: false,
            notes: vec![],
            executable_sha256: "sha256-test-projector".into(),
            preflight_witness_id: String::new(),
            required_document_types: vec![],
            schema_preflight_passed: false,
            schema_catalog_sha256: String::new(),
            managed_policy_id: exact.policy_id.clone(),
            managed_policy_digest: policy_digest,
            provider_daemon_id: "epiphany-memory-semantic-projector".into(),
            startup_correlation_id: "9f63fa72-a2e1-4ca5-9c1a-9292b7798891".into(),
            process_creation_token: 1,
            process_created_at_rfc3339: None,
            process_executable_path: "C:\\epiphany\\semantic-projector.exe".into(),
        };
        let written = write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
            &store,
            "local",
            receipt.clone(),
        )?;
        assert_eq!(
            authenticate_epiphany_cultmesh_semantic_projector_launch(
                &store,
                "local",
                &receipt.receipt_id,
            )?,
            written
        );
        assert_eq!(
            write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
                &store,
                "local",
                receipt.clone(),
            )?,
            receipt
        );
        let mut collision = receipt;
        collision.process_id = Some(4343);
        assert!(
            write_epiphany_cultmesh_daemon_service_lifecycle_receipt(&store, "local", collision,)
                .unwrap_err()
                .to_string()
                .contains("identity collision")
        );
        Ok(())
    }

    #[test]
    fn reserved_workspace_coverage_projector_contract_is_exact_and_policy_bound() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("verse.ccmp");
        let binary = std::env::current_exe()?.with_file_name(if cfg!(windows) {
            "epiphany-workspace-coverage-projector.exe"
        } else {
            "epiphany-workspace-coverage-projector"
        });
        let exact = EpiphanyCultMeshManagedServicePolicyEntry {
            schema_version: EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION.to_string(),
            policy_id: "managed-service-policy-epiphany-workspace-coverage-projector-service"
                .into(),
            service_id: EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID.into(),
            command: binary.display().to_string(),
            args: vec![
                "serve",
                "--runtime-store",
                "runtime.ccmp",
                "--local-verse-store",
                "verse.ccmp",
                "--runtime-id",
                "local",
                "--interval-seconds",
                "60",
                "--heartbeat-interval-seconds",
                "10",
                "--qdrant-url",
                "http://127.0.0.1:6333",
                "--ollama-base-url",
                "http://127.0.0.1:11434",
                "--ollama-model",
                "qwen3-embedding:0.6b",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            cwd: None,
            enabled: true,
            cooldown_seconds: 0,
            stdout_artifact: "workspace-projector.stdout.log".into(),
            stderr_artifact: "workspace-projector.stderr.log".into(),
            private_state_exposed: false,
        };
        let mut arbitrary_binary = exact.clone();
        arbitrary_binary.command = "arbitrary-projector.exe".into();
        assert!(
            write_epiphany_cultmesh_workspace_coverage_projector_service_policy(
                &store,
                "local",
                arbitrary_binary,
            )
            .is_err()
        );
        let mut injected_workspace = exact.clone();
        injected_workspace.args.insert(3, "--workspace".into());
        injected_workspace.args.insert(4, "stolen".into());
        assert!(
            write_epiphany_cultmesh_workspace_coverage_projector_service_policy(
                &store,
                "local",
                injected_workspace,
            )
            .is_err()
        );
        write_epiphany_cultmesh_workspace_coverage_projector_service_policy(
            &store,
            "local",
            exact.clone(),
        )?;
        let receipt_id = "fd3b7be9-02b0-4ac7-a47e-2d25097ff1f5";
        let receipt = EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry {
            schema_version: EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION
                .into(),
            receipt_id: receipt_id.into(),
            service_id: EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID.into(),
            scheduler_id: "epiphany-daemon-supervisor".into(),
            runtime_id: "local".into(),
            daemon_selector: "epiphany-daemon-supervisor".into(),
            action: "launch".into(),
            status: "launched".into(),
            command: exact.command.clone(),
            args: exact.args.clone(),
            cwd: exact.cwd.clone(),
            process_id: Some(4242),
            exit_code: None,
            started_at_utc: "2026-07-15T12:00:00Z".into(),
            completed_at_utc: Some("2026-07-15T12:00:01Z".into()),
            operator_artifact_ref: "service://workspace-coverage-projector/launch".into(),
            private_state_exposed: false,
            notes: vec![],
            executable_sha256: "sha256-test-workspace-projector".into(),
            preflight_witness_id: String::new(),
            required_document_types: vec![],
            schema_preflight_passed: false,
            schema_catalog_sha256: String::new(),
            managed_policy_id: exact.policy_id.clone(),
            managed_policy_digest: "sha256-dead-authority".into(),
            provider_daemon_id: EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID.into(),
            startup_correlation_id: receipt_id.into(),
            process_creation_token: 1,
            process_created_at_rfc3339: None,
            process_executable_path: "C:\\epiphany\\semantic-projector.exe".into(),
        };
        let mut wrong_provider = receipt.clone();
        wrong_provider.receipt_id = "f6d454dd-3765-44cb-930a-bae0d47487aa".into();
        wrong_provider.startup_correlation_id = wrong_provider.receipt_id.clone();
        wrong_provider.provider_daemon_id = "epiphany-memory-semantic-projector".into();
        assert!(
            write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
                &store,
                "local",
                wrong_provider,
            )
            .is_err()
        );
        let mut stale = receipt.clone();
        stale.receipt_id = "a0ea76d1-9a9a-4dc7-a8bc-a56ab1d8079a".into();
        stale.startup_correlation_id = stale.receipt_id.clone();
        stale.managed_policy_digest = "sha256-stale".into();
        assert!(
            write_epiphany_cultmesh_daemon_service_lifecycle_receipt(&store, "local", stale)
                .is_err()
        );
        assert!(
            write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
                &store,
                "local",
                receipt.clone(),
            )
            .unwrap_err()
            .to_string()
            .contains("specialized managed process documents")
        );

        let mut advanced = exact;
        advanced.args[8] = "61".into();
        write_epiphany_cultmesh_workspace_coverage_projector_service_policy(
            &store, "local", advanced,
        )?;
        assert!(
            load_epiphany_cultmesh_daemon_service_lifecycle_receipt(&store, "local", receipt_id,)?
                .is_none()
        );
        let generic_heartbeat = EpiphanyCultMeshDaemonHeartbeatEventEntry {
            schema_version: EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_SCHEMA_VERSION.to_string(),
            heartbeat_id: Uuid::new_v4().to_string(),
            daemon_id: EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID.to_string(),
            cluster_id: "local".to_string(),
            provider_incarnation: Uuid::new_v4().to_string(),
            sequence: 1,
            status: "ready".to_string(),
            heartbeat_at: "2026-07-15T12:00:03Z".to_string(),
            private_state_exposed: false,
            startup_lifecycle_receipt_id: receipt_id.to_string(),
        };
        assert!(
            write_epiphany_cultmesh_daemon_heartbeat_event(&store, "local", generic_heartbeat,)
                .unwrap_err()
                .to_string()
                .contains("specialized signed heartbeat")
        );
        Ok(())
    }

    fn semantic_health_input(
        store: &Path,
        swarm_id: &str,
        partition: &str,
        generation: u64,
    ) -> Result<crate::MemorySemanticProjectionInput> {
        let graph_id = format!("{partition}-graph");
        let obligation = crate::MemorySemanticProjectionObligation {
            schema_version: crate::MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION.to_string(),
            obligation_id: format!("obligation-{partition}-{generation}"),
            swarm_id: swarm_id.to_string(),
            partition: partition.to_string(),
            canonical_source_id: format!("canonical/{partition}"),
            source_commit_id: format!("commit-{generation}"),
            graph_id: graph_id.clone(),
            source_generation: generation,
            source_model_hash: format!("model-{generation}"),
            canonical_content_set_hash: format!("content-{generation}"),
            projection_schema_version: crate::SEMANTIC_PROJECTION_SCHEMA_VERSION.to_string(),
            created_at: "2026-07-15T12:00:00Z".to_string(),
        };
        let head = crate::MemorySemanticProjectionSourceHead {
            swarm_id: swarm_id.to_string(),
            partition: partition.to_string(),
            canonical_source_id: format!("canonical/{partition}"),
            source_commit_id: format!("commit-{generation}"),
            graph_id: graph_id.clone(),
            source_generation: generation,
            source_model_hash: format!("model-{generation}"),
            canonical_content_set_hash: format!("content-{generation}"),
        };
        let mut cache = crate::memory_graph::semantic_projector::semantic_projector_cache(store)?;
        cache.put(&obligation.obligation_id, &obligation)?;
        let envelopes = SingleFileMessagePackBackingStore::new(store).pull_all()?;
        let authority = envelopes
            .into_iter()
            .find(|row| {
                row.r#type == "gamecult.epiphany.memory_semantic_projection_obligation"
                    && row.key == obligation.obligation_id
            })
            .expect("persisted obligation envelope");
        Ok(crate::MemorySemanticProjectionInput {
            snapshot: crate::EpiphanyMemoryGraphSnapshot {
                schema_version: Some("v0".to_string()),
                graph_id,
                model_revision: generation,
                ..Default::default()
            },
            obligation,
            authority:
                crate::memory_graph::semantic_projector::MemorySemanticProjectionAuthoritySnapshot {
                    head,
                    envelopes: vec![authority],
                },
        })
    }

    #[test]
    fn semantic_health_publication_is_monotonic_sight_only() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let canonical = temp.path().join("canonical.msgpack");
        let verse = temp.path().join("verse.ccmp");
        let modeling_1 = semantic_health_input(&canonical, "swarm-a", "modeling", 1)?;
        let modeling_2 = semantic_health_input(&canonical, "swarm-a", "modeling", 2)?;
        let before = SingleFileMessagePackBackingStore::new(&canonical).pull_all()?;

        publish_epiphany_cultmesh_semantic_projection_health(
            &verse,
            "runtime",
            &canonical,
            &modeling_1,
            "incarnation-a",
        )?;
        publish_epiphany_cultmesh_semantic_projection_health(
            &verse,
            "runtime",
            &canonical,
            &modeling_2,
            "incarnation-a",
        )?;
        let delayed = publish_epiphany_cultmesh_semantic_projection_health(
            &verse,
            "runtime",
            &canonical,
            &modeling_1,
            "incarnation-a",
        )?;
        assert_eq!(delayed.source_generation, 2);
        let rows = load_epiphany_cultmesh_semantic_projection_health(&verse, "runtime")?;
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].partition, "modeling");
        assert_eq!(rows[0].source_generation, 2);
        assert!(rows.iter().all(|row| !row.private_state_exposed));
        assert_eq!(
            SingleFileMessagePackBackingStore::new(&canonical).pull_all()?,
            before,
            "publishing sight must not create canonical projection work"
        );
        Ok(())
    }

    #[test]
    fn semantic_health_rejects_older_conflicting_obligation_for_same_generation() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let canonical = temp.path().join("canonical.msgpack");
        let verse = temp.path().join("verse.ccmp");
        let mut latest = semantic_health_input(&canonical, "swarm-a", "modeling", 2)?;
        latest.obligation.created_at = "2026-07-15T12:02:00Z".to_string();
        let mut cache =
            crate::memory_graph::semantic_projector::semantic_projector_cache(&canonical)?;
        cache.put(&latest.obligation.obligation_id, &latest.obligation)?;
        latest.authority.envelopes = SingleFileMessagePackBackingStore::new(&canonical)
            .pull_all()?
            .into_iter()
            .filter(|row| {
                row.r#type == "gamecult.epiphany.memory_semantic_projection_obligation"
                    && row.key == latest.obligation.obligation_id
            })
            .collect();
        publish_epiphany_cultmesh_semantic_projection_health(
            &verse,
            "runtime",
            &canonical,
            &latest,
            "incarnation-a",
        )?;

        let mut older_conflict = latest.clone();
        older_conflict.obligation.obligation_id = "conflicting-obligation-modeling-2".to_string();
        older_conflict.obligation.created_at = "2026-07-15T12:01:00Z".to_string();
        cache.put(
            &older_conflict.obligation.obligation_id,
            &older_conflict.obligation,
        )?;
        older_conflict.authority.envelopes = SingleFileMessagePackBackingStore::new(&canonical)
            .pull_all()?
            .into_iter()
            .filter(|row| {
                row.r#type == "gamecult.epiphany.memory_semantic_projection_obligation"
                    && row.key == older_conflict.obligation.obligation_id
            })
            .collect();

        let error = publish_epiphany_cultmesh_semantic_projection_health(
            &verse,
            "runtime",
            &canonical,
            &older_conflict,
            "incarnation-a",
        )
        .expect_err("chronology must not hide a same-generation obligation conflict");
        assert!(
            error
                .to_string()
                .contains("conflicting canonical obligations")
        );
        Ok(())
    }

    #[test]
    fn semantic_health_loader_rejects_latest_outside_declared_scope() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let canonical = temp.path().join("canonical.msgpack");
        let verse = temp.path().join("verse.ccmp");
        let input = semantic_health_input(&canonical, "swarm-a", "modeling", 1)?;
        let row = publish_epiphany_cultmesh_semantic_projection_health(
            &verse,
            "runtime",
            &canonical,
            &input,
            "incarnation-a",
        )?;
        let hostile_key = format!(
            "{}/latest",
            semantic_projection_health_scope_key("other-swarm", "modeling")
        );
        let mut node = open_epiphany_cultmesh_node(&verse, "hostile")?;
        node.put(hostile_key, &row)?;
        node.flush()?;

        let error = load_epiphany_cultmesh_semantic_projection_health(&verse, "runtime")
            .expect_err("a latest row must authenticate its key scope");
        assert!(
            error
                .to_string()
                .contains("does not match its declared scope")
        );
        Ok(())
    }

    #[test]
    fn forged_ready_health_cannot_mint_canonical_readiness() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let canonical = temp.path().join("canonical.msgpack");
        let verse = temp.path().join("verse.ccmp");
        let input = semantic_health_input(&canonical, "swarm-a", "modeling", 1)?;
        let mut empty_authority = input.clone();
        empty_authority.authority.envelopes.clear();
        assert!(
            publish_epiphany_cultmesh_semantic_projection_health(
                &verse,
                "runtime",
                &canonical,
                &empty_authority,
                "incarnation-a"
            )
            .is_err()
        );
        let other_store = temp.path().join("other.msgpack");
        assert!(
            publish_epiphany_cultmesh_semantic_projection_health(
                &verse,
                "runtime",
                &other_store,
                &input,
                "incarnation-a"
            )
            .is_err()
        );
        let mut forged = publish_epiphany_cultmesh_semantic_projection_health(
            &verse,
            "runtime",
            &canonical,
            &input,
            "incarnation-a",
        )?;
        forged.status = "ready".to_string();
        forged.receipt_id = Some("forged-receipt".to_string());
        forged.indexed_document_count = Some(999);
        forged.vector_dimensions = Some(999);
        let mut node = open_epiphany_cultmesh_node(&verse, "hostile")?;
        node.put("gamecult-local/hostile/forged-ready", &forged)?;
        node.flush()?;

        assert!(
            crate::load_memory_semantic_projection_readiness(&canonical, &input)?.is_none(),
            "CultMesh mirrors are not an import edge into canonical readiness"
        );
        let mut config = crate::MemorySemanticIndexConfig::from_env();
        config.qdrant_url = "http://127.0.0.1:1".to_string();
        let packet = crate::semantic_memory_context(
            input.snapshot(),
            "swarm-a",
            &crate::EpiphanyMemoryContextQuery {
                id: "hostile-query".to_string(),
                text: Some("test".to_string()),
                ..Default::default()
            },
            None,
            &config,
        );
        assert!(
            packet
                .warnings
                .iter()
                .any(|warning| warning.contains("canonical BM25"))
        );
        Ok(())
    }

    #[test]
    fn semantic_health_preserves_projection_states_and_later_repair_failure() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let canonical = temp.path().join("canonical.msgpack");
        let verse = temp.path().join("verse.ccmp");
        let input = semantic_health_input(&canonical, "swarm-four", "modeling", 7)?;
        let pending = publish_epiphany_cultmesh_semantic_projection_health(
            &verse,
            "provider",
            &canonical,
            &input,
            "incarnation-a",
        )?;
        assert_eq!(pending.status, "pending");

        let claim =
            crate::memory_graph::semantic_projector::idunn_acquire_memory_semantic_projection(
                &canonical,
                &input,
                "executor-a",
                "executor-a-incarnation",
                "execute",
                "idunn-test-incarnation",
                "2026-07-15T12:01:00Z",
            )?
            .claim;
        let raw_receipt = crate::MemorySemanticIndexReceipt {
            schema_version: crate::MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "receipt-four-state".to_string(),
            swarm_id: input.obligation().swarm_id.clone(),
            partition: input.obligation().partition.clone(),
            collection_name: "projection".to_string(),
            graph_id: input.obligation().graph_id.clone(),
            model_revision: input.obligation().source_generation,
            model_hash: input.obligation().source_model_hash.clone(),
            embedding_provider_id: "embedder".to_string(),
            embedding_model: "model".to_string(),
            vector_dimensions: 3,
            indexed_document_count: 2,
            deleted_document_count: 0,
            canonical_content_set_hash: input.obligation().canonical_content_set_hash.clone(),
            indexed_at: "2026-07-15T12:02:00Z".to_string(),
            status: "ready".to_string(),
            obligation_id: claim.obligation_id.clone(),
            canonical_source_id: String::new(),
            source_commit_id: String::new(),
            source_generation: input.obligation().source_generation,
            projection_schema_version: crate::SEMANTIC_PROJECTION_SCHEMA_VERSION.to_string(),
            claim_id: claim.claim_id.clone(),
            claim_epoch: claim.epoch,
            observed_vector_binding_root_sha256: "0".repeat(64),
        };
        crate::memory_graph::semantic_projector::succeed_memory_semantic_projection_claim(
            &canonical,
            &claim.claim_id,
            &input.authority,
            raw_receipt,
            "2026-07-15T12:02:01Z",
        )?;
        let ready = publish_epiphany_cultmesh_semantic_projection_health(
            &verse,
            "provider",
            &canonical,
            &input,
            "incarnation-a",
        )?;
        assert_eq!(ready.status, "ready");
        assert!(ready.query_eligible_display_only);
        let mut empty_ready = ready.clone();
        empty_ready.indexed_document_count = Some(0);
        empty_ready.vector_dimensions = Some(0);
        empty_ready.query_eligible_display_only = false;
        validate_semantic_projection_health(&empty_ready)?;

        let repair =
            crate::memory_graph::semantic_projector::idunn_acquire_memory_semantic_projection(
                &canonical,
                &input,
                "executor-b",
                "executor-b-incarnation",
                "repair",
                "idunn-test-incarnation",
                "2026-07-15T12:03:00Z",
            )?
            .claim;
        crate::memory_graph::semantic_projector::fail_memory_semantic_projection_claim(
            &canonical,
            &repair.claim_id,
            "2026-07-15T12:04:00Z",
            "private backend failure /secret/path",
        )?;
        let failed = publish_epiphany_cultmesh_semantic_projection_health(
            &verse,
            "provider",
            &canonical,
            &input,
            "incarnation-a",
        )?;
        assert_eq!(failed.status, "failed");
        assert!(!failed.query_eligible_display_only);
        assert!(failed.receipt_id.is_none());
        let encoded = format!("{failed:?}");
        assert!(!encoded.contains("private backend failure"));
        assert!(!encoded.contains("/secret/path"));

        let mut stale_input = input.clone();
        stale_input.authority.head.source_generation += 1;
        assert!(crate::observe_memory_semantic_projection(&canonical, &stale_input).is_err());
        assert!(
            publish_epiphany_cultmesh_semantic_projection_health(
                &verse,
                "provider",
                &canonical,
                &stale_input,
                "incarnation-a"
            )
            .is_err()
        );
        assert!(
            publish_epiphany_cultmesh_semantic_projection_health(
                &verse,
                "provider",
                &canonical,
                &input,
                "C:\\secret\\token"
            )
            .is_err()
        );

        Ok(())
    }

    #[test]
    fn service_lifecycle_receipt_refuses_invalid_or_reversed_time() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp
            .path()
            .join("epiphany-service-lifecycle-invalid-time.ccmp");
        let mut receipt = EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry {
            schema_version: EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION
                .to_string(),
            receipt_id: "service-lifecycle-invalid-time".to_string(),
            service_id: "epiphany-daemon-supervisor-service".to_string(),
            scheduler_id: "epiphany-daemon-supervisor".to_string(),
            runtime_id: "epiphany-test".to_string(),
            daemon_selector: "epiphany-daemon-supervisor".to_string(),
            action: "windows-service-status".to_string(),
            status: "running".to_string(),
            command: "powershell.exe".to_string(),
            args: Vec::new(),
            cwd: None,
            process_id: None,
            exit_code: Some(0),
            started_at_utc: "not-a-time".to_string(),
            completed_at_utc: None,
            operator_artifact_ref: "test://invalid-time".to_string(),
            private_state_exposed: false,
            notes: Vec::new(),
            executable_sha256: String::new(),
            preflight_witness_id: String::new(),
            required_document_types: Vec::new(),
            schema_preflight_passed: false,
            schema_catalog_sha256: String::new(),
            managed_policy_id: String::new(),
            managed_policy_digest: String::new(),
            provider_daemon_id: String::new(),
            startup_correlation_id: String::new(),
            process_creation_token: 0,
            process_created_at_rfc3339: None,
            process_executable_path: String::new(),
        };
        assert!(
            write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
                &store,
                "epiphany-test",
                receipt.clone(),
            )
            .is_err()
        );
        receipt.started_at_utc = "2026-07-13T02:00:00Z".to_string();
        receipt.completed_at_utc = Some("2026-07-13T01:00:00Z".to_string());
        assert!(
            write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
                &store,
                "epiphany-test",
                receipt,
            )
            .is_err()
        );
        assert!(
            load_current_epiphany_cultmesh_daemon_service_lifecycle_receipt_for_service(
                &store,
                "epiphany-test",
                "epiphany-daemon-supervisor-service",
            )?
            .is_none()
        );
        Ok(())
    }

    #[test]
    fn diagnostic_loaders_do_not_materialize_missing_body_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let missing_parent = temp.path().join("missing-body");
        let store = missing_parent.join("missing-local-verse.ccmp");

        assert!(load_epiphany_cultmesh_cluster_topology(&store, "epiphany-test")?.is_empty());
        assert!(
            !store.exists(),
            "read-only diagnostic loaders must not create a CultCache store"
        );
        assert!(
            !missing_parent.exists(),
            "read-only diagnostic loaders must not create the store parent"
        );
        let error = query_epiphany_local_verse_context(&store, "epiphany-test")
            .expect_err("a missing Verse cannot project a context");
        assert!(error.to_string().contains("store does not exist"));
        assert!(!missing_parent.exists());
        Ok(())
    }

    #[test]
    fn canonical_engagement_protects_daemon_lifecycle_actuation() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-canonical-swarm-brake.ccmp");
        let brake = engage_epiphany_cultmesh_swarm_brake(
            &store,
            "epiphany-test",
            "Hold every consequence surface during shakedown.",
            "Idunn",
            "2026-08-11T00:00:00Z",
            false,
        )?;

        assert_eq!(brake.status, "engaged");
        assert_eq!(brake.scope, "all");
        assert!(
            brake
                .protected_surfaces
                .iter()
                .any(|surface| surface == "daemon.lifecycle_poke")
        );
        Ok(())
    }

    #[test]
    fn swarm_brake_refuses_private_state_or_unreasoned_engagement() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-swarm-brake-refusal.ccmp");
        let mut brake = default_epiphany_cultmesh_swarm_brake("2026-06-17T00:00:00Z");
        brake.private_state_exposed = true;
        let error = write_epiphany_cultmesh_swarm_brake(&store, "epiphany-test", brake)
            .expect_err("private swarm brake must be refused");
        assert!(error.to_string().contains("private state"));

        let mut brake = default_epiphany_cultmesh_swarm_brake("2026-06-17T00:00:00Z");
        brake.status = "engaged".to_string();
        brake.reason.clear();
        let error = write_epiphany_cultmesh_swarm_brake(&store, "epiphany-test", brake)
            .expect_err("unreasoned engaged swarm brake must be refused");
        assert!(error.to_string().contains("operator id and reason"));
        Ok(())
    }

    fn heartbeat_event(
        heartbeat_id: &str,
        incarnation: &str,
        sequence: u64,
        heartbeat_at: &str,
    ) -> EpiphanyCultMeshDaemonHeartbeatEventEntry {
        EpiphanyCultMeshDaemonHeartbeatEventEntry {
            schema_version: EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_SCHEMA_VERSION.to_string(),
            heartbeat_id: heartbeat_id.to_string(),
            daemon_id: "daemon-test".to_string(),
            cluster_id: "cluster-test".to_string(),
            provider_incarnation: incarnation.to_string(),
            sequence,
            status: "ready".to_string(),
            heartbeat_at: heartbeat_at.to_string(),
            private_state_exposed: false,
            startup_lifecycle_receipt_id: String::new(),
        }
    }

    #[test]
    fn daemon_heartbeat_events_are_immutable_and_advance_latest_monotonically() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("daemon-heartbeats.ccmp");
        let first = heartbeat_event("heartbeat-1", "incarnation-a", 1, "2026-07-15T12:00:00Z");
        write_epiphany_cultmesh_daemon_heartbeat_event(&store, "runtime-test", first.clone())?;
        assert_eq!(
            load_epiphany_cultmesh_daemon_heartbeat_event(&store, "runtime-test", "heartbeat-1")?,
            Some(first.clone())
        );

        let delayed = heartbeat_event(
            "heartbeat-delayed",
            "incarnation-a",
            1,
            "2026-07-15T11:59:59Z",
        );
        write_epiphany_cultmesh_daemon_heartbeat_event(&store, "runtime-test", delayed)?;
        assert_eq!(
            load_latest_epiphany_cultmesh_daemon_heartbeat(&store, "runtime-test", "daemon-test")?,
            Some(first.clone())
        );

        let restarted = heartbeat_event("heartbeat-2", "incarnation-b", 1, "2026-07-15T12:00:01Z");
        write_epiphany_cultmesh_daemon_heartbeat_event(&store, "runtime-test", restarted.clone())?;
        assert_eq!(
            load_latest_epiphany_cultmesh_daemon_heartbeat(&store, "runtime-test", "daemon-test")?,
            Some(restarted)
        );

        let mut collision = first.clone();
        collision.status = "degraded".to_string();
        assert!(
            write_epiphany_cultmesh_daemon_heartbeat_event(&store, "runtime-test", collision)
                .expect_err("heartbeat identity is immutable")
                .to_string()
                .contains("identity collision")
        );
        let mut private = heartbeat_event(
            "heartbeat-private",
            "incarnation-b",
            2,
            "2026-07-15T12:00:02Z",
        );
        private.private_state_exposed = true;
        assert!(
            write_epiphany_cultmesh_daemon_heartbeat_event(&store, "runtime-test", private)
                .expect_err("private heartbeat must be refused")
                .to_string()
                .contains("must not expose private state")
        );
        Ok(())
    }

    #[test]
    fn semantic_recovery_requires_current_policy_launch_heartbeat_chain_and_is_single_use()
    -> Result<()> {
        let temp = tempfile::tempdir()?;
        let canonical = temp.path().join("canonical.msgpack");
        let verse = temp.path().join("verse.ccmp");
        let input = semantic_health_input(&canonical, "swarm-recovery", "modeling", 1)?;
        let claim =
            crate::memory_graph::semantic_projector::idunn_acquire_memory_semantic_projection(
                &canonical,
                &input,
                "executor-old",
                "provider-old",
                "execute",
                "idunn-test-incarnation",
                "2026-07-15T12:00:00Z",
            )?
            .claim;
        let binary = if cfg!(windows) {
            "C:\\epiphany-memory-semantic-projector.exe"
        } else {
            "/tmp/epiphany-memory-semantic-projector"
        };
        let policy = EpiphanyCultMeshManagedServicePolicyEntry {
            schema_version: EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION.to_string(),
            policy_id: "managed-service-policy-epiphany-memory-semantic-projector-service".into(),
            service_id: EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID.into(),
            command: binary.into(),
            args: vec![
                "serve",
                "--runtime-store",
                "modeling.ccmp",
                "--local-verse-store",
                "verse.ccmp",
                "--runtime-id",
                "runtime-test",
                "--interval-seconds",
                "60",
                "--qdrant-url",
                "http://127.0.0.1:16333",
                "--ollama-base-url",
                "http://10.77.0.1:11435",
                "--ollama-model",
                "qwen3-embedding:0.6b",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            cwd: None,
            enabled: true,
            cooldown_seconds: 0,
            stdout_artifact: "projector.stdout.log".into(),
            stderr_artifact: "projector.stderr.log".into(),
            private_state_exposed: false,
        };
        write_epiphany_cultmesh_semantic_projector_service_policy(
            &verse,
            "runtime-test",
            policy.clone(),
        )?;
        let (_, policy_digest) = load_epiphany_cultmesh_managed_service_policy_with_digest(
            &verse,
            "runtime-test",
            EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID,
        )?
        .context("semantic policy missing")?;
        let receipt = EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry {
            schema_version: EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION
                .into(),
            receipt_id: "f32666a9-94ce-47c5-b2bd-7d18624dfe9b".into(),
            service_id: EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID.into(),
            scheduler_id: "epiphany-daemon-supervisor".into(),
            runtime_id: "runtime-test".into(),
            daemon_selector: "epiphany-daemon-supervisor".into(),
            action: "launch".into(),
            status: "launched".into(),
            command: policy.command.clone(),
            args: policy.args.clone(),
            cwd: None,
            process_id: Some(4242),
            exit_code: None,
            started_at_utc: "2026-07-15T12:01:00Z".into(),
            completed_at_utc: Some("2026-07-15T12:02:00Z".into()),
            operator_artifact_ref: "service://semantic-projector/launch".into(),
            private_state_exposed: false,
            notes: vec![],
            executable_sha256: "sha256-test-projector".into(),
            preflight_witness_id: String::new(),
            required_document_types: vec![],
            schema_preflight_passed: false,
            schema_catalog_sha256: String::new(),
            managed_policy_id: policy.policy_id.clone(),
            managed_policy_digest: policy_digest,
            provider_daemon_id: "epiphany-memory-semantic-projector".into(),
            startup_correlation_id: "f32666a9-94ce-47c5-b2bd-7d18624dfe9b".into(),
            process_creation_token: 1,
            process_created_at_rfc3339: None,
            process_executable_path: "C:\\epiphany\\semantic-projector.exe".into(),
        };
        write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
            &verse,
            "runtime-test",
            receipt.clone(),
        )?;

        let unrelated = EpiphanyCultMeshDaemonHeartbeatEventEntry {
            schema_version: EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_SCHEMA_VERSION.to_string(),
            heartbeat_id: "semantic-heartbeat-unrelated".to_string(),
            daemon_id: "epiphany-memory-semantic-projector".into(),
            cluster_id: "local".into(),
            provider_incarnation: "provider-new".to_string(),
            sequence: 1,
            status: "ready".to_string(),
            heartbeat_at: "2026-07-15T12:03:00Z".to_string(),
            private_state_exposed: false,
            startup_lifecycle_receipt_id: String::new(),
        };
        write_epiphany_cultmesh_daemon_heartbeat_event(&verse, "runtime-test", unrelated)?;
        assert!(
            idunn_recover_memory_semantic_projection_from_cultmesh(
                &verse,
                "runtime-test",
                &canonical,
                &input,
                &claim.claim_id,
                "executor-new",
                &receipt.receipt_id,
                "semantic-heartbeat-unrelated",
                "2026-07-15T12:04:00Z",
            )
            .is_err()
        );

        let mut advanced_policy = policy.clone();
        advanced_policy.args[8] = "61".into();
        write_epiphany_cultmesh_semantic_projector_service_policy(
            &verse,
            "runtime-test",
            advanced_policy,
        )?;
        assert!(
            idunn_recover_memory_semantic_projection_from_cultmesh(
                &verse,
                "runtime-test",
                &canonical,
                &input,
                &claim.claim_id,
                "executor-new",
                &receipt.receipt_id,
                "semantic-heartbeat-unrelated",
                "2026-07-15T12:04:00Z",
            )
            .expect_err("an obsolete launch receipt cannot authorize a newer policy")
            .to_string()
            .contains("disagrees with current managed policy")
        );
        write_epiphany_cultmesh_semantic_projector_service_policy(&verse, "runtime-test", policy)?;

        let correlated = EpiphanyCultMeshDaemonHeartbeatEventEntry {
            heartbeat_id: "semantic-heartbeat-correlated".to_string(),
            sequence: 2,
            startup_lifecycle_receipt_id: receipt.receipt_id.clone(),
            ..heartbeat_event(
                "semantic-heartbeat-template",
                "provider-new",
                2,
                "2026-07-15T12:03:00Z",
            )
        };
        let mut correlated = correlated;
        correlated.daemon_id = "epiphany-memory-semantic-projector".into();
        correlated.cluster_id = "local".into();
        write_epiphany_cultmesh_daemon_heartbeat_event(&verse, "runtime-test", correlated.clone())?;
        let (_, recovered) = idunn_recover_memory_semantic_projection_from_cultmesh(
            &verse,
            "runtime-test",
            &canonical,
            &input,
            &claim.claim_id,
            "executor-new",
            &receipt.receipt_id,
            &correlated.heartbeat_id,
            "2026-07-15T12:04:00Z",
        )?;
        assert_eq!(recovered.epoch, claim.epoch + 1);
        assert_eq!(recovered.executor_incarnation, "provider-new");
        assert!(
            idunn_recover_memory_semantic_projection_from_cultmesh(
                &verse,
                "runtime-test",
                &canonical,
                &input,
                &recovered.claim_id,
                "executor-third",
                &receipt.receipt_id,
                &correlated.heartbeat_id,
                "2026-07-15T12:05:00Z",
            )
            .is_err()
        );
        Ok(())
    }
}
