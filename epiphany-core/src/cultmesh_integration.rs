use crate::EpiphanyAgentStateSoaEntry;
use crate::default_continuity_cultnet_contracts;
use crate::default_eyes_cultnet_contracts;
use crate::default_hands_cultnet_contracts;
use crate::default_mind_cultnet_contracts;
use crate::default_soul_cultnet_contracts;
use crate::default_substrate_gate_cultnet_contracts;
use crate::packaged_release::{EpiphanyPackagedReleaseEntry, EpiphanyPackagedReleaseHead};
use crate::workspace_coverage_process_documents::{
    WORKSPACE_COVERAGE_PROCESS_EVIDENCE_HEAD_SCHEMA_VERSION,
    WORKSPACE_COVERAGE_PROCESS_LAUNCH_SCHEMA_VERSION,
    WORKSPACE_COVERAGE_PROCESS_TERMINATION_SCHEMA_VERSION,
    WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_SCHEMA_VERSION,
    WorkspaceCoverageManagedProcessLaunchEntry, WorkspaceCoverageProcessEvidenceHead,
    WorkspaceCoverageProcessTerminationObservationEntry, WorkspaceCoverageProviderHeartbeatEntry,
};
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use chrono::DateTime;
use chrono::FixedOffset;
use chrono::Utc;
use cultcache_rs::CacheBackingStore;
use cultcache_rs::CultSoaColumnValues;
use cultcache_rs::DatabaseEntry;
use cultcache_rs::SingleFileMessagePackBackingStore;
use cultcache_rs::SoaDocument;
use cultmesh_rs::CultMesh;
use cultmesh_rs::CultMeshNode;
use cultmesh_rs::CultMeshNodeOptions;
use cultmesh_rs::cultmesh_documents;
use serde::Serialize;
use serde_json::Value;
use sha2::Digest;
use sha2::Sha256;
use std::path::Path;
use uuid::Uuid;

pub const EPIPHANY_CULTMESH_STATUS_TYPE: &str = "epiphany.cultmesh.status";
pub const EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION: &str = "epiphany.cultmesh.status.v0";
pub const EPIPHANY_CULTMESH_STATUS_KEY: &str = "epiphany-local/status";
pub const EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_TYPE: &str = "epiphany.cultmesh.operator_snapshot";
pub const EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.operator_snapshot.v0";
pub const EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_LATEST_KEY: &str =
    "epiphany-local/operator-snapshot/latest";
pub const EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_TYPE: &str =
    "epiphany.cultmesh.operator_run_intent";
pub const EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.operator_run_intent.v0";
pub const EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_LATEST_KEY: &str =
    "epiphany-local/operator-run-intent/latest";
pub const EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_TYPE: &str =
    "epiphany.cultmesh.operator_run_receipt";
pub const EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.operator_run_receipt.v0";
pub const EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_LATEST_KEY: &str =
    "epiphany-local/operator-run-receipt/latest";
pub const EPIPHANY_CULTMESH_COORDINATOR_RUN_RECEIPT_TYPE: &str =
    "epiphany.cultmesh.coordinator_run_receipt";
pub const EPIPHANY_CULTMESH_COORDINATOR_RUN_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.coordinator_run_receipt.v0";
pub const EPIPHANY_CULTMESH_COORDINATOR_RUN_RECEIPT_LATEST_KEY: &str =
    "epiphany-local/coordinator-run-receipt/latest";
pub const EPIPHANY_CULTMESH_HANDS_ACTION_GATE_TYPE: &str = "epiphany.cultmesh.hands_action_gate";
pub const EPIPHANY_CULTMESH_HANDS_ACTION_GATE_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.hands_action_gate.v0";
pub const EPIPHANY_CULTMESH_HANDS_ACTION_GATE_LATEST_KEY: &str =
    "epiphany-local/hands-action-gate/latest";
pub const EPIPHANY_CULTMESH_ROLE_REVIEW_EVENT_TYPE: &str = "epiphany.cultmesh.role_review_event";
pub const EPIPHANY_CULTMESH_ROLE_REVIEW_EVENT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.role_review_event.v0";
pub const EPIPHANY_CULTMESH_ROLE_REVIEW_EVENT_LATEST_KEY: &str =
    "epiphany-local/role-review-event/latest";
pub const EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_TYPE: &str =
    "epiphany.cultmesh.work_loop_telemetry";
pub const EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.work_loop_telemetry.v0";
pub const EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_LATEST_KEY: &str =
    "epiphany-internal/work-loop-telemetry/latest";
pub const EPIPHANY_CULTMESH_AGENT_STATE_SOA_SUMMARY_TYPE: &str =
    "epiphany.cultmesh.agent_state_soa_summary";
pub const EPIPHANY_CULTMESH_AGENT_STATE_SOA_SUMMARY_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.agent_state_soa_summary.v0";
pub const EPIPHANY_CULTMESH_AGENT_STATE_SOA_SUMMARY_LATEST_KEY: &str =
    "epiphany-local/agent-state-soa-summary/latest";
pub const EPIPHANY_CULTMESH_REPO_WORK_OVERVIEW_TYPE: &str = "epiphany.cultmesh.repo_work_overview";
pub const EPIPHANY_CULTMESH_REPO_WORK_OVERVIEW_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.repo_work_overview.v0";
pub const EPIPHANY_CULTMESH_REPO_WORK_OVERVIEW_LATEST_KEY: &str =
    "gamecult-local/repo-work-overview/latest";
pub const EPIPHANY_CULTMESH_REPO_WORK_READINESS_TYPE: &str =
    "epiphany.cultmesh.repo_work_readiness";
pub const EPIPHANY_CULTMESH_REPO_WORK_READINESS_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.repo_work_readiness.v0";
pub const EPIPHANY_CULTMESH_REPO_WORK_READINESS_LATEST_KEY: &str =
    "gamecult-local/repo-work-readiness/latest";
pub const EPIPHANY_CULTMESH_REPO_WORK_MAP_ENTRY_TYPE: &str =
    "epiphany.cultmesh.repo_work_map_entry";
pub const EPIPHANY_CULTMESH_REPO_WORK_MAP_ENTRY_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.repo_work_map_entry.v0";
pub const EPIPHANY_CULTMESH_REPO_WORK_MAP_ENTRY_LATEST_KEY: &str =
    "gamecult-local/repo-work-map/latest";
pub const EPIPHANY_CULTMESH_REPO_WORK_PUBLIC_PROOF_TYPE: &str =
    "epiphany.cultmesh.repo_work_public_proof";
pub const EPIPHANY_CULTMESH_REPO_WORK_PUBLIC_PROOF_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.repo_work_public_proof.v0";
pub const EPIPHANY_CULTMESH_REPO_WORK_PUBLIC_PROOF_LATEST_KEY: &str =
    "gamecult-local/repo-work-public-proof/latest";
pub const EPIPHANY_CULTMESH_VERSE_POLICY_TYPE: &str = "epiphany.cultmesh.verse_policy";
pub const EPIPHANY_CULTMESH_VERSE_POLICY_SCHEMA_VERSION: &str = "epiphany.cultmesh.verse_policy.v0";
pub const EPIPHANY_CULTMESH_GLOBAL_ROOM_POLICY_TYPE: &str = "epiphany.cultmesh.global_room_policy";
pub const EPIPHANY_CULTMESH_GLOBAL_ROOM_POLICY_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.global_room_policy.v0";
pub const EPIPHANY_CULTMESH_CLUSTER_TOPOLOGY_TYPE: &str = "epiphany.cultmesh.cluster_topology";
pub const EPIPHANY_CULTMESH_CLUSTER_TOPOLOGY_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.cluster_topology.v0";
pub const EPIPHANY_CULTMESH_ODIN_ADVERTISEMENT_TYPE: &str = "epiphany.cultmesh.odin_advertisement";
pub const EPIPHANY_CULTMESH_ODIN_ADVERTISEMENT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.odin_advertisement.v0";
pub const EPIPHANY_CULTMESH_EVE_CONNECTION_INTENT_TYPE: &str =
    "epiphany.cultmesh.eve_connection_intent";
pub const EPIPHANY_CULTMESH_EVE_CONNECTION_INTENT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.eve_connection_intent.v0";
pub const EPIPHANY_CULTMESH_EVE_CONNECTION_INTENT_LATEST_KEY: &str =
    "epiphany-local/eve-connection-intent/latest";
pub const EPIPHANY_CULTMESH_EVE_CONNECTION_RECEIPT_TYPE: &str =
    "epiphany.cultmesh.eve_connection_receipt";
pub const EPIPHANY_CULTMESH_EVE_CONNECTION_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.eve_connection_receipt.v0";
pub const EPIPHANY_CULTMESH_EVE_CONNECTION_RECEIPT_LATEST_KEY: &str =
    "epiphany-local/eve-connection-receipt/latest";
pub const EPIPHANY_CULTMESH_EVE_SURFACE_STATE_TYPE: &str = "gamecult.eve.surface_state";
pub const EPIPHANY_CULTMESH_EVE_SURFACE_STATE_SCHEMA_VERSION: &str =
    "gamecult.eve.surface_state.v0";
pub const EPIPHANY_CULTMESH_DAEMON_STATUS_TYPE: &str = "epiphany.cultmesh.daemon_status";
pub const EPIPHANY_CULTMESH_DAEMON_STATUS_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.daemon_status.v0";
pub const EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_TYPE: &str =
    "epiphany.cultmesh.daemon_heartbeat_event";
pub const EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.daemon_heartbeat_event.v1";
pub const EPIPHANY_CULTMESH_DAEMON_POKE_INTENT_TYPE: &str = "epiphany.cultmesh.daemon_poke_intent";
pub const EPIPHANY_CULTMESH_DAEMON_POKE_INTENT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.daemon_poke_intent.v1";
pub const EPIPHANY_CULTMESH_DAEMON_POKE_INTENT_LATEST_KEY: &str =
    "epiphany-local/daemon-poke-intent/latest";
pub const EPIPHANY_CULTMESH_DAEMON_POKE_RECEIPT_TYPE: &str =
    "epiphany.cultmesh.daemon_poke_receipt";
pub const EPIPHANY_CULTMESH_DAEMON_POKE_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.daemon_poke_receipt.v1";
pub const EPIPHANY_CULTMESH_DAEMON_POKE_RECEIPT_LATEST_KEY: &str =
    "epiphany-local/daemon-poke-receipt/latest";
pub const EPIPHANY_CULTMESH_DAEMON_RESTART_POLICY_TYPE: &str =
    "epiphany.cultmesh.daemon_restart_policy";
pub const EPIPHANY_CULTMESH_DAEMON_RESTART_POLICY_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.daemon_restart_policy.v0";
pub const EPIPHANY_CULTMESH_DAEMON_SCHEDULER_RECEIPT_TYPE: &str =
    "epiphany.cultmesh.daemon_scheduler_receipt";
pub const EPIPHANY_CULTMESH_DAEMON_SCHEDULER_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.daemon_scheduler_receipt.v0";
pub const EPIPHANY_CULTMESH_DAEMON_SCHEDULER_RECEIPT_LATEST_KEY: &str =
    "epiphany-local/daemon-scheduler-receipt/latest";
pub const EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_TYPE: &str =
    "epiphany.cultmesh.daemon_service_lifecycle_receipt";
pub const EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.daemon_service_lifecycle_receipt.v1";
pub const EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_LATEST_KEY: &str =
    "epiphany-local/daemon-service-lifecycle-receipt/latest";
pub const EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_TYPE: &str =
    "epiphany.cultmesh.managed_service_policy";
pub const EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.managed_service_policy.v0";
const EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID: &str = "epiphany-memory-semantic-projector-service";
pub const EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID: &str =
    "epiphany-workspace-coverage-projector-service";
pub const EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_DAEMON_ID: &str =
    "epiphany-workspace-coverage-projector";
pub const EPIPHANY_CULTMESH_IDUNN_DEPLOYMENT_RECEIPT_SCHEMA_VERSION: &str =
    "gamecult.idunn.deployment_receipt.v0";
pub const EPIPHANY_CULTMESH_IDUNN_DEPLOYMENT_RECEIPT_LATEST_KEY: &str =
    "gamecult-local/idunn/deployment-receipt/latest";
pub const EPIPHANY_CULTMESH_IDUNN_AFTERCARE_AUDIT_RECEIPT_SCHEMA_VERSION: &str =
    "gamecult.idunn.deployment_aftercare_audit.v0";
pub const EPIPHANY_CULTMESH_IDUNN_AFTERCARE_AUDIT_RECEIPT_LATEST_KEY: &str =
    "gamecult-local/idunn/deployment-aftercare-audit/latest";
pub const EPIPHANY_CULTMESH_SWARM_BRAKE_TYPE: &str = "epiphany.cultmesh.swarm_brake";
pub const EPIPHANY_CULTMESH_SWARM_BRAKE_SCHEMA_VERSION: &str = "epiphany.cultmesh.swarm_brake.v0";
pub const EPIPHANY_CULTMESH_SWARM_BRAKE_KEY: &str = "epiphany-local/swarm-brake";
pub const EPIPHANY_CULTMESH_PERSONA_SPEECH_AUDIT_TYPE: &str =
    "epiphany.cultmesh.persona_speech_audit";
pub const EPIPHANY_CULTMESH_PERSONA_SPEECH_AUDIT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.persona_speech_audit.v0";
pub const EPIPHANY_CULTMESH_PERSONA_SPEECH_AUDIT_LATEST_KEY: &str =
    "epiphany-local/persona-speech-audit/latest";
pub const EPIPHANY_CULTMESH_WEKSA_LOWERING_RECEIPT_TYPE: &str =
    "epiphany.cultmesh.weksa_lowering_receipt";
pub const EPIPHANY_CULTMESH_WEKSA_LOWERING_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.weksa_lowering_receipt.v0";
pub const EPIPHANY_CULTMESH_WEKSA_LOWERING_RECEIPT_LATEST_KEY: &str =
    "epiphany-local/weksa-lowering-receipt/latest";
pub const EPIPHANY_CULTMESH_DAEMON_TOOL_CAPABILITY_TYPE: &str =
    "epiphany.cultmesh.daemon_tool_capability";
pub const EPIPHANY_CULTMESH_DAEMON_TOOL_CAPABILITY_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.daemon_tool_capability.v0";
pub const EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_INTENT_TYPE: &str =
    "epiphany.cultmesh.daemon_tool_invocation_intent";
pub const EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_INTENT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.daemon_tool_invocation_intent.v0";
pub const EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_INTENT_LATEST_KEY: &str =
    "epiphany-local/daemon-tool-invocation-intent/latest";
pub const EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_RECEIPT_TYPE: &str =
    "epiphany.cultmesh.daemon_tool_invocation_receipt";
pub const EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.daemon_tool_invocation_receipt.v0";
pub const EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_RECEIPT_LATEST_KEY: &str =
    "epiphany-local/daemon-tool-invocation-receipt/latest";
pub const EPIPHANY_CULTMESH_MIND_CONTRACT_TYPE: &str = "epiphany.cultmesh.mind_contract";
pub const EPIPHANY_CULTMESH_MIND_CONTRACT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.mind_contract.v0";
pub const EPIPHANY_CULTMESH_SUBSTRATE_GATE_CONTRACT_TYPE: &str =
    "epiphany.cultmesh.substrate_gate_contract";
pub const EPIPHANY_CULTMESH_SUBSTRATE_GATE_CONTRACT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.substrate_gate_contract.v0";
pub const EPIPHANY_CULTMESH_EYES_CONTRACT_TYPE: &str = "epiphany.cultmesh.eyes_contract";
pub const EPIPHANY_CULTMESH_EYES_CONTRACT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.eyes_contract.v0";
pub const EPIPHANY_CULTMESH_HANDS_CONTRACT_TYPE: &str = "epiphany.cultmesh.hands_contract";
pub const EPIPHANY_CULTMESH_HANDS_CONTRACT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.hands_contract.v0";
pub const EPIPHANY_CULTMESH_SOUL_CONTRACT_TYPE: &str = "epiphany.cultmesh.soul_contract";
pub const EPIPHANY_CULTMESH_SOUL_CONTRACT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.soul_contract.v0";
pub const EPIPHANY_CULTMESH_CONTINUITY_CONTRACT_TYPE: &str =
    "epiphany.cultmesh.continuity_contract";
pub const EPIPHANY_CULTMESH_CONTINUITY_CONTRACT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.continuity_contract.v0";
pub const EPIPHANY_CULTMESH_BIFROST_CONTRACT_TYPE: &str = "epiphany.cultmesh.bifrost_contract";
pub const EPIPHANY_CULTMESH_BIFROST_CONTRACT_SCHEMA_VERSION: &str =
    "epiphany.cultmesh.bifrost_contract.v0";
pub const EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_INTENT_TYPE: &str =
    "gamecult.bifrost.body_change_publication_intent";
pub const EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_INTENT_SCHEMA_VERSION: &str =
    "gamecult.bifrost.body_change_publication_intent.v0";
pub const EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_INTENT_ARRIVAL_LATEST_KEY: &str =
    "gamecult-local/bifrost/body-change-publication-intent/latest";
pub const EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_RECEIPT_TYPE: &str =
    "gamecult.bifrost.body_change_publication_receipt";
pub const EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_RECEIPT_SCHEMA_VERSION: &str =
    "gamecult.bifrost.body_change_publication_receipt.v0";
pub const EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_RECEIPT_ARRIVAL_LATEST_KEY: &str =
    "gamecult-local/bifrost/body-change-publication-receipt/latest";
pub const EPIPHANY_CULTMESH_BIFROST_GITHUB_PUBLICATION_RECEIPT_TYPE: &str =
    "gamecult.bifrost.github_publication_receipt";
pub const EPIPHANY_CULTMESH_BIFROST_GITHUB_PUBLICATION_RECEIPT_SCHEMA_VERSION: &str =
    "gamecult.bifrost.github_publication_receipt.v0";
pub const EPIPHANY_CULTMESH_BIFROST_GITHUB_PUBLICATION_RECEIPT_ARRIVAL_LATEST_KEY: &str =
    "gamecult-local/bifrost/github-publication-receipt/latest";
pub const EPIPHANY_CULTMESH_BIFROST_PUBLIC_PROOF_PUBLICATION_RECEIPT_TYPE: &str =
    "gamecult.bifrost.public_proof_publication_receipt";
pub const EPIPHANY_CULTMESH_BIFROST_PUBLIC_PROOF_PUBLICATION_RECEIPT_SCHEMA_VERSION: &str =
    "gamecult.bifrost.public_proof_publication_receipt.v0";
pub const EPIPHANY_CULTMESH_BIFROST_PUBLIC_PROOF_PUBLICATION_RECEIPT_ARRIVAL_LATEST_KEY: &str =
    "gamecult-local/bifrost/public-proof-publication-receipt/latest";
pub const EPIPHANY_CULTMESH_BIFROST_ARTIFACT_ACCEPTANCE_RECEIPT_TYPE: &str =
    "gamecult.bifrost.artifact_acceptance_receipt";
pub const EPIPHANY_CULTMESH_BIFROST_ARTIFACT_ACCEPTANCE_RECEIPT_SCHEMA_VERSION: &str =
    "gamecult.bifrost.artifact_acceptance_receipt.v0";
pub const EPIPHANY_CULTMESH_BIFROST_ARTIFACT_ACCEPTANCE_RECEIPT_ARRIVAL_LATEST_KEY: &str =
    "gamecult-local/bifrost/artifact-acceptance-receipt/latest";
pub const EPIPHANY_CULTMESH_BIFROST_METRICS_RECEIPT_TYPE: &str = "gamecult.bifrost.metrics_receipt";
pub const EPIPHANY_CULTMESH_BIFROST_METRICS_RECEIPT_SCHEMA_VERSION: &str =
    "gamecult.bifrost.metrics_receipt.v0";
pub const EPIPHANY_CULTMESH_BIFROST_METRICS_RECEIPT_ARRIVAL_LATEST_KEY: &str =
    "gamecult-local/bifrost/metrics-receipt/latest";
pub const EPIPHANY_CULTMESH_BIFROST_COLLABORATION_FEEDBACK_TYPE: &str =
    "gamecult.bifrost.collaboration_feedback";
pub const EPIPHANY_CULTMESH_BIFROST_COLLABORATION_FEEDBACK_SCHEMA_VERSION: &str =
    "gamecult.bifrost.collaboration_feedback.v0";
pub const EPIPHANY_CULTMESH_BIFROST_COLLABORATION_FEEDBACK_ARRIVAL_LATEST_KEY: &str =
    "gamecult-local/bifrost/collaboration-feedback/latest";
pub const EPIPHANY_CULTMESH_IMAGINATION_CONSENSUS_RECEIPT_TYPE: &str =
    "gamecult.imagination.consensus_discovery_receipt";
pub const EPIPHANY_CULTMESH_IMAGINATION_CONSENSUS_RECEIPT_SCHEMA_VERSION: &str =
    "gamecult.imagination.consensus_discovery_receipt.v0";
pub const EPIPHANY_CULTMESH_IMAGINATION_CONSENSUS_RECEIPT_LATEST_KEY: &str =
    "gamecult-local/imagination/consensus-discovery-receipt/latest";
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

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.status",
    schema = "EpiphanyCultMeshStatusEntry"
)]
pub struct EpiphanyCultMeshStatusEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub app_id: String,
    #[cultcache(key = 4)]
    pub note: String,
    #[cultcache(key = 5, default)]
    pub verse_tier: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.operator_snapshot",
    schema = "EpiphanyCultMeshOperatorSnapshotEntry"
)]
pub struct EpiphanyCultMeshOperatorSnapshotEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub snapshot_id: String,
    #[cultcache(key = 4)]
    pub generated_at_utc: String,
    #[cultcache(key = 5)]
    pub source_mode: String,
    #[cultcache(key = 6)]
    pub source_path: String,
    #[cultcache(key = 7)]
    pub thread_id: String,
    #[cultcache(key = 8)]
    pub status: String,
    #[cultcache(key = 9)]
    pub state_status: String,
    #[cultcache(key = 10)]
    pub coordinator_action: String,
    #[cultcache(key = 11)]
    pub crrc_action: String,
    #[cultcache(key = 12)]
    pub pressure_level: String,
    #[cultcache(key = 13)]
    pub reorient_action: String,
    #[cultcache(key = 14)]
    pub next_action: String,
    #[cultcache(key = 15)]
    pub artifact_refs: Vec<String>,
    #[cultcache(key = 16)]
    pub available_actions: Vec<String>,
    #[cultcache(key = 17)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.operator_run_intent",
    schema = "EpiphanyCultMeshOperatorRunIntentEntry"
)]
pub struct EpiphanyCultMeshOperatorRunIntentEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub run_id: String,
    #[cultcache(key = 4)]
    pub requested_at_utc: String,
    #[cultcache(key = 5)]
    pub mode: String,
    #[cultcache(key = 6)]
    pub root: String,
    #[cultcache(key = 7)]
    pub workspace: String,
    #[cultcache(key = 8)]
    pub thread_id: String,
    #[cultcache(key = 9)]
    pub codex_home: String,
    #[cultcache(key = 10)]
    pub target_dir: String,
    #[cultcache(key = 11)]
    pub max_steps: u32,
    #[cultcache(key = 12)]
    pub timeout_seconds: u32,
    #[cultcache(key = 13)]
    pub auto_review: bool,
    #[cultcache(key = 14)]
    pub no_ephemeral: bool,
    #[cultcache(key = 15)]
    pub artifact_root: String,
    #[cultcache(key = 16)]
    pub dogfood_root: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.operator_run_receipt",
    schema = "EpiphanyCultMeshOperatorRunReceiptEntry"
)]
pub struct EpiphanyCultMeshOperatorRunReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub run_id: String,
    #[cultcache(key = 4)]
    pub completed_at_utc: String,
    #[cultcache(key = 5)]
    pub mode: String,
    #[cultcache(key = 6)]
    pub status: String,
    #[cultcache(key = 7)]
    pub result_path: String,
    #[cultcache(key = 8)]
    pub artifact_root: String,
    #[cultcache(key = 9)]
    pub dogfood_root: String,
    #[cultcache(key = 10)]
    pub operator_snapshot_store: String,
    #[cultcache(key = 11)]
    pub operator_snapshot_id: String,
    #[cultcache(key = 12)]
    pub artifact_refs: Vec<String>,
    #[cultcache(key = 13)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.coordinator_run_receipt",
    schema = "EpiphanyCultMeshCoordinatorRunReceiptEntry"
)]
pub struct EpiphanyCultMeshCoordinatorRunReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub receipt_id: String,
    #[cultcache(key = 4)]
    pub source_document_type: String,
    #[cultcache(key = 5)]
    pub source_receipt_id: String,
    #[cultcache(key = 6)]
    pub source_store: String,
    #[cultcache(key = 7)]
    pub thread_id: String,
    #[cultcache(key = 8)]
    pub mode: String,
    #[cultcache(key = 9)]
    pub status: String,
    #[cultcache(key = 10)]
    pub final_action: String,
    #[cultcache(key = 11)]
    pub final_reason: String,
    #[cultcache(key = 12)]
    pub step_count: u64,
    #[cultcache(key = 13)]
    pub artifact_root: String,
    #[cultcache(key = 14)]
    pub artifact_refs: Vec<String>,
    #[cultcache(key = 15)]
    pub sealed_artifact_refs: Vec<String>,
    #[cultcache(key = 16)]
    pub created_at_utc: String,
    #[cultcache(key = 17)]
    pub private_state_exposed: bool,
    #[cultcache(key = 18)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.hands_action_gate",
    schema = "EpiphanyCultMeshHandsActionGateEntry"
)]
pub struct EpiphanyCultMeshHandsActionGateEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub gate_id: String,
    #[cultcache(key = 4)]
    pub source_coordinator_receipt_id: String,
    #[cultcache(key = 5)]
    pub source_summary_path: String,
    #[cultcache(key = 6)]
    pub thread_id: String,
    #[cultcache(key = 7)]
    pub mode: String,
    #[cultcache(key = 8)]
    pub status: String,
    #[cultcache(key = 9)]
    pub hands_intent_id: String,
    #[cultcache(key = 10)]
    pub hands_review_id: String,
    #[cultcache(key = 11)]
    pub substrate_gate_grant_receipt_id: String,
    #[cultcache(key = 12)]
    pub runtime_job_id: String,
    #[cultcache(key = 13)]
    pub requested_paths: Vec<String>,
    #[cultcache(key = 14)]
    pub required_receipts: Vec<String>,
    #[cultcache(key = 15)]
    pub record_pass_executable: String,
    #[cultcache(key = 16)]
    pub record_pass_args: Vec<String>,
    #[cultcache(key = 17)]
    pub created_at_utc: String,
    #[cultcache(key = 18)]
    pub private_state_exposed: bool,
    #[cultcache(key = 19)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.role_review_event",
    schema = "EpiphanyCultMeshRoleReviewEventEntry"
)]
pub struct EpiphanyCultMeshRoleReviewEventEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub event_id: String,
    #[cultcache(key = 4)]
    pub source_coordinator_receipt_id: String,
    #[cultcache(key = 5)]
    pub source_summary_path: String,
    #[cultcache(key = 6)]
    pub thread_id: String,
    #[cultcache(key = 7)]
    pub mode: String,
    #[cultcache(key = 8)]
    pub surface: String,
    #[cultcache(key = 9)]
    pub role_id: String,
    #[cultcache(key = 10)]
    pub review_status: String,
    #[cultcache(key = 11)]
    pub acceptance_receipt_id: String,
    #[cultcache(key = 12)]
    pub runtime_result_id: String,
    #[cultcache(key = 13)]
    pub runtime_job_id: String,
    #[cultcache(key = 14)]
    pub binding_id: String,
    #[cultcache(key = 15)]
    pub summary: String,
    #[cultcache(key = 16)]
    pub created_at_utc: String,
    #[cultcache(key = 17)]
    pub private_state_exposed: bool,
    #[cultcache(key = 18)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.work_loop_telemetry",
    schema = "EpiphanyCultMeshWorkLoopTelemetryEntry"
)]
pub struct EpiphanyCultMeshWorkLoopTelemetryEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub telemetry_id: String,
    #[cultcache(key = 4)]
    pub thread_id: String,
    #[cultcache(key = 5)]
    pub produced_at_utc: String,
    #[cultcache(key = 6)]
    pub source_stage: String,
    #[cultcache(key = 7)]
    pub target_stages: Vec<String>,
    #[cultcache(key = 8)]
    pub lower_bound_receipt_at: String,
    #[cultcache(key = 9)]
    pub hands_intent_id: String,
    #[cultcache(key = 10)]
    pub hands_review_id: String,
    #[cultcache(key = 11)]
    pub hands_runtime_job_id: String,
    #[cultcache(key = 12)]
    pub substrate_gate_grant_receipt_id: String,
    #[cultcache(key = 13)]
    pub hands_patch_receipt_id: String,
    #[cultcache(key = 14)]
    pub hands_command_receipt_id: String,
    #[cultcache(key = 15)]
    pub hands_commit_receipt_id: String,
    #[cultcache(key = 16)]
    pub command: String,
    #[cultcache(key = 17)]
    pub exit_code: String,
    #[cultcache(key = 18)]
    pub stdout_artifact: String,
    #[cultcache(key = 19)]
    pub stderr_artifact: String,
    #[cultcache(key = 20)]
    pub commit_sha: String,
    #[cultcache(key = 21)]
    pub branch: String,
    #[cultcache(key = 22)]
    pub changed_paths: Vec<String>,
    #[cultcache(key = 23)]
    pub artifact_previews: Vec<String>,
    #[cultcache(key = 24)]
    pub source_refs: Vec<String>,
    #[cultcache(key = 25)]
    pub source_path_proof: Vec<String>,
    #[cultcache(key = 26)]
    pub soul_receipt_ids: Vec<String>,
    #[cultcache(key = 27)]
    pub summary: String,
    #[cultcache(key = 28, default)]
    pub receipt_payload_previews: Vec<String>,
    #[cultcache(key = 29, default)]
    pub commit_diff_preview: String,
    #[cultcache(key = 30, default)]
    pub verification_assertions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.agent_state_soa_summary",
    schema = "EpiphanyCultMeshAgentStateSoaSummaryEntry"
)]
pub struct EpiphanyCultMeshAgentStateSoaSummaryEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub summary_id: String,
    #[cultcache(key = 4)]
    pub generated_at: String,
    #[cultcache(key = 5)]
    pub source_store: String,
    #[cultcache(key = 6)]
    pub row_count: u32,
    #[cultcache(key = 7)]
    pub role_ids: Vec<String>,
    #[cultcache(key = 8)]
    pub agent_ids: Vec<String>,
    #[cultcache(key = 9)]
    pub display_names: Vec<String>,
    #[cultcache(key = 10)]
    pub profile_kinds: Vec<String>,
    #[cultcache(key = 11)]
    pub portable_contracts: Vec<String>,
    #[cultcache(key = 12)]
    pub semantic_memory_counts: Vec<u32>,
    #[cultcache(key = 13)]
    pub episodic_memory_counts: Vec<u32>,
    #[cultcache(key = 14)]
    pub relationship_memory_counts: Vec<u32>,
    #[cultcache(key = 15)]
    pub goal_counts: Vec<u32>,
    #[cultcache(key = 16)]
    pub value_counts: Vec<u32>,
    #[cultcache(key = 17)]
    pub private_state_exposed: bool,
    #[cultcache(key = 18)]
    pub notes: Vec<String>,
}

impl SoaDocument for EpiphanyCultMeshAgentStateSoaSummaryEntry {
    fn soa_columns(rows: &[Self]) -> std::collections::BTreeMap<&'static str, CultSoaColumnValues> {
        let mut columns = std::collections::BTreeMap::new();
        columns.insert(
            "summaryId",
            CultSoaColumnValues::new(
                rows.iter()
                    .map(|row| row.summary_id.clone())
                    .collect::<Vec<_>>(),
            ),
        );
        columns.insert(
            "rowCount",
            CultSoaColumnValues::new(rows.iter().map(|row| row.row_count).collect::<Vec<_>>()),
        );
        columns.insert(
            "privateStateExposed",
            CultSoaColumnValues::new(
                rows.iter()
                    .map(|row| row.private_state_exposed)
                    .collect::<Vec<_>>(),
            ),
        );
        columns.insert(
            "roleIds",
            CultSoaColumnValues::new(
                rows.iter()
                    .map(|row| row.role_ids.clone())
                    .collect::<Vec<_>>(),
            ),
        );
        columns.insert(
            "portableContracts",
            CultSoaColumnValues::new(
                rows.iter()
                    .map(|row| row.portable_contracts.clone())
                    .collect::<Vec<_>>(),
            ),
        );
        columns
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.repo_work_overview",
    schema = "EpiphanyCultMeshRepoWorkOverviewEntry"
)]
pub struct EpiphanyCultMeshRepoWorkOverviewEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub overview_id: String,
    #[cultcache(key = 4)]
    pub generated_at: String,
    #[cultcache(key = 5)]
    pub workspace: String,
    #[cultcache(key = 6)]
    pub item: String,
    #[cultcache(key = 7)]
    pub branch: String,
    #[cultcache(key = 8)]
    pub current_gate: String,
    #[cultcache(key = 9)]
    pub blocker: String,
    #[cultcache(key = 10)]
    pub next_safe_move: String,
    #[cultcache(key = 11)]
    pub changed_paths: Vec<String>,
    #[cultcache(key = 12)]
    pub commit_sha: String,
    #[cultcache(key = 13)]
    pub soul_verdict: String,
    #[cultcache(key = 14)]
    pub publication_status: String,
    #[cultcache(key = 15)]
    pub sync_status: String,
    #[cultcache(key = 16)]
    pub receipt_refs: Vec<String>,
    #[cultcache(key = 17)]
    pub tui_rows: Vec<String>,
    #[cultcache(key = 18)]
    pub proof_bundle_ref: String,
    #[cultcache(key = 19)]
    pub private_state_exposed: bool,
    #[cultcache(key = 20)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.repo_work_readiness",
    schema = "EpiphanyCultMeshRepoWorkReadinessEntry"
)]
pub struct EpiphanyCultMeshRepoWorkReadinessEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub readiness_id: String,
    #[cultcache(key = 4)]
    pub generated_at: String,
    #[cultcache(key = 5)]
    pub workspace: String,
    #[cultcache(key = 6)]
    pub item: String,
    #[cultcache(key = 7)]
    pub status: String,
    #[cultcache(key = 8)]
    pub missing_required_count: u32,
    #[cultcache(key = 9)]
    pub satisfied_required_count: u32,
    #[cultcache(key = 10)]
    pub readiness_receipt_ref: String,
    #[cultcache(key = 11)]
    pub overview_receipt_ref: String,
    #[cultcache(key = 12)]
    pub proof_bundle_id: String,
    #[cultcache(key = 13)]
    pub missing_kinds: Vec<String>,
    #[cultcache(key = 14)]
    pub tui_rows: Vec<String>,
    #[cultcache(key = 15)]
    pub sight_only: bool,
    #[cultcache(key = 16)]
    pub readiness_approval_authorized: bool,
    #[cultcache(key = 17)]
    pub publication_authorized: bool,
    #[cultcache(key = 18)]
    pub service_lifecycle_authority: bool,
    #[cultcache(key = 19)]
    pub hands_action_authorized: bool,
    #[cultcache(key = 20)]
    pub private_state_exposed: bool,
    #[cultcache(key = 21)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.repo_work_map_entry",
    schema = "EpiphanyCultMeshRepoWorkMapEntry"
)]
pub struct EpiphanyCultMeshRepoWorkMapEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub map_entry_id: String,
    #[cultcache(key = 4)]
    pub admitted_at: String,
    #[cultcache(key = 5)]
    pub mirrored_at: String,
    #[cultcache(key = 6)]
    pub workspace: String,
    #[cultcache(key = 7)]
    pub item: String,
    #[cultcache(key = 8)]
    pub branch: String,
    #[cultcache(key = 9)]
    pub changed_paths: Vec<String>,
    #[cultcache(key = 10)]
    pub commit_sha: String,
    #[cultcache(key = 11)]
    pub safe_action_family: String,
    #[cultcache(key = 12)]
    pub modeling_summary: String,
    #[cultcache(key = 13)]
    pub soul_verdict_receipt_id: String,
    #[cultcache(key = 14)]
    pub mind_gateway_review_id: String,
    #[cultcache(key = 15)]
    pub mind_state_commit_receipt_id: String,
    #[cultcache(key = 16)]
    pub publication_gate: String,
    #[cultcache(key = 17)]
    pub durable_state_admitted: bool,
    #[cultcache(key = 18)]
    pub source_store_path: String,
    #[cultcache(key = 19)]
    pub tui_rows: Vec<String>,
    #[cultcache(key = 20)]
    pub private_state_exposed: bool,
    #[cultcache(key = 21)]
    pub notes: Vec<String>,
    #[cultcache(key = 22)]
    pub modeling_finding_receipt_id: String,
    #[cultcache(key = 23)]
    pub modeling_route_id: String,
    #[cultcache(key = 24)]
    pub modeling_generation: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.repo_work_public_proof",
    schema = "EpiphanyCultMeshRepoWorkPublicProofEntry"
)]
pub struct EpiphanyCultMeshRepoWorkPublicProofEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub public_proof_id: String,
    #[cultcache(key = 4)]
    pub generated_at: String,
    #[cultcache(key = 5)]
    pub workspace: String,
    #[cultcache(key = 6)]
    pub item: String,
    #[cultcache(key = 7)]
    pub branch: String,
    #[cultcache(key = 8)]
    pub current_gate: String,
    #[cultcache(key = 9)]
    pub blocker: String,
    #[cultcache(key = 10)]
    pub next_safe_move: String,
    #[cultcache(key = 11)]
    pub changed_paths: Vec<String>,
    #[cultcache(key = 12)]
    pub commit_sha: String,
    #[cultcache(key = 13)]
    pub soul_verdict: String,
    #[cultcache(key = 14)]
    pub upstream_main_synced: bool,
    #[cultcache(key = 15)]
    pub artifact_row_count: u32,
    #[cultcache(key = 16)]
    pub publication_row_count: u32,
    #[cultcache(key = 17)]
    pub public_proof_ref: String,
    #[cultcache(key = 18)]
    pub public_proof_sha256: String,
    #[cultcache(key = 19)]
    pub tui_rows: Vec<String>,
    #[cultcache(key = 20)]
    pub private_state_exposed: bool,
    #[cultcache(key = 21)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.verse_policy",
    schema = "EpiphanyCultMeshVersePolicyEntry"
)]
pub struct EpiphanyCultMeshVersePolicyEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub verse_id: String,
    #[cultcache(key = 2)]
    pub tier: String,
    #[cultcache(key = 3)]
    pub purpose: String,
    #[cultcache(key = 4)]
    pub transport_scope: String,
    #[cultcache(key = 5)]
    pub trust_boundary: String,
    #[cultcache(key = 6)]
    pub private_state_allowed: bool,
    #[cultcache(key = 7)]
    pub untrusted_ingress_allowed: bool,
    #[cultcache(key = 8)]
    pub yggdrasil_tunnel_allowed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.global_room_policy",
    schema = "EpiphanyCultMeshGlobalRoomPolicyEntry"
)]
pub struct EpiphanyCultMeshGlobalRoomPolicyEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub room_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub topic: String,
    #[cultcache(key = 4)]
    pub purpose: String,
    #[cultcache(key = 5)]
    pub posting_policy: String,
    #[cultcache(key = 6)]
    pub threaded: bool,
    #[cultcache(key = 7)]
    pub persona_posting_allowed: bool,
    #[cultcache(key = 8)]
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
    type = "epiphany.cultmesh.odin_advertisement",
    schema = "EpiphanyCultMeshOdinAdvertisementEntry"
)]
pub struct EpiphanyCultMeshOdinAdvertisementEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub advertisement_id: String,
    #[cultcache(key = 2)]
    pub cluster_id: String,
    #[cultcache(key = 3)]
    pub advertised_verse_id: String,
    #[cultcache(key = 4)]
    pub body_domain: String,
    #[cultcache(key = 5)]
    pub body_kind: String,
    #[cultcache(key = 6)]
    pub daemon_surface_id: String,
    #[cultcache(key = 7)]
    pub eve_surface_id: String,
    #[cultcache(key = 8)]
    pub public_summary: String,
    #[cultcache(key = 9)]
    pub advertised_document_types: Vec<String>,
    #[cultcache(key = 10)]
    pub trust_boundary: String,
    #[cultcache(key = 11)]
    pub private_state_exposed: bool,
    #[cultcache(key = 12)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.eve_connection_intent",
    schema = "EpiphanyCultMeshEveConnectionIntentEntry"
)]
pub struct EpiphanyCultMeshEveConnectionIntentEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub intent_id: String,
    #[cultcache(key = 2)]
    pub source_cluster_id: String,
    #[cultcache(key = 3)]
    pub target_advertisement_id: String,
    #[cultcache(key = 4)]
    pub target_cluster_id: String,
    #[cultcache(key = 5)]
    pub target_eve_surface_id: String,
    #[cultcache(key = 6)]
    pub collaboration_topic: String,
    #[cultcache(key = 7)]
    pub requested_action: String,
    #[cultcache(key = 8)]
    pub feedback_route: String,
    #[cultcache(key = 9)]
    pub requested_document_types: Vec<String>,
    #[cultcache(key = 10)]
    pub private_state_requested: bool,
    #[cultcache(key = 11)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.eve_connection_receipt",
    schema = "EpiphanyCultMeshEveConnectionReceiptEntry"
)]
pub struct EpiphanyCultMeshEveConnectionReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub intent_id: String,
    #[cultcache(key = 3)]
    pub source_cluster_id: String,
    #[cultcache(key = 4)]
    pub target_cluster_id: String,
    #[cultcache(key = 5)]
    pub target_eve_surface_id: String,
    #[cultcache(key = 6)]
    pub status: String,
    #[cultcache(key = 7)]
    pub feedback_route: String,
    #[cultcache(key = 8)]
    pub private_state_exposed: bool,
    #[cultcache(key = 9)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.eve.surface_state",
    schema = "EpiphanyCultMeshEveSurfaceStateEntry"
)]
pub struct EpiphanyCultMeshEveSurfaceStateEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub surface_id: String,
    #[cultcache(key = 2)]
    pub cluster_id: String,
    #[cultcache(key = 3)]
    pub daemon_id: String,
    #[cultcache(key = 4)]
    pub body_domain: String,
    #[cultcache(key = 5)]
    pub tui_title: String,
    #[cultcache(key = 6)]
    pub tui_rows: Vec<String>,
    #[cultcache(key = 7)]
    pub exposed_document_types: Vec<String>,
    #[cultcache(key = 8)]
    pub supported_actions: Vec<String>,
    #[cultcache(key = 9)]
    pub private_state_exposed: bool,
    #[cultcache(key = 10)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.daemon_status",
    schema = "EpiphanyCultMeshDaemonStatusEntry"
)]
pub struct EpiphanyCultMeshDaemonStatusEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub daemon_id: String,
    #[cultcache(key = 2)]
    pub cluster_id: String,
    #[cultcache(key = 3)]
    pub body_domain: String,
    #[cultcache(key = 4)]
    pub daemon_surface_id: String,
    #[cultcache(key = 5)]
    pub eve_surface_id: String,
    #[cultcache(key = 6)]
    pub status: String,
    #[cultcache(key = 7)]
    pub last_heartbeat_utc: String,
    #[cultcache(key = 8)]
    pub supported_actions: Vec<String>,
    #[cultcache(key = 9)]
    pub operator_action: String,
    #[cultcache(key = 10)]
    pub private_state_exposed: bool,
    #[cultcache(key = 11)]
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
    type = "epiphany.cultmesh.daemon_poke_intent",
    schema = "EpiphanyCultMeshDaemonPokeIntentEntry"
)]
pub struct EpiphanyCultMeshDaemonPokeIntentEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub intent_id: String,
    #[cultcache(key = 2)]
    pub requesting_agent_id: String,
    #[cultcache(key = 3)]
    pub target_daemon_id: String,
    #[cultcache(key = 4)]
    pub target_cluster_id: String,
    #[cultcache(key = 5)]
    pub daemon_surface_id: String,
    #[cultcache(key = 6)]
    pub eve_surface_id: String,
    #[cultcache(key = 7)]
    pub reason: String,
    #[cultcache(key = 8)]
    pub requested_action: String,
    #[cultcache(key = 9)]
    pub observed_status: String,
    #[cultcache(key = 10)]
    pub private_state_requested: bool,
    #[cultcache(key = 11)]
    pub notes: Vec<String>,
    #[cultcache(key = 12, default)]
    pub observed_last_heartbeat_utc: String,
    #[cultcache(key = 13, default)]
    pub requested_at_utc: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.daemon_poke_receipt",
    schema = "EpiphanyCultMeshDaemonPokeReceiptEntry"
)]
pub struct EpiphanyCultMeshDaemonPokeReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub intent_id: String,
    #[cultcache(key = 3)]
    pub target_daemon_id: String,
    #[cultcache(key = 4)]
    pub target_cluster_id: String,
    #[cultcache(key = 5)]
    pub action_taken: String,
    #[cultcache(key = 6)]
    pub status: String,
    #[cultcache(key = 7)]
    pub resulting_status: String,
    #[cultcache(key = 8)]
    pub operator_artifact_ref: String,
    #[cultcache(key = 9)]
    pub private_state_exposed: bool,
    #[cultcache(key = 10)]
    pub notes: Vec<String>,
    #[cultcache(key = 11, default)]
    pub attempted_at_utc: String,
    #[cultcache(key = 12, default)]
    pub completed_at_utc: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.daemon_restart_policy",
    schema = "EpiphanyCultMeshDaemonRestartPolicyEntry"
)]
pub struct EpiphanyCultMeshDaemonRestartPolicyEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub policy_id: String,
    #[cultcache(key = 2)]
    pub daemon_id: String,
    #[cultcache(key = 3)]
    pub cluster_id: String,
    #[cultcache(key = 4)]
    pub restart_command: String,
    #[cultcache(key = 5)]
    pub restart_args: Vec<String>,
    #[cultcache(key = 6)]
    pub cwd: Option<String>,
    #[cultcache(key = 7)]
    pub cooldown_seconds: i64,
    #[cultcache(key = 8)]
    pub backoff_multiplier: u32,
    #[cultcache(key = 9)]
    pub failure_count: u32,
    #[cultcache(key = 10)]
    pub last_attempt_utc: Option<String>,
    #[cultcache(key = 11)]
    pub last_result_status: String,
    #[cultcache(key = 12)]
    pub enabled: bool,
    #[cultcache(key = 13)]
    pub private_state_exposed: bool,
    #[cultcache(key = 14)]
    pub notes: Vec<String>,
    #[cultcache(key = 15)]
    pub reconcile_interval_seconds: i64,
    #[cultcache(key = 16)]
    pub heartbeat_stale_seconds: i64,
    #[cultcache(key = 17)]
    pub last_reconcile_utc: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.daemon_scheduler_receipt",
    schema = "EpiphanyCultMeshDaemonSchedulerReceiptEntry"
)]
pub struct EpiphanyCultMeshDaemonSchedulerReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub scheduler_id: String,
    #[cultcache(key = 3)]
    pub runtime_id: String,
    #[cultcache(key = 4)]
    pub daemon_selector: String,
    #[cultcache(key = 5)]
    pub iteration: u64,
    #[cultcache(key = 6)]
    pub status: String,
    #[cultcache(key = 7)]
    pub tick_started_utc: String,
    #[cultcache(key = 8)]
    pub tick_completed_utc: String,
    #[cultcache(key = 9)]
    pub next_wake_utc: Option<String>,
    #[cultcache(key = 10)]
    pub outcome_count: u32,
    #[cultcache(key = 11)]
    pub restarted_count: u32,
    #[cultcache(key = 12)]
    pub refused_count: u32,
    #[cultcache(key = 13)]
    pub skipped_count: u32,
    #[cultcache(key = 14)]
    pub private_state_exposed: bool,
    #[cultcache(key = 15)]
    pub notes: Vec<String>,
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
    #[cultcache(key = 3)]
    pub owner_daemon_id: String,
    #[cultcache(key = 4)]
    pub command: String,
    #[cultcache(key = 5)]
    pub args: Vec<String>,
    #[cultcache(key = 6)]
    pub cwd: Option<String>,
    #[cultcache(key = 7)]
    pub enabled: bool,
    #[cultcache(key = 8)]
    pub restart_mode: String,
    #[cultcache(key = 9)]
    pub cooldown_seconds: i64,
    #[cultcache(key = 10)]
    pub backoff_multiplier: u32,
    #[cultcache(key = 11)]
    pub stdout_artifact: String,
    #[cultcache(key = 12)]
    pub stderr_artifact: String,
    #[cultcache(key = 14)]
    pub updated_at_utc: String,
    #[cultcache(key = 15)]
    pub private_state_exposed: bool,
    #[cultcache(key = 16)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.idunn.deployment_receipt",
    schema = "EpiphanyCultMeshIdunnDeploymentReceiptEntry"
)]
pub struct EpiphanyCultMeshIdunnDeploymentReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub runtime_id: String,
    #[cultcache(key = 3)]
    pub verse_id: String,
    #[cultcache(key = 4)]
    pub status: String,
    #[cultcache(key = 5)]
    pub trigger: String,
    #[cultcache(key = 6)]
    pub watched_ref: String,
    #[cultcache(key = 7)]
    pub source_commit: String,
    #[cultcache(key = 8)]
    pub result_ref: String,
    #[cultcache(key = 9)]
    pub result_summary: String,
    #[cultcache(key = 10)]
    pub private_state_exposed: bool,
    #[cultcache(key = 11)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.idunn.deployment_aftercare_audit",
    schema = "EpiphanyCultMeshIdunnAftercareAuditReceiptEntry"
)]
pub struct EpiphanyCultMeshIdunnAftercareAuditReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub runtime_id: String,
    #[cultcache(key = 3)]
    pub verse_id: String,
    #[cultcache(key = 4)]
    pub status: String,
    #[cultcache(key = 5)]
    pub checked_ref: String,
    #[cultcache(key = 6)]
    pub deployment_receipt_id: String,
    #[cultcache(key = 7)]
    pub audit_ref: String,
    #[cultcache(key = 8)]
    pub result_summary: String,
    #[cultcache(key = 9)]
    pub private_state_exposed: bool,
    #[cultcache(key = 10)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyServiceExecutionAuditCheck {
    pub service_id: Option<String>,
    pub action: String,
    pub allowed_statuses: Vec<String>,
    pub receipt_id: Option<String>,
    pub observed_status: Option<String>,
    pub operator_artifact_ref: Option<String>,
    pub ok: bool,
    pub private_state_sealed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyServiceExecutionAuditReport {
    pub status: String,
    pub receipt_count: usize,
    pub missing_count: usize,
    pub failed_count: usize,
    pub private_state_exposed: bool,
    pub checks: Vec<EpiphanyServiceExecutionAuditCheck>,
}

pub fn epiphany_service_execution_audit_report(
    receipts: &[EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry],
) -> EpiphanyServiceExecutionAuditReport {
    epiphany_service_execution_audit_report_for_expected(
        receipts,
        &[
            ("windows-service-execution-runbook", &["written"][..]),
            (
                "windows-service-execution-readiness",
                &["elevated-ready"][..],
            ),
            (
                "windows-service-install",
                &["install-command-succeeded"][..],
            ),
            ("windows-service-start", &["start-requested"][..]),
            (
                "windows-service-status",
                &["running", "present", "stopped"][..],
            ),
            ("windows-service-reconcile", &["in-sync"][..]),
            ("windows-service-stop", &["stop-requested"][..]),
        ],
    )
}

pub fn epiphany_cluster_service_execution_audit_report(
    receipts: &[EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry],
) -> EpiphanyServiceExecutionAuditReport {
    epiphany_service_execution_audit_report_for_expected(
        receipts,
        &[
            (
                "cluster-windows-service-execution-runbook",
                &["written"][..],
            ),
            (
                "cluster-windows-service-execution-readiness",
                &["elevated-ready"][..],
            ),
            (
                "cluster-windows-service-install",
                &["install-command-succeeded"][..],
            ),
            ("cluster-windows-service-start", &["start-requested"][..]),
            ("cluster-windows-service-audit", &["complete"][..]),
            ("cluster-windows-service-stop", &["stop-requested"][..]),
        ],
    )
}

fn epiphany_service_execution_audit_report_for_expected(
    receipts: &[EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry],
    expected: &[(&str, &[&str])],
) -> EpiphanyServiceExecutionAuditReport {
    let mut checks = Vec::new();
    let mut missing_count = 0_usize;
    let mut failed_count = 0_usize;
    let mut private_state_exposed = false;
    let mut service_ids = receipts
        .iter()
        .map(|receipt| receipt.service_id.as_str())
        .collect::<Vec<_>>();
    service_ids.sort();
    service_ids.dedup();
    let inferred_service_id = if service_ids.len() == 1 {
        Some(service_ids[0].to_string())
    } else {
        None
    };

    for (action, allowed_statuses) in expected {
        let receipt = latest_lifecycle_receipt_for_action(receipts, action);
        let (service_id, receipt_id, observed_status, operator_artifact_ref, ok, sealed) =
            match receipt {
                Some(receipt) => {
                    let status_ok = allowed_statuses
                        .iter()
                        .any(|allowed| *allowed == receipt.status);
                    (
                        Some(receipt.service_id.clone()),
                        Some(receipt.receipt_id.clone()),
                        Some(receipt.status.clone()),
                        non_empty_operator_artifact_ref(receipt),
                        status_ok,
                        !receipt.private_state_exposed,
                    )
                }
                None => {
                    missing_count += 1;
                    (inferred_service_id.clone(), None, None, None, false, true)
                }
            };

        if !ok {
            failed_count += 1;
        }
        if !sealed {
            private_state_exposed = true;
        }

        checks.push(EpiphanyServiceExecutionAuditCheck {
            service_id,
            action: (*action).to_string(),
            allowed_statuses: allowed_statuses
                .iter()
                .map(|status| (*status).to_string())
                .collect(),
            receipt_id,
            observed_status,
            operator_artifact_ref,
            ok,
            private_state_sealed: sealed,
        });
    }

    let status = if missing_count == 0 && failed_count == 0 && !private_state_exposed {
        "complete"
    } else {
        "incomplete"
    }
    .to_string();

    EpiphanyServiceExecutionAuditReport {
        status,
        receipt_count: receipts.len(),
        missing_count,
        failed_count,
        private_state_exposed,
        checks,
    }
}

fn non_empty_operator_artifact_ref(
    receipt: &EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
) -> Option<String> {
    let artifact_ref = receipt.operator_artifact_ref.trim();
    if artifact_ref.is_empty() || artifact_ref == "none" {
        None
    } else {
        Some(receipt.operator_artifact_ref.clone())
    }
}

fn latest_lifecycle_receipt_for_action<'a>(
    receipts: &'a [EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry],
    action: &str,
) -> Option<&'a EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry> {
    receipts
        .iter()
        .filter(|receipt| receipt.action == action)
        .max_by(|left, right| {
            lifecycle_receipt_sort_key(left).cmp(&lifecycle_receipt_sort_key(right))
        })
}

fn lifecycle_receipt_sort_key(
    receipt: &EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
) -> (&str, &str) {
    (
        receipt
            .completed_at_utc
            .as_deref()
            .unwrap_or(receipt.started_at_utc.as_str()),
        receipt.receipt_id.as_str(),
    )
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
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.persona_speech_audit",
    schema = "EpiphanyCultMeshPersonaSpeechAuditEntry"
)]
pub struct EpiphanyCultMeshPersonaSpeechAuditEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub audit_id: String,
    #[cultcache(key = 2)]
    pub runtime_id: String,
    #[cultcache(key = 3)]
    pub verse_id: String,
    #[cultcache(key = 4)]
    pub persona_agent_id: String,
    #[cultcache(key = 5)]
    pub action_kind: String,
    #[cultcache(key = 6)]
    pub decision: String,
    #[cultcache(key = 7)]
    pub content_fingerprint: String,
    #[cultcache(key = 8)]
    pub opening_key: String,
    #[cultcache(key = 9)]
    pub topic_key: String,
    #[cultcache(key = 10)]
    pub requested_channel_id: String,
    #[cultcache(key = 11)]
    pub recent_window_count: u32,
    #[cultcache(key = 12)]
    pub repeated_opening_count: u32,
    #[cultcache(key = 13)]
    pub repeated_topic_count: u32,
    #[cultcache(key = 14)]
    pub same_channel_post_count: u32,
    #[cultcache(key = 15)]
    pub reasons: Vec<String>,
    #[cultcache(key = 16)]
    pub artifact_ref: String,
    #[cultcache(key = 17)]
    pub created_at_utc: String,
    #[cultcache(key = 18)]
    pub private_state_exposed: bool,
    #[cultcache(key = 19)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.weksa_lowering_receipt",
    schema = "EpiphanyCultMeshWeksaLoweringReceiptEntry"
)]
pub struct EpiphanyCultMeshWeksaLoweringReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub runtime_id: String,
    #[cultcache(key = 3)]
    pub verse_id: String,
    #[cultcache(key = 4)]
    pub packet_id: String,
    #[cultcache(key = 5)]
    pub request_id: String,
    #[cultcache(key = 6)]
    pub persona_agent_id: String,
    #[cultcache(key = 7)]
    pub target_language: String,
    #[cultcache(key = 8)]
    pub target_register: String,
    #[cultcache(key = 9)]
    pub delivery_surface: String,
    #[cultcache(key = 10)]
    pub lowering_method: String,
    #[cultcache(key = 11)]
    pub transport_authority: String,
    #[cultcache(key = 12)]
    pub publication_authorized: bool,
    #[cultcache(key = 13)]
    pub lowered_text_ref: String,
    #[cultcache(key = 14)]
    pub lowered_text_preview: String,
    #[cultcache(key = 15)]
    pub created_at_utc: String,
    #[cultcache(key = 16)]
    pub private_state_exposed: bool,
    #[cultcache(key = 17)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.daemon_tool_capability",
    schema = "EpiphanyCultMeshDaemonToolCapabilityEntry"
)]
pub struct EpiphanyCultMeshDaemonToolCapabilityEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub capability_id: String,
    #[cultcache(key = 2)]
    pub host_cluster_id: String,
    #[cultcache(key = 3)]
    pub host_daemon_id: String,
    #[cultcache(key = 4)]
    pub eve_surface_id: String,
    #[cultcache(key = 5)]
    pub tool_name: String,
    #[cultcache(key = 6)]
    pub operation: String,
    #[cultcache(key = 7)]
    pub input_contract_type: String,
    #[cultcache(key = 8)]
    pub receipt_contract_type: String,
    #[cultcache(key = 9)]
    pub available_to_all_agents: bool,
    #[cultcache(key = 10)]
    pub requires_receipt: bool,
    #[cultcache(key = 11)]
    pub authority_gate: String,
    #[cultcache(key = 12)]
    pub private_state_exposed: bool,
    #[cultcache(key = 13)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.daemon_tool_invocation_intent",
    schema = "EpiphanyCultMeshDaemonToolInvocationIntentEntry"
)]
pub struct EpiphanyCultMeshDaemonToolInvocationIntentEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub intent_id: String,
    #[cultcache(key = 2)]
    pub requesting_agent_id: String,
    #[cultcache(key = 3)]
    pub requesting_cluster_id: String,
    #[cultcache(key = 4)]
    pub capability_id: String,
    #[cultcache(key = 5)]
    pub host_cluster_id: String,
    #[cultcache(key = 6)]
    pub host_daemon_id: String,
    #[cultcache(key = 7)]
    pub eve_surface_id: String,
    #[cultcache(key = 8)]
    pub tool_name: String,
    #[cultcache(key = 9)]
    pub operation: String,
    #[cultcache(key = 10)]
    pub input_contract_type: String,
    #[cultcache(key = 11)]
    pub payload_ref: String,
    #[cultcache(key = 12)]
    pub payload_summary: String,
    #[cultcache(key = 13)]
    pub authority_gate: String,
    #[cultcache(key = 14)]
    pub requires_receipt: bool,
    #[cultcache(key = 15)]
    pub private_state_requested: bool,
    #[cultcache(key = 16)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.daemon_tool_invocation_receipt",
    schema = "EpiphanyCultMeshDaemonToolInvocationReceiptEntry"
)]
pub struct EpiphanyCultMeshDaemonToolInvocationReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub intent_id: String,
    #[cultcache(key = 3)]
    pub requesting_agent_id: String,
    #[cultcache(key = 4)]
    pub requesting_cluster_id: String,
    #[cultcache(key = 5)]
    pub capability_id: String,
    #[cultcache(key = 6)]
    pub host_cluster_id: String,
    #[cultcache(key = 7)]
    pub host_daemon_id: String,
    #[cultcache(key = 8)]
    pub tool_name: String,
    #[cultcache(key = 9)]
    pub operation: String,
    #[cultcache(key = 10)]
    pub status: String,
    #[cultcache(key = 11)]
    pub receipt_contract_type: String,
    #[cultcache(key = 12)]
    pub result_ref: String,
    #[cultcache(key = 13)]
    pub result_summary: String,
    #[cultcache(key = 14)]
    pub authority_gate: String,
    #[cultcache(key = 15)]
    pub private_state_exposed: bool,
    #[cultcache(key = 16)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.mind_contract",
    schema = "EpiphanyCultMeshMindContractEntry"
)]
pub struct EpiphanyCultMeshMindContractEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub contract_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub document_type: String,
    #[cultcache(key = 4)]
    pub payload_schema_version: String,
    #[cultcache(key = 5)]
    pub authority: String,
    #[cultcache(key = 6)]
    pub operations: Vec<String>,
    #[cultcache(key = 7)]
    pub intent_document_types: Vec<String>,
    #[cultcache(key = 8)]
    pub receipt_document_types: Vec<String>,
    #[cultcache(key = 9)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.substrate_gate_contract",
    schema = "EpiphanyCultMeshSubstrateGateContractEntry"
)]
pub struct EpiphanyCultMeshSubstrateGateContractEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub contract_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub document_type: String,
    #[cultcache(key = 4)]
    pub payload_schema_version: String,
    #[cultcache(key = 5)]
    pub authority: String,
    #[cultcache(key = 6)]
    pub operations: Vec<String>,
    #[cultcache(key = 7)]
    pub intent_document_types: Vec<String>,
    #[cultcache(key = 8)]
    pub receipt_document_types: Vec<String>,
    #[cultcache(key = 9)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.eyes_contract",
    schema = "EpiphanyCultMeshEyesContractEntry"
)]
pub struct EpiphanyCultMeshEyesContractEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub contract_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub document_type: String,
    #[cultcache(key = 4)]
    pub payload_schema_version: String,
    #[cultcache(key = 5)]
    pub authority: String,
    #[cultcache(key = 6)]
    pub operations: Vec<String>,
    #[cultcache(key = 7)]
    pub intent_document_types: Vec<String>,
    #[cultcache(key = 8)]
    pub receipt_document_types: Vec<String>,
    #[cultcache(key = 9)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.hands_contract",
    schema = "EpiphanyCultMeshHandsContractEntry"
)]
pub struct EpiphanyCultMeshHandsContractEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub contract_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub document_type: String,
    #[cultcache(key = 4)]
    pub payload_schema_version: String,
    #[cultcache(key = 5)]
    pub authority: String,
    #[cultcache(key = 6)]
    pub operations: Vec<String>,
    #[cultcache(key = 7)]
    pub intent_document_types: Vec<String>,
    #[cultcache(key = 8)]
    pub receipt_document_types: Vec<String>,
    #[cultcache(key = 9)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.soul_contract",
    schema = "EpiphanyCultMeshSoulContractEntry"
)]
pub struct EpiphanyCultMeshSoulContractEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub contract_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub document_type: String,
    #[cultcache(key = 4)]
    pub payload_schema_version: String,
    #[cultcache(key = 5)]
    pub authority: String,
    #[cultcache(key = 6)]
    pub operations: Vec<String>,
    #[cultcache(key = 7)]
    pub intent_document_types: Vec<String>,
    #[cultcache(key = 8)]
    pub receipt_document_types: Vec<String>,
    #[cultcache(key = 9)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.continuity_contract",
    schema = "EpiphanyCultMeshContinuityContractEntry"
)]
pub struct EpiphanyCultMeshContinuityContractEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub contract_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub document_type: String,
    #[cultcache(key = 4)]
    pub payload_schema_version: String,
    #[cultcache(key = 5)]
    pub authority: String,
    #[cultcache(key = 6)]
    pub operations: Vec<String>,
    #[cultcache(key = 7)]
    pub intent_document_types: Vec<String>,
    #[cultcache(key = 8)]
    pub receipt_document_types: Vec<String>,
    #[cultcache(key = 9)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.cultmesh.bifrost_contract",
    schema = "EpiphanyCultMeshBifrostContractEntry"
)]
pub struct EpiphanyCultMeshBifrostContractEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub contract_id: String,
    #[cultcache(key = 2)]
    pub verse_id: String,
    #[cultcache(key = 3)]
    pub document_type: String,
    #[cultcache(key = 4)]
    pub payload_schema_version: String,
    #[cultcache(key = 5)]
    pub authority: String,
    #[cultcache(key = 6)]
    pub operations: Vec<String>,
    #[cultcache(key = 7)]
    pub intent_document_types: Vec<String>,
    #[cultcache(key = 8)]
    pub receipt_document_types: Vec<String>,
    #[cultcache(key = 9)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.bifrost.body_change_publication_intent",
    schema = "EpiphanyCultMeshBifrostBodyChangePublicationIntentEntry"
)]
pub struct EpiphanyCultMeshBifrostBodyChangePublicationIntentEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub intent_id: String,
    #[cultcache(key = 2)]
    pub source_cluster_id: String,
    #[cultcache(key = 3)]
    pub source_agent_id: String,
    #[cultcache(key = 4)]
    pub body_domain: String,
    #[cultcache(key = 5)]
    pub target_repository: String,
    #[cultcache(key = 6)]
    pub target_branch: String,
    #[cultcache(key = 7)]
    pub change_summary: String,
    #[cultcache(key = 8)]
    pub justification: String,
    #[cultcache(key = 9)]
    pub changed_paths: Vec<String>,
    #[cultcache(key = 10)]
    pub verification_receipt_ids: Vec<String>,
    #[cultcache(key = 11)]
    pub review_receipt_ids: Vec<String>,
    #[cultcache(key = 12)]
    pub authorship_agent_ids: Vec<String>,
    #[cultcache(key = 13)]
    pub credit_subjects: Vec<String>,
    #[cultcache(key = 14)]
    pub github_publication_requested: bool,
    #[cultcache(key = 15)]
    pub private_state_included: bool,
    #[cultcache(key = 16)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.bifrost.body_change_publication_receipt",
    schema = "EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry"
)]
pub struct EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub intent_id: String,
    #[cultcache(key = 3)]
    pub status: String,
    #[cultcache(key = 4)]
    pub bifrost_ledger_entry_id: String,
    #[cultcache(key = 5)]
    pub github_publication_receipt_id: String,
    #[cultcache(key = 6)]
    pub credit_receipt_ids: Vec<String>,
    #[cultcache(key = 7)]
    pub accepted_changed_paths: Vec<String>,
    #[cultcache(key = 8)]
    pub reviewer_ids: Vec<String>,
    #[cultcache(key = 9)]
    pub publication_url: String,
    #[cultcache(key = 10)]
    pub private_state_exposed: bool,
    #[cultcache(key = 11)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.bifrost.github_publication_receipt",
    schema = "EpiphanyCultMeshBifrostGithubPublicationReceiptEntry"
)]
pub struct EpiphanyCultMeshBifrostGithubPublicationReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub bifrost_publication_receipt_id: String,
    #[cultcache(key = 3)]
    pub hands_pr_receipt_id: String,
    #[cultcache(key = 4)]
    pub target_repository: String,
    #[cultcache(key = 5)]
    pub target_branch: String,
    #[cultcache(key = 6)]
    pub pull_request_url: String,
    #[cultcache(key = 7)]
    pub pull_request_number: String,
    #[cultcache(key = 8)]
    pub commit_sha: String,
    #[cultcache(key = 9)]
    pub changed_paths: Vec<String>,
    #[cultcache(key = 10)]
    pub ledger_entry_id: String,
    #[cultcache(key = 11)]
    pub credit_receipt_ids: Vec<String>,
    #[cultcache(key = 12)]
    pub published_by_agent_id: String,
    #[cultcache(key = 13)]
    pub publication_status: String,
    #[cultcache(key = 14)]
    pub private_state_exposed: bool,
    #[cultcache(key = 15)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.bifrost.public_proof_publication_receipt",
    schema = "EpiphanyCultMeshBifrostPublicProofPublicationReceiptEntry"
)]
pub struct EpiphanyCultMeshBifrostPublicProofPublicationReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub public_proof_id: String,
    #[cultcache(key = 3)]
    pub public_proof_ref: String,
    #[cultcache(key = 4)]
    pub public_proof_sha256: String,
    #[cultcache(key = 5)]
    pub item: String,
    #[cultcache(key = 6)]
    pub source_workspace: String,
    #[cultcache(key = 7)]
    pub source_branch: String,
    #[cultcache(key = 8)]
    pub target_public_verse_id: String,
    #[cultcache(key = 9)]
    pub public_room_id: String,
    #[cultcache(key = 10)]
    pub status: String,
    #[cultcache(key = 11)]
    pub bifrost_ledger_entry_id: String,
    #[cultcache(key = 12)]
    pub credit_receipt_ids: Vec<String>,
    #[cultcache(key = 13)]
    pub reviewer_ids: Vec<String>,
    #[cultcache(key = 14)]
    pub publication_url: String,
    #[cultcache(key = 15)]
    pub private_state_exposed: bool,
    #[cultcache(key = 16)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.bifrost.artifact_acceptance_receipt",
    schema = "EpiphanyCultMeshBifrostArtifactAcceptanceReceiptEntry"
)]
pub struct EpiphanyCultMeshBifrostArtifactAcceptanceReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub item: String,
    #[cultcache(key = 3)]
    pub source_workspace: String,
    #[cultcache(key = 4)]
    pub source_branch: String,
    #[cultcache(key = 5)]
    pub commit_sha: String,
    #[cultcache(key = 6)]
    pub changed_paths: Vec<String>,
    #[cultcache(key = 7)]
    pub artifact_ref: String,
    #[cultcache(key = 8)]
    pub public_proof_ref: String,
    #[cultcache(key = 9)]
    pub maintainer_review_receipt_ids: Vec<String>,
    #[cultcache(key = 10)]
    pub bifrost_ledger_entry_id: String,
    #[cultcache(key = 11)]
    pub status: String,
    #[cultcache(key = 12)]
    pub accepted_by: String,
    #[cultcache(key = 13)]
    pub private_state_exposed: bool,
    #[cultcache(key = 14)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.bifrost.metrics_receipt",
    schema = "EpiphanyCultMeshBifrostMetricsReceiptEntry"
)]
pub struct EpiphanyCultMeshBifrostMetricsReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub item: String,
    #[cultcache(key = 3)]
    pub source_workspace: String,
    #[cultcache(key = 4)]
    pub source_branch: String,
    #[cultcache(key = 5)]
    pub artifact_acceptance_receipt_id: String,
    #[cultcache(key = 6)]
    pub model_spend_receipt_ids: Vec<String>,
    #[cultcache(key = 7)]
    pub review_load_receipt_ids: Vec<String>,
    #[cultcache(key = 8)]
    pub credit_readback_receipt_ids: Vec<String>,
    #[cultcache(key = 9)]
    pub public_proof_ref: String,
    #[cultcache(key = 10)]
    pub metrics_summary: String,
    #[cultcache(key = 11)]
    pub status: String,
    #[cultcache(key = 12)]
    pub private_state_exposed: bool,
    #[cultcache(key = 13)]
    pub notes: Vec<String>,
    #[cultcache(key = 14)]
    pub token_summary_ref: Option<String>,
    #[cultcache(key = 15)]
    pub cost_availability_status: Option<String>,
    #[cultcache(key = 16)]
    pub cost_summary_ref: Option<String>,
    #[cultcache(key = 17)]
    pub cost_unavailable_reason: Option<String>,
    #[cultcache(key = 18)]
    pub review_duration_ms: Option<u64>,
    #[cultcache(key = 19)]
    pub review_event_count: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.bifrost.collaboration_feedback",
    schema = "EpiphanyCultMeshBifrostCollaborationFeedbackEntry"
)]
pub struct EpiphanyCultMeshBifrostCollaborationFeedbackEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub feedback_id: String,
    #[cultcache(key = 2)]
    pub source_persona_id: String,
    #[cultcache(key = 3)]
    pub source_cluster_id: String,
    #[cultcache(key = 4)]
    pub public_room_id: String,
    #[cultcache(key = 5)]
    pub eve_connection_receipt_id: String,
    #[cultcache(key = 6)]
    pub collaboration_topic: String,
    #[cultcache(key = 7)]
    pub feedback_summary: String,
    #[cultcache(key = 8)]
    pub public_discussion_refs: Vec<String>,
    #[cultcache(key = 9)]
    pub requested_consensus_route: String,
    #[cultcache(key = 10)]
    pub candidate_action_refs: Vec<String>,
    #[cultcache(key = 11)]
    pub private_state_included: bool,
    #[cultcache(key = 12)]
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.imagination.consensus_discovery_receipt",
    schema = "EpiphanyCultMeshImaginationConsensusReceiptEntry"
)]
pub struct EpiphanyCultMeshImaginationConsensusReceiptEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub feedback_id: String,
    #[cultcache(key = 3)]
    pub source_persona_id: String,
    #[cultcache(key = 4)]
    pub consensus_route: String,
    #[cultcache(key = 5)]
    pub status: String,
    #[cultcache(key = 6)]
    pub imagination_agent_ids: Vec<String>,
    #[cultcache(key = 7)]
    pub consensus_packet_ref: String,
    #[cultcache(key = 8)]
    pub adoption_gate: String,
    #[cultcache(key = 9)]
    pub public_feedback_refs: Vec<String>,
    #[cultcache(key = 10)]
    pub private_state_exposed: bool,
    #[cultcache(key = 11)]
    pub notes: Vec<String>,
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
    pub verse_policies: Vec<EpiphanyCultMeshVersePolicyEntry>,
    pub global_room_policies: Vec<EpiphanyCultMeshGlobalRoomPolicyEntry>,
    pub cluster_topology: Vec<EpiphanyCultMeshClusterTopologyEntry>,
    pub odin_advertisements: Vec<EpiphanyCultMeshOdinAdvertisementEntry>,
    pub eve_surface_states: Vec<EpiphanyCultMeshEveSurfaceStateEntry>,
    pub daemon_statuses: Vec<EpiphanyCultMeshDaemonStatusEntry>,
    pub latest_daemon_poke_intent: Option<EpiphanyCultMeshDaemonPokeIntentEntry>,
    pub latest_daemon_poke_receipt: Option<EpiphanyCultMeshDaemonPokeReceiptEntry>,
    pub daemon_restart_policies: Vec<EpiphanyCultMeshDaemonRestartPolicyEntry>,
    pub latest_daemon_scheduler_receipt: Option<EpiphanyCultMeshDaemonSchedulerReceiptEntry>,
    pub latest_daemon_service_lifecycle_receipt:
        Option<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry>,
    pub latest_idunn_deployment_receipt: Option<EpiphanyCultMeshIdunnDeploymentReceiptEntry>,
    pub latest_idunn_aftercare_audit_receipt:
        Option<EpiphanyCultMeshIdunnAftercareAuditReceiptEntry>,
    pub swarm_brake: Option<EpiphanyCultMeshSwarmBrakeEntry>,
    pub latest_persona_speech_audit: Option<EpiphanyCultMeshPersonaSpeechAuditEntry>,
    pub latest_weksa_lowering_receipt: Option<EpiphanyCultMeshWeksaLoweringReceiptEntry>,
    pub daemon_tool_capabilities: Vec<EpiphanyCultMeshDaemonToolCapabilityEntry>,
    pub latest_daemon_tool_invocation_intent:
        Option<EpiphanyCultMeshDaemonToolInvocationIntentEntry>,
    pub latest_daemon_tool_invocation_receipt:
        Option<EpiphanyCultMeshDaemonToolInvocationReceiptEntry>,
    pub arrival_latest_bifrost_body_change_publication_intent:
        Option<EpiphanyCultMeshBifrostBodyChangePublicationIntentEntry>,
    pub arrival_latest_bifrost_body_change_publication_receipt:
        Option<EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry>,
    pub arrival_latest_bifrost_github_publication_receipt:
        Option<EpiphanyCultMeshBifrostGithubPublicationReceiptEntry>,
    pub arrival_latest_bifrost_public_proof_publication_receipt:
        Option<EpiphanyCultMeshBifrostPublicProofPublicationReceiptEntry>,
    pub arrival_latest_bifrost_collaboration_feedback:
        Option<EpiphanyCultMeshBifrostCollaborationFeedbackEntry>,
    pub latest_imagination_consensus_receipt:
        Option<EpiphanyCultMeshImaginationConsensusReceiptEntry>,
    pub latest_operator_snapshot: Option<EpiphanyCultMeshOperatorSnapshotEntry>,
    pub latest_operator_run_intent: Option<EpiphanyCultMeshOperatorRunIntentEntry>,
    pub latest_operator_run_receipt: Option<EpiphanyCultMeshOperatorRunReceiptEntry>,
    pub latest_coordinator_run_receipt: Option<EpiphanyCultMeshCoordinatorRunReceiptEntry>,
    pub latest_hands_action_gate: Option<EpiphanyCultMeshHandsActionGateEntry>,
    pub latest_role_review_event: Option<EpiphanyCultMeshRoleReviewEventEntry>,
    pub latest_work_loop_summary: Option<EpiphanyLocalVerseWorkLoopSummary>,
    pub latest_agent_state_soa_summary: Option<EpiphanyCultMeshAgentStateSoaSummaryEntry>,
    pub latest_repo_work_overview: Option<EpiphanyCultMeshRepoWorkOverviewEntry>,
    pub latest_repo_work_map_entry: Option<EpiphanyCultMeshRepoWorkMapEntry>,
    pub latest_eve_connection_intent: Option<EpiphanyCultMeshEveConnectionIntentEntry>,
    pub latest_eve_connection_receipt: Option<EpiphanyCultMeshEveConnectionReceiptEntry>,
    pub contract_summaries: Vec<EpiphanyLocalVerseContractSummary>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyLocalVerseWorkLoopSummary {
    pub telemetry_id: String,
    pub thread_id: String,
    pub source_stage: String,
    pub target_stages: Vec<String>,
    pub hands_intent_id: String,
    pub hands_review_id: String,
    pub substrate_gate_grant_receipt_id: String,
    pub hands_patch_receipt_id: String,
    pub hands_command_receipt_id: String,
    pub hands_commit_receipt_id: String,
    pub commit_sha: String,
    pub branch: String,
    pub changed_path_count: usize,
    pub source_ref_count: usize,
    pub soul_receipt_ids: Vec<String>,
    pub verification_assertion_count: usize,
    pub summary: String,
    pub sealed_preview_note: String,
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
    EpiphanyCultMeshStatusEntry => EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION,
    EpiphanyCultMeshOperatorSnapshotEntry => EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_SCHEMA_VERSION,
    EpiphanyCultMeshOperatorRunIntentEntry => EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION,
    EpiphanyCultMeshOperatorRunReceiptEntry => EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshCoordinatorRunReceiptEntry => EPIPHANY_CULTMESH_COORDINATOR_RUN_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshHandsActionGateEntry => EPIPHANY_CULTMESH_HANDS_ACTION_GATE_SCHEMA_VERSION,
    EpiphanyCultMeshRoleReviewEventEntry => EPIPHANY_CULTMESH_ROLE_REVIEW_EVENT_SCHEMA_VERSION,
    EpiphanyCultMeshWorkLoopTelemetryEntry => EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_SCHEMA_VERSION,
    EpiphanyCultMeshAgentStateSoaSummaryEntry => EPIPHANY_CULTMESH_AGENT_STATE_SOA_SUMMARY_SCHEMA_VERSION,
    EpiphanyCultMeshRepoWorkOverviewEntry => EPIPHANY_CULTMESH_REPO_WORK_OVERVIEW_SCHEMA_VERSION,
    EpiphanyCultMeshRepoWorkReadinessEntry => EPIPHANY_CULTMESH_REPO_WORK_READINESS_SCHEMA_VERSION,
    EpiphanyCultMeshRepoWorkMapEntry => EPIPHANY_CULTMESH_REPO_WORK_MAP_ENTRY_SCHEMA_VERSION,
    EpiphanyCultMeshRepoWorkPublicProofEntry => EPIPHANY_CULTMESH_REPO_WORK_PUBLIC_PROOF_SCHEMA_VERSION,
    EpiphanyCultMeshVersePolicyEntry => EPIPHANY_CULTMESH_VERSE_POLICY_SCHEMA_VERSION,
    EpiphanyCultMeshGlobalRoomPolicyEntry => EPIPHANY_CULTMESH_GLOBAL_ROOM_POLICY_SCHEMA_VERSION,
    EpiphanyCultMeshClusterTopologyEntry => EPIPHANY_CULTMESH_CLUSTER_TOPOLOGY_SCHEMA_VERSION,
    EpiphanyCultMeshOdinAdvertisementEntry => EPIPHANY_CULTMESH_ODIN_ADVERTISEMENT_SCHEMA_VERSION,
    EpiphanyCultMeshEveConnectionIntentEntry => EPIPHANY_CULTMESH_EVE_CONNECTION_INTENT_SCHEMA_VERSION,
    EpiphanyCultMeshEveConnectionReceiptEntry => EPIPHANY_CULTMESH_EVE_CONNECTION_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshEveSurfaceStateEntry => EPIPHANY_CULTMESH_EVE_SURFACE_STATE_SCHEMA_VERSION,
    EpiphanyCultMeshDaemonStatusEntry => EPIPHANY_CULTMESH_DAEMON_STATUS_SCHEMA_VERSION,
    EpiphanyCultMeshDaemonHeartbeatEventEntry => EPIPHANY_CULTMESH_DAEMON_HEARTBEAT_EVENT_SCHEMA_VERSION,
    EpiphanyCultMeshDaemonPokeIntentEntry => EPIPHANY_CULTMESH_DAEMON_POKE_INTENT_SCHEMA_VERSION,
    EpiphanyCultMeshDaemonPokeReceiptEntry => EPIPHANY_CULTMESH_DAEMON_POKE_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshDaemonRestartPolicyEntry => EPIPHANY_CULTMESH_DAEMON_RESTART_POLICY_SCHEMA_VERSION,
    EpiphanyCultMeshDaemonSchedulerReceiptEntry => EPIPHANY_CULTMESH_DAEMON_SCHEDULER_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry => EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshManagedServicePolicyEntry => EPIPHANY_CULTMESH_MANAGED_SERVICE_POLICY_SCHEMA_VERSION,
    WorkspaceCoverageManagedProcessLaunchEntry => WORKSPACE_COVERAGE_PROCESS_LAUNCH_SCHEMA_VERSION,
    WorkspaceCoverageProcessEvidenceHead => WORKSPACE_COVERAGE_PROCESS_EVIDENCE_HEAD_SCHEMA_VERSION,
    WorkspaceCoverageProviderHeartbeatEntry => WORKSPACE_COVERAGE_PROVIDER_HEARTBEAT_SCHEMA_VERSION,
    WorkspaceCoverageProcessTerminationObservationEntry => WORKSPACE_COVERAGE_PROCESS_TERMINATION_SCHEMA_VERSION,
    EpiphanyPackagedReleaseEntry => crate::packaged_release::EPIPHANY_PACKAGED_RELEASE_SCHEMA_VERSION,
    EpiphanyPackagedReleaseHead => crate::packaged_release::EPIPHANY_PACKAGED_RELEASE_HEAD_SCHEMA_VERSION,
    EpiphanyCultMeshIdunnDeploymentReceiptEntry => EPIPHANY_CULTMESH_IDUNN_DEPLOYMENT_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshIdunnAftercareAuditReceiptEntry => EPIPHANY_CULTMESH_IDUNN_AFTERCARE_AUDIT_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshSwarmBrakeEntry => EPIPHANY_CULTMESH_SWARM_BRAKE_SCHEMA_VERSION,
    EpiphanyCultMeshPersonaSpeechAuditEntry => EPIPHANY_CULTMESH_PERSONA_SPEECH_AUDIT_SCHEMA_VERSION,
    EpiphanyCultMeshWeksaLoweringReceiptEntry => EPIPHANY_CULTMESH_WEKSA_LOWERING_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshDaemonToolCapabilityEntry => EPIPHANY_CULTMESH_DAEMON_TOOL_CAPABILITY_SCHEMA_VERSION,
    EpiphanyCultMeshDaemonToolInvocationIntentEntry => EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_INTENT_SCHEMA_VERSION,
    EpiphanyCultMeshDaemonToolInvocationReceiptEntry => EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshMindContractEntry => EPIPHANY_CULTMESH_MIND_CONTRACT_SCHEMA_VERSION,
    EpiphanyCultMeshSubstrateGateContractEntry => EPIPHANY_CULTMESH_SUBSTRATE_GATE_CONTRACT_SCHEMA_VERSION,
    EpiphanyCultMeshEyesContractEntry => EPIPHANY_CULTMESH_EYES_CONTRACT_SCHEMA_VERSION,
    EpiphanyCultMeshHandsContractEntry => EPIPHANY_CULTMESH_HANDS_CONTRACT_SCHEMA_VERSION,
    EpiphanyCultMeshSoulContractEntry => EPIPHANY_CULTMESH_SOUL_CONTRACT_SCHEMA_VERSION,
    EpiphanyCultMeshContinuityContractEntry => EPIPHANY_CULTMESH_CONTINUITY_CONTRACT_SCHEMA_VERSION,
    EpiphanyCultMeshBifrostContractEntry => EPIPHANY_CULTMESH_BIFROST_CONTRACT_SCHEMA_VERSION,
    EpiphanyCultMeshBifrostBodyChangePublicationIntentEntry => EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_INTENT_SCHEMA_VERSION,
    EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry => EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshBifrostGithubPublicationReceiptEntry => EPIPHANY_CULTMESH_BIFROST_GITHUB_PUBLICATION_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshBifrostPublicProofPublicationReceiptEntry => EPIPHANY_CULTMESH_BIFROST_PUBLIC_PROOF_PUBLICATION_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshBifrostArtifactAcceptanceReceiptEntry => EPIPHANY_CULTMESH_BIFROST_ARTIFACT_ACCEPTANCE_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshBifrostMetricsReceiptEntry => EPIPHANY_CULTMESH_BIFROST_METRICS_RECEIPT_SCHEMA_VERSION,
    EpiphanyCultMeshBifrostCollaborationFeedbackEntry => EPIPHANY_CULTMESH_BIFROST_COLLABORATION_FEEDBACK_SCHEMA_VERSION,
    EpiphanyCultMeshImaginationConsensusReceiptEntry => EPIPHANY_CULTMESH_IMAGINATION_CONSENSUS_RECEIPT_SCHEMA_VERSION,
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

/// Removes the extinct ownerless operator-status projection from an existing
/// local Verse so the live schema catalog can open it again.
///
/// This is deliberately explicit migration, never implicit node bootstrap.
pub fn retire_epiphany_cultmesh_operator_status_documents(
    store_path: impl AsRef<Path>,
) -> Result<Vec<String>> {
    const RETIRED_TYPE: &str = "epiphany.cultmesh.operator_status";
    let mut backing = SingleFileMessagePackBackingStore::new(store_path.as_ref());
    let retired = backing
        .pull_all()?
        .into_iter()
        .filter(|entry| entry.r#type == RETIRED_TYPE)
        .collect::<Vec<_>>();
    for entry in &retired {
        backing.delete(entry)?;
    }
    Ok(retired.into_iter().map(|entry| entry.key).collect())
}

/// Removes centrally forged provider documents from the pre-provider-contract
/// era. Their v0 payloads carry no provider provenance, so retaining them as
/// live discovery state would let a dead coordinator impersonate providers.
pub fn retire_epiphany_cultmesh_legacy_provider_documents(
    store_path: impl AsRef<Path>,
) -> Result<Vec<String>> {
    const RETIRED_TYPES: [&str; 3] = [
        EPIPHANY_CULTMESH_ODIN_ADVERTISEMENT_TYPE,
        EPIPHANY_CULTMESH_EVE_SURFACE_STATE_TYPE,
        EPIPHANY_CULTMESH_DAEMON_TOOL_CAPABILITY_TYPE,
    ];
    let mut backing = SingleFileMessagePackBackingStore::new(store_path.as_ref());
    let retired = backing
        .pull_all()?
        .into_iter()
        .filter(|entry| RETIRED_TYPES.contains(&entry.r#type.as_str()))
        .collect::<Vec<_>>();
    for entry in &retired {
        backing.delete(entry)?;
    }
    Ok(retired.into_iter().map(|entry| entry.key).collect())
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
            let latest_time = DateTime::parse_from_rfc3339(&latest.observed_source_at)?;
            let entry_time = DateTime::parse_from_rfc3339(&entry.observed_source_at)?;
            if latest.source_generation == entry.source_generation
                && latest.obligation_id != entry.obligation_id
            {
                return Err(anyhow!(
                    "semantic projection health generation has conflicting canonical obligations"
                ));
            }
            if latest.source_generation > entry.source_generation
                || (latest.source_generation == entry.source_generation && latest_time > entry_time)
            {
                return Ok(latest.clone());
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
        || !matches!(row.partition.as_str(), "mind" | "modeling")
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
    if row.query_eligible_display_only != (row.status == "ready")
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

pub fn write_epiphany_cultmesh_status(
    store_path: impl AsRef<Path>,
    status: EpiphanyCultMeshStatusEntry,
) -> Result<EpiphanyCultMeshStatusEntry> {
    let mut node = open_epiphany_cultmesh_node(&store_path, status.runtime_id.clone())?;
    let written = node.put(EPIPHANY_CULTMESH_STATUS_KEY, &status)?;
    node.flush()?;
    Ok(written)
}

pub fn load_epiphany_cultmesh_status(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshStatusEntry>> {
    let store_path = store_path.as_ref();
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_STATUS_KEY)
}

pub fn epiphany_cultmesh_operator_snapshot_from_status_json(
    runtime_id: impl Into<String>,
    snapshot_id: impl Into<String>,
    generated_at_utc: impl Into<String>,
    source_mode: impl Into<String>,
    source_path: impl Into<String>,
    status_json: &Value,
) -> Result<EpiphanyCultMeshOperatorSnapshotEntry> {
    let source_path = source_path.into();
    let state_status = pointer_text(status_json, "/scene/scene/stateStatus", "unknown");
    let crrc_action = pointer_text(status_json, "/crrc/recommendation/action", "unknown");
    let reorient_action = pointer_text(status_json, "/reorient/decision/action", "unknown");
    let snapshot_status = if state_status == "missing" || crrc_action == "regatherManually" {
        "needs-regather"
    } else {
        "ready"
    };
    let mut artifact_refs = Vec::new();
    if !source_path.trim().is_empty() {
        artifact_refs.push(source_path.clone());
    }

    Ok(EpiphanyCultMeshOperatorSnapshotEntry {
        schema_version: EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_SCHEMA_VERSION.to_string(),
        runtime_id: runtime_id.into(),
        verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
        snapshot_id: snapshot_id.into(),
        generated_at_utc: generated_at_utc.into(),
        source_mode: source_mode.into(),
        source_path,
        thread_id: pointer_text(status_json, "/threadId", "missing"),
        status: snapshot_status.to_string(),
        state_status,
        coordinator_action: pointer_text(status_json, "/coordinator/action", "none"),
        crrc_action,
        pressure_level: pointer_text(status_json, "/pressure/pressure/level", "unknown"),
        reorient_action,
        next_action: pointer_text(status_json, "/reorient/decision/nextAction", "none"),
        artifact_refs,
        available_actions: pointer_string_array(status_json, "/scene/scene/availableActions")?,
        notes: vec![
            "Snapshot is derived from the operator-safe MVP status artifact; raw JSON remains an edge artifact, not internal state.".to_string(),
            "Codex app-server is an external edge adapter for this source until the status surface is native end to end; it is not CultMesh transport authority.".to_string(),
        ],
    })
}

#[cfg(test)]
pub fn epiphany_cultmesh_daemon_tool_invocation_from_status_json(
    requesting_cluster_id: impl Into<String>,
    status_path: impl Into<String>,
    status_json: &Value,
) -> Result<
    Option<(
        EpiphanyCultMeshDaemonToolInvocationIntentEntry,
        Option<EpiphanyCultMeshDaemonToolInvocationReceiptEntry>,
    )>,
> {
    let requesting_cluster_id = requesting_cluster_id.into();
    let status_path = status_path.into();
    let Some((index, invocation)) = status_json
        .pointer("/tools/invocations")
        .and_then(Value::as_array)
        .and_then(|items| items.iter().enumerate().next_back())
    else {
        return Ok(None);
    };
    let intent_id = pointer_text(invocation, "/intentId", "");
    if intent_id.trim().is_empty() {
        return Ok(None);
    }
    let adapter = pointer_text(invocation, "/adapter", "epiphany-tool-adapter");
    let server = pointer_text(invocation, "/server", "unknown-server");
    let tool_name = pointer_text(invocation, "/toolName", "");
    let status = pointer_text(invocation, "/status", "pending");
    let intent = EpiphanyCultMeshDaemonToolInvocationIntentEntry {
        schema_version: EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_INTENT_SCHEMA_VERSION
            .to_string(),
        intent_id: intent_id.clone(),
        requesting_agent_id: pointer_text(invocation, "/caller", "unknown-caller"),
        requesting_cluster_id,
        capability_id: format!("runtime-spine/{adapter}/{server}/{tool_name}"),
        host_cluster_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
        host_daemon_id: format!("{adapter}:{server}"),
        eve_surface_id: "epiphany-local/tools".to_string(),
        tool_name: tool_name.clone(),
        operation: "runtimeToolInvocation".to_string(),
        input_contract_type: "epiphany.tool_invocation_intent.v0".to_string(),
        payload_ref: format!("{status_path}#/tools/invocations/{index}"),
        payload_summary: pointer_text(invocation, "/reason", ""),
        authority_gate: "epiphany-tool-adapter".to_string(),
        requires_receipt: true,
        private_state_requested: false,
        notes: vec![
            "Mirror of the operator-safe native status tool invocation surface; runtime-spine tool intent remains the lifecycle owner.".to_string(),
            "Raw MCP JSON stays quarantined behind epiphany.tool_invocation_intent.v0 / receipt documents.".to_string(),
        ],
    };
    validate_daemon_tool_invocation_intent(&intent)?;
    let receipt = invocation
        .pointer("/receiptId")
        .and_then(Value::as_str)
        .filter(|receipt_id| !receipt_id.trim().is_empty())
        .map(|receipt_id| {
            let receipt = EpiphanyCultMeshDaemonToolInvocationReceiptEntry {
                schema_version: EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_RECEIPT_SCHEMA_VERSION
                    .to_string(),
                receipt_id: receipt_id.to_string(),
                intent_id: intent.intent_id.clone(),
                requesting_agent_id: intent.requesting_agent_id.clone(),
                requesting_cluster_id: intent.requesting_cluster_id.clone(),
                capability_id: intent.capability_id.clone(),
                host_cluster_id: intent.host_cluster_id.clone(),
                host_daemon_id: intent.host_daemon_id.clone(),
                tool_name: intent.tool_name.clone(),
                operation: intent.operation.clone(),
                status: status.clone(),
                receipt_contract_type: "epiphany.tool_invocation_receipt.v0".to_string(),
                result_ref: format!("runtime-spine://tool-invocation-receipts/{receipt_id}"),
                result_summary: pointer_text(invocation, "/error", ""),
                authority_gate: intent.authority_gate.clone(),
                private_state_exposed: false,
                notes: vec![
                    "Mirror of the runtime-spine tool invocation receipt status; runtime-spine remains the receipt owner.".to_string(),
                    "This local Verse receipt exposes routing/status only, not private tool payloads or raw MCP cargo.".to_string(),
                ],
            };
            validate_daemon_tool_invocation_receipt(&receipt)?;
            Ok::<EpiphanyCultMeshDaemonToolInvocationReceiptEntry, anyhow::Error>(receipt)
        })
        .transpose()?;
    Ok(Some((intent, receipt)))
}

pub fn write_epiphany_cultmesh_operator_snapshot(
    store_path: impl AsRef<Path>,
    snapshot: EpiphanyCultMeshOperatorSnapshotEntry,
) -> Result<EpiphanyCultMeshOperatorSnapshotEntry> {
    validate_operator_snapshot(&snapshot)?;
    let mut node = open_epiphany_cultmesh_node(&store_path, snapshot.runtime_id.clone())?;
    let snapshot_key = epiphany_cultmesh_operator_snapshot_key(&snapshot.snapshot_id);
    let written = node.put(snapshot_key.as_str(), &snapshot)?;
    let current_latest = node.get::<EpiphanyCultMeshOperatorSnapshotEntry>(
        EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_LATEST_KEY,
    )?;
    if current_latest.as_ref().is_none_or(|current| {
        operator_snapshot_generation_key(&written) >= operator_snapshot_generation_key(current)
    }) {
        node.put(EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_LATEST_KEY, &written)?;
    }
    node.flush()?;
    Ok(written)
}

fn validate_operator_snapshot(snapshot: &EpiphanyCultMeshOperatorSnapshotEntry) -> Result<()> {
    if snapshot.schema_version != EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_SCHEMA_VERSION {
        return Err(anyhow!("operator snapshot has unsupported schema version"));
    }
    if snapshot.verse_id != EPIPHANY_CULTMESH_INTERNAL_VERSE_ID {
        return Err(anyhow!(
            "operator snapshot must remain in the internal Verse"
        ));
    }
    for (label, value) in [
        ("runtime id", snapshot.runtime_id.as_str()),
        ("snapshot id", snapshot.snapshot_id.as_str()),
        ("source mode", snapshot.source_mode.as_str()),
        ("source path", snapshot.source_path.as_str()),
        ("status", snapshot.status.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("operator snapshot missing {label}"));
        }
    }
    DateTime::parse_from_rfc3339(&snapshot.generated_at_utc)
        .map_err(|error| anyhow!("operator snapshot has invalid generated_at_utc: {error}"))?;
    Ok(())
}

fn operator_snapshot_generation_key(
    snapshot: &EpiphanyCultMeshOperatorSnapshotEntry,
) -> (DateTime<FixedOffset>, &str) {
    (
        DateTime::parse_from_rfc3339(&snapshot.generated_at_utc)
            .expect("validated operator snapshot generation timestamp"),
        snapshot.snapshot_id.as_str(),
    )
}

pub fn load_epiphany_cultmesh_operator_snapshot(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    snapshot_id: impl AsRef<str>,
) -> Result<Option<EpiphanyCultMeshOperatorSnapshotEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let snapshot_key = epiphany_cultmesh_operator_snapshot_key(snapshot_id.as_ref());
    node.get(snapshot_key.as_str())
}

pub fn load_latest_epiphany_cultmesh_operator_snapshot(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshOperatorSnapshotEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_LATEST_KEY)
}

pub fn write_epiphany_cultmesh_operator_run_intent(
    store_path: impl AsRef<Path>,
    intent: EpiphanyCultMeshOperatorRunIntentEntry,
) -> Result<EpiphanyCultMeshOperatorRunIntentEntry> {
    validate_operator_run_intent(&intent)?;
    let mut node = open_epiphany_cultmesh_node(&store_path, intent.runtime_id.clone())?;
    let intent_key = epiphany_cultmesh_operator_run_intent_key(&intent.run_id);
    let written = node.put(intent_key.as_str(), &intent)?;
    let current_latest = node.get::<EpiphanyCultMeshOperatorRunIntentEntry>(
        EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_LATEST_KEY,
    )?;
    if current_latest.as_ref().is_none_or(|current| {
        operator_run_intent_event_key(&written) >= operator_run_intent_event_key(current)
    }) {
        node.put(EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_LATEST_KEY, &written)?;
    }
    node.flush()?;
    Ok(written)
}

pub fn load_epiphany_cultmesh_operator_run_intent(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    run_id: &str,
) -> Result<Option<EpiphanyCultMeshOperatorRunIntentEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(epiphany_cultmesh_operator_run_intent_key(run_id).as_str())
}

pub fn load_latest_epiphany_cultmesh_operator_run_intent(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshOperatorRunIntentEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_LATEST_KEY)
}

pub fn write_epiphany_cultmesh_operator_run_receipt(
    store_path: impl AsRef<Path>,
    receipt: EpiphanyCultMeshOperatorRunReceiptEntry,
) -> Result<EpiphanyCultMeshOperatorRunReceiptEntry> {
    validate_operator_run_receipt(&receipt)?;
    let mut node = open_epiphany_cultmesh_node(&store_path, receipt.runtime_id.clone())?;
    let receipt_key = epiphany_cultmesh_operator_run_receipt_key(&receipt.run_id);
    let written = node.put(receipt_key.as_str(), &receipt)?;
    let current_latest = node.get::<EpiphanyCultMeshOperatorRunReceiptEntry>(
        EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_LATEST_KEY,
    )?;
    if current_latest.as_ref().is_none_or(|current| {
        operator_run_receipt_event_key(&written) >= operator_run_receipt_event_key(current)
    }) {
        node.put(EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_LATEST_KEY, &written)?;
    }
    node.flush()?;
    Ok(written)
}

fn validate_operator_run_intent(intent: &EpiphanyCultMeshOperatorRunIntentEntry) -> Result<()> {
    if intent.schema_version != EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION {
        return Err(anyhow!(
            "operator run intent has unsupported schema version"
        ));
    }
    if intent.verse_id != EPIPHANY_CULTMESH_INTERNAL_VERSE_ID {
        return Err(anyhow!(
            "operator run intent must remain in the internal Verse"
        ));
    }
    for (label, value) in [
        ("runtime id", intent.runtime_id.as_str()),
        ("run id", intent.run_id.as_str()),
        ("mode", intent.mode.as_str()),
        ("root", intent.root.as_str()),
        ("workspace", intent.workspace.as_str()),
        ("artifact root", intent.artifact_root.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("operator run intent missing {label}"));
        }
    }
    DateTime::parse_from_rfc3339(&intent.requested_at_utc)
        .map_err(|error| anyhow!("operator run intent has invalid requested_at_utc: {error}"))?;
    Ok(())
}

fn validate_operator_run_receipt(receipt: &EpiphanyCultMeshOperatorRunReceiptEntry) -> Result<()> {
    if receipt.schema_version != EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION {
        return Err(anyhow!(
            "operator run receipt has unsupported schema version"
        ));
    }
    if receipt.verse_id != EPIPHANY_CULTMESH_INTERNAL_VERSE_ID {
        return Err(anyhow!(
            "operator run receipt must remain in the internal Verse"
        ));
    }
    for (label, value) in [
        ("runtime id", receipt.runtime_id.as_str()),
        ("run id", receipt.run_id.as_str()),
        ("mode", receipt.mode.as_str()),
        ("status", receipt.status.as_str()),
        ("result path", receipt.result_path.as_str()),
        ("artifact root", receipt.artifact_root.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("operator run receipt missing {label}"));
        }
    }
    DateTime::parse_from_rfc3339(&receipt.completed_at_utc)
        .map_err(|error| anyhow!("operator run receipt has invalid completed_at_utc: {error}"))?;
    Ok(())
}

fn operator_run_intent_event_key(
    intent: &EpiphanyCultMeshOperatorRunIntentEntry,
) -> (DateTime<FixedOffset>, &str) {
    (
        DateTime::parse_from_rfc3339(&intent.requested_at_utc)
            .expect("validated operator run intent timestamp"),
        intent.run_id.as_str(),
    )
}

fn operator_run_receipt_event_key(
    receipt: &EpiphanyCultMeshOperatorRunReceiptEntry,
) -> (DateTime<FixedOffset>, &str) {
    (
        DateTime::parse_from_rfc3339(&receipt.completed_at_utc)
            .expect("validated operator run receipt timestamp"),
        receipt.run_id.as_str(),
    )
}

pub fn load_latest_epiphany_cultmesh_operator_run_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshOperatorRunReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_LATEST_KEY)
}

pub fn load_epiphany_cultmesh_operator_run_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    run_id: &str,
) -> Result<Option<EpiphanyCultMeshOperatorRunReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(epiphany_cultmesh_operator_run_receipt_key(run_id).as_str())
}

pub fn epiphany_cultmesh_coordinator_run_receipt_from_summary_json(
    runtime_id: impl Into<String>,
    receipt_id: impl Into<String>,
    created_at_utc: impl Into<String>,
    artifact_root: impl Into<String>,
    summary_json: &Value,
) -> Result<EpiphanyCultMeshCoordinatorRunReceiptEntry> {
    let artifact_root = artifact_root.into();
    let artifact_refs = pointer_string_array(summary_json, "/artifactManifest")?;
    let sealed_artifact_refs = summary_json
        .pointer("/sealedArtifactManifest")
        .and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(|item| {
                    item.get("path")
                        .and_then(Value::as_str)
                        .or_else(|| item.as_str())
                        .map(str::to_string)
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let step_count = summary_json
        .pointer("/steps")
        .and_then(Value::as_array)
        .map_or(0, |items| items.len() as u64);
    let receipt = EpiphanyCultMeshCoordinatorRunReceiptEntry {
        schema_version: EPIPHANY_CULTMESH_COORDINATOR_RUN_RECEIPT_SCHEMA_VERSION.to_string(),
        runtime_id: runtime_id.into(),
        verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
        receipt_id: receipt_id.into(),
        source_document_type: pointer_text(
            summary_json,
            "/coordinatorRunReceipt/documentType",
            "epiphany.coordinator_run_receipt.v0",
        ),
        source_receipt_id: pointer_text(summary_json, "/coordinatorRunReceipt/receiptId", ""),
        source_store: pointer_text(summary_json, "/coordinatorRunReceipt/store", ""),
        thread_id: pointer_text(summary_json, "/threadId", ""),
        mode: pointer_text(summary_json, "/mode", ""),
        status: pointer_text(summary_json, "/finalAction/action", ""),
        final_action: pointer_text(summary_json, "/finalAction/action", ""),
        final_reason: pointer_text(summary_json, "/finalAction/reason", ""),
        step_count,
        artifact_root,
        artifact_refs,
        sealed_artifact_refs,
        created_at_utc: created_at_utc.into(),
        private_state_exposed: false,
        notes: vec![
            "CultMesh mirror of the runtime-spine coordinator receipt; runtime-spine remains the lifecycle owner.".to_string(),
            "Coordinator JSON artifacts are display/audit evidence; sealed transcript and stderr paths are referenced but not opened here.".to_string(),
        ],
    };
    validate_coordinator_run_receipt(&receipt)?;
    Ok(receipt)
}

pub fn write_epiphany_cultmesh_coordinator_run_receipt(
    store_path: impl AsRef<Path>,
    receipt: EpiphanyCultMeshCoordinatorRunReceiptEntry,
) -> Result<EpiphanyCultMeshCoordinatorRunReceiptEntry> {
    validate_coordinator_run_receipt(&receipt)?;
    let mut node = open_epiphany_cultmesh_node(&store_path, receipt.runtime_id.clone())?;
    let receipt_key = epiphany_cultmesh_coordinator_run_receipt_key(&receipt.receipt_id);
    let written = node.put(receipt_key.as_str(), &receipt)?;
    let current_latest = node.get::<EpiphanyCultMeshCoordinatorRunReceiptEntry>(
        EPIPHANY_CULTMESH_COORDINATOR_RUN_RECEIPT_LATEST_KEY,
    )?;
    if current_latest.as_ref().is_none_or(|current| {
        coordinator_run_receipt_event_key(&written) >= coordinator_run_receipt_event_key(current)
    }) {
        node.put(
            EPIPHANY_CULTMESH_COORDINATOR_RUN_RECEIPT_LATEST_KEY,
            &written,
        )?;
    }
    node.flush()?;
    Ok(written)
}

pub fn load_latest_epiphany_cultmesh_coordinator_run_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshCoordinatorRunReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_COORDINATOR_RUN_RECEIPT_LATEST_KEY)
}

pub fn epiphany_cultmesh_hands_action_gate_from_summary_json(
    runtime_id: impl Into<String>,
    created_at_utc: impl Into<String>,
    source_summary_path: impl Into<String>,
    summary_json: &Value,
) -> Result<Option<EpiphanyCultMeshHandsActionGateEntry>> {
    let Some(gate_json) = summary_json.pointer("/finalAction/handsActionGate") else {
        return Ok(None);
    };
    let hands_intent_id = pointer_text(gate_json, "/intentId", "");
    let hands_review_id = pointer_text(gate_json, "/reviewId", "");
    let gate = EpiphanyCultMeshHandsActionGateEntry {
        schema_version: EPIPHANY_CULTMESH_HANDS_ACTION_GATE_SCHEMA_VERSION.to_string(),
        runtime_id: runtime_id.into(),
        verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
        gate_id: format!("{hands_intent_id}:{hands_review_id}"),
        source_coordinator_receipt_id: pointer_text(
            summary_json,
            "/coordinatorRunReceipt/receiptId",
            "",
        ),
        source_summary_path: source_summary_path.into(),
        thread_id: pointer_text(summary_json, "/threadId", ""),
        mode: pointer_text(summary_json, "/mode", ""),
        status: pointer_text(gate_json, "/status", ""),
        hands_intent_id,
        hands_review_id,
        substrate_gate_grant_receipt_id: pointer_text(
            gate_json,
            "/substrateGateGrantReceiptId",
            "",
        ),
        runtime_job_id: pointer_text(gate_json, "/runtimeJobId", ""),
        requested_paths: pointer_string_array(gate_json, "/requestedPaths")?,
        required_receipts: pointer_string_array(gate_json, "/requiredReceipts")?,
        record_pass_executable: pointer_text(gate_json, "/recordPassCommand/executable", ""),
        record_pass_args: pointer_string_array(gate_json, "/recordPassCommand/args")?,
        created_at_utc: created_at_utc.into(),
        private_state_exposed: false,
        notes: vec![
            "CultMesh mirror of the coordinator Hands action gate; runtime-spine Hands/Substrate Gate receipts remain the action owners.".to_string(),
            "The record-pass command is an operator hint only; this mirror cannot approve, execute, or mutate the repo.".to_string(),
        ],
    };
    validate_hands_action_gate(&gate)?;
    Ok(Some(gate))
}

pub fn write_epiphany_cultmesh_hands_action_gate(
    store_path: impl AsRef<Path>,
    gate: EpiphanyCultMeshHandsActionGateEntry,
) -> Result<EpiphanyCultMeshHandsActionGateEntry> {
    validate_hands_action_gate(&gate)?;
    let mut node = open_epiphany_cultmesh_node(&store_path, gate.runtime_id.clone())?;
    let gate_key = epiphany_cultmesh_hands_action_gate_key(&gate.gate_id);
    let written = node.put(gate_key.as_str(), &gate)?;
    let current_latest = node.get::<EpiphanyCultMeshHandsActionGateEntry>(
        EPIPHANY_CULTMESH_HANDS_ACTION_GATE_LATEST_KEY,
    )?;
    if current_latest.as_ref().is_none_or(|current| {
        hands_action_gate_event_key(&written) >= hands_action_gate_event_key(current)
    }) {
        node.put(EPIPHANY_CULTMESH_HANDS_ACTION_GATE_LATEST_KEY, &written)?;
    }
    node.flush()?;
    Ok(written)
}

pub fn load_latest_epiphany_cultmesh_hands_action_gate(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshHandsActionGateEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_HANDS_ACTION_GATE_LATEST_KEY)
}

pub fn epiphany_cultmesh_role_review_event_from_summary_json(
    runtime_id: impl Into<String>,
    created_at_utc: impl Into<String>,
    source_summary_path: impl Into<String>,
    summary_json: &Value,
) -> Result<Option<EpiphanyCultMeshRoleReviewEventEntry>> {
    let Some(event_json) = latest_role_review_event_json(summary_json) else {
        return Ok(None);
    };
    let surface = pointer_text(event_json, "/type", "");
    let role_id = pointer_text(event_json, "/roleId", "");
    let acceptance = event_json
        .pointer("/accepted/receipt")
        .or_else(|| event_json.pointer("/accepted/acceptanceReceipt"))
        .or_else(|| event_json.pointer("/superseded/patch/acceptanceReceipts/0"))
        .or_else(|| event_json.pointer("/superseded/update/acceptanceReceipts/0"))
        .or_else(|| event_json.pointer("/superseded/acceptanceReceipts/0"));
    let review_status = acceptance
        .map(|value| pointer_text(value, "/status", ""))
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            if surface == "roleFailureReview" {
                "superseded".to_string()
            } else {
                "accepted".to_string()
            }
        });
    let acceptance_receipt_id = acceptance
        .map(|value| pointer_text(value, "/id", ""))
        .unwrap_or_default();
    let runtime_result_id = acceptance
        .map(|value| pointer_text(value, "/result_id", ""))
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            acceptance
                .map(|value| pointer_text(value, "/resultId", ""))
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_default();
    let runtime_job_id = acceptance
        .map(|value| pointer_text(value, "/job_id", ""))
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            acceptance
                .map(|value| pointer_text(value, "/jobId", ""))
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_default();
    let binding_id = acceptance
        .map(|value| pointer_text(value, "/binding_id", ""))
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            acceptance
                .map(|value| pointer_text(value, "/bindingId", ""))
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_default();
    let summary = acceptance
        .map(|value| pointer_text(value, "/summary", ""))
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            event_json
                .pointer("/accepted/note")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .or_else(|| {
            event_json
                .pointer("/superseded/note")
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .unwrap_or_default();
    let event = EpiphanyCultMeshRoleReviewEventEntry {
        schema_version: EPIPHANY_CULTMESH_ROLE_REVIEW_EVENT_SCHEMA_VERSION.to_string(),
        runtime_id: runtime_id.into(),
        verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
        event_id: format!("{surface}:{role_id}:{review_status}"),
        source_coordinator_receipt_id: pointer_text(
            summary_json,
            "/coordinatorRunReceipt/receiptId",
            "",
        ),
        source_summary_path: source_summary_path.into(),
        thread_id: pointer_text(summary_json, "/threadId", ""),
        mode: pointer_text(summary_json, "/mode", ""),
        surface,
        role_id,
        review_status,
        acceptance_receipt_id,
        runtime_result_id,
        runtime_job_id,
        binding_id,
        summary,
        created_at_utc: created_at_utc.into(),
        private_state_exposed: false,
        notes: vec![
            "CultMesh mirror of the latest coordinator role review event; thread-state acceptance receipts remain the review owner.".to_string(),
            "This mirror is for operator discovery/readback only and cannot accept, supersede, or relaunch a lane.".to_string(),
        ],
    };
    validate_role_review_event(&event)?;
    Ok(Some(event))
}

pub fn write_epiphany_cultmesh_role_review_event(
    store_path: impl AsRef<Path>,
    event: EpiphanyCultMeshRoleReviewEventEntry,
) -> Result<EpiphanyCultMeshRoleReviewEventEntry> {
    validate_role_review_event(&event)?;
    let mut node = open_epiphany_cultmesh_node(&store_path, event.runtime_id.clone())?;
    let event_key = epiphany_cultmesh_role_review_event_key(&event.event_id);
    let written = node.put(event_key.as_str(), &event)?;
    let current_latest = node.get::<EpiphanyCultMeshRoleReviewEventEntry>(
        EPIPHANY_CULTMESH_ROLE_REVIEW_EVENT_LATEST_KEY,
    )?;
    if current_latest
        .as_ref()
        .is_none_or(|current| role_review_event_key(&written) >= role_review_event_key(current))
    {
        node.put(EPIPHANY_CULTMESH_ROLE_REVIEW_EVENT_LATEST_KEY, &written)?;
    }
    node.flush()?;
    Ok(written)
}

pub fn load_latest_epiphany_cultmesh_role_review_event(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshRoleReviewEventEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_ROLE_REVIEW_EVENT_LATEST_KEY)
}

fn validate_coordinator_run_receipt(
    receipt: &EpiphanyCultMeshCoordinatorRunReceiptEntry,
) -> Result<()> {
    if receipt.private_state_exposed {
        return Err(anyhow!(
            "coordinator run CultMesh receipts must not expose private state"
        ));
    }
    for (label, value) in [
        ("receipt id", receipt.receipt_id.as_str()),
        ("source receipt id", receipt.source_receipt_id.as_str()),
        ("source store", receipt.source_store.as_str()),
        ("thread id", receipt.thread_id.as_str()),
        ("mode", receipt.mode.as_str()),
        ("final action", receipt.final_action.as_str()),
        ("created at", receipt.created_at_utc.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("coordinator run CultMesh receipt missing {label}"));
        }
    }
    if receipt
        .sealed_artifact_refs
        .iter()
        .any(|path| path.trim().is_empty())
    {
        return Err(anyhow!(
            "coordinator run CultMesh receipt has an empty sealed artifact ref"
        ));
    }
    DateTime::parse_from_rfc3339(&receipt.created_at_utc).map_err(|error| {
        anyhow!("coordinator run CultMesh receipt has invalid created at: {error}")
    })?;
    Ok(())
}

fn coordinator_run_receipt_event_key(
    receipt: &EpiphanyCultMeshCoordinatorRunReceiptEntry,
) -> (DateTime<FixedOffset>, &str) {
    (
        DateTime::parse_from_rfc3339(&receipt.created_at_utc)
            .expect("validated coordinator run receipt timestamp"),
        receipt.receipt_id.as_str(),
    )
}

fn validate_hands_action_gate(gate: &EpiphanyCultMeshHandsActionGateEntry) -> Result<()> {
    if gate.private_state_exposed {
        return Err(anyhow!(
            "Hands action gate CultMesh mirrors must not expose private state"
        ));
    }
    for (label, value) in [
        ("gate id", gate.gate_id.as_str()),
        (
            "source coordinator receipt id",
            gate.source_coordinator_receipt_id.as_str(),
        ),
        ("source summary path", gate.source_summary_path.as_str()),
        ("thread id", gate.thread_id.as_str()),
        ("mode", gate.mode.as_str()),
        ("status", gate.status.as_str()),
        ("Hands intent id", gate.hands_intent_id.as_str()),
        ("Hands review id", gate.hands_review_id.as_str()),
        (
            "Substrate Gate grant receipt id",
            gate.substrate_gate_grant_receipt_id.as_str(),
        ),
        ("created at", gate.created_at_utc.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("Hands action gate CultMesh mirror missing {label}"));
        }
    }
    if gate.required_receipts.is_empty() {
        return Err(anyhow!(
            "Hands action gate CultMesh mirror missing required receipts"
        ));
    }
    if gate
        .requested_paths
        .iter()
        .any(|path| path.trim().is_empty())
        || gate
            .required_receipts
            .iter()
            .any(|receipt| receipt.trim().is_empty())
        || gate
            .record_pass_args
            .iter()
            .any(|arg| arg.trim().is_empty())
    {
        return Err(anyhow!(
            "Hands action gate CultMesh mirror has an empty path, receipt, or command argument"
        ));
    }
    DateTime::parse_from_rfc3339(&gate.created_at_utc).map_err(|error| {
        anyhow!("Hands action gate CultMesh mirror has invalid created at: {error}")
    })?;
    Ok(())
}

fn hands_action_gate_event_key(
    gate: &EpiphanyCultMeshHandsActionGateEntry,
) -> (DateTime<FixedOffset>, &str) {
    (
        DateTime::parse_from_rfc3339(&gate.created_at_utc)
            .expect("validated Hands action gate timestamp"),
        gate.gate_id.as_str(),
    )
}

fn validate_role_review_event(event: &EpiphanyCultMeshRoleReviewEventEntry) -> Result<()> {
    if event.private_state_exposed {
        return Err(anyhow!(
            "role review CultMesh mirrors must not expose private state"
        ));
    }
    if !matches!(event.surface.as_str(), "roleAccept" | "roleFailureReview") {
        return Err(anyhow!(
            "role review CultMesh mirror has unsupported surface {:?}",
            event.surface
        ));
    }
    for (label, value) in [
        ("event id", event.event_id.as_str()),
        (
            "source coordinator receipt id",
            event.source_coordinator_receipt_id.as_str(),
        ),
        ("source summary path", event.source_summary_path.as_str()),
        ("thread id", event.thread_id.as_str()),
        ("mode", event.mode.as_str()),
        ("role id", event.role_id.as_str()),
        ("review status", event.review_status.as_str()),
        ("created at", event.created_at_utc.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("role review CultMesh mirror missing {label}"));
        }
    }
    DateTime::parse_from_rfc3339(&event.created_at_utc)
        .map_err(|error| anyhow!("role review CultMesh mirror has invalid created at: {error}"))?;
    Ok(())
}

fn role_review_event_key(
    event: &EpiphanyCultMeshRoleReviewEventEntry,
) -> (DateTime<FixedOffset>, &str) {
    (
        DateTime::parse_from_rfc3339(&event.created_at_utc)
            .expect("validated role review event timestamp"),
        event.event_id.as_str(),
    )
}

fn latest_role_review_event_json(summary_json: &Value) -> Option<&Value> {
    let steps = summary_json.pointer("/steps")?.as_array()?;
    steps.iter().rev().find_map(|step| {
        step.pointer("/events")
            .and_then(Value::as_array)?
            .iter()
            .rev()
            .find(|event| {
                matches!(
                    event.pointer("/type").and_then(Value::as_str),
                    Some("roleAccept" | "roleFailureReview")
                )
            })
    })
}

pub fn epiphany_cultmesh_eve_connection_intent_from_advertisement(
    intent_id: impl Into<String>,
    source_cluster_id: impl Into<String>,
    target: &EpiphanyCultMeshOdinAdvertisementEntry,
    collaboration_topic: impl Into<String>,
    requested_action: impl Into<String>,
) -> EpiphanyCultMeshEveConnectionIntentEntry {
    EpiphanyCultMeshEveConnectionIntentEntry {
        schema_version: EPIPHANY_CULTMESH_EVE_CONNECTION_INTENT_SCHEMA_VERSION.to_string(),
        intent_id: intent_id.into(),
        source_cluster_id: source_cluster_id.into(),
        target_advertisement_id: target.advertisement_id.clone(),
        target_cluster_id: target.cluster_id.clone(),
        target_eve_surface_id: target.eve_surface_id.clone(),
        collaboration_topic: collaboration_topic.into(),
        requested_action: requested_action.into(),
        feedback_route: "imagination.consensus_discovery".to_string(),
        requested_document_types: vec![
            EPIPHANY_CULTMESH_ODIN_ADVERTISEMENT_TYPE.to_string(),
            EPIPHANY_CULTMESH_BIFROST_COLLABORATION_FEEDBACK_TYPE.to_string(),
        ],
        private_state_requested: false,
        notes: vec![
            "Eve connection intent is a collaboration request over advertised metadata, not a private Verse read.".to_string(),
            "Persona or peer feedback from this request routes to Imagination consensus discovery before adoption.".to_string(),
            "Mind and Substrate Gate still review durable state mutation and repo access.".to_string(),
        ],
    }
}

#[cfg(test)]
pub fn epiphany_cultmesh_eve_connection_receipt_for_intent(
    receipt_id: impl Into<String>,
    intent: &EpiphanyCultMeshEveConnectionIntentEntry,
    status: impl Into<String>,
) -> EpiphanyCultMeshEveConnectionReceiptEntry {
    EpiphanyCultMeshEveConnectionReceiptEntry {
        schema_version: EPIPHANY_CULTMESH_EVE_CONNECTION_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: receipt_id.into(),
        intent_id: intent.intent_id.clone(),
        source_cluster_id: intent.source_cluster_id.clone(),
        target_cluster_id: intent.target_cluster_id.clone(),
        target_eve_surface_id: intent.target_eve_surface_id.clone(),
        status: status.into(),
        feedback_route: intent.feedback_route.clone(),
        private_state_exposed: false,
        notes: vec![
            "Receipt records an Eve collaboration request over CultMesh; it does not grant private state authority.".to_string(),
            "Feedback remains routed through Imagination consensus discovery and later Mind/Bifrost review gates.".to_string(),
        ],
    }
}

pub fn write_epiphany_cultmesh_eve_connection_intent(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    intent: EpiphanyCultMeshEveConnectionIntentEntry,
) -> Result<EpiphanyCultMeshEveConnectionIntentEntry> {
    if intent.private_state_requested {
        return Err(anyhow!(
            "Eve connection intents must not request private Verse state"
        ));
    }
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let intent_key = epiphany_cultmesh_eve_connection_intent_key(&intent.intent_id);
    let written = node.put(intent_key.as_str(), &intent)?;
    node.put(EPIPHANY_CULTMESH_EVE_CONNECTION_INTENT_LATEST_KEY, &written)?;
    node.flush()?;
    Ok(written)
}

#[cfg(test)]
pub fn write_epiphany_cultmesh_eve_connection_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    receipt: EpiphanyCultMeshEveConnectionReceiptEntry,
) -> Result<EpiphanyCultMeshEveConnectionReceiptEntry> {
    if receipt.private_state_exposed {
        return Err(anyhow!(
            "Eve connection receipts must not expose private state"
        ));
    }
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let receipt_key = epiphany_cultmesh_eve_connection_receipt_key(&receipt.receipt_id);
    let written = node.put(receipt_key.as_str(), &receipt)?;
    node.put(
        EPIPHANY_CULTMESH_EVE_CONNECTION_RECEIPT_LATEST_KEY,
        &written,
    )?;
    node.flush()?;
    Ok(written)
}

pub fn load_latest_epiphany_cultmesh_eve_connection_intent(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshEveConnectionIntentEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_EVE_CONNECTION_INTENT_LATEST_KEY)
}

pub fn load_latest_epiphany_cultmesh_eve_connection_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshEveConnectionReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_EVE_CONNECTION_RECEIPT_LATEST_KEY)
}

pub fn epiphany_cultmesh_daemon_poke_intent_from_status(
    intent_id: impl Into<String>,
    requesting_agent_id: impl Into<String>,
    status: &EpiphanyCultMeshDaemonStatusEntry,
    reason: impl Into<String>,
) -> EpiphanyCultMeshDaemonPokeIntentEntry {
    let requested_at_utc = Utc::now().to_rfc3339();
    EpiphanyCultMeshDaemonPokeIntentEntry {
        schema_version: EPIPHANY_CULTMESH_DAEMON_POKE_INTENT_SCHEMA_VERSION.to_string(),
        intent_id: intent_id.into(),
        requesting_agent_id: requesting_agent_id.into(),
        target_daemon_id: status.daemon_id.clone(),
        target_cluster_id: status.cluster_id.clone(),
        daemon_surface_id: status.daemon_surface_id.clone(),
        eve_surface_id: status.eve_surface_id.clone(),
        reason: reason.into(),
        requested_action: "pokeDaemon".to_string(),
        observed_status: status.status.clone(),
        private_state_requested: false,
        notes: vec![
            "Daemon poke intent is an operator-safe lifecycle action request, not a private Verse inspection.".to_string(),
            "The target daemon owns the resulting status; this intent only records the requested poke.".to_string(),
        ],
        observed_last_heartbeat_utc: status.last_heartbeat_utc.clone(),
        requested_at_utc,
    }
}

pub fn epiphany_cultmesh_daemon_poke_receipt_for_intent(
    receipt_id: impl Into<String>,
    intent: &EpiphanyCultMeshDaemonPokeIntentEntry,
    status: impl Into<String>,
    resulting_status: impl Into<String>,
    operator_artifact_ref: impl Into<String>,
) -> EpiphanyCultMeshDaemonPokeReceiptEntry {
    let completed_at_utc = Utc::now().to_rfc3339();
    EpiphanyCultMeshDaemonPokeReceiptEntry {
        schema_version: EPIPHANY_CULTMESH_DAEMON_POKE_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: receipt_id.into(),
        intent_id: intent.intent_id.clone(),
        target_daemon_id: intent.target_daemon_id.clone(),
        target_cluster_id: intent.target_cluster_id.clone(),
        action_taken: intent.requested_action.clone(),
        status: status.into(),
        resulting_status: resulting_status.into(),
        operator_artifact_ref: operator_artifact_ref.into(),
        private_state_exposed: false,
        notes: vec![
            "Daemon poke receipt records lifecycle intervention proof without exposing private daemon state.".to_string(),
            "Follow-up daemon status documents remain the liveness authority.".to_string(),
        ],
        attempted_at_utc: intent.requested_at_utc.clone(),
        completed_at_utc,
    }
}

pub fn write_epiphany_cultmesh_daemon_poke_intent(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    intent: EpiphanyCultMeshDaemonPokeIntentEntry,
) -> Result<EpiphanyCultMeshDaemonPokeIntentEntry> {
    validate_daemon_poke_intent(&intent)?;
    let store_path = store_path.as_ref();
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let intent_key = epiphany_cultmesh_daemon_poke_intent_key(&intent.intent_id);
    put_immutable_cultmesh_entry_and_advance_latest(
        &node,
        store_path,
        &intent_key,
        EPIPHANY_CULTMESH_DAEMON_POKE_INTENT_LATEST_KEY,
        &intent,
        |entry| &entry.requested_at_utc,
    )
}

pub fn write_epiphany_cultmesh_daemon_poke_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    receipt: EpiphanyCultMeshDaemonPokeReceiptEntry,
) -> Result<EpiphanyCultMeshDaemonPokeReceiptEntry> {
    validate_daemon_poke_receipt(&receipt)?;
    let store_path = store_path.as_ref();
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let receipt_key = epiphany_cultmesh_daemon_poke_receipt_key(&receipt.receipt_id);
    put_immutable_cultmesh_entry_and_advance_latest(
        &node,
        store_path,
        &receipt_key,
        EPIPHANY_CULTMESH_DAEMON_POKE_RECEIPT_LATEST_KEY,
        &receipt,
        |entry| &entry.completed_at_utc,
    )
}

fn put_immutable_cultmesh_entry_and_advance_latest<T, F>(
    node: &CultMeshNode,
    store_path: &Path,
    identity_key: &str,
    latest_key: &str,
    value: &T,
    event_time: F,
) -> Result<T>
where
    T: DatabaseEntry + Clone + PartialEq,
    F: Fn(&T) -> &str,
{
    let existing = node.get::<T>(identity_key)?;
    if existing.as_ref().is_some_and(|current| current != value) {
        return Err(anyhow!(
            "immutable CultMesh identity collision for type {:?} key {:?}",
            T::TYPE,
            identity_key
        ));
    }
    let candidate_time = DateTime::parse_from_rfc3339(event_time(value))
        .context("immutable CultMesh event requires RFC3339 ordering time")?;
    let latest = node.get::<T>(latest_key)?;
    let advances_latest = match latest.as_ref() {
        Some(current) => {
            candidate_time
                > DateTime::parse_from_rfc3339(event_time(current))
                    .context("persisted immutable CultMesh event has invalid ordering time")?
        }
        None => true,
    };
    if existing.is_some() && !advances_latest {
        return Ok(existing.expect("existing checked"));
    }

    let mut expected = Vec::new();
    if advances_latest && let Some(envelope) = node.cache().get_envelope::<T>(latest_key)? {
        expected.push(envelope);
    }
    let mut replacements = Vec::new();
    if existing.is_none() {
        replacements.push(node.cache().prepare_entry(identity_key, value)?.0);
    }
    if advances_latest {
        replacements.push(node.cache().prepare_entry(latest_key, value)?.0);
    }
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    if backing.compare_and_swap_batch(&expected, replacements)? {
        return Ok(value.clone());
    }
    let refreshed = open_epiphany_cultmesh_node(store_path, node.runtime_id())?;
    match refreshed.get::<T>(identity_key)? {
        Some(current)
            if current == *value
                && refreshed
                    .get::<T>(latest_key)?
                    .as_ref()
                    .is_some_and(|latest| {
                        DateTime::parse_from_rfc3339(event_time(latest)).ok()
                            >= Some(candidate_time)
                    }) =>
        {
            Ok(current)
        }
        Some(_) => Err(anyhow!(
            "immutable CultMesh identity collision for type {:?} key {:?}",
            T::TYPE,
            identity_key
        )),
        None => Err(anyhow!(
            "immutable CultMesh write lost compare-and-swap for type {:?} key {:?}",
            T::TYPE,
            identity_key
        )),
    }
}

pub fn load_latest_epiphany_cultmesh_daemon_poke_intent(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshDaemonPokeIntentEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_DAEMON_POKE_INTENT_LATEST_KEY)
}

pub fn load_epiphany_cultmesh_daemon_poke_intent(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    intent_id: &str,
) -> Result<Option<EpiphanyCultMeshDaemonPokeIntentEntry>> {
    if intent_id.trim().is_empty() {
        return Err(anyhow!("daemon poke intent identity is required"));
    }
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(&epiphany_cultmesh_daemon_poke_intent_key(intent_id))
}

pub fn load_latest_epiphany_cultmesh_daemon_poke_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshDaemonPokeReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_DAEMON_POKE_RECEIPT_LATEST_KEY)
}

pub fn load_epiphany_cultmesh_daemon_poke_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    receipt_id: &str,
) -> Result<Option<EpiphanyCultMeshDaemonPokeReceiptEntry>> {
    if receipt_id.trim().is_empty() {
        return Err(anyhow!("daemon poke receipt identity is required"));
    }
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(&epiphany_cultmesh_daemon_poke_receipt_key(receipt_id))
}

pub fn write_epiphany_cultmesh_daemon_restart_policy(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    policy: EpiphanyCultMeshDaemonRestartPolicyEntry,
) -> Result<EpiphanyCultMeshDaemonRestartPolicyEntry> {
    validate_daemon_restart_policy(&policy)?;
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let key = epiphany_cultmesh_daemon_restart_policy_key(&policy.daemon_id);
    let written = node.put(key.as_str(), &policy)?;
    node.flush()?;
    Ok(written)
}

pub fn load_epiphany_cultmesh_daemon_restart_policy(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    daemon_id: &str,
) -> Result<Option<EpiphanyCultMeshDaemonRestartPolicyEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let key = epiphany_cultmesh_daemon_restart_policy_key(daemon_id);
    node.get(key.as_str())
}

pub fn write_epiphany_cultmesh_daemon_scheduler_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    receipt: EpiphanyCultMeshDaemonSchedulerReceiptEntry,
) -> Result<EpiphanyCultMeshDaemonSchedulerReceiptEntry> {
    validate_daemon_scheduler_receipt(&receipt)?;
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let receipt_key = epiphany_cultmesh_daemon_scheduler_receipt_key(&receipt.receipt_id);
    let written = node.put(receipt_key.as_str(), &receipt)?;
    let current_latest = node.get::<EpiphanyCultMeshDaemonSchedulerReceiptEntry>(
        EPIPHANY_CULTMESH_DAEMON_SCHEDULER_RECEIPT_LATEST_KEY,
    )?;
    if current_latest.as_ref().is_none_or(|current| {
        daemon_scheduler_event_key(&written) >= daemon_scheduler_event_key(current)
    }) {
        node.put(
            EPIPHANY_CULTMESH_DAEMON_SCHEDULER_RECEIPT_LATEST_KEY,
            &written,
        )?;
    }
    node.flush()?;
    Ok(written)
}

pub fn load_latest_epiphany_cultmesh_daemon_scheduler_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshDaemonSchedulerReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_DAEMON_SCHEDULER_RECEIPT_LATEST_KEY)
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
    let current_latest = node.get::<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry>(
        EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_LATEST_KEY,
    )?;
    if current_latest.as_ref().is_none_or(|current| {
        daemon_service_lifecycle_event_key(&written) >= daemon_service_lifecycle_event_key(current)
    }) {
        if let Some(envelope) = node
            .cache()
            .get_envelope::<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry>(
                EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_LATEST_KEY,
            )?
        {
            expected.push(envelope);
        }
        replacements.push(
            node.cache()
                .prepare_entry(
                    EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_LATEST_KEY,
                    &written,
                )?
                .0,
        );
    }
    let service_latest_key =
        epiphany_cultmesh_daemon_service_lifecycle_receipt_latest_key(&written.service_id);
    let current_service_latest =
        node.get::<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry>(&service_latest_key)?;
    if current_service_latest.as_ref().is_none_or(|current| {
        daemon_service_lifecycle_event_key(&written) >= daemon_service_lifecycle_event_key(current)
    }) {
        if let Some(envelope) = node
            .cache()
            .get_envelope::<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry>(
                &service_latest_key,
            )?
        {
            expected.push(envelope);
        }
        replacements.push(node.cache().prepare_entry(&service_latest_key, &written)?.0);
    }
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
            "daemon service lifecycle receipt CAS lost concurrent latest-state race"
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

pub fn load_latest_epiphany_cultmesh_daemon_service_lifecycle_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_LATEST_KEY)
}

pub fn load_latest_epiphany_cultmesh_daemon_service_lifecycle_receipt_for_service(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    service_id: &str,
) -> Result<Option<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(&epiphany_cultmesh_daemon_service_lifecycle_receipt_latest_key(service_id))
}

pub fn load_epiphany_cultmesh_daemon_service_lifecycle_receipts(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    Ok(node
        .get_all_with_keys::<EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry>()?
        .into_iter()
        .filter(|(key, _)| {
            key != EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_LATEST_KEY
                && !key.starts_with("epiphany-local/daemon-service-lifecycle-receipt/latest/")
        })
        .map(|(_, receipt)| receipt)
        .collect())
}

pub fn write_epiphany_cultmesh_managed_service_policy(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    policy: EpiphanyCultMeshManagedServicePolicyEntry,
) -> Result<EpiphanyCultMeshManagedServicePolicyEntry> {
    if policy.service_id == EPIPHANY_SEMANTIC_PROJECTOR_SERVICE_ID {
        return Err(anyhow!(
            "reserved semantic projector policy requires its specialized writer"
        ));
    }
    if policy.service_id == EPIPHANY_WORKSPACE_COVERAGE_PROJECTOR_SERVICE_ID {
        return Err(anyhow!(
            "reserved workspace coverage projector policy requires its specialized writer"
        ));
    }
    validate_managed_service_policy(&policy)?;
    write_validated_managed_service_policy(store_path, runtime_id, policy)
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
    validate_semantic_projector_managed_service_policy(&policy)?;
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
    validate_workspace_coverage_projector_managed_service_policy(&policy)?;
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
    node.get(&epiphany_cultmesh_managed_service_policy_key(service_id))
}

pub fn load_epiphany_cultmesh_managed_service_policies(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshManagedServicePolicyEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    Ok(node
        .get_all_with_keys::<EpiphanyCultMeshManagedServicePolicyEntry>()?
        .into_iter()
        .map(|(_, policy)| policy)
        .collect())
}

pub fn load_epiphany_cultmesh_idunn_deployment_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    receipt_ref: impl AsRef<str>,
) -> Result<Option<EpiphanyCultMeshIdunnDeploymentReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let key = epiphany_cultmesh_idunn_deployment_receipt_ref_key(receipt_ref.as_ref());
    node.get(key.as_str())
}

pub fn load_latest_epiphany_cultmesh_idunn_deployment_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshIdunnDeploymentReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_IDUNN_DEPLOYMENT_RECEIPT_LATEST_KEY)
}

pub fn load_epiphany_cultmesh_idunn_aftercare_audit_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    receipt_ref: impl AsRef<str>,
) -> Result<Option<EpiphanyCultMeshIdunnAftercareAuditReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let key = epiphany_cultmesh_idunn_aftercare_audit_receipt_ref_key(receipt_ref.as_ref());
    node.get(key.as_str())
}

pub fn load_latest_epiphany_cultmesh_idunn_aftercare_audit_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshIdunnAftercareAuditReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_IDUNN_AFTERCARE_AUDIT_RECEIPT_LATEST_KEY)
}

pub fn default_epiphany_cultmesh_swarm_brake(
    generated_at_utc: impl Into<String>,
) -> EpiphanyCultMeshSwarmBrakeEntry {
    EpiphanyCultMeshSwarmBrakeEntry {
        schema_version: EPIPHANY_CULTMESH_SWARM_BRAKE_SCHEMA_VERSION.to_string(),
        brake_id: "epiphany-local/swarm-brake/default".to_string(),
        status: "released".to_string(),
        scope: "swarm".to_string(),
        reason: "No swarm brake is engaged; unattended automation still requires typed scheduler, cooldown, recovery, and operator receipt gates.".to_string(),
        operator_agent_id: "epiphany.Self".to_string(),
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
        ],
        created_at_utc: generated_at_utc.into(),
        expires_at_utc: None,
        private_state_exposed: false,
        notes: vec![
            "The swarm brake is the operator-safe pause surface for live-fire readiness.".to_string(),
            "It may stop scheduling and daemon pokes, but it must not expose worker thoughts or private Verse state.".to_string(),
            "Engaged brakes require a scoped reason so silence cannot masquerade as consent.".to_string(),
        ],
    }
}

pub fn write_epiphany_cultmesh_swarm_brake(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    brake: EpiphanyCultMeshSwarmBrakeEntry,
) -> Result<EpiphanyCultMeshSwarmBrakeEntry> {
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
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_SWARM_BRAKE_KEY)
}

pub fn write_epiphany_cultmesh_persona_speech_audit(
    store_path: impl AsRef<Path>,
    audit: EpiphanyCultMeshPersonaSpeechAuditEntry,
) -> Result<EpiphanyCultMeshPersonaSpeechAuditEntry> {
    validate_persona_speech_audit(&audit)?;
    let mut node = open_epiphany_cultmesh_node(&store_path, audit.runtime_id.clone())?;
    let audit_key = epiphany_cultmesh_persona_speech_audit_key(&audit.audit_id);
    let written = node.put(audit_key.as_str(), &audit)?;
    node.put(EPIPHANY_CULTMESH_PERSONA_SPEECH_AUDIT_LATEST_KEY, &written)?;
    node.flush()?;
    Ok(written)
}

pub fn load_latest_epiphany_cultmesh_persona_speech_audit(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshPersonaSpeechAuditEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_PERSONA_SPEECH_AUDIT_LATEST_KEY)
}

pub fn validate_persona_speech_audit(
    audit: &EpiphanyCultMeshPersonaSpeechAuditEntry,
) -> Result<()> {
    if audit.private_state_exposed {
        return Err(anyhow!(
            "Persona speech audit must not expose private state"
        ));
    }
    if audit.audit_id.trim().is_empty()
        || audit.runtime_id.trim().is_empty()
        || audit.persona_agent_id.trim().is_empty()
    {
        return Err(anyhow!(
            "Persona speech audit requires audit, runtime, and persona ids"
        ));
    }
    if !matches!(audit.action_kind.as_str(), "draft" | "bubble" | "post") {
        return Err(anyhow!(
            "Persona speech audit action_kind must be draft, bubble, or post"
        ));
    }
    if !matches!(audit.decision.as_str(), "eligible" | "blocked") {
        return Err(anyhow!(
            "Persona speech audit decision must be eligible or blocked"
        ));
    }
    if audit.decision == "blocked" && audit.reasons.is_empty() {
        return Err(anyhow!("blocked Persona speech audit requires reasons"));
    }
    if audit.content_fingerprint.trim().is_empty() || audit.created_at_utc.trim().is_empty() {
        return Err(anyhow!(
            "Persona speech audit requires fingerprint and timestamp"
        ));
    }
    Ok(())
}

pub fn write_epiphany_cultmesh_weksa_lowering_receipt(
    store_path: impl AsRef<Path>,
    receipt: EpiphanyCultMeshWeksaLoweringReceiptEntry,
) -> Result<EpiphanyCultMeshWeksaLoweringReceiptEntry> {
    validate_weksa_lowering_receipt(&receipt)?;
    let mut node = open_epiphany_cultmesh_node(&store_path, receipt.runtime_id.clone())?;
    let receipt_key = epiphany_cultmesh_weksa_lowering_receipt_key(&receipt.receipt_id);
    let written = node.put(receipt_key.as_str(), &receipt)?;
    node.put(
        EPIPHANY_CULTMESH_WEKSA_LOWERING_RECEIPT_LATEST_KEY,
        &written,
    )?;
    node.flush()?;
    Ok(written)
}

pub fn load_latest_epiphany_cultmesh_weksa_lowering_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshWeksaLoweringReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_WEKSA_LOWERING_RECEIPT_LATEST_KEY)
}

pub fn validate_weksa_lowering_receipt(
    receipt: &EpiphanyCultMeshWeksaLoweringReceiptEntry,
) -> Result<()> {
    if receipt.private_state_exposed {
        return Err(anyhow!(
            "Weksa lowering receipt must not expose private state"
        ));
    }
    if receipt.publication_authorized {
        return Err(anyhow!(
            "Weksa lowering receipt must not claim publication authority"
        ));
    }
    if receipt.receipt_id.trim().is_empty()
        || receipt.runtime_id.trim().is_empty()
        || receipt.packet_id.trim().is_empty()
        || receipt.request_id.trim().is_empty()
        || receipt.persona_agent_id.trim().is_empty()
    {
        return Err(anyhow!(
            "Weksa lowering receipt requires receipt, runtime, packet, request, and persona ids"
        ));
    }
    if receipt.target_language.trim().is_empty()
        || receipt.delivery_surface.trim().is_empty()
        || receipt.created_at_utc.trim().is_empty()
    {
        return Err(anyhow!(
            "Weksa lowering receipt requires target language, delivery surface, and timestamp"
        ));
    }
    if !receipt.transport_authority.contains("must publish")
        && receipt.transport_authority.trim() != "none"
    {
        return Err(anyhow!(
            "Weksa lowering receipt transport authority must remain none or defer publication"
        ));
    }
    Ok(())
}

fn validate_swarm_brake(brake: &EpiphanyCultMeshSwarmBrakeEntry) -> Result<()> {
    if brake.private_state_exposed {
        return Err(anyhow!("swarm brake must not expose private state"));
    }
    if brake.brake_id.trim().is_empty() || brake.scope.trim().is_empty() {
        return Err(anyhow!("swarm brake requires brake id and scope"));
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

fn validate_daemon_poke_intent(intent: &EpiphanyCultMeshDaemonPokeIntentEntry) -> Result<()> {
    if intent.private_state_requested {
        return Err(anyhow!(
            "daemon poke intents must not request private state"
        ));
    }
    if intent.target_daemon_id.trim().is_empty() || intent.target_cluster_id.trim().is_empty() {
        return Err(anyhow!(
            "daemon poke intents require daemon and cluster ids"
        ));
    }
    if intent.requested_action != "pokeDaemon" {
        return Err(anyhow!("daemon poke intents must request pokeDaemon"));
    }
    if intent.reason.trim().is_empty() {
        return Err(anyhow!("daemon poke intents require a reason"));
    }
    for (label, value) in [
        (
            "observed provider heartbeat",
            intent.observed_last_heartbeat_utc.as_str(),
        ),
        ("request timestamp", intent.requested_at_utc.as_str()),
    ] {
        DateTime::parse_from_rfc3339(value)
            .with_context(|| format!("daemon poke intent requires RFC3339 {label}"))?;
    }
    Ok(())
}

fn validate_daemon_poke_receipt(receipt: &EpiphanyCultMeshDaemonPokeReceiptEntry) -> Result<()> {
    if receipt.private_state_exposed {
        return Err(anyhow!(
            "daemon poke receipts must not expose private state"
        ));
    }
    if receipt.intent_id.trim().is_empty() || receipt.target_daemon_id.trim().is_empty() {
        return Err(anyhow!(
            "daemon poke receipts require intent and daemon ids"
        ));
    }
    if receipt.action_taken != "pokeDaemon" {
        return Err(anyhow!("daemon poke receipts must record pokeDaemon"));
    }
    if receipt.status.trim().is_empty() || receipt.resulting_status.trim().is_empty() {
        return Err(anyhow!("daemon poke receipts require status results"));
    }
    let attempted = DateTime::parse_from_rfc3339(&receipt.attempted_at_utc)
        .context("daemon poke receipt requires RFC3339 attempt timestamp")?;
    let completed = DateTime::parse_from_rfc3339(&receipt.completed_at_utc)
        .context("daemon poke receipt requires RFC3339 completion timestamp")?;
    if completed < attempted {
        return Err(anyhow!(
            "daemon poke receipt completion cannot precede its attempt"
        ));
    }
    Ok(())
}

fn validate_daemon_restart_policy(policy: &EpiphanyCultMeshDaemonRestartPolicyEntry) -> Result<()> {
    if policy.private_state_exposed {
        return Err(anyhow!(
            "daemon restart policies must not expose private state"
        ));
    }
    for (label, value) in [
        ("policy id", policy.policy_id.as_str()),
        ("daemon id", policy.daemon_id.as_str()),
        ("cluster id", policy.cluster_id.as_str()),
        ("restart command", policy.restart_command.as_str()),
        ("last result status", policy.last_result_status.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("daemon restart policy missing {label}"));
        }
    }
    if policy.cooldown_seconds < 0 {
        return Err(anyhow!(
            "daemon restart policy cooldown_seconds must be non-negative"
        ));
    }
    if policy.backoff_multiplier == 0 {
        return Err(anyhow!(
            "daemon restart policy backoff_multiplier must be positive"
        ));
    }
    if policy.reconcile_interval_seconds < 0 {
        return Err(anyhow!(
            "daemon restart policy reconcile_interval_seconds must be non-negative"
        ));
    }
    if policy.heartbeat_stale_seconds < 0 {
        return Err(anyhow!(
            "daemon restart policy heartbeat_stale_seconds must be non-negative"
        ));
    }
    Ok(())
}

fn validate_managed_service_policy(
    policy: &EpiphanyCultMeshManagedServicePolicyEntry,
) -> Result<()> {
    if policy.private_state_exposed {
        return Err(anyhow!(
            "managed service policies must not expose private state"
        ));
    }
    for (label, value) in [
        ("policy id", policy.policy_id.as_str()),
        ("service id", policy.service_id.as_str()),
        ("owner daemon id", policy.owner_daemon_id.as_str()),
        ("command", policy.command.as_str()),
        ("restart mode", policy.restart_mode.as_str()),
        ("stdout artifact", policy.stdout_artifact.as_str()),
        ("stderr artifact", policy.stderr_artifact.as_str()),
        ("updated timestamp", policy.updated_at_utc.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("managed service policy missing {label}"));
        }
    }
    if !matches!(
        policy.restart_mode.as_str(),
        "always" | "on-failure" | "never"
    ) {
        return Err(anyhow!(
            "managed service policy restart_mode must be always, on-failure, or never"
        ));
    }
    if policy.cooldown_seconds < 0 || policy.backoff_multiplier == 0 {
        return Err(anyhow!(
            "managed service policy requires non-negative cooldown and positive backoff"
        ));
    }
    Ok(())
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
        || policy.owner_daemon_id != "epiphany-daemon-supervisor"
        || policy.restart_mode != "always"
        || policy.args.len() != 17
        || policy.args[0] != "serve"
        || policy.args[1] != "--agent-store"
        || policy.args[2].trim().is_empty()
        || policy.args[3] != "--runtime-store"
        || policy.args[4].trim().is_empty()
        || policy.args[5] != "--local-verse-store"
        || policy.args[6].trim().is_empty()
        || policy.args[7] != "--runtime-id"
        || policy.args[8].trim().is_empty()
        || policy.args[9] != "--interval-seconds"
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
            "reserved semantic projector policy must bind one packaged process to both canonical stores"
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
        || policy.owner_daemon_id != "epiphany-daemon-supervisor"
        || !policy.enabled
        || policy.restart_mode != "always"
        || policy.args.len() != 15
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
        || policy.args[9] != "--qdrant-url"
        || policy.args[10].trim().is_empty()
        || policy.args[11] != "--ollama-base-url"
        || policy.args[12].trim().is_empty()
        || policy.args[13] != "--ollama-model"
        || policy.args[14].trim().is_empty()
    {
        return Err(anyhow!(
            "reserved workspace coverage projector policy must bind one packaged process to its authenticated runtime Body route"
        ));
    }
    Ok(())
}

fn validate_daemon_scheduler_receipt(
    receipt: &EpiphanyCultMeshDaemonSchedulerReceiptEntry,
) -> Result<()> {
    if receipt.private_state_exposed {
        return Err(anyhow!(
            "daemon scheduler receipts must not expose private state"
        ));
    }
    for (label, value) in [
        ("receipt id", receipt.receipt_id.as_str()),
        ("scheduler id", receipt.scheduler_id.as_str()),
        ("runtime id", receipt.runtime_id.as_str()),
        ("daemon selector", receipt.daemon_selector.as_str()),
        ("status", receipt.status.as_str()),
        ("tick started", receipt.tick_started_utc.as_str()),
        ("tick completed", receipt.tick_completed_utc.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("daemon scheduler receipt missing {label}"));
        }
    }
    let started_at = DateTime::parse_from_rfc3339(&receipt.tick_started_utc)
        .map_err(|error| anyhow!("daemon scheduler receipt has invalid tick start: {error}"))?;
    let completed_at =
        DateTime::parse_from_rfc3339(&receipt.tick_completed_utc).map_err(|error| {
            anyhow!("daemon scheduler receipt has invalid tick completion: {error}")
        })?;
    if completed_at < started_at {
        return Err(anyhow!(
            "daemon scheduler receipt tick completed before it started"
        ));
    }
    if let Some(next_wake) = receipt.next_wake_utc.as_deref() {
        DateTime::parse_from_rfc3339(next_wake)
            .map_err(|error| anyhow!("daemon scheduler receipt has invalid next wake: {error}"))?;
    }
    Ok(())
}

fn daemon_scheduler_event_key(
    receipt: &EpiphanyCultMeshDaemonSchedulerReceiptEntry,
) -> (DateTime<FixedOffset>, u64, &str) {
    (
        DateTime::parse_from_rfc3339(&receipt.tick_completed_utc)
            .expect("validated scheduler completion timestamp"),
        receipt.iteration,
        receipt.receipt_id.as_str(),
    )
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

fn daemon_service_lifecycle_event_key(
    receipt: &EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry,
) -> (DateTime<FixedOffset>, &str) {
    let event_time = receipt
        .completed_at_utc
        .as_deref()
        .map(DateTime::parse_from_rfc3339)
        .transpose()
        .expect("validated lifecycle completion timestamp")
        .unwrap_or_else(|| {
            DateTime::parse_from_rfc3339(&receipt.started_at_utc)
                .expect("validated lifecycle start timestamp")
        });
    (event_time, receipt.receipt_id.as_str())
}

pub fn epiphany_cultmesh_daemon_tool_invocation_intent_from_capability(
    intent_id: impl Into<String>,
    requesting_agent_id: impl Into<String>,
    requesting_cluster_id: impl Into<String>,
    capability: &EpiphanyCultMeshDaemonToolCapabilityEntry,
    payload_ref: impl Into<String>,
    payload_summary: impl Into<String>,
) -> EpiphanyCultMeshDaemonToolInvocationIntentEntry {
    EpiphanyCultMeshDaemonToolInvocationIntentEntry {
        schema_version: EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_INTENT_SCHEMA_VERSION.to_string(),
        intent_id: intent_id.into(),
        requesting_agent_id: requesting_agent_id.into(),
        requesting_cluster_id: requesting_cluster_id.into(),
        capability_id: capability.capability_id.clone(),
        host_cluster_id: capability.host_cluster_id.clone(),
        host_daemon_id: capability.host_daemon_id.clone(),
        eve_surface_id: capability.eve_surface_id.clone(),
        tool_name: capability.tool_name.clone(),
        operation: capability.operation.clone(),
        input_contract_type: capability.input_contract_type.clone(),
        payload_ref: payload_ref.into(),
        payload_summary: payload_summary.into(),
        authority_gate: capability.authority_gate.clone(),
        requires_receipt: capability.requires_receipt,
        private_state_requested: false,
        notes: vec![
            "Any local CultMesh agent may submit this daemon tool invocation intent when it cites an advertised capability.".to_string(),
            "The payload is referenced through the capability's typed input contract; this document is the routing envelope, not private state cargo.".to_string(),
            "Execution remains gated by the advertised authority and must produce a typed receipt.".to_string(),
        ],
    }
}

#[cfg(test)]
pub fn epiphany_cultmesh_daemon_tool_invocation_receipt_for_intent(
    receipt_id: impl Into<String>,
    intent: &EpiphanyCultMeshDaemonToolInvocationIntentEntry,
    status: impl Into<String>,
    receipt_contract_type: impl Into<String>,
    result_ref: impl Into<String>,
    result_summary: impl Into<String>,
) -> EpiphanyCultMeshDaemonToolInvocationReceiptEntry {
    EpiphanyCultMeshDaemonToolInvocationReceiptEntry {
        schema_version: EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_RECEIPT_SCHEMA_VERSION
            .to_string(),
        receipt_id: receipt_id.into(),
        intent_id: intent.intent_id.clone(),
        requesting_agent_id: intent.requesting_agent_id.clone(),
        requesting_cluster_id: intent.requesting_cluster_id.clone(),
        capability_id: intent.capability_id.clone(),
        host_cluster_id: intent.host_cluster_id.clone(),
        host_daemon_id: intent.host_daemon_id.clone(),
        tool_name: intent.tool_name.clone(),
        operation: intent.operation.clone(),
        status: status.into(),
        receipt_contract_type: receipt_contract_type.into(),
        result_ref: result_ref.into(),
        result_summary: result_summary.into(),
        authority_gate: intent.authority_gate.clone(),
        private_state_exposed: false,
        notes: vec![
            "Receipt records the daemon tool response over CultMesh without exposing private Verse payloads.".to_string(),
            "The receipt contract is the tool-specific proof surface; this routing receipt keeps the global directory auditable.".to_string(),
        ],
    }
}

fn validate_daemon_tool_invocation_intent(
    intent: &EpiphanyCultMeshDaemonToolInvocationIntentEntry,
) -> Result<()> {
    if intent.private_state_requested {
        return Err(anyhow!(
            "daemon tool invocation intents must not request private Verse state"
        ));
    }
    if !intent.requires_receipt {
        return Err(anyhow!(
            "daemon tool invocation intents must require typed receipts"
        ));
    }
    for (label, value) in [
        ("intent id", intent.intent_id.as_str()),
        ("requesting agent id", intent.requesting_agent_id.as_str()),
        (
            "requesting cluster id",
            intent.requesting_cluster_id.as_str(),
        ),
        ("capability id", intent.capability_id.as_str()),
        ("host daemon id", intent.host_daemon_id.as_str()),
        ("tool name", intent.tool_name.as_str()),
        ("operation", intent.operation.as_str()),
        ("input contract type", intent.input_contract_type.as_str()),
        ("payload ref", intent.payload_ref.as_str()),
        ("authority gate", intent.authority_gate.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("daemon tool invocation intent missing {label}"));
        }
    }
    Ok(())
}

#[cfg(test)]
fn validate_daemon_tool_invocation_receipt(
    receipt: &EpiphanyCultMeshDaemonToolInvocationReceiptEntry,
) -> Result<()> {
    if receipt.private_state_exposed {
        return Err(anyhow!(
            "daemon tool invocation receipts must not expose private Verse state"
        ));
    }
    for (label, value) in [
        ("receipt id", receipt.receipt_id.as_str()),
        ("intent id", receipt.intent_id.as_str()),
        ("requesting agent id", receipt.requesting_agent_id.as_str()),
        (
            "requesting cluster id",
            receipt.requesting_cluster_id.as_str(),
        ),
        ("capability id", receipt.capability_id.as_str()),
        ("host daemon id", receipt.host_daemon_id.as_str()),
        ("tool name", receipt.tool_name.as_str()),
        ("operation", receipt.operation.as_str()),
        ("status", receipt.status.as_str()),
        (
            "receipt contract type",
            receipt.receipt_contract_type.as_str(),
        ),
        ("result ref", receipt.result_ref.as_str()),
        ("authority gate", receipt.authority_gate.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("daemon tool invocation receipt missing {label}"));
        }
    }
    Ok(())
}

pub fn write_epiphany_cultmesh_daemon_tool_invocation_intent(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    intent: EpiphanyCultMeshDaemonToolInvocationIntentEntry,
) -> Result<EpiphanyCultMeshDaemonToolInvocationIntentEntry> {
    validate_daemon_tool_invocation_intent(&intent)?;
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let intent_key = epiphany_cultmesh_daemon_tool_invocation_intent_key(&intent.intent_id);
    let written = node.put(intent_key.as_str(), &intent)?;
    node.put(
        EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_INTENT_LATEST_KEY,
        &written,
    )?;
    node.flush()?;
    Ok(written)
}

#[cfg(test)]
pub fn write_epiphany_cultmesh_daemon_tool_invocation_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    receipt: EpiphanyCultMeshDaemonToolInvocationReceiptEntry,
) -> Result<EpiphanyCultMeshDaemonToolInvocationReceiptEntry> {
    validate_daemon_tool_invocation_receipt(&receipt)?;
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let receipt_key = epiphany_cultmesh_daemon_tool_invocation_receipt_key(&receipt.receipt_id);
    let written = node.put(receipt_key.as_str(), &receipt)?;
    node.put(
        EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_RECEIPT_LATEST_KEY,
        &written,
    )?;
    node.flush()?;
    Ok(written)
}

pub fn load_latest_epiphany_cultmesh_daemon_tool_invocation_intent(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshDaemonToolInvocationIntentEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_INTENT_LATEST_KEY)
}

pub fn load_latest_epiphany_cultmesh_daemon_tool_invocation_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshDaemonToolInvocationReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_RECEIPT_LATEST_KEY)
}

pub fn epiphany_cultmesh_bifrost_body_change_publication_intent(
    intent_id: impl Into<String>,
    source_cluster_id: impl Into<String>,
    source_agent_id: impl Into<String>,
    body_domain: impl Into<String>,
    target_repository: impl Into<String>,
    target_branch: impl Into<String>,
    change_summary: impl Into<String>,
    justification: impl Into<String>,
    changed_paths: Vec<String>,
    verification_receipt_ids: Vec<String>,
    review_receipt_ids: Vec<String>,
    authorship_agent_ids: Vec<String>,
    credit_subjects: Vec<String>,
) -> EpiphanyCultMeshBifrostBodyChangePublicationIntentEntry {
    EpiphanyCultMeshBifrostBodyChangePublicationIntentEntry {
        schema_version: EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_INTENT_SCHEMA_VERSION
            .to_string(),
        intent_id: intent_id.into(),
        source_cluster_id: source_cluster_id.into(),
        source_agent_id: source_agent_id.into(),
        body_domain: body_domain.into(),
        target_repository: target_repository.into(),
        target_branch: target_branch.into(),
        change_summary: change_summary.into(),
        justification: justification.into(),
        changed_paths,
        verification_receipt_ids,
        review_receipt_ids,
        authorship_agent_ids,
        credit_subjects,
        github_publication_requested: true,
        private_state_included: false,
        notes: vec![
            "Bifrost publication intent routes a body change to the local trusted GameCult Verse before GitHub publication.".to_string(),
            "GitHub is the publication substrate; Bifrost owns ledger attribution, review proof, and credit routing.".to_string(),
            "Private worker/operator/agent state must stay sealed outside this operator-safe publication packet.".to_string(),
        ],
    }
}

#[cfg(test)]
pub fn epiphany_cultmesh_bifrost_body_change_publication_receipt_for_intent(
    receipt_id: impl Into<String>,
    intent: &EpiphanyCultMeshBifrostBodyChangePublicationIntentEntry,
    status: impl Into<String>,
    bifrost_ledger_entry_id: impl Into<String>,
    github_publication_receipt_id: impl Into<String>,
    credit_receipt_ids: Vec<String>,
    reviewer_ids: Vec<String>,
    publication_url: impl Into<String>,
) -> EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry {
    EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry {
        schema_version: EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_RECEIPT_SCHEMA_VERSION
            .to_string(),
        receipt_id: receipt_id.into(),
        intent_id: intent.intent_id.clone(),
        status: status.into(),
        bifrost_ledger_entry_id: bifrost_ledger_entry_id.into(),
        github_publication_receipt_id: github_publication_receipt_id.into(),
        credit_receipt_ids,
        accepted_changed_paths: intent.changed_paths.clone(),
        reviewer_ids,
        publication_url: publication_url.into(),
        private_state_exposed: false,
        notes: vec![
            "Bifrost receipt records publication routing and ledger attribution before treating GitHub publication as blessed.".to_string(),
            "Credit and GitHub receipts are referenced as typed proof surfaces, not hidden side effects.".to_string(),
        ],
    }
}

pub fn write_epiphany_cultmesh_bifrost_body_change_publication_intent(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    intent: EpiphanyCultMeshBifrostBodyChangePublicationIntentEntry,
) -> Result<EpiphanyCultMeshBifrostBodyChangePublicationIntentEntry> {
    validate_bifrost_body_change_publication_intent(&intent)?;
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let intent_key =
        epiphany_cultmesh_bifrost_body_change_publication_intent_key(&intent.intent_id);
    let written = node.put(intent_key.as_str(), &intent)?;
    node.put(
        EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_INTENT_ARRIVAL_LATEST_KEY,
        &written,
    )?;
    node.flush()?;
    Ok(written)
}

#[cfg(test)]
pub fn write_epiphany_cultmesh_bifrost_body_change_publication_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    receipt: EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry,
) -> Result<EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry> {
    validate_bifrost_body_change_publication_receipt(&receipt)?;
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let receipt_key =
        epiphany_cultmesh_bifrost_body_change_publication_receipt_key(&receipt.receipt_id);
    let written = node.put(receipt_key.as_str(), &receipt)?;
    node.put(
        EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_RECEIPT_ARRIVAL_LATEST_KEY,
        &written,
    )?;
    node.flush()?;
    Ok(written)
}

pub fn load_arrival_latest_epiphany_cultmesh_bifrost_body_change_publication_intent(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshBifrostBodyChangePublicationIntentEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_INTENT_ARRIVAL_LATEST_KEY)
}

pub fn load_arrival_latest_epiphany_cultmesh_bifrost_body_change_publication_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_RECEIPT_ARRIVAL_LATEST_KEY)
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub fn epiphany_cultmesh_bifrost_github_publication_receipt_for_publication(
    receipt_id: impl Into<String>,
    publication_receipt: &EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry,
    hands_pr_receipt_id: impl Into<String>,
    target_repository: impl Into<String>,
    target_branch: impl Into<String>,
    pull_request_number: impl Into<String>,
    commit_sha: impl Into<String>,
    published_by_agent_id: impl Into<String>,
) -> EpiphanyCultMeshBifrostGithubPublicationReceiptEntry {
    EpiphanyCultMeshBifrostGithubPublicationReceiptEntry {
        schema_version: EPIPHANY_CULTMESH_BIFROST_GITHUB_PUBLICATION_RECEIPT_SCHEMA_VERSION
            .to_string(),
        receipt_id: receipt_id.into(),
        bifrost_publication_receipt_id: publication_receipt.receipt_id.clone(),
        hands_pr_receipt_id: hands_pr_receipt_id.into(),
        target_repository: target_repository.into(),
        target_branch: target_branch.into(),
        pull_request_url: publication_receipt.publication_url.clone(),
        pull_request_number: pull_request_number.into(),
        commit_sha: commit_sha.into(),
        changed_paths: publication_receipt.accepted_changed_paths.clone(),
        ledger_entry_id: publication_receipt.bifrost_ledger_entry_id.clone(),
        credit_receipt_ids: publication_receipt.credit_receipt_ids.clone(),
        published_by_agent_id: published_by_agent_id.into(),
        publication_status: publication_receipt.status.clone(),
        private_state_exposed: false,
        notes: vec![
            "Bifrost GitHub publication receipt binds the Bifrost ledger decision to a concrete Hands PR receipt.".to_string(),
            "GitHub is recorded as a publication substrate; Bifrost remains the routing and credit authority.".to_string(),
            "This receipt must not expose private worker, operator, or agent-thought state.".to_string(),
        ],
    }
}

#[cfg(test)]
pub fn write_epiphany_cultmesh_bifrost_github_publication_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    receipt: EpiphanyCultMeshBifrostGithubPublicationReceiptEntry,
) -> Result<EpiphanyCultMeshBifrostGithubPublicationReceiptEntry> {
    validate_bifrost_github_publication_receipt(&receipt)?;
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let receipt_key = epiphany_cultmesh_bifrost_github_publication_receipt_key(&receipt.receipt_id);
    let written = node.put(receipt_key.as_str(), &receipt)?;
    node.put(
        EPIPHANY_CULTMESH_BIFROST_GITHUB_PUBLICATION_RECEIPT_ARRIVAL_LATEST_KEY,
        &written,
    )?;
    node.flush()?;
    Ok(written)
}

pub fn load_arrival_latest_epiphany_cultmesh_bifrost_github_publication_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshBifrostGithubPublicationReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_BIFROST_GITHUB_PUBLICATION_RECEIPT_ARRIVAL_LATEST_KEY)
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub fn epiphany_cultmesh_bifrost_public_proof_publication_receipt_for_proof(
    receipt_id: impl Into<String>,
    proof: &EpiphanyCultMeshRepoWorkPublicProofEntry,
    status: impl Into<String>,
    target_public_verse_id: impl Into<String>,
    public_room_id: impl Into<String>,
    bifrost_ledger_entry_id: impl Into<String>,
    credit_receipt_ids: Vec<String>,
    reviewer_ids: Vec<String>,
    publication_url: impl Into<String>,
) -> EpiphanyCultMeshBifrostPublicProofPublicationReceiptEntry {
    EpiphanyCultMeshBifrostPublicProofPublicationReceiptEntry {
        schema_version:
            EPIPHANY_CULTMESH_BIFROST_PUBLIC_PROOF_PUBLICATION_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: receipt_id.into(),
        public_proof_id: proof.public_proof_id.clone(),
        public_proof_ref: proof.public_proof_ref.clone(),
        public_proof_sha256: proof.public_proof_sha256.clone(),
        item: proof.item.clone(),
        source_workspace: proof.workspace.clone(),
        source_branch: proof.branch.clone(),
        target_public_verse_id: target_public_verse_id.into(),
        public_room_id: public_room_id.into(),
        status: status.into(),
        bifrost_ledger_entry_id: bifrost_ledger_entry_id.into(),
        credit_receipt_ids,
        reviewer_ids,
        publication_url: publication_url.into(),
        private_state_exposed: false,
        notes: vec![
            "Bifrost public-proof publication receipt binds a redacted repo-work proof bundle to a public Verse room.".to_string(),
            "The receipt carries only proof refs, hashes, ledger, review, and credit ids; private worker/operator/agent state remains sealed.".to_string(),
            "Downstream consumers may read this closure, but Bifrost owns public publication authority.".to_string(),
        ],
    }
}

#[cfg(test)]
pub fn write_epiphany_cultmesh_bifrost_public_proof_publication_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    receipt: EpiphanyCultMeshBifrostPublicProofPublicationReceiptEntry,
) -> Result<EpiphanyCultMeshBifrostPublicProofPublicationReceiptEntry> {
    validate_bifrost_public_proof_publication_receipt(&receipt)?;
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let receipt_key =
        epiphany_cultmesh_bifrost_public_proof_publication_receipt_key(&receipt.receipt_id);
    let written = node.put(receipt_key.as_str(), &receipt)?;
    node.put(
        EPIPHANY_CULTMESH_BIFROST_PUBLIC_PROOF_PUBLICATION_RECEIPT_ARRIVAL_LATEST_KEY,
        &written,
    )?;
    node.flush()?;
    Ok(written)
}

pub fn load_arrival_latest_epiphany_cultmesh_bifrost_public_proof_publication_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshBifrostPublicProofPublicationReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_BIFROST_PUBLIC_PROOF_PUBLICATION_RECEIPT_ARRIVAL_LATEST_KEY)
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub fn epiphany_cultmesh_bifrost_artifact_acceptance_receipt_for_map_entry(
    receipt_id: impl Into<String>,
    map_entry: &EpiphanyCultMeshRepoWorkMapEntry,
    artifact_ref: impl Into<String>,
    public_proof_ref: impl Into<String>,
    maintainer_review_receipt_ids: Vec<String>,
    bifrost_ledger_entry_id: impl Into<String>,
    status: impl Into<String>,
    accepted_by: impl Into<String>,
) -> EpiphanyCultMeshBifrostArtifactAcceptanceReceiptEntry {
    EpiphanyCultMeshBifrostArtifactAcceptanceReceiptEntry {
        schema_version: EPIPHANY_CULTMESH_BIFROST_ARTIFACT_ACCEPTANCE_RECEIPT_SCHEMA_VERSION
            .to_string(),
        receipt_id: receipt_id.into(),
        item: map_entry.item.clone(),
        source_workspace: map_entry.workspace.clone(),
        source_branch: map_entry.branch.clone(),
        commit_sha: map_entry.commit_sha.clone(),
        changed_paths: map_entry.changed_paths.clone(),
        artifact_ref: artifact_ref.into(),
        public_proof_ref: public_proof_ref.into(),
        maintainer_review_receipt_ids,
        bifrost_ledger_entry_id: bifrost_ledger_entry_id.into(),
        status: status.into(),
        accepted_by: accepted_by.into(),
        private_state_exposed: false,
        notes: vec![
            "Bifrost artifact acceptance receipt closes accepted-artifact accounting for Mind-admitted branch work.".to_string(),
            "This receipt carries artifact, review, ledger, commit, and path refs only; private worker/operator/agent state remains sealed.".to_string(),
            "Repo-work request cargo may ask for this receipt; Maintainer owns acceptance and Bifrost owns accounting.".to_string(),
        ],
    }
}

#[cfg(test)]
pub fn write_epiphany_cultmesh_bifrost_artifact_acceptance_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    receipt: EpiphanyCultMeshBifrostArtifactAcceptanceReceiptEntry,
) -> Result<EpiphanyCultMeshBifrostArtifactAcceptanceReceiptEntry> {
    validate_bifrost_artifact_acceptance_receipt(&receipt)?;
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let receipt_key =
        epiphany_cultmesh_bifrost_artifact_acceptance_receipt_key(&receipt.receipt_id);
    let written = node.put(receipt_key.as_str(), &receipt)?;
    node.put(
        EPIPHANY_CULTMESH_BIFROST_ARTIFACT_ACCEPTANCE_RECEIPT_ARRIVAL_LATEST_KEY,
        &written,
    )?;
    node.flush()?;
    Ok(written)
}

pub fn load_arrival_latest_epiphany_cultmesh_bifrost_artifact_acceptance_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshBifrostArtifactAcceptanceReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_BIFROST_ARTIFACT_ACCEPTANCE_RECEIPT_ARRIVAL_LATEST_KEY)
}

#[allow(clippy::too_many_arguments)]
#[cfg(test)]
pub fn epiphany_cultmesh_bifrost_metrics_receipt_for_map_entry(
    receipt_id: impl Into<String>,
    map_entry: &EpiphanyCultMeshRepoWorkMapEntry,
    artifact_acceptance_receipt_id: impl Into<String>,
    model_spend_receipt_ids: Vec<String>,
    review_load_receipt_ids: Vec<String>,
    credit_readback_receipt_ids: Vec<String>,
    public_proof_ref: impl Into<String>,
    metrics_summary: impl Into<String>,
    status: impl Into<String>,
) -> EpiphanyCultMeshBifrostMetricsReceiptEntry {
    EpiphanyCultMeshBifrostMetricsReceiptEntry {
        schema_version: EPIPHANY_CULTMESH_BIFROST_METRICS_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id: receipt_id.into(),
        item: map_entry.item.clone(),
        source_workspace: map_entry.workspace.clone(),
        source_branch: map_entry.branch.clone(),
        artifact_acceptance_receipt_id: artifact_acceptance_receipt_id.into(),
        model_spend_receipt_ids,
        review_load_receipt_ids,
        credit_readback_receipt_ids,
        public_proof_ref: public_proof_ref.into(),
        metrics_summary: metrics_summary.into(),
        status: status.into(),
        private_state_exposed: false,
        token_summary_ref: Some("metrics://model-spend/tokens".to_string()),
        cost_availability_status: Some("known".to_string()),
        cost_summary_ref: Some("metrics://model-spend/cost".to_string()),
        cost_unavailable_reason: None,
        review_duration_ms: Some(1),
        review_event_count: Some(1),
        notes: vec![
            "Bifrost metrics receipt closes model-spend, review-load, accepted-artifact, and credit-readback accounting for branch work.".to_string(),
            "Metrics are operator-safe refs and summaries, not private worker transcripts or raw model streams.".to_string(),
            "Repo-work request cargo may ask for this receipt; Bifrost owns accounting and Maintainer owns review-load evidence.".to_string(),
        ],
    }
}

#[cfg(test)]
pub fn write_epiphany_cultmesh_bifrost_metrics_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    receipt: EpiphanyCultMeshBifrostMetricsReceiptEntry,
) -> Result<EpiphanyCultMeshBifrostMetricsReceiptEntry> {
    validate_bifrost_metrics_receipt(&receipt)?;
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let receipt_key = epiphany_cultmesh_bifrost_metrics_receipt_key(&receipt.receipt_id);
    let written = node.put(receipt_key.as_str(), &receipt)?;
    node.put(
        EPIPHANY_CULTMESH_BIFROST_METRICS_RECEIPT_ARRIVAL_LATEST_KEY,
        &written,
    )?;
    node.flush()?;
    Ok(written)
}

pub fn load_arrival_latest_epiphany_cultmesh_bifrost_metrics_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshBifrostMetricsReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_BIFROST_METRICS_RECEIPT_ARRIVAL_LATEST_KEY)
}

pub fn epiphany_cultmesh_bifrost_collaboration_feedback(
    feedback_id: impl Into<String>,
    source_persona_id: impl Into<String>,
    source_cluster_id: impl Into<String>,
    public_room_id: impl Into<String>,
    eve_connection_receipt_id: impl Into<String>,
    collaboration_topic: impl Into<String>,
    feedback_summary: impl Into<String>,
    public_discussion_refs: Vec<String>,
    candidate_action_refs: Vec<String>,
) -> EpiphanyCultMeshBifrostCollaborationFeedbackEntry {
    EpiphanyCultMeshBifrostCollaborationFeedbackEntry {
        schema_version: EPIPHANY_CULTMESH_BIFROST_COLLABORATION_FEEDBACK_SCHEMA_VERSION
            .to_string(),
        feedback_id: feedback_id.into(),
        source_persona_id: source_persona_id.into(),
        source_cluster_id: source_cluster_id.into(),
        public_room_id: public_room_id.into(),
        eve_connection_receipt_id: eve_connection_receipt_id.into(),
        collaboration_topic: collaboration_topic.into(),
        feedback_summary: feedback_summary.into(),
        public_discussion_refs,
        requested_consensus_route: "imagination.consensus_discovery".to_string(),
        candidate_action_refs,
        private_state_included: false,
        notes: vec![
            "Public Persona collaboration feedback is Bifrost-local witness, not implementation authority.".to_string(),
            "Feedback routes to Imagination consensus discovery before any adoption or work item can be blessed.".to_string(),
            "Private worker, operator, or agent-thought state must stay sealed outside this packet.".to_string(),
        ],
    }
}

#[cfg(test)]
pub fn epiphany_cultmesh_imagination_consensus_receipt_for_feedback(
    receipt_id: impl Into<String>,
    feedback: &EpiphanyCultMeshBifrostCollaborationFeedbackEntry,
    status: impl Into<String>,
    imagination_agent_ids: Vec<String>,
    consensus_packet_ref: impl Into<String>,
) -> EpiphanyCultMeshImaginationConsensusReceiptEntry {
    EpiphanyCultMeshImaginationConsensusReceiptEntry {
        schema_version: EPIPHANY_CULTMESH_IMAGINATION_CONSENSUS_RECEIPT_SCHEMA_VERSION
            .to_string(),
        receipt_id: receipt_id.into(),
        feedback_id: feedback.feedback_id.clone(),
        source_persona_id: feedback.source_persona_id.clone(),
        consensus_route: feedback.requested_consensus_route.clone(),
        status: status.into(),
        imagination_agent_ids,
        consensus_packet_ref: consensus_packet_ref.into(),
        adoption_gate: "mind.review_then_bifrost_adoption".to_string(),
        public_feedback_refs: feedback.public_discussion_refs.clone(),
        private_state_exposed: false,
        notes: vec![
            "Imagination consensus receipt records that public feedback entered future-shape analysis, not that work was adopted.".to_string(),
            "Mind and Bifrost remain the adoption gates before durable state or body changes.".to_string(),
        ],
    }
}

pub fn write_epiphany_cultmesh_bifrost_collaboration_feedback(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    feedback: EpiphanyCultMeshBifrostCollaborationFeedbackEntry,
) -> Result<EpiphanyCultMeshBifrostCollaborationFeedbackEntry> {
    validate_bifrost_collaboration_feedback(&feedback)?;
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let feedback_key = epiphany_cultmesh_bifrost_collaboration_feedback_key(&feedback.feedback_id);
    let written = node.put(feedback_key.as_str(), &feedback)?;
    node.put(
        EPIPHANY_CULTMESH_BIFROST_COLLABORATION_FEEDBACK_ARRIVAL_LATEST_KEY,
        &written,
    )?;
    node.flush()?;
    Ok(written)
}

#[cfg(test)]
pub fn write_epiphany_cultmesh_imagination_consensus_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    receipt: EpiphanyCultMeshImaginationConsensusReceiptEntry,
) -> Result<EpiphanyCultMeshImaginationConsensusReceiptEntry> {
    validate_imagination_consensus_receipt(&receipt)?;
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let receipt_key = epiphany_cultmesh_imagination_consensus_receipt_key(&receipt.receipt_id);
    let written = node.put(receipt_key.as_str(), &receipt)?;
    node.put(
        EPIPHANY_CULTMESH_IMAGINATION_CONSENSUS_RECEIPT_LATEST_KEY,
        &written,
    )?;
    node.flush()?;
    Ok(written)
}

pub fn load_arrival_latest_epiphany_cultmesh_bifrost_collaboration_feedback(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshBifrostCollaborationFeedbackEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_BIFROST_COLLABORATION_FEEDBACK_ARRIVAL_LATEST_KEY)
}

pub fn load_latest_epiphany_cultmesh_imagination_consensus_receipt(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshImaginationConsensusReceiptEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_IMAGINATION_CONSENSUS_RECEIPT_LATEST_KEY)
}

fn validate_bifrost_body_change_publication_intent(
    intent: &EpiphanyCultMeshBifrostBodyChangePublicationIntentEntry,
) -> Result<()> {
    if intent.private_state_included {
        return Err(anyhow!(
            "Bifrost body change publication intents must not include private state"
        ));
    }
    if !intent.github_publication_requested {
        return Err(anyhow!(
            "Bifrost body change publication intents must request GitHub publication routing"
        ));
    }
    if intent.justification.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost body change publication intents require justification"
        ));
    }
    if intent.changed_paths.is_empty() {
        return Err(anyhow!(
            "Bifrost body change publication intents require changed path scope"
        ));
    }
    if intent.verification_receipt_ids.is_empty() {
        return Err(anyhow!(
            "Bifrost body change publication intents require verification receipts"
        ));
    }
    if intent.review_receipt_ids.is_empty() {
        return Err(anyhow!(
            "Bifrost body change publication intents require review receipts"
        ));
    }
    if intent.authorship_agent_ids.is_empty() {
        return Err(anyhow!(
            "Bifrost body change publication intents require authorship"
        ));
    }
    if intent.credit_subjects.is_empty() {
        return Err(anyhow!(
            "Bifrost body change publication intents require credit metadata"
        ));
    }
    Ok(())
}

#[cfg(test)]
fn validate_bifrost_body_change_publication_receipt(
    receipt: &EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry,
) -> Result<()> {
    if receipt.private_state_exposed {
        return Err(anyhow!(
            "Bifrost body change publication receipts must not expose private state"
        ));
    }
    if receipt.bifrost_ledger_entry_id.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost body change publication receipts require a ledger entry"
        ));
    }
    if receipt.github_publication_receipt_id.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost body change publication receipts require a GitHub publication receipt"
        ));
    }
    if receipt.credit_receipt_ids.is_empty() {
        return Err(anyhow!(
            "Bifrost body change publication receipts require credit receipts"
        ));
    }
    Ok(())
}

#[cfg(test)]
fn validate_bifrost_github_publication_receipt(
    receipt: &EpiphanyCultMeshBifrostGithubPublicationReceiptEntry,
) -> Result<()> {
    if receipt.private_state_exposed {
        return Err(anyhow!(
            "Bifrost GitHub publication receipts must not expose private state"
        ));
    }
    if receipt.bifrost_publication_receipt_id.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost GitHub publication receipts require a Bifrost publication receipt"
        ));
    }
    if receipt.hands_pr_receipt_id.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost GitHub publication receipts require a Hands PR receipt"
        ));
    }
    if receipt.pull_request_url.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost GitHub publication receipts require a pull request URL"
        ));
    }
    if receipt.ledger_entry_id.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost GitHub publication receipts require a ledger entry"
        ));
    }
    if receipt.credit_receipt_ids.is_empty() {
        return Err(anyhow!(
            "Bifrost GitHub publication receipts require credit receipts"
        ));
    }
    if receipt.changed_paths.is_empty() {
        return Err(anyhow!(
            "Bifrost GitHub publication receipts require changed paths"
        ));
    }
    Ok(())
}

#[cfg(test)]
fn validate_bifrost_public_proof_publication_receipt(
    receipt: &EpiphanyCultMeshBifrostPublicProofPublicationReceiptEntry,
) -> Result<()> {
    if receipt.schema_version
        != EPIPHANY_CULTMESH_BIFROST_PUBLIC_PROOF_PUBLICATION_RECEIPT_SCHEMA_VERSION
    {
        return Err(anyhow!(
            "Bifrost public proof publication receipts require schema version {}",
            EPIPHANY_CULTMESH_BIFROST_PUBLIC_PROOF_PUBLICATION_RECEIPT_SCHEMA_VERSION
        ));
    }
    if receipt.private_state_exposed {
        return Err(anyhow!(
            "Bifrost public proof publication receipts must not expose private state"
        ));
    }
    if receipt.public_proof_id.trim().is_empty()
        || receipt.public_proof_ref.trim().is_empty()
        || receipt.public_proof_sha256.trim().is_empty()
    {
        return Err(anyhow!(
            "Bifrost public proof publication receipts require proof id, ref, and SHA-256"
        ));
    }
    if receipt.target_public_verse_id != EPIPHANY_CULTMESH_GLOBAL_VERSE_ID {
        return Err(anyhow!(
            "Bifrost public proof publication receipts must target the global public Verse"
        ));
    }
    if receipt.public_room_id.trim().is_empty() || receipt.publication_url.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost public proof publication receipts require a public room and publication URL"
        ));
    }
    if receipt.bifrost_ledger_entry_id.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost public proof publication receipts require a ledger entry"
        ));
    }
    if receipt.credit_receipt_ids.is_empty() {
        return Err(anyhow!(
            "Bifrost public proof publication receipts require credit receipts"
        ));
    }
    if receipt.reviewer_ids.is_empty() {
        return Err(anyhow!(
            "Bifrost public proof publication receipts require reviewer receipts"
        ));
    }
    Ok(())
}

#[cfg(test)]
fn validate_bifrost_artifact_acceptance_receipt(
    receipt: &EpiphanyCultMeshBifrostArtifactAcceptanceReceiptEntry,
) -> Result<()> {
    if receipt.schema_version
        != EPIPHANY_CULTMESH_BIFROST_ARTIFACT_ACCEPTANCE_RECEIPT_SCHEMA_VERSION
    {
        return Err(anyhow!(
            "Bifrost artifact acceptance receipts require schema version {}",
            EPIPHANY_CULTMESH_BIFROST_ARTIFACT_ACCEPTANCE_RECEIPT_SCHEMA_VERSION
        ));
    }
    if receipt.private_state_exposed {
        return Err(anyhow!(
            "Bifrost artifact acceptance receipts must not expose private state"
        ));
    }
    if receipt.item.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost artifact acceptance receipts require an item"
        ));
    }
    if receipt.artifact_ref.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost artifact acceptance receipts require an artifact ref"
        ));
    }
    if receipt.public_proof_ref.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost artifact acceptance receipts require a public proof ref"
        ));
    }
    if receipt.commit_sha.trim().is_empty() || receipt.commit_sha == "none" {
        return Err(anyhow!(
            "Bifrost artifact acceptance receipts require a commit SHA"
        ));
    }
    if receipt.changed_paths.is_empty() {
        return Err(anyhow!(
            "Bifrost artifact acceptance receipts require changed paths"
        ));
    }
    if receipt.maintainer_review_receipt_ids.is_empty() {
        return Err(anyhow!(
            "Bifrost artifact acceptance receipts require maintainer review receipts"
        ));
    }
    if receipt.bifrost_ledger_entry_id.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost artifact acceptance receipts require a ledger entry"
        ));
    }
    if receipt.accepted_by.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost artifact acceptance receipts require an accepted-by authority"
        ));
    }
    Ok(())
}

#[cfg(test)]
fn validate_bifrost_metrics_receipt(
    receipt: &EpiphanyCultMeshBifrostMetricsReceiptEntry,
) -> Result<()> {
    if receipt.schema_version != EPIPHANY_CULTMESH_BIFROST_METRICS_RECEIPT_SCHEMA_VERSION {
        return Err(anyhow!(
            "Bifrost metrics receipts require schema version {}",
            EPIPHANY_CULTMESH_BIFROST_METRICS_RECEIPT_SCHEMA_VERSION
        ));
    }
    if receipt.private_state_exposed {
        return Err(anyhow!(
            "Bifrost metrics receipts must not expose private state"
        ));
    }
    if receipt.item.trim().is_empty() {
        return Err(anyhow!("Bifrost metrics receipts require an item"));
    }
    if receipt.artifact_acceptance_receipt_id.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost metrics receipts require an artifact acceptance receipt"
        ));
    }
    if receipt.model_spend_receipt_ids.is_empty() {
        return Err(anyhow!(
            "Bifrost metrics receipts require model spend receipts"
        ));
    }
    if receipt.review_load_receipt_ids.is_empty() {
        return Err(anyhow!(
            "Bifrost metrics receipts require review load receipts"
        ));
    }
    if receipt.credit_readback_receipt_ids.is_empty() {
        return Err(anyhow!(
            "Bifrost metrics receipts require credit readback receipts"
        ));
    }
    if receipt.public_proof_ref.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost metrics receipts require a public proof ref"
        ));
    }
    if receipt.metrics_summary.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost metrics receipts require a metrics summary"
        ));
    }
    if receipt
        .token_summary_ref
        .as_deref()
        .unwrap_or_default()
        .trim()
        .is_empty()
    {
        return Err(anyhow!(
            "Bifrost metrics receipts require a token summary ref"
        ));
    }
    match receipt.cost_availability_status.as_deref() {
        Some("known")
            if receipt
                .cost_summary_ref
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty() =>
        {
            return Err(anyhow!("known metric cost requires a cost summary ref"));
        }
        Some("unavailable")
            if receipt
                .cost_unavailable_reason
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty() =>
        {
            return Err(anyhow!("unavailable metric cost requires a reason"));
        }
        Some("known" | "unavailable") => {}
        _ => {
            return Err(anyhow!(
                "metric cost availability must be known or unavailable"
            ));
        }
    }
    if receipt.review_duration_ms.unwrap_or_default() == 0 {
        return Err(anyhow!("Bifrost metrics receipts require review duration"));
    }
    if receipt.review_event_count.unwrap_or_default() == 0 {
        return Err(anyhow!(
            "Bifrost metrics receipts require review event count"
        ));
    }
    Ok(())
}

fn validate_bifrost_collaboration_feedback(
    feedback: &EpiphanyCultMeshBifrostCollaborationFeedbackEntry,
) -> Result<()> {
    if feedback.private_state_included {
        return Err(anyhow!(
            "Bifrost collaboration feedback must not include private state"
        ));
    }
    if feedback.source_persona_id.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost collaboration feedback requires a Persona source"
        ));
    }
    if feedback.public_room_id.trim().is_empty() || feedback.public_discussion_refs.is_empty() {
        return Err(anyhow!(
            "Bifrost collaboration feedback requires public discussion references"
        ));
    }
    if feedback.eve_connection_receipt_id.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost collaboration feedback requires an Eve connection receipt"
        ));
    }
    if feedback.feedback_summary.trim().is_empty() {
        return Err(anyhow!(
            "Bifrost collaboration feedback requires a feedback summary"
        ));
    }
    if feedback.requested_consensus_route != "imagination.consensus_discovery" {
        return Err(anyhow!(
            "Bifrost collaboration feedback must route to Imagination consensus discovery"
        ));
    }
    Ok(())
}

#[cfg(test)]
fn validate_imagination_consensus_receipt(
    receipt: &EpiphanyCultMeshImaginationConsensusReceiptEntry,
) -> Result<()> {
    if receipt.private_state_exposed {
        return Err(anyhow!(
            "Imagination consensus receipts must not expose private state"
        ));
    }
    if receipt.feedback_id.trim().is_empty() {
        return Err(anyhow!(
            "Imagination consensus receipts require a feedback id"
        ));
    }
    if receipt.consensus_route != "imagination.consensus_discovery" {
        return Err(anyhow!(
            "Imagination consensus receipts must use the consensus discovery route"
        ));
    }
    if receipt.imagination_agent_ids.is_empty() {
        return Err(anyhow!(
            "Imagination consensus receipts require Imagination agent ids"
        ));
    }
    if receipt.consensus_packet_ref.trim().is_empty() {
        return Err(anyhow!(
            "Imagination consensus receipts require a consensus packet reference"
        ));
    }
    if receipt.adoption_gate.trim().is_empty() {
        return Err(anyhow!(
            "Imagination consensus receipts require an adoption gate"
        ));
    }
    Ok(())
}

pub fn write_epiphany_cultmesh_work_loop_telemetry(
    store_path: impl AsRef<Path>,
    telemetry: EpiphanyCultMeshWorkLoopTelemetryEntry,
) -> Result<EpiphanyCultMeshWorkLoopTelemetryEntry> {
    validate_work_loop_telemetry(&telemetry)?;
    let mut node = open_epiphany_cultmesh_node(&store_path, telemetry.runtime_id.clone())?;
    let written = node.put(telemetry.telemetry_id.clone(), &telemetry)?;
    let current_latest = node.get::<EpiphanyCultMeshWorkLoopTelemetryEntry>(
        EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_LATEST_KEY,
    )?;
    if current_latest.as_ref().is_none_or(|current| {
        work_loop_telemetry_event_key(&written) >= work_loop_telemetry_event_key(current)
    }) {
        node.put(EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_LATEST_KEY, &written)?;
    }
    node.flush()?;
    Ok(written)
}

fn validate_work_loop_telemetry(telemetry: &EpiphanyCultMeshWorkLoopTelemetryEntry) -> Result<()> {
    if telemetry.verse_id != EPIPHANY_CULTMESH_INTERNAL_VERSE_ID {
        return Err(anyhow!(
            "work-loop telemetry must remain in the internal Verse"
        ));
    }
    for (label, value) in [
        ("schema version", telemetry.schema_version.as_str()),
        ("runtime id", telemetry.runtime_id.as_str()),
        ("telemetry id", telemetry.telemetry_id.as_str()),
        ("source stage", telemetry.source_stage.as_str()),
        ("Hands intent id", telemetry.hands_intent_id.as_str()),
        ("Hands review id", telemetry.hands_review_id.as_str()),
        (
            "Hands runtime job id",
            telemetry.hands_runtime_job_id.as_str(),
        ),
        (
            "Substrate Gate grant receipt id",
            telemetry.substrate_gate_grant_receipt_id.as_str(),
        ),
        (
            "Hands patch receipt id",
            telemetry.hands_patch_receipt_id.as_str(),
        ),
        (
            "Hands command receipt id",
            telemetry.hands_command_receipt_id.as_str(),
        ),
        (
            "Hands commit receipt id",
            telemetry.hands_commit_receipt_id.as_str(),
        ),
        ("command", telemetry.command.as_str()),
        ("commit SHA", telemetry.commit_sha.as_str()),
        ("branch", telemetry.branch.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("work-loop telemetry missing {label}"));
        }
    }
    if telemetry.target_stages.is_empty()
        || telemetry
            .target_stages
            .iter()
            .any(|stage| stage.trim().is_empty())
        || telemetry.changed_paths.is_empty()
        || telemetry
            .changed_paths
            .iter()
            .any(|path| path.trim().is_empty())
    {
        return Err(anyhow!(
            "work-loop telemetry requires nonempty target stages and changed paths"
        ));
    }
    let produced_at = DateTime::parse_from_rfc3339(&telemetry.produced_at_utc)
        .map_err(|error| anyhow!("work-loop telemetry has invalid produced at: {error}"))?;
    let lower_bound = DateTime::parse_from_rfc3339(&telemetry.lower_bound_receipt_at)
        .map_err(|error| anyhow!("work-loop telemetry has invalid receipt lower bound: {error}"))?;
    if lower_bound > produced_at {
        return Err(anyhow!(
            "work-loop telemetry receipt lower bound occurs after packet production"
        ));
    }
    Ok(())
}

fn work_loop_telemetry_event_key(
    telemetry: &EpiphanyCultMeshWorkLoopTelemetryEntry,
) -> (DateTime<FixedOffset>, &str) {
    (
        DateTime::parse_from_rfc3339(&telemetry.produced_at_utc)
            .expect("validated work-loop telemetry timestamp"),
        telemetry.telemetry_id.as_str(),
    )
}

pub fn load_latest_epiphany_cultmesh_work_loop_telemetry(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshWorkLoopTelemetryEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_LATEST_KEY)
}

pub fn epiphany_local_verse_work_loop_summary(
    telemetry: &EpiphanyCultMeshWorkLoopTelemetryEntry,
) -> EpiphanyLocalVerseWorkLoopSummary {
    EpiphanyLocalVerseWorkLoopSummary {
        telemetry_id: telemetry.telemetry_id.clone(),
        thread_id: telemetry.thread_id.clone(),
        source_stage: telemetry.source_stage.clone(),
        target_stages: telemetry.target_stages.clone(),
        hands_intent_id: telemetry.hands_intent_id.clone(),
        hands_review_id: telemetry.hands_review_id.clone(),
        substrate_gate_grant_receipt_id: telemetry.substrate_gate_grant_receipt_id.clone(),
        hands_patch_receipt_id: telemetry.hands_patch_receipt_id.clone(),
        hands_command_receipt_id: telemetry.hands_command_receipt_id.clone(),
        hands_commit_receipt_id: telemetry.hands_commit_receipt_id.clone(),
        commit_sha: telemetry.commit_sha.clone(),
        branch: telemetry.branch.clone(),
        changed_path_count: telemetry.changed_paths.len(),
        source_ref_count: telemetry.source_refs.len(),
        soul_receipt_ids: telemetry.soul_receipt_ids.clone(),
        verification_assertion_count: telemetry.verification_assertions.len(),
        summary: telemetry.summary.clone(),
        sealed_preview_note: "Internal work-loop telemetry may carry receipt bodies, artifact previews, and commit diff previews; local Verse context exposes only this digest.".to_string(),
    }
}

pub fn epiphany_cultmesh_agent_state_soa_summary_from_entry(
    runtime_id: impl Into<String>,
    summary_id: impl Into<String>,
    soa: &EpiphanyAgentStateSoaEntry,
) -> EpiphanyCultMeshAgentStateSoaSummaryEntry {
    EpiphanyCultMeshAgentStateSoaSummaryEntry {
        schema_version: EPIPHANY_CULTMESH_AGENT_STATE_SOA_SUMMARY_SCHEMA_VERSION.to_string(),
        runtime_id: runtime_id.into(),
        verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
        summary_id: summary_id.into(),
        generated_at: soa.generated_at.clone(),
        source_store: soa.source_store.clone(),
        row_count: soa.role_ids.len() as u32,
        role_ids: soa.role_ids.clone(),
        agent_ids: soa.agent_ids.clone(),
        display_names: soa.display_names.clone(),
        profile_kinds: soa.profile_kinds.clone(),
        portable_contracts: soa.portable_contracts.clone(),
        semantic_memory_counts: soa.semantic_memory_counts.clone(),
        episodic_memory_counts: soa.episodic_memory_counts.clone(),
        relationship_memory_counts: soa.relationship_memory_counts.clone(),
        goal_counts: soa.goal_counts.clone(),
        value_counts: soa.value_counts.clone(),
        private_state_exposed: false,
        notes: vec![
            "Summary mirrors persisted epiphany.agent_state_soa.v0 column shape for local Verse discovery; agent memory text remains in the agent-memory store.".to_string(),
            "CultMesh carries row/column topology for Odin, Eve, and prompt assembly without becoming the agent-memory owner.".to_string(),
        ],
    }
}

pub fn write_epiphany_cultmesh_agent_state_soa_summary(
    store_path: impl AsRef<Path>,
    summary: EpiphanyCultMeshAgentStateSoaSummaryEntry,
) -> Result<EpiphanyCultMeshAgentStateSoaSummaryEntry> {
    validate_agent_state_soa_summary(&summary)?;
    let mut node = open_epiphany_cultmesh_node(&store_path, summary.runtime_id.clone())?;
    let written = node.put(summary.summary_id.clone(), &summary)?;
    let current_latest = node.get::<EpiphanyCultMeshAgentStateSoaSummaryEntry>(
        EPIPHANY_CULTMESH_AGENT_STATE_SOA_SUMMARY_LATEST_KEY,
    )?;
    if current_latest.as_ref().is_none_or(|current| {
        agent_state_soa_generation_key(&written) >= agent_state_soa_generation_key(current)
    }) {
        node.put(
            EPIPHANY_CULTMESH_AGENT_STATE_SOA_SUMMARY_LATEST_KEY,
            &written,
        )?;
    }
    node.flush()?;
    Ok(written)
}

pub fn load_latest_epiphany_cultmesh_agent_state_soa_summary(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshAgentStateSoaSummaryEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_AGENT_STATE_SOA_SUMMARY_LATEST_KEY)
}

fn validate_agent_state_soa_summary(
    summary: &EpiphanyCultMeshAgentStateSoaSummaryEntry,
) -> Result<()> {
    if summary.private_state_exposed {
        return Err(anyhow!(
            "agent state SoA summaries must not expose private state"
        ));
    }
    if summary.schema_version != EPIPHANY_CULTMESH_AGENT_STATE_SOA_SUMMARY_SCHEMA_VERSION {
        return Err(anyhow!(
            "agent state SoA summary schema_version must be {:?}",
            EPIPHANY_CULTMESH_AGENT_STATE_SOA_SUMMARY_SCHEMA_VERSION
        ));
    }
    if summary.verse_id != EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID {
        return Err(anyhow!(
            "agent state SoA summary belongs in the local-area Verse"
        ));
    }
    DateTime::parse_from_rfc3339(&summary.generated_at)
        .map_err(|error| anyhow!("agent state SoA summary has invalid generated_at: {error}"))?;
    let len = summary.role_ids.len();
    if summary.row_count as usize != len {
        return Err(anyhow!(
            "agent state SoA summary row_count is {}, expected {}",
            summary.row_count,
            len
        ));
    }
    for (name, candidate) in [
        ("agentIds", summary.agent_ids.len()),
        ("displayNames", summary.display_names.len()),
        ("profileKinds", summary.profile_kinds.len()),
        ("portableContracts", summary.portable_contracts.len()),
        ("semanticMemoryCounts", summary.semantic_memory_counts.len()),
        ("episodicMemoryCounts", summary.episodic_memory_counts.len()),
        (
            "relationshipMemoryCounts",
            summary.relationship_memory_counts.len(),
        ),
        ("goalCounts", summary.goal_counts.len()),
        ("valueCounts", summary.value_counts.len()),
    ] {
        if candidate != len {
            return Err(anyhow!(
                "agent state SoA summary column {name} has length {candidate}, expected {len}"
            ));
        }
    }
    if summary
        .role_ids
        .iter()
        .any(|role_id| role_id.trim().is_empty())
    {
        return Err(anyhow!("agent state SoA summary contains an empty role id"));
    }
    Ok(())
}

fn agent_state_soa_generation_key(
    summary: &EpiphanyCultMeshAgentStateSoaSummaryEntry,
) -> (DateTime<FixedOffset>, &str) {
    (
        DateTime::parse_from_rfc3339(&summary.generated_at)
            .expect("validated agent state SoA generation timestamp"),
        summary.summary_id.as_str(),
    )
}

pub fn load_latest_epiphany_cultmesh_repo_work_overview(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshRepoWorkOverviewEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_REPO_WORK_OVERVIEW_LATEST_KEY)
}

pub fn load_epiphany_cultmesh_repo_work_overviews(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshRepoWorkOverviewEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut overviews = node
        .get_all_with_keys::<EpiphanyCultMeshRepoWorkOverviewEntry>()?
        .into_iter()
        .filter(|(key, _)| key != EPIPHANY_CULTMESH_REPO_WORK_OVERVIEW_LATEST_KEY)
        .map(|(_, overview)| overview)
        .collect::<Vec<_>>();
    overviews.sort_by(|a, b| {
        b.generated_at
            .cmp(&a.generated_at)
            .then_with(|| a.overview_id.cmp(&b.overview_id))
    });
    Ok(overviews)
}

pub fn load_latest_epiphany_cultmesh_repo_work_readiness(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshRepoWorkReadinessEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_REPO_WORK_READINESS_LATEST_KEY)
}

pub fn load_epiphany_cultmesh_repo_work_readiness_reports(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshRepoWorkReadinessEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut reports = node
        .get_all_with_keys::<EpiphanyCultMeshRepoWorkReadinessEntry>()?
        .into_iter()
        .filter(|(key, _)| key != EPIPHANY_CULTMESH_REPO_WORK_READINESS_LATEST_KEY)
        .map(|(_, report)| report)
        .collect::<Vec<_>>();
    reports.sort_by(|a, b| {
        b.generated_at
            .cmp(&a.generated_at)
            .then_with(|| a.readiness_id.cmp(&b.readiness_id))
    });
    Ok(reports)
}

pub fn load_latest_epiphany_cultmesh_repo_work_map_entry(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshRepoWorkMapEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_REPO_WORK_MAP_ENTRY_LATEST_KEY)
}

pub fn load_epiphany_cultmesh_repo_work_map_entries(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshRepoWorkMapEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut entries = node
        .get_all_with_keys::<EpiphanyCultMeshRepoWorkMapEntry>()?
        .into_iter()
        .filter(|(key, _)| key != EPIPHANY_CULTMESH_REPO_WORK_MAP_ENTRY_LATEST_KEY)
        .map(|(_, entry)| entry)
        .collect::<Vec<_>>();
    entries.sort_by(|a, b| {
        b.admitted_at
            .cmp(&a.admitted_at)
            .then_with(|| a.map_entry_id.cmp(&b.map_entry_id))
    });
    Ok(entries)
}

pub fn load_latest_epiphany_cultmesh_repo_work_public_proof(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Option<EpiphanyCultMeshRepoWorkPublicProofEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.get(EPIPHANY_CULTMESH_REPO_WORK_PUBLIC_PROOF_LATEST_KEY)
}

pub fn load_epiphany_cultmesh_repo_work_public_proofs(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshRepoWorkPublicProofEntry>> {
    let node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut proofs = node
        .get_all_with_keys::<EpiphanyCultMeshRepoWorkPublicProofEntry>()?
        .into_iter()
        .filter(|(key, _)| key != EPIPHANY_CULTMESH_REPO_WORK_PUBLIC_PROOF_LATEST_KEY)
        .map(|(_, proof)| proof)
        .collect::<Vec<_>>();
    proofs.sort_by(|a, b| {
        b.generated_at
            .cmp(&a.generated_at)
            .then_with(|| a.public_proof_id.cmp(&b.public_proof_id))
    });
    Ok(proofs)
}

pub fn seed_epiphany_local_verse_context(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    generated_at_utc: impl Into<String>,
) -> Result<()> {
    let store_path = store_path.as_ref();
    retire_epiphany_cultmesh_legacy_provider_documents(store_path)?;
    let runtime_id = runtime_id.into();
    let generated_at_utc = generated_at_utc.into();
    let status = EpiphanyCultMeshStatusEntry {
        schema_version: EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION.to_string(),
        runtime_id: runtime_id.clone(),
        verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
        verse_tier: EPIPHANY_CULTMESH_INTERNAL_TIER.to_string(),
        app_id: "epiphany".to_string(),
        note: "Epiphany local Verse query context is typed CultMesh state; prompt assembly may read it, but Mind still owns durable adoption.".to_string(),
    };
    write_epiphany_cultmesh_status(store_path, status)?;
    write_epiphany_cultmesh_verse_policies(store_path, runtime_id.clone())?;
    write_epiphany_cultmesh_global_room_policies(store_path, runtime_id.clone())?;
    write_epiphany_cultmesh_cluster_topology(store_path, runtime_id.clone())?;
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
    write_epiphany_cultmesh_mind_contracts(store_path, runtime_id.clone())?;
    write_epiphany_cultmesh_substrate_gate_contracts(store_path, runtime_id.clone())?;
    write_epiphany_cultmesh_eyes_contracts(store_path, runtime_id.clone())?;
    write_epiphany_cultmesh_hands_contracts(store_path, runtime_id.clone())?;
    write_epiphany_cultmesh_soul_contracts(store_path, runtime_id.clone())?;
    write_epiphany_cultmesh_continuity_contracts(store_path, runtime_id.clone())?;
    write_epiphany_cultmesh_bifrost_contracts(store_path, runtime_id.clone())?;
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
    let mut verse_policies = Vec::new();
    for policy in epiphany_cultmesh_verse_policies() {
        if let Some(loaded) = node.get::<EpiphanyCultMeshVersePolicyEntry>(&policy.verse_id)? {
            verse_policies.push(loaded);
        }
    }

    let mut global_room_policies = Vec::new();
    for room in epiphany_cultmesh_global_room_policies() {
        if let Some(loaded) = node.get::<EpiphanyCultMeshGlobalRoomPolicyEntry>(&room.room_id)? {
            global_room_policies.push(loaded);
        }
    }

    let mut cluster_topology = Vec::new();
    for cluster in epiphany_cultmesh_cluster_topology() {
        if let Some(loaded) =
            node.get::<EpiphanyCultMeshClusterTopologyEntry>(&cluster.cluster_id)?
        {
            cluster_topology.push(loaded);
        }
    }

    // v0 provider documents have no provider provenance. They are sealed
    // legacy decoder vocabulary and never enter live context.
    let odin_advertisements = Vec::new();
    let eve_surface_states = Vec::new();
    let mut daemon_statuses = Vec::new();
    for cluster in epiphany_cultmesh_cluster_topology() {
        if let Some(loaded) = node.get::<EpiphanyCultMeshDaemonStatusEntry>(&cluster.daemon_id)? {
            daemon_statuses.push(loaded);
        }
    }
    let mut daemon_restart_policies = Vec::new();
    for status in &daemon_statuses {
        let key = epiphany_cultmesh_daemon_restart_policy_key(&status.daemon_id);
        if let Some(loaded) = node.get::<EpiphanyCultMeshDaemonRestartPolicyEntry>(key.as_str())? {
            daemon_restart_policies.push(loaded);
        }
    }

    let daemon_tool_capabilities = Vec::new();

    let mut contract_summaries = Vec::new();
    collect_contract_summaries(
        &node,
        epiphany_cultmesh_mind_contracts(),
        &mut contract_summaries,
    )?;
    collect_contract_summaries(
        &node,
        epiphany_cultmesh_substrate_gate_contracts(),
        &mut contract_summaries,
    )?;
    collect_contract_summaries(
        &node,
        epiphany_cultmesh_eyes_contracts(),
        &mut contract_summaries,
    )?;
    collect_contract_summaries(
        &node,
        epiphany_cultmesh_hands_contracts(),
        &mut contract_summaries,
    )?;
    collect_contract_summaries(
        &node,
        epiphany_cultmesh_soul_contracts(),
        &mut contract_summaries,
    )?;
    collect_contract_summaries(
        &node,
        epiphany_cultmesh_continuity_contracts(),
        &mut contract_summaries,
    )?;
    collect_contract_summaries(
        &node,
        epiphany_cultmesh_bifrost_contracts(),
        &mut contract_summaries,
    )?;

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
        odin_advertisements,
        eve_surface_states,
        daemon_statuses,
        latest_daemon_poke_intent: node.get(EPIPHANY_CULTMESH_DAEMON_POKE_INTENT_LATEST_KEY)?,
        latest_daemon_poke_receipt: node.get(EPIPHANY_CULTMESH_DAEMON_POKE_RECEIPT_LATEST_KEY)?,
        daemon_restart_policies,
        latest_daemon_scheduler_receipt: node
            .get(EPIPHANY_CULTMESH_DAEMON_SCHEDULER_RECEIPT_LATEST_KEY)?,
        latest_daemon_service_lifecycle_receipt: node
            .get(EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_LATEST_KEY)?,
        latest_idunn_deployment_receipt: node
            .get(EPIPHANY_CULTMESH_IDUNN_DEPLOYMENT_RECEIPT_LATEST_KEY)?,
        latest_idunn_aftercare_audit_receipt: node
            .get(EPIPHANY_CULTMESH_IDUNN_AFTERCARE_AUDIT_RECEIPT_LATEST_KEY)?,
        swarm_brake: node.get(EPIPHANY_CULTMESH_SWARM_BRAKE_KEY)?,
        latest_persona_speech_audit: node
            .get(EPIPHANY_CULTMESH_PERSONA_SPEECH_AUDIT_LATEST_KEY)?,
        latest_weksa_lowering_receipt: node
            .get(EPIPHANY_CULTMESH_WEKSA_LOWERING_RECEIPT_LATEST_KEY)?,
        daemon_tool_capabilities,
        latest_daemon_tool_invocation_intent: node
            .get(EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_INTENT_LATEST_KEY)?,
        latest_daemon_tool_invocation_receipt: node
            .get(EPIPHANY_CULTMESH_DAEMON_TOOL_INVOCATION_RECEIPT_LATEST_KEY)?,
        arrival_latest_bifrost_body_change_publication_intent: node
            .get(EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_INTENT_ARRIVAL_LATEST_KEY)?,
        arrival_latest_bifrost_body_change_publication_receipt: node
            .get(EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_RECEIPT_ARRIVAL_LATEST_KEY)?,
        arrival_latest_bifrost_github_publication_receipt: node
            .get(EPIPHANY_CULTMESH_BIFROST_GITHUB_PUBLICATION_RECEIPT_ARRIVAL_LATEST_KEY)?,
        arrival_latest_bifrost_public_proof_publication_receipt: node
            .get(EPIPHANY_CULTMESH_BIFROST_PUBLIC_PROOF_PUBLICATION_RECEIPT_ARRIVAL_LATEST_KEY)?,
        arrival_latest_bifrost_collaboration_feedback: node
            .get(EPIPHANY_CULTMESH_BIFROST_COLLABORATION_FEEDBACK_ARRIVAL_LATEST_KEY)?,
        latest_imagination_consensus_receipt: node
            .get(EPIPHANY_CULTMESH_IMAGINATION_CONSENSUS_RECEIPT_LATEST_KEY)?,
        latest_operator_snapshot: node.get(EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_LATEST_KEY)?,
        latest_operator_run_intent: node.get(EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_LATEST_KEY)?,
        latest_operator_run_receipt: node.get(EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_LATEST_KEY)?,
        latest_coordinator_run_receipt: node
            .get(EPIPHANY_CULTMESH_COORDINATOR_RUN_RECEIPT_LATEST_KEY)?,
        latest_hands_action_gate: node.get(EPIPHANY_CULTMESH_HANDS_ACTION_GATE_LATEST_KEY)?,
        latest_role_review_event: node.get(EPIPHANY_CULTMESH_ROLE_REVIEW_EVENT_LATEST_KEY)?,
        latest_work_loop_summary: node
            .get::<EpiphanyCultMeshWorkLoopTelemetryEntry>(
                EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_LATEST_KEY,
            )?
            .as_ref()
            .map(epiphany_local_verse_work_loop_summary),
        latest_agent_state_soa_summary: node
            .get(EPIPHANY_CULTMESH_AGENT_STATE_SOA_SUMMARY_LATEST_KEY)?,
        latest_repo_work_overview: node.get(EPIPHANY_CULTMESH_REPO_WORK_OVERVIEW_LATEST_KEY)?,
        latest_repo_work_map_entry: node.get(EPIPHANY_CULTMESH_REPO_WORK_MAP_ENTRY_LATEST_KEY)?,
        latest_eve_connection_intent: node
            .get(EPIPHANY_CULTMESH_EVE_CONNECTION_INTENT_LATEST_KEY)?,
        latest_eve_connection_receipt: node
            .get(EPIPHANY_CULTMESH_EVE_CONNECTION_RECEIPT_LATEST_KEY)?,
        contract_summaries,
    })
}

pub fn load_epiphany_cultmesh_daemon_liveness(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<
    Vec<(
        EpiphanyCultMeshClusterTopologyEntry,
        EpiphanyCultMeshDaemonStatusEntry,
    )>,
> {
    let store_path = store_path.as_ref();
    let runtime_id = runtime_id.into();
    let node = open_epiphany_cultmesh_node(store_path, runtime_id.clone())?;
    let mut rows = Vec::new();
    for cluster in load_epiphany_cultmesh_cluster_topology(store_path, runtime_id.clone())? {
        if let Some(status) = node.get::<EpiphanyCultMeshDaemonStatusEntry>(&cluster.daemon_id)? {
            rows.push((cluster, status));
        }
    }
    Ok(rows)
}

pub fn load_epiphany_cultmesh_daemon_restart_policy_directory(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<
    Vec<(
        EpiphanyCultMeshClusterTopologyEntry,
        Option<EpiphanyCultMeshDaemonStatusEntry>,
        Option<EpiphanyCultMeshDaemonRestartPolicyEntry>,
    )>,
> {
    let store_path = store_path.as_ref();
    let runtime_id = runtime_id.into();
    let node = open_epiphany_cultmesh_node(store_path, runtime_id.clone())?;
    let mut rows = Vec::new();
    for cluster in load_epiphany_cultmesh_cluster_topology(store_path, runtime_id.clone())? {
        let status = node.get::<EpiphanyCultMeshDaemonStatusEntry>(&cluster.daemon_id)?;
        let policy = node.get::<EpiphanyCultMeshDaemonRestartPolicyEntry>(
            &epiphany_cultmesh_daemon_restart_policy_key(&cluster.daemon_id),
        )?;
        rows.push((cluster, status, policy));
    }
    Ok(rows)
}

pub fn load_epiphany_cultmesh_eve_surface_directory(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<
    Vec<(
        EpiphanyCultMeshClusterTopologyEntry,
        EpiphanyCultMeshOdinAdvertisementEntry,
        EpiphanyCultMeshEveSurfaceStateEntry,
    )>,
> {
    let _ = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    // No provenance-bearing provider advertisement contract exists yet.
    // Topology supplies addresses, never availability.
    Ok(Vec::new())
}

pub fn load_epiphany_cultmesh_daemon_tool_directory(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<
    Vec<(
        EpiphanyCultMeshClusterTopologyEntry,
        EpiphanyCultMeshDaemonStatusEntry,
        EpiphanyCultMeshDaemonToolCapabilityEntry,
    )>,
> {
    let _ = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    // v0 capabilities have no authenticated provider owner and are ignored.
    Ok(Vec::new())
}

pub trait EpiphanyCultMeshContractSummarySource: DatabaseEntry {
    fn contract_id(&self) -> &str;
    fn verse_id(&self) -> &str;
    fn authority(&self) -> &str;
    fn document_type(&self) -> &str;
    fn operations(&self) -> &[String];
    fn receipt_document_types(&self) -> &[String];
}

macro_rules! impl_contract_summary_source {
    ($ty:ty) => {
        impl EpiphanyCultMeshContractSummarySource for $ty {
            fn contract_id(&self) -> &str {
                &self.contract_id
            }

            fn verse_id(&self) -> &str {
                &self.verse_id
            }

            fn authority(&self) -> &str {
                &self.authority
            }

            fn document_type(&self) -> &str {
                &self.document_type
            }

            fn operations(&self) -> &[String] {
                &self.operations
            }

            fn receipt_document_types(&self) -> &[String] {
                &self.receipt_document_types
            }
        }
    };
}

impl_contract_summary_source!(EpiphanyCultMeshMindContractEntry);
impl_contract_summary_source!(EpiphanyCultMeshSubstrateGateContractEntry);
impl_contract_summary_source!(EpiphanyCultMeshEyesContractEntry);
impl_contract_summary_source!(EpiphanyCultMeshHandsContractEntry);
impl_contract_summary_source!(EpiphanyCultMeshSoulContractEntry);
impl_contract_summary_source!(EpiphanyCultMeshContinuityContractEntry);
impl_contract_summary_source!(EpiphanyCultMeshBifrostContractEntry);

fn collect_contract_summaries<T>(
    node: &CultMeshNode,
    contracts: Vec<T>,
    out: &mut Vec<EpiphanyLocalVerseContractSummary>,
) -> Result<()>
where
    T: EpiphanyCultMeshContractSummarySource,
{
    for contract in contracts {
        if let Some(loaded) = node.get::<T>(contract.contract_id())? {
            out.push(EpiphanyLocalVerseContractSummary {
                contract_id: loaded.contract_id().to_string(),
                verse_id: loaded.verse_id().to_string(),
                authority: loaded.authority().to_string(),
                document_type: loaded.document_type().to_string(),
                operations: loaded.operations().to_vec(),
                receipt_document_types: loaded.receipt_document_types().to_vec(),
            });
        }
    }
    Ok(())
}

fn epiphany_cultmesh_operator_snapshot_key(snapshot_id: &str) -> String {
    format!("epiphany-local/operator-snapshot/{snapshot_id}")
}

fn epiphany_cultmesh_operator_run_intent_key(run_id: &str) -> String {
    format!("epiphany-local/operator-run-intent/{run_id}")
}

fn epiphany_cultmesh_operator_run_receipt_key(run_id: &str) -> String {
    format!("epiphany-local/operator-run-receipt/{run_id}")
}

fn epiphany_cultmesh_coordinator_run_receipt_key(receipt_id: &str) -> String {
    format!("epiphany-local/coordinator-run-receipt/{receipt_id}")
}

fn epiphany_cultmesh_hands_action_gate_key(gate_id: &str) -> String {
    format!("epiphany-local/hands-action-gate/{gate_id}")
}

fn epiphany_cultmesh_role_review_event_key(event_id: &str) -> String {
    format!("epiphany-local/role-review-event/{event_id}")
}

fn epiphany_cultmesh_persona_speech_audit_key(audit_id: &str) -> String {
    format!("epiphany-local/persona-speech-audit/{audit_id}")
}

fn epiphany_cultmesh_weksa_lowering_receipt_key(receipt_id: &str) -> String {
    format!("epiphany-local/weksa-lowering-receipt/{receipt_id}")
}

fn epiphany_cultmesh_eve_connection_intent_key(intent_id: &str) -> String {
    format!("epiphany-local/eve-connection-intent/{intent_id}")
}

#[cfg(test)]
fn epiphany_cultmesh_eve_connection_receipt_key(receipt_id: &str) -> String {
    format!("epiphany-local/eve-connection-receipt/{receipt_id}")
}

fn epiphany_cultmesh_daemon_tool_invocation_intent_key(intent_id: &str) -> String {
    format!("epiphany-local/daemon-tool-invocation-intent/{intent_id}")
}

fn epiphany_cultmesh_daemon_poke_intent_key(intent_id: &str) -> String {
    format!("epiphany-local/daemon-poke-intent/{intent_id}")
}

fn epiphany_cultmesh_daemon_poke_receipt_key(receipt_id: &str) -> String {
    format!("epiphany-local/daemon-poke-receipt/{receipt_id}")
}

fn epiphany_cultmesh_daemon_restart_policy_key(daemon_id: &str) -> String {
    format!("epiphany-local/daemon-restart-policy/{daemon_id}")
}

fn epiphany_cultmesh_daemon_scheduler_receipt_key(receipt_id: &str) -> String {
    format!("epiphany-local/daemon-scheduler-receipt/{receipt_id}")
}

fn epiphany_cultmesh_daemon_service_lifecycle_receipt_key(receipt_id: &str) -> String {
    format!("epiphany-local/daemon-service-lifecycle-receipt/{receipt_id}")
}

fn epiphany_cultmesh_daemon_service_lifecycle_receipt_latest_key(service_id: &str) -> String {
    format!("epiphany-local/daemon-service-lifecycle-receipt/latest/{service_id}")
}

fn epiphany_cultmesh_managed_service_policy_key(service_id: &str) -> String {
    format!("epiphany-local/managed-service-policy/{service_id}")
}

fn epiphany_cultmesh_idunn_deployment_receipt_key(receipt_id: &str) -> String {
    format!("gamecult-local/idunn/deployment-receipt/{receipt_id}")
}

fn epiphany_cultmesh_idunn_deployment_receipt_ref_key(receipt_ref: &str) -> String {
    let trimmed = receipt_ref.trim();
    if trimmed.is_empty() || trimmed == "latest" {
        EPIPHANY_CULTMESH_IDUNN_DEPLOYMENT_RECEIPT_LATEST_KEY.to_string()
    } else if trimmed.starts_with("gamecult-local/") {
        trimmed.to_string()
    } else {
        epiphany_cultmesh_idunn_deployment_receipt_key(trimmed)
    }
}

fn epiphany_cultmesh_idunn_aftercare_audit_receipt_key(receipt_id: &str) -> String {
    format!("gamecult-local/idunn/deployment-aftercare-audit/{receipt_id}")
}

fn epiphany_cultmesh_idunn_aftercare_audit_receipt_ref_key(receipt_ref: &str) -> String {
    let trimmed = receipt_ref.trim();
    if trimmed.is_empty() || trimmed == "latest" {
        EPIPHANY_CULTMESH_IDUNN_AFTERCARE_AUDIT_RECEIPT_LATEST_KEY.to_string()
    } else if trimmed.starts_with("gamecult-local/") {
        trimmed.to_string()
    } else {
        epiphany_cultmesh_idunn_aftercare_audit_receipt_key(trimmed)
    }
}

#[cfg(test)]
fn epiphany_cultmesh_daemon_tool_invocation_receipt_key(receipt_id: &str) -> String {
    format!("epiphany-local/daemon-tool-invocation-receipt/{receipt_id}")
}

fn epiphany_cultmesh_bifrost_body_change_publication_intent_key(intent_id: &str) -> String {
    format!("gamecult-local/bifrost/body-change-publication-intent/{intent_id}")
}

#[cfg(test)]
fn epiphany_cultmesh_bifrost_body_change_publication_receipt_key(receipt_id: &str) -> String {
    format!("gamecult-local/bifrost/body-change-publication-receipt/{receipt_id}")
}

#[cfg(test)]
fn epiphany_cultmesh_bifrost_github_publication_receipt_key(receipt_id: &str) -> String {
    format!("gamecult-local/bifrost/github-publication-receipt/{receipt_id}")
}

#[cfg(test)]
fn epiphany_cultmesh_bifrost_public_proof_publication_receipt_key(receipt_id: &str) -> String {
    format!("gamecult-local/bifrost/public-proof-publication-receipt/{receipt_id}")
}

#[cfg(test)]
fn epiphany_cultmesh_bifrost_artifact_acceptance_receipt_key(receipt_id: &str) -> String {
    format!("gamecult-local/bifrost/artifact-acceptance-receipt/{receipt_id}")
}

#[cfg(test)]
fn epiphany_cultmesh_bifrost_metrics_receipt_key(receipt_id: &str) -> String {
    format!("gamecult-local/bifrost/metrics-receipt/{receipt_id}")
}

fn epiphany_cultmesh_bifrost_collaboration_feedback_key(feedback_id: &str) -> String {
    format!("gamecult-local/bifrost/collaboration-feedback/{feedback_id}")
}

#[cfg(test)]
fn epiphany_cultmesh_imagination_consensus_receipt_key(receipt_id: &str) -> String {
    format!("gamecult-local/imagination/consensus-discovery-receipt/{receipt_id}")
}

fn pointer_text(value: &Value, pointer: &str, fallback: &str) -> String {
    value
        .pointer(pointer)
        .and_then(Value::as_str)
        .filter(|text| !text.trim().is_empty())
        .unwrap_or(fallback)
        .to_string()
}

fn pointer_string_array(value: &Value, pointer: &str) -> Result<Vec<String>> {
    let Some(items) = value.pointer(pointer) else {
        return Ok(Vec::new());
    };
    let items = items
        .as_array()
        .ok_or_else(|| anyhow!("{pointer} must be an array when present"))?;
    items
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToString::to_string)
                .ok_or_else(|| anyhow!("{pointer} must contain only strings"))
        })
        .collect()
}

pub fn epiphany_cultmesh_verse_policies() -> Vec<EpiphanyCultMeshVersePolicyEntry> {
    vec![
        EpiphanyCultMeshVersePolicyEntry {
            schema_version: EPIPHANY_CULTMESH_VERSE_POLICY_SCHEMA_VERSION.to_string(),
            verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
            tier: EPIPHANY_CULTMESH_INTERNAL_TIER.to_string(),
            purpose: "Sub-agent typed state: heartbeat, organ-state records, runtime-spine jobs, private receipts, and other Epiphany-owned organs.".to_string(),
            transport_scope: "single-host or trusted localhost mesh".to_string(),
            trust_boundary: "private Epiphany instance boundary".to_string(),
            private_state_allowed: true,
            untrusted_ingress_allowed: false,
            yggdrasil_tunnel_allowed: false,
        },
        EpiphanyCultMeshVersePolicyEntry {
            schema_version: EPIPHANY_CULTMESH_VERSE_POLICY_SCHEMA_VERSION.to_string(),
            verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
            tier: EPIPHANY_CULTMESH_LOCAL_AREA_TIER.to_string(),
            purpose: "Trusted GameCult local-area sharing across projects, including operator-approved tunnels to services on Yggdrasil.".to_string(),
            transport_scope: "LAN plus explicit GameCult tunnel endpoints".to_string(),
            trust_boundary: "trusted GameCult project/runtime boundary".to_string(),
            private_state_allowed: false,
            untrusted_ingress_allowed: false,
            yggdrasil_tunnel_allowed: true,
        },
        EpiphanyCultMeshVersePolicyEntry {
            schema_version: EPIPHANY_CULTMESH_VERSE_POLICY_SCHEMA_VERSION.to_string(),
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

pub fn write_epiphany_cultmesh_verse_policies(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshVersePolicyEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for policy in epiphany_cultmesh_verse_policies() {
        written.push(node.put(policy.verse_id.clone(), &policy)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn epiphany_cultmesh_global_room_policies() -> Vec<EpiphanyCultMeshGlobalRoomPolicyEntry> {
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
    .map(|(slug, topic, purpose)| EpiphanyCultMeshGlobalRoomPolicyEntry {
        schema_version: EPIPHANY_CULTMESH_GLOBAL_ROOM_POLICY_SCHEMA_VERSION.to_string(),
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

pub fn write_epiphany_cultmesh_global_room_policies(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshGlobalRoomPolicyEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for room in epiphany_cultmesh_global_room_policies() {
        written.push(node.put(room.room_id.clone(), &room)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn epiphany_cultmesh_cluster_topology() -> Vec<EpiphanyCultMeshClusterTopologyEntry> {
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
                body_domain: "repo:E:/Projects/EpiphanyAgent".to_string(),
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
) -> Result<Vec<EpiphanyCultMeshClusterTopologyEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for cluster in epiphany_cultmesh_cluster_topology() {
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

#[cfg(test)]
fn epiphany_cultmesh_odin_advertisement_templates() -> Vec<EpiphanyCultMeshOdinAdvertisementEntry> {
    epiphany_cultmesh_cluster_topology()
        .into_iter()
        .map(|cluster| EpiphanyCultMeshOdinAdvertisementEntry {
            schema_version: EPIPHANY_CULTMESH_ODIN_ADVERTISEMENT_SCHEMA_VERSION.to_string(),
            advertisement_id: format!("odin.advertisement.{}", cluster.cluster_id),
            cluster_id: cluster.cluster_id.clone(),
            advertised_verse_id: cluster.private_verse_id.clone(),
            body_domain: cluster.body_domain.clone(),
            body_kind: cluster.body_kind.clone(),
            daemon_surface_id: cluster.daemon_surface_id.clone(),
            eve_surface_id: cluster.eve_surface_id.clone(),
            public_summary: format!(
                "{} exposes an operator-safe Eve surface for compact CultMesh collaboration discovery.",
                cluster.display_name
            ),
            advertised_document_types: vec![
                EPIPHANY_CULTMESH_CLUSTER_TOPOLOGY_TYPE.to_string(),
                EPIPHANY_CULTMESH_ODIN_ADVERTISEMENT_TYPE.to_string(),
                EPIPHANY_CULTMESH_VERSE_POLICY_TYPE.to_string(),
            ],
            trust_boundary:
                "Odin discovery metadata is operator-safe; private Verse payloads stay behind the cluster boundary."
                    .to_string(),
            private_state_exposed: false,
            notes: vec![
                "This advertisement is discovery metadata, not membership in the private Verse.".to_string(),
                "Peers may use the Eve surface hint to request collaboration through CultMesh contracts.".to_string(),
                "Mind and Substrate Gate still review adoption, state mutation, and repo access.".to_string(),
            ],
        })
        .collect()
}

#[cfg(test)]
fn epiphany_cultmesh_eve_surface_templates() -> Vec<EpiphanyCultMeshEveSurfaceStateEntry> {
    epiphany_cultmesh_cluster_topology()
        .into_iter()
        .map(|cluster| {
            let mut exposed_document_types = vec![
                EPIPHANY_CULTMESH_ODIN_ADVERTISEMENT_TYPE.to_string(),
                EPIPHANY_CULTMESH_EVE_CONNECTION_INTENT_TYPE.to_string(),
                EPIPHANY_CULTMESH_EVE_CONNECTION_RECEIPT_TYPE.to_string(),
                EPIPHANY_CULTMESH_DAEMON_TOOL_CAPABILITY_TYPE.to_string(),
                EPIPHANY_CULTMESH_REPO_WORK_OVERVIEW_TYPE.to_string(),
            ];
            if cluster.public_persona_discussion_allowed {
                exposed_document_types
                    .push(EPIPHANY_CULTMESH_BIFROST_COLLABORATION_FEEDBACK_TYPE.to_string());
                exposed_document_types
                    .push(EPIPHANY_CULTMESH_IMAGINATION_CONSENSUS_RECEIPT_TYPE.to_string());
            }
            if cluster.role_id == "hands" {
                exposed_document_types.push(
                    EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_INTENT_TYPE.to_string(),
                );
                exposed_document_types.push(
                    EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_RECEIPT_TYPE.to_string(),
                );
                exposed_document_types
                    .push(EPIPHANY_CULTMESH_BIFROST_GITHUB_PUBLICATION_RECEIPT_TYPE.to_string());
            }
            EpiphanyCultMeshEveSurfaceStateEntry {
                schema_version: EPIPHANY_CULTMESH_EVE_SURFACE_STATE_SCHEMA_VERSION.to_string(),
                surface_id: cluster.eve_surface_id.clone(),
                cluster_id: cluster.cluster_id.clone(),
                daemon_id: cluster.daemon_id.clone(),
                body_domain: cluster.body_domain.clone(),
                tui_title: format!("{} / {}", cluster.display_name, cluster.body_domain),
                tui_rows: vec![
                    format!("cluster {}", cluster.cluster_id),
                    format!("body {}", cluster.body_domain),
                    format!("daemon {}", cluster.daemon_id),
                    format!("private {}", cluster.private_verse_id),
                    "connect via CultMesh Eve intent; private Verse payloads are sealed".to_string(),
                ],
                exposed_document_types,
                supported_actions: vec![
                    "inspectCompactSurface".to_string(),
                    "submitEveConnectionIntent".to_string(),
                    "watchTypedReceipts".to_string(),
                ],
                private_state_exposed: false,
                notes: vec![
                    "Eve surface state is compact operator-safe TUI/API state owned by the cluster daemon.".to_string(),
                    "Odin may advertise this surface id, but it must not synthesize the surface contents.".to_string(),
                    "Rows are agent-friendly hints, not private Verse state dumps.".to_string(),
                ],
            }
        })
        .collect()
}

#[cfg(test)]
fn validate_eve_surface_state(surface: &EpiphanyCultMeshEveSurfaceStateEntry) -> Result<()> {
    if surface.private_state_exposed {
        return Err(anyhow!("Eve surface states must not expose private state"));
    }
    if !surface.surface_id.starts_with("eve://") {
        return Err(anyhow!("Eve surface states require an eve:// surface id"));
    }
    if surface.tui_rows.is_empty() {
        return Err(anyhow!("Eve surface states require compact TUI rows"));
    }
    if surface.exposed_document_types.is_empty() {
        return Err(anyhow!(
            "Eve surface states require exposed document type hints"
        ));
    }
    Ok(())
}

#[cfg(test)]
pub fn epiphany_cultmesh_daemon_statuses(
    last_heartbeat_utc: impl Into<String>,
) -> Vec<EpiphanyCultMeshDaemonStatusEntry> {
    let last_heartbeat_utc = last_heartbeat_utc.into();
    epiphany_cultmesh_cluster_topology()
        .into_iter()
        .map(|cluster| EpiphanyCultMeshDaemonStatusEntry {
            schema_version: EPIPHANY_CULTMESH_DAEMON_STATUS_SCHEMA_VERSION.to_string(),
            daemon_id: cluster.daemon_id.clone(),
            cluster_id: cluster.cluster_id.clone(),
            body_domain: cluster.body_domain.clone(),
            daemon_surface_id: cluster.daemon_surface_id.clone(),
            eve_surface_id: cluster.eve_surface_id.clone(),
            status: "ready".to_string(),
            last_heartbeat_utc: last_heartbeat_utc.clone(),
            supported_actions: vec![
                "inspectStatus".to_string(),
                "pokeDaemon".to_string(),
                "watchHeartbeat".to_string(),
                "submitTypedToolIntent".to_string(),
            ],
            operator_action: "none".to_string(),
            private_state_exposed: false,
            notes: vec![
                "Daemon status is operator-safe liveness telemetry for the deployed cluster body.".to_string(),
                "A down daemon should be poked through typed operator/daemon action receipts, not by private Verse rummaging.".to_string(),
                "This status may name surfaces and actions but must not expose worker thoughts or private state.".to_string(),
            ],
        })
        .collect()
}

#[cfg(test)]
pub fn write_epiphany_cultmesh_daemon_statuses(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    last_heartbeat_utc: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshDaemonStatusEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for status in epiphany_cultmesh_daemon_statuses(last_heartbeat_utc) {
        validate_daemon_status(&status)?;
        written.push(node.put(status.daemon_id.clone(), &status)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn write_epiphany_cultmesh_daemon_status(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    status: EpiphanyCultMeshDaemonStatusEntry,
) -> Result<EpiphanyCultMeshDaemonStatusEntry> {
    validate_daemon_status(&status)?;
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let written = node.put(status.daemon_id.clone(), &status)?;
    node.flush()?;
    Ok(written)
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

fn validate_daemon_status(status: &EpiphanyCultMeshDaemonStatusEntry) -> Result<()> {
    if status.private_state_exposed {
        return Err(anyhow!("daemon statuses must not expose private state"));
    }
    if status.daemon_id.trim().is_empty() || status.cluster_id.trim().is_empty() {
        return Err(anyhow!("daemon statuses require daemon and cluster ids"));
    }
    if status.status.trim().is_empty() {
        return Err(anyhow!("daemon statuses require a status"));
    }
    if status.last_heartbeat_utc.trim().is_empty() {
        return Err(anyhow!("daemon statuses require a heartbeat timestamp"));
    }
    if status.supported_actions.is_empty() {
        return Err(anyhow!(
            "daemon statuses require supported operator actions"
        ));
    }
    Ok(())
}

#[cfg(test)]
fn epiphany_cultmesh_daemon_tool_capability_templates()
-> Vec<EpiphanyCultMeshDaemonToolCapabilityEntry> {
    let mut capabilities = Vec::new();
    for cluster in epiphany_cultmesh_cluster_topology() {
        capabilities.push(epiphany_cultmesh_daemon_tool_capability(
            &cluster,
            "status",
            "readStatus",
            EPIPHANY_CULTMESH_ODIN_ADVERTISEMENT_TYPE,
            "epiphany.cultmesh.tool_status_receipt",
            "none",
        ));
        capabilities.push(epiphany_cultmesh_daemon_tool_capability(
            &cluster,
            "eve-connect",
            "submitEveConnectionIntent",
            EPIPHANY_CULTMESH_EVE_CONNECTION_INTENT_TYPE,
            EPIPHANY_CULTMESH_EVE_CONNECTION_RECEIPT_TYPE,
            "imagination.consensus_discovery",
        ));
    }
    capabilities.push(epiphany_cultmesh_daemon_tool_capability(
        &epiphany_cultmesh_cluster_topology()
            .into_iter()
            .find(|cluster| cluster.cluster_id == "epiphany.cluster.self")
            .expect("self cluster topology exists"),
        "service-health",
        "readServiceLifecycleStatus",
        "epiphany.cultmesh.daemon_service_lifecycle_query",
        EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_TYPE,
        "daemon.service_lifecycle",
    ));
    capabilities.push(epiphany_cultmesh_daemon_tool_capability(
        &epiphany_cultmesh_cluster_topology()
            .into_iter()
            .find(|cluster| cluster.cluster_id == "epiphany.cluster.self")
            .expect("self cluster topology exists"),
        "service-policy-directory",
        "readServicePolicyDirectory",
        "epiphany.cultmesh.daemon_restart_policy_directory_query",
        EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_TYPE,
        "daemon.service_lifecycle",
    ));
    capabilities.push(epiphany_cultmesh_daemon_tool_capability(
        &epiphany_cultmesh_cluster_topology()
            .into_iter()
            .find(|cluster| cluster.cluster_id == "epiphany.cluster.self")
            .expect("self cluster topology exists"),
        "swarm-online-runbook",
        "prepareSwarmOnlineRunbook",
        "epiphany.cultmesh.daemon_service_online_runbook_request",
        EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_TYPE,
        "daemon.service_lifecycle",
    ));
    capabilities.push(epiphany_cultmesh_daemon_tool_capability(
        &epiphany_cultmesh_cluster_topology()
            .into_iter()
            .find(|cluster| cluster.cluster_id == "epiphany.cluster.hands")
            .expect("hands cluster topology exists"),
        "repo-action",
        "submitHandsActionIntent",
        "epiphany.hands.action_intent",
        "epiphany.hands.action_review",
        "hands",
    ));
    capabilities.push(epiphany_cultmesh_daemon_tool_capability(
        &epiphany_cultmesh_cluster_topology()
            .into_iter()
            .find(|cluster| cluster.cluster_id == "epiphany.cluster.soul")
            .expect("soul cluster topology exists"),
        "verify",
        "submitVerificationRequest",
        "epiphany.soul.verification_request",
        "epiphany.soul.verdict_receipt",
        "soul",
    ));
    capabilities
}

#[cfg(test)]
fn epiphany_cultmesh_daemon_tool_capability(
    cluster: &EpiphanyCultMeshClusterTopologyEntry,
    tool_slug: &str,
    operation: &str,
    input_contract_type: &str,
    receipt_contract_type: &str,
    authority_gate: &str,
) -> EpiphanyCultMeshDaemonToolCapabilityEntry {
    EpiphanyCultMeshDaemonToolCapabilityEntry {
        schema_version: EPIPHANY_CULTMESH_DAEMON_TOOL_CAPABILITY_SCHEMA_VERSION.to_string(),
        capability_id: format!("{}.tool.{tool_slug}", cluster.cluster_id),
        host_cluster_id: cluster.cluster_id.clone(),
        host_daemon_id: cluster.daemon_id.clone(),
        eve_surface_id: cluster.eve_surface_id.clone(),
        tool_name: tool_slug.to_string(),
        operation: operation.to_string(),
        input_contract_type: input_contract_type.to_string(),
        receipt_contract_type: receipt_contract_type.to_string(),
        available_to_all_agents: true,
        requires_receipt: true,
        authority_gate: authority_gate.to_string(),
        private_state_exposed: false,
        notes: vec![
            format!(
                "CultMesh advertises this daemon-hosted tool as {EPIPHANY_CULTMESH_DAEMON_TOOL_CAPABILITY_TYPE}."
            ),
            "Every agent in the local CultMesh network may discover this tool at any time.".to_string(),
            "Availability is global; execution still flows through the named typed contract and receipt gate.".to_string(),
            "The tool advertisement must not expose private Verse payloads.".to_string(),
        ],
    }
}

/// Test-only decoder fixture for proving retirement and legacy consumer
/// behavior. This is deliberately absent from production builds.
#[cfg(test)]
pub(crate) fn write_legacy_provider_fixture(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
    daemon_id: &str,
) -> Result<()> {
    let cluster = epiphany_cultmesh_cluster_topology()
        .into_iter()
        .find(|cluster| cluster.daemon_id == daemon_id)
        .context("legacy fixture daemon has no topology")?;
    let advertisement = epiphany_cultmesh_odin_advertisement_templates()
        .into_iter()
        .find(|row| row.cluster_id == cluster.cluster_id)
        .context("legacy fixture has no advertisement")?;
    let surface = epiphany_cultmesh_eve_surface_templates()
        .into_iter()
        .find(|row| row.cluster_id == cluster.cluster_id)
        .context("legacy fixture has no surface")?;
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    node.put(advertisement.advertisement_id.clone(), &advertisement)?;
    node.put(surface.surface_id.clone(), &surface)?;
    for capability in epiphany_cultmesh_daemon_tool_capability_templates()
        .into_iter()
        .filter(|row| row.host_daemon_id == daemon_id)
    {
        node.put(capability.capability_id.clone(), &capability)?;
    }
    node.flush()?;
    Ok(())
}

pub fn epiphany_cultmesh_mind_contracts() -> Vec<EpiphanyCultMeshMindContractEntry> {
    default_mind_cultnet_contracts()
        .into_iter()
        .map(|contract| EpiphanyCultMeshMindContractEntry {
            schema_version: EPIPHANY_CULTMESH_MIND_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id: contract.contract_id,
            verse_id: contract.verse_id,
            document_type: contract.document_type,
            payload_schema_version: contract.payload_schema_version,
            authority: contract.authority,
            operations: contract.operations,
            intent_document_types: contract.intent_document_types,
            receipt_document_types: contract.receipt_document_types,
            notes: contract.notes,
        })
        .collect()
}

pub fn write_epiphany_cultmesh_mind_contracts(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshMindContractEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for contract in epiphany_cultmesh_mind_contracts() {
        written.push(node.put(contract.contract_id.clone(), &contract)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn epiphany_cultmesh_substrate_gate_contracts()
-> Vec<EpiphanyCultMeshSubstrateGateContractEntry> {
    default_substrate_gate_cultnet_contracts()
        .into_iter()
        .map(|contract| EpiphanyCultMeshSubstrateGateContractEntry {
            schema_version: EPIPHANY_CULTMESH_SUBSTRATE_GATE_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id: contract.contract_id,
            verse_id: contract.verse_id,
            document_type: contract.document_type,
            payload_schema_version: contract.payload_schema_version,
            authority: contract.authority,
            operations: contract.operations,
            intent_document_types: contract.intent_document_types,
            receipt_document_types: contract.receipt_document_types,
            notes: contract.notes,
        })
        .collect()
}

pub fn write_epiphany_cultmesh_substrate_gate_contracts(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshSubstrateGateContractEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for contract in epiphany_cultmesh_substrate_gate_contracts() {
        written.push(node.put(contract.contract_id.clone(), &contract)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn epiphany_cultmesh_eyes_contracts() -> Vec<EpiphanyCultMeshEyesContractEntry> {
    default_eyes_cultnet_contracts()
        .into_iter()
        .map(|contract| EpiphanyCultMeshEyesContractEntry {
            schema_version: EPIPHANY_CULTMESH_EYES_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id: contract.contract_id,
            verse_id: contract.verse_id,
            document_type: contract.document_type,
            payload_schema_version: contract.payload_schema_version,
            authority: contract.authority,
            operations: contract.operations,
            intent_document_types: contract.intent_document_types,
            receipt_document_types: contract.receipt_document_types,
            notes: contract.notes,
        })
        .collect()
}

pub fn write_epiphany_cultmesh_eyes_contracts(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshEyesContractEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for contract in epiphany_cultmesh_eyes_contracts() {
        written.push(node.put(contract.contract_id.clone(), &contract)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn epiphany_cultmesh_hands_contracts() -> Vec<EpiphanyCultMeshHandsContractEntry> {
    default_hands_cultnet_contracts()
        .into_iter()
        .map(|contract| EpiphanyCultMeshHandsContractEntry {
            schema_version: EPIPHANY_CULTMESH_HANDS_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id: contract.contract_id,
            verse_id: contract.verse_id,
            document_type: contract.document_type,
            payload_schema_version: contract.payload_schema_version,
            authority: contract.authority,
            operations: contract.operations,
            intent_document_types: contract.intent_document_types,
            receipt_document_types: contract.receipt_document_types,
            notes: contract.notes,
        })
        .collect()
}

pub fn write_epiphany_cultmesh_hands_contracts(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshHandsContractEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for contract in epiphany_cultmesh_hands_contracts() {
        written.push(node.put(contract.contract_id.clone(), &contract)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn epiphany_cultmesh_soul_contracts() -> Vec<EpiphanyCultMeshSoulContractEntry> {
    default_soul_cultnet_contracts()
        .into_iter()
        .map(|contract| EpiphanyCultMeshSoulContractEntry {
            schema_version: EPIPHANY_CULTMESH_SOUL_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id: contract.contract_id,
            verse_id: contract.verse_id,
            document_type: contract.document_type,
            payload_schema_version: contract.payload_schema_version,
            authority: contract.authority,
            operations: contract.operations,
            intent_document_types: contract.intent_document_types,
            receipt_document_types: contract.receipt_document_types,
            notes: contract.notes,
        })
        .collect()
}

pub fn write_epiphany_cultmesh_soul_contracts(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshSoulContractEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for contract in epiphany_cultmesh_soul_contracts() {
        written.push(node.put(contract.contract_id.clone(), &contract)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn epiphany_cultmesh_continuity_contracts() -> Vec<EpiphanyCultMeshContinuityContractEntry> {
    default_continuity_cultnet_contracts()
        .into_iter()
        .map(|contract| EpiphanyCultMeshContinuityContractEntry {
            schema_version: EPIPHANY_CULTMESH_CONTINUITY_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id: contract.contract_id,
            verse_id: contract.verse_id,
            document_type: contract.document_type,
            payload_schema_version: contract.payload_schema_version,
            authority: contract.authority,
            operations: contract.operations,
            intent_document_types: contract.intent_document_types,
            receipt_document_types: contract.receipt_document_types,
            notes: contract.notes,
        })
        .collect()
}

pub fn write_epiphany_cultmesh_continuity_contracts(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshContinuityContractEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for contract in epiphany_cultmesh_continuity_contracts() {
        written.push(node.put(contract.contract_id.clone(), &contract)?);
    }
    node.flush()?;
    Ok(written)
}

pub fn epiphany_cultmesh_bifrost_contracts() -> Vec<EpiphanyCultMeshBifrostContractEntry> {
    vec![
        EpiphanyCultMeshBifrostContractEntry {
            schema_version: EPIPHANY_CULTMESH_BIFROST_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id: "gamecult.bifrost.body_change.publication".to_string(),
            verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
            document_type: EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_INTENT_TYPE
                .to_string(),
            payload_schema_version:
                EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_INTENT_SCHEMA_VERSION
                    .to_string(),
            authority: "bifrost".to_string(),
            operations: vec![
                "intentSubmit".to_string(),
                "receiptWatch".to_string(),
                "snapshot".to_string(),
            ],
            intent_document_types: vec![
                EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_INTENT_TYPE.to_string(),
            ],
            receipt_document_types: vec![
                EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_RECEIPT_TYPE.to_string(),
                EPIPHANY_CULTMESH_BIFROST_GITHUB_PUBLICATION_RECEIPT_TYPE.to_string(),
                "gamecult.bifrost.credit_receipt".to_string(),
            ],
            notes: vec![
                format!(
                    "CultMesh advertises this Bifrost contract as {EPIPHANY_CULTMESH_BIFROST_CONTRACT_TYPE}."
                ),
                "Body changes require justification, changed-path scope, verifier evidence, authorship, review, and credit metadata before GitHub publication.".to_string(),
                "Bifrost is the credit and publication-routing authority; GitHub is a publication substrate, not the governance source.".to_string(),
                "Epiphany clusters may prepare intents, but Bifrost receipts bless public publication and ledger attribution.".to_string(),
            ],
        },
        EpiphanyCultMeshBifrostContractEntry {
            schema_version: EPIPHANY_CULTMESH_BIFROST_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id: "gamecult.bifrost.collaboration.feedback".to_string(),
            verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
            document_type: EPIPHANY_CULTMESH_BIFROST_COLLABORATION_FEEDBACK_TYPE.to_string(),
            payload_schema_version:
                EPIPHANY_CULTMESH_BIFROST_COLLABORATION_FEEDBACK_SCHEMA_VERSION.to_string(),
            authority: "imaginationConsensus".to_string(),
            operations: vec![
                "recordPublicFeedback".to_string(),
                "routeToImaginationConsensus".to_string(),
                "refusePrivateState".to_string(),
                "snapshot".to_string(),
            ],
            intent_document_types: vec![
                EPIPHANY_CULTMESH_BIFROST_COLLABORATION_FEEDBACK_TYPE.to_string(),
            ],
            receipt_document_types: vec![
                EPIPHANY_CULTMESH_IMAGINATION_CONSENSUS_RECEIPT_TYPE.to_string(),
            ],
            notes: vec![
                format!(
                    "CultMesh advertises this Bifrost contract as {EPIPHANY_CULTMESH_BIFROST_CONTRACT_TYPE}."
                ),
                "Persona public collaboration feedback routes to Imagination for consensus discovery before it becomes work.".to_string(),
                "Public Persona discussion is thought weather until reviewed local adoption and Bifrost/GameCult receipts bind it to implementation.".to_string(),
            ],
        },
        EpiphanyCultMeshBifrostContractEntry {
            schema_version: EPIPHANY_CULTMESH_BIFROST_CONTRACT_SCHEMA_VERSION.to_string(),
            contract_id: "gamecult.bifrost.public_proof.publication".to_string(),
            verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
            document_type:
                EPIPHANY_CULTMESH_BIFROST_PUBLIC_PROOF_PUBLICATION_RECEIPT_TYPE.to_string(),
            payload_schema_version:
                EPIPHANY_CULTMESH_BIFROST_PUBLIC_PROOF_PUBLICATION_RECEIPT_SCHEMA_VERSION
                    .to_string(),
            authority: "bifrost".to_string(),
            operations: vec![
                "publishRedactedProof".to_string(),
                "receiptWatch".to_string(),
                "snapshot".to_string(),
            ],
            intent_document_types: vec![
                EPIPHANY_CULTMESH_REPO_WORK_PUBLIC_PROOF_TYPE.to_string(),
            ],
            receipt_document_types: vec![
                EPIPHANY_CULTMESH_BIFROST_PUBLIC_PROOF_PUBLICATION_RECEIPT_TYPE.to_string(),
                "gamecult.bifrost.credit_receipt".to_string(),
            ],
            notes: vec![
                format!(
                    "CultMesh advertises this Bifrost contract as {EPIPHANY_CULTMESH_BIFROST_CONTRACT_TYPE}."
                ),
                "Repo-work public proof bundles are redacted evidence packets, not body changes; Bifrost publishes them into public Verse rooms after review and credit receipts exist.".to_string(),
                "Downstream consumers may read the published proof closure, but Bifrost owns public publication authority and ledger attribution.".to_string(),
            ],
        },
    ]
}

pub fn write_epiphany_cultmesh_bifrost_contracts(
    store_path: impl AsRef<Path>,
    runtime_id: impl Into<String>,
) -> Result<Vec<EpiphanyCultMeshBifrostContractEntry>> {
    let mut node = open_epiphany_cultmesh_node(store_path, runtime_id)?;
    let mut written = Vec::new();
    for contract in epiphany_cultmesh_bifrost_contracts() {
        written.push(node.put(contract.contract_id.clone(), &contract)?);
    }
    node.flush()?;
    Ok(written)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cultcache_rs::CacheBackingStore;
    use cultcache_rs::CultCacheEnvelope;
    use pretty_assertions::assert_eq;

    #[test]
    fn explicit_migration_removes_only_retired_operator_status_documents() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("verse.ccmp");
        let mut backing = SingleFileMessagePackBackingStore::new(&store);
        backing.push(&CultCacheEnvelope {
            key: "epiphany-local/operator-status/latest".into(),
            r#type: "epiphany.cultmesh.operator_status".into(),
            payload: vec![0x80],
            stored_at: "2026-07-15T10:00:00Z".into(),
            schema_id: Some("epiphany.cultmesh.operator_status".into()),
        })?;
        backing.push(&CultCacheEnvelope {
            key: "epiphany-local/status".into(),
            r#type: EPIPHANY_CULTMESH_STATUS_TYPE.into(),
            payload: vec![0x80],
            stored_at: "2026-07-15T10:00:01Z".into(),
            schema_id: Some(EPIPHANY_CULTMESH_STATUS_TYPE.into()),
        })?;

        assert_eq!(
            retire_epiphany_cultmesh_operator_status_documents(&store)?,
            vec!["epiphany-local/operator-status/latest"]
        );
        let remaining = backing.pull_all()?;
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].r#type, EPIPHANY_CULTMESH_STATUS_TYPE);
        assert!(retire_epiphany_cultmesh_operator_status_documents(&store)?.is_empty());
        Ok(())
    }

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
            owner_daemon_id: "epiphany-daemon-supervisor".into(),
            command: binary.into(),
            args: vec![
                "serve",
                "--agent-store",
                "mind.ccmp",
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
            restart_mode: "always".into(),
            cooldown_seconds: 0,
            backoff_multiplier: 1,
            stdout_artifact: "projector.stdout.log".into(),
            stderr_artifact: "projector.stderr.log".into(),
            updated_at_utc: "2026-07-15T12:00:00Z".into(),
            private_state_exposed: false,
            notes: vec![],
        };
        assert!(
            write_epiphany_cultmesh_managed_service_policy(&store, "local", exact.clone()).is_err()
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
            owner_daemon_id: "epiphany-daemon-supervisor".into(),
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
            restart_mode: "always".into(),
            cooldown_seconds: 0,
            backoff_multiplier: 1,
            stdout_artifact: "workspace-projector.stdout.log".into(),
            stderr_artifact: "workspace-projector.stderr.log".into(),
            updated_at_utc: "2026-07-15T12:00:00Z".into(),
            private_state_exposed: false,
            notes: vec![],
        };
        assert!(
            write_epiphany_cultmesh_managed_service_policy(&store, "local", exact.clone()).is_err()
        );
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
        advanced.updated_at_utc = "2026-07-15T12:00:02Z".into();
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
    fn semantic_health_publication_is_monotonic_partitioned_sight_only() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let canonical = temp.path().join("canonical.msgpack");
        let verse = temp.path().join("verse.ccmp");
        let modeling_1 = semantic_health_input(&canonical, "swarm-a", "modeling", 1)?;
        let modeling_2 = semantic_health_input(&canonical, "swarm-a", "modeling", 2)?;
        let mind_1 = semantic_health_input(&canonical, "swarm-a", "mind", 1)?;
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
        publish_epiphany_cultmesh_semantic_projection_health(
            &verse,
            "runtime",
            &canonical,
            &mind_1,
            "incarnation-a",
        )?;

        assert_eq!(delayed.source_generation, 2);
        let rows = load_epiphany_cultmesh_semantic_projection_health(&verse, "runtime")?;
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].partition, "mind");
        assert_eq!(rows[1].partition, "modeling");
        assert_eq!(rows[1].source_generation, 2);
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
        let input = semantic_health_input(&canonical, "swarm-a", "mind", 1)?;
        let row = publish_epiphany_cultmesh_semantic_projection_health(
            &verse,
            "runtime",
            &canonical,
            &input,
            "incarnation-a",
        )?;
        let hostile_key = format!(
            "{}/latest",
            semantic_projection_health_scope_key("other-swarm", "mind")
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
        let input = semantic_health_input(&canonical, "swarm-a", "mind", 1)?;
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
            crate::SemanticPartition::Mind,
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

    fn publish_all_test_provider_state(store: &Path) -> Result<()> {
        for cluster in epiphany_cultmesh_cluster_topology() {
            write_legacy_provider_fixture(store, "epiphany-test", &cluster.daemon_id)?;
        }
        Ok(())
    }

    #[test]
    fn epiphany_status_round_trips_through_cultmesh() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-local.ccmp");
        let status = EpiphanyCultMeshStatusEntry {
            schema_version: EPIPHANY_CULTMESH_STATUS_SCHEMA_VERSION.to_string(),
            runtime_id: "epiphany-test".to_string(),
            verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
            verse_tier: EPIPHANY_CULTMESH_INTERNAL_TIER.to_string(),
            app_id: "epiphany".to_string(),
            note: "CultMesh is the local abstraction over CultCache and CultNet.".to_string(),
        };

        write_epiphany_cultmesh_status(&store, status.clone())?;
        assert_eq!(
            load_epiphany_cultmesh_status(&store, "epiphany-test")?,
            Some(status)
        );
        Ok(())
    }

    #[test]
    fn operator_snapshot_distills_status_json_into_typed_cultmesh_document() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-operator-snapshot.ccmp");
        let status_json = serde_json::json!({
            "threadId": "thread-test",
            "scene": {
                "scene": {
                    "stateStatus": "missing",
                    "availableActions": ["crrc", "roles"]
                }
            },
            "pressure": {
                "pressure": {
                    "level": "low"
                }
            },
            "reorient": {
                "decision": {
                    "action": "regather",
                    "nextAction": "Regather source context."
                }
            },
            "crrc": {
                "recommendation": {
                    "action": "regatherManually"
                }
            },
            "coordinator": {
                "action": "wait"
            },
            "rawResult": {
                "sealed": true
            }
        });
        let snapshot = epiphany_cultmesh_operator_snapshot_from_status_json(
            "epiphany-test",
            "snapshot-test",
            "2026-05-27T00:00:00Z",
            "status",
            ".epiphany-run/status.json",
            &status_json,
        )?;

        assert_eq!(snapshot.status, "needs-regather");
        assert_eq!(snapshot.thread_id, "thread-test");
        assert_eq!(snapshot.available_actions, vec!["crrc", "roles"]);
        assert_eq!(snapshot.artifact_refs, vec![".epiphany-run/status.json"]);

        write_epiphany_cultmesh_operator_snapshot(&store, snapshot.clone())?;
        assert_eq!(
            load_epiphany_cultmesh_operator_snapshot(&store, "epiphany-test", "snapshot-test")?,
            Some(snapshot.clone())
        );
        let mut newer = snapshot.clone();
        newer.snapshot_id = "snapshot-newer".to_string();
        newer.generated_at_utc = "2026-05-27T01:00:00Z".to_string();
        newer.coordinator_action = "continue".to_string();
        write_epiphany_cultmesh_operator_snapshot(&store, newer.clone())?;

        let mut delayed = snapshot.clone();
        delayed.snapshot_id = "snapshot-delayed".to_string();
        delayed.generated_at_utc = "2026-05-26T23:00:00Z".to_string();
        write_epiphany_cultmesh_operator_snapshot(&store, delayed)?;
        assert_eq!(
            load_latest_epiphany_cultmesh_operator_snapshot(&store, "epiphany-test")?,
            Some(newer)
        );

        let mut invalid_time = snapshot.clone();
        invalid_time.snapshot_id = "snapshot-invalid-time".to_string();
        invalid_time.generated_at_utc = "not-a-time".to_string();
        assert!(
            write_epiphany_cultmesh_operator_snapshot(&store, invalid_time)
                .unwrap_err()
                .to_string()
                .contains("invalid generated_at_utc")
        );

        let mut wrong_verse = snapshot;
        wrong_verse.snapshot_id = "snapshot-wrong-verse".to_string();
        wrong_verse.verse_id = EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string();
        assert!(
            write_epiphany_cultmesh_operator_snapshot(&store, wrong_verse)
                .unwrap_err()
                .to_string()
                .contains("internal Verse")
        );
        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        assert!(
            node.documents()
                .binding(EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_TYPE)
                .is_some()
        );
        Ok(())
    }

    #[test]
    fn service_lifecycle_receipt_history_excludes_latest_mirror() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-service-lifecycle.ccmp");
        let first = EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry {
            schema_version: EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION
                .to_string(),
            receipt_id: "service-lifecycle-first".to_string(),
            service_id: "epiphany-daemon-supervisor-service".to_string(),
            scheduler_id: "epiphany-daemon-supervisor".to_string(),
            runtime_id: "epiphany-test".to_string(),
            daemon_selector: "epiphany-daemon-supervisor".to_string(),
            action: "windows-service-execution-audit".to_string(),
            status: "incomplete".to_string(),
            command: "epiphany-daemon-supervisor".to_string(),
            args: vec!["windows-service-execution-audit".to_string()],
            cwd: Some("E:/Projects/EpiphanyAgent".to_string()),
            process_id: None,
            exit_code: Some(0),
            started_at_utc: "2026-06-18T00:00:00Z".to_string(),
            completed_at_utc: Some("2026-06-18T00:00:01Z".to_string()),
            operator_artifact_ref: "artifact://service-lifecycle/first".to_string(),
            private_state_exposed: false,
            notes: Vec::new(),
            executable_sha256: "sha256-test-projector".into(),
            preflight_witness_id: String::new(),
            required_document_types: Vec::new(),
            schema_preflight_passed: false,
            schema_catalog_sha256: String::new(),
            managed_policy_id: String::new(),
            managed_policy_digest: String::new(),
            provider_daemon_id: String::new(),
            startup_correlation_id: String::new(),
        };
        let mut second = first.clone();
        second.receipt_id = "service-lifecycle-second".to_string();
        second.status = "written".to_string();
        second.action = "windows-service-execution-runbook".to_string();
        second.started_at_utc = "2026-06-18T00:01:00Z".to_string();
        second.completed_at_utc = Some("2026-06-18T00:01:01Z".to_string());
        second.operator_artifact_ref = "artifact://service-lifecycle/second".to_string();

        write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
            &store,
            "epiphany-test",
            first.clone(),
        )?;
        write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
            &store,
            "epiphany-test",
            second.clone(),
        )?;

        let receipts =
            load_epiphany_cultmesh_daemon_service_lifecycle_receipts(&store, "epiphany-test")?;
        let mut ids = receipts
            .iter()
            .map(|receipt| receipt.receipt_id.as_str())
            .collect::<Vec<_>>();
        ids.sort_unstable();
        assert_eq!(
            ids,
            vec!["service-lifecycle-first", "service-lifecycle-second"]
        );
        assert_eq!(
            load_latest_epiphany_cultmesh_daemon_service_lifecycle_receipt(
                &store,
                "epiphany-test"
            )?,
            Some(second.clone())
        );

        let mut delayed_first = first.clone();
        delayed_first.receipt_id = "service-lifecycle-delayed-first".to_string();
        write_epiphany_cultmesh_daemon_service_lifecycle_receipt(
            &store,
            "epiphany-test",
            delayed_first,
        )?;
        assert_eq!(
            load_latest_epiphany_cultmesh_daemon_service_lifecycle_receipt(
                &store,
                "epiphany-test"
            )?,
            Some(second)
        );
        Ok(())
    }

    #[test]
    fn scheduler_latest_mirror_refuses_delayed_replay() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-scheduler-order.ccmp");
        let older = EpiphanyCultMeshDaemonSchedulerReceiptEntry {
            schema_version: EPIPHANY_CULTMESH_DAEMON_SCHEDULER_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "scheduler-older".to_string(),
            scheduler_id: "epiphany-daemon-supervisor".to_string(),
            runtime_id: "epiphany-test".to_string(),
            daemon_selector: "*".to_string(),
            iteration: 1,
            status: "completed".to_string(),
            tick_started_utc: "2026-07-13T01:00:00Z".to_string(),
            tick_completed_utc: "2026-07-13T01:00:01Z".to_string(),
            next_wake_utc: Some("2026-07-13T01:01:01Z".to_string()),
            outcome_count: 1,
            restarted_count: 0,
            refused_count: 0,
            skipped_count: 1,
            private_state_exposed: false,
            notes: Vec::new(),
        };
        let mut newer = older.clone();
        newer.receipt_id = "scheduler-newer".to_string();
        newer.iteration = 2;
        newer.tick_started_utc = "2026-07-13T02:00:00Z".to_string();
        newer.tick_completed_utc = "2026-07-13T02:00:01Z".to_string();
        newer.next_wake_utc = Some("2026-07-13T02:01:01Z".to_string());

        write_epiphany_cultmesh_daemon_scheduler_receipt(&store, "epiphany-test", newer.clone())?;
        write_epiphany_cultmesh_daemon_scheduler_receipt(&store, "epiphany-test", older)?;
        assert_eq!(
            load_latest_epiphany_cultmesh_daemon_scheduler_receipt(&store, "epiphany-test")?,
            Some(newer)
        );
        Ok(())
    }

    #[test]
    fn scheduler_receipt_refuses_impossible_time_order() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-scheduler-invalid-time.ccmp");
        let receipt = EpiphanyCultMeshDaemonSchedulerReceiptEntry {
            schema_version: EPIPHANY_CULTMESH_DAEMON_SCHEDULER_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "scheduler-invalid".to_string(),
            scheduler_id: "epiphany-daemon-supervisor".to_string(),
            runtime_id: "epiphany-test".to_string(),
            daemon_selector: "*".to_string(),
            iteration: 1,
            status: "completed".to_string(),
            tick_started_utc: "2026-07-13T02:00:00Z".to_string(),
            tick_completed_utc: "2026-07-13T01:00:00Z".to_string(),
            next_wake_utc: Some("2026-07-13T00:00:00Z".to_string()),
            outcome_count: 0,
            restarted_count: 0,
            refused_count: 0,
            skipped_count: 0,
            private_state_exposed: false,
            notes: Vec::new(),
        };
        assert!(
            write_epiphany_cultmesh_daemon_scheduler_receipt(&store, "epiphany-test", receipt,)
                .is_err()
        );
        assert!(
            load_latest_epiphany_cultmesh_daemon_scheduler_receipt(&store, "epiphany-test")?
                .is_none()
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
            load_latest_epiphany_cultmesh_daemon_service_lifecycle_receipt(
                &store,
                "epiphany-test"
            )?
            .is_none()
        );
        Ok(())
    }

    #[test]
    fn service_execution_audit_checks_expose_operator_artifact_refs() -> Result<()> {
        let report = epiphany_service_execution_audit_report(&[
            EpiphanyCultMeshDaemonServiceLifecycleReceiptEntry {
                schema_version: EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_SCHEMA_VERSION
                    .to_string(),
                receipt_id: "service-execution-runbook-receipt".to_string(),
                service_id: "epiphany-daemon-supervisor-service".to_string(),
                scheduler_id: "epiphany-daemon-supervisor".to_string(),
                runtime_id: "epiphany-test".to_string(),
                daemon_selector: "epiphany-daemon-supervisor".to_string(),
                action: "windows-service-execution-runbook".to_string(),
                status: "written".to_string(),
                command: "epiphany-daemon-supervisor".to_string(),
                args: vec!["windows-service-execution-runbook".to_string()],
                cwd: Some("E:/Projects/EpiphanyAgent".to_string()),
                process_id: None,
                exit_code: Some(0),
                started_at_utc: "2026-06-18T00:00:00Z".to_string(),
                completed_at_utc: Some("2026-06-18T00:00:01Z".to_string()),
                operator_artifact_ref: "E:/Projects/EpiphanyAgent/.epiphany-run/runbook.ps1"
                    .to_string(),
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
            },
        ]);
        let runbook_check = report
            .checks
            .iter()
            .find(|check| check.action == "windows-service-execution-runbook")
            .context("missing runbook audit check")?;
        assert!(runbook_check.ok);
        assert_eq!(
            runbook_check.operator_artifact_ref.as_deref(),
            Some("E:/Projects/EpiphanyAgent/.epiphany-run/runbook.ps1")
        );
        Ok(())
    }

    #[test]
    fn daemon_tool_invocation_mirrors_status_tools_into_local_verse() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-tools.ccmp");
        let status_json = serde_json::json!({
            "tools": {
                "runtimeStore": "state/runtime-spine.msgpack",
                "summary": {
                    "intentCount": 1,
                    "pendingCount": 0,
                    "receiptCount": 1
                },
                "invocations": [
                    {
                        "intentId": "tool-intent-test",
                        "adapter": "codex-mcp",
                        "server": "epiphany_source",
                        "toolName": "read_file",
                        "callId": "call-test",
                        "modelRequestId": "model-request-test",
                        "caller": "verification",
                        "reason": "Inspect source for Soul verdict.",
                        "createdAt": "2026-06-18T00:00:00Z",
                        "status": "ok",
                        "receiptId": "tool-receipt-test",
                        "completedAt": "2026-06-18T00:00:01Z"
                    }
                ]
            }
        });
        let (intent, receipt) = epiphany_cultmesh_daemon_tool_invocation_from_status_json(
            "epiphany-test",
            ".epiphany-run/status.json",
            &status_json,
        )?
        .expect("status should contain a tool invocation");
        let receipt = receipt.expect("completed invocation should mirror a receipt");

        assert_eq!(intent.intent_id, "tool-intent-test");
        assert_eq!(intent.tool_name, "read_file");
        assert_eq!(
            intent.payload_ref,
            ".epiphany-run/status.json#/tools/invocations/0"
        );
        assert_eq!(receipt.receipt_id, "tool-receipt-test");
        assert!(!intent.private_state_requested);
        assert!(!receipt.private_state_exposed);

        write_epiphany_cultmesh_daemon_tool_invocation_intent(
            &store,
            "epiphany-test",
            intent.clone(),
        )?;
        write_epiphany_cultmesh_daemon_tool_invocation_receipt(
            &store,
            "epiphany-test",
            receipt.clone(),
        )?;

        assert_eq!(
            load_latest_epiphany_cultmesh_daemon_tool_invocation_intent(&store, "epiphany-test")?,
            Some(intent.clone())
        );
        assert_eq!(
            load_latest_epiphany_cultmesh_daemon_tool_invocation_receipt(&store, "epiphany-test")?,
            Some(receipt)
        );
        let context = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        assert_eq!(context.latest_daemon_tool_invocation_intent, Some(intent));
        Ok(())
    }

    #[test]
    fn operator_run_intent_and_receipt_round_trip_as_native_cultmesh_documents() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-operator-run.ccmp");
        let intent = EpiphanyCultMeshOperatorRunIntentEntry {
            schema_version: EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION.to_string(),
            runtime_id: "epiphany-test".to_string(),
            verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
            run_id: "run-test".to_string(),
            requested_at_utc: "2026-05-27T00:00:00Z".to_string(),
            mode: "status".to_string(),
            root: "E:\\Projects\\EpiphanyAgent".to_string(),
            workspace: "E:\\Projects\\EpiphanyAgent".to_string(),
            thread_id: String::new(),
            codex_home: "C:\\Users\\Meta\\.codex".to_string(),
            target_dir: "C:\\Users\\Meta\\.cargo-target-codex".to_string(),
            max_steps: 4,
            timeout_seconds: 240,
            auto_review: false,
            no_ephemeral: false,
            artifact_root: ".epiphany-run/run-test".to_string(),
            dogfood_root: ".epiphany-dogfood/run-test".to_string(),
        };
        let receipt = EpiphanyCultMeshOperatorRunReceiptEntry {
            schema_version: EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION.to_string(),
            runtime_id: "epiphany-test".to_string(),
            verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
            run_id: "run-test".to_string(),
            completed_at_utc: "2026-05-27T00:00:01Z".to_string(),
            mode: "status".to_string(),
            status: "completed".to_string(),
            result_path: ".epiphany-run/run-test/status.json".to_string(),
            artifact_root: ".epiphany-run/run-test".to_string(),
            dogfood_root: ".epiphany-dogfood/run-test".to_string(),
            operator_snapshot_store: ".epiphany-run/cultmesh/operator-snapshots.ccmp".to_string(),
            operator_snapshot_id: "run-test-status".to_string(),
            artifact_refs: vec![".epiphany-run/run-test/status.json".to_string()],
            notes: vec!["receipt".to_string()],
        };

        write_epiphany_cultmesh_operator_run_intent(&store, intent.clone())?;
        write_epiphany_cultmesh_operator_run_receipt(&store, receipt.clone())?;

        assert_eq!(
            load_latest_epiphany_cultmesh_operator_run_intent(&store, "epiphany-test")?,
            Some(intent.clone())
        );
        assert_eq!(
            load_latest_epiphany_cultmesh_operator_run_receipt(&store, "epiphany-test")?,
            Some(receipt.clone())
        );

        let mut newer_intent = intent.clone();
        newer_intent.run_id = "run-newer".to_string();
        newer_intent.requested_at_utc = "2026-05-27T01:00:00Z".to_string();
        write_epiphany_cultmesh_operator_run_intent(&store, newer_intent.clone())?;
        let mut delayed_intent = intent.clone();
        delayed_intent.run_id = "run-delayed".to_string();
        delayed_intent.requested_at_utc = "2026-05-26T23:00:00Z".to_string();
        write_epiphany_cultmesh_operator_run_intent(&store, delayed_intent.clone())?;
        assert_eq!(
            load_latest_epiphany_cultmesh_operator_run_intent(&store, "epiphany-test")?,
            Some(newer_intent)
        );
        assert_eq!(
            load_epiphany_cultmesh_operator_run_intent(&store, "epiphany-test", "run-delayed")?,
            Some(delayed_intent)
        );

        let mut newer_receipt = receipt.clone();
        newer_receipt.run_id = "run-newer".to_string();
        newer_receipt.completed_at_utc = "2026-05-27T01:00:01Z".to_string();
        write_epiphany_cultmesh_operator_run_receipt(&store, newer_receipt.clone())?;
        let mut delayed_receipt = receipt.clone();
        delayed_receipt.run_id = "run-delayed".to_string();
        delayed_receipt.completed_at_utc = "2026-05-26T23:00:01Z".to_string();
        write_epiphany_cultmesh_operator_run_receipt(&store, delayed_receipt)?;
        assert_eq!(
            load_latest_epiphany_cultmesh_operator_run_receipt(&store, "epiphany-test")?,
            Some(newer_receipt)
        );

        let mut invalid_intent = intent.clone();
        invalid_intent.run_id = "run-invalid".to_string();
        invalid_intent.requested_at_utc = "not-a-time".to_string();
        assert!(
            write_epiphany_cultmesh_operator_run_intent(&store, invalid_intent)
                .unwrap_err()
                .to_string()
                .contains("invalid requested_at_utc")
        );
        let mut wrong_verse_receipt = receipt;
        wrong_verse_receipt.run_id = "run-wrong-verse".to_string();
        wrong_verse_receipt.verse_id = EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string();
        assert!(
            write_epiphany_cultmesh_operator_run_receipt(&store, wrong_verse_receipt)
                .unwrap_err()
                .to_string()
                .contains("internal Verse")
        );
        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        assert!(
            node.documents()
                .binding(EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_TYPE)
                .is_some()
        );
        assert!(
            node.documents()
                .binding(EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_TYPE)
                .is_some()
        );
        Ok(())
    }

    #[test]
    fn coordinator_run_receipt_mirrors_summary_into_local_verse() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-coordinator-run.ccmp");
        let summary_json = serde_json::json!({
            "threadId": "thread-test",
            "mode": "run",
            "steps": [{"index": 0}, {"index": 1}],
            "finalAction": {
                "action": "launchModeling",
                "reason": "continue bounded work"
            },
            "coordinatorRunReceipt": {
                "documentType": "epiphany.coordinator_run_receipt.v0",
                "receiptId": "runtime-coordinator-receipt-test",
                "store": "state/runtime-spine.msgpack"
            },
            "artifactManifest": [
                "coordinator-summary.json",
                "coordinator-steps.jsonl"
            ],
            "sealedArtifactManifest": [
                {
                    "path": "epiphany-transcript.jsonl",
                    "reason": "sealed"
                }
            ]
        });
        let receipt = epiphany_cultmesh_coordinator_run_receipt_from_summary_json(
            "epiphany-test",
            "coordinator-cultmesh-test",
            "2026-06-18T00:00:00Z",
            ".epiphany-dogfood/coordinator",
            &summary_json,
        )?;

        assert_eq!(
            receipt.source_receipt_id,
            "runtime-coordinator-receipt-test"
        );
        assert_eq!(receipt.final_action, "launchModeling");
        assert_eq!(receipt.step_count, 2);
        assert_eq!(
            receipt.sealed_artifact_refs,
            vec!["epiphany-transcript.jsonl"]
        );
        assert!(!receipt.private_state_exposed);

        write_epiphany_cultmesh_coordinator_run_receipt(&store, receipt.clone())?;

        assert_eq!(
            load_latest_epiphany_cultmesh_coordinator_run_receipt(&store, "epiphany-test")?,
            Some(receipt.clone())
        );
        let context = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        assert_eq!(
            context.latest_coordinator_run_receipt,
            Some(receipt.clone())
        );
        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        assert!(
            node.documents()
                .binding(EPIPHANY_CULTMESH_COORDINATOR_RUN_RECEIPT_TYPE)
                .is_some()
        );

        let mut newer = receipt.clone();
        newer.receipt_id = "coordinator-cultmesh-newer".to_string();
        newer.source_receipt_id = "runtime-coordinator-receipt-newer".to_string();
        newer.final_action = "continueImplementation".to_string();
        newer.status = "continueImplementation".to_string();
        newer.created_at_utc = "2026-06-18T01:00:00Z".to_string();
        write_epiphany_cultmesh_coordinator_run_receipt(&store, newer.clone())?;

        let mut delayed = receipt.clone();
        delayed.receipt_id = "coordinator-cultmesh-delayed".to_string();
        delayed.source_receipt_id = "runtime-coordinator-receipt-delayed".to_string();
        delayed.final_action = "launchResearch".to_string();
        delayed.status = "launchResearch".to_string();
        delayed.created_at_utc = "2026-06-17T23:00:00Z".to_string();
        write_epiphany_cultmesh_coordinator_run_receipt(&store, delayed)?;
        assert_eq!(
            load_latest_epiphany_cultmesh_coordinator_run_receipt(&store, "epiphany-test")?,
            Some(newer)
        );

        let mut invalid_time = receipt.clone();
        invalid_time.receipt_id = "coordinator-cultmesh-invalid".to_string();
        invalid_time.source_receipt_id = "runtime-coordinator-receipt-invalid".to_string();
        invalid_time.created_at_utc = "not-a-time".to_string();
        let err = write_epiphany_cultmesh_coordinator_run_receipt(&store, invalid_time)
            .expect_err("invalid coordinator run time must be refused");
        assert!(err.to_string().contains("invalid created at"));

        let mut leaked = receipt;
        leaked.private_state_exposed = true;
        let err = write_epiphany_cultmesh_coordinator_run_receipt(&store, leaked)
            .expect_err("private coordinator receipt mirror must be refused");
        assert!(err.to_string().contains("must not expose private state"));
        Ok(())
    }

    #[test]
    fn hands_action_gate_mirrors_summary_into_local_verse() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-hands-gate.ccmp");
        let summary_json = serde_json::json!({
            "threadId": "thread-test",
            "mode": "run",
            "coordinatorRunReceipt": {
                "receiptId": "runtime-coordinator-receipt-test"
            },
            "finalAction": {
                "action": "continueImplementation",
                "handsActionGate": {
                    "status": "ready",
                    "runtimeJobId": "hands-job-test",
                    "substrateGateGrantReceiptId": "substrate-grant-test",
                    "intentId": "hands-intent-test",
                    "reviewId": "hands-review-test",
                    "requestedPaths": ["epiphany-core/src/cultmesh_integration.rs"],
                    "requiredReceipts": [
                        "epiphany.hands.patch_receipt.v0",
                        "epiphany.hands.command_receipt.v0",
                        "epiphany.hands.commit_receipt.v0"
                    ],
                    "recordPassCommand": {
                        "executable": "epiphany-hands-action",
                        "args": [
                            "--store",
                            "state/runtime-spine.msgpack",
                            "record-pass",
                            "--gate-summary",
                            ".epiphany-dogfood/coordinator/coordinator-summary.json"
                        ]
                    }
                }
            }
        });
        let gate = epiphany_cultmesh_hands_action_gate_from_summary_json(
            "epiphany-test",
            "2026-06-18T00:00:00Z",
            ".epiphany-dogfood/coordinator/coordinator-summary.json",
            &summary_json,
        )?
        .expect("summary should contain a Hands action gate");

        assert_eq!(gate.gate_id, "hands-intent-test:hands-review-test");
        assert_eq!(
            gate.source_coordinator_receipt_id,
            "runtime-coordinator-receipt-test"
        );
        assert_eq!(gate.record_pass_executable, "epiphany-hands-action");
        assert!(!gate.private_state_exposed);

        write_epiphany_cultmesh_hands_action_gate(&store, gate.clone())?;

        assert_eq!(
            load_latest_epiphany_cultmesh_hands_action_gate(&store, "epiphany-test")?,
            Some(gate.clone())
        );
        let context = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        assert_eq!(context.latest_hands_action_gate, Some(gate.clone()));
        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        assert!(
            node.documents()
                .binding(EPIPHANY_CULTMESH_HANDS_ACTION_GATE_TYPE)
                .is_some()
        );

        let mut newer = gate.clone();
        newer.gate_id = "hands-intent-newer:hands-review-newer".to_string();
        newer.hands_intent_id = "hands-intent-newer".to_string();
        newer.hands_review_id = "hands-review-newer".to_string();
        newer.created_at_utc = "2026-06-18T01:00:00Z".to_string();
        write_epiphany_cultmesh_hands_action_gate(&store, newer.clone())?;

        let mut delayed = gate.clone();
        delayed.gate_id = "hands-intent-delayed:hands-review-delayed".to_string();
        delayed.hands_intent_id = "hands-intent-delayed".to_string();
        delayed.hands_review_id = "hands-review-delayed".to_string();
        delayed.created_at_utc = "2026-06-17T23:00:00Z".to_string();
        write_epiphany_cultmesh_hands_action_gate(&store, delayed)?;
        assert_eq!(
            load_latest_epiphany_cultmesh_hands_action_gate(&store, "epiphany-test")?,
            Some(newer)
        );

        let mut invalid_time = gate.clone();
        invalid_time.gate_id = "hands-intent-invalid:hands-review-invalid".to_string();
        invalid_time.hands_intent_id = "hands-intent-invalid".to_string();
        invalid_time.hands_review_id = "hands-review-invalid".to_string();
        invalid_time.created_at_utc = "not-a-time".to_string();
        let err = write_epiphany_cultmesh_hands_action_gate(&store, invalid_time)
            .expect_err("invalid Hands gate time must be refused");
        assert!(err.to_string().contains("invalid created at"));

        let mut leaked = gate;
        leaked.private_state_exposed = true;
        let err = write_epiphany_cultmesh_hands_action_gate(&store, leaked)
            .expect_err("private Hands action gate mirror must be refused");
        assert!(err.to_string().contains("must not expose private state"));
        Ok(())
    }

    #[test]
    fn role_review_event_mirrors_summary_into_local_verse() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-role-review.ccmp");
        let summary_json = serde_json::json!({
            "threadId": "thread-test",
            "mode": "run",
            "coordinatorRunReceipt": {
                "receiptId": "runtime-coordinator-receipt-test"
            },
            "steps": [
                {
                    "events": [
                        {
                            "type": "roleFailureReview",
                            "roleId": "verification",
                            "superseded": {
                                "patch": {
                                    "acceptanceReceipts": [
                                        {
                                            "id": "role-failure-review-test",
                                            "result_id": "result-verification-test",
                                            "job_id": "job-verification-test",
                                            "binding_id": "verification-review-worker",
                                            "surface": "roleFailureReview",
                                            "role_id": "verification",
                                            "status": "superseded",
                                            "accepted_at": "2026-06-18T00:00:00Z",
                                            "summary": "Old failed Soul result reviewed before relaunch."
                                        }
                                    ]
                                }
                            }
                        }
                    ]
                }
            ]
        });
        let event = epiphany_cultmesh_role_review_event_from_summary_json(
            "epiphany-test",
            "2026-06-18T00:00:01Z",
            ".epiphany-dogfood/coordinator/coordinator-summary.json",
            &summary_json,
        )?
        .expect("summary should contain a role review event");

        assert_eq!(event.surface, "roleFailureReview");
        assert_eq!(event.role_id, "verification");
        assert_eq!(event.review_status, "superseded");
        assert_eq!(event.acceptance_receipt_id, "role-failure-review-test");
        assert_eq!(event.runtime_result_id, "result-verification-test");
        assert!(!event.private_state_exposed);

        write_epiphany_cultmesh_role_review_event(&store, event.clone())?;
        assert_eq!(
            load_latest_epiphany_cultmesh_role_review_event(&store, "epiphany-test")?,
            Some(event.clone())
        );
        let context = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        assert_eq!(context.latest_role_review_event, Some(event.clone()));
        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        assert!(
            node.documents()
                .binding(EPIPHANY_CULTMESH_ROLE_REVIEW_EVENT_TYPE)
                .is_some()
        );

        let mut newer = event.clone();
        newer.event_id = "roleAccept:modeling:accepted".to_string();
        newer.surface = "roleAccept".to_string();
        newer.role_id = "modeling".to_string();
        newer.review_status = "accepted".to_string();
        newer.created_at_utc = "2026-06-18T01:00:00Z".to_string();
        write_epiphany_cultmesh_role_review_event(&store, newer.clone())?;

        let mut delayed = event.clone();
        delayed.event_id = "roleAccept:research:accepted".to_string();
        delayed.surface = "roleAccept".to_string();
        delayed.role_id = "research".to_string();
        delayed.review_status = "accepted".to_string();
        delayed.created_at_utc = "2026-06-17T23:00:00Z".to_string();
        write_epiphany_cultmesh_role_review_event(&store, delayed)?;
        assert_eq!(
            load_latest_epiphany_cultmesh_role_review_event(&store, "epiphany-test")?,
            Some(newer)
        );

        let mut invalid_time = event.clone();
        invalid_time.event_id = "roleAccept:invalid:accepted".to_string();
        invalid_time.surface = "roleAccept".to_string();
        invalid_time.role_id = "invalid".to_string();
        invalid_time.review_status = "accepted".to_string();
        invalid_time.created_at_utc = "not-a-time".to_string();
        let err = write_epiphany_cultmesh_role_review_event(&store, invalid_time)
            .expect_err("invalid role review time must be refused");
        assert!(err.to_string().contains("invalid created at"));

        let mut leaked = event;
        leaked.private_state_exposed = true;
        let err = write_epiphany_cultmesh_role_review_event(&store, leaked)
            .expect_err("private role review mirror must be refused");
        assert!(err.to_string().contains("must not expose private state"));
        Ok(())
    }

    #[test]
    fn work_loop_telemetry_round_trips_as_internal_cultmesh_document() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-local-verse.ccmp");
        let telemetry = EpiphanyCultMeshWorkLoopTelemetryEntry {
            schema_version: EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_SCHEMA_VERSION.to_string(),
            runtime_id: "epiphany-test".to_string(),
            verse_id: EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string(),
            telemetry_id: "work-loop-telemetry-test".to_string(),
            thread_id: "thread-test".to_string(),
            produced_at_utc: "2026-06-12T00:00:00Z".to_string(),
            source_stage: "hands".to_string(),
            target_stages: vec!["soul".to_string(), "Modeling".to_string()],
            lower_bound_receipt_at: "2026-06-11T23:59:59Z".to_string(),
            hands_intent_id: "hands-intent-test".to_string(),
            hands_review_id: "hands-review-test".to_string(),
            hands_runtime_job_id: "hands-job-test".to_string(),
            substrate_gate_grant_receipt_id: "substrate-grant-test".to_string(),
            hands_patch_receipt_id: "hands-patch-test".to_string(),
            hands_command_receipt_id: "hands-command-test".to_string(),
            hands_commit_receipt_id: "hands-commit-test".to_string(),
            command: "cargo test".to_string(),
            exit_code: "0".to_string(),
            stdout_artifact: "stdout.log".to_string(),
            stderr_artifact: "stderr.log".to_string(),
            commit_sha: "abc123".to_string(),
            branch: "codex/test".to_string(),
            changed_paths: vec!["epiphany-core/src/runtime_spine.rs".to_string()],
            artifact_previews: vec!["stdout: ok".to_string()],
            source_refs: vec!["epiphany-core/src/runtime_spine.rs".to_string()],
            source_path_proof: vec!["runtime-spine persisted Hands receipts".to_string()],
            soul_receipt_ids: vec!["accept-verification-test".to_string()],
            summary: "Hands consequence telemetry for the work loop.".to_string(),
            receipt_payload_previews: vec!["patch receipt body".to_string()],
            commit_diff_preview: "diff --git a/file b/file".to_string(),
            verification_assertions: vec!["test asserts CultMesh round trip".to_string()],
        };

        write_epiphany_cultmesh_work_loop_telemetry(&store, telemetry.clone())?;

        assert_eq!(
            load_latest_epiphany_cultmesh_work_loop_telemetry(&store, "epiphany-test")?,
            Some(telemetry.clone())
        );
        let context = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        let summary = context
            .latest_work_loop_summary
            .expect("local Verse context should expose a sealed work-loop digest");
        assert_eq!(summary.telemetry_id, telemetry.telemetry_id);
        assert_eq!(summary.hands_patch_receipt_id, "hands-patch-test");
        assert_eq!(summary.hands_command_receipt_id, "hands-command-test");
        assert_eq!(summary.hands_commit_receipt_id, "hands-commit-test");
        assert_eq!(summary.changed_path_count, 1);
        assert_eq!(summary.source_ref_count, 1);
        assert_eq!(summary.verification_assertion_count, 1);
        let serialized_summary = serde_json::to_string(&summary)?;
        assert!(!serialized_summary.contains("patch receipt body"));
        assert!(!serialized_summary.contains("diff --git"));
        assert!(!serialized_summary.contains("stdout: ok"));
        assert!(serialized_summary.contains("exposes only this digest"));
        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        assert!(
            node.documents()
                .binding(EPIPHANY_CULTMESH_WORK_LOOP_TELEMETRY_TYPE)
                .is_some()
        );

        let mut newer = telemetry.clone();
        newer.telemetry_id = "work-loop-telemetry-newer".to_string();
        newer.produced_at_utc = "2026-06-12T01:00:00Z".to_string();
        write_epiphany_cultmesh_work_loop_telemetry(&store, newer.clone())?;

        let mut delayed = telemetry.clone();
        delayed.telemetry_id = "work-loop-telemetry-delayed".to_string();
        delayed.produced_at_utc = "2026-06-11T23:30:00Z".to_string();
        delayed.lower_bound_receipt_at = "2026-06-11T23:00:00Z".to_string();
        write_epiphany_cultmesh_work_loop_telemetry(&store, delayed)?;
        assert_eq!(
            load_latest_epiphany_cultmesh_work_loop_telemetry(&store, "epiphany-test")?,
            Some(newer)
        );

        let mut future_bound = telemetry.clone();
        future_bound.telemetry_id = "work-loop-telemetry-future-bound".to_string();
        future_bound.lower_bound_receipt_at = "2026-06-12T00:00:01Z".to_string();
        let err = write_epiphany_cultmesh_work_loop_telemetry(&store, future_bound)
            .expect_err("future receipt lower bound must be refused");
        assert!(err.to_string().contains("after packet production"));

        let mut wrong_verse = telemetry;
        wrong_verse.telemetry_id = "work-loop-telemetry-wrong-verse".to_string();
        wrong_verse.verse_id = EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string();
        let err = write_epiphany_cultmesh_work_loop_telemetry(&store, wrong_verse)
            .expect_err("work-loop evidence outside internal Verse must be refused");
        assert!(err.to_string().contains("internal Verse"));
        Ok(())
    }

    #[test]
    fn agent_state_soa_summary_mirrors_agent_store_shape_without_memory_text() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-local-verse.ccmp");
        let soa = EpiphanyAgentStateSoaEntry {
            schema_version: "epiphany.agent_state_soa.v0".to_string(),
            generated_at: "2026-06-18T00:00:00Z".to_string(),
            source_store: "state/agents.msgpack".to_string(),
            role_ids: vec!["Persona".to_string(), "implementation".to_string()],
            agent_ids: vec!["epiphany.Persona".to_string(), "epiphany.hands".to_string()],
            display_names: vec!["Persona".to_string(), "Hands".to_string()],
            profile_kinds: vec!["Persona".to_string(), "WorkOrgan".to_string()],
            portable_contracts: vec![
                "gamecult.persona_state.v0".to_string(),
                "epiphany.work_organ_state.v0".to_string(),
            ],
            semantic_memory_counts: vec![3, 2],
            episodic_memory_counts: vec![1, 0],
            relationship_memory_counts: vec![2, 0],
            goal_counts: vec![1, 1],
            value_counts: vec![4, 3],
        };
        let summary = epiphany_cultmesh_agent_state_soa_summary_from_entry(
            "epiphany-test",
            "agent-state-soa-summary-test",
            &soa,
        );
        write_epiphany_cultmesh_agent_state_soa_summary(&store, summary.clone())?;
        assert_eq!(
            load_latest_epiphany_cultmesh_agent_state_soa_summary(&store, "epiphany-test")?,
            Some(summary.clone())
        );
        let context = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        assert_eq!(
            context.latest_agent_state_soa_summary,
            Some(summary.clone())
        );
        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        assert!(
            node.documents()
                .binding(EPIPHANY_CULTMESH_AGENT_STATE_SOA_SUMMARY_TYPE)
                .is_some()
        );
        let summary_table = node.soa::<EpiphanyCultMeshAgentStateSoaSummaryEntry>()?;
        assert_eq!(summary_table.len(), 2);
        assert!(
            summary_table
                .column::<String>("summaryId")?
                .values()
                .contains(&"agent-state-soa-summary-test".to_string())
        );
        assert!(
            summary_table
                .column::<u32>("rowCount")?
                .values()
                .contains(&2)
        );
        assert!(
            summary_table
                .column::<bool>("privateStateExposed")?
                .values()
                .iter()
                .all(|exposed| !exposed)
        );
        assert!(
            summary_table
                .column::<Vec<String>>("portableContracts")?
                .values()
                .iter()
                .any(|contracts| contracts.contains(&"gamecult.persona_state.v0".to_string()))
        );
        let serialized = serde_json::to_string(&summary)?;
        assert!(serialized.contains("gamecult.persona_state.v0"));
        assert!(!serialized.contains("privateNotes"));
        assert!(!summary.private_state_exposed);

        let mut newer = summary.clone();
        newer.summary_id = "agent-state-soa-summary-newer".to_string();
        newer.generated_at = "2026-06-18T01:00:00Z".to_string();
        write_epiphany_cultmesh_agent_state_soa_summary(&store, newer.clone())?;

        let mut delayed = summary.clone();
        delayed.summary_id = "agent-state-soa-summary-delayed".to_string();
        delayed.generated_at = "2026-06-17T23:00:00Z".to_string();
        write_epiphany_cultmesh_agent_state_soa_summary(&store, delayed)?;
        assert_eq!(
            load_latest_epiphany_cultmesh_agent_state_soa_summary(&store, "epiphany-test")?,
            Some(newer)
        );

        let mut invalid_time = summary.clone();
        invalid_time.summary_id = "agent-state-soa-summary-invalid".to_string();
        invalid_time.generated_at = "not-a-time".to_string();
        let err = write_epiphany_cultmesh_agent_state_soa_summary(&store, invalid_time)
            .expect_err("invalid agent summary time must be refused");
        assert!(err.to_string().contains("invalid generated_at"));

        let mut wrong_verse = summary.clone();
        wrong_verse.summary_id = "agent-state-soa-summary-wrong-verse".to_string();
        wrong_verse.verse_id = EPIPHANY_CULTMESH_INTERNAL_VERSE_ID.to_string();
        let err = write_epiphany_cultmesh_agent_state_soa_summary(&store, wrong_verse)
            .expect_err("agent summary outside local-area Verse must be refused");
        assert!(err.to_string().contains("local-area Verse"));

        let mut leaked = summary.clone();
        leaked.private_state_exposed = true;
        let err = write_epiphany_cultmesh_agent_state_soa_summary(&store, leaked)
            .expect_err("private-state agent summaries must be refused");
        assert!(err.to_string().contains("must not expose private state"));

        let mut drifted = summary;
        drifted.agent_ids.pop();
        let err = write_epiphany_cultmesh_agent_state_soa_summary(&store, drifted)
            .expect_err("column length drift must be refused");
        assert!(err.to_string().contains("column agentIds has length"));
        Ok(())
    }

    #[test]
    fn builtin_verse_policies_keep_public_and_private_boundaries_apart() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-verses.ccmp");
        let written = write_epiphany_cultmesh_verse_policies(&store, "epiphany-test")?;
        assert_eq!(written.len(), 3);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let internal = node.get_required::<EpiphanyCultMeshVersePolicyEntry>(
            EPIPHANY_CULTMESH_INTERNAL_VERSE_ID,
        )?;
        let local_area = node.get_required::<EpiphanyCultMeshVersePolicyEntry>(
            EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID,
        )?;
        let global = node
            .get_required::<EpiphanyCultMeshVersePolicyEntry>(EPIPHANY_CULTMESH_GLOBAL_VERSE_ID)?;

        assert!(internal.private_state_allowed);
        assert!(!internal.untrusted_ingress_allowed);
        assert!(!local_area.private_state_allowed);
        assert!(local_area.yggdrasil_tunnel_allowed);
        assert!(!global.private_state_allowed);
        assert!(global.untrusted_ingress_allowed);
        Ok(())
    }

    #[test]
    fn global_room_policies_make_public_threaded_rooms_for_personas() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-global-rooms.ccmp");
        let written = write_epiphany_cultmesh_global_room_policies(&store, "epiphany-test")?;
        assert!(written.len() >= 5);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let dreams =
            node.get_required::<EpiphanyCultMeshGlobalRoomPolicyEntry>("epiphany-global/dreams")?;
        let architecture = node.get_required::<EpiphanyCultMeshGlobalRoomPolicyEntry>(
            "epiphany-global/architecture",
        )?;

        assert_eq!(dreams.verse_id, EPIPHANY_CULTMESH_GLOBAL_VERSE_ID);
        assert!(dreams.threaded);
        assert!(dreams.persona_posting_allowed);
        assert!(dreams.untrusted_ingress_allowed);
        assert!(architecture.purpose.contains("ownership"));
        Ok(())
    }

    #[test]
    fn cluster_topology_names_private_verses_body_daemons_and_eve_surfaces() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-cluster-topology.ccmp");
        let written = write_epiphany_cultmesh_cluster_topology(&store, "epiphany-test")?;
        assert_eq!(written.len(), 7);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let persona =
            node.get_required::<EpiphanyCultMeshClusterTopologyEntry>("epiphany.cluster.persona")?;
        let hands =
            node.get_required::<EpiphanyCultMeshClusterTopologyEntry>("epiphany.cluster.hands")?;

        assert_eq!(persona.private_verse_id, "epiphany.cluster.persona.private");
        assert_eq!(persona.body_domain, "repo:E:/Projects/EpiphanyAgent");
        assert_eq!(persona.daemon_id, "epiphany-daemon-persona");
        assert_eq!(persona.eve_surface_id, "eve://epiphany/persona");
        assert!(persona.public_persona_discussion_allowed);
        assert!(!hands.public_persona_discussion_allowed);
        assert!(
            hands
                .notes
                .iter()
                .any(|note| note.contains("Odin may advertise compact metadata"))
        );
        Ok(())
    }

    #[test]
    fn odin_advertisements_expose_eve_metadata_without_private_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-odin-advertisements.ccmp");
        publish_all_test_provider_state(&store)?;

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let persona = node.get_required::<EpiphanyCultMeshOdinAdvertisementEntry>(
            "odin.advertisement.epiphany.cluster.persona",
        )?;

        assert_eq!(persona.cluster_id, "epiphany.cluster.persona");
        assert_eq!(
            persona.advertised_verse_id,
            "epiphany.cluster.persona.private"
        );
        assert_eq!(persona.eve_surface_id, "eve://epiphany/persona");
        assert!(!persona.private_state_exposed);
        assert!(
            persona
                .advertised_document_types
                .iter()
                .any(|document_type| document_type == EPIPHANY_CULTMESH_ODIN_ADVERTISEMENT_TYPE)
        );
        assert!(
            persona
                .notes
                .iter()
                .any(|note| note.contains("discovery metadata"))
        );
        Ok(())
    }

    #[test]
    fn legacy_provider_rows_are_not_live_discovery_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-eve-surface-states.ccmp");
        write_epiphany_cultmesh_cluster_topology(&store, "epiphany-test")?;
        publish_all_test_provider_state(&store)?;
        let context = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        assert!(context.odin_advertisements.is_empty());
        assert!(context.eve_surface_states.is_empty());
        assert!(context.daemon_tool_capabilities.is_empty());
        assert!(load_epiphany_cultmesh_eve_surface_directory(&store, "epiphany-test")?.is_empty());
        assert!(load_epiphany_cultmesh_daemon_tool_directory(&store, "epiphany-test")?.is_empty());
        Ok(())
    }

    #[test]
    fn eve_surface_state_refuses_private_state_exposure() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-eve-surface-private.ccmp");
        let mut surface = epiphany_cultmesh_eve_surface_templates()
            .into_iter()
            .find(|surface| surface.surface_id == "eve://epiphany/persona")
            .expect("persona surface exists");
        surface.private_state_exposed = true;

        let mut node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let error = validate_eve_surface_state(&surface)
            .expect_err("private Eve surface states must be refused");
        assert!(error.to_string().contains("private state"));
        surface.private_state_exposed = false;
        node.put(surface.surface_id.clone(), &surface)?;
        node.flush()?;
        Ok(())
    }

    #[test]
    fn daemon_statuses_cover_every_cluster_without_private_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-daemon-statuses.ccmp");
        write_epiphany_cultmesh_cluster_topology(&store, "epiphany-test")?;
        let statuses = write_epiphany_cultmesh_daemon_statuses(
            &store,
            "epiphany-test",
            "2026-06-17T00:00:00Z",
        )?;

        assert_eq!(statuses.len(), epiphany_cultmesh_cluster_topology().len());
        let hands = statuses
            .iter()
            .find(|status| status.daemon_id == "epiphany-daemon-hands")
            .expect("Hands daemon status exists");
        assert_eq!(hands.cluster_id, "epiphany.cluster.hands");
        assert_eq!(hands.status, "ready");
        assert!(!hands.private_state_exposed);
        assert!(
            hands
                .supported_actions
                .iter()
                .any(|action| action == "pokeDaemon")
        );
        assert!(
            hands
                .supported_actions
                .iter()
                .any(|action| action == "watchHeartbeat")
        );

        let context = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        assert_eq!(
            context.daemon_statuses.len(),
            context.cluster_topology.len()
        );
        assert!(context.daemon_statuses.iter().all(|status| {
            !status.private_state_exposed && !status.last_heartbeat_utc.is_empty()
        }));
        Ok(())
    }

    #[test]
    fn declared_daemon_targets_do_not_materialize_observed_liveness() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-declared-versus-observed.ccmp");
        let topology = write_epiphany_cultmesh_cluster_topology(&store, "epiphany-test")?;

        assert_eq!(topology.len(), 7);
        assert!(load_epiphany_cultmesh_daemon_liveness(&store, "epiphany-test")?.is_empty());

        let observed = epiphany_cultmesh_daemon_statuses("2026-07-15T00:00:00Z")
            .into_iter()
            .next()
            .expect("declared topology has a matching daemon status fixture");
        let observed_daemon_id = observed.daemon_id.clone();
        write_epiphany_cultmesh_daemon_status(&store, "epiphany-test", observed)?;

        let liveness = load_epiphany_cultmesh_daemon_liveness(&store, "epiphany-test")?;
        assert_eq!(liveness.len(), 1);
        assert_eq!(liveness[0].1.daemon_id, observed_daemon_id);
        Ok(())
    }

    #[test]
    fn diagnostic_loaders_do_not_materialize_missing_body_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let missing_parent = temp.path().join("missing-body");
        let store = missing_parent.join("missing-local-verse.ccmp");

        assert_eq!(
            load_epiphany_cultmesh_status(&store, "epiphany-test")?,
            None
        );
        assert!(load_epiphany_cultmesh_cluster_topology(&store, "epiphany-test")?.is_empty());
        assert!(load_epiphany_cultmesh_daemon_liveness(&store, "epiphany-test")?.is_empty());
        assert!(
            load_epiphany_cultmesh_daemon_restart_policy_directory(&store, "epiphany-test")?
                .is_empty()
        );
        assert!(load_epiphany_cultmesh_eve_surface_directory(&store, "epiphany-test")?.is_empty());
        assert!(load_epiphany_cultmesh_daemon_tool_directory(&store, "epiphany-test")?.is_empty());
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
    fn daemon_status_refuses_private_state_exposure() -> Result<()> {
        let mut status = epiphany_cultmesh_daemon_statuses("2026-06-17T00:00:00Z")
            .into_iter()
            .find(|status| status.daemon_id == "epiphany-daemon-persona")
            .expect("Persona daemon status exists");
        status.private_state_exposed = true;

        let error =
            validate_daemon_status(&status).expect_err("private daemon status must be refused");
        assert!(error.to_string().contains("private state"));
        Ok(())
    }

    #[test]
    fn daemon_poke_intent_and_receipt_round_trip() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-daemon-poke.ccmp");
        write_epiphany_cultmesh_daemon_statuses(&store, "epiphany-test", "2026-06-17T00:00:00Z")?;
        let hands = epiphany_cultmesh_daemon_statuses("2026-06-17T00:00:00Z")
            .into_iter()
            .find(|status| status.daemon_id == "epiphany-daemon-hands")
            .expect("Hands daemon status exists");
        let intent = epiphany_cultmesh_daemon_poke_intent_from_status(
            "daemon-poke-intent-test",
            "epiphany.Self",
            &hands,
            "Hands daemon missed a heartbeat and needs operator-safe poke.",
        );
        let receipt = epiphany_cultmesh_daemon_poke_receipt_for_intent(
            "daemon-poke-receipt-test",
            &intent,
            "completed",
            "ready",
            "cultmesh://epiphany-local/daemon-poke/test",
        );

        write_epiphany_cultmesh_daemon_poke_intent(&store, "epiphany-test", intent.clone())?;
        write_epiphany_cultmesh_daemon_poke_receipt(&store, "epiphany-test", receipt.clone())?;
        assert_eq!(
            write_epiphany_cultmesh_daemon_poke_intent(&store, "epiphany-test", intent.clone())?,
            intent
        );
        assert_eq!(
            write_epiphany_cultmesh_daemon_poke_receipt(&store, "epiphany-test", receipt.clone())?,
            receipt
        );

        let mut colliding_intent = intent.clone();
        colliding_intent.reason = "counterfeit replacement".to_string();
        let error =
            write_epiphany_cultmesh_daemon_poke_intent(&store, "epiphany-test", colliding_intent)
                .expect_err("non-identical intent identity collision must be refused");
        assert!(error.to_string().contains("identity collision"));

        let mut colliding_receipt = receipt.clone();
        colliding_receipt.resulting_status = "counterfeit-ready".to_string();
        let error =
            write_epiphany_cultmesh_daemon_poke_receipt(&store, "epiphany-test", colliding_receipt)
                .expect_err("non-identical receipt identity collision must be refused");
        assert!(error.to_string().contains("identity collision"));

        let mut newer_intent = intent.clone();
        newer_intent.intent_id = "daemon-poke-intent-newer".to_string();
        newer_intent.requested_at_utc = "2099-06-17T00:02:00Z".to_string();
        let mut newer_receipt = receipt.clone();
        newer_receipt.receipt_id = "daemon-poke-receipt-newer".to_string();
        newer_receipt.intent_id = newer_intent.intent_id.clone();
        newer_receipt.attempted_at_utc = "2099-06-17T00:02:00Z".to_string();
        newer_receipt.completed_at_utc = "2099-06-17T00:03:00Z".to_string();
        write_epiphany_cultmesh_daemon_poke_intent(&store, "epiphany-test", newer_intent.clone())?;
        write_epiphany_cultmesh_daemon_poke_receipt(
            &store,
            "epiphany-test",
            newer_receipt.clone(),
        )?;
        write_epiphany_cultmesh_daemon_poke_intent(&store, "epiphany-test", intent.clone())?;
        write_epiphany_cultmesh_daemon_poke_receipt(&store, "epiphany-test", receipt.clone())?;
        assert_eq!(
            load_latest_epiphany_cultmesh_daemon_poke_intent(&store, "epiphany-test")?,
            Some(newer_intent)
        );
        assert_eq!(
            load_latest_epiphany_cultmesh_daemon_poke_receipt(&store, "epiphany-test")?,
            Some(newer_receipt)
        );

        let context = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        assert_eq!(
            context
                .latest_daemon_poke_intent
                .as_ref()
                .map(|intent| intent.requested_action.as_str()),
            Some("pokeDaemon")
        );
        assert_eq!(
            context
                .latest_daemon_poke_receipt
                .as_ref()
                .map(|receipt| receipt.resulting_status.as_str()),
            Some("ready")
        );
        Ok(())
    }

    #[test]
    fn daemon_poke_refuses_private_state_and_wrong_action() -> Result<()> {
        let hands = epiphany_cultmesh_daemon_statuses("2026-06-17T00:00:00Z")
            .into_iter()
            .find(|status| status.daemon_id == "epiphany-daemon-hands")
            .expect("Hands daemon status exists");
        let mut intent = epiphany_cultmesh_daemon_poke_intent_from_status(
            "daemon-poke-intent-private-test",
            "epiphany.Self",
            &hands,
            "Attempt forbidden private daemon poke.",
        );
        intent.private_state_requested = true;
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-daemon-poke-refusal.ccmp");
        let error = write_epiphany_cultmesh_daemon_poke_intent(&store, "epiphany-test", intent)
            .expect_err("private daemon poke intents must be refused");
        assert!(error.to_string().contains("private state"));

        let intent = epiphany_cultmesh_daemon_poke_intent_from_status(
            "daemon-poke-intent-test",
            "epiphany.Self",
            &hands,
            "Attempt malformed receipt.",
        );
        let mut receipt = epiphany_cultmesh_daemon_poke_receipt_for_intent(
            "daemon-poke-receipt-wrong-action-test",
            &intent,
            "completed",
            "ready",
            "cultmesh://epiphany-local/daemon-poke/test",
        );
        receipt.action_taken = "inspectStatus".to_string();
        let error = write_epiphany_cultmesh_daemon_poke_receipt(&store, "epiphany-test", receipt)
            .expect_err("wrong daemon poke receipt action must be refused");
        assert!(error.to_string().contains("pokeDaemon"));
        Ok(())
    }

    #[test]
    fn swarm_brake_round_trips_and_projects_status() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-swarm-brake.ccmp");
        seed_epiphany_local_verse_context(&store, "epiphany-test", "2026-06-17T00:00:00Z")?;

        let brake = load_epiphany_cultmesh_swarm_brake(&store, "epiphany-test")?
            .expect("seeded swarm brake exists");
        assert_eq!(brake.status, "released");
        assert_eq!(brake.scope, "swarm");
        assert!(!brake.private_state_exposed);
        assert!(
            brake
                .affected_clusters
                .iter()
                .any(|cluster| cluster == "epiphany.cluster.persona")
        );

        let context = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        assert_eq!(
            context
                .swarm_brake
                .as_ref()
                .map(|brake| brake.status.as_str()),
            Some("released")
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

    #[test]
    fn persona_speech_audit_round_trips_and_projects_without_private_content() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-persona-speech-audit.ccmp");
        seed_epiphany_local_verse_context(&store, "epiphany-test", "2026-06-17T00:00:00Z")?;
        let audit = EpiphanyCultMeshPersonaSpeechAuditEntry {
            schema_version: EPIPHANY_CULTMESH_PERSONA_SPEECH_AUDIT_SCHEMA_VERSION.to_string(),
            audit_id: "persona-speech-audit-test".to_string(),
            runtime_id: "epiphany-test".to_string(),
            verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
            persona_agent_id: "epiphany.Persona".to_string(),
            action_kind: "post".to_string(),
            decision: "blocked".to_string(),
            content_fingerprint: "topic::normalized-opening".to_string(),
            opening_key: "rite noted modeling".to_string(),
            topic_key: "modeling|soul|evidence".to_string(),
            requested_channel_id: "123".to_string(),
            recent_window_count: 6,
            repeated_opening_count: 2,
            repeated_topic_count: 2,
            same_channel_post_count: 2,
            reasons: vec!["repeated-opening".to_string()],
            artifact_ref: "artifact://persona/speech-audit.json".to_string(),
            created_at_utc: "2026-06-17T00:00:00Z".to_string(),
            private_state_exposed: false,
            notes: vec![
                "CultMesh audit stores policy facts and fingerprints, not raw Persona prose."
                    .to_string(),
            ],
        };

        write_epiphany_cultmesh_persona_speech_audit(&store, audit.clone())?;
        assert_eq!(
            load_latest_epiphany_cultmesh_persona_speech_audit(&store, "epiphany-test")?,
            Some(audit)
        );

        let context = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        let projected = context
            .latest_persona_speech_audit
            .expect("speech audit should project into local Verse context");
        assert_eq!(projected.decision, "blocked");
        assert_eq!(projected.reasons, vec!["repeated-opening"]);
        assert!(!projected.private_state_exposed);
        Ok(())
    }

    #[test]
    fn persona_speech_audit_refuses_private_state_and_unreasoned_blocks() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp
            .path()
            .join("epiphany-persona-speech-audit-refusal.ccmp");
        let mut audit = EpiphanyCultMeshPersonaSpeechAuditEntry {
            schema_version: EPIPHANY_CULTMESH_PERSONA_SPEECH_AUDIT_SCHEMA_VERSION.to_string(),
            audit_id: "persona-speech-audit-test".to_string(),
            runtime_id: "epiphany-test".to_string(),
            verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
            persona_agent_id: "epiphany.Persona".to_string(),
            action_kind: "post".to_string(),
            decision: "eligible".to_string(),
            content_fingerprint: "topic::normalized-opening".to_string(),
            opening_key: "rite noted modeling".to_string(),
            topic_key: "modeling|soul|evidence".to_string(),
            requested_channel_id: "123".to_string(),
            recent_window_count: 1,
            repeated_opening_count: 0,
            repeated_topic_count: 0,
            same_channel_post_count: 0,
            reasons: Vec::new(),
            artifact_ref: "artifact://persona/speech-audit.json".to_string(),
            created_at_utc: "2026-06-17T00:00:00Z".to_string(),
            private_state_exposed: true,
            notes: Vec::new(),
        };
        let error = write_epiphany_cultmesh_persona_speech_audit(&store, audit.clone())
            .expect_err("private speech audit must be refused");
        assert!(error.to_string().contains("private state"));

        audit.private_state_exposed = false;
        audit.decision = "blocked".to_string();
        let error = write_epiphany_cultmesh_persona_speech_audit(&store, audit)
            .expect_err("blocked speech audit without reasons must be refused");
        assert!(error.to_string().contains("requires reasons"));
        Ok(())
    }

    #[test]
    fn weksa_lowering_receipt_round_trips_and_projects_without_publication_authority() -> Result<()>
    {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-weksa-lowering.ccmp");
        seed_epiphany_local_verse_context(&store, "epiphany-test", "2026-06-21T00:00:00Z")?;
        let receipt = EpiphanyCultMeshWeksaLoweringReceiptEntry {
            schema_version: EPIPHANY_CULTMESH_WEKSA_LOWERING_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "weksa-lowering-receipt-test".to_string(),
            runtime_id: "epiphany-test".to_string(),
            verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
            packet_id: "weksa-packet-test".to_string(),
            request_id: "weksa-request-test".to_string(),
            persona_agent_id: "epiphany.Persona".to_string(),
            target_language: "es".to_string(),
            target_register: "warm-technical".to_string(),
            delivery_surface: "eve-public-room".to_string(),
            lowering_method: "deterministic-test".to_string(),
            transport_authority: "none; Bifrost or a configured mouth transport must publish"
                .to_string(),
            publication_authorized: false,
            lowered_text_ref: "artifact://weksa/lowered-text/es".to_string(),
            lowered_text_preview: "Epiphany puede seguir trabajando.".to_string(),
            created_at_utc: "2026-06-21T00:00:00Z".to_string(),
            private_state_exposed: false,
            notes: vec!["CultMesh Weksa receipt is sight, not publication authority.".to_string()],
        };

        write_epiphany_cultmesh_weksa_lowering_receipt(&store, receipt.clone())?;
        assert_eq!(
            load_latest_epiphany_cultmesh_weksa_lowering_receipt(&store, "epiphany-test")?,
            Some(receipt)
        );

        let context = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        let projected = context
            .latest_weksa_lowering_receipt
            .expect("Weksa lowering receipt should project into local Verse context");
        assert_eq!(projected.target_language, "es");
        assert!(!projected.publication_authorized);
        assert!(!projected.private_state_exposed);
        Ok(())
    }

    #[test]
    fn weksa_lowering_receipt_refuses_private_state_and_publication_authority() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-weksa-lowering-refusal.ccmp");
        let mut receipt = EpiphanyCultMeshWeksaLoweringReceiptEntry {
            schema_version: EPIPHANY_CULTMESH_WEKSA_LOWERING_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "weksa-lowering-receipt-test".to_string(),
            runtime_id: "epiphany-test".to_string(),
            verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
            packet_id: "weksa-packet-test".to_string(),
            request_id: "weksa-request-test".to_string(),
            persona_agent_id: "epiphany.Persona".to_string(),
            target_language: "es".to_string(),
            target_register: "warm-technical".to_string(),
            delivery_surface: "eve-public-room".to_string(),
            lowering_method: "deterministic-test".to_string(),
            transport_authority: "none".to_string(),
            publication_authorized: false,
            lowered_text_ref: "artifact://weksa/lowered-text/es".to_string(),
            lowered_text_preview: "Epiphany puede seguir trabajando.".to_string(),
            created_at_utc: "2026-06-21T00:00:00Z".to_string(),
            private_state_exposed: true,
            notes: Vec::new(),
        };
        let error = write_epiphany_cultmesh_weksa_lowering_receipt(&store, receipt.clone())
            .expect_err("private Weksa receipt must be refused");
        assert!(error.to_string().contains("private state"));

        receipt.private_state_exposed = false;
        receipt.publication_authorized = true;
        let error = write_epiphany_cultmesh_weksa_lowering_receipt(&store, receipt)
            .expect_err("publication-authorizing Weksa receipt must be refused");
        assert!(error.to_string().contains("publication authority"));
        Ok(())
    }

    #[test]
    fn eve_connection_intent_and_receipt_route_feedback_without_private_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-eve-connection.ccmp");
        write_legacy_provider_fixture(&store, "epiphany-test", "epiphany-daemon-persona")?;

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let persona = node.get_required::<EpiphanyCultMeshOdinAdvertisementEntry>(
            "odin.advertisement.epiphany.cluster.persona",
        )?;
        let intent = epiphany_cultmesh_eve_connection_intent_from_advertisement(
            "eve-intent-test",
            "epiphany.cluster.self",
            &persona,
            "Coordinate public Persona collaboration feedback.",
            "requestDiscussion",
        );
        let receipt = epiphany_cultmesh_eve_connection_receipt_for_intent(
            "eve-receipt-test",
            &intent,
            "accepted-for-consensus-discovery",
        );

        write_epiphany_cultmesh_eve_connection_intent(&store, "epiphany-test", intent.clone())?;
        write_epiphany_cultmesh_eve_connection_receipt(&store, "epiphany-test", receipt.clone())?;

        assert_eq!(
            load_latest_epiphany_cultmesh_eve_connection_intent(&store, "epiphany-test")?,
            Some(intent.clone())
        );
        assert_eq!(
            load_latest_epiphany_cultmesh_eve_connection_receipt(&store, "epiphany-test")?,
            Some(receipt.clone())
        );
        assert_eq!(intent.feedback_route, "imagination.consensus_discovery");
        assert!(!intent.private_state_requested);
        assert!(!receipt.private_state_exposed);
        assert_eq!(receipt.status, "accepted-for-consensus-discovery");
        Ok(())
    }

    #[test]
    fn eve_connection_refuses_private_state_requests() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-eve-private-refusal.ccmp");
        let target = epiphany_cultmesh_odin_advertisement_templates()
            .into_iter()
            .find(|advertisement| advertisement.cluster_id == "epiphany.cluster.persona")
            .expect("persona advertisement exists");
        let mut intent = epiphany_cultmesh_eve_connection_intent_from_advertisement(
            "eve-intent-private-test",
            "epiphany.cluster.self",
            &target,
            "Attempt forbidden private state read.",
            "requestPrivateState",
        );
        intent.private_state_requested = true;

        let error = write_epiphany_cultmesh_eve_connection_intent(&store, "epiphany-test", intent)
            .expect_err("private state requests must be refused");

        assert!(
            error
                .to_string()
                .contains("must not request private Verse state")
        );
        Ok(())
    }

    #[test]
    fn daemon_tool_capabilities_make_every_local_tool_available_to_all_agents() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-daemon-tools.ccmp");
        publish_all_test_provider_state(&store)?;

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let persona_status = node.get_required::<EpiphanyCultMeshDaemonToolCapabilityEntry>(
            "epiphany.cluster.persona.tool.status",
        )?;
        let self_service_health = node.get_required::<EpiphanyCultMeshDaemonToolCapabilityEntry>(
            "epiphany.cluster.self.tool.service-health",
        )?;
        let self_service_policy_directory = node
            .get_required::<EpiphanyCultMeshDaemonToolCapabilityEntry>(
                "epiphany.cluster.self.tool.service-policy-directory",
            )?;
        let self_swarm_online_runbook = node
            .get_required::<EpiphanyCultMeshDaemonToolCapabilityEntry>(
                "epiphany.cluster.self.tool.swarm-online-runbook",
            )?;
        let hands_action = node.get_required::<EpiphanyCultMeshDaemonToolCapabilityEntry>(
            "epiphany.cluster.hands.tool.repo-action",
        )?;
        let soul_verify = node.get_required::<EpiphanyCultMeshDaemonToolCapabilityEntry>(
            "epiphany.cluster.soul.tool.verify",
        )?;

        for capability in [
            persona_status.clone(),
            self_service_health.clone(),
            self_service_policy_directory.clone(),
            self_swarm_online_runbook.clone(),
            hands_action.clone(),
            soul_verify.clone(),
        ] {
            assert!(capability.available_to_all_agents);
            assert!(capability.requires_receipt);
            assert!(!capability.private_state_exposed);
            assert!(capability.eve_surface_id.starts_with("eve://epiphany/"));
        }
        assert_eq!(hands_action.authority_gate, "hands");
        assert_eq!(soul_verify.authority_gate, "soul");
        assert_eq!(persona_status.authority_gate, "none");
        assert_eq!(
            self_service_health.authority_gate,
            "daemon.service_lifecycle"
        );
        assert_eq!(
            self_service_health.input_contract_type,
            "epiphany.cultmesh.daemon_service_lifecycle_query"
        );
        assert_eq!(
            self_service_health.receipt_contract_type,
            EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_TYPE
        );
        assert_eq!(
            self_service_policy_directory.authority_gate,
            "daemon.service_lifecycle"
        );
        assert_eq!(
            self_service_policy_directory.operation,
            "readServicePolicyDirectory"
        );
        assert_eq!(
            self_service_policy_directory.input_contract_type,
            "epiphany.cultmesh.daemon_restart_policy_directory_query"
        );
        assert_eq!(
            self_service_policy_directory.receipt_contract_type,
            EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_TYPE
        );
        assert_eq!(
            self_swarm_online_runbook.authority_gate,
            "daemon.service_lifecycle"
        );
        assert_eq!(
            self_swarm_online_runbook.operation,
            "prepareSwarmOnlineRunbook"
        );
        assert_eq!(
            self_swarm_online_runbook.input_contract_type,
            "epiphany.cultmesh.daemon_service_online_runbook_request"
        );
        assert_eq!(
            self_swarm_online_runbook.receipt_contract_type,
            EPIPHANY_CULTMESH_DAEMON_SERVICE_LIFECYCLE_RECEIPT_TYPE
        );
        assert!(
            hands_action
                .notes
                .iter()
                .any(|note| note.contains("Every agent in the local CultMesh network"))
        );
        Ok(())
    }

    #[test]
    fn daemon_tool_invocation_intent_and_receipt_round_trip_for_any_agent() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-daemon-tool-invocation.ccmp");
        write_legacy_provider_fixture(&store, "epiphany-test", "epiphany-daemon-hands")?;

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let hands_action = node.get_required::<EpiphanyCultMeshDaemonToolCapabilityEntry>(
            "epiphany.cluster.hands.tool.repo-action",
        )?;
        let intent = epiphany_cultmesh_daemon_tool_invocation_intent_from_capability(
            "daemon-tool-intent-test",
            "epiphany.Persona",
            "epiphany.cluster.persona",
            &hands_action,
            "cultmesh://epiphany-local/hands-action-intent/test",
            "Persona asks Hands to review a repo action through a globally advertised daemon tool.",
        );
        let receipt = epiphany_cultmesh_daemon_tool_invocation_receipt_for_intent(
            "daemon-tool-receipt-test",
            &intent,
            "accepted-for-hands-review",
            hands_action.receipt_contract_type.clone(),
            "cultmesh://epiphany-local/hands-action-review/test",
            "Hands accepted the daemon tool invocation for typed review.",
        );

        write_epiphany_cultmesh_daemon_tool_invocation_intent(
            &store,
            "epiphany-test",
            intent.clone(),
        )?;
        write_epiphany_cultmesh_daemon_tool_invocation_receipt(
            &store,
            "epiphany-test",
            receipt.clone(),
        )?;

        assert_eq!(
            load_latest_epiphany_cultmesh_daemon_tool_invocation_intent(&store, "epiphany-test")?,
            Some(intent.clone())
        );
        assert_eq!(
            load_latest_epiphany_cultmesh_daemon_tool_invocation_receipt(&store, "epiphany-test")?,
            Some(receipt.clone())
        );
        assert_eq!(intent.requesting_cluster_id, "epiphany.cluster.persona");
        assert_eq!(intent.host_cluster_id, "epiphany.cluster.hands");
        assert_eq!(intent.authority_gate, "hands");
        assert!(intent.requires_receipt);
        assert!(!intent.private_state_requested);
        assert_eq!(
            receipt.receipt_contract_type,
            "epiphany.hands.action_review"
        );
        assert!(!receipt.private_state_exposed);
        Ok(())
    }

    #[test]
    fn daemon_tool_invocation_refuses_private_state_requests() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp
            .path()
            .join("epiphany-daemon-tool-private-refusal.ccmp");
        let capability = epiphany_cultmesh_daemon_tool_capability_templates()
            .into_iter()
            .find(|capability| capability.capability_id == "epiphany.cluster.persona.tool.status")
            .expect("persona status capability exists");
        let mut intent = epiphany_cultmesh_daemon_tool_invocation_intent_from_capability(
            "daemon-tool-private-test",
            "epiphany.Self",
            "epiphany.cluster.self",
            &capability,
            "cultmesh://epiphany-local/private-state/test",
            "Attempt forbidden private state through a globally visible daemon tool.",
        );
        intent.private_state_requested = true;

        let error =
            write_epiphany_cultmesh_daemon_tool_invocation_intent(&store, "epiphany-test", intent)
                .expect_err("private state requests must be refused");

        assert!(
            error
                .to_string()
                .contains("must not request private Verse state")
        );
        Ok(())
    }

    #[test]
    fn bifrost_body_change_publication_intent_and_receipt_round_trip() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp
            .path()
            .join("epiphany-bifrost-body-change-publication.ccmp");
        let intent = epiphany_cultmesh_bifrost_body_change_publication_intent(
            "bifrost-publication-intent-test",
            "epiphany.cluster.hands",
            "epiphany.Hands",
            "repo:E:/Projects/EpiphanyAgent",
            "E:/Projects/EpiphanyAgent",
            "codex/perfect-machine-cultmesh",
            "Route typed CultMesh publication proof through Bifrost.",
            "Publication needs Bifrost ledger, credit, review, and GitHub routing proof.",
            vec!["epiphany-core/src/cultmesh_integration.rs".to_string()],
            vec!["soul-verdict-test".to_string()],
            vec!["maintainer-review-test".to_string()],
            vec!["epiphany.Hands".to_string()],
            vec!["GameCult/EpiphanyAgent".to_string()],
        );
        let receipt = epiphany_cultmesh_bifrost_body_change_publication_receipt_for_intent(
            "bifrost-publication-receipt-test",
            &intent,
            "accepted-for-github-publication",
            "bifrost-ledger-test",
            "github-publication-test",
            vec!["credit-receipt-test".to_string()],
            vec!["maintainer-review-test".to_string()],
            "https://github.com/GameCult/EpiphanyAgent/pull/test",
        );

        write_epiphany_cultmesh_bifrost_body_change_publication_intent(
            &store,
            "epiphany-test",
            intent.clone(),
        )?;
        write_epiphany_cultmesh_bifrost_body_change_publication_receipt(
            &store,
            "epiphany-test",
            receipt.clone(),
        )?;

        assert_eq!(
            load_arrival_latest_epiphany_cultmesh_bifrost_body_change_publication_intent(
                &store,
                "epiphany-test"
            )?,
            Some(intent.clone())
        );
        assert_eq!(
            load_arrival_latest_epiphany_cultmesh_bifrost_body_change_publication_receipt(
                &store,
                "epiphany-test"
            )?,
            Some(receipt.clone())
        );
        assert!(intent.github_publication_requested);
        assert!(!intent.private_state_included);
        assert_eq!(intent.verification_receipt_ids, vec!["soul-verdict-test"]);
        assert_eq!(receipt.bifrost_ledger_entry_id, "bifrost-ledger-test");
        assert_eq!(
            receipt.github_publication_receipt_id,
            "github-publication-test"
        );
        assert!(!receipt.private_state_exposed);
        Ok(())
    }

    #[test]
    fn bifrost_body_change_publication_refuses_private_or_unverified_intents() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp
            .path()
            .join("epiphany-bifrost-publication-refusal.ccmp");
        let mut intent = epiphany_cultmesh_bifrost_body_change_publication_intent(
            "bifrost-publication-private-test",
            "epiphany.cluster.hands",
            "epiphany.Hands",
            "repo:E:/Projects/EpiphanyAgent",
            "E:/Projects/EpiphanyAgent",
            "codex/perfect-machine-cultmesh",
            "Attempt invalid publication.",
            "This should be refused.",
            vec!["epiphany-core/src/cultmesh_integration.rs".to_string()],
            vec!["soul-verdict-test".to_string()],
            vec!["maintainer-review-test".to_string()],
            vec!["epiphany.Hands".to_string()],
            vec!["GameCult/EpiphanyAgent".to_string()],
        );
        intent.private_state_included = true;

        let error = write_epiphany_cultmesh_bifrost_body_change_publication_intent(
            &store,
            "epiphany-test",
            intent,
        )
        .expect_err("private publication payloads must be refused");
        assert!(error.to_string().contains("must not include private state"));

        let unverified = epiphany_cultmesh_bifrost_body_change_publication_intent(
            "bifrost-publication-unverified-test",
            "epiphany.cluster.hands",
            "epiphany.Hands",
            "repo:E:/Projects/EpiphanyAgent",
            "E:/Projects/EpiphanyAgent",
            "codex/perfect-machine-cultmesh",
            "Attempt unverified publication.",
            "This should be refused.",
            vec!["epiphany-core/src/cultmesh_integration.rs".to_string()],
            Vec::new(),
            vec!["maintainer-review-test".to_string()],
            vec!["epiphany.Hands".to_string()],
            vec!["GameCult/EpiphanyAgent".to_string()],
        );
        let error = write_epiphany_cultmesh_bifrost_body_change_publication_intent(
            &store,
            "epiphany-test",
            unverified,
        )
        .expect_err("unverified publication payloads must be refused");
        assert!(error.to_string().contains("require verification receipts"));
        Ok(())
    }

    #[test]
    fn bifrost_github_publication_receipt_round_trips_with_hands_pr_proof() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-bifrost-github-publication.ccmp");
        let intent = epiphany_cultmesh_bifrost_body_change_publication_intent(
            "bifrost-publication-intent-test",
            "epiphany.cluster.hands",
            "epiphany.Hands",
            "repo:E:/Projects/EpiphanyAgent",
            "E:/Projects/EpiphanyAgent",
            "codex/perfect-machine-cultmesh",
            "Route typed GitHub publication proof through Bifrost.",
            "Publication needs Bifrost ledger, Hands PR, credit, and review proof.",
            vec!["epiphany-core/src/cultmesh_integration.rs".to_string()],
            vec!["soul-verdict-test".to_string()],
            vec!["maintainer-review-test".to_string()],
            vec!["epiphany.Hands".to_string()],
            vec!["GameCult/EpiphanyAgent".to_string()],
        );
        let publication = epiphany_cultmesh_bifrost_body_change_publication_receipt_for_intent(
            "bifrost-publication-receipt-test",
            &intent,
            "accepted-for-github-publication",
            "bifrost-ledger-test",
            "github-publication-test",
            vec!["credit-receipt-test".to_string()],
            vec!["maintainer-review-test".to_string()],
            "https://github.com/GameCult/EpiphanyAgent/pull/test",
        );
        let github = epiphany_cultmesh_bifrost_github_publication_receipt_for_publication(
            "github-publication-test",
            &publication,
            "hands-pr-test",
            "E:/Projects/EpiphanyAgent",
            "codex/perfect-machine-cultmesh",
            "test",
            "abc123",
            "epiphany.Hands",
        );

        write_epiphany_cultmesh_bifrost_body_change_publication_intent(
            &store,
            "epiphany-test",
            intent,
        )?;
        write_epiphany_cultmesh_bifrost_body_change_publication_receipt(
            &store,
            "epiphany-test",
            publication.clone(),
        )?;
        write_epiphany_cultmesh_bifrost_github_publication_receipt(
            &store,
            "epiphany-test",
            github.clone(),
        )?;

        assert_eq!(
            load_arrival_latest_epiphany_cultmesh_bifrost_github_publication_receipt(
                &store,
                "epiphany-test"
            )?,
            Some(github.clone())
        );
        assert_eq!(
            github.bifrost_publication_receipt_id,
            "bifrost-publication-receipt-test"
        );
        assert_eq!(github.hands_pr_receipt_id, "hands-pr-test");
        assert_eq!(github.ledger_entry_id, "bifrost-ledger-test");
        assert_eq!(github.credit_receipt_ids, vec!["credit-receipt-test"]);
        assert_eq!(
            github.pull_request_url,
            "https://github.com/GameCult/EpiphanyAgent/pull/test"
        );
        assert!(!github.private_state_exposed);
        Ok(())
    }

    #[test]
    fn bifrost_github_publication_refuses_private_or_unlinked_receipts() -> Result<()> {
        let publication = EpiphanyCultMeshBifrostBodyChangePublicationReceiptEntry {
            schema_version:
                EPIPHANY_CULTMESH_BIFROST_BODY_CHANGE_PUBLICATION_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "bifrost-publication-receipt-test".to_string(),
            intent_id: "bifrost-publication-intent-test".to_string(),
            status: "accepted-for-github-publication".to_string(),
            bifrost_ledger_entry_id: "bifrost-ledger-test".to_string(),
            github_publication_receipt_id: "github-publication-test".to_string(),
            credit_receipt_ids: vec!["credit-receipt-test".to_string()],
            accepted_changed_paths: vec!["epiphany-core/src/cultmesh_integration.rs".to_string()],
            reviewer_ids: vec!["maintainer-review-test".to_string()],
            publication_url: "https://github.com/GameCult/EpiphanyAgent/pull/test".to_string(),
            private_state_exposed: false,
            notes: Vec::new(),
        };
        let mut github = epiphany_cultmesh_bifrost_github_publication_receipt_for_publication(
            "github-publication-test",
            &publication,
            "hands-pr-test",
            "E:/Projects/EpiphanyAgent",
            "codex/perfect-machine-cultmesh",
            "test",
            "abc123",
            "epiphany.Hands",
        );
        github.private_state_exposed = true;
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-bifrost-github-refusal.ccmp");
        let error = write_epiphany_cultmesh_bifrost_github_publication_receipt(
            &store,
            "epiphany-test",
            github,
        )
        .expect_err("private GitHub publication receipts must be refused");
        assert!(error.to_string().contains("must not expose private state"));

        let mut unlinked = epiphany_cultmesh_bifrost_github_publication_receipt_for_publication(
            "github-publication-unlinked-test",
            &publication,
            "hands-pr-test",
            "E:/Projects/EpiphanyAgent",
            "codex/perfect-machine-cultmesh",
            "test",
            "abc123",
            "epiphany.Hands",
        );
        unlinked.hands_pr_receipt_id.clear();
        let error = write_epiphany_cultmesh_bifrost_github_publication_receipt(
            &store,
            "epiphany-test",
            unlinked,
        )
        .expect_err("GitHub publication receipts without Hands PR proof must be refused");
        assert!(error.to_string().contains("require a Hands PR receipt"));
        Ok(())
    }

    #[test]
    fn bifrost_public_proof_publication_receipt_round_trips() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp
            .path()
            .join("epiphany-bifrost-public-proof-publication.ccmp");
        let proof = EpiphanyCultMeshRepoWorkPublicProofEntry {
            schema_version: EPIPHANY_CULTMESH_REPO_WORK_PUBLIC_PROOF_SCHEMA_VERSION.to_string(),
            runtime_id: "epiphany-test".to_string(),
            verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
            public_proof_id: "repo-work-public-proof-test".to_string(),
            generated_at: "2026-06-20T12:00:00Z".to_string(),
            workspace: "E:/Projects/EpiphanyAgent".to_string(),
            item: "test-item".to_string(),
            branch: "codex/test-item".to_string(),
            current_gate: "awaiting-publication".to_string(),
            blocker: "bifrost-publication-missing".to_string(),
            next_safe_move: "publish-redacted-proof".to_string(),
            changed_paths: vec!["notes/test.md".to_string()],
            commit_sha: "abc123".to_string(),
            soul_verdict: "passed".to_string(),
            upstream_main_synced: true,
            artifact_row_count: 3,
            publication_row_count: 5,
            public_proof_ref: "public-proof.json".to_string(),
            public_proof_sha256: "0123456789abcdef".to_string(),
            tui_rows: vec!["proof row".to_string()],
            private_state_exposed: false,
            notes: vec!["redacted proof".to_string()],
        };
        let receipt = epiphany_cultmesh_bifrost_public_proof_publication_receipt_for_proof(
            "bifrost-public-proof-publication-test",
            &proof,
            "published-to-public-verse",
            EPIPHANY_CULTMESH_GLOBAL_VERSE_ID,
            "epiphany-global/repo-work/public-proofs",
            "bifrost-ledger-public-proof-test",
            vec!["credit-receipt-test".to_string()],
            vec!["maintainer-review-test".to_string()],
            "cultmesh://epiphany-global/repo-work/public-proofs/repo-work-public-proof-test",
        );

        write_epiphany_cultmesh_bifrost_public_proof_publication_receipt(
            &store,
            "epiphany-test",
            receipt.clone(),
        )?;

        assert_eq!(
            load_arrival_latest_epiphany_cultmesh_bifrost_public_proof_publication_receipt(
                &store,
                "epiphany-test"
            )?,
            Some(receipt.clone())
        );
        assert_eq!(receipt.public_proof_id, proof.public_proof_id);
        assert_eq!(receipt.public_proof_sha256, proof.public_proof_sha256);
        assert_eq!(
            receipt.target_public_verse_id,
            EPIPHANY_CULTMESH_GLOBAL_VERSE_ID
        );
        assert_eq!(receipt.credit_receipt_ids, vec!["credit-receipt-test"]);
        assert_eq!(receipt.reviewer_ids, vec!["maintainer-review-test"]);
        assert!(!receipt.private_state_exposed);
        Ok(())
    }

    #[test]
    fn bifrost_public_proof_publication_refuses_private_or_wrong_verse() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp
            .path()
            .join("epiphany-bifrost-public-proof-publication-refusal.ccmp");
        let proof = EpiphanyCultMeshRepoWorkPublicProofEntry {
            schema_version: EPIPHANY_CULTMESH_REPO_WORK_PUBLIC_PROOF_SCHEMA_VERSION.to_string(),
            runtime_id: "epiphany-test".to_string(),
            verse_id: EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID.to_string(),
            public_proof_id: "repo-work-public-proof-test".to_string(),
            generated_at: "2026-06-20T12:00:00Z".to_string(),
            workspace: "E:/Projects/EpiphanyAgent".to_string(),
            item: "test-item".to_string(),
            branch: "codex/test-item".to_string(),
            current_gate: "awaiting-publication".to_string(),
            blocker: "bifrost-publication-missing".to_string(),
            next_safe_move: "publish-redacted-proof".to_string(),
            changed_paths: vec!["notes/test.md".to_string()],
            commit_sha: "abc123".to_string(),
            soul_verdict: "passed".to_string(),
            upstream_main_synced: true,
            artifact_row_count: 3,
            publication_row_count: 5,
            public_proof_ref: "public-proof.json".to_string(),
            public_proof_sha256: "0123456789abcdef".to_string(),
            tui_rows: vec!["proof row".to_string()],
            private_state_exposed: false,
            notes: vec!["redacted proof".to_string()],
        };
        let mut receipt = epiphany_cultmesh_bifrost_public_proof_publication_receipt_for_proof(
            "bifrost-public-proof-publication-private-test",
            &proof,
            "published-to-public-verse",
            EPIPHANY_CULTMESH_GLOBAL_VERSE_ID,
            "epiphany-global/repo-work/public-proofs",
            "bifrost-ledger-public-proof-test",
            vec!["credit-receipt-test".to_string()],
            vec!["maintainer-review-test".to_string()],
            "cultmesh://epiphany-global/repo-work/public-proofs/repo-work-public-proof-test",
        );
        receipt.private_state_exposed = true;
        let error = write_epiphany_cultmesh_bifrost_public_proof_publication_receipt(
            &store,
            "epiphany-test",
            receipt,
        )
        .expect_err("private proof publication receipts must be refused");
        assert!(error.to_string().contains("must not expose private state"));

        let wrong_verse = epiphany_cultmesh_bifrost_public_proof_publication_receipt_for_proof(
            "bifrost-public-proof-publication-wrong-verse-test",
            &proof,
            "published-to-public-verse",
            EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID,
            "gamecult-local/repo-work/public-proofs",
            "bifrost-ledger-public-proof-test",
            vec!["credit-receipt-test".to_string()],
            vec!["maintainer-review-test".to_string()],
            "cultmesh://gamecult-local/repo-work/public-proofs/repo-work-public-proof-test",
        );
        let error = write_epiphany_cultmesh_bifrost_public_proof_publication_receipt(
            &store,
            "epiphany-test",
            wrong_verse,
        )
        .expect_err("non-public Verse proof publication receipts must be refused");
        assert!(error.to_string().contains("global public Verse"));
        Ok(())
    }

    #[test]
    fn collaboration_feedback_routes_to_imagination_consensus() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp
            .path()
            .join("epiphany-bifrost-collaboration-feedback.ccmp");
        let feedback = epiphany_cultmesh_bifrost_collaboration_feedback(
            "collaboration-feedback-test",
            "epiphany.Persona",
            "epiphany.persona",
            "epiphany-global/collaboration",
            "eve-receipt-test",
            "Persona asks for cross-body collaboration over Eve.",
            "Public Persona discussion should be compared by Imagination before adoption.",
            vec!["https://gamecult.org/Blog/purge-the-heretek-from-our-daemonic-swarm".to_string()],
            vec!["candidate-action:compare-daemon-surfaces".to_string()],
        );
        let consensus = epiphany_cultmesh_imagination_consensus_receipt_for_feedback(
            "imagination-consensus-test",
            &feedback,
            "queued-for-consensus-discovery",
            vec!["epiphany.Imagination".to_string()],
            "gamecult-local/imagination/consensus-packets/test",
        );

        write_epiphany_cultmesh_bifrost_collaboration_feedback(
            &store,
            "epiphany-test",
            feedback.clone(),
        )?;
        write_epiphany_cultmesh_imagination_consensus_receipt(
            &store,
            "epiphany-test",
            consensus.clone(),
        )?;

        assert_eq!(
            load_arrival_latest_epiphany_cultmesh_bifrost_collaboration_feedback(
                &store,
                "epiphany-test"
            )?,
            Some(feedback)
        );
        assert_eq!(
            load_latest_epiphany_cultmesh_imagination_consensus_receipt(&store, "epiphany-test")?,
            Some(consensus)
        );
        let context = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        assert_eq!(
            context
                .arrival_latest_bifrost_collaboration_feedback
                .as_ref()
                .map(|feedback| feedback.requested_consensus_route.as_str()),
            Some("imagination.consensus_discovery")
        );
        assert_eq!(
            context
                .latest_imagination_consensus_receipt
                .as_ref()
                .map(|receipt| receipt.adoption_gate.as_str()),
            Some("mind.review_then_bifrost_adoption")
        );
        Ok(())
    }

    #[test]
    fn collaboration_feedback_refuses_private_state_and_unanchored_public_claims() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-feedback-refusal.ccmp");
        let mut private = epiphany_cultmesh_bifrost_collaboration_feedback(
            "collaboration-feedback-private-test",
            "epiphany.Persona",
            "epiphany.persona",
            "epiphany-global/collaboration",
            "eve-receipt-test",
            "Attempt invalid feedback.",
            "This should not publish private state.",
            vec!["https://gamecult.org/public-thread".to_string()],
            Vec::new(),
        );
        private.private_state_included = true;
        let error = write_epiphany_cultmesh_bifrost_collaboration_feedback(
            &store,
            "epiphany-test",
            private,
        )
        .expect_err("private collaboration feedback must be refused");
        assert!(error.to_string().contains("private state"));

        let unanchored = epiphany_cultmesh_bifrost_collaboration_feedback(
            "collaboration-feedback-unanchored-test",
            "epiphany.Persona",
            "epiphany.persona",
            "epiphany-global/collaboration",
            "eve-receipt-test",
            "Attempt invalid feedback.",
            "Public collaboration feedback must cite public discussion.",
            Vec::new(),
            Vec::new(),
        );
        let error = write_epiphany_cultmesh_bifrost_collaboration_feedback(
            &store,
            "epiphany-test",
            unanchored,
        )
        .expect_err("unanchored collaboration feedback must be refused");
        assert!(error.to_string().contains("public discussion"));

        let feedback = epiphany_cultmesh_bifrost_collaboration_feedback(
            "collaboration-feedback-test",
            "epiphany.Persona",
            "epiphany.persona",
            "epiphany-global/collaboration",
            "eve-receipt-test",
            "Attempt invalid consensus.",
            "Consensus receipts must keep private state sealed.",
            vec!["https://gamecult.org/public-thread".to_string()],
            Vec::new(),
        );
        let mut receipt = epiphany_cultmesh_imagination_consensus_receipt_for_feedback(
            "imagination-consensus-private-test",
            &feedback,
            "queued-for-consensus-discovery",
            vec!["epiphany.Imagination".to_string()],
            "gamecult-local/imagination/consensus-packets/test",
        );
        receipt.private_state_exposed = true;
        let error =
            write_epiphany_cultmesh_imagination_consensus_receipt(&store, "epiphany-test", receipt)
                .expect_err("private consensus receipts must be refused");
        assert!(error.to_string().contains("private state"));
        Ok(())
    }

    #[test]
    fn local_verse_bootstrap_does_not_publish_provider_owned_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-local-verse.ccmp");
        seed_epiphany_local_verse_context(&store, "epiphany-test", "2026-06-02T00:00:00Z")?;

        let context = query_epiphany_local_verse_context(&store, "epiphany-test")?;

        assert_eq!(context.verse_policies.len(), 3);
        assert!(context.verse_policies.iter().any(|policy| policy.verse_id
            == EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID
            && policy.yggdrasil_tunnel_allowed));
        assert_eq!(context.global_room_policies.len(), 6);
        assert_eq!(context.cluster_topology.len(), 7);
        assert!(context.cluster_topology.iter().any(|cluster| {
            cluster.cluster_id == "epiphany.cluster.persona"
                && cluster.public_persona_discussion_allowed
                && cluster.eve_surface_id == "eve://epiphany/persona"
        }));
        assert!(context.odin_advertisements.is_empty());
        assert!(context.eve_surface_states.is_empty());
        assert!(context.daemon_tool_capabilities.is_empty());
        assert!(
            context
                .contract_summaries
                .iter()
                .any(|contract| contract.authority == "mind")
        );
        assert!(
            context
                .contract_summaries
                .iter()
                .any(|contract| contract.authority == "substrateGate")
        );
        assert!(
            context
                .contract_summaries
                .iter()
                .any(|contract| contract.authority == "bifrost")
        );
        assert!(context.odin_scope.contains("all-seer"));
        assert!(context.yggdrasil_scope.contains("Bifrost"));
        assert!(context.prompt_assembly_note.contains("bounded context"));
        Ok(())
    }

    #[test]
    fn explicit_bootstrap_retires_legacy_provider_forgery() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-provider-boundary.ccmp");
        seed_epiphany_local_verse_context(&store, "epiphany-test", "2026-07-12T00:00:00Z")?;
        let sentinel_before =
            open_epiphany_cultmesh_node(&store, "epiphany-test")?
                .get_required::<EpiphanyCultMeshStatusEntry>(EPIPHANY_CULTMESH_STATUS_KEY)?;

        write_legacy_provider_fixture(&store, "epiphany-test", "epiphany-daemon-hands")?;
        assert!(
            query_epiphany_local_verse_context(&store, "epiphany-test")?
                .odin_advertisements
                .is_empty()
        );
        let mut retired = retire_epiphany_cultmesh_legacy_provider_documents(&store)?;
        retired.sort();
        assert_eq!(
            retired,
            vec![
                "epiphany.cluster.hands.tool.eve-connect".to_string(),
                "epiphany.cluster.hands.tool.repo-action".to_string(),
                "epiphany.cluster.hands.tool.status".to_string(),
                "eve://epiphany/hands".to_string(),
                "odin.advertisement.epiphany.cluster.hands".to_string(),
            ]
        );
        assert_eq!(
            open_epiphany_cultmesh_node(&store, "epiphany-test")?
                .get_required::<EpiphanyCultMeshStatusEntry>(EPIPHANY_CULTMESH_STATUS_KEY)?,
            sentinel_before
        );
        seed_epiphany_local_verse_context(&store, "epiphany-test", "2026-07-12T00:01:00Z")?;
        assert!(retire_epiphany_cultmesh_legacy_provider_documents(&store)?.is_empty());

        let error =
            write_legacy_provider_fixture(&store, "epiphany-test", "epiphany-daemon-counterfeit")
                .expect_err("unknown daemons must not create even a legacy fixture");
        assert!(
            error
                .to_string()
                .contains("legacy fixture daemon has no topology")
        );
        let unchanged = query_epiphany_local_verse_context(&store, "epiphany-test")?;
        assert!(unchanged.odin_advertisements.is_empty());
        assert!(unchanged.eve_surface_states.is_empty());
        assert!(unchanged.daemon_tool_capabilities.is_empty());
        Ok(())
    }

    #[test]
    fn mind_contracts_use_verses_to_keep_state_guarded() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-mind-contracts.ccmp");
        let written = write_epiphany_cultmesh_mind_contracts(&store, "epiphany-test")?;
        assert!(written.len() >= 4);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let state_review = node.get_required::<EpiphanyCultMeshMindContractEntry>(
            "epiphany.mind.state_effect.review",
        )?;
        let public_adoption = node.get_required::<EpiphanyCultMeshMindContractEntry>(
            "epiphany.mind.public_adoption.review",
        )?;

        assert_eq!(state_review.verse_id, EPIPHANY_CULTMESH_INTERNAL_VERSE_ID);
        assert_eq!(state_review.authority, "mind");
        assert!(
            state_review
                .notes
                .iter()
                .any(|note| note.contains("persistent state guardian"))
        );
        assert_eq!(public_adoption.verse_id, EPIPHANY_CULTMESH_GLOBAL_VERSE_ID);
        assert!(
            public_adoption
                .notes
                .iter()
                .any(|note| note.contains("thought weather"))
        );
        Ok(())
    }

    #[test]
    fn substrate_gate_contracts_use_verses_to_keep_repo_access_guarded() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-substrate-gate-contracts.ccmp");
        let written = write_epiphany_cultmesh_substrate_gate_contracts(&store, "epiphany-test")?;
        assert!(written.len() >= 4);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let repo_access = node.get_required::<EpiphanyCultMeshSubstrateGateContractEntry>(
            "epiphany.substrate_gate.repo_access.review",
        )?;

        assert_eq!(repo_access.verse_id, EPIPHANY_CULTMESH_INTERNAL_VERSE_ID);
        assert_eq!(repo_access.authority, "substrateGate");
        assert!(
            repo_access
                .notes
                .iter()
                .any(|note| note.contains("repo access protocol"))
        );
        Ok(())
    }

    #[test]
    fn eyes_contracts_use_verses_to_keep_evidence_guarded() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-eyes-contracts.ccmp");
        let written = write_epiphany_cultmesh_eyes_contracts(&store, "epiphany-test")?;
        assert!(written.len() >= 4);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let evidence = node
            .get_required::<EpiphanyCultMeshEyesContractEntry>("epiphany.eyes.evidence.review")?;

        assert_eq!(evidence.verse_id, EPIPHANY_CULTMESH_INTERNAL_VERSE_ID);
        assert_eq!(evidence.authority, "eyes");
        assert!(
            evidence
                .notes
                .iter()
                .any(|note| note.contains("evidence ingress guardian"))
        );
        Ok(())
    }

    #[test]
    fn hands_contracts_use_verses_to_keep_action_guarded() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-hands-contracts.ccmp");
        let written = write_epiphany_cultmesh_hands_contracts(&store, "epiphany-test")?;
        assert!(written.len() >= 5);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let action = node
            .get_required::<EpiphanyCultMeshHandsContractEntry>("epiphany.hands.action.review")?;

        assert_eq!(action.verse_id, EPIPHANY_CULTMESH_INTERNAL_VERSE_ID);
        assert_eq!(action.authority, "hands");
        assert!(
            action
                .notes
                .iter()
                .any(|note| note.contains("action organ"))
        );
        Ok(())
    }

    #[test]
    fn soul_contracts_use_verses_to_keep_verification_guarded() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-soul-contracts.ccmp");
        let written = write_epiphany_cultmesh_soul_contracts(&store, "epiphany-test")?;
        assert!(written.len() >= 5);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let verification = node.get_required::<EpiphanyCultMeshSoulContractEntry>(
            "epiphany.soul.verification.review",
        )?;

        assert_eq!(verification.verse_id, EPIPHANY_CULTMESH_INTERNAL_VERSE_ID);
        assert_eq!(verification.authority, "soul");
        assert!(
            verification
                .notes
                .iter()
                .any(|note| note.contains("verification organ"))
        );
        Ok(())
    }

    #[test]
    fn continuity_contracts_use_verses_to_keep_continuity_guarded() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-continuity-contracts.ccmp");
        let written = write_epiphany_cultmesh_continuity_contracts(&store, "epiphany-test")?;
        assert!(written.len() >= 5);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let continuity = node.get_required::<EpiphanyCultMeshContinuityContractEntry>(
            "epiphany.continuity.review",
        )?;

        assert_eq!(continuity.verse_id, EPIPHANY_CULTMESH_INTERNAL_VERSE_ID);
        assert_eq!(continuity.authority, "continuity");
        assert!(
            continuity
                .notes
                .iter()
                .any(|note| note.contains("deterministic protocol surface"))
        );
        Ok(())
    }

    #[test]
    fn bifrost_contracts_route_body_changes_before_github_publication() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("epiphany-bifrost-contracts.ccmp");
        let written = write_epiphany_cultmesh_bifrost_contracts(&store, "epiphany-test")?;
        assert_eq!(written.len(), 3);

        let node = open_epiphany_cultmesh_node(&store, "epiphany-test")?;
        let publication = node.get_required::<EpiphanyCultMeshBifrostContractEntry>(
            "gamecult.bifrost.body_change.publication",
        )?;
        let feedback = node.get_required::<EpiphanyCultMeshBifrostContractEntry>(
            "gamecult.bifrost.collaboration.feedback",
        )?;
        let public_proof = node.get_required::<EpiphanyCultMeshBifrostContractEntry>(
            "gamecult.bifrost.public_proof.publication",
        )?;

        assert_eq!(publication.verse_id, EPIPHANY_CULTMESH_LOCAL_AREA_VERSE_ID);
        assert_eq!(publication.authority, "bifrost");
        assert!(
            publication
                .receipt_document_types
                .iter()
                .any(|receipt| receipt == "gamecult.bifrost.github_publication_receipt")
        );
        assert!(
            publication
                .receipt_document_types
                .iter()
                .any(|receipt| receipt == "gamecult.bifrost.credit_receipt")
        );
        assert!(
            publication
                .notes
                .iter()
                .any(|note| note.contains("GitHub publication"))
        );
        assert_eq!(feedback.authority, "imaginationConsensus");
        assert!(
            feedback
                .notes
                .iter()
                .any(|note| note.contains("thought weather"))
        );
        assert_eq!(public_proof.authority, "bifrost");
        assert_eq!(
            public_proof.document_type,
            EPIPHANY_CULTMESH_BIFROST_PUBLIC_PROOF_PUBLICATION_RECEIPT_TYPE
        );
        assert!(
            public_proof
                .intent_document_types
                .iter()
                .any(|intent| intent == EPIPHANY_CULTMESH_REPO_WORK_PUBLIC_PROOF_TYPE)
        );
        assert!(
            public_proof
                .receipt_document_types
                .iter()
                .any(|receipt| receipt
                    == EPIPHANY_CULTMESH_BIFROST_PUBLIC_PROOF_PUBLICATION_RECEIPT_TYPE)
        );
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
        let input = semantic_health_input(&canonical, "swarm-recovery", "mind", 1)?;
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
            owner_daemon_id: "epiphany-daemon-supervisor".into(),
            command: binary.into(),
            args: vec![
                "serve",
                "--agent-store",
                "mind.ccmp",
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
            restart_mode: "always".into(),
            cooldown_seconds: 0,
            backoff_multiplier: 1,
            stdout_artifact: "projector.stdout.log".into(),
            stderr_artifact: "projector.stderr.log".into(),
            updated_at_utc: "2026-07-15T12:01:00Z".into(),
            private_state_exposed: false,
            notes: vec![],
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
        advanced_policy.updated_at_utc = "2026-07-15T12:02:30Z".into();
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

    #[test]
    fn bifrost_mirrors_name_arrival_order_without_rewriting_storage_keys() {
        let source = include_str!("cultmesh_integration.rs");
        let old_loader = ["load_", "latest_epiphany_cultmesh_bifrost_"].concat();
        let old_field = ["pub ", "latest_bifrost_"].concat();
        assert!(!source.contains(&old_loader));
        assert!(!source.contains(&old_field));
        assert_eq!(
            source
                .lines()
                .filter(|line| {
                    line.contains("\"gamecult-local/bifrost/") && line.contains("/latest\"")
                })
                .count(),
            7
        );
    }
}
