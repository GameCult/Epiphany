use crate::EpiphanyWorkerLaunchDocument;
use crate::RepoFrontierPlanMindContextProjection;
use crate::agent_launch::{
    EPIPHANY_IMAGINATION_OWNER_ROLE, EPIPHANY_IMAGINATION_ROLE_BINDING_ID,
    EPIPHANY_MIND_OWNER_ROLE, EPIPHANY_MIND_ROLE_BINDING_ID, EPIPHANY_MODELING_OWNER_ROLE,
    EPIPHANY_MODELING_ROLE_BINDING_ID,
};
use crate::agent_memory::{
    AGENT_MEMORY_SWARM_IDENTITY_KEY, AGENT_MEMORY_SWARM_IDENTITY_SCHEMA_VERSION,
    AGENT_MEMORY_SWARM_IDENTITY_TYPE, AGENT_MEMORY_TYPE, load_agent_memory_swarm_identity,
};
use crate::continuity_gateway::ContinuityRecoveryReceipt;
use crate::continuity_gateway::*;
use crate::cultmesh_integration::EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION;
use crate::cultmesh_integration::EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_TYPE;
use crate::cultmesh_integration::EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION;
use crate::cultmesh_integration::EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_TYPE;
use crate::cultmesh_integration::EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_SCHEMA_VERSION;
use crate::cultmesh_integration::EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_TYPE;
use crate::eyes_gateway::EYES_EVIDENCE_PACKET_SCHEMA_VERSION;
use crate::eyes_gateway::EYES_EVIDENCE_PACKET_TYPE;
use crate::eyes_gateway::EYES_EVIDENCE_REFUSAL_RECEIPT_SCHEMA_VERSION;
use crate::eyes_gateway::EYES_EVIDENCE_REFUSAL_RECEIPT_TYPE;
use crate::eyes_gateway::EYES_EVIDENCE_REQUEST_SCHEMA_VERSION;
use crate::eyes_gateway::EYES_EVIDENCE_REQUEST_TYPE;
use crate::eyes_gateway::EYES_EVIDENCE_REVIEW_SCHEMA_VERSION;
use crate::eyes_gateway::EYES_EVIDENCE_REVIEW_TYPE;
use crate::eyes_gateway::EYES_SOURCE_LOOKUP_RECEIPT_SCHEMA_VERSION;
use crate::eyes_gateway::EYES_SOURCE_LOOKUP_RECEIPT_TYPE;
use crate::eyes_gateway::EyesEvidencePacket;
use crate::eyes_gateway::EyesSourceLookupReceipt;
use crate::hands_gateway::*;
use crate::heartbeat_state::HEARTBEAT_STATE_SCHEMA_VERSION;
use crate::heartbeat_state::HEARTBEAT_STATE_TYPE;
use crate::mind_gateway::MIND_GATEWAY_REVIEW_SCHEMA_VERSION;
use crate::mind_gateway::MIND_GATEWAY_REVIEW_TYPE;
use crate::mind_gateway::MIND_STATE_COMMIT_RECEIPT_SCHEMA_VERSION;
use crate::mind_gateway::MIND_STATE_COMMIT_RECEIPT_TYPE;
use crate::mind_gateway::MIND_STATE_EFFECT_PROPOSAL_SCHEMA_VERSION;
use crate::mind_gateway::MIND_STATE_EFFECT_PROPOSAL_TYPE;
use crate::mind_gateway::MIND_STATE_REJECTION_RECEIPT_SCHEMA_VERSION;
use crate::mind_gateway::MIND_STATE_REJECTION_RECEIPT_TYPE;
use crate::mind_gateway::MIND_THOUGHT_SCHEMA_VERSION;
use crate::mind_gateway::MIND_THOUGHT_TYPE;
use crate::mind_gateway::MIND_VERSE_ADOPTION_RECEIPT_SCHEMA_VERSION;
use crate::mind_gateway::MIND_VERSE_ADOPTION_RECEIPT_TYPE;
use crate::mind_gateway::MindGatewayReview;
use crate::mind_gateway::MindStateCommitReceipt;
use crate::organ_dependencies::EpiphanyLaunchOrganContract;
use crate::repo_model_gateway::{
    REPO_FRONTIER_AUTONOMOUS_PROPOSAL_BINDING_CONTRACT,
    REPO_FRONTIER_AUTONOMOUS_PROPOSAL_BINDING_SCHEMA_VERSION,
    REPO_FRONTIER_EXECUTION_AMENDMENT_RECEIPT_CONTRACT,
    REPO_FRONTIER_EXECUTION_AMENDMENT_RECEIPT_SCHEMA_VERSION,
    REPO_FRONTIER_HANDS_AUTHORITY_CONTRACT, REPO_FRONTIER_HANDS_AUTHORITY_SCHEMA_VERSION,
    REPO_FRONTIER_MODELING_REQUEST_CONTRACT, REPO_FRONTIER_MODELING_REQUEST_SCHEMA_VERSION,
    REPO_FRONTIER_PLAN_CANDIDATE_SCHEMA_VERSION, REPO_FRONTIER_PLAN_DECISION_CONTRACT,
    REPO_FRONTIER_PLAN_DECISION_RECEIPT_SCHEMA_VERSION,
    REPO_FRONTIER_PLAN_MIND_LAUNCH_BINDING_CONTRACT,
    REPO_FRONTIER_PLAN_MIND_LAUNCH_BINDING_SCHEMA_VERSION,
    REPO_FRONTIER_PLAN_MIND_REQUEST_CONTRACT, REPO_FRONTIER_PLAN_MIND_REQUEST_SCHEMA_VERSION,
    REPO_FRONTIER_PLANNING_CONTRACT, REPO_FRONTIER_PLANNING_LAUNCH_BINDING_CONTRACT,
    REPO_FRONTIER_PLANNING_LAUNCH_BINDING_SCHEMA_VERSION,
    REPO_FRONTIER_PLANNING_REQUEST_SCHEMA_VERSION,
    REPO_FRONTIER_PROPOSAL_MODELING_LAUNCH_BINDING_CONTRACT,
    REPO_FRONTIER_PROPOSAL_MODELING_LAUNCH_BINDING_SCHEMA_VERSION,
    REPO_FRONTIER_PROPOSAL_MODELING_REQUEST_CONTRACT,
    REPO_FRONTIER_PROPOSAL_MODELING_REQUEST_SCHEMA_VERSION,
    REPO_FRONTIER_RESEARCH_REQUEST_CONTRACT, REPO_FRONTIER_RESEARCH_REQUEST_SCHEMA_VERSION,
    REPO_FRONTIER_ROUTE_CONTRACT, REPO_FRONTIER_ROUTE_SCHEMA_VERSION,
    REPO_FRONTIER_WORK_PROPOSAL_CONTRACT, REPO_FRONTIER_WORK_PROPOSAL_SCHEMA_VERSION,
    REPO_MODEL_CLAIM_CHALLENGE_CONTRACT, REPO_MODEL_CLAIM_CHALLENGE_SCHEMA_VERSION,
    REPO_MODEL_CLAIM_REPAIR_REQUEST_CONTRACT, REPO_MODEL_CLAIM_REPAIR_REQUEST_SCHEMA_VERSION,
    RUNTIME_REPOSITORY_DOMAIN_BINDING_CONTRACT, RUNTIME_REPOSITORY_DOMAIN_BINDING_KEY,
    RUNTIME_REPOSITORY_DOMAIN_BINDING_SCHEMA_VERSION, RepoFrontierAutonomousProposalBinding,
    RepoFrontierExecutionAmendmentReceipt, RepoFrontierHandsAuthority, RepoFrontierModelingRequest,
    RepoFrontierNextOrgan, RepoFrontierPlanCandidate, RepoFrontierPlanDecision,
    RepoFrontierPlanDecisionReceipt, RepoFrontierPlanMindDecision,
    RepoFrontierPlanMindLaunchBinding, RepoFrontierPlanMindRequest,
    RepoFrontierPlanningCandidateEligibility, RepoFrontierPlanningEligibility,
    RepoFrontierPlanningLaunchBinding, RepoFrontierPlanningLifecycle,
    RepoFrontierPlanningLifecycleStage, RepoFrontierPlanningRequest,
    RepoFrontierProposalModelingLaunchBinding, RepoFrontierProposalModelingRequest,
    RepoFrontierRelinquishmentReceipt, RepoFrontierResearchRequest, RepoFrontierRoute,
    RepoFrontierVerdictDisposition, RepoFrontierWorkProposal, RepoModelClaimChallenge,
    RepoModelClaimRepairFrontierRef, RepoModelClaimRepairLaunchBinding,
    RepoModelClaimRepairRequest, RuntimeRepositoryDomainBinding,
};
use crate::runtime_store_backend::{
    RuntimeSpineBackingStore as SingleFileMessagePackBackingStore, runtime_spine_backing_store,
};
use crate::soul_gateway::SoulVerdictReceipt;
use crate::soul_gateway::*;
use crate::state_ledger::STATE_LEDGER_SCHEMA_VERSION;
use crate::state_ledger::STATE_LEDGER_STORE_TYPE;
use crate::substrate_gate::SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION;
use crate::substrate_gate::SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE;
use crate::substrate_gate::SUBSTRATE_GATE_REPO_ACCESS_REFUSAL_RECEIPT_SCHEMA_VERSION;
use crate::substrate_gate::SUBSTRATE_GATE_REPO_ACCESS_REFUSAL_RECEIPT_TYPE;
use crate::substrate_gate::SUBSTRATE_GATE_REPO_ACCESS_REQUEST_SCHEMA_VERSION;
use crate::substrate_gate::SUBSTRATE_GATE_REPO_ACCESS_REQUEST_TYPE;
use crate::substrate_gate::SUBSTRATE_GATE_REPO_ACCESS_REVIEW_SCHEMA_VERSION;
use crate::substrate_gate::SUBSTRATE_GATE_REPO_ACCESS_REVIEW_TYPE;
use crate::substrate_gate::SUBSTRATE_GATE_REPO_MUTATION_RECEIPT_SCHEMA_VERSION;
use crate::substrate_gate::SUBSTRATE_GATE_REPO_MUTATION_RECEIPT_TYPE;
use crate::substrate_gate::SUBSTRATE_GATE_REPO_SNAPSHOT_RECEIPT_SCHEMA_VERSION;
use crate::substrate_gate::SUBSTRATE_GATE_REPO_SNAPSHOT_RECEIPT_TYPE;
use crate::substrate_gate::SubstrateGateRepoAccessGrantReceipt;
use crate::thread_state_store::THREAD_STATE_SCHEMA_VERSION;
use crate::thread_state_store::THREAD_STATE_TYPE;
use crate::{RuntimeTypedRequestRef, WorkerProcessStatus};
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use cultcache_rs::CacheBackingStore;
use cultcache_rs::CultCache;
use cultcache_rs::CultCacheEnvelope;
use cultcache_rs::DatabaseEntry;
use cultnet_rs::CultNetDocumentMutationContract;
use cultnet_rs::CultNetDocumentOperation;
use cultnet_rs::CultNetMessage;
use cultnet_rs::CultNetMutationAuthority;
use cultnet_rs::CultNetSchemaKind;
use cultnet_rs::CultNetSchemaRegistration;
use cultnet_rs::CultNetSchemaRegistry;
use cultnet_rs::CultNetWireContract;
use cultnet_rs::builtin_schema_registry;
use cultnet_rs::encode_cultnet_message_to_vec;
use cultnet_rs::encode_frame;
use epiphany_model_adapter::EpiphanyModelAdapterStatus;
use epiphany_model_adapter::EpiphanyModelReceipt;
use epiphany_model_adapter::EpiphanyModelRequest;
use epiphany_model_adapter::EpiphanyModelStreamEvent;
use epiphany_model_adapter::EpiphanyModelStreamPayload;
use epiphany_openai_adapter::EpiphanyOpenAiAdapterStatus;
use epiphany_openai_adapter::EpiphanyOpenAiModelReceipt;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
use epiphany_openai_adapter::EpiphanyOpenAiStreamEvent;
use epiphany_openai_adapter::EpiphanyOpenAiStreamPayload;
use epiphany_state_model::EpiphanyJobBinding;
use epiphany_state_model::EpiphanyJobKind;
use epiphany_state_model::EpiphanyRuntimeLink;
use epiphany_state_model::EpiphanyThreadState;
use epiphany_tool_adapter::EpiphanyToolCapability;
use epiphany_tool_adapter::EpiphanyToolInvocationIntent;
use epiphany_tool_adapter::EpiphanyToolInvocationReceipt;
use epiphany_tool_adapter::TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID;
use epiphany_tool_adapter::TOOL_ADAPTER_INVOCATION_RECEIPT_SCHEMA_ID;
use epiphany_tool_adapter::tool_invocation_intent_key;
use epiphany_tool_adapter::tool_invocation_receipt_key;
use serde::Deserialize;
use serde::Serialize;
use sha2::Digest;
use sha2::Sha256;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub const RUNTIME_IDENTITY_TYPE: &str = "epiphany.runtime.identity";
pub const RUNTIME_SESSION_TYPE: &str = "epiphany.runtime.session";
pub const RUNTIME_JOB_TYPE: &str = "epiphany.runtime.job";
pub const RUNTIME_MODEL_EXECUTION_BINDING_TYPE: &str = "epiphany.runtime.model_execution_binding";
pub const RUNTIME_TOOL_EXECUTION_BINDING_TYPE: &str = "epiphany.runtime.tool_execution_binding";
pub const ARCHIVED_RUNTIME_SESSION_TYPE: &str = "epiphany.runtime.archived_session";
pub const RUNTIME_WORKER_LAUNCH_REQUEST_TYPE: &str = "epiphany.runtime.worker_launch_request";
pub const RUNTIME_WORKER_PROCESS_CLAIM_TYPE: &str = "epiphany.runtime.worker_process_claim.v0";
pub const ARCHIVED_RUNTIME_WORKER_ATTEMPT_TYPE: &str =
    "epiphany.runtime.archived_worker_attempt.v0";
pub const RUNTIME_ROLE_WORKER_RESULT_TYPE: &str = "epiphany.runtime.role_worker_result";
pub const RUNTIME_REORIENT_WORKER_RESULT_TYPE: &str = "epiphany.runtime.reorient_worker_result";
pub const RUNTIME_JOB_RESULT_TYPE: &str = "epiphany.runtime.job_result";
pub const RUNTIME_EVENT_TYPE: &str = "epiphany.runtime.event";
pub const COORDINATOR_RUN_RECEIPT_TYPE: &str = "epiphany.coordinator_run_receipt.v0";
pub const COORDINATOR_RUN_RECEIPT_RETENTION_HEAD_SCHEMA_VERSION: &str =
    "epiphany.coordinator_run_receipt_retention_head.v0";
pub const COORDINATOR_RUN_RECEIPT_RETENTION_HEAD_KEY: &str = "coordinator-run-receipt-retention";
pub const OPENAI_ADAPTER_STATUS_TYPE: &str = "epiphany.openai_adapter_status.v0";
pub const OPENAI_MODEL_REQUEST_TYPE: &str = "epiphany.openai_model_request.v0";
pub const OPENAI_MODEL_STREAM_EVENT_TYPE: &str = "epiphany.openai_model_stream_event.v0";
pub const OPENAI_MODEL_RECEIPT_TYPE: &str = "epiphany.openai_model_receipt.v0";
pub const MODEL_ADAPTER_STATUS_TYPE: &str = "epiphany.model_adapter_status.v0";
pub const MODEL_REQUEST_TYPE: &str = "epiphany.model_request.v0";
pub const MODEL_STREAM_EVENT_TYPE: &str = "epiphany.model_stream_event.v0";
pub const MODEL_RECEIPT_TYPE: &str = "epiphany.model_receipt.v0";
pub const TOOL_CAPABILITY_TYPE: &str = "epiphany.tool_capability.v0";
pub const TOOL_INVOCATION_INTENT_TYPE: &str = "epiphany.tool_invocation_intent.v0";
pub const TOOL_INVOCATION_RECEIPT_TYPE: &str = "epiphany.tool_invocation_receipt.v0";
pub const RUNTIME_IDENTITY_KEY: &str = "self";
pub const RUNTIME_SWARM_BINDING_KEY: &str = "runtime-swarm-binding";
pub const RUNTIME_SWARM_BINDING_SCHEMA_VERSION: &str = "epiphany.runtime.swarm_binding.v0";
pub const RUNTIME_SPINE_SCHEMA_VERSION: &str = "epiphany.runtime_spine.v0";
pub const EPIPHANY_RUNTIME_ROOT_SESSION_ID: &str = "epiphany-main";
pub const RUNTIME_MODEL_EXECUTION_BINDING_SCHEMA_VERSION: &str =
    "epiphany.runtime.model_execution_binding.v0";
pub const RUNTIME_TOOL_EXECUTION_BINDING_SCHEMA_VERSION: &str =
    "epiphany.runtime.tool_execution_binding.v0";
pub const ARCHIVED_RUNTIME_SESSION_SCHEMA_VERSION: &str = "epiphany.runtime.archived_session.v0";
pub const RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION: &str =
    "epiphany.runtime.worker_launch_request.v1";
pub const RUNTIME_WORKER_PROCESS_CLAIM_SCHEMA_VERSION: &str =
    "epiphany.runtime.worker_process_claim.v0";
pub const ARCHIVED_RUNTIME_WORKER_ATTEMPT_SCHEMA_VERSION: &str =
    "epiphany.runtime.archived_worker_attempt.v0";
pub const RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION: &str =
    "epiphany.runtime.role_worker_result.v3";
pub const RUNTIME_REORIENT_WORKER_RESULT_SCHEMA_VERSION: &str =
    "epiphany.runtime.reorient_worker_result.v0";
pub const COORDINATOR_RUN_RECEIPT_SCHEMA_VERSION: &str = "epiphany.coordinator_run_receipt.v0";
pub const OPENAI_ADAPTER_STATUS_SCHEMA_VERSION: &str = "epiphany.openai_adapter_status.v0";
pub const OPENAI_MODEL_REQUEST_SCHEMA_VERSION: &str = "epiphany.openai_model_request.v0";
pub const OPENAI_MODEL_STREAM_EVENT_SCHEMA_VERSION: &str = "epiphany.openai_model_stream_event.v0";
pub const OPENAI_MODEL_RECEIPT_SCHEMA_VERSION: &str = "epiphany.openai_model_receipt.v0";
pub const MODEL_ADAPTER_STATUS_SCHEMA_VERSION: &str = "epiphany.model_adapter_status.v0";
pub const MODEL_REQUEST_SCHEMA_VERSION: &str = "epiphany.model_request.v0";
pub const MODEL_STREAM_EVENT_SCHEMA_VERSION: &str = "epiphany.model_stream_event.v0";
pub const MODEL_RECEIPT_SCHEMA_VERSION: &str = "epiphany.model_receipt.v0";
pub const TOOL_CAPABILITY_SCHEMA_VERSION: &str = "epiphany.tool_capability.v0";
pub const TOOL_INVOCATION_INTENT_SCHEMA_VERSION: &str = "epiphany.tool_invocation_intent.v0";
pub const TOOL_INVOCATION_RECEIPT_SCHEMA_VERSION: &str = "epiphany.tool_invocation_receipt.v0";
pub const AGENT_MEMORY_PAYLOAD_SCHEMA_VERSION: &str = "epiphany.agent_memory.v0";
pub const CULTNET_SCHEMA_INDEX_RELATIVE: &str = "schemas/cultnet/index.json";

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpiphanyCultNetSchemaIndex {
    schema_version: String,
    schemas: Vec<EpiphanyCultNetSchemaIndexEntry>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct EpiphanyCultNetSchemaIndexEntry {
    schema_id: String,
    kind: CultNetSchemaKind,
    wire_contracts: Vec<CultNetWireContract>,
    #[serde(default)]
    schema_version: Option<String>,
    #[serde(default)]
    document_type: Option<String>,
    #[serde(default)]
    title: Option<String>,
    path: String,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(type = "epiphany.runtime.identity", schema = "EpiphanyRuntimeIdentity")]
pub struct EpiphanyRuntimeIdentity {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub runtime_id: String,
    #[cultcache(key = 2)]
    pub display_name: String,
    #[cultcache(key = 3)]
    pub runtime_kind: String,
    #[cultcache(key = 4)]
    pub created_at: String,
    #[cultcache(key = 5)]
    pub updated_at: String,
    #[cultcache(key = 6)]
    pub supported_document_types: Vec<String>,
    #[cultcache(key = 7, default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.swarm_binding",
    schema = "EpiphanyRuntimeSwarmBinding"
)]
pub struct EpiphanyRuntimeSwarmBinding {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub binding_id: String,
    #[cultcache(key = 2)]
    pub runtime_id: String,
    #[cultcache(key = 3)]
    pub swarm_id: String,
    #[cultcache(key = 4)]
    pub source_identity_type: String,
    #[cultcache(key = 5)]
    pub source_identity_key: String,
    #[cultcache(key = 6)]
    pub source_identity_sha256: String,
    #[cultcache(key = 7)]
    pub bound_at: String,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(type = "epiphany.runtime.session", schema = "EpiphanyRuntimeSession")]
pub struct EpiphanyRuntimeSession {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub session_id: String,
    #[cultcache(key = 2)]
    pub objective: String,
    #[cultcache(key = 3)]
    pub status: EpiphanyRuntimeSessionStatus,
    #[cultcache(key = 4)]
    pub created_at: String,
    #[cultcache(key = 5)]
    pub updated_at: String,
    #[cultcache(key = 6, default)]
    pub coordinator_note: String,
    #[cultcache(key = 7, default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(type = "epiphany.runtime.job", schema = "EpiphanyRuntimeJob")]
pub struct EpiphanyRuntimeJob {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub job_id: String,
    #[cultcache(key = 2)]
    pub session_id: String,
    #[cultcache(key = 3)]
    pub role: String,
    #[cultcache(key = 4)]
    pub status: EpiphanyRuntimeJobStatus,
    #[cultcache(key = 5)]
    pub created_at: String,
    #[cultcache(key = 6)]
    pub updated_at: String,
    #[cultcache(key = 7, default)]
    pub summary: String,
    #[cultcache(key = 8, default)]
    pub artifact_refs: Vec<String>,
    #[cultcache(key = 9, default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.model_execution_binding",
    schema = "EpiphanyRuntimeModelExecutionBinding"
)]
pub struct EpiphanyRuntimeModelExecutionBinding {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub binding_id: String,
    #[cultcache(key = 2)]
    pub request_id: String,
    #[cultcache(key = 3)]
    pub session_id: String,
    #[cultcache(key = 4)]
    pub job_id: String,
    #[cultcache(key = 5)]
    pub provider: String,
    #[cultcache(key = 6)]
    pub bound_at: String,
    #[cultcache(key = 7, default)]
    pub source_worker_job_id: Option<String>,
    #[cultcache(key = 8, default)]
    pub reasoning_basis_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.tool_execution_binding",
    schema = "EpiphanyRuntimeToolExecutionBinding"
)]
pub struct EpiphanyRuntimeToolExecutionBinding {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub binding_id: String,
    #[cultcache(key = 2)]
    pub intent_id: String,
    #[cultcache(key = 3)]
    pub session_id: String,
    #[cultcache(key = 4)]
    pub job_id: String,
    #[cultcache(key = 5, default)]
    pub model_request_id: Option<String>,
    #[cultcache(key = 6)]
    pub bound_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.archived_session",
    schema = "EpiphanyArchivedRuntimeSession"
)]
pub struct EpiphanyArchivedRuntimeSession {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub archive_id: String,
    #[cultcache(key = 2)]
    pub session_id: String,
    #[cultcache(key = 3)]
    pub archived_at: String,
    #[cultcache(key = 4)]
    pub job_ids: Vec<String>,
    #[cultcache(key = 5)]
    pub job_result_ids: Vec<String>,
    #[cultcache(key = 6)]
    pub model_request_ids: Vec<String>,
    #[cultcache(key = 7)]
    pub tool_intent_ids: Vec<String>,
    #[cultcache(key = 8)]
    pub terminal_job_status_counts: BTreeMap<String, u64>,
    #[cultcache(key = 9)]
    pub retired_type_counts: BTreeMap<String, u64>,
    #[cultcache(key = 10)]
    pub retired_envelope_count: u64,
    #[cultcache(key = 11)]
    pub retired_chain_digest: String,
    #[cultcache(key = 12, default)]
    pub reasoning_basis_ids: Vec<String>,
    #[cultcache(key = 13, default)]
    pub decision_context_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.worker_launch_request",
    schema = "EpiphanyRuntimeWorkerLaunchRequest"
)]
pub struct EpiphanyRuntimeWorkerLaunchRequest {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub job_id: String,
    #[cultcache(key = 2)]
    pub binding_id: String,
    #[cultcache(key = 3)]
    pub role: String,
    #[cultcache(key = 4)]
    pub authority_scope: String,
    #[cultcache(key = 5)]
    pub instruction: String,
    #[cultcache(key = 6)]
    pub output_contract_id: String,
    #[cultcache(key = 7)]
    pub document_kind: String,
    #[cultcache(key = 8)]
    pub launch_document_msgpack: Vec<u8>,
    #[cultcache(key = 9, default)]
    pub metadata: BTreeMap<String, String>,
    #[cultcache(key = 10, default)]
    pub organ_launch_contract: EpiphanyLaunchOrganContract,
    #[cultcache(key = 11, default)]
    pub proposal_modeling_request_id: Option<String>,
    #[cultcache(key = 12, default)]
    pub claim_repair_request_id: Option<String>,
    #[cultcache(key = 13, default)]
    pub frontier_planning_request_id: Option<String>,
    #[cultcache(key = 14, default)]
    pub frontier_plan_mind_request_id: Option<String>,
    #[cultcache(key = 15, default)]
    pub imagination_consideration_request_id: Option<String>,
    #[cultcache(key = 16, default)]
    pub admitted_model_direction_consideration_request_id: Option<String>,
    #[cultcache(key = 17, default)]
    pub repo_frontier_modeling_request_id: Option<String>,
    #[cultcache(key = 18, default)]
    pub repo_frontier_verdict_modeling_authority_msgpack: Option<Vec<u8>>,
    #[cultcache(key = 19, default)]
    pub repo_frontier_research_request_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.worker_process_claim.v0",
    schema = "EpiphanyRuntimeWorkerProcessClaim"
)]
pub struct EpiphanyRuntimeWorkerProcessClaim {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub claim_id: String,
    #[cultcache(key = 2)]
    pub job_id: String,
    #[cultcache(key = 3)]
    pub process_id: u32,
    #[cultcache(key = 4)]
    pub process_creation_token: u64,
    #[cultcache(key = 5)]
    pub process_executable_path: String,
    #[cultcache(key = 6)]
    pub activation_token_sha256: String,
    #[cultcache(key = 7)]
    pub status: String,
    #[cultcache(key = 8)]
    pub claimed_at: String,
    #[cultcache(key = 9, default)]
    pub activated_at: Option<String>,
    #[cultcache(key = 10, default)]
    pub terminal_at: Option<String>,
    #[cultcache(key = 11, default)]
    pub terminal_authority_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.archived_worker_attempt.v0",
    schema = "EpiphanyArchivedRuntimeWorkerAttempt"
)]
pub struct EpiphanyArchivedRuntimeWorkerAttempt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub archive_id: String,
    #[cultcache(key = 2)]
    pub job_id: String,
    #[cultcache(key = 3)]
    pub request_kind: String,
    #[cultcache(key = 4)]
    pub request_id: String,
    #[cultcache(key = 5)]
    pub terminal_process_status: String,
    #[cultcache(key = 6, default)]
    pub result_id: Option<String>,
    #[cultcache(key = 7)]
    pub archived_at: String,
    #[cultcache(key = 8)]
    pub retired_type_counts: BTreeMap<String, u64>,
    #[cultcache(key = 9)]
    pub retired_envelope_count: u64,
    #[cultcache(key = 10)]
    pub retired_chain_digest: String,
    #[cultcache(key = 11, default)]
    pub decision_context_id: Option<String>,
}

fn worker_process_claim_id(job_id: &str) -> String {
    format!("runtime-worker-process-{job_id}")
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoFrontierVerdictModelingLaunchAuthority {
    pub request: RepoFrontierModelingRequest,
    pub frontier_item: crate::RepoFrontierItem,
}

impl EpiphanyRuntimeWorkerLaunchRequest {
    pub fn repo_frontier_verdict_modeling_authority(
        &self,
    ) -> Result<Option<RepoFrontierVerdictModelingLaunchAuthority>> {
        decode_optional_msgpack(
            self.repo_frontier_verdict_modeling_authority_msgpack
                .as_deref(),
            "verdict-bound Modeling launch authority",
        )
    }

    pub fn launch_document(&self) -> Result<EpiphanyWorkerLaunchDocument> {
        let document: EpiphanyWorkerLaunchDocument =
            rmp_serde::from_slice(&self.launch_document_msgpack)
                .context("failed to decode worker launch document MessagePack")?;
        let actual_kind = worker_launch_document_kind(&document);
        if actual_kind != self.document_kind {
            return Err(anyhow!(
                "worker launch document kind mismatch: indexed {:?}, payload {:?}",
                self.document_kind,
                actual_kind
            ));
        }
        Ok(document)
    }

    pub fn repository_body_observation_basis(
        &self,
    ) -> Result<Option<crate::RepositoryBodyObservationBasis>> {
        Ok(match self.launch_document()? {
            EpiphanyWorkerLaunchDocument::Role(document) => {
                document.repository_body_observation_basis
            }
            EpiphanyWorkerLaunchDocument::Reorient(_) => None,
        })
    }
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.role_worker_result",
    schema = "EpiphanyRuntimeRoleWorkerResult"
)]
pub struct EpiphanyRuntimeRoleWorkerResult {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub result_id: String,
    #[cultcache(key = 2)]
    pub job_id: String,
    #[cultcache(key = 3)]
    pub role_id: String,
    #[cultcache(key = 4)]
    pub verdict: String,
    #[cultcache(key = 5)]
    pub summary: String,
    #[cultcache(key = 6)]
    pub next_safe_move: String,
    #[cultcache(key = 7, default)]
    pub checkpoint_summary: Option<String>,
    #[cultcache(key = 8, default)]
    pub scratch_summary: Option<String>,
    #[cultcache(key = 9, default)]
    pub files_inspected: Vec<String>,
    #[cultcache(key = 10, default)]
    pub frontier_node_ids: Vec<String>,
    #[cultcache(key = 11, default)]
    pub evidence_ids: Vec<String>,
    #[cultcache(key = 12, default)]
    pub artifact_refs: Vec<String>,
    #[cultcache(key = 13, default)]
    pub open_questions: Vec<String>,
    #[cultcache(key = 14, default)]
    pub evidence_gaps: Vec<String>,
    #[cultcache(key = 15, default)]
    pub risks: Vec<String>,
    #[cultcache(key = 16, default)]
    pub state_patch_msgpack: Option<Vec<u8>>,
    #[cultcache(key = 17, default)]
    pub self_patch_msgpack: Option<Vec<u8>>,
    #[cultcache(key = 18, default)]
    pub item_error: Option<String>,
    #[cultcache(key = 19, default)]
    pub metadata: BTreeMap<String, String>,
    #[cultcache(key = 20, default)]
    pub repo_model_mutation_proposal_msgpack: Option<Vec<u8>>,
    #[cultcache(key = 21, default)]
    pub verification_request_id: Option<String>,
    #[cultcache(key = 22, default)]
    pub frontier_route_id: Option<String>,
    #[cultcache(key = 23, default)]
    pub repo_frontier_modeling_request_id: Option<String>,
    #[cultcache(key = 24, default)]
    pub proposal_modeling_request_id: Option<String>,
    #[cultcache(key = 25, default)]
    pub claim_repair_request_id: Option<String>,
    #[cultcache(key = 26, default)]
    pub frontier_planning_request_id: Option<String>,
    #[cultcache(key = 27, default)]
    pub frontier_plan_candidate_msgpack: Option<Vec<u8>>,
    #[cultcache(key = 28, default)]
    pub frontier_plan_mind_request_id: Option<String>,
    #[cultcache(key = 29, default)]
    pub frontier_plan_mind_decision_msgpack: Option<Vec<u8>>,
    #[cultcache(key = 30, default)]
    pub repository_body_observation_basis: Option<crate::RepositoryBodyObservationBasis>,
    #[cultcache(key = 31, default)]
    pub imagination_consideration_request_id: Option<String>,
    #[cultcache(key = 32, default)]
    pub imagination_consideration_candidate_msgpack: Option<Vec<u8>>,
    #[cultcache(key = 33, default)]
    pub admitted_model_direction_consideration_request_id: Option<String>,
    #[cultcache(key = 34, default)]
    pub admitted_model_direction_consideration_result_msgpack: Option<Vec<u8>>,
    #[cultcache(key = 35)]
    pub decision_context_id: String,
}

impl EpiphanyRuntimeRoleWorkerResult {
    pub fn state_patch(&self) -> Result<Option<crate::EpiphanyRoleStatePatchDocument>> {
        decode_optional_msgpack(
            self.state_patch_msgpack.as_deref(),
            "role worker statePatch",
        )
    }

    pub fn self_patch(&self) -> Result<Option<crate::AgentSelfPatch>> {
        decode_optional_msgpack(self.self_patch_msgpack.as_deref(), "role worker selfPatch")
    }

    pub fn repo_model_mutation_proposal(
        &self,
    ) -> Result<Option<crate::EpiphanyRepoModelMutationProposal>> {
        decode_optional_msgpack(
            self.repo_model_mutation_proposal_msgpack.as_deref(),
            "role worker RepoModel mutation proposal",
        )
    }

    pub fn frontier_plan_candidate(&self) -> Result<Option<RepoFrontierPlanCandidate>> {
        decode_optional_msgpack(
            self.frontier_plan_candidate_msgpack.as_deref(),
            "role worker frontierPlanCandidate",
        )
    }

    pub fn frontier_plan_mind_decision(&self) -> Result<Option<RepoFrontierPlanMindDecision>> {
        decode_optional_msgpack(
            self.frontier_plan_mind_decision_msgpack.as_deref(),
            "role worker frontierPlanMindDecision",
        )
    }

    pub fn imagination_consideration_candidate(
        &self,
    ) -> Result<Option<crate::ImaginationConsiderationCandidate>> {
        decode_optional_msgpack(
            self.imagination_consideration_candidate_msgpack.as_deref(),
            "role worker imaginationConsiderationCandidate",
        )
    }

    pub fn admitted_model_direction_consideration_result(
        &self,
    ) -> Result<Option<crate::AdmittedModelDirectionConsiderationResult>> {
        decode_optional_msgpack(
            self.admitted_model_direction_consideration_result_msgpack
                .as_deref(),
            "role worker admittedModelDirectionConsiderationResult",
        )
    }
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.reorient_worker_result",
    schema = "EpiphanyRuntimeReorientWorkerResult"
)]
pub struct EpiphanyRuntimeReorientWorkerResult {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub result_id: String,
    #[cultcache(key = 2)]
    pub job_id: String,
    #[cultcache(key = 3)]
    pub mode: String,
    #[cultcache(key = 4)]
    pub summary: String,
    #[cultcache(key = 5)]
    pub next_safe_move: String,
    #[cultcache(key = 6, default)]
    pub checkpoint_still_valid: Option<bool>,
    #[cultcache(key = 7, default)]
    pub files_inspected: Vec<String>,
    #[cultcache(key = 8, default)]
    pub frontier_node_ids: Vec<String>,
    #[cultcache(key = 9, default)]
    pub evidence_ids: Vec<String>,
    #[cultcache(key = 10, default)]
    pub artifact_refs: Vec<String>,
    #[cultcache(key = 11, default)]
    pub open_questions: Vec<String>,
    #[cultcache(key = 12, default)]
    pub continuity_risks: Vec<String>,
    #[cultcache(key = 13, default)]
    pub item_error: Option<String>,
    #[cultcache(key = 14, default)]
    pub metadata: BTreeMap<String, String>,
    #[cultcache(key = 15)]
    pub decision_context_id: String,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.job_result",
    schema = "EpiphanyRuntimeJobResult"
)]
pub struct EpiphanyRuntimeJobResult {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub result_id: String,
    #[cultcache(key = 2)]
    pub job_id: String,
    #[cultcache(key = 3)]
    pub session_id: String,
    #[cultcache(key = 4)]
    pub role: String,
    #[cultcache(key = 5)]
    pub verdict: String,
    #[cultcache(key = 6)]
    pub summary: String,
    #[cultcache(key = 7)]
    pub completed_at: String,
    #[cultcache(key = 8, default)]
    pub next_safe_move: String,
    #[cultcache(key = 9, default)]
    pub evidence_refs: Vec<String>,
    #[cultcache(key = 10, default)]
    pub artifact_refs: Vec<String>,
    #[cultcache(key = 11, default)]
    pub metadata: BTreeMap<String, String>,
    #[cultcache(key = 12, default)]
    pub decision_context_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(type = "epiphany.runtime.event", schema = "EpiphanyRuntimeEvent")]
pub struct EpiphanyRuntimeEvent {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub event_id: String,
    #[cultcache(key = 2)]
    pub occurred_at: String,
    #[cultcache(key = 3)]
    pub event_type: String,
    #[cultcache(key = 4)]
    pub source: String,
    #[cultcache(key = 5, default)]
    pub session_id: Option<String>,
    #[cultcache(key = 6, default)]
    pub job_id: Option<String>,
    #[cultcache(key = 7, default)]
    pub summary: String,
    #[cultcache(key = 8, default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.coordinator_run_receipt.v0",
    schema = "EpiphanyCoordinatorRunReceipt"
)]
pub struct EpiphanyCoordinatorRunReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub session_id: String,
    #[cultcache(key = 3)]
    pub thread_id: String,
    #[cultcache(key = 4)]
    pub mode: String,
    #[cultcache(key = 5)]
    pub status: String,
    #[cultcache(key = 6)]
    pub final_action: String,
    #[cultcache(key = 7, default)]
    pub final_reason: Option<String>,
    #[cultcache(key = 8)]
    pub step_count: u64,
    #[cultcache(key = 9)]
    pub created_at: String,
    #[cultcache(key = 10, default)]
    pub model_provider: Option<String>,
    #[cultcache(key = 11, default)]
    pub runtime_store: String,
    #[cultcache(key = 12, default)]
    pub artifact_refs: Vec<String>,
    #[cultcache(key = 13, default)]
    pub sealed_artifact_refs: Vec<String>,
    #[cultcache(key = 14, default)]
    pub metadata: BTreeMap<String, String>,
    #[cultcache(key = 15, default)]
    pub resident_grant_id: Option<String>,
    #[cultcache(key = 16, default)]
    pub resident_launch_digest: Option<String>,
    #[cultcache(key = 17, default)]
    pub resident_policy_digest: Option<String>,
    #[cultcache(key = 18, default)]
    pub resident_argv_digest: Option<String>,
    #[cultcache(key = 19, default)]
    pub resident_objective_digest: Option<String>,
    #[cultcache(key = 20, default)]
    pub resident_release_commit: Option<String>,
    #[cultcache(key = 21, default)]
    pub resident_release_manifest_digest: Option<String>,
    #[cultcache(key = 22, default)]
    pub resident_executable_digest: Option<String>,
    #[cultcache(key = 23, default)]
    pub final_runtime_job_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.coordinator_run_receipt_retention_head.v0",
    schema = "EpiphanyCoordinatorRunReceiptRetentionHead"
)]
pub struct EpiphanyCoordinatorRunReceiptRetentionHead {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub revision: u64,
    #[cultcache(key = 2)]
    pub retired_receipt_count: u64,
    #[cultcache(key = 3)]
    pub retired_status_counts: BTreeMap<String, u64>,
    #[cultcache(key = 4)]
    pub retired_chain_digest: String,
    #[cultcache(key = 5)]
    pub retained_at: String,
    #[cultcache(key = 6, default)]
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyRuntimeSessionStatus {
    #[default]
    Proposed,
    Active,
    WaitingForReview,
    Sleeping,
    Completed,
    Archived,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyRuntimeJobStatus {
    #[default]
    Queued,
    Running,
    WaitingForReview,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyRuntimeSpineStatus {
    pub store: String,
    pub present: bool,
    pub runtime_id: Option<String>,
    pub display_name: Option<String>,
    pub sessions: usize,
    pub active_sessions: usize,
    pub jobs: usize,
    pub open_jobs: usize,
    pub job_results: usize,
    pub events: usize,
    pub tool_invocation_intents: usize,
    pub tool_invocation_receipts: usize,
    pub pending_tool_invocations: usize,
    pub supported_document_types: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyToolInvocationStatus {
    pub intent_id: String,
    pub adapter: String,
    pub server: String,
    pub tool_name: String,
    pub call_id: Option<String>,
    pub model_request_id: Option<String>,
    pub caller: String,
    pub reason: String,
    pub created_at: String,
    pub status: String,
    pub receipt_id: Option<String>,
    pub completed_at: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSpineInitOptions {
    pub runtime_id: String,
    pub display_name: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSpineSessionOptions {
    pub session_id: String,
    pub objective: String,
    pub created_at: String,
    pub coordinator_note: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSpineSessionClosureOptions {
    pub session_id: String,
    pub completed_at: String,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSpineEventOptions {
    pub event_id: String,
    pub occurred_at: String,
    pub event_type: String,
    pub source: String,
    pub session_id: Option<String>,
    pub job_id: Option<String>,
    pub summary: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSpineJobOptions {
    pub job_id: String,
    pub session_id: String,
    pub role: String,
    pub created_at: String,
    pub summary: String,
    pub artifact_refs: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSpineJobResultOptions {
    pub result_id: String,
    pub job_id: String,
    pub completed_at: String,
    pub verdict: String,
    pub summary: String,
    pub next_safe_move: String,
    pub evidence_refs: Vec<String>,
    pub artifact_refs: Vec<String>,
    pub decision_context_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeSpineHeartbeatJobOptions {
    pub runtime_id: String,
    pub display_name: String,
    pub session_id: String,
    pub objective: String,
    pub coordinator_note: String,
    pub job_id: String,
    pub role: String,
    pub binding_id: String,
    pub authority_scope: String,
    pub instruction: String,
    pub launch_document: EpiphanyWorkerLaunchDocument,
    pub output_contract_id: String,
    pub organ_launch_contract: EpiphanyLaunchOrganContract,
    pub proposal_modeling_request_id: Option<String>,
    pub claim_repair_request_id: Option<String>,
    pub frontier_planning_request_id: Option<String>,
    pub frontier_plan_mind_request_id: Option<String>,
    pub imagination_consideration_request_id: Option<String>,
    pub admitted_model_direction_consideration_request_id: Option<String>,
    pub repo_frontier_modeling_request_id: Option<String>,
    pub repo_frontier_research_request_id: Option<String>,
    pub repo_frontier_verdict_modeling_authority:
        Option<RepoFrontierVerdictModelingLaunchAuthority>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PreparedRuntimeSpineHeartbeatJob {
    pub job: EpiphanyRuntimeJob,
    pub envelopes: Vec<CultCacheEnvelope>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeSpineHeartbeatLaunchPlanOptions {
    pub binding_id: String,
    pub kind: EpiphanyJobKind,
    pub scope: String,
    pub owner_role: String,
    pub authority_scope: String,
    pub linked_subgoal_ids: Vec<String>,
    pub linked_graph_node_ids: Vec<String>,
    pub instruction: String,
    pub launch_document: EpiphanyWorkerLaunchDocument,
    pub output_contract_id: String,
    pub organ_launch_contract: EpiphanyLaunchOrganContract,
    pub max_runtime_seconds: Option<u64>,
    pub runtime_job_id: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RuntimeSpineHeartbeatLaunchPlan {
    pub binding: EpiphanyJobBinding,
    pub runtime_link: EpiphanyRuntimeLink,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EpiphanyJobLaunchRequest {
    pub expected_revision: Option<u64>,
    pub binding_id: String,
    pub kind: EpiphanyJobKind,
    pub scope: String,
    pub owner_role: String,
    pub authority_scope: String,
    pub linked_subgoal_ids: Vec<String>,
    pub linked_graph_node_ids: Vec<String>,
    pub instruction: String,
    pub launch_document: EpiphanyWorkerLaunchDocument,
    pub output_contract_id: String,
    pub organ_launch_contract: EpiphanyLaunchOrganContract,
    pub max_runtime_seconds: Option<u64>,
    pub proposal_modeling_request_id: Option<String>,
    pub claim_repair_request_id: Option<String>,
    pub frontier_planning_request_id: Option<String>,
    pub frontier_plan_mind_request_id: Option<String>,
    pub imagination_consideration_request_id: Option<String>,
    pub admitted_model_direction_consideration_request_id: Option<String>,
    pub repo_frontier_modeling_request_id: Option<String>,
    pub repo_frontier_research_request_id: Option<String>,
    pub repo_frontier_verdict_modeling_authority:
        Option<RepoFrontierVerdictModelingLaunchAuthority>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EpiphanyJobLaunchResult {
    pub epiphany_state: EpiphanyThreadState,
    pub binding_id: String,
    pub launcher_job_id: String,
    pub backend_job_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyJobInterruptRequest {
    pub expected_revision: Option<u64>,
    pub binding_id: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EpiphanyJobInterruptResult {
    pub epiphany_state: EpiphanyThreadState,
    pub binding_id: String,
    pub cancel_requested: bool,
    pub interrupted_thread_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EpiphanyRuntimeJobSnapshot {
    pub job: EpiphanyRuntimeJob,
    pub result: Option<EpiphanyRuntimeJobResult>,
}

pub fn runtime_spine_cache(store_path: impl AsRef<Path>) -> Result<CultCache> {
    let store_path = store_path.as_ref();
    let mut cache = CultCache::new();
    crate::mind_documents::register_mind_document_types(&mut cache)?;
    cache.register_entry_type::<crate::EpiphanyThreadStateEntry>()?;
    cache.register_entry_type::<crate::UserObjectiveIntake>()?;
    cache.register_entry_type::<EpiphanyRuntimeIdentity>()?;
    cache.register_entry_type::<crate::RuntimeStoreMigrationReceipt>()?;
    cache.register_entry_type::<EpiphanyRuntimeSwarmBinding>()?;
    cache.register_entry_type::<crate::AtlasSurfaceOfferWriteIntent>()?;
    cache.register_entry_type::<crate::AtlasDependencyClaimWriteIntent>()?;
    cache.register_entry_type::<crate::AtlasDependencyVerificationWriteIntent>()?;
    cache.register_entry_type::<crate::AtlasDependencyImpactWriteIntent>()?;
    cache.register_entry_type::<crate::MemorySemanticProjectionObligation>()?;
    cache.register_entry_type::<crate::MemorySemanticProjectionClaim>()?;
    cache.register_entry_type::<crate::MemorySemanticProjectionAttempt>()?;
    cache.register_entry_type::<crate::MemorySemanticIndexReceipt>()?;
    cache.register_entry_type::<crate::MemorySemanticProjectorExecutorGrant>()?;
    cache.register_entry_type::<crate::MemorySemanticProjectorRecoveryAuthorization>()?;
    cache.register_entry_type::<crate::MemorySemanticProjectionRetentionHead>()?;
    cache.register_entry_type::<crate::MemorySemanticPhysicalRetirementObligation>()?;
    cache.register_entry_type::<crate::MemorySemanticPhysicalRetirementReceipt>()?;
    cache.register_entry_type::<EpiphanyRuntimeSession>()?;
    cache.register_entry_type::<EpiphanyRuntimeJob>()?;
    cache.register_entry_type::<EpiphanyRuntimeModelExecutionBinding>()?;
    cache.register_entry_type::<crate::EpiphanyReasoningBasis>()?;
    cache.register_entry_type::<crate::EpiphanyDecisionContext>()?;
    cache.register_entry_type::<crate::EpiphanyMindCommitReceipt>()?;
    cache.register_entry_type::<EpiphanyRuntimeToolExecutionBinding>()?;
    cache.register_entry_type::<EpiphanyArchivedRuntimeSession>()?;
    cache.register_entry_type::<EpiphanyRuntimeWorkerLaunchRequest>()?;
    cache.register_entry_type::<EpiphanyRuntimeWorkerProcessClaim>()?;
    cache.register_entry_type::<EpiphanyArchivedRuntimeWorkerAttempt>()?;
    cache.register_entry_type::<EpiphanyRuntimeRoleWorkerResult>()?;
    cache.register_entry_type::<crate::RepositoryReadinessProjection>()?;
    cache.register_entry_type::<crate::RuntimeRepositoryBodyStoreBinding>()?;
    cache.register_entry_type::<crate::RuntimeWorkspaceCoverageStoreBinding>()?;
    cache.register_entry_type::<RepoModelClaimChallenge>()?;
    cache.register_entry_type::<RepoModelClaimRepairRequest>()?;
    cache.register_entry_type::<RepoModelClaimRepairLaunchBinding>()?;
    cache.register_entry_type::<RepoFrontierRoute>()?;
    cache.register_entry_type::<RepoFrontierHandsAuthority>()?;
    cache.register_entry_type::<HandsActionRefusalReceipt>()?;
    cache.register_entry_type::<RepoFrontierModelingRequest>()?;
    cache.register_entry_type::<RepoFrontierWorkProposal>()?;
    cache.register_entry_type::<RepoFrontierAutonomousProposalBinding>()?;
    cache.register_entry_type::<RuntimeRepositoryDomainBinding>()?;
    cache.register_entry_type::<RepoFrontierProposalModelingRequest>()?;
    cache.register_entry_type::<RepoFrontierProposalModelingLaunchBinding>()?;
    cache.register_entry_type::<RepoFrontierPlanningRequest>()?;
    cache.register_entry_type::<RepoFrontierResearchRequest>()?;
    cache.register_entry_type::<RepoFrontierPlanningLaunchBinding>()?;
    cache.register_entry_type::<crate::ImaginationConsiderationRequest>()?;
    cache.register_entry_type::<crate::ImaginationConsiderationLaunchBinding>()?;
    cache.register_entry_type::<crate::ImaginationConsiderationCandidate>()?;
    cache.register_entry_type::<crate::AdmittedModelDirectionConsiderationRequest>()?;
    cache.register_entry_type::<crate::AdmittedModelDirectionConsiderationResult>()?;
    cache.register_entry_type::<crate::ImaginationConsiderationReviewRequest>()?;
    cache.register_entry_type::<RepoFrontierPlanCandidate>()?;
    cache.register_entry_type::<RepoFrontierPlanMindRequest>()?;
    cache.register_entry_type::<RepoFrontierPlanMindLaunchBinding>()?;
    cache.register_entry_type::<RepoFrontierVerificationRequest>()?;
    cache.register_entry_type::<EpiphanyRuntimeReorientWorkerResult>()?;
    cache.register_entry_type::<EpiphanyRuntimeJobResult>()?;
    cache.register_entry_type::<EpiphanyRuntimeEvent>()?;
    cache.register_entry_type::<EpiphanyCoordinatorRunReceipt>()?;
    cache.register_entry_type::<EpiphanyCoordinatorDeathRecovery>()?;
    cache.register_entry_type::<EpiphanyCoordinatorRunReceiptRetentionHead>()?;
    cache.register_entry_type::<MindGatewayReview>()?;
    cache.register_entry_type::<MindStateCommitReceipt>()?;
    cache.register_entry_type::<EyesEvidencePacket>()?;
    cache.register_entry_type::<EyesSourceLookupReceipt>()?;
    cache.register_entry_type::<SubstrateGateRepoAccessGrantReceipt>()?;
    cache.register_entry_type::<HandsActionIntent>()?;
    cache.register_entry_type::<HandsActionReview>()?;
    cache.register_entry_type::<HandsPatchReceipt>()?;
    cache.register_entry_type::<HandsCommandReceipt>()?;
    cache.register_entry_type::<HandsCommitReceipt>()?;
    cache.register_entry_type::<HandsPrReceipt>()?;
    cache.register_entry_type::<SoulVerdictReceipt>()?;
    cache.register_entry_type::<ContinuityRecoveryReceipt>()?;
    cache.register_entry_type::<EpiphanyOpenAiAdapterStatus>()?;
    cache.register_entry_type::<EpiphanyOpenAiModelRequest>()?;
    cache.register_entry_type::<EpiphanyOpenAiStreamEvent>()?;
    cache.register_entry_type::<EpiphanyOpenAiModelReceipt>()?;
    cache.register_entry_type::<EpiphanyModelAdapterStatus>()?;
    cache.register_entry_type::<EpiphanyModelRequest>()?;
    cache.register_entry_type::<EpiphanyModelStreamEvent>()?;
    cache.register_entry_type::<EpiphanyModelReceipt>()?;
    cache.register_entry_type::<crate::PersonaInterpreterEffectDocument>()?;
    cache.register_entry_type::<crate::PersonaModelStageReceipt>()?;
    cache.register_entry_type::<crate::PersonaModelTerminalReceipt>()?;
    cache.register_entry_type::<crate::PersonaDiscordDeliveryEvidence>()?;
    cache.register_entry_type::<crate::PersonaConversationExecutionReceipt>()?;
    cache.register_entry_type::<crate::PersonaEffectExecutionIntent>()?;
    cache.register_entry_type::<crate::PersonaConversationStoreRetirementReceipt>()?;
    cache.register_entry_type::<EpiphanyToolCapability>()?;
    cache.register_entry_type::<EpiphanyToolInvocationIntent>()?;
    cache.register_entry_type::<EpiphanyToolInvocationReceipt>()?;
    cache.add_generic_backing_store(runtime_spine_backing_store(store_path)?);
    Ok(cache)
}

pub fn initialize_runtime_spine(
    store_path: impl AsRef<Path>,
    options: RuntimeSpineInitOptions,
) -> Result<EpiphanyRuntimeIdentity> {
    validate_non_empty(&options.runtime_id, "runtime id")?;
    validate_non_empty(&options.display_name, "display name")?;
    validate_non_empty(&options.created_at, "created at")?;
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let existing = cache.get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?;
    let existing_mind = cache.get::<crate::EpiphanyMindIdentity>(crate::MIND_SCHEMA_EPOCH)?;
    if existing.is_some() != existing_mind.is_some() {
        return Err(anyhow!(
            "runtime and Mind schema identities are split across epochs"
        ));
    }
    if existing
        .as_ref()
        .is_some_and(|identity| identity.runtime_id != options.runtime_id)
    {
        return Err(anyhow!(
            "runtime identity cannot change during initialization"
        ));
    }
    let created_at = existing
        .as_ref()
        .map(|identity| identity.created_at.clone())
        .unwrap_or_else(|| options.created_at.clone());
    let identity = EpiphanyRuntimeIdentity {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        runtime_id: options.runtime_id,
        display_name: options.display_name,
        runtime_kind: "epiphany.native".to_string(),
        created_at,
        updated_at: options.created_at,
        supported_document_types: runtime_registered_document_types(),
        metadata: BTreeMap::from([("codexEvacuationBridge".to_string(), "temporary".to_string())]),
    };
    let mind_identity = crate::EpiphanyMindIdentity {
        schema_epoch: crate::MIND_SCHEMA_EPOCH.to_string(),
        runtime_id: identity.runtime_id.clone(),
    };
    if let Some(existing_mind) = existing_mind {
        if existing_mind != mind_identity {
            return Err(anyhow!("Mind schema identity collision"));
        }
        cache.put(RUNTIME_IDENTITY_KEY, &identity)?;
        return Ok(identity);
    }
    let runtime_envelope = cache.prepare_entry(RUNTIME_IDENTITY_KEY, &identity)?.0;
    let mind_envelope = cache
        .prepare_entry(crate::MIND_SCHEMA_EPOCH, &mind_identity)?
        .0;
    if !runtime_spine_backing_store(store_path)?
        .compare_and_swap_batch(&[], vec![runtime_envelope, mind_envelope])?
    {
        return Err(anyhow!(
            "runtime and Mind schema identities lost their atomic initialization"
        ));
    }
    Ok(identity)
}

pub fn bind_runtime_to_agent_memory_swarm(
    runtime_store: impl AsRef<Path>,
    agent_store: impl AsRef<Path>,
    bound_at: &str,
) -> Result<EpiphanyRuntimeSwarmBinding> {
    chrono::DateTime::parse_from_rfc3339(bound_at)
        .map_err(|_| anyhow!("runtime swarm binding timestamp must be RFC3339"))?;
    let source = load_agent_memory_swarm_identity(agent_store)?
        .ok_or_else(|| anyhow!("agent memory store has no canonical swarm identity"))?;
    if source.schema_version != AGENT_MEMORY_SWARM_IDENTITY_SCHEMA_VERSION {
        return Err(anyhow!("unsupported canonical agent-memory swarm identity"));
    }
    let runtime_store = runtime_store.as_ref();
    let mut cache = runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    let identity = cache
        .get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("runtime swarm binding requires runtime identity"))?;
    let binding = EpiphanyRuntimeSwarmBinding {
        schema_version: RUNTIME_SWARM_BINDING_SCHEMA_VERSION.to_string(),
        binding_id: RUNTIME_SWARM_BINDING_KEY.to_string(),
        runtime_id: identity.runtime_id.clone(),
        swarm_id: source.swarm_id.clone(),
        source_identity_type: AGENT_MEMORY_SWARM_IDENTITY_TYPE.to_string(),
        source_identity_key: AGENT_MEMORY_SWARM_IDENTITY_KEY.to_string(),
        source_identity_sha256: format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&source)?)),
        bound_at: bound_at.to_string(),
    };
    if let Some(existing) = cache.get::<EpiphanyRuntimeSwarmBinding>(RUNTIME_SWARM_BINDING_KEY)? {
        return if existing.schema_version == binding.schema_version
            && existing.binding_id == binding.binding_id
            && existing.runtime_id == binding.runtime_id
            && existing.swarm_id == binding.swarm_id
            && existing.source_identity_type == binding.source_identity_type
            && existing.source_identity_key == binding.source_identity_key
            && existing.source_identity_sha256 == binding.source_identity_sha256
        {
            Ok(existing)
        } else {
            Err(anyhow!("runtime swarm binding identity collision"))
        };
    }
    let backing = SingleFileMessagePackBackingStore::new(runtime_store);
    let identity_envelope = backing
        .pull_all()?
        .into_iter()
        .find(|entry| entry.r#type == RUNTIME_IDENTITY_TYPE && entry.key == RUNTIME_IDENTITY_KEY)
        .ok_or_else(|| anyhow!("runtime swarm binding lost runtime identity envelope"))?;
    let (binding_envelope, _) = cache.prepare_entry(RUNTIME_SWARM_BINDING_KEY, &binding)?;
    if backing.compare_and_swap_batch(
        &[identity_envelope.clone()],
        vec![identity_envelope, binding_envelope],
    )? {
        return Ok(binding);
    }
    let mut reloaded = runtime_spine_cache(runtime_store)?;
    reloaded.pull_all_backing_stores()?;
    match reloaded.get::<EpiphanyRuntimeSwarmBinding>(RUNTIME_SWARM_BINDING_KEY)? {
        Some(existing) if existing == binding => Ok(existing),
        _ => Err(anyhow!("runtime swarm binding lost immutable CAS")),
    }
}

pub fn runtime_swarm_binding(
    runtime_store: impl AsRef<Path>,
) -> Result<Option<EpiphanyRuntimeSwarmBinding>> {
    let mut cache = runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    cache.get(RUNTIME_SWARM_BINDING_KEY)
}

fn require_runtime_swarm_binding(cache: &CultCache) -> Result<EpiphanyRuntimeSwarmBinding> {
    let identity = require_identity(cache)?;
    let binding = cache
        .get::<EpiphanyRuntimeSwarmBinding>(RUNTIME_SWARM_BINDING_KEY)?
        .ok_or_else(|| anyhow!("RepoModel admission requires immutable runtime swarm binding"))?;
    if binding.schema_version != RUNTIME_SWARM_BINDING_SCHEMA_VERSION
        || binding.binding_id != RUNTIME_SWARM_BINDING_KEY
        || binding.runtime_id != identity.runtime_id
        || binding.swarm_id.trim().is_empty()
        || binding.source_identity_type != AGENT_MEMORY_SWARM_IDENTITY_TYPE
        || binding.source_identity_key != AGENT_MEMORY_SWARM_IDENTITY_KEY
        || binding.source_identity_sha256.trim().is_empty()
        || chrono::DateTime::parse_from_rfc3339(&binding.bound_at).is_err()
    {
        return Err(anyhow!("runtime swarm binding is invalid"));
    }
    Ok(binding)
}

pub fn create_runtime_session(
    store_path: impl AsRef<Path>,
    options: RuntimeSpineSessionOptions,
) -> Result<EpiphanyRuntimeSession> {
    validate_non_empty(&options.session_id, "session id")?;
    validate_non_empty(&options.objective, "objective")?;
    validate_non_empty(&options.created_at, "created at")?;
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    require_runtime_identity_not_archived(&cache, "session", &options.session_id)?;
    if cache
        .get::<EpiphanyRuntimeSession>(&options.session_id)?
        .is_some()
    {
        return Err(anyhow!(
            "runtime session {:?} already exists",
            options.session_id
        ));
    }
    let session = EpiphanyRuntimeSession {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        session_id: options.session_id.clone(),
        objective: options.objective,
        status: EpiphanyRuntimeSessionStatus::Active,
        created_at: options.created_at.clone(),
        updated_at: options.created_at,
        coordinator_note: options.coordinator_note,
        metadata: BTreeMap::new(),
    };
    cache.put(&options.session_id, &session)?;
    Ok(session)
}

pub fn ensure_runtime_session(
    store_path: impl AsRef<Path>,
    options: RuntimeSpineSessionOptions,
) -> Result<EpiphanyRuntimeSession> {
    validate_non_empty(&options.session_id, "session id")?;
    validate_non_empty(&options.objective, "objective")?;
    validate_non_empty(&options.created_at, "created at")?;
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    require_runtime_identity_not_archived(&cache, "session", &options.session_id)?;
    if let Some(existing) = cache.get::<EpiphanyRuntimeSession>(&options.session_id)? {
        if matches!(
            existing.status,
            EpiphanyRuntimeSessionStatus::Completed | EpiphanyRuntimeSessionStatus::Archived
        ) {
            return Err(anyhow!(
                "runtime session {:?} is terminal and cannot accept jobs",
                options.session_id
            ));
        }
        return Ok(existing);
    }
    let session = EpiphanyRuntimeSession {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        session_id: options.session_id.clone(),
        objective: options.objective,
        status: EpiphanyRuntimeSessionStatus::Active,
        created_at: options.created_at.clone(),
        updated_at: options.created_at,
        coordinator_note: options.coordinator_note,
        metadata: BTreeMap::new(),
    };
    cache.put(&options.session_id, &session)?;
    Ok(session)
}

pub fn close_runtime_session(
    store_path: impl AsRef<Path>,
    options: RuntimeSpineSessionClosureOptions,
) -> Result<EpiphanyRuntimeSession> {
    validate_non_empty(&options.session_id, "session id")?;
    validate_non_empty(&options.completed_at, "session completion time")?;
    validate_non_empty(&options.summary, "session completion summary")?;
    if options.session_id == EPIPHANY_RUNTIME_ROOT_SESSION_ID {
        return Err(anyhow!(
            "runtime root session {:?} is long-lived and cannot be generically completed",
            options.session_id
        ));
    }
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let mut session = cache
        .get::<EpiphanyRuntimeSession>(&options.session_id)?
        .ok_or_else(|| anyhow!("runtime session {:?} does not exist", options.session_id))?;
    if session.status == EpiphanyRuntimeSessionStatus::Completed {
        return Ok(session);
    }
    if session.status == EpiphanyRuntimeSessionStatus::Archived {
        return Err(anyhow!(
            "runtime session {:?} is archived and cannot be completed",
            options.session_id
        ));
    }
    let open_job_ids = cache
        .get_all::<EpiphanyRuntimeJob>()?
        .into_iter()
        .filter(|job| {
            job.session_id == options.session_id
                && matches!(
                    job.status,
                    EpiphanyRuntimeJobStatus::Queued
                        | EpiphanyRuntimeJobStatus::Running
                        | EpiphanyRuntimeJobStatus::WaitingForReview
                )
        })
        .map(|job| job.job_id)
        .collect::<Vec<_>>();
    if !open_job_ids.is_empty() {
        return Err(anyhow!(
            "runtime session {:?} has open jobs: {}",
            options.session_id,
            open_job_ids.join(", ")
        ));
    }
    let event_id = format!("event-session-completed-{}", options.session_id);
    if cache.get::<EpiphanyRuntimeEvent>(&event_id)?.is_some() {
        return Err(anyhow!(
            "runtime session {:?} has a completion event but is not completed",
            options.session_id
        ));
    }
    session.status = EpiphanyRuntimeSessionStatus::Completed;
    session.updated_at = options.completed_at.clone();
    session.coordinator_note = options.summary.clone();
    let event = EpiphanyRuntimeEvent {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        event_id,
        occurred_at: options.completed_at,
        event_type: "session.completed".to_string(),
        source: "continuity".to_string(),
        session_id: Some(options.session_id),
        job_id: None,
        summary: options.summary,
        metadata: BTreeMap::new(),
    };
    cache.put(&session.session_id, &session)?;
    cache.put(&event.event_id, &event)?;
    Ok(session)
}

pub fn repair_runtime_root_session_after_invalid_completion(
    store_path: impl AsRef<Path>,
    repaired_at: &str,
    reason: &str,
) -> Result<EpiphanyRuntimeSession> {
    validate_non_empty(repaired_at, "root session repair time")?;
    validate_non_empty(reason, "root session repair reason")?;
    chrono::DateTime::parse_from_rfc3339(repaired_at)
        .context("root session repair time must be RFC 3339")?;
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let snapshot = cache.snapshot_envelopes();
    let mut session = cache
        .get::<EpiphanyRuntimeSession>(EPIPHANY_RUNTIME_ROOT_SESSION_ID)?
        .ok_or_else(|| anyhow!("runtime root session does not exist"))?;
    let event_id = format!("event-session-completed-{EPIPHANY_RUNTIME_ROOT_SESSION_ID}");
    let event = cache.get::<EpiphanyRuntimeEvent>(&event_id)?;
    if session.status == EpiphanyRuntimeSessionStatus::Active {
        if event.is_some() {
            return Err(anyhow!(
                "active runtime root session has a hostile completion event"
            ));
        }
        return Ok(session);
    }
    if session.status != EpiphanyRuntimeSessionStatus::Completed {
        return Err(anyhow!(
            "runtime root session is not repairable from its current status"
        ));
    }
    if cache
        .get::<EpiphanyArchivedRuntimeSession>(EPIPHANY_RUNTIME_ROOT_SESSION_ID)?
        .is_some()
    {
        return Err(anyhow!("archived runtime root session cannot be repaired"));
    }
    let event = event
        .ok_or_else(|| anyhow!("completed runtime root session lacks its completion event"))?;
    if event.schema_version != RUNTIME_SPINE_SCHEMA_VERSION
        || event.event_type != "session.completed"
        || event.source != "continuity"
        || event.session_id.as_deref() != Some(EPIPHANY_RUNTIME_ROOT_SESSION_ID)
        || event.job_id.is_some()
        || event.occurred_at != session.updated_at
        || event.summary != session.coordinator_note
    {
        return Err(anyhow!(
            "runtime root completion event does not exactly bind the completed session"
        ));
    }
    if cache
        .get_all::<EpiphanyRuntimeJob>()?
        .into_iter()
        .any(|job| {
            job.session_id == EPIPHANY_RUNTIME_ROOT_SESSION_ID
                && matches!(
                    job.status,
                    EpiphanyRuntimeJobStatus::Queued
                        | EpiphanyRuntimeJobStatus::Running
                        | EpiphanyRuntimeJobStatus::WaitingForReview
                )
        })
    {
        return Err(anyhow!(
            "runtime root session has open jobs and cannot be repaired"
        ));
    }
    let event_envelope = snapshot
        .iter()
        .find(|entry| entry.r#type == EpiphanyRuntimeEvent::TYPE && entry.key == event_id)
        .cloned()
        .ok_or_else(|| anyhow!("runtime root completion event envelope is missing"))?;
    session.status = EpiphanyRuntimeSessionStatus::Active;
    session.updated_at = repaired_at.to_string();
    session.coordinator_note = reason.to_string();
    let replacement = cache
        .prepare_entry(EPIPHANY_RUNTIME_ROOT_SESSION_ID, &session)?
        .0;
    if !runtime_spine_backing_store(store_path)?.replace_and_delete_if_snapshot_unchanged(
        &snapshot,
        vec![replacement],
        &[event_envelope],
    )? {
        return Err(anyhow!(
            "runtime root session repair lost its full snapshot fence"
        ));
    }
    Ok(session)
}

pub fn create_runtime_job(
    store_path: impl AsRef<Path>,
    options: RuntimeSpineJobOptions,
) -> Result<EpiphanyRuntimeJob> {
    validate_non_empty(&options.job_id, "job id")?;
    validate_non_empty(&options.session_id, "session id")?;
    validate_non_empty(&options.role, "role")?;
    validate_non_empty(&options.created_at, "created at")?;
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    require_runtime_identity_not_archived(&cache, "job", &options.job_id)?;
    let session = cache
        .get::<EpiphanyRuntimeSession>(&options.session_id)?
        .ok_or_else(|| anyhow!("runtime session {:?} does not exist", options.session_id))?;
    if matches!(
        session.status,
        EpiphanyRuntimeSessionStatus::Completed | EpiphanyRuntimeSessionStatus::Archived
    ) {
        return Err(anyhow!(
            "runtime session {:?} is not open for jobs",
            options.session_id
        ));
    }
    if cache.get::<EpiphanyRuntimeJob>(&options.job_id)?.is_some() {
        return Err(anyhow!("runtime job {:?} already exists", options.job_id));
    }
    let job = EpiphanyRuntimeJob {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        job_id: options.job_id.clone(),
        session_id: options.session_id.clone(),
        role: options.role,
        status: EpiphanyRuntimeJobStatus::Queued,
        created_at: options.created_at.clone(),
        updated_at: options.created_at.clone(),
        summary: options.summary,
        artifact_refs: options.artifact_refs,
        metadata: BTreeMap::new(),
    };
    cache.put(&options.job_id, &job)?;
    let event = EpiphanyRuntimeEvent {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        event_id: format!("event-job-opened-{}", options.job_id),
        occurred_at: options.created_at,
        event_type: "job.opened".to_string(),
        source: "runtime-spine".to_string(),
        session_id: Some(options.session_id),
        job_id: Some(options.job_id),
        summary: "Native runtime job opened.".to_string(),
        metadata: BTreeMap::new(),
    };
    cache.put(&event.event_id, &event)?;
    Ok(job)
}

pub fn open_runtime_model_execution(
    store_path: impl AsRef<Path>,
    session_options: RuntimeSpineSessionOptions,
    job_options: RuntimeSpineJobOptions,
    model_request: &EpiphanyModelRequest,
    provider_request: &EpiphanyOpenAiModelRequest,
    bound_at: &str,
) -> Result<EpiphanyRuntimeModelExecutionBinding> {
    validate_non_empty(&session_options.session_id, "model execution session id")?;
    validate_non_empty(&session_options.objective, "model execution objective")?;
    validate_non_empty(
        &session_options.created_at,
        "model execution session creation time",
    )?;
    validate_non_empty(&job_options.job_id, "model execution job id")?;
    validate_non_empty(&job_options.role, "model execution job role")?;
    validate_non_empty(&job_options.created_at, "model execution job creation time")?;
    if job_options.session_id != session_options.session_id {
        return Err(anyhow!(
            "model execution job and session options disagree on session identity"
        ));
    }
    validate_non_empty(&model_request.request_id, "model execution request id")?;
    validate_non_empty(&model_request.provider, "model execution provider")?;
    validate_non_empty(bound_at, "model execution binding time")?;
    chrono::DateTime::parse_from_rfc3339(bound_at)
        .map_err(|error| anyhow!("model execution binding time is invalid: {error}"))?;
    if provider_request != &epiphany_openai_adapter::request_from_native(model_request) {
        return Err(anyhow!(
            "native and provider model requests do not describe one execution"
        ));
    }

    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    require_runtime_identity_not_archived(&cache, "session", &session_options.session_id)?;
    require_runtime_identity_not_archived(&cache, "job", &job_options.job_id)?;
    require_runtime_identity_not_archived(&cache, "model-request", &model_request.request_id)?;
    let mut reasoning_basis_envelope = None;
    let source_worker_envelopes =
        if let Some(worker_job_id) = model_request.source_worker_job_id.as_deref() {
            let basis_id = model_request.reasoning_basis_id.as_deref().ok_or_else(|| {
                anyhow!("decision-bearing worker model execution has no reasoning basis")
            })?;
            let basis = cache
                .get::<crate::EpiphanyReasoningBasis>(basis_id)?
                .ok_or_else(|| anyhow!("model execution reasoning basis is absent"))?;
            basis.validate()?;
            if basis.pass_id != worker_job_id {
                return Err(anyhow!(
                    "model execution reasoning basis belongs to another pass"
                ));
            }
            reasoning_basis_envelope = Some(
                cache
                    .get_envelope::<crate::EpiphanyReasoningBasis>(basis_id)?
                    .ok_or_else(|| anyhow!("model execution lost its reasoning basis envelope"))?,
            );
            let launch = cache
                .get::<EpiphanyRuntimeWorkerLaunchRequest>(worker_job_id)?
                .ok_or_else(|| {
                    anyhow!("model execution source worker {worker_job_id:?} has no launch request")
                })?;
            if launch.job_id != worker_job_id {
                return Err(anyhow!(
                    "model execution source worker launch identity disagrees"
                ));
            }
            let worker_job = cache
                .get::<EpiphanyRuntimeJob>(worker_job_id)?
                .ok_or_else(|| anyhow!("model execution source worker has no runtime job"))?;
            if worker_job.role != launch.role
                || matches!(
                    worker_job.status,
                    EpiphanyRuntimeJobStatus::Completed
                        | EpiphanyRuntimeJobStatus::Failed
                        | EpiphanyRuntimeJobStatus::Cancelled
                )
            {
                return Err(anyhow!(
                    "model execution source worker job is foreign or terminal"
                ));
            }
            vec![
                cache
                    .get_envelope::<EpiphanyRuntimeJob>(worker_job_id)?
                    .ok_or_else(|| anyhow!("model execution lost source worker job envelope"))?,
                cache
                    .get_envelope::<EpiphanyRuntimeWorkerLaunchRequest>(worker_job_id)?
                    .ok_or_else(|| anyhow!("model execution lost source worker launch envelope"))?,
            ]
        } else {
            Vec::new()
        };
    let existing_session = cache.get::<EpiphanyRuntimeSession>(&session_options.session_id)?;
    let session = existing_session
        .clone()
        .unwrap_or_else(|| EpiphanyRuntimeSession {
            schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
            session_id: session_options.session_id.clone(),
            objective: session_options.objective.clone(),
            status: EpiphanyRuntimeSessionStatus::Active,
            created_at: session_options.created_at.clone(),
            updated_at: session_options.created_at.clone(),
            coordinator_note: session_options.coordinator_note.clone(),
            metadata: BTreeMap::new(),
        });
    if matches!(
        session.status,
        EpiphanyRuntimeSessionStatus::Completed | EpiphanyRuntimeSessionStatus::Archived
    ) {
        return Err(anyhow!(
            "model execution session {:?} is terminal",
            session.session_id
        ));
    }
    if cache
        .get::<EpiphanyRuntimeJob>(&job_options.job_id)?
        .is_some()
    {
        return Err(anyhow!(
            "model execution job {:?} already exists",
            job_options.job_id
        ));
    }
    let job = EpiphanyRuntimeJob {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        job_id: job_options.job_id.clone(),
        session_id: session.session_id.clone(),
        role: job_options.role,
        status: EpiphanyRuntimeJobStatus::Queued,
        created_at: job_options.created_at.clone(),
        updated_at: job_options.created_at.clone(),
        summary: job_options.summary,
        artifact_refs: job_options.artifact_refs,
        metadata: BTreeMap::new(),
    };
    let opened_event = EpiphanyRuntimeEvent {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        event_id: format!("event-job-opened-{}", job.job_id),
        occurred_at: job_options.created_at,
        event_type: "job.opened".to_string(),
        source: "runtime-spine".to_string(),
        session_id: Some(session.session_id.clone()),
        job_id: Some(job.job_id.clone()),
        summary: "Native runtime job opened.".to_string(),
        metadata: BTreeMap::new(),
    };
    if cache
        .get::<EpiphanyRuntimeEvent>(&opened_event.event_id)?
        .is_some()
    {
        return Err(anyhow!(
            "model execution opened event {:?} already exists",
            opened_event.event_id
        ));
    }

    let binding_id = model_request.request_id.clone();
    if cache
        .get::<EpiphanyRuntimeModelExecutionBinding>(&binding_id)?
        .is_some()
        || cache
            .get::<EpiphanyModelRequest>(&model_request.request_id)?
            .is_some()
        || cache
            .get::<EpiphanyOpenAiModelRequest>(&provider_request.request_id)?
            .is_some()
    {
        return Err(anyhow!(
            "model execution request {:?} already exists",
            model_request.request_id
        ));
    }
    let binding = EpiphanyRuntimeModelExecutionBinding {
        schema_version: RUNTIME_MODEL_EXECUTION_BINDING_SCHEMA_VERSION.to_string(),
        binding_id: binding_id.clone(),
        request_id: model_request.request_id.clone(),
        session_id: session.session_id.clone(),
        job_id: job.job_id.clone(),
        provider: model_request.provider.clone(),
        bound_at: bound_at.to_string(),
        source_worker_job_id: model_request.source_worker_job_id.clone(),
        reasoning_basis_id: model_request.reasoning_basis_id.clone(),
    };
    let identity_envelope = cache
        .get_envelope::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("model execution lost its exact runtime identity envelope"))?;
    let mut expected = vec![identity_envelope.clone()];
    expected.extend(source_worker_envelopes.iter().cloned());
    if let Some(envelope) = reasoning_basis_envelope.as_ref() {
        expected.push(envelope.clone());
    }
    if existing_session.is_some() {
        expected.push(
            cache
                .get_envelope::<EpiphanyRuntimeSession>(&session.session_id)?
                .ok_or_else(|| anyhow!("model execution lost its exact session envelope"))?,
        );
    }
    let mut replacements = vec![
        identity_envelope,
        cache.prepare_entry(&session.session_id, &session)?.0,
        cache.prepare_entry(&job.job_id, &job)?.0,
        cache
            .prepare_entry(&opened_event.event_id, &opened_event)?
            .0,
        cache.prepare_entry(&binding_id, &binding)?.0,
        cache
            .prepare_entry(&model_request.request_id, model_request)?
            .0,
        cache
            .prepare_entry(&provider_request.request_id, provider_request)?
            .0,
    ];
    replacements.extend(source_worker_envelopes);
    if let Some(envelope) = reasoning_basis_envelope {
        replacements.push(envelope);
    }
    if !runtime_spine_backing_store(store_path)?.compare_and_swap_batch(&expected, replacements)? {
        return Err(anyhow!(
            "model execution request publication lost its snapshot fence"
        ));
    }
    Ok(binding)
}

pub fn put_runtime_tool_execution_intent(
    store_path: impl AsRef<Path>,
    session_id: &str,
    job_id: &str,
    intent: &EpiphanyToolInvocationIntent,
    bound_at: &str,
) -> Result<EpiphanyRuntimeToolExecutionBinding> {
    validate_non_empty(session_id, "tool execution session id")?;
    validate_non_empty(job_id, "tool execution job id")?;
    validate_non_empty(&intent.intent_id, "tool execution intent id")?;
    validate_non_empty(&intent.adapter, "tool execution adapter")?;
    validate_non_empty(&intent.server, "tool execution server")?;
    validate_non_empty(&intent.tool_name, "tool execution tool name")?;
    chrono::DateTime::parse_from_rfc3339(bound_at)
        .map_err(|error| anyhow!("tool execution binding time is invalid: {error}"))?;

    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    require_runtime_identity_not_archived(&cache, "tool-intent", &intent.intent_id)?;
    let session = cache
        .get::<EpiphanyRuntimeSession>(session_id)?
        .ok_or_else(|| anyhow!("tool execution session {session_id:?} does not exist"))?;
    if matches!(
        session.status,
        EpiphanyRuntimeSessionStatus::Completed | EpiphanyRuntimeSessionStatus::Archived
    ) {
        return Err(anyhow!("tool execution session {session_id:?} is terminal"));
    }
    let job = cache
        .get::<EpiphanyRuntimeJob>(job_id)?
        .ok_or_else(|| anyhow!("tool execution job {job_id:?} does not exist"))?;
    if job.session_id != session_id
        || matches!(
            job.status,
            EpiphanyRuntimeJobStatus::Completed
                | EpiphanyRuntimeJobStatus::Failed
                | EpiphanyRuntimeJobStatus::Cancelled
        )
    {
        return Err(anyhow!(
            "tool execution job {job_id:?} is foreign or terminal"
        ));
    }
    let governed_source_authority = validate_governed_source_tool_intent(&cache, job_id, intent)?;
    if cache
        .get::<EpiphanyRuntimeToolExecutionBinding>(&intent.intent_id)?
        .is_some()
        || cache
            .get::<EpiphanyToolInvocationIntent>(&tool_invocation_intent_key(&intent.intent_id))?
            .is_some()
        || cache
            .get::<EpiphanyToolInvocationReceipt>(&tool_invocation_receipt_key(&intent.intent_id))?
            .is_some()
    {
        return Err(anyhow!(
            "tool execution intent {:?} already exists",
            intent.intent_id
        ));
    }

    let identity_envelope = cache
        .get_envelope::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("tool execution lost runtime identity envelope"))?;
    let session_envelope = cache
        .get_envelope::<EpiphanyRuntimeSession>(session_id)?
        .ok_or_else(|| anyhow!("tool execution lost session envelope"))?;
    let job_envelope = cache
        .get_envelope::<EpiphanyRuntimeJob>(job_id)?
        .ok_or_else(|| anyhow!("tool execution lost job envelope"))?;
    let mut expected = vec![
        identity_envelope.clone(),
        session_envelope.clone(),
        job_envelope.clone(),
    ];
    let mut replacements = expected.clone();
    for envelope in governed_source_authority {
        let already_fenced = expected
            .iter()
            .any(|existing| existing.key == envelope.key && existing.r#type == envelope.r#type);
        if !already_fenced {
            expected.push(envelope.clone());
            replacements.push(envelope);
        }
    }
    if let Some(model_request_id) = intent.model_request_id.as_deref() {
        let model_binding = cache
            .get::<EpiphanyRuntimeModelExecutionBinding>(model_request_id)?
            .ok_or_else(|| {
                anyhow!(
                    "model-derived tool intent {:?} has no execution binding",
                    intent.intent_id
                )
            })?;
        if model_binding.request_id != model_request_id
            || model_binding.session_id != session_id
            || model_binding.job_id != job_id
        {
            return Err(anyhow!(
                "model-derived tool intent {:?} has foreign execution ownership",
                intent.intent_id
            ));
        }
        let persisted_model_request = cache
            .get::<EpiphanyModelRequest>(model_request_id)?
            .ok_or_else(|| anyhow!("model-derived tool intent lost its model request"))?;
        if persisted_model_request.source_worker_job_id != model_binding.source_worker_job_id {
            return Err(anyhow!(
                "model-derived tool intent {:?} has substituted worker provenance",
                intent.intent_id
            ));
        }
        for envelope in [
            cache.get_envelope::<EpiphanyRuntimeModelExecutionBinding>(model_request_id)?,
            cache.get_envelope::<EpiphanyModelRequest>(model_request_id)?,
            cache.get_envelope::<EpiphanyOpenAiModelRequest>(model_request_id)?,
        ] {
            let envelope = envelope.ok_or_else(|| {
                anyhow!(
                    "model-derived tool intent {:?} lost its model execution family",
                    intent.intent_id
                )
            })?;
            expected.push(envelope.clone());
            replacements.push(envelope);
        }
    }
    let binding = EpiphanyRuntimeToolExecutionBinding {
        schema_version: RUNTIME_TOOL_EXECUTION_BINDING_SCHEMA_VERSION.to_string(),
        binding_id: intent.intent_id.clone(),
        intent_id: intent.intent_id.clone(),
        session_id: session_id.to_string(),
        job_id: job_id.to_string(),
        model_request_id: intent.model_request_id.clone(),
        bound_at: bound_at.to_string(),
    };
    replacements.push(cache.prepare_entry(&binding.binding_id, &binding)?.0);
    replacements.push(
        cache
            .prepare_entry(&tool_invocation_intent_key(&intent.intent_id), intent)?
            .0,
    );
    if !runtime_spine_backing_store(store_path)?.compare_and_swap_batch(&expected, replacements)? {
        return Err(anyhow!(
            "tool execution intent publication lost its ownership fence"
        ));
    }
    Ok(binding)
}

pub fn require_runtime_tool_execution_binding(
    store_path: impl AsRef<Path>,
    intent_id: &str,
) -> Result<EpiphanyRuntimeToolExecutionBinding> {
    validate_non_empty(intent_id, "tool execution intent id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let binding = cache
        .get::<EpiphanyRuntimeToolExecutionBinding>(intent_id)?
        .ok_or_else(|| anyhow!("tool execution intent {intent_id:?} is unbound"))?;
    if binding.schema_version != RUNTIME_TOOL_EXECUTION_BINDING_SCHEMA_VERSION
        || binding.binding_id != intent_id
        || binding.intent_id != intent_id
        || chrono::DateTime::parse_from_rfc3339(&binding.bound_at).is_err()
    {
        return Err(anyhow!(
            "tool execution intent {intent_id:?} has an invalid binding"
        ));
    }
    let intent = cache
        .get::<EpiphanyToolInvocationIntent>(&tool_invocation_intent_key(intent_id))?
        .ok_or_else(|| anyhow!("tool execution binding {intent_id:?} lost its intent"))?;
    if intent.intent_id != intent_id || intent.model_request_id != binding.model_request_id {
        return Err(anyhow!(
            "tool execution binding {intent_id:?} disagrees with its intent"
        ));
    }
    let session = cache
        .get::<EpiphanyRuntimeSession>(&binding.session_id)?
        .ok_or_else(|| anyhow!("tool execution binding {intent_id:?} lost its session"))?;
    let job = cache
        .get::<EpiphanyRuntimeJob>(&binding.job_id)?
        .ok_or_else(|| anyhow!("tool execution binding {intent_id:?} lost its job"))?;
    if session.status == EpiphanyRuntimeSessionStatus::Archived
        || job.session_id != binding.session_id
    {
        return Err(anyhow!(
            "tool execution binding {intent_id:?} has foreign or archived ownership"
        ));
    }
    let _ = validate_governed_source_tool_intent(&cache, &binding.job_id, &intent)?;
    if let Some(model_request_id) = binding.model_request_id.as_deref() {
        let model_binding = cache
            .get::<EpiphanyRuntimeModelExecutionBinding>(model_request_id)?
            .ok_or_else(|| anyhow!("tool execution binding lost its model execution"))?;
        if model_binding.session_id != binding.session_id
            || model_binding.job_id != binding.job_id
            || model_binding.request_id != model_request_id
        {
            return Err(anyhow!(
                "tool execution binding {intent_id:?} has foreign model ownership"
            ));
        }
    }
    Ok(binding)
}

pub fn put_runtime_tool_execution_receipt(
    store_path: impl AsRef<Path>,
    receipt: &EpiphanyToolInvocationReceipt,
) -> Result<()> {
    validate_non_empty(&receipt.receipt_id, "tool execution receipt id")?;
    validate_non_empty(&receipt.intent_id, "tool execution receipt intent id")?;
    validate_non_empty(&receipt.status, "tool execution receipt status")?;
    validate_non_empty(
        &receipt.completed_at,
        "tool execution receipt completion time",
    )?;
    let store_path = store_path.as_ref();
    let binding = require_runtime_tool_execution_binding(store_path, &receipt.intent_id)?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let intent = cache
        .get::<EpiphanyToolInvocationIntent>(&tool_invocation_intent_key(&receipt.intent_id))?
        .ok_or_else(|| anyhow!("tool execution receipt lost its intent"))?;
    validate_terminal_tool_execution_family(&binding, &intent, receipt)?;
    let receipt_key = tool_invocation_receipt_key(&receipt.intent_id);
    if cache
        .get::<EpiphanyToolInvocationReceipt>(&receipt_key)?
        .is_some()
    {
        return Err(anyhow!(
            "tool execution intent {:?} already has a receipt",
            receipt.intent_id
        ));
    }
    let binding_envelope = cache
        .get_envelope::<EpiphanyRuntimeToolExecutionBinding>(&binding.binding_id)?
        .ok_or_else(|| anyhow!("tool execution receipt lost its binding envelope"))?;
    let intent_envelope = cache
        .get_envelope::<EpiphanyToolInvocationIntent>(&tool_invocation_intent_key(
            &receipt.intent_id,
        ))?
        .ok_or_else(|| anyhow!("tool execution receipt lost its intent envelope"))?;
    if !runtime_spine_backing_store(store_path)?.compare_and_swap_batch(
        &[binding_envelope.clone(), intent_envelope.clone()],
        vec![
            binding_envelope,
            intent_envelope,
            cache.prepare_entry(&receipt_key, receipt)?.0,
        ],
    )? {
        return Err(anyhow!(
            "tool execution receipt publication lost its ownership fence"
        ));
    }
    Ok(())
}

fn validate_terminal_tool_execution_family(
    binding: &EpiphanyRuntimeToolExecutionBinding,
    intent: &EpiphanyToolInvocationIntent,
    receipt: &EpiphanyToolInvocationReceipt,
) -> Result<()> {
    if binding.schema_version != RUNTIME_TOOL_EXECUTION_BINDING_SCHEMA_VERSION
        || binding.binding_id != intent.intent_id
        || binding.intent_id != intent.intent_id
        || binding.model_request_id != intent.model_request_id
        || intent.schema_id != TOOL_ADAPTER_INVOCATION_INTENT_SCHEMA_ID
        || receipt.schema_id != TOOL_ADAPTER_INVOCATION_RECEIPT_SCHEMA_ID
        || receipt.receipt_id.is_empty()
        || receipt.intent_id != intent.intent_id
        || receipt.adapter != intent.adapter
        || receipt.server != intent.server
        || receipt.tool_name != intent.tool_name
        || !matches!(receipt.status.as_str(), "completed" | "failed")
        || chrono::DateTime::parse_from_rfc3339(&receipt.completed_at).is_err()
    {
        return Err(anyhow!(
            "tool execution binding, intent, and terminal receipt are not one exact family"
        ));
    }
    Ok(())
}

pub fn archive_completed_model_session(
    store_path: impl AsRef<Path>,
    session_id: &str,
    archived_at: &str,
) -> Result<EpiphanyArchivedRuntimeSession> {
    archive_completed_model_session_with_before_commit(store_path, session_id, archived_at, || {
        Ok(())
    })
}

pub fn retain_completed_runtime_sessions(
    store_path: impl AsRef<Path>,
    retain_recent: usize,
    preserve_coordinator_receipt_ids: &BTreeSet<String>,
    archived_at: &str,
) -> Result<Vec<EpiphanyArchivedRuntimeSession>> {
    chrono::DateTime::parse_from_rfc3339(archived_at)
        .map_err(|error| anyhow!("runtime session retention timestamp is invalid: {error}"))?;
    let store_path = store_path.as_ref();
    repair_legacy_terminal_coordinator_sessions(store_path)?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;

    let jobs = cache.get_all::<EpiphanyRuntimeJob>()?;
    let bindings = cache.get_all::<EpiphanyRuntimeModelExecutionBinding>()?;
    let receipts = cache.get_all::<EpiphanyCoordinatorRunReceipt>()?;
    let death_recoveries = cache.get_all::<EpiphanyCoordinatorDeathRecovery>()?;
    let events = cache.get_all::<EpiphanyRuntimeEvent>()?;
    let mut candidates = cache
        .get_all::<EpiphanyRuntimeSession>()?
        .into_iter()
        .filter(|session| session.status == EpiphanyRuntimeSessionStatus::Completed)
        .filter_map(|session| {
            let session_jobs = jobs
                .iter()
                .filter(|job| job.session_id == session.session_id)
                .collect::<Vec<_>>();
            let model_family = !session_jobs.is_empty()
                && session_jobs.iter().all(|job| {
                    job.role == "openai-model-adapter"
                        && bindings.iter().any(|binding| binding.job_id == job.job_id)
                });
            let coordinator_family = session_jobs.is_empty()
                && session.session_id.starts_with("coordinator-")
                && (receipts
                    .iter()
                    .filter(|receipt| receipt.session_id == session.session_id)
                    .count()
                    + death_recoveries
                        .iter()
                        .filter(|recovery| recovery.session_id == session.session_id)
                        .count()
                    == 1)
                && !receipts.iter().any(|receipt| {
                    receipt.session_id == session.session_id
                        && preserve_coordinator_receipt_ids.contains(&receipt.receipt_id)
                })
                && events.iter().any(|event| {
                    event.session_id.as_deref() == Some(session.session_id.as_str())
                        && event.event_type == "session.completed"
                        && event.job_id.is_none()
                });
            (model_family || coordinator_family).then_some((session, coordinator_family))
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .0
            .updated_at
            .cmp(&left.0.updated_at)
            .then_with(|| right.0.session_id.cmp(&left.0.session_id))
    });

    let mut archived = Vec::new();
    for (session, coordinator_family) in candidates.into_iter().skip(retain_recent.max(1)) {
        archived.push(if coordinator_family {
            archive_completed_coordinator_session(store_path, &session.session_id, archived_at)?
        } else {
            archive_completed_model_session(store_path, &session.session_id, archived_at)?
        });
    }
    Ok(archived)
}

fn archive_completed_model_session_with_before_commit<F>(
    store_path: impl AsRef<Path>,
    session_id: &str,
    archived_at: &str,
    before_commit: F,
) -> Result<EpiphanyArchivedRuntimeSession>
where
    F: FnOnce() -> Result<()>,
{
    validate_non_empty(session_id, "archived runtime session id")?;
    chrono::DateTime::parse_from_rfc3339(archived_at)
        .map_err(|error| anyhow!("runtime session archive timestamp is invalid: {error}"))?;
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    if let Some(existing) = cache.get::<EpiphanyArchivedRuntimeSession>(session_id)? {
        if existing.schema_version != ARCHIVED_RUNTIME_SESSION_SCHEMA_VERSION
            || existing.archive_id != session_id
            || existing.session_id != session_id
            || !existing.retired_chain_digest.starts_with("sha256:")
        {
            return Err(anyhow!("archived runtime session tombstone is invalid"));
        }
        if cache.get::<EpiphanyRuntimeSession>(session_id)?.is_some() {
            return Err(anyhow!(
                "archived runtime session still has live session authority"
            ));
        }
        return Ok(existing);
    }
    let session = cache
        .get::<EpiphanyRuntimeSession>(session_id)?
        .ok_or_else(|| anyhow!("runtime session {session_id:?} does not exist"))?;
    if session.status != EpiphanyRuntimeSessionStatus::Completed {
        return Err(anyhow!("runtime session {session_id:?} is not completed"));
    }
    let mut jobs = cache
        .get_all::<EpiphanyRuntimeJob>()?
        .into_iter()
        .filter(|job| job.session_id == session_id)
        .collect::<Vec<_>>();
    jobs.sort_by(|left, right| left.job_id.cmp(&right.job_id));
    if jobs.is_empty()
        || jobs.iter().any(|job| {
            job.role != "openai-model-adapter"
                || matches!(
                    job.status,
                    EpiphanyRuntimeJobStatus::Queued
                        | EpiphanyRuntimeJobStatus::Running
                        | EpiphanyRuntimeJobStatus::WaitingForReview
                )
        })
    {
        return Err(anyhow!(
            "runtime session archive accepts only terminal model-adapter jobs"
        ));
    }
    let job_ids = jobs
        .iter()
        .map(|job| job.job_id.clone())
        .collect::<BTreeSet<_>>();
    let mut job_results = cache
        .get_all::<EpiphanyRuntimeJobResult>()?
        .into_iter()
        .filter(|result| result.session_id == session_id)
        .collect::<Vec<_>>();
    job_results.sort_by(|left, right| left.result_id.cmp(&right.result_id));
    if jobs.iter().any(|job| {
        job_results
            .iter()
            .filter(|result| result.job_id == job.job_id)
            .count()
            != 1
    }) || job_results
        .iter()
        .any(|result| !job_ids.contains(&result.job_id))
    {
        return Err(anyhow!(
            "runtime session archive requires exactly one terminal result per job"
        ));
    }
    if cache
        .get_all::<EpiphanyRuntimeWorkerLaunchRequest>()?
        .iter()
        .any(|item| job_ids.contains(&item.job_id))
        || cache
            .get_all::<EpiphanyRuntimeRoleWorkerResult>()?
            .iter()
            .any(|item| job_ids.contains(&item.job_id))
        || cache
            .get_all::<EpiphanyRuntimeReorientWorkerResult>()?
            .iter()
            .any(|item| job_ids.contains(&item.job_id))
        || cache
            .get_all::<EpiphanyCoordinatorRunReceipt>()?
            .iter()
            .any(|item| item.session_id == session_id)
    {
        return Err(anyhow!(
            "runtime session archive refuses outer-worker or coordinator authority"
        ));
    }

    let mut model_bindings = cache
        .get_all::<EpiphanyRuntimeModelExecutionBinding>()?
        .into_iter()
        .filter(|binding| binding.session_id == session_id)
        .collect::<Vec<_>>();
    model_bindings.sort_by(|left, right| left.request_id.cmp(&right.request_id));
    if jobs.iter().any(|job| {
        model_bindings
            .iter()
            .filter(|binding| binding.job_id == job.job_id)
            .count()
            != 1
    }) || model_bindings
        .iter()
        .any(|binding| !job_ids.contains(&binding.job_id))
    {
        return Err(anyhow!(
            "runtime session archive requires one model execution binding per job"
        ));
    }
    let model_request_ids = model_bindings
        .iter()
        .map(|binding| binding.request_id.clone())
        .collect::<BTreeSet<_>>();
    let reasoning_basis_ids = model_bindings
        .iter()
        .filter_map(|binding| binding.reasoning_basis_id.clone())
        .collect::<BTreeSet<_>>();
    for basis_id in &reasoning_basis_ids {
        cache
            .get::<crate::EpiphanyReasoningBasis>(basis_id)?
            .ok_or_else(|| anyhow!("runtime session archive lost a reasoning basis"))?;
    }
    let mut decision_context_ids = BTreeSet::new();
    for context in cache.get_all::<crate::EpiphanyDecisionContext>()? {
        let native = context.native_request()?;
        if model_request_ids.contains(&native.request_id) {
            if !reasoning_basis_ids.contains(&context.basis_id) {
                return Err(anyhow!(
                    "runtime session archive found a context outside its basis family"
                ));
            }
            decision_context_ids.insert(context.context_id);
        }
    }
    if model_bindings
        .iter()
        .any(|binding| binding.source_worker_job_id.is_some())
        && decision_context_ids.is_empty()
    {
        return Err(anyhow!(
            "decision-bearing model session archive requires its terminal context"
        ));
    }
    for request_id in &model_request_ids {
        let native_request = cache
            .get::<EpiphanyModelRequest>(request_id)?
            .ok_or_else(|| anyhow!("archived model execution lost native request"))?;
        let provider_request = cache
            .get::<EpiphanyOpenAiModelRequest>(request_id)?
            .ok_or_else(|| anyhow!("archived model execution lost provider request"))?;
        if native_request.request_id != *request_id
            || provider_request.request_id != *request_id
            || native_request.conversation_id != provider_request.conversation_id
            || native_request.model != provider_request.model
        {
            return Err(anyhow!(
                "archived model execution request family is inconsistent"
            ));
        }
        let mut native_events = cache
            .get_all::<EpiphanyModelStreamEvent>()?
            .into_iter()
            .filter(|event| event.request_id == *request_id)
            .collect::<Vec<_>>();
        native_events.sort_by_key(|event| event.sequence);
        let native_terminals = native_events
            .iter()
            .filter(|event| {
                matches!(
                    &event.payload,
                    EpiphanyModelStreamPayload::Completed { .. }
                        | EpiphanyModelStreamPayload::Failed { .. }
                )
            })
            .collect::<Vec<_>>();
        let mut provider_events = cache
            .get_all::<EpiphanyOpenAiStreamEvent>()?
            .into_iter()
            .filter(|event| event.request_id == *request_id)
            .collect::<Vec<_>>();
        provider_events.sort_by_key(|event| event.sequence);
        let provider_terminals = provider_events
            .iter()
            .filter(|event| {
                matches!(
                    &event.payload,
                    EpiphanyOpenAiStreamPayload::Completed { .. }
                        | EpiphanyOpenAiStreamPayload::Failed { .. }
                )
            })
            .collect::<Vec<_>>();
        if native_terminals.len() != 1
            || provider_terminals.len() != 1
            || native_events.last().map(|event| event.sequence)
                != native_terminals.first().map(|event| event.sequence)
            || provider_events.last().map(|event| event.sequence)
                != provider_terminals.first().map(|event| event.sequence)
        {
            return Err(anyhow!(
                "runtime session archive requires one native and provider terminal stream event"
            ));
        }
        match &native_terminals[0].payload {
            EpiphanyModelStreamPayload::Completed { receipt } => {
                if cache.get::<EpiphanyModelReceipt>(request_id)?.as_ref() != Some(receipt) {
                    return Err(anyhow!(
                        "runtime session archive found inconsistent native model receipt"
                    ));
                }
            }
            EpiphanyModelStreamPayload::Failed { .. } => {
                if cache.get::<EpiphanyModelReceipt>(request_id)?.is_some() {
                    return Err(anyhow!(
                        "failed native model stream retained a success receipt"
                    ));
                }
            }
            _ => unreachable!("terminal event filtered above"),
        }
        match &provider_terminals[0].payload {
            EpiphanyOpenAiStreamPayload::Completed { receipt } => {
                if cache
                    .get::<EpiphanyOpenAiModelReceipt>(request_id)?
                    .as_ref()
                    != Some(receipt)
                {
                    return Err(anyhow!(
                        "runtime session archive found inconsistent provider model receipt"
                    ));
                }
            }
            EpiphanyOpenAiStreamPayload::Failed { .. } => {
                if cache
                    .get::<EpiphanyOpenAiModelReceipt>(request_id)?
                    .is_some()
                {
                    return Err(anyhow!(
                        "failed provider model stream retained a success receipt"
                    ));
                }
            }
            _ => unreachable!("terminal event filtered above"),
        }
    }

    let mut tool_bindings = cache
        .get_all::<EpiphanyRuntimeToolExecutionBinding>()?
        .into_iter()
        .filter(|binding| binding.session_id == session_id)
        .collect::<Vec<_>>();
    tool_bindings.sort_by(|left, right| left.intent_id.cmp(&right.intent_id));
    if tool_bindings.iter().any(|binding| {
        !job_ids.contains(&binding.job_id)
            || binding
                .model_request_id
                .as_ref()
                .is_some_and(|request_id| !model_request_ids.contains(request_id))
    }) {
        return Err(anyhow!(
            "runtime session archive found foreign tool execution ownership"
        ));
    }
    let tool_intent_ids = tool_bindings
        .iter()
        .map(|binding| binding.intent_id.clone())
        .collect::<BTreeSet<_>>();
    let unbound_model_intent = cache
        .get_all::<EpiphanyToolInvocationIntent>()?
        .into_iter()
        .any(|intent| {
            intent
                .model_request_id
                .as_ref()
                .is_some_and(|request_id| model_request_ids.contains(request_id))
                && !tool_intent_ids.contains(&intent.intent_id)
        });
    if unbound_model_intent {
        return Err(anyhow!(
            "runtime session archive found a legacy unbound model tool intent"
        ));
    }
    for binding in &tool_bindings {
        require_runtime_tool_execution_binding(store_path, &binding.intent_id)?;
        let receipt = cache
            .get::<EpiphanyToolInvocationReceipt>(&tool_invocation_receipt_key(&binding.intent_id))?
            .ok_or_else(|| anyhow!("runtime session archive found pending tool authority"))?;
        if receipt.intent_id != binding.intent_id
            || !matches!(receipt.status.as_str(), "completed" | "failed")
        {
            return Err(anyhow!(
                "runtime session archive found invalid tool terminal evidence"
            ));
        }
    }

    let events = cache
        .get_all::<EpiphanyRuntimeEvent>()?
        .into_iter()
        .filter(|event| event.session_id.as_deref() == Some(session_id))
        .collect::<Vec<_>>();
    if events
        .iter()
        .filter(|event| event.event_type == "session.completed" && event.job_id.is_none())
        .count()
        != 1
    {
        return Err(anyhow!(
            "runtime session archive requires one session completion event"
        ));
    }

    let snapshot = cache.snapshot_envelopes();
    let mut retired_identities = BTreeSet::<(String, String)>::new();
    retired_identities.insert((
        EpiphanyRuntimeSession::TYPE.to_string(),
        session_id.to_string(),
    ));
    for job in &jobs {
        retired_identities.insert((EpiphanyRuntimeJob::TYPE.to_string(), job.job_id.clone()));
    }
    for result in &job_results {
        retired_identities.insert((
            EpiphanyRuntimeJobResult::TYPE.to_string(),
            result.result_id.clone(),
        ));
    }
    for event in &events {
        retired_identities.insert((
            EpiphanyRuntimeEvent::TYPE.to_string(),
            event.event_id.clone(),
        ));
    }
    for binding in &model_bindings {
        retired_identities.insert((
            EpiphanyRuntimeModelExecutionBinding::TYPE.to_string(),
            binding.binding_id.clone(),
        ));
        retired_identities.insert((
            EpiphanyModelRequest::TYPE.to_string(),
            binding.request_id.clone(),
        ));
        retired_identities.insert((
            EpiphanyOpenAiModelRequest::TYPE.to_string(),
            binding.request_id.clone(),
        ));
        for event in cache
            .get_all::<EpiphanyModelStreamEvent>()?
            .into_iter()
            .filter(|event| event.request_id == binding.request_id)
        {
            retired_identities.insert((
                EpiphanyModelStreamEvent::TYPE.to_string(),
                format!("{}:{:08}", event.request_id, event.sequence),
            ));
        }
        for event in cache
            .get_all::<EpiphanyOpenAiStreamEvent>()?
            .into_iter()
            .filter(|event| event.request_id == binding.request_id)
        {
            retired_identities.insert((
                EpiphanyOpenAiStreamEvent::TYPE.to_string(),
                format!("{}:{:08}", event.request_id, event.sequence),
            ));
        }
        if cache
            .get::<EpiphanyModelReceipt>(&binding.request_id)?
            .is_some()
        {
            retired_identities.insert((
                EpiphanyModelReceipt::TYPE.to_string(),
                binding.request_id.clone(),
            ));
        }
        if cache
            .get::<EpiphanyOpenAiModelReceipt>(&binding.request_id)?
            .is_some()
        {
            retired_identities.insert((
                EpiphanyOpenAiModelReceipt::TYPE.to_string(),
                binding.request_id.clone(),
            ));
        }
    }
    for binding in &tool_bindings {
        retired_identities.insert((
            EpiphanyRuntimeToolExecutionBinding::TYPE.to_string(),
            binding.binding_id.clone(),
        ));
        retired_identities.insert((
            EpiphanyToolInvocationIntent::TYPE.to_string(),
            tool_invocation_intent_key(&binding.intent_id),
        ));
        retired_identities.insert((
            EpiphanyToolInvocationReceipt::TYPE.to_string(),
            tool_invocation_receipt_key(&binding.intent_id),
        ));
    }
    let mut deletions = retired_identities
        .iter()
        .map(|(document_type, key)| {
            snapshot
                .iter()
                .find(|entry| entry.r#type == *document_type && entry.key == *key)
                .cloned()
                .ok_or_else(|| {
                    anyhow!("runtime session archive lost exact envelope {document_type:?}/{key:?}")
                })
        })
        .collect::<Result<Vec<_>>>()?;
    deletions.sort_by(|left, right| {
        left.r#type
            .cmp(&right.r#type)
            .then(left.key.cmp(&right.key))
    });
    let mut retired_type_counts = BTreeMap::new();
    let mut digest = Sha256::new();
    digest.update(b"epiphany-runtime-archived-session-root");
    for entry in &deletions {
        *retired_type_counts.entry(entry.r#type.clone()).or_default() += 1;
        for bytes in [
            entry.r#type.as_bytes(),
            entry.key.as_bytes(),
            entry.payload.as_slice(),
        ] {
            digest.update((bytes.len() as u64).to_le_bytes());
            digest.update(bytes);
        }
    }
    let mut terminal_job_status_counts = BTreeMap::new();
    for job in &jobs {
        let status = match job.status {
            EpiphanyRuntimeJobStatus::Completed => "completed",
            EpiphanyRuntimeJobStatus::Failed => "failed",
            EpiphanyRuntimeJobStatus::Cancelled => "cancelled",
            _ => unreachable!("open jobs refused above"),
        };
        *terminal_job_status_counts
            .entry(status.to_string())
            .or_default() += 1;
    }
    let archive = EpiphanyArchivedRuntimeSession {
        schema_version: ARCHIVED_RUNTIME_SESSION_SCHEMA_VERSION.to_string(),
        archive_id: session_id.to_string(),
        session_id: session_id.to_string(),
        archived_at: archived_at.to_string(),
        job_ids: job_ids.into_iter().collect(),
        job_result_ids: job_results
            .iter()
            .map(|result| result.result_id.clone())
            .collect(),
        model_request_ids: model_request_ids.into_iter().collect(),
        tool_intent_ids: tool_intent_ids.into_iter().collect(),
        terminal_job_status_counts,
        retired_type_counts,
        retired_envelope_count: deletions.len() as u64,
        retired_chain_digest: format!("sha256:{:x}", digest.finalize()),
        reasoning_basis_ids: reasoning_basis_ids.into_iter().collect(),
        decision_context_ids: decision_context_ids.into_iter().collect(),
    };
    let (replacement, _) = cache.prepare_entry(session_id, &archive)?;
    before_commit()?;
    if !runtime_spine_backing_store(store_path)?.replace_and_delete_if_snapshot_unchanged(
        &snapshot,
        vec![replacement],
        &deletions,
    )? {
        return Err(anyhow!(
            "runtime session archive lost its full snapshot fence"
        ));
    }
    Ok(archive)
}

pub fn repair_legacy_terminal_coordinator_sessions(
    store_path: impl AsRef<Path>,
) -> Result<Vec<EpiphanyRuntimeSession>> {
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let jobs = cache.get_all::<EpiphanyRuntimeJob>()?;
    let receipts = cache.get_all::<EpiphanyCoordinatorRunReceipt>()?;
    let events = cache.get_all::<EpiphanyRuntimeEvent>()?;
    let mut candidates = Vec::new();
    let mut sessions = cache
        .get_all::<EpiphanyRuntimeSession>()?
        .into_iter()
        .filter(|session| {
            session.status == EpiphanyRuntimeSessionStatus::Active
                && session.session_id.starts_with("coordinator-")
        })
        .collect::<Vec<_>>();
    sessions.sort_by(|left, right| left.session_id.cmp(&right.session_id));
    for session in sessions {
        let session_receipts = receipts
            .iter()
            .filter(|receipt| receipt.session_id == session.session_id)
            .collect::<Vec<_>>();
        if session_receipts.is_empty() {
            continue;
        }
        if session_receipts.len() != 1 {
            return Err(anyhow!(
                "legacy coordinator session {:?} has ambiguous terminal receipts",
                session.session_id
            ));
        }
        let receipt = session_receipts[0];
        if session
            .session_id
            .strip_prefix("coordinator-")
            .is_none_or(|thread_id| thread_id != receipt.thread_id)
            || jobs.iter().any(|job| job.session_id == session.session_id)
        {
            return Err(anyhow!(
                "legacy coordinator session terminal authority is inconsistent"
            ));
        }
        let session_events = events
            .iter()
            .filter(|event| event.session_id.as_deref() == Some(&session.session_id))
            .collect::<Vec<_>>();
        if session_events.is_empty()
            || session_events.iter().any(|event| {
                event.event_type != "coordinator.started"
                    || event.source != "epiphany-mvp-coordinator"
                    || event.job_id.is_some()
            })
        {
            return Err(anyhow!(
                "legacy coordinator session start evidence is inconsistent"
            ));
        }
        candidates.push((session, receipt.clone()));
    }
    if candidates.is_empty() {
        return Ok(Vec::new());
    }
    let snapshot = cache.snapshot_envelopes();
    let mut replacements = Vec::with_capacity(candidates.len() * 2);
    let mut repaired = Vec::with_capacity(candidates.len());
    for (mut session, receipt) in candidates {
        let event = coordinator_completion_event(&receipt);
        session.status = EpiphanyRuntimeSessionStatus::Completed;
        session.updated_at = receipt.created_at.clone();
        session.coordinator_note = event.summary.clone();
        replacements.push(cache.prepare_entry(&session.session_id, &session)?.0);
        replacements.push(cache.prepare_entry(&event.event_id, &event)?.0);
        repaired.push(session);
    }
    if !runtime_spine_backing_store(store_path)?
        .replace_and_append_if_snapshot_unchanged(&snapshot, replacements)?
    {
        return Err(anyhow!(
            "legacy coordinator session repair lost its full snapshot fence"
        ));
    }
    Ok(repaired)
}

pub fn archive_completed_coordinator_session(
    store_path: impl AsRef<Path>,
    session_id: &str,
    archived_at: &str,
) -> Result<EpiphanyArchivedRuntimeSession> {
    archive_completed_coordinator_session_with_before_commit(
        store_path,
        session_id,
        archived_at,
        || Ok(()),
    )
}

fn archive_completed_coordinator_session_with_before_commit<F>(
    store_path: impl AsRef<Path>,
    session_id: &str,
    archived_at: &str,
    before_commit: F,
) -> Result<EpiphanyArchivedRuntimeSession>
where
    F: FnOnce() -> Result<()>,
{
    validate_non_empty(session_id, "archived coordinator session id")?;
    chrono::DateTime::parse_from_rfc3339(archived_at)
        .map_err(|error| anyhow!("coordinator session archive timestamp is invalid: {error}"))?;
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    if let Some(existing) = cache.get::<EpiphanyArchivedRuntimeSession>(session_id)? {
        if existing.schema_version != ARCHIVED_RUNTIME_SESSION_SCHEMA_VERSION
            || existing.archive_id != session_id
            || existing.session_id != session_id
            || !existing.retired_chain_digest.starts_with("sha256:")
            || !existing.job_ids.is_empty()
            || !existing.model_request_ids.is_empty()
            || !existing.tool_intent_ids.is_empty()
        {
            return Err(anyhow!("archived coordinator session tombstone is invalid"));
        }
        if cache.get::<EpiphanyRuntimeSession>(session_id)?.is_some() {
            return Err(anyhow!(
                "archived coordinator session still has live session authority"
            ));
        }
        if cache
            .get_all::<EpiphanyCoordinatorRunReceipt>()?
            .iter()
            .any(|receipt| receipt.session_id == session_id)
            || cache
                .get_all::<EpiphanyCoordinatorDeathRecovery>()?
                .iter()
                .any(|recovery| recovery.session_id == session_id)
            || cache
                .get_all::<EpiphanyRuntimeEvent>()?
                .iter()
                .any(|event| event.session_id.as_deref() == Some(session_id))
        {
            return Err(anyhow!(
                "archived coordinator session retained executable family evidence"
            ));
        }
        return Ok(existing);
    }
    let session = cache
        .get::<EpiphanyRuntimeSession>(session_id)?
        .ok_or_else(|| anyhow!("coordinator session {session_id:?} does not exist"))?;
    if session.status != EpiphanyRuntimeSessionStatus::Completed
        || !session_id.starts_with("coordinator-")
        || cache
            .get_all::<EpiphanyRuntimeJob>()?
            .iter()
            .any(|job| job.session_id == session_id)
    {
        return Err(anyhow!(
            "coordinator session archive requires a completed jobless coordinator session"
        ));
    }
    let receipts = cache
        .get_all::<EpiphanyCoordinatorRunReceipt>()?
        .into_iter()
        .filter(|receipt| receipt.session_id == session_id)
        .collect::<Vec<_>>();
    let recoveries = cache
        .get_all::<EpiphanyCoordinatorDeathRecovery>()?
        .into_iter()
        .filter(|recovery| recovery.session_id == session_id)
        .collect::<Vec<_>>();
    if receipts.len() + recoveries.len() != 1 {
        return Err(anyhow!(
            "coordinator session archive requires one exact terminal authority"
        ));
    }
    let expected_session_id = if let Some(receipt) = receipts.first() {
        coordinator_run_session_id(
            &receipt.thread_id,
            receipt.resident_launch_digest.as_deref(),
        )?
    } else {
        coordinator_run_session_id(
            &recoveries[0].thread_id,
            Some(&recoveries[0].resident_launch_digest),
        )?
    };
    if session_id != expected_session_id {
        return Err(anyhow!(
            "coordinator session archive found a substituted thread binding"
        ));
    }
    let mut events = cache
        .get_all::<EpiphanyRuntimeEvent>()?
        .into_iter()
        .filter(|event| event.session_id.as_deref() == Some(session_id))
        .collect::<Vec<_>>();
    events.sort_by(|left, right| left.event_id.cmp(&right.event_id));
    let expected_completion = if let Some(receipt) = receipts.first() {
        coordinator_completion_event(receipt)
    } else {
        coordinator_death_recovery_event(&recoveries[0])
    };
    if events
        .iter()
        .filter(|event| **event == expected_completion)
        .count()
        != 1
        || !events.iter().any(|event| {
            event.event_type == "coordinator.started"
                && event.source == "epiphany-mvp-coordinator"
                && event.job_id.is_none()
        })
        || events.iter().any(|event| {
            event != &expected_completion
                && (event.event_type != "coordinator.started"
                    || event.source != "epiphany-mvp-coordinator"
                    || event.job_id.is_some())
        })
        || session.updated_at
            != match receipts.first() {
                Some(receipt) => receipt.created_at.as_str(),
                None => recoveries[0].recovered_at.as_str(),
            }
        || session.coordinator_note != expected_completion.summary
    {
        return Err(anyhow!(
            "coordinator session archive found inconsistent terminal evidence"
        ));
    }
    let snapshot = cache.snapshot_envelopes();
    let mut deletions = Vec::new();
    let terminal_envelope = if let Some(receipt) = receipts.first() {
        (
            EpiphanyCoordinatorRunReceipt::TYPE,
            receipt.receipt_id.clone(),
        )
    } else {
        (
            EpiphanyCoordinatorDeathRecovery::TYPE,
            recoveries[0].recovery_id.clone(),
        )
    };
    for (document_type, key) in
        std::iter::once((EpiphanyRuntimeSession::TYPE, session_id.to_string()))
            .chain(std::iter::once(terminal_envelope))
            .chain(
                events
                    .iter()
                    .map(|event| (EpiphanyRuntimeEvent::TYPE, event.event_id.clone())),
            )
    {
        deletions.push(
            snapshot
                .iter()
                .find(|entry| entry.r#type == document_type && entry.key == key)
                .cloned()
                .ok_or_else(|| anyhow!("coordinator session archive lost an exact envelope"))?,
        );
    }
    deletions.sort_by(|left, right| {
        left.r#type
            .cmp(&right.r#type)
            .then(left.key.cmp(&right.key))
    });
    let mut retired_type_counts = BTreeMap::new();
    let mut digest = Sha256::new();
    digest.update(b"epiphany-runtime-archived-session-root");
    for entry in &deletions {
        *retired_type_counts.entry(entry.r#type.clone()).or_default() += 1;
        for bytes in [
            entry.r#type.as_bytes(),
            entry.key.as_bytes(),
            entry.payload.as_slice(),
        ] {
            digest.update((bytes.len() as u64).to_le_bytes());
            digest.update(bytes);
        }
    }
    let archive = EpiphanyArchivedRuntimeSession {
        schema_version: ARCHIVED_RUNTIME_SESSION_SCHEMA_VERSION.to_string(),
        archive_id: session_id.to_string(),
        session_id: session_id.to_string(),
        archived_at: archived_at.to_string(),
        job_ids: Vec::new(),
        job_result_ids: Vec::new(),
        model_request_ids: Vec::new(),
        tool_intent_ids: Vec::new(),
        terminal_job_status_counts: BTreeMap::new(),
        retired_type_counts,
        retired_envelope_count: deletions.len() as u64,
        retired_chain_digest: format!("sha256:{:x}", digest.finalize()),
        reasoning_basis_ids: Vec::new(),
        decision_context_ids: Vec::new(),
    };
    let replacement = cache.prepare_entry(session_id, &archive)?.0;
    before_commit()?;
    if !runtime_spine_backing_store(store_path)?.replace_and_delete_if_snapshot_unchanged(
        &snapshot,
        vec![replacement],
        &deletions,
    )? {
        return Err(anyhow!(
            "coordinator session archive lost its full snapshot fence"
        ));
    }
    Ok(archive)
}

pub fn plan_runtime_spine_heartbeat_launch(
    state: &EpiphanyThreadState,
    options: RuntimeSpineHeartbeatLaunchPlanOptions,
) -> Result<RuntimeSpineHeartbeatLaunchPlan> {
    validate_heartbeat_launch_options(state, &options)?;
    Ok(RuntimeSpineHeartbeatLaunchPlan {
        binding: EpiphanyJobBinding {
            id: options.binding_id.clone(),
            kind: options.kind,
            scope: options.scope.clone(),
            owner_role: options.owner_role.clone(),
            authority_scope: Some(options.authority_scope.clone()),
            linked_subgoal_ids: options.linked_subgoal_ids.clone(),
            linked_graph_node_ids: options.linked_graph_node_ids.clone(),
            blocking_reason: None,
        },
        runtime_link: EpiphanyRuntimeLink {
            id: format!(
                "runtime-link-{}-{}",
                options.binding_id, options.runtime_job_id
            ),
            binding_id: options.binding_id,
            surface: "jobLaunch".to_string(),
            role_id: options.owner_role,
            authority_scope: options.authority_scope,
            runtime_job_id: options.runtime_job_id,
            runtime_result_id: None,
            linked_subgoal_ids: options.linked_subgoal_ids,
            linked_graph_node_ids: options.linked_graph_node_ids,
        },
    })
}

pub fn replace_or_append_epiphany_job_binding(
    mut bindings: Vec<EpiphanyJobBinding>,
    replacement: EpiphanyJobBinding,
) -> Vec<EpiphanyJobBinding> {
    if let Some(existing) = bindings
        .iter_mut()
        .find(|binding| binding.id == replacement.id)
    {
        *existing = replacement;
        return bindings;
    }
    bindings.push(replacement);
    bindings
}

pub fn clear_epiphany_job_binding_backend(
    mut bindings: Vec<EpiphanyJobBinding>,
    binding_index: usize,
    blocking_reason: &str,
) -> Vec<EpiphanyJobBinding> {
    let binding = &mut bindings[binding_index];
    binding.blocking_reason = Some(blocking_reason.to_string());
    bindings
}

pub fn open_runtime_spine_heartbeat_job(
    store_path: impl AsRef<Path>,
    options: RuntimeSpineHeartbeatJobOptions,
) -> Result<EpiphanyRuntimeJob> {
    validate_non_empty(&options.runtime_id, "runtime id")?;
    validate_non_empty(&options.display_name, "display name")?;
    validate_non_empty(&options.session_id, "session id")?;
    validate_non_empty(&options.objective, "objective")?;
    validate_non_empty(&options.job_id, "job id")?;
    validate_non_empty(&options.role, "role")?;
    validate_repository_body_launch_carrier(&options.role, &options.launch_document)?;
    validate_proposal_modeling_launch_carrier(
        &options.role,
        &options.binding_id,
        options.proposal_modeling_request_id.as_deref(),
        &options.launch_document,
    )?;
    validate_claim_repair_launch_carrier(
        &options.role,
        &options.binding_id,
        options.claim_repair_request_id.as_deref(),
        &options.launch_document,
    )?;
    validate_frontier_planning_launch_carrier(
        &options.role,
        &options.binding_id,
        options.frontier_planning_request_id.as_deref(),
        &options.launch_document,
    )?;
    validate_frontier_research_launch_carrier(
        &options.role,
        &options.binding_id,
        options.repo_frontier_research_request_id.as_deref(),
        &options.launch_document,
    )?;
    validate_imagination_consideration_launch_carrier(
        &options.role,
        &options.binding_id,
        options.imagination_consideration_request_id.as_deref(),
        &options.launch_document,
    )?;
    validate_repo_frontier_verdict_modeling_launch_authority(
        &options.role,
        options.repo_frontier_modeling_request_id.as_deref(),
        options.repo_frontier_verdict_modeling_authority.as_ref(),
    )?;
    validate_non_empty(&options.binding_id, "binding id")?;
    validate_non_empty(&options.authority_scope, "authority scope")?;
    validate_non_empty(&options.instruction, "instruction")?;
    validate_non_empty(
        options.launch_document.thread_id(),
        "worker launch document thread id",
    )?;
    validate_non_empty(&options.output_contract_id, "output contract id")?;
    if options.output_contract_id != options.launch_document.output_contract_id() {
        return Err(anyhow!(
            "worker launch output_contract_id must match the typed launch document"
        ));
    }
    validate_launch_organ_contract(
        &options.organ_launch_contract,
        &options.authority_scope,
        options.launch_document.document_kind(),
        &options.output_contract_id,
    )?;
    validate_non_empty(&options.created_at, "created at")?;
    let store_path = store_path.as_ref();
    let job_id = options.job_id.clone();
    let binding_id = options.binding_id.clone();
    let role = options.role.clone();
    let authority_scope = options.authority_scope.clone();
    let instruction = options.instruction.clone();
    let output_contract_id = options.output_contract_id.clone();
    let organ_launch_contract = options.organ_launch_contract.clone();
    let launch_document = options.launch_document.clone();
    initialize_runtime_spine(
        store_path,
        RuntimeSpineInitOptions {
            runtime_id: options.runtime_id,
            display_name: options.display_name,
            created_at: options.created_at.clone(),
        },
    )?;
    ensure_runtime_session(
        store_path,
        RuntimeSpineSessionOptions {
            session_id: options.session_id.clone(),
            objective: options.objective,
            created_at: options.created_at.clone(),
            coordinator_note: options.coordinator_note,
        },
    )?;
    let job = create_runtime_job(
        store_path,
        RuntimeSpineJobOptions {
            job_id: options.job_id,
            session_id: options.session_id,
            role: options.role,
            created_at: options.created_at,
            summary: format!(
                "Heartbeat activation queued for binding {} with authority {}.",
                options.binding_id, options.authority_scope
            ),
            artifact_refs: Vec::new(),
        },
    )?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    if cache
        .get::<EpiphanyRuntimeWorkerLaunchRequest>(&job_id)?
        .is_some()
    {
        return Err(anyhow!(
            "runtime worker launch request {:?} already exists",
            job_id
        ));
    }
    let request = EpiphanyRuntimeWorkerLaunchRequest {
        schema_version: RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION.to_string(),
        job_id: job_id.clone(),
        binding_id,
        role,
        authority_scope,
        instruction,
        output_contract_id,
        document_kind: worker_launch_document_kind(&launch_document).to_string(),
        launch_document_msgpack: encode_worker_launch_document(&launch_document)?,
        metadata: BTreeMap::new(),
        organ_launch_contract,
        proposal_modeling_request_id: options.proposal_modeling_request_id,
        claim_repair_request_id: options.claim_repair_request_id,
        frontier_planning_request_id: options.frontier_planning_request_id,
        frontier_plan_mind_request_id: options.frontier_plan_mind_request_id,
        imagination_consideration_request_id: options.imagination_consideration_request_id,
        admitted_model_direction_consideration_request_id: options
            .admitted_model_direction_consideration_request_id,
        repo_frontier_modeling_request_id: options.repo_frontier_modeling_request_id,
        repo_frontier_research_request_id: options.repo_frontier_research_request_id,
        repo_frontier_verdict_modeling_authority_msgpack: options
            .repo_frontier_verdict_modeling_authority
            .as_ref()
            .map(rmp_serde::to_vec_named)
            .transpose()?,
    };
    cache.put(&job_id, &request)?;
    Ok(job)
}

pub fn prepare_runtime_spine_heartbeat_job(
    cache: &CultCache,
    options: RuntimeSpineHeartbeatJobOptions,
) -> Result<PreparedRuntimeSpineHeartbeatJob> {
    validate_non_empty(&options.runtime_id, "runtime id")?;
    validate_non_empty(&options.display_name, "display name")?;
    validate_non_empty(&options.session_id, "session id")?;
    validate_non_empty(&options.objective, "objective")?;
    validate_non_empty(&options.job_id, "job id")?;
    validate_non_empty(&options.role, "role")?;
    validate_repository_body_launch_carrier(&options.role, &options.launch_document)?;
    validate_proposal_modeling_launch_carrier(
        &options.role,
        &options.binding_id,
        options.proposal_modeling_request_id.as_deref(),
        &options.launch_document,
    )?;
    validate_claim_repair_launch_carrier(
        &options.role,
        &options.binding_id,
        options.claim_repair_request_id.as_deref(),
        &options.launch_document,
    )?;
    validate_frontier_planning_launch_carrier(
        &options.role,
        &options.binding_id,
        options.frontier_planning_request_id.as_deref(),
        &options.launch_document,
    )?;
    validate_frontier_research_launch_carrier(
        &options.role,
        &options.binding_id,
        options.repo_frontier_research_request_id.as_deref(),
        &options.launch_document,
    )?;
    validate_imagination_consideration_launch_carrier(
        &options.role,
        &options.binding_id,
        options.imagination_consideration_request_id.as_deref(),
        &options.launch_document,
    )?;
    validate_repo_frontier_verdict_modeling_launch_authority(
        &options.role,
        options.repo_frontier_modeling_request_id.as_deref(),
        options.repo_frontier_verdict_modeling_authority.as_ref(),
    )?;
    validate_non_empty(&options.binding_id, "binding id")?;
    validate_non_empty(&options.authority_scope, "authority scope")?;
    validate_non_empty(&options.instruction, "instruction")?;
    validate_non_empty(
        options.launch_document.thread_id(),
        "worker launch document thread id",
    )?;
    validate_non_empty(&options.output_contract_id, "output contract id")?;
    if options.output_contract_id != options.launch_document.output_contract_id() {
        return Err(anyhow!(
            "worker launch output_contract_id must match the typed launch document"
        ));
    }
    validate_launch_organ_contract(
        &options.organ_launch_contract,
        &options.authority_scope,
        options.launch_document.document_kind(),
        &options.output_contract_id,
    )?;
    validate_non_empty(&options.created_at, "created at")?;

    let existing_identity = cache.get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?;
    let identity = EpiphanyRuntimeIdentity {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        runtime_id: options.runtime_id,
        display_name: options.display_name,
        runtime_kind: "epiphany.native".to_string(),
        created_at: existing_identity
            .as_ref()
            .map(|value| value.created_at.clone())
            .unwrap_or_else(|| options.created_at.clone()),
        updated_at: options.created_at.clone(),
        supported_document_types: runtime_registered_document_types(),
        metadata: BTreeMap::from([("codexEvacuationBridge".to_string(), "temporary".to_string())]),
    };
    let session = match cache.get::<EpiphanyRuntimeSession>(&options.session_id)? {
        Some(existing)
            if matches!(
                existing.status,
                EpiphanyRuntimeSessionStatus::Completed | EpiphanyRuntimeSessionStatus::Archived
            ) =>
        {
            return Err(anyhow!(
                "runtime session {:?} is terminal and cannot accept jobs",
                options.session_id
            ));
        }
        Some(existing) => existing,
        None => EpiphanyRuntimeSession {
            schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
            session_id: options.session_id.clone(),
            objective: options.objective,
            status: EpiphanyRuntimeSessionStatus::Active,
            created_at: options.created_at.clone(),
            updated_at: options.created_at.clone(),
            coordinator_note: options.coordinator_note,
            metadata: BTreeMap::new(),
        },
    };
    if cache.get::<EpiphanyRuntimeJob>(&options.job_id)?.is_some() {
        return Err(anyhow!("runtime job {:?} already exists", options.job_id));
    }
    if cache
        .get::<EpiphanyRuntimeWorkerLaunchRequest>(&options.job_id)?
        .is_some()
    {
        return Err(anyhow!(
            "runtime worker launch request {:?} already exists",
            options.job_id
        ));
    }
    let job = EpiphanyRuntimeJob {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        job_id: options.job_id.clone(),
        session_id: options.session_id.clone(),
        role: options.role.clone(),
        status: EpiphanyRuntimeJobStatus::Queued,
        created_at: options.created_at.clone(),
        updated_at: options.created_at.clone(),
        summary: format!(
            "Heartbeat activation queued for binding {} with authority {}.",
            options.binding_id, options.authority_scope
        ),
        artifact_refs: Vec::new(),
        metadata: BTreeMap::new(),
    };
    let event = EpiphanyRuntimeEvent {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        event_id: format!("event-job-opened-{}", options.job_id),
        occurred_at: options.created_at,
        event_type: "job.opened".to_string(),
        source: "runtime-spine".to_string(),
        session_id: Some(options.session_id),
        job_id: Some(options.job_id.clone()),
        summary: "Native runtime job opened.".to_string(),
        metadata: BTreeMap::new(),
    };
    let request = EpiphanyRuntimeWorkerLaunchRequest {
        schema_version: RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION.to_string(),
        job_id: options.job_id.clone(),
        binding_id: options.binding_id,
        role: options.role,
        authority_scope: options.authority_scope,
        instruction: options.instruction,
        output_contract_id: options.output_contract_id,
        document_kind: worker_launch_document_kind(&options.launch_document).to_string(),
        launch_document_msgpack: encode_worker_launch_document(&options.launch_document)?,
        metadata: BTreeMap::new(),
        organ_launch_contract: options.organ_launch_contract,
        proposal_modeling_request_id: options.proposal_modeling_request_id,
        claim_repair_request_id: options.claim_repair_request_id,
        frontier_planning_request_id: options.frontier_planning_request_id,
        frontier_plan_mind_request_id: options.frontier_plan_mind_request_id,
        imagination_consideration_request_id: options.imagination_consideration_request_id,
        admitted_model_direction_consideration_request_id: options
            .admitted_model_direction_consideration_request_id,
        repo_frontier_modeling_request_id: options.repo_frontier_modeling_request_id,
        repo_frontier_research_request_id: options.repo_frontier_research_request_id,
        repo_frontier_verdict_modeling_authority_msgpack: options
            .repo_frontier_verdict_modeling_authority
            .as_ref()
            .map(rmp_serde::to_vec_named)
            .transpose()?,
    };
    let envelopes = vec![
        cache.prepare_entry(RUNTIME_IDENTITY_KEY, &identity)?.0,
        cache.prepare_entry(&session.session_id, &session)?.0,
        cache.prepare_entry(&job.job_id, &job)?.0,
        cache.prepare_entry(&event.event_id, &event)?.0,
        cache.prepare_entry(&request.job_id, &request)?.0,
    ];
    Ok(PreparedRuntimeSpineHeartbeatJob { job, envelopes })
}

fn validate_repo_frontier_verdict_modeling_launch_authority(
    role: &str,
    request_id: Option<&str>,
    authority: Option<&RepoFrontierVerdictModelingLaunchAuthority>,
) -> Result<()> {
    match (request_id, authority) {
        (None, None) => Ok(()),
        (Some(_), None) => Err(anyhow!(
            "verdict-bound Modeling launch omitted its typed authority body"
        )),
        (None, Some(_)) => Err(anyhow!(
            "verdict-bound Modeling authority body has no indexed request id"
        )),
        (Some(request_id), Some(authority)) => {
            if role != EPIPHANY_MODELING_OWNER_ROLE {
                return Err(anyhow!(
                    "verdict-bound Modeling authority belongs only to the Modeling owner role"
                ));
            }
            if request_id != authority.request.request_id {
                return Err(anyhow!(
                    "verdict-bound Modeling request id does not match its typed authority body"
                ));
            }
            if authority.request.frontier_item_id != authority.frontier_item.id {
                return Err(anyhow!(
                    "verdict-bound Modeling authority does not carry its exact frontier item"
                ));
            }
            Ok(())
        }
    }
}

fn validate_proposal_modeling_launch_carrier(
    role: &str,
    binding_id: &str,
    proposal_modeling_request_id: Option<&str>,
    launch_document: &EpiphanyWorkerLaunchDocument,
) -> Result<()> {
    let projection = match launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => document.proposal_modeling_context.as_ref(),
        EpiphanyWorkerLaunchDocument::Reorient(_) => None,
    };
    let Some(request_id) = proposal_modeling_request_id else {
        if projection.is_some() {
            return Err(anyhow!(
                "proposal Modeling context requires its typed request id"
            ));
        }
        return Ok(());
    };
    validate_non_empty(request_id, "proposal Modeling request id")?;
    if role != EPIPHANY_MODELING_OWNER_ROLE || binding_id != EPIPHANY_MODELING_ROLE_BINDING_ID {
        return Err(anyhow!(
            "proposal Modeling request id may only be transported by the Modeling role launch"
        ));
    }
    let projection = projection.ok_or_else(|| {
        anyhow!("proposal Modeling request id requires coordinator-owned typed context")
    })?;
    if projection.request_id != request_id {
        return Err(anyhow!("proposal Modeling context/request mismatch"));
    }
    Ok(())
}

fn validate_repository_body_launch_carrier(
    role: &str,
    launch_document: &EpiphanyWorkerLaunchDocument,
) -> Result<()> {
    let (document_role, basis) = match launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => (
            Some(document.role_id.as_str()),
            document.repository_body_observation_basis.as_ref(),
        ),
        EpiphanyWorkerLaunchDocument::Reorient(_) => (None, None),
    };
    let owner_is_modeling = role == EPIPHANY_MODELING_OWNER_ROLE;
    let document_is_modeling = document_role.is_some_and(|value| value == "modeling");
    if owner_is_modeling != document_is_modeling {
        return Err(anyhow!(
            "Modeling runtime owner and typed launch role must agree"
        ));
    }
    if owner_is_modeling && basis.is_none() {
        return Err(anyhow!(
            "Modeling runtime launch requires a pre-thought repository Body basis"
        ));
    }
    if !owner_is_modeling && basis.is_some() {
        return Err(anyhow!(
            "non-Modeling runtime launch cannot carry a repository Body basis"
        ));
    }
    Ok(())
}

fn validate_claim_repair_launch_carrier(
    role: &str,
    binding_id: &str,
    claim_repair_request_id: Option<&str>,
    launch_document: &EpiphanyWorkerLaunchDocument,
) -> Result<()> {
    let projection = match launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => document.claim_repair_context.as_ref(),
        EpiphanyWorkerLaunchDocument::Reorient(_) => None,
    };
    let Some(request_id) = claim_repair_request_id else {
        if projection.is_some() {
            return Err(anyhow!(
                "claim repair context requires its typed request id"
            ));
        }
        return Ok(());
    };
    validate_non_empty(request_id, "claim repair request id")?;
    if role != EPIPHANY_MODELING_OWNER_ROLE || binding_id != EPIPHANY_MODELING_ROLE_BINDING_ID {
        return Err(anyhow!(
            "claim repair request id may only be transported by the Modeling role launch"
        ));
    }
    let projection = projection.ok_or_else(|| {
        anyhow!("claim repair request id requires coordinator-owned typed context")
    })?;
    if projection.request_id != request_id {
        return Err(anyhow!("claim repair context/request mismatch"));
    }
    Ok(())
}

fn validate_frontier_planning_launch_carrier(
    role: &str,
    binding_id: &str,
    planning_request_id: Option<&str>,
    launch_document: &EpiphanyWorkerLaunchDocument,
) -> Result<()> {
    let projection = match launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => document.frontier_planning_context.as_ref(),
        EpiphanyWorkerLaunchDocument::Reorient(_) => None,
    };
    let Some(request_id) = planning_request_id else {
        if projection.is_some() {
            return Err(anyhow!(
                "frontier planning context requires its typed request id"
            ));
        }
        return Ok(());
    };
    validate_non_empty(request_id, "frontier planning request id")?;
    if role != EPIPHANY_IMAGINATION_OWNER_ROLE || binding_id != EPIPHANY_IMAGINATION_ROLE_BINDING_ID
    {
        return Err(anyhow!(
            "frontier planning request may only be transported by the Imagination role launch"
        ));
    }
    let projection = projection
        .ok_or_else(|| anyhow!("frontier planning request requires its typed context"))?;
    if projection.request_id != request_id {
        return Err(anyhow!("frontier planning context/request mismatch"));
    }
    Ok(())
}

fn validate_imagination_consideration_launch_carrier(
    role: &str,
    binding_id: &str,
    request_id: Option<&str>,
    launch_document: &EpiphanyWorkerLaunchDocument,
) -> Result<()> {
    let projection = match launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => {
            document.imagination_consideration_context.as_ref()
        }
        EpiphanyWorkerLaunchDocument::Reorient(_) => None,
    };
    let Some(request_id) = request_id else {
        if projection.is_some() {
            return Err(anyhow!(
                "consideration context requires its typed request id"
            ));
        }
        return Ok(());
    };
    validate_non_empty(request_id, "imagination consideration request id")?;
    if role != EPIPHANY_IMAGINATION_OWNER_ROLE || binding_id != EPIPHANY_IMAGINATION_ROLE_BINDING_ID
    {
        return Err(anyhow!(
            "consideration may only be transported by Imagination"
        ));
    }
    if projection.map(|p| p.request.request_id.as_str()) != Some(request_id) {
        return Err(anyhow!("consideration context/request mismatch"));
    }
    Ok(())
}

pub fn runtime_job_snapshot(
    store_path: impl AsRef<Path>,
    job_id: &str,
) -> Result<Option<EpiphanyRuntimeJobSnapshot>> {
    validate_non_empty(job_id, "job id")?;
    let store_path = store_path.as_ref();
    if !store_path.exists() {
        return Ok(None);
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let Some(job) = cache.get::<EpiphanyRuntimeJob>(job_id)? else {
        return Ok(None);
    };
    let result = cache
        .get_all::<EpiphanyRuntimeJobResult>()?
        .into_iter()
        .filter(|result| result.job_id == job_id)
        .max_by(|left, right| {
            left.completed_at
                .cmp(&right.completed_at)
                .then_with(|| left.result_id.cmp(&right.result_id))
        });
    Ok(Some(EpiphanyRuntimeJobSnapshot { job, result }))
}

pub fn runtime_worker_launch_request(
    store_path: impl AsRef<Path>,
    job_id: &str,
) -> Result<Option<EpiphanyRuntimeWorkerLaunchRequest>> {
    validate_non_empty(job_id, "worker launch request job id")?;
    let store_path = store_path.as_ref();
    if !store_path.exists() {
        return Ok(None);
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<EpiphanyRuntimeWorkerLaunchRequest>(job_id)
}

pub fn runtime_worker_launch_body_basis(
    store_path: impl AsRef<Path>,
    job_id: &str,
) -> Result<Option<crate::RepositoryBodyObservationBasis>> {
    runtime_worker_launch_request(store_path, job_id)?
        .ok_or_else(|| anyhow!("worker launch request {job_id:?} is missing"))?
        .repository_body_observation_basis()
}

pub fn runtime_worker_process_claim(
    store_path: impl AsRef<Path>,
    job_id: &str,
) -> Result<Option<EpiphanyRuntimeWorkerProcessClaim>> {
    validate_non_empty(job_id, "worker process claim job id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<EpiphanyRuntimeWorkerProcessClaim>(&worker_process_claim_id(job_id))
}

fn validate_frontier_research_launch_carrier(
    role: &str,
    binding_id: &str,
    research_request_id: Option<&str>,
    launch_document: &EpiphanyWorkerLaunchDocument,
) -> Result<()> {
    let projection = match launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => document.frontier_research_context.as_ref(),
        EpiphanyWorkerLaunchDocument::Reorient(_) => None,
    };
    let Some(request_id) = research_request_id else {
        if projection.is_some() {
            return Err(anyhow!(
                "frontier Research context requires its typed request id"
            ));
        }
        return Ok(());
    };
    validate_non_empty(request_id, "frontier Research request id")?;
    if role != crate::EPIPHANY_RESEARCH_OWNER_ROLE
        || binding_id != crate::EPIPHANY_RESEARCH_ROLE_BINDING_ID
    {
        return Err(anyhow!(
            "frontier Research request may only be transported by the Research role launch"
        ));
    }
    let projection = projection
        .ok_or_else(|| anyhow!("frontier Research request requires its typed context"))?;
    if projection.request_id != request_id
        || projection.schema_version != crate::REPO_FRONTIER_RESEARCH_CONTEXT_SCHEMA_VERSION
        || projection.contract != crate::REPO_FRONTIER_RESEARCH_CONTEXT_CONTRACT
    {
        return Err(anyhow!("frontier Research context/request mismatch"));
    }
    Ok(())
}

pub fn put_runtime_requested_public_source_intents(
    store_path: impl AsRef<Path>,
    worker_job_id: &str,
    created_at: &str,
) -> Result<Vec<EpiphanyToolInvocationIntent>> {
    validate_non_empty(worker_job_id, "requested public source worker job id")?;
    chrono::DateTime::parse_from_rfc3339(created_at)
        .map_err(|error| anyhow!("requested public source intent time is invalid: {error}"))?;
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let launch = cache
        .get::<EpiphanyRuntimeWorkerLaunchRequest>(worker_job_id)?
        .ok_or_else(|| anyhow!("requested public source worker has no launch request"))?;
    let request = frontier_research_request_for_launch(&cache, &launch)?
        .ok_or_else(|| anyhow!("requested public source worker has no typed Research request"))?;
    let job = cache
        .get::<EpiphanyRuntimeJob>(worker_job_id)?
        .ok_or_else(|| anyhow!("requested public source worker has no runtime job"))?;
    drop(cache);

    let mut intents = Vec::new();
    for source_ref in request.public_source_refs {
        let source = crate::ImmutableGithubSource::parse(&source_ref)?;
        let intent_id = requested_public_source_intent_id(worker_job_id, &source_ref);
        let call_id = format!("call-{intent_id}");
        let mut intent = EpiphanyToolInvocationIntent::new(
            intent_id.clone(),
            epiphany_tool_adapter::EPIPHANY_TOOL_RUNTIME_ADAPTER_ID,
            "epiphany_public",
            "github_file",
            requested_public_source_arguments(&source),
            "epiphany-runtime-requested-public-source",
            format!(
                "Typed Research request {} requires immutable source {}.",
                request.request_id, source_ref
            ),
            created_at,
        );
        intent.call_id = Some(call_id);

        let mut cache = runtime_spine_cache(store_path)?;
        cache.pull_all_backing_stores()?;
        let existing_intent =
            cache.get::<EpiphanyToolInvocationIntent>(&tool_invocation_intent_key(&intent_id))?;
        let existing_binding = cache.get::<EpiphanyRuntimeToolExecutionBinding>(&intent_id)?;
        if let (Some(existing_intent), Some(existing_binding)) =
            (&existing_intent, &existing_binding)
        {
            validate_requested_public_source_intent(&cache, worker_job_id, existing_intent)?;
            if existing_intent.reason != intent.reason
                || chrono::DateTime::parse_from_rfc3339(&existing_intent.created_at).is_err()
                || existing_binding.intent_id != intent_id
                || existing_binding.job_id != worker_job_id
                || existing_binding.session_id != job.session_id
                || existing_binding.model_request_id.is_some()
            {
                return Err(anyhow!(
                    "requested public source intent {intent_id:?} collides with foreign authority"
                ));
            }
            let existing_intent = existing_intent.clone();
            drop(cache);
            require_runtime_tool_execution_binding(store_path, &intent_id)?;
            intents.push(existing_intent);
            continue;
        }
        if existing_intent.is_some() || existing_binding.is_some() {
            return Err(anyhow!(
                "requested public source intent {intent_id:?} has a partial persisted family"
            ));
        }
        drop(cache);
        put_runtime_tool_execution_intent(
            store_path,
            &job.session_id,
            worker_job_id,
            &intent,
            created_at,
        )?;
        intents.push(intent);
    }
    Ok(intents)
}

pub fn runtime_requested_public_source_refs_for_worker(
    store_path: impl AsRef<Path>,
    worker_job_id: &str,
) -> Result<Vec<String>> {
    validate_non_empty(worker_job_id, "requested public source worker job id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let launch = cache
        .get::<EpiphanyRuntimeWorkerLaunchRequest>(worker_job_id)?
        .ok_or_else(|| anyhow!("requested public source worker has no launch request"))?;
    Ok(frontier_research_request_for_launch(&cache, &launch)?
        .map(|request| request.public_source_refs)
        .unwrap_or_default())
}

fn frontier_research_request_for_launch(
    cache: &CultCache,
    launch: &EpiphanyRuntimeWorkerLaunchRequest,
) -> Result<Option<RepoFrontierResearchRequest>> {
    let document: EpiphanyWorkerLaunchDocument =
        rmp_serde::from_slice(&launch.launch_document_msgpack)?;
    validate_frontier_research_launch_carrier(
        &launch.role,
        &launch.binding_id,
        launch.repo_frontier_research_request_id.as_deref(),
        &document,
    )?;
    let Some(request_id) = launch.repo_frontier_research_request_id.as_deref() else {
        return Ok(None);
    };
    let request = cache
        .get::<RepoFrontierResearchRequest>(request_id)?
        .ok_or_else(|| anyhow!("frontier Research launch lost its typed request"))?;
    let runtime = require_identity(cache)?;
    let expected_request_id = crate::frontier_research_request_id(
        &request.runtime_id,
        &request.frontier_item_id,
        &request.frontier_item_hash,
    );
    if request.schema_version != REPO_FRONTIER_RESEARCH_REQUEST_SCHEMA_VERSION
        || request.contract != REPO_FRONTIER_RESEARCH_REQUEST_CONTRACT
        || request.request_id != request_id
        || request.request_id != expected_request_id
        || request.runtime_id != runtime.runtime_id
        || (crate::EpiphanyRepoModelBasis {
            projection_digest: request.model_projection_digest.clone(),
            source_documents: request.model_source_documents.clone(),
        })
        .validate_against_cache(cache)
        .is_err()
        || request.frontier_item_id.is_empty()
        || request.frontier_item_hash.is_empty()
        || request.thread_id.is_empty()
        || chrono::DateTime::parse_from_rfc3339(&request.requested_at).is_err()
        || request.source_scope.is_empty()
        || !safe_sorted_unique_paths(&request.source_scope)
        || !request
            .public_source_refs
            .windows(2)
            .all(|pair| pair[0] < pair[1])
        || !request.public_source_refs.iter().all(|source_ref| {
            crate::ImmutableGithubSource::parse(source_ref)
                .map(|source| source.to_string() == *source_ref)
                .unwrap_or(false)
        })
    {
        return Err(anyhow!(
            "frontier Research launch names an invalid typed request"
        ));
    }
    let carried = match document {
        EpiphanyWorkerLaunchDocument::Role(document) => document.frontier_research_context,
        EpiphanyWorkerLaunchDocument::Reorient(_) => None,
    };
    if carried.as_ref()
        != Some(&crate::RepoFrontierResearchContextProjection::from(
            &request,
        ))
    {
        return Err(anyhow!(
            "frontier Research launch carries substituted context"
        ));
    }
    Ok(Some(request))
}

pub fn runtime_authenticated_public_source_lookups_for_worker(
    store_path: impl AsRef<Path>,
    worker_job_id: &str,
) -> Result<Vec<EyesSourceLookupReceipt>> {
    validate_non_empty(worker_job_id, "public source worker job id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    authenticated_requested_public_source_lookups_for_worker(&cache, worker_job_id)
}

fn authenticated_public_source_lookup_receipts_for_worker(
    cache: &CultCache,
    worker_job_id: &str,
) -> Result<Vec<EyesSourceLookupReceipt>> {
    validate_non_empty(worker_job_id, "public source worker job id")?;
    require_identity(cache)?;
    let launch = cache
        .get::<EpiphanyRuntimeWorkerLaunchRequest>(worker_job_id)?
        .ok_or_else(|| anyhow!("public source worker has no launch request"))?;
    if launch.binding_id != crate::EPIPHANY_RESEARCH_ROLE_BINDING_ID
        || launch.role != crate::EPIPHANY_RESEARCH_OWNER_ROLE
    {
        return Err(anyhow!("public source evidence is Eyes-owned"));
    }
    let grant_id = format!("substrate-grant-{worker_job_id}");
    let model_request_ids = cache
        .get_all::<EpiphanyRuntimeModelExecutionBinding>()?
        .into_iter()
        .filter(|binding| binding.source_worker_job_id.as_deref() == Some(worker_job_id))
        .map(|binding| binding.request_id)
        .collect::<BTreeSet<_>>();
    let mut lookups = Vec::new();
    for binding in cache.get_all::<EpiphanyRuntimeToolExecutionBinding>()? {
        let model_owned = binding
            .model_request_id
            .as_deref()
            .is_some_and(|model_request_id| model_request_ids.contains(model_request_id));
        let request_owned = binding.model_request_id.is_none() && binding.job_id == worker_job_id;
        if !model_owned && !request_owned {
            continue;
        }
        let intent = cache
            .get::<EpiphanyToolInvocationIntent>(&tool_invocation_intent_key(&binding.intent_id))?
            .ok_or_else(|| anyhow!("public source tool binding lost its intent"))?;
        if intent.server != "epiphany_public" || intent.tool_name != "github_file" {
            continue;
        }
        let receipt = cache
            .get::<EpiphanyToolInvocationReceipt>(&tool_invocation_receipt_key(&intent.intent_id))?
            .ok_or_else(|| anyhow!("public source tool intent has no terminal receipt"))?;
        let _ = governed_source_tool_authority(&cache, &binding.job_id, &intent)?;
        validate_terminal_tool_execution_family(&binding, &intent, &receipt)?;
        if receipt.status != "completed" {
            return Err(anyhow!(
                "public source lookup did not complete successfully"
            ));
        }
        let arguments: serde_json::Value = serde_json::from_str(&intent.arguments_json)
            .context("public source intent arguments are invalid")?;
        let result: serde_json::Value = serde_json::from_str(
            receipt
                .result_json
                .as_deref()
                .ok_or_else(|| anyhow!("public source receipt has no result"))?,
        )
        .context("public source receipt result is invalid")?;
        let source = crate::ImmutableGithubSource::from_components(
            public_source_json_string(&arguments, "owner")?,
            public_source_json_string(&arguments, "repository")?,
            public_source_json_string(&arguments, "revision")?,
            public_source_json_string(&arguments, "path")?,
        )?;
        let content = public_source_json_string(&result, "content")?;
        let repository = source.repository_ref();
        let source_ref = source.to_string();
        let content_sha256 = format!("{:x}", Sha256::digest(content.as_bytes()));
        let byte_count = content.len() as u64;
        let lookup_receipt_id = format!("eyes-source-{}", intent.intent_id);
        if public_source_json_string(&result, "provider")? != "github"
            || public_source_json_string(&result, "repository")? != repository
            || public_source_json_string(&result, "revision")? != source.revision()
            || public_source_json_string(&result, "path")? != source.path()
            || public_source_json_string(&result, "sourceRef")? != source_ref
            || public_source_json_string(&result, "contentSha256")? != content_sha256
            || result.get("byteCount").and_then(serde_json::Value::as_u64) != Some(byte_count)
            || public_source_json_string(&result, "evidenceReceiptId")? != lookup_receipt_id
        {
            return Err(anyhow!("public source result provenance is substituted"));
        }
        lookups.push(EyesSourceLookupReceipt {
            schema_version: crate::EYES_SOURCE_LOOKUP_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: lookup_receipt_id,
            source_job_id: worker_job_id.to_string(),
            substrate_grant_receipt_id: grant_id.clone(),
            tool_intent_id: intent.intent_id,
            tool_receipt_id: receipt.receipt_id,
            provider: "github".to_string(),
            repository,
            revision: source.revision().to_string(),
            path: source.path().to_string(),
            source_ref,
            content_sha256,
            byte_count,
            observed_at: receipt.completed_at,
            contract: "Eyes admitted one bounded immutable public source only after authenticating its worker, causal request or model execution, tool, grant, provider identity, and content digest.".to_string(),
        });
    }
    lookups.sort_by(|left, right| left.receipt_id.cmp(&right.receipt_id));
    Ok(lookups)
}

/// Authenticate the complete public-source consequence of one Research
/// attempt. The typed frontier request owns the allowed set; a capability
/// grant or a valid receipt cannot widen it, and an omitted lookup cannot
/// silently satisfy it.
pub(crate) fn authenticated_requested_public_source_lookups_for_worker(
    cache: &CultCache,
    worker_job_id: &str,
) -> Result<Vec<EyesSourceLookupReceipt>> {
    validate_non_empty(worker_job_id, "frontier public source worker job id")?;
    let launch = cache
        .get::<EpiphanyRuntimeWorkerLaunchRequest>(worker_job_id)?
        .ok_or_else(|| anyhow!("frontier public source worker has no launch request"))?;
    let request = frontier_research_request_for_launch(cache, &launch)?;
    let lookups = authenticated_public_source_lookup_receipts_for_worker(cache, worker_job_id)?;
    let expected = request
        .into_iter()
        .flat_map(|request| request.public_source_refs)
        .collect::<BTreeSet<_>>();
    let observed = lookups
        .iter()
        .map(|lookup| lookup.source_ref.clone())
        .collect::<BTreeSet<_>>();
    if observed != expected {
        let missing = expected.difference(&observed).cloned().collect::<Vec<_>>();
        let unrequested = observed.difference(&expected).cloned().collect::<Vec<_>>();
        return Err(anyhow!(
            "frontier Research public source coverage is not exact; missing={missing:?}; unrequested={unrequested:?}"
        ));
    }
    Ok(lookups)
}

fn public_source_json_string<'a>(value: &'a serde_json::Value, field: &str) -> Result<&'a str> {
    value
        .get(field)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow!("public source cargo omitted {field:?}"))
}

fn validate_governed_source_tool_intent(
    cache: &CultCache,
    job_id: &str,
    intent: &EpiphanyToolInvocationIntent,
) -> Result<Vec<CultCacheEnvelope>> {
    let authority = governed_source_tool_authority(cache, job_id, intent)?;
    if let Some(source_worker_job_id) = authority.source_worker_job_id.as_deref() {
        let source_worker_job = cache
            .get::<EpiphanyRuntimeJob>(source_worker_job_id)?
            .ok_or_else(|| anyhow!("governed source tool intent lost its source worker job"))?;
        if matches!(
            source_worker_job.status,
            EpiphanyRuntimeJobStatus::Completed
                | EpiphanyRuntimeJobStatus::Failed
                | EpiphanyRuntimeJobStatus::Cancelled
        ) {
            return Err(anyhow!(
                "governed source tool intent {:?} is outside its launch grant",
                intent.intent_id
            ));
        }
    }
    Ok(authority.envelopes)
}

struct GovernedSourceToolAuthority {
    source_worker_job_id: Option<String>,
    envelopes: Vec<CultCacheEnvelope>,
}

fn governed_source_tool_authority(
    cache: &CultCache,
    job_id: &str,
    intent: &EpiphanyToolInvocationIntent,
) -> Result<GovernedSourceToolAuthority> {
    let required_operation =
        crate::substrate_gate_operation_for_governed_tool(&intent.server, &intent.tool_name);
    let source_worker_job_id = match intent.model_request_id.as_deref() {
        Some(model_request_id) => {
            let model_binding = cache
                .get::<EpiphanyRuntimeModelExecutionBinding>(model_request_id)?
                .ok_or_else(|| {
                    anyhow!(
                        "model-derived tool intent {:?} has no execution binding",
                        intent.intent_id
                    )
                })?;
            match model_binding.source_worker_job_id {
                Some(worker_job_id) => Some(worker_job_id),
                None if required_operation.is_some() => {
                    return Err(anyhow!(
                        "governed source tool intent {:?} has no source worker authority",
                        intent.intent_id
                    ));
                }
                None => None,
            }
        }
        None => {
            if cache
                .get::<EpiphanyRuntimeWorkerLaunchRequest>(job_id)?
                .is_some_and(|launch| {
                    matches!(
                        launch.binding_id.as_str(),
                        crate::EPIPHANY_RESEARCH_ROLE_BINDING_ID
                            | crate::EPIPHANY_MODELING_ROLE_BINDING_ID
                            | crate::EPIPHANY_VERIFICATION_ROLE_BINDING_ID
                    )
                })
            {
                validate_requested_public_source_intent(cache, job_id, intent)?;
                Some(job_id.to_string())
            } else {
                None
            }
        }
    };
    let Some(source_worker_job_id) = source_worker_job_id else {
        return Ok(GovernedSourceToolAuthority {
            source_worker_job_id: None,
            envelopes: Vec::new(),
        });
    };
    let launch = cache
        .get::<EpiphanyRuntimeWorkerLaunchRequest>(&source_worker_job_id)?
        .ok_or_else(|| anyhow!("governed source tool intent lost its source worker launch"))?;
    let source_worker_job = cache
        .get::<EpiphanyRuntimeJob>(&source_worker_job_id)?
        .ok_or_else(|| anyhow!("governed source tool intent lost its source worker job"))?;
    if !matches!(
        launch.binding_id.as_str(),
        crate::EPIPHANY_RESEARCH_ROLE_BINDING_ID
            | crate::EPIPHANY_MODELING_ROLE_BINDING_ID
            | crate::EPIPHANY_VERIFICATION_ROLE_BINDING_ID
    ) {
        return Ok(GovernedSourceToolAuthority {
            source_worker_job_id: None,
            envelopes: Vec::new(),
        });
    }
    let required_operation = required_operation.ok_or_else(|| {
        anyhow!(
            "governed source worker rejected unknown tool {}::{}",
            intent.server,
            intent.tool_name
        )
    })?;
    let grant_id = format!("substrate-grant-{source_worker_job_id}");
    let grant = cache
        .get::<SubstrateGateRepoAccessGrantReceipt>(&grant_id)?
        .ok_or_else(|| {
            anyhow!(
                "governed source tool intent {:?} has no exact launch grant",
                intent.intent_id
            )
        })?;
    if source_worker_job.role != launch.role
        || grant.runtime_job_id != source_worker_job_id
        || grant.binding_id != launch.binding_id
        || grant.role != launch.role
        || grant.authority_scope != launch.authority_scope
        || !grant
            .granted_operations
            .iter()
            .any(|operation| operation == required_operation)
    {
        return Err(anyhow!(
            "governed source tool intent {:?} is outside its launch grant",
            intent.intent_id
        ));
    }
    Ok(GovernedSourceToolAuthority {
        source_worker_job_id: Some(source_worker_job_id.clone()),
        envelopes: vec![
            cache
                .get_envelope::<EpiphanyRuntimeJob>(&source_worker_job_id)?
                .ok_or_else(|| anyhow!("governed source tool lost worker job envelope"))?,
            cache
                .get_envelope::<EpiphanyRuntimeWorkerLaunchRequest>(&source_worker_job_id)?
                .ok_or_else(|| anyhow!("governed source tool lost worker launch envelope"))?,
            cache
                .get_envelope::<SubstrateGateRepoAccessGrantReceipt>(&grant_id)?
                .ok_or_else(|| anyhow!("governed source tool lost launch grant envelope"))?,
        ],
    })
}

fn requested_public_source_intent_id(worker_job_id: &str, source_ref: &str) -> String {
    format!(
        "requested-public-source-{:x}",
        Sha256::digest(format!("{worker_job_id}:{source_ref}").as_bytes())
    )
}

fn requested_public_source_arguments(source: &crate::ImmutableGithubSource) -> String {
    serde_json::json!({
        "owner": source.owner(),
        "repository": source.repository_name(),
        "revision": source.revision(),
        "path": source.path(),
        "maxBytes": 65536
    })
    .to_string()
}

fn validate_requested_public_source_intent(
    cache: &CultCache,
    worker_job_id: &str,
    intent: &EpiphanyToolInvocationIntent,
) -> Result<()> {
    let launch = cache
        .get::<EpiphanyRuntimeWorkerLaunchRequest>(worker_job_id)?
        .ok_or_else(|| anyhow!("requested public source intent lost its worker launch"))?;
    let request = frontier_research_request_for_launch(cache, &launch)?
        .ok_or_else(|| anyhow!("requested public source intent has no typed Research request"))?;
    let arguments: serde_json::Value = serde_json::from_str(&intent.arguments_json)
        .context("requested public source intent arguments are invalid")?;
    let source = crate::ImmutableGithubSource::from_components(
        public_source_json_string(&arguments, "owner")?,
        public_source_json_string(&arguments, "repository")?,
        public_source_json_string(&arguments, "revision")?,
        public_source_json_string(&arguments, "path")?,
    )?;
    let source_ref = source.to_string();
    let expected_id = requested_public_source_intent_id(worker_job_id, &source_ref);
    let expected_call_id = format!("call-{expected_id}");
    if intent.intent_id != expected_id
        || intent.adapter != epiphany_tool_adapter::EPIPHANY_TOOL_RUNTIME_ADAPTER_ID
        || intent.server != "epiphany_public"
        || intent.tool_name != "github_file"
        || intent.arguments_json != requested_public_source_arguments(&source)
        || intent.caller != "epiphany-runtime-requested-public-source"
        || intent.call_id.as_deref() != Some(expected_call_id.as_str())
        || intent.model_request_id.is_some()
        || !request.public_source_refs.contains(&source_ref)
    {
        return Err(anyhow!(
            "governed source worker refuses a noncanonical request-owned public lookup"
        ));
    }
    Ok(())
}

pub fn runtime_worker_process_claims(
    store_path: impl AsRef<Path>,
) -> Result<Vec<EpiphanyRuntimeWorkerProcessClaim>> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut claims = cache.get_all::<EpiphanyRuntimeWorkerProcessClaim>()?;
    claims.sort_by(|left, right| left.job_id.cmp(&right.job_id));
    Ok(claims)
}

pub fn claim_runtime_worker_process(
    store_path: impl AsRef<Path>,
    job_id: &str,
    process: &crate::ProcessInstanceIdentity,
    activation_token_sha256: &str,
    claimed_at: &str,
) -> Result<EpiphanyRuntimeWorkerProcessClaim> {
    let store_path = store_path.as_ref();
    validate_non_empty(job_id, "worker process claim job id")?;
    validate_non_empty(activation_token_sha256, "worker activation token digest")?;
    if activation_token_sha256.len() != 64
        || !activation_token_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(anyhow!("worker activation token digest is not SHA-256 hex"));
    }
    chrono::DateTime::parse_from_rfc3339(claimed_at)
        .map_err(|error| anyhow!("worker process claim timestamp is invalid: {error}"))?;
    if process.process_id == 0
        || process.creation_token == 0
        || process.executable_path.as_os_str().is_empty()
    {
        return Err(anyhow!("worker process claim identity is incomplete"));
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    if cache
        .get::<EpiphanyRuntimeWorkerLaunchRequest>(job_id)?
        .is_none()
    {
        return Err(anyhow!(
            "worker process claim requires its immutable launch"
        ));
    }
    if cache
        .get::<EpiphanyRuntimeRoleWorkerResult>(job_id)?
        .is_some()
    {
        return Err(anyhow!("worker process cannot claim after terminal result"));
    }
    let claim_id = worker_process_claim_id(job_id);
    let claim = EpiphanyRuntimeWorkerProcessClaim {
        schema_version: RUNTIME_WORKER_PROCESS_CLAIM_SCHEMA_VERSION.into(),
        claim_id: claim_id.clone(),
        job_id: job_id.into(),
        process_id: process.process_id,
        process_creation_token: process.creation_token,
        process_executable_path: process.executable_path.display().to_string(),
        activation_token_sha256: activation_token_sha256.into(),
        status: WorkerProcessStatus::Claimed.as_str().into(),
        claimed_at: claimed_at.into(),
        activated_at: None,
        terminal_at: None,
        terminal_authority_id: None,
    };
    if let Some(existing) = cache.get::<EpiphanyRuntimeWorkerProcessClaim>(&claim_id)? {
        return if existing == claim {
            Ok(existing)
        } else {
            Err(anyhow!("worker process claim identity is already owned"))
        };
    }
    let envelope = cache.prepare_entry(&claim_id, &claim)?.0;
    if SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&[], vec![envelope])?
    {
        Ok(claim)
    } else {
        Err(anyhow!("worker process claim lost immutable insertion"))
    }
}

pub fn activate_runtime_worker_process(
    store_path: impl AsRef<Path>,
    job_id: &str,
    process: &crate::ProcessInstanceIdentity,
    activation_token: &str,
    activated_at: &str,
) -> Result<EpiphanyRuntimeWorkerProcessClaim> {
    let store_path = store_path.as_ref();
    validate_non_empty(activation_token, "worker activation token")?;
    chrono::DateTime::parse_from_rfc3339(activated_at)
        .map_err(|error| anyhow!("worker activation timestamp is invalid: {error}"))?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let claim_id = worker_process_claim_id(job_id);
    let current = cache
        .get::<EpiphanyRuntimeWorkerProcessClaim>(&claim_id)?
        .ok_or_else(|| anyhow!("worker activation requires its process claim"))?;
    let digest = format!("{:x}", Sha256::digest(activation_token.as_bytes()));
    if current.job_id != job_id
        || current.process_id != process.process_id
        || current.process_creation_token != process.creation_token
        || current.process_executable_path != process.executable_path.display().to_string()
        || current.activation_token_sha256 != digest
    {
        return Err(anyhow!(
            "worker activation does not bind the exact process claim"
        ));
    }
    let status = WorkerProcessStatus::parse(&current.status)?;
    if status == WorkerProcessStatus::Active {
        return Ok(current);
    }
    if status != WorkerProcessStatus::Claimed {
        return Err(anyhow!(
            "worker activation found terminal process authority"
        ));
    }
    let current_envelope = cache
        .get_envelope::<EpiphanyRuntimeWorkerProcessClaim>(&claim_id)?
        .ok_or_else(|| anyhow!("worker activation lost its claim envelope"))?;
    let mut next = current;
    next.status = WorkerProcessStatus::Active.as_str().into();
    next.activated_at = Some(activated_at.into());
    let next_envelope = cache.prepare_entry(&claim_id, &next)?.0;
    if SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&[current_envelope], vec![next_envelope])?
    {
        Ok(next)
    } else {
        Err(anyhow!("worker activation lost its exact claim snapshot"))
    }
}

pub fn abandon_unactivated_runtime_worker_process(
    store_path: impl AsRef<Path>,
    job_id: &str,
    process: &crate::ProcessInstanceIdentity,
    terminal_at: &str,
) -> Result<EpiphanyRuntimeWorkerProcessClaim> {
    let store_path = store_path.as_ref();
    chrono::DateTime::parse_from_rfc3339(terminal_at)
        .map_err(|error| anyhow!("worker abandonment timestamp is invalid: {error}"))?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let claim_id = worker_process_claim_id(job_id);
    let current = cache
        .get::<EpiphanyRuntimeWorkerProcessClaim>(&claim_id)?
        .ok_or_else(|| anyhow!("worker abandonment requires its process claim"))?;
    if current.process_id != process.process_id
        || current.process_creation_token != process.creation_token
        || current.process_executable_path != process.executable_path.display().to_string()
        || WorkerProcessStatus::parse(&current.status)? != WorkerProcessStatus::Claimed
    {
        return Err(anyhow!(
            "worker abandonment does not own an unactivated exact claim"
        ));
    }
    let current_envelope = cache
        .get_envelope::<EpiphanyRuntimeWorkerProcessClaim>(&claim_id)?
        .ok_or_else(|| anyhow!("worker abandonment lost its claim envelope"))?;
    let mut next = current;
    next.status = WorkerProcessStatus::TerminalUnactivated.as_str().into();
    next.terminal_at = Some(terminal_at.into());
    next.terminal_authority_id = Some(format!("worker-unactivated-{job_id}"));
    let next_envelope = cache.prepare_entry(&claim_id, &next)?.0;
    if SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&[current_envelope], vec![next_envelope])?
    {
        Ok(next)
    } else {
        Err(anyhow!("worker abandonment lost its exact claim snapshot"))
    }
}

pub(crate) fn terminalize_runtime_worker_process_death(
    store_path: impl AsRef<Path>,
    job_id: &str,
    terminal_authority_id: &str,
    terminal_at: &str,
) -> Result<EpiphanyRuntimeWorkerProcessClaim> {
    let store_path = store_path.as_ref();
    validate_non_empty(terminal_authority_id, "worker death terminal authority")?;
    chrono::DateTime::parse_from_rfc3339(terminal_at)
        .map_err(|error| anyhow!("worker death timestamp is invalid: {error}"))?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let claim_id = worker_process_claim_id(job_id);
    let current = cache
        .get::<EpiphanyRuntimeWorkerProcessClaim>(&claim_id)?
        .ok_or_else(|| anyhow!("worker death recovery requires its process claim"))?;
    let status = WorkerProcessStatus::parse(&current.status)?;
    if status == WorkerProcessStatus::TerminalDeath
        && current.terminal_authority_id.as_deref() == Some(terminal_authority_id)
    {
        return Ok(current);
    }
    if !status.is_live() {
        return Err(anyhow!(
            "worker death recovery found terminal process authority"
        ));
    }
    if cache
        .get::<EpiphanyRuntimeRoleWorkerResult>(job_id)?
        .is_some()
    {
        return Err(anyhow!("worker death recovery races a terminal result"));
    }
    let current_envelope = cache
        .get_envelope::<EpiphanyRuntimeWorkerProcessClaim>(&claim_id)?
        .ok_or_else(|| anyhow!("worker death recovery lost its claim envelope"))?;
    let mut next = current;
    next.status = WorkerProcessStatus::TerminalDeath.as_str().into();
    next.terminal_at = Some(terminal_at.into());
    next.terminal_authority_id = Some(terminal_authority_id.into());
    let next_envelope = cache.prepare_entry(&claim_id, &next)?.0;
    if SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&[current_envelope], vec![next_envelope])?
    {
        Ok(next)
    } else {
        Err(anyhow!(
            "worker death recovery lost its exact claim snapshot"
        ))
    }
}

pub fn put_runtime_role_worker_result(
    store_path: impl AsRef<Path>,
    result: &EpiphanyRuntimeRoleWorkerResult,
) -> Result<()> {
    let store_path = store_path.as_ref();
    validate_non_empty(&result.job_id, "role worker result job id")?;
    validate_non_empty(&result.result_id, "role worker result id")?;
    validate_non_empty(&result.role_id, "role worker result role id")?;
    validate_non_empty(
        &result.decision_context_id,
        "role worker result decision context id",
    )?;
    if result.schema_version != RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION {
        return Err(anyhow!("role worker result schema version mismatch"));
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_worker_decision_context(&cache, &result.decision_context_id, &result.job_id)?;
    let worker_launch = cache
        .get::<EpiphanyRuntimeWorkerLaunchRequest>(&result.job_id)?
        .ok_or_else(|| anyhow!("role worker result requires its immutable worker launch"))?;
    let launch_document = worker_launch.launch_document()?;
    let launch_body_basis = match &launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => {
            document.repository_body_observation_basis.as_ref()
        }
        EpiphanyWorkerLaunchDocument::Reorient(_) => None,
    };
    let launch_is_modeling = worker_launch.role == EPIPHANY_MODELING_OWNER_ROLE;
    let result_is_modeling = result.role_id.eq_ignore_ascii_case("modeling");
    if launch_is_modeling != result_is_modeling {
        return Err(anyhow!(
            "role worker result cannot substitute Modeling launch authority"
        ));
    }
    if launch_is_modeling {
        let expected = launch_body_basis.ok_or_else(|| {
            anyhow!("Modeling worker launch is missing its repository Body observation basis")
        })?;
        if result.repository_body_observation_basis.as_ref() != Some(expected) {
            return Err(anyhow!(
                "Modeling runtime result must exactly bind its immutable launch repository Body observation basis"
            ));
        }
    } else if launch_body_basis.is_some() || result.repository_body_observation_basis.is_some() {
        return Err(anyhow!(
            "non-Modeling worker launch and result must not carry a repository Body observation basis"
        ));
    }
    let is_verification = result.role_id == "verification";
    if is_verification
        != (result
            .verification_request_id
            .as_ref()
            .is_some_and(|id| !id.trim().is_empty())
            && result
                .frontier_route_id
                .as_ref()
                .is_some_and(|id| !id.trim().is_empty()))
    {
        return Err(anyhow!(
            "Verification results require verificationRequestId and frontierRouteId; other roles must not claim them"
        ));
    }
    if result.repo_frontier_modeling_request_id.is_some()
        && !result.role_id.eq_ignore_ascii_case("modeling")
    {
        return Err(anyhow!(
            "only Modeling results may carry a frontier Modeling request binding"
        ));
    }
    if result.proposal_modeling_request_id.is_some()
        && !result.role_id.eq_ignore_ascii_case("modeling")
    {
        return Err(anyhow!(
            "only Modeling results may carry a proposal Modeling request binding"
        ));
    }
    if result.claim_repair_request_id.is_some() && !result.role_id.eq_ignore_ascii_case("modeling")
    {
        return Err(anyhow!(
            "only Modeling results may carry a claim repair request binding"
        ));
    }
    let is_modeling = result.role_id.eq_ignore_ascii_case("modeling");
    let modeling_binding_count = [
        result.repo_frontier_modeling_request_id.is_some(),
        result.proposal_modeling_request_id.is_some(),
        result.claim_repair_request_id.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count();
    if is_modeling && modeling_binding_count > 1 {
        return Err(anyhow!(
            "Modeling result authority bindings are mutually exclusive"
        ));
    }
    if is_modeling
        && (result.repo_frontier_modeling_request_id
            != worker_launch.repo_frontier_modeling_request_id
            || result.proposal_modeling_request_id != worker_launch.proposal_modeling_request_id
            || result.claim_repair_request_id != worker_launch.claim_repair_request_id)
    {
        return Err(anyhow!(
            "Modeling result must exactly preserve its runtime-owned request authority"
        ));
    }
    if is_modeling
        && (result.verification_request_id.is_some() || result.frontier_route_id.is_some())
    {
        return Err(anyhow!(
            "Modeling result cannot claim Verification route authority"
        ));
    }
    if is_modeling
        && (modeling_binding_count == 1 || result.repo_model_mutation_proposal_msgpack.is_some())
    {
        let proposal = result.repo_model_mutation_proposal()?.ok_or_else(|| {
            anyhow!("Modeling authority binding requires a RepoModel mutation proposal")
        })?;
        proposal.validate()?;
        let operations = proposal.operations()?;
        if worker_launch.claim_repair_request_id.is_some() {
            if result.state_patch_msgpack.is_some()
                || result.self_patch_msgpack.is_some()
                || operations.iter().any(|operation| {
                    !matches!(
                        operation,
                        crate::EpiphanyRepoModelMutationOperation::PutNode { .. }
                    )
                })
            {
                return Err(anyhow!(
                    "claim repair request may authorize only semantic RepoModel node mutation"
                ));
            }
        } else if (worker_launch.repo_frontier_modeling_request_id.is_some()
            || worker_launch.proposal_modeling_request_id.is_some())
            && operations.iter().any(|operation| {
                !matches!(
                    operation,
                    crate::EpiphanyRepoModelMutationOperation::PutFrontier { .. }
                )
            })
        {
            return Err(anyhow!(
                "frontier Modeling request may authorize only semantic RepoModel frontier mutation"
            ));
        }
    }
    let has_planning_echo = result.frontier_planning_request_id.is_some();
    let has_planning_candidate = result.frontier_plan_candidate_msgpack.is_some();
    if has_planning_echo != has_planning_candidate {
        return Err(anyhow!(
            "frontier planning result requires both its request echo and typed candidate"
        ));
    }
    if has_planning_echo {
        if !result.role_id.eq_ignore_ascii_case("imagination") {
            return Err(anyhow!(
                "only Imagination results may echo a frontier planning request"
            ));
        }
        if result.item_error.is_some() {
            return Err(anyhow!(
                "frontier planning result with an item error cannot carry an executable candidate"
            ));
        }
        if result.state_patch_msgpack.is_some()
            || result.self_patch_msgpack.is_some()
            || result.repo_model_mutation_proposal_msgpack.is_some()
            || result.verification_request_id.is_some()
            || result.frontier_route_id.is_some()
            || result.repo_frontier_modeling_request_id.is_some()
            || result.proposal_modeling_request_id.is_some()
            || result.claim_repair_request_id.is_some()
        {
            return Err(anyhow!(
                "frontier planning result may carry only its request echo and typed candidate authority"
            ));
        }
        let request_id = result
            .frontier_planning_request_id
            .as_deref()
            .ok_or_else(|| anyhow!("frontier planning request echo disappeared"))?;
        let request = cache
            .get::<RepoFrontierPlanningRequest>(request_id)?
            .ok_or_else(|| anyhow!("frontier planning result requires persisted request"))?;
        validate_actionable_repo_frontier_planning_request(&cache, &request)?;
        let candidate = result
            .frontier_plan_candidate()?
            .ok_or_else(|| anyhow!("frontier planning candidate disappeared"))?;
        validate_repo_frontier_plan_candidate_against_request(&cache, &candidate, &request)?;
        let bindings = cache
            .get_all::<RepoFrontierPlanningLaunchBinding>()?
            .into_iter()
            .filter(|binding| {
                binding.planning_request_id == request.request_id && binding.job_id == result.job_id
            })
            .collect::<Vec<_>>();
        if bindings.len() != 1 {
            return Err(anyhow!(
                "frontier planning result requires exactly one coordinator launch binding"
            ));
        }
        let binding = &bindings[0];
        let worker_launch = cache
            .get::<EpiphanyRuntimeWorkerLaunchRequest>(&result.job_id)?
            .ok_or_else(|| anyhow!("frontier planning result requires its worker launch"))?;
        let launch_document = worker_launch.launch_document()?;
        let projection = match &launch_document {
            EpiphanyWorkerLaunchDocument::Role(document) => {
                document.frontier_planning_context.as_ref()
            }
            EpiphanyWorkerLaunchDocument::Reorient(_) => None,
        };
        let expected_projection =
            crate::RepoFrontierPlanningContextProjection::from_request(&request);
        let launch_hash = format!(
            "{:x}",
            Sha256::digest(&worker_launch.launch_document_msgpack)
        );
        let expected_binding_record_id = if binding.attempt_ordinal == 0 {
            format!("repo-frontier-planning-launch-{}", request.request_id)
        } else {
            format!(
                "repo-frontier-planning-launch-{}-attempt-{}",
                request.request_id, binding.attempt_ordinal
            )
        };
        if binding.schema_version != REPO_FRONTIER_PLANNING_LAUNCH_BINDING_SCHEMA_VERSION
            || binding.contract != REPO_FRONTIER_PLANNING_LAUNCH_BINDING_CONTRACT
            || binding.binding_record_id != expected_binding_record_id
            || binding.job_id != result.job_id
            || binding.binding_id != EPIPHANY_IMAGINATION_ROLE_BINDING_ID
            || binding.runtime_id != request.runtime_id
            || binding.thread_id != request.thread_id
            || binding.worker_launch_document_sha256 != launch_hash
            || worker_launch.job_id != result.job_id
            || worker_launch.role != EPIPHANY_IMAGINATION_OWNER_ROLE
            || worker_launch.binding_id != EPIPHANY_IMAGINATION_ROLE_BINDING_ID
            || worker_launch.frontier_planning_request_id.as_deref()
                != Some(request.request_id.as_str())
            || worker_launch.proposal_modeling_request_id.is_some()
            || worker_launch.claim_repair_request_id.is_some()
            || projection != Some(&expected_projection)
        {
            return Err(anyhow!(
                "frontier planning result does not exactly bind request, launch, runtime, thread, and candidate"
            ));
        }
    }
    let mut model_direction_companion = None;
    let has_model_direction_echo = result
        .admitted_model_direction_consideration_request_id
        .is_some();
    let has_model_direction_result = result
        .admitted_model_direction_consideration_result_msgpack
        .is_some();
    if has_model_direction_echo != has_model_direction_result {
        return Err(anyhow!(
            "model direction result requires exact request echo and result"
        ));
    }
    if has_model_direction_echo {
        if !result.role_id.eq_ignore_ascii_case("imagination")
            || result.item_error.is_some()
            || result.state_patch_msgpack.is_some()
            || result.self_patch_msgpack.is_some()
            || result.repo_model_mutation_proposal_msgpack.is_some()
            || result.imagination_consideration_request_id.is_some()
            || result.imagination_consideration_candidate_msgpack.is_some()
        {
            return Err(anyhow!(
                "model direction result carries foreign authority cargo"
            ));
        }
        let request_id = result
            .admitted_model_direction_consideration_request_id
            .as_deref()
            .unwrap();
        let request = cache
            .get::<crate::AdmittedModelDirectionConsiderationRequest>(request_id)?
            .ok_or_else(|| anyhow!("model direction result request disappeared"))?;
        crate::validate_current_admitted_model_direction_consideration_request(&cache, &request)?;
        let direction_result = result
            .admitted_model_direction_consideration_result()?
            .ok_or_else(|| anyhow!("model direction result disappeared"))?;
        crate::validate_admitted_model_direction_consideration_result(&request, &direction_result)?;
        if direction_result.result_id
            != crate::admitted_model_direction_consideration_result_id_for_launch(
                request_id,
                &result.job_id,
            )
        {
            return Err(anyhow!(
                "model direction result identity was not assigned by exact launch"
            ));
        }
        let worker = cache
            .get::<EpiphanyRuntimeWorkerLaunchRequest>(&result.job_id)?
            .ok_or_else(|| anyhow!("model direction result requires worker launch"))?;
        let document = worker.launch_document()?;
        let projection = match &document {
            EpiphanyWorkerLaunchDocument::Role(document) => document
                .admitted_model_direction_consideration_context
                .as_ref(),
            EpiphanyWorkerLaunchDocument::Reorient(_) => None,
        };
        if worker.role != EPIPHANY_IMAGINATION_OWNER_ROLE
            || worker.binding_id != EPIPHANY_IMAGINATION_ROLE_BINDING_ID
            || worker
                .admitted_model_direction_consideration_request_id
                .as_deref()
                != Some(request_id)
            || projection.map(|projection| &projection.request) != Some(&request)
            || projection.map(|projection| projection.model.reasoning_basis())
                != Some(crate::EpiphanyRepoModelBasis {
                    projection_digest: request.model_projection_digest.clone(),
                    source_documents: request.model_source_documents.clone(),
                })
        {
            return Err(anyhow!(
                "model direction result does not exactly bind request and launch"
            ));
        }
        model_direction_companion = Some(direction_result);
    }
    let mut imagination_candidate_companion = None;
    let has_consideration_echo = result.imagination_consideration_request_id.is_some();
    let has_consideration_candidate = result.imagination_consideration_candidate_msgpack.is_some();
    if has_consideration_echo != has_consideration_candidate {
        return Err(anyhow!(
            "consideration result requires request echo and candidate"
        ));
    }
    if has_consideration_echo {
        if !result.role_id.eq_ignore_ascii_case("imagination")
            || result.item_error.is_some()
            || result.state_patch_msgpack.is_some()
            || result.self_patch_msgpack.is_some()
            || result.repo_model_mutation_proposal_msgpack.is_some()
            || result.verification_request_id.is_some()
            || result.frontier_route_id.is_some()
            || result.repo_frontier_modeling_request_id.is_some()
            || result.proposal_modeling_request_id.is_some()
            || result.claim_repair_request_id.is_some()
            || result.frontier_planning_request_id.is_some()
            || result.frontier_plan_candidate_msgpack.is_some()
            || result.frontier_plan_mind_request_id.is_some()
            || result.frontier_plan_mind_decision_msgpack.is_some()
        {
            return Err(anyhow!(
                "consideration result carries foreign authority cargo"
            ));
        }
        let request_id = result
            .imagination_consideration_request_id
            .as_deref()
            .unwrap();
        let request = cache
            .get::<crate::ImaginationConsiderationRequest>(request_id)?
            .ok_or_else(|| anyhow!("consideration result request disappeared"))?;
        crate::validate_current_imagination_consideration_request(&cache, &request)?;
        let candidate = result
            .imagination_consideration_candidate()?
            .ok_or_else(|| anyhow!("consideration candidate disappeared"))?;
        crate::validate_imagination_consideration_candidate(&request, &candidate)?;
        if candidate.candidate_id
            != crate::imagination_consideration_candidate_id_for_launch(request_id, &result.job_id)
        {
            return Err(anyhow!(
                "consideration candidate identity was not assigned by exact launch"
            ));
        }
        let bindings = cache
            .get_all::<crate::ImaginationConsiderationLaunchBinding>()?
            .into_iter()
            .filter(|binding| binding.request_id == request_id)
            .collect::<Vec<_>>();
        if bindings.len() != 1 {
            return Err(anyhow!("consideration result requires one launch binding"));
        }
        let binding = &bindings[0];
        let worker = cache
            .get::<EpiphanyRuntimeWorkerLaunchRequest>(&result.job_id)?
            .ok_or_else(|| anyhow!("consideration result requires worker launch"))?;
        let document = worker.launch_document()?;
        let projection = match &document {
            EpiphanyWorkerLaunchDocument::Role(document) => {
                document.imagination_consideration_context.as_ref()
            }
            EpiphanyWorkerLaunchDocument::Reorient(_) => None,
        };
        let launch_hash = format!("{:x}", Sha256::digest(&worker.launch_document_msgpack));
        if binding.job_id != result.job_id
            || binding.binding_id != EPIPHANY_IMAGINATION_ROLE_BINDING_ID
            || binding.runtime_id != request.runtime_id
            || binding.thread_id != request.thread_id
            || binding.worker_launch_document_sha256 != launch_hash
            || worker.role != EPIPHANY_IMAGINATION_OWNER_ROLE
            || worker.binding_id != EPIPHANY_IMAGINATION_ROLE_BINDING_ID
            || worker.imagination_consideration_request_id.as_deref() != Some(request_id)
            || projection.map(|projection| &projection.request) != Some(&request)
            || projection.map(|projection| projection.model.reasoning_basis())
                != Some(crate::EpiphanyRepoModelBasis {
                    projection_digest: request.model_projection_digest.clone(),
                    source_documents: request.model_source_documents.clone(),
                })
        {
            return Err(anyhow!(
                "consideration result substituted request, launch, or context"
            ));
        }
        imagination_candidate_companion = Some(candidate);
    }
    let has_mind_echo = result.frontier_plan_mind_request_id.is_some();
    let has_mind_decision = result.frontier_plan_mind_decision_msgpack.is_some();
    if has_mind_echo != has_mind_decision {
        return Err(anyhow!(
            "Mind result requires both exact request echo and typed decision"
        ));
    }
    if has_mind_echo {
        if !result.role_id.eq_ignore_ascii_case("mindAdmissionReview")
            || result.item_error.is_some()
            || result.state_patch_msgpack.is_some()
            || result.self_patch_msgpack.is_some()
            || result.repo_model_mutation_proposal_msgpack.is_some()
            || result.frontier_planning_request_id.is_some()
            || result.frontier_plan_candidate_msgpack.is_some()
            || result.verification_request_id.is_some()
            || result.frontier_route_id.is_some()
            || result.repo_frontier_modeling_request_id.is_some()
            || result.proposal_modeling_request_id.is_some()
            || result.claim_repair_request_id.is_some()
        {
            return Err(anyhow!(
                "Mind decision result carries foreign authority cargo"
            ));
        }
        let request_id = result.frontier_plan_mind_request_id.as_deref().unwrap();
        let request = cache
            .get::<RepoFrontierPlanMindRequest>(request_id)?
            .ok_or_else(|| anyhow!("Mind result request disappeared"))?;
        let (planning, candidate) = validate_repo_frontier_plan_mind_request(&cache, &request)?;
        let decision = result
            .frontier_plan_mind_decision()?
            .ok_or_else(|| anyhow!("Mind decision disappeared"))?;
        if decision.mind_request_id != request.request_id
            || decision.planning_request_id != planning.request_id
            || decision.imagination_result_id != request.imagination_result_id
            || decision.candidate_id != candidate.candidate_id
            || decision.candidate_sha256 != request.candidate_sha256
            || decision.rationale.trim().is_empty()
            || chrono::DateTime::parse_from_rfc3339(&decision.decided_at).is_err()
        {
            return Err(anyhow!(
                "Mind decision substituted request echo or immutable candidate identity"
            ));
        }
        let bindings = cache
            .get_all::<RepoFrontierPlanMindLaunchBinding>()?
            .into_iter()
            .filter(|binding| {
                binding.mind_request_id == request.request_id && binding.job_id == result.job_id
            })
            .collect::<Vec<_>>();
        if bindings.len() != 1 {
            return Err(anyhow!(
                "Mind result requires exactly one coordinator launch binding"
            ));
        }
        let binding = &bindings[0];
        let launch = cache
            .get::<EpiphanyRuntimeWorkerLaunchRequest>(&result.job_id)?
            .ok_or_else(|| anyhow!("Mind result launch disappeared"))?;
        let document = launch.launch_document()?;
        let projection = match &document {
            EpiphanyWorkerLaunchDocument::Role(d) => d.frontier_plan_mind_context.as_ref(),
            _ => None,
        };
        let expected = RepoFrontierPlanMindContextProjection::new(&request, &planning, &candidate);
        let hash = format!("{:x}", Sha256::digest(&launch.launch_document_msgpack));
        if binding.schema_version != REPO_FRONTIER_PLAN_MIND_LAUNCH_BINDING_SCHEMA_VERSION
            || binding.contract != REPO_FRONTIER_PLAN_MIND_LAUNCH_BINDING_CONTRACT
            || binding.job_id != result.job_id
            || binding.binding_id != EPIPHANY_MIND_ROLE_BINDING_ID
            || binding.runtime_id != request.runtime_id
            || binding.thread_id != request.thread_id
            || binding.worker_launch_document_sha256 != hash
            || launch.role != EPIPHANY_MIND_OWNER_ROLE
            || launch.binding_id != EPIPHANY_MIND_ROLE_BINDING_ID
            || launch.frontier_plan_mind_request_id.as_deref() != Some(request.request_id.as_str())
            || projection != Some(&expected)
        {
            return Err(anyhow!(
                "Mind result does not exactly bind request, launch, runtime, thread, and candidate"
            ));
        }
    }
    let process_claim_id = worker_process_claim_id(&result.job_id);
    let process_claim = cache.get::<EpiphanyRuntimeWorkerProcessClaim>(&process_claim_id)?;
    if let Some(existing) = cache.get::<EpiphanyRuntimeRoleWorkerResult>(&result.job_id)? {
        if existing != *result {
            return Err(anyhow!(
                "role worker result is immutable for its runtime job"
            ));
        }
        if let Some(claim) = process_claim.as_ref()
            && (!crate::WorkerProcessStatus::parse(&claim.status)?.is_fulfilled_terminal()
                || claim.terminal_authority_id.as_deref() != Some(result.result_id.as_str()))
        {
            return Err(anyhow!(
                "role worker result is not terminal authority for its process claim"
            ));
        }
        if let Some(companion) = model_direction_companion.as_ref() {
            match cache
                .get::<crate::AdmittedModelDirectionConsiderationResult>(&companion.result_id)?
            {
                Some(existing) if existing == *companion => {}
                _ => {
                    return Err(anyhow!(
                        "model direction worker result lost its exact typed companion"
                    ));
                }
            }
        }
        if let Some(companion) = imagination_candidate_companion.as_ref() {
            match cache.get::<crate::ImaginationConsiderationCandidate>(&companion.candidate_id)? {
                Some(existing) if existing == *companion => {}
                _ => {
                    return Err(anyhow!(
                        "Imagination worker result lost its exact typed candidate companion"
                    ));
                }
            }
        }
        return Ok(());
    }
    let (envelope, _) = cache.prepare_entry(&result.job_id, result)?;
    let mut writes = vec![envelope];
    let mut expected = Vec::new();
    if let Some(claim) = process_claim.as_ref() {
        if crate::WorkerProcessStatus::parse(&claim.status)? != crate::WorkerProcessStatus::Active {
            return Err(anyhow!(
                "role worker result requires its active process claim"
            ));
        }
        let current_envelope = cache
            .get_envelope::<EpiphanyRuntimeWorkerProcessClaim>(&process_claim_id)?
            .ok_or_else(|| anyhow!("role worker result lost its process claim envelope"))?;
        let mut terminal_claim = claim.clone();
        terminal_claim.status = crate::WorkerProcessStatus::TerminalResult.as_str().into();
        terminal_claim.terminal_at = Some(chrono::Utc::now().to_rfc3339());
        terminal_claim.terminal_authority_id = Some(result.result_id.clone());
        expected.push(current_envelope);
        writes.push(cache.prepare_entry(&process_claim_id, &terminal_claim)?.0);
    }
    if let Some(companion) = model_direction_companion.as_ref() {
        if cache
            .get::<crate::AdmittedModelDirectionConsiderationResult>(&companion.result_id)?
            .is_some()
        {
            return Err(anyhow!(
                "model direction result companion identity already exists without its worker result"
            ));
        }
        let (companion_envelope, _) = cache.prepare_entry(&companion.result_id, companion)?;
        writes.push(companion_envelope);
    }
    if let Some(companion) = imagination_candidate_companion.as_ref() {
        if cache
            .get::<crate::ImaginationConsiderationCandidate>(&companion.candidate_id)?
            .is_some()
        {
            return Err(anyhow!(
                "Imagination candidate identity already exists without its worker result"
            ));
        }
        let (companion_envelope, _) = cache.prepare_entry(&companion.candidate_id, companion)?;
        writes.push(companion_envelope);
    }
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    if backing.compare_and_swap_batch(&expected, writes)? {
        return Ok(());
    }
    let mut reloaded = runtime_spine_cache(store_path)?;
    reloaded.pull_all_backing_stores()?;
    let worker_matches = reloaded
        .get::<EpiphanyRuntimeRoleWorkerResult>(&result.job_id)?
        .is_some_and(|existing| existing == *result);
    let model_direction_companion_matches = match model_direction_companion.as_ref() {
        Some(companion) => reloaded
            .get::<crate::AdmittedModelDirectionConsiderationResult>(&companion.result_id)?
            .is_some_and(|existing| existing == *companion),
        None => true,
    };
    let imagination_candidate_companion_matches = match imagination_candidate_companion.as_ref() {
        Some(companion) => reloaded
            .get::<crate::ImaginationConsiderationCandidate>(&companion.candidate_id)?
            .is_some_and(|existing| existing == *companion),
        None => true,
    };
    let claim_matches = match process_claim.as_ref() {
        Some(_) => reloaded
            .get::<EpiphanyRuntimeWorkerProcessClaim>(&process_claim_id)?
            .is_some_and(|claim| {
                crate::WorkerProcessStatus::parse(&claim.status)
                    .is_ok_and(|status| status.is_fulfilled_terminal())
                    && claim.terminal_authority_id.as_deref() == Some(result.result_id.as_str())
            }),
        None => true,
    };
    if worker_matches
        && model_direction_companion_matches
        && imagination_candidate_companion_matches
        && claim_matches
    {
        Ok(())
    } else {
        Err(anyhow!(
            "role worker result lost immutable insertion to a different result or companion"
        ))
    }
}

pub fn runtime_role_worker_result(
    store_path: impl AsRef<Path>,
    job_id: &str,
) -> Result<Option<EpiphanyRuntimeRoleWorkerResult>> {
    validate_non_empty(job_id, "role worker result job id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<EpiphanyRuntimeRoleWorkerResult>(job_id)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeTypedFulfillmentEvidence {
    pub job_id: String,
    pub result_id: String,
    pub request_id: String,
}

pub(crate) fn validate_proposal_modeling_worker_fulfillment(
    cache: &CultCache,
    result: &EpiphanyRuntimeRoleWorkerResult,
) -> Result<()> {
    let request_id = result
        .proposal_modeling_request_id
        .as_deref()
        .ok_or_else(|| anyhow!("proposal Modeling fulfillment lost request echo"))?;
    if !result.role_id.eq_ignore_ascii_case("modeling") || result.item_error.is_some() {
        return Err(anyhow!(
            "proposal Modeling fulfillment requires a successful Modeling result"
        ));
    }
    let mutation_proposal = result
        .repo_model_mutation_proposal()?
        .ok_or_else(|| anyhow!("proposal Modeling fulfillment lost RepoModel mutation proposal"))?;
    mutation_proposal.validate()?;
    let mutation_operations = mutation_proposal.operations()?;
    let request = cache
        .get::<RepoFrontierProposalModelingRequest>(request_id)?
        .ok_or_else(|| anyhow!("proposal Modeling fulfillment request is missing"))?;
    validate_repo_frontier_proposal_modeling_request(&request)?;
    let proposal = cache
        .get::<RepoFrontierWorkProposal>(&request.proposal_id)?
        .ok_or_else(|| anyhow!("proposal Modeling fulfillment proposal is missing"))?;
    validate_repo_frontier_work_proposal(&proposal)?;
    if proposal.source_kind == crate::RepoFrontierProposalSourceKind::Imagination {
        validate_autonomous_proposal_origin_binding(cache, &proposal)?;
    }
    let bindings = cache
        .get_all::<RepoFrontierProposalModelingLaunchBinding>()?
        .into_iter()
        .filter(|binding| binding.job_id == result.job_id)
        .collect::<Vec<_>>();
    if bindings.len() != 1 {
        return Err(anyhow!(
            "proposal Modeling fulfillment requires exactly one launch binding"
        ));
    }
    let binding = &bindings[0];
    let launch = cache
        .get::<EpiphanyRuntimeWorkerLaunchRequest>(&result.job_id)?
        .ok_or_else(|| anyhow!("proposal Modeling fulfillment worker launch is missing"))?;
    let document = launch.launch_document()?;
    let projection = match &document {
        EpiphanyWorkerLaunchDocument::Role(document) => document.proposal_modeling_context.as_ref(),
        EpiphanyWorkerLaunchDocument::Reorient(_) => None,
    };
    let identity = require_identity(cache)?;
    let proposal_payload_sha256 = crate::repo_frontier_proposal_payload_sha256(
        &proposal.title,
        &proposal.body,
        &proposal.desired_outcome,
        &proposal.constraints,
        &proposal.scope_hints,
        &proposal.evidence_refs,
        &proposal.public_source_refs,
    )?;
    let projection_matches_authenticated_request = projection.is_some_and(|projection| {
        let expected = crate::RepoFrontierProposalModelingContextProjection {
            schema_version: crate::REPO_FRONTIER_PROPOSAL_MODELING_CONTEXT_SCHEMA_VERSION.into(),
            contract: crate::REPO_FRONTIER_PROPOSAL_MODELING_CONTEXT_CONTRACT.into(),
            request_id: request.request_id.clone(),
            proposal_id: proposal.proposal_id.clone(),
            proposal_payload_sha256: proposal.payload_sha256.clone(),
            runtime_id: request.runtime_id.clone(),
            thread_id: request.thread_id.clone(),
            repository: request.repository.clone(),
            workspace: request.workspace.clone(),
            source_kind: proposal.source_kind,
            source_actor: proposal.source_actor.clone(),
            source_ref: proposal.source_ref.clone(),
            title: proposal.title.clone(),
            body: proposal.body.clone(),
            desired_outcome: proposal.desired_outcome.clone(),
            constraints: proposal.constraints.clone(),
            scope_hints: proposal.scope_hints.clone(),
            evidence_refs: proposal.evidence_refs.clone(),
            public_source_refs: proposal.public_source_refs.clone(),
            private_state_included: proposal.private_state_included,
            model_projection_digest: projection.model_projection_digest.clone(),
            model_source_documents: projection.model_source_documents.clone(),
        };
        projection == &expected
    });
    let launch_sha256 = format!("{:x}", Sha256::digest(&launch.launch_document_msgpack));
    let mismatches = [
        (
            "request.payload",
            request.proposal_payload_sha256 != proposal.payload_sha256,
        ),
        (
            "request.identity",
            request.request_id
                != crate::proposal_modeling_request_id(
                    &request.runtime_id,
                    &request.proposal_id,
                    &request.proposal_payload_sha256,
                ),
        ),
        (
            "proposal.payload",
            proposal.payload_sha256 != proposal_payload_sha256,
        ),
        ("request.runtime", request.runtime_id != identity.runtime_id),
        ("request.thread", request.thread_id != proposal.thread_id),
        (
            "request.repository",
            request.repository != proposal.repository,
        ),
        ("request.workspace", request.workspace != proposal.workspace),
        (
            "binding.id",
            binding.binding_record_id
                != format!("repo-frontier-proposal-modeling-launch-{}", result.job_id),
        ),
        (
            "binding.schema",
            binding.schema_version != REPO_FRONTIER_PROPOSAL_MODELING_LAUNCH_BINDING_SCHEMA_VERSION,
        ),
        (
            "binding.contract",
            binding.contract != REPO_FRONTIER_PROPOSAL_MODELING_LAUNCH_BINDING_CONTRACT,
        ),
        (
            "binding.request",
            binding.proposal_modeling_request_id != request.request_id,
        ),
        (
            "binding.proposal",
            binding.proposal_id != proposal.proposal_id,
        ),
        (
            "binding.payload",
            binding.proposal_payload_sha256 != proposal.payload_sha256,
        ),
        ("binding.job", binding.job_id != result.job_id),
        (
            "binding.role",
            binding.binding_id != EPIPHANY_MODELING_ROLE_BINDING_ID,
        ),
        ("binding.runtime", binding.runtime_id != identity.runtime_id),
        ("binding.thread", binding.thread_id != request.thread_id),
        (
            "binding.time",
            chrono::DateTime::parse_from_rfc3339(&binding.launched_at).is_err(),
        ),
        (
            "binding.hash",
            binding.worker_launch_document_sha256 != launch_sha256,
        ),
        (
            "launch.schema",
            launch.schema_version != RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION,
        ),
        ("launch.job", launch.job_id != result.job_id),
        ("launch.binding", launch.binding_id != binding.binding_id),
        ("launch.role", launch.role != EPIPHANY_MODELING_OWNER_ROLE),
        (
            "launch.request",
            launch.proposal_modeling_request_id.as_deref() != Some(request.request_id.as_str()),
        ),
        (
            "launch.projection",
            !projection_matches_authenticated_request,
        ),
    ]
    .into_iter()
    .filter_map(|(name, failed)| failed.then_some(name))
    .collect::<Vec<_>>();
    if !mismatches.is_empty() {
        return Err(anyhow!(
            "proposal Modeling fulfillment provenance mismatch: {}",
            mismatches.join(", ")
        ));
    }
    let upserts = mutation_operations
        .iter()
        .filter_map(|operation| match operation {
            crate::EpiphanyRepoModelMutationOperation::PutFrontier { item } => Some(item),
            _ => None,
        })
        .collect::<Vec<_>>();
    let frontier_operations = mutation_operations
        .iter()
        .filter(|operation| {
            matches!(
                operation,
                crate::EpiphanyRepoModelMutationOperation::PutFrontier { .. }
            )
        })
        .count();
    if !result
        .evidence_ids
        .iter()
        .any(|id| id == &proposal.proposal_id)
        || frontier_operations != 1
        || upserts.len() != 1
        || !upserts[0]
            .evidence_refs
            .iter()
            .any(|id| id == &proposal.proposal_id)
        || upserts[0].source_scope.is_empty()
        || !safe_sorted_unique_paths(&upserts[0].source_scope)
        || upserts[0].public_source_refs
            != if upserts[0].recommended_next_organ == "Eyes" {
                proposal.public_source_refs.clone()
            } else {
                Vec::new()
            }
        || upserts[0].status != crate::RepoFrontierStatus::Active
        || !matches!(
            upserts[0].recommended_next_organ.as_str(),
            "Hands" | "Eyes" | "Imagination"
        )
        || (proposal.source_kind == crate::RepoFrontierProposalSourceKind::Imagination
            && (upserts[0].recommended_next_organ == "Hands" || upserts[0].adopted_plan.is_some()))
    {
        return Err(anyhow!(
            "proposal Modeling fulfillment result is not one safe proposal-citing routeable frontier"
        ));
    }
    Ok(())
}

pub fn runtime_typed_request_fulfillment(
    store_path: impl AsRef<Path>,
    request: RuntimeTypedRequestRef<'_>,
) -> Result<Option<RuntimeTypedFulfillmentEvidence>> {
    let store_path = store_path.as_ref();
    let request_id = request.request_id();
    validate_non_empty(request_id, "typed fulfillment request id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let archived_matches = cache
        .get_all::<EpiphanyArchivedRuntimeWorkerAttempt>()?
        .into_iter()
        .filter(|attempt| attempt.request_id == request_id && attempt.result_id.is_some())
        .collect::<Vec<_>>();
    if archived_matches.len() > 1 {
        return Err(anyhow!(
            "typed fulfillment request has multiple archived terminal claimants"
        ));
    }
    if let Some(attempt) = archived_matches.first() {
        if attempt.schema_version != ARCHIVED_RUNTIME_WORKER_ATTEMPT_SCHEMA_VERSION
            || attempt.archive_id != attempt.job_id
            || attempt.request_kind != request.kind()
            || !crate::WorkerProcessStatus::parse(&attempt.terminal_process_status)?
                .is_fulfilled_terminal()
            || !attempt.retired_chain_digest.starts_with("sha256:")
        {
            return Err(anyhow!("archived typed fulfillment tombstone is invalid"));
        }
        if cache
            .get::<EpiphanyRuntimeWorkerLaunchRequest>(&attempt.job_id)?
            .is_some()
            || cache
                .get::<EpiphanyRuntimeWorkerProcessClaim>(&worker_process_claim_id(
                    &attempt.job_id,
                ))?
                .is_some()
            || cache
                .get::<EpiphanyRuntimeRoleWorkerResult>(&attempt.job_id)?
                .is_some()
        {
            return Err(anyhow!(
                "archived typed fulfillment retained live attempt authority"
            ));
        }
        return Ok(Some(RuntimeTypedFulfillmentEvidence {
            job_id: attempt.job_id.clone(),
            result_id: attempt
                .result_id
                .clone()
                .expect("validated archived result"),
            request_id: request_id.to_string(),
        }));
    }
    let matches = cache
        .get_all::<EpiphanyRuntimeRoleWorkerResult>()?
        .into_iter()
        .filter(|result| request.matches_result(result))
        .collect::<Vec<_>>();
    if matches.is_empty() {
        return Ok(None);
    }
    if matches.len() != 1 {
        return Err(anyhow!(
            "typed fulfillment request has multiple terminal worker claimants"
        ));
    }
    let result = &matches[0];
    if result.item_error.is_some() {
        return Err(anyhow!(
            "typed fulfillment claimant terminated with an item error"
        ));
    }
    // This is the runtime admission owner. Replaying an already persisted
    // immutable result performs the same full validation without writing.
    put_runtime_role_worker_result(store_path, result)?;
    match request {
        RuntimeTypedRequestRef::ProposalModeling(_) => {
            validate_proposal_modeling_worker_fulfillment(&cache, result)?;
        }
        RuntimeTypedRequestRef::ImaginationConsideration(_) => {
            let candidate = result
                .imagination_consideration_candidate()?
                .ok_or_else(|| anyhow!("Imagination fulfillment lost typed candidate"))?;
            if cache
                .get::<crate::ImaginationConsiderationCandidate>(&candidate.candidate_id)?
                .as_ref()
                != Some(&candidate)
            {
                return Err(anyhow!(
                    "Imagination fulfillment lost its persisted typed candidate"
                ));
            }
        }
        RuntimeTypedRequestRef::AdmittedModelDirection(_) => {
            let direction = result
                .admitted_model_direction_consideration_result()?
                .ok_or_else(|| anyhow!("model direction fulfillment lost typed result"))?;
            if cache
                .get::<crate::AdmittedModelDirectionConsiderationResult>(&direction.result_id)?
                .as_ref()
                != Some(&direction)
            {
                return Err(anyhow!(
                    "model direction fulfillment lost its persisted typed result"
                ));
            }
        }
    }
    Ok(Some(RuntimeTypedFulfillmentEvidence {
        job_id: result.job_id.clone(),
        result_id: result.result_id.clone(),
        request_id: request_id.to_string(),
    }))
}

fn put_immutable_planning_entry<T: cultcache_rs::DatabaseEntry + PartialEq + Clone>(
    store_path: &Path,
    key: &str,
    value: &T,
) -> Result<()> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    if let Some(existing) = cache.get::<T>(key)? {
        return if existing == *value {
            Ok(())
        } else {
            Err(anyhow!("planning document ids are immutable"))
        };
    }
    let (envelope, _) = cache.prepare_entry(key, value)?;
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    if backing.compare_and_swap_batch(&[], vec![envelope])? {
        return Ok(());
    }
    let mut reloaded = runtime_spine_cache(store_path)?;
    reloaded.pull_all_backing_stores()?;
    match reloaded.get::<T>(key)? {
        Some(existing) if existing == *value => Ok(()),
        _ => Err(anyhow!("planning document CAS collision")),
    }
}

pub(crate) fn validate_repo_frontier_work_proposal(
    proposal: &RepoFrontierWorkProposal,
) -> Result<()> {
    if proposal.schema_version != REPO_FRONTIER_WORK_PROPOSAL_SCHEMA_VERSION
        || proposal.contract != REPO_FRONTIER_WORK_PROPOSAL_CONTRACT
        || proposal.proposal_id.trim().is_empty()
        || proposal.source_actor.trim().is_empty()
        || proposal.source_ref.trim().is_empty()
        || proposal.repository.trim().is_empty()
        || proposal.workspace.trim().is_empty()
        || proposal.thread_id.trim().is_empty()
        || proposal.runtime_id.trim().is_empty()
        || proposal.title.trim().is_empty()
        || proposal.body.trim().is_empty()
        || proposal.desired_outcome.trim().is_empty()
        || proposal.private_state_included
        || chrono::DateTime::parse_from_rfc3339(&proposal.proposed_at).is_err()
    {
        return Err(anyhow!("invalid inert repo frontier work proposal"));
    }
    let canonical_public_sources = crate::ImmutableGithubSource::canonicalize_set(
        proposal.public_source_refs.iter().map(String::as_str),
    )?;
    if canonical_public_sources != proposal.public_source_refs {
        return Err(anyhow!("proposal public source set is not canonical"));
    }
    let expected_payload_sha256 = crate::repo_frontier_proposal_payload_sha256(
        &proposal.title,
        &proposal.body,
        &proposal.desired_outcome,
        &proposal.constraints,
        &proposal.scope_hints,
        &proposal.evidence_refs,
        &proposal.public_source_refs,
    )?;
    if proposal.payload_sha256 != expected_payload_sha256 {
        return Err(anyhow!("proposal content hash mismatch"));
    }
    Ok(())
}

fn validate_autonomous_proposal_origin_binding(
    cache: &CultCache,
    proposal: &RepoFrontierWorkProposal,
) -> Result<RepoFrontierAutonomousProposalBinding> {
    if proposal.source_kind != crate::RepoFrontierProposalSourceKind::Imagination {
        return Err(anyhow!(
            "autonomous binding requires an Imagination proposal"
        ));
    }
    let binding_id = format!("autonomous-proposal-binding-{}", proposal.proposal_id);
    let binding = cache
        .get::<RepoFrontierAutonomousProposalBinding>(&binding_id)?
        .ok_or_else(|| anyhow!("Imagination proposal lacks its autonomous origin binding"))?;
    let request = cache
        .get::<crate::AdmittedModelDirectionConsiderationRequest>(&binding.direction_request_id)?
        .ok_or_else(|| anyhow!("autonomous proposal binding lost its direction request"))?;
    let result = cache
        .get::<crate::AdmittedModelDirectionConsiderationResult>(&binding.direction_result_id)?
        .ok_or_else(|| anyhow!("autonomous proposal binding lost its direction result"))?;
    let worker_result = cache
        .get::<EpiphanyRuntimeRoleWorkerResult>(&binding.direction_worker_job_id)?
        .ok_or_else(|| anyhow!("autonomous proposal binding lost its Imagination worker result"))?;
    let worker_launch = cache
        .get::<EpiphanyRuntimeWorkerLaunchRequest>(&binding.direction_worker_job_id)?
        .ok_or_else(|| anyhow!("autonomous proposal binding lost its Imagination worker launch"))?;
    crate::validate_current_admitted_model_direction_consideration_request(cache, &request)?;
    crate::validate_admitted_model_direction_consideration_result(&request, &result)?;
    let result_sha256 = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&result)?));
    let worker_result_sha256 = format!(
        "{:x}",
        Sha256::digest(rmp_serde::to_vec_named(&worker_result)?)
    );
    let worker_launch_sha256 = format!(
        "{:x}",
        Sha256::digest(rmp_serde::to_vec_named(&worker_launch)?)
    );
    let worker_direction = worker_result
        .admitted_model_direction_consideration_result()?
        .ok_or_else(|| anyhow!("autonomous proposal worker result lost its direction cargo"))?;
    let launch_projection = match worker_launch.launch_document()? {
        EpiphanyWorkerLaunchDocument::Role(document) => {
            document.admitted_model_direction_consideration_context
        }
        EpiphanyWorkerLaunchDocument::Reorient(_) => None,
    };
    let option = result
        .option_drafts
        .get(binding.option_ordinal as usize)
        .ok_or_else(|| anyhow!("autonomous proposal binding names a missing option"))?;
    let option_sha256 = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(option)?));
    let route = cache
        .get::<crate::RuntimeRepositoryBodyStoreBinding>(crate::RUNTIME_BODY_STORE_BINDING_KEY)?
        .ok_or_else(|| anyhow!("autonomous proposal requires repository Body binding"))?;
    let domain = cache
        .get::<RuntimeRepositoryDomainBinding>(RUNTIME_REPOSITORY_DOMAIN_BINDING_KEY)?
        .ok_or_else(|| anyhow!("autonomous proposal requires repository domain binding"))?;
    let chain_checks = [
        (
            "worker result identity",
            binding.direction_worker_result_id == worker_result.result_id,
        ),
        (
            "worker result hash",
            binding.direction_worker_result_sha256 == worker_result_sha256,
        ),
        (
            "worker launch hash",
            binding.direction_worker_launch_sha256 == worker_launch_sha256,
        ),
        (
            "worker job",
            worker_result.job_id == binding.direction_worker_job_id,
        ),
        (
            "worker role",
            worker_result.role_id.eq_ignore_ascii_case("imagination"),
        ),
        (
            "worker request echo",
            worker_result
                .admitted_model_direction_consideration_request_id
                .as_deref()
                == Some(request.request_id.as_str()),
        ),
        ("worker direction cargo", worker_direction == result),
        (
            "direction launch identity",
            result.result_id
                == crate::admitted_model_direction_consideration_result_id_for_launch(
                    &request.request_id,
                    &binding.direction_worker_job_id,
                ),
        ),
        (
            "launch role",
            worker_launch
                .role
                .eq_ignore_ascii_case(EPIPHANY_IMAGINATION_OWNER_ROLE),
        ),
        (
            "launch binding",
            worker_launch.binding_id == EPIPHANY_IMAGINATION_ROLE_BINDING_ID,
        ),
        (
            "launch request echo",
            worker_launch
                .admitted_model_direction_consideration_request_id
                .as_deref()
                == Some(request.request_id.as_str()),
        ),
        (
            "launch request projection",
            launch_projection.map(|projection| projection.request) == Some(request.clone()),
        ),
    ];
    if let Some((failed, _)) = chain_checks.into_iter().find(|(_, matches)| !matches) {
        return Err(anyhow!(
            "autonomous proposal Imagination chain mismatch: {failed}"
        ));
    }
    if binding.schema_version != REPO_FRONTIER_AUTONOMOUS_PROPOSAL_BINDING_SCHEMA_VERSION
        || binding.contract != REPO_FRONTIER_AUTONOMOUS_PROPOSAL_BINDING_CONTRACT
        || binding.binding_id != binding_id
        || binding.proposal_id != proposal.proposal_id
        || binding.proposal_payload_sha256 != proposal.payload_sha256
        || binding.direction_request_id != result.request_id
        || binding.direction_result_id != result.result_id
        || binding.direction_result_sha256 != result_sha256
        || binding.model_projection_digest != result.model_projection_digest
        || binding.model_source_documents != result.model_source_documents
        || binding.option_sha256 != option_sha256
        || binding.runtime_id != proposal.runtime_id
        || binding.thread_id != proposal.thread_id
        || binding.workspace_id != route.workspace_id
        || binding.body_binding_sha256 != route.body_binding_sha256
        || domain.schema_version != RUNTIME_REPOSITORY_DOMAIN_BINDING_SCHEMA_VERSION
        || domain.contract != RUNTIME_REPOSITORY_DOMAIN_BINDING_CONTRACT
        || domain.binding_id != RUNTIME_REPOSITORY_DOMAIN_BINDING_KEY
        || domain.repository_full_name != proposal.repository
        || domain.runtime_id != route.runtime_id
        || domain.swarm_id != route.swarm_id
        || domain.workspace_id != route.workspace_id
        || domain.body_binding_sha256 != route.body_binding_sha256
        || chrono::DateTime::parse_from_rfc3339(&binding.created_at).is_err()
        || proposal.source_actor != EPIPHANY_IMAGINATION_OWNER_ROLE
        || proposal.source_ref != result.result_id
        || proposal.title != option.title
        || proposal.body != option.summary
    {
        return Err(anyhow!("autonomous proposal origin binding mismatch"));
    }
    Ok(binding)
}

pub(crate) fn validate_autonomous_proposal_binding(
    cache: &CultCache,
    proposal: &RepoFrontierWorkProposal,
) -> Result<RepoFrontierAutonomousProposalBinding> {
    let binding = validate_autonomous_proposal_origin_binding(cache, proposal)?;
    let route = cache
        .get::<crate::RuntimeRepositoryBodyStoreBinding>(crate::RUNTIME_BODY_STORE_BINDING_KEY)?
        .ok_or_else(|| anyhow!("autonomous proposal requires repository Body binding"))?;
    let (body_binding, _) = crate::load_repository_body_status(Path::new(&route.body_store_path))?
        .ok_or_else(|| anyhow!("autonomous proposal requires authenticated Body status"))?;
    if body_binding.runtime_id != route.runtime_id
        || body_binding.swarm_id != route.swarm_id
        || body_binding.workspace_id != route.workspace_id
        || crate::repository_body_observer::body_binding_sha256(&body_binding)?
            != route.body_binding_sha256
        || proposal.workspace != body_binding.git_top_level
    {
        return Err(anyhow!("autonomous proposal Body binding mismatch"));
    }
    Ok(binding)
}

pub(crate) fn validate_repo_frontier_proposal_modeling_request(
    request: &RepoFrontierProposalModelingRequest,
) -> Result<()> {
    if request.schema_version != REPO_FRONTIER_PROPOSAL_MODELING_REQUEST_SCHEMA_VERSION
        || request.contract != REPO_FRONTIER_PROPOSAL_MODELING_REQUEST_CONTRACT
        || request.request_id.trim().is_empty()
        || request.proposal_id.trim().is_empty()
        || request.proposal_payload_sha256.trim().is_empty()
        || request.runtime_id.trim().is_empty()
        || request.thread_id.trim().is_empty()
        || request.repository.trim().is_empty()
        || request.workspace.trim().is_empty()
        || chrono::DateTime::parse_from_rfc3339(&request.selected_at).is_err()
    {
        return Err(anyhow!(
            "invalid coordinator repo frontier proposal Modeling request"
        ));
    }
    Ok(())
}

pub fn put_repo_frontier_work_proposal(
    store_path: impl AsRef<Path>,
    proposal: &RepoFrontierWorkProposal,
) -> Result<()> {
    validate_repo_frontier_work_proposal(proposal)?;
    if proposal.source_kind == crate::RepoFrontierProposalSourceKind::Imagination {
        return Err(anyhow!(
            "generic proposal intake cannot author Imagination provenance"
        ));
    }
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    let identity = require_identity(&cache)?;
    if identity.runtime_id != proposal.runtime_id {
        return Err(anyhow!("proposal runtime identity mismatch"));
    }
    // The proposal's thread is immutable creation provenance. Runtime identity
    // owns intake; no mutable coordinator incarnation may admit or reject the
    // same semantic proposal.
    put_immutable_planning_entry(store_path.as_ref(), &proposal.proposal_id, proposal)
}

pub fn intake_user_repo_frontier_proposal(
    store_path: impl AsRef<Path>,
    input: crate::RepoFrontierUserProposalInput,
) -> Result<RepoFrontierWorkProposal> {
    let public_source_refs = crate::ImmutableGithubSource::canonicalize_set(
        input.public_source_refs.iter().map(String::as_str),
    )?;
    let payload_sha256 = crate::repo_frontier_proposal_payload_sha256(
        &input.title,
        &input.body,
        &input.desired_outcome,
        &input.constraints,
        &input.scope_hints,
        &input.evidence_refs,
        &public_source_refs,
    )?;
    let proposal = RepoFrontierWorkProposal {
        schema_version: REPO_FRONTIER_WORK_PROPOSAL_SCHEMA_VERSION.into(),
        proposal_id: input.proposal_id,
        source_kind: crate::RepoFrontierProposalSourceKind::User,
        source_actor: input.source_actor,
        source_ref: input.source_ref,
        repository: input.repository,
        workspace: input.workspace,
        thread_id: input.thread_id,
        runtime_id: input.runtime_id,
        payload_sha256,
        title: input.title,
        body: input.body,
        desired_outcome: input.desired_outcome,
        constraints: input.constraints,
        scope_hints: input.scope_hints,
        evidence_refs: input.evidence_refs,
        public_source_refs,
        private_state_included: input.private_state_included,
        proposed_at: input.proposed_at,
        contract: REPO_FRONTIER_WORK_PROPOSAL_CONTRACT.into(),
    };
    put_repo_frontier_work_proposal(store_path, &proposal)?;
    Ok(proposal)
}

pub fn runtime_repo_frontier_work_proposal(
    store_path: impl AsRef<Path>,
    proposal_id: &str,
) -> Result<Option<RepoFrontierWorkProposal>> {
    validate_non_empty(proposal_id, "repo frontier work proposal id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<RepoFrontierWorkProposal>(proposal_id)
}

pub fn bind_runtime_repository_domain(
    runtime_store: impl AsRef<Path>,
    repository_full_name: &str,
    bound_at: &str,
) -> Result<RuntimeRepositoryDomainBinding> {
    if !repository_full_name.starts_with("GameCult/")
        || repository_full_name["GameCult/".len()..].trim().is_empty()
        || repository_full_name.chars().any(char::is_whitespace)
    {
        return Err(anyhow!(
            "runtime repository domain requires a canonical GameCult name"
        ));
    }
    chrono::DateTime::parse_from_rfc3339(bound_at)
        .map_err(|_| anyhow!("runtime repository domain timestamp must be RFC3339"))?;
    let runtime_store = runtime_store.as_ref();
    let mut cache = runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    let identity = require_identity(&cache)?;
    let route = cache
        .get::<crate::RuntimeRepositoryBodyStoreBinding>(crate::RUNTIME_BODY_STORE_BINDING_KEY)?
        .ok_or_else(|| anyhow!("runtime repository domain requires Body binding"))?;
    let (body, _) = crate::load_repository_body_status(Path::new(&route.body_store_path))?
        .ok_or_else(|| anyhow!("runtime repository domain requires authenticated Body status"))?;
    if route.runtime_id != identity.runtime_id
        || body.runtime_id != route.runtime_id
        || body.swarm_id != route.swarm_id
        || body.workspace_id != route.workspace_id
        || crate::repository_body_observer::body_binding_sha256(&body)? != route.body_binding_sha256
    {
        return Err(anyhow!("runtime repository domain Body authority mismatch"));
    }
    let binding = RuntimeRepositoryDomainBinding {
        schema_version: RUNTIME_REPOSITORY_DOMAIN_BINDING_SCHEMA_VERSION.into(),
        binding_id: RUNTIME_REPOSITORY_DOMAIN_BINDING_KEY.into(),
        repository_full_name: repository_full_name.into(),
        runtime_id: route.runtime_id.clone(),
        swarm_id: route.swarm_id.clone(),
        workspace_id: route.workspace_id.clone(),
        body_binding_sha256: route.body_binding_sha256.clone(),
        bound_at: bound_at.into(),
        contract: RUNTIME_REPOSITORY_DOMAIN_BINDING_CONTRACT.into(),
    };
    if let Some(existing) =
        cache.get::<RuntimeRepositoryDomainBinding>(RUNTIME_REPOSITORY_DOMAIN_BINDING_KEY)?
    {
        let mut replay = binding;
        replay.bound_at = existing.bound_at.clone();
        return if replay == existing {
            Ok(existing)
        } else {
            Err(anyhow!("runtime repository domain is immutable"))
        };
    }
    let identity_envelope = cache
        .get_envelope::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("runtime repository domain identity disappeared"))?;
    let route_envelope = cache
        .get_envelope::<crate::RuntimeRepositoryBodyStoreBinding>(
            crate::RUNTIME_BODY_STORE_BINDING_KEY,
        )?
        .ok_or_else(|| anyhow!("runtime repository domain Body route disappeared"))?;
    let (binding_envelope, _) =
        cache.prepare_entry(RUNTIME_REPOSITORY_DOMAIN_BINDING_KEY, &binding)?;
    let expected = vec![identity_envelope, route_envelope];
    let mut replacement = expected.clone();
    replacement.push(binding_envelope);
    if !SingleFileMessagePackBackingStore::new(runtime_store)
        .compare_and_swap_batch(&expected, replacement)?
    {
        return Err(anyhow!("runtime repository domain lost atomic binding"));
    }
    Ok(binding)
}

pub fn promote_autonomous_direction_options_for_modeling(
    runtime_store: impl AsRef<Path>,
    repository: &str,
    workspace: &str,
    selected_at: &str,
) -> Result<Vec<RepoFrontierProposalModelingRequest>> {
    chrono::DateTime::parse_from_rfc3339(selected_at)
        .map_err(|_| anyhow!("autonomous proposal promotion timestamp must be RFC3339"))?;
    validate_non_empty(repository, "autonomous proposal repository")?;
    validate_non_empty(workspace, "autonomous proposal workspace")?;
    let runtime_store = runtime_store.as_ref();
    let mut opening = runtime_spine_cache(runtime_store)?;
    opening.pull_all_backing_stores()?;
    let identity = require_identity(&opening)?;
    if opening
        .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
        .is_none()
    {
        return Ok(Vec::new());
    }
    if opening
        .get::<crate::EpiphanyRepoModelIdentityDocument>(crate::REPO_MODEL_IDENTITY_KEY)?
        .is_none()
    {
        return Ok(Vec::new());
    }
    let model_basis = crate::repo_model_documents::assemble_repo_model_view_from_cache(&opening)?
        .reasoning_basis();
    let mut results = opening
        .get_all::<crate::AdmittedModelDirectionConsiderationResult>()?
        .into_iter()
        .filter(|result| {
            result.disposition == crate::AdmittedModelDirectionDisposition::Suggest
                && result.model_projection_digest == model_basis.projection_digest
                && result.model_source_documents == model_basis.source_documents
        })
        .collect::<Vec<_>>();
    if results.is_empty() {
        return Ok(Vec::new());
    }
    let route = opening
        .get::<crate::RuntimeRepositoryBodyStoreBinding>(crate::RUNTIME_BODY_STORE_BINDING_KEY)?
        .ok_or_else(|| anyhow!("autonomous proposal promotion requires Body binding"))?;
    if route.runtime_id != identity.runtime_id {
        return Err(anyhow!("autonomous proposal Body binding runtime mismatch"));
    }
    let domain = opening
        .get::<RuntimeRepositoryDomainBinding>(RUNTIME_REPOSITORY_DOMAIN_BINDING_KEY)?
        .ok_or_else(|| {
            anyhow!("autonomous proposal promotion requires repository domain binding")
        })?;
    let body_store = PathBuf::from(&route.body_store_path);
    let (body_binding, _) = crate::load_repository_body_status(&body_store)?
        .ok_or_else(|| anyhow!("autonomous proposal Body store has no authenticated status"))?;
    if body_binding.workspace_id != route.workspace_id
        || body_binding.runtime_id != route.runtime_id
        || body_binding.swarm_id != route.swarm_id
        || crate::repository_body_observer::body_binding_sha256(&body_binding)?
            != route.body_binding_sha256
        || Path::new(workspace).canonicalize()?
            != Path::new(&body_binding.git_top_level).canonicalize()?
        || domain.schema_version != RUNTIME_REPOSITORY_DOMAIN_BINDING_SCHEMA_VERSION
        || domain.contract != RUNTIME_REPOSITORY_DOMAIN_BINDING_CONTRACT
        || domain.binding_id != RUNTIME_REPOSITORY_DOMAIN_BINDING_KEY
        || domain.repository_full_name != repository
        || domain.runtime_id != route.runtime_id
        || domain.swarm_id != route.swarm_id
        || domain.workspace_id != route.workspace_id
        || domain.body_binding_sha256 != route.body_binding_sha256
    {
        return Err(anyhow!(
            "autonomous proposal workspace is not the bound repository Body"
        ));
    }
    results.sort_by(|left, right| left.result_id.cmp(&right.result_id));
    let mut promoted = Vec::new();
    for result in results {
        let request = opening
            .get::<crate::AdmittedModelDirectionConsiderationRequest>(&result.request_id)?
            .ok_or_else(|| anyhow!("autonomous proposal result lost its request"))?;
        crate::validate_admitted_model_direction_consideration_result(&request, &result)?;
        let mut direction_workers = opening
            .get_all::<EpiphanyRuntimeRoleWorkerResult>()?
            .into_iter()
            .filter_map(|worker| {
                worker
                    .admitted_model_direction_consideration_result()
                    .ok()
                    .flatten()
                    .filter(|embedded| embedded == &result)
                    .map(|_| worker)
            })
            .collect::<Vec<_>>();
        if direction_workers.len() != 1 {
            return Err(anyhow!(
                "autonomous proposal requires exactly one immutable Imagination worker result"
            ));
        }
        let direction_worker = direction_workers.remove(0);
        let direction_launch = opening
            .get::<EpiphanyRuntimeWorkerLaunchRequest>(&direction_worker.job_id)?
            .ok_or_else(|| anyhow!("autonomous proposal direction worker lost its launch"))?;
        let direction_worker_result_sha256 = format!(
            "{:x}",
            Sha256::digest(rmp_serde::to_vec_named(&direction_worker)?)
        );
        let direction_worker_launch_sha256 = format!(
            "{:x}",
            Sha256::digest(rmp_serde::to_vec_named(&direction_launch)?)
        );
        let result_sha256 = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&result)?));
        for (ordinal, option) in result.option_drafts.iter().enumerate() {
            if option.title.trim().is_empty() || option.summary.trim().is_empty() {
                return Err(anyhow!("autonomous direction option is empty"));
            }
            let option_sha256 = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(option)?));
            let proposal_id = format!(
                "repo-frontier-autonomous-{:x}",
                Sha256::digest(rmp_serde::to_vec_named(&(
                    &result.result_id,
                    ordinal as u32,
                    &option_sha256
                ))?)
            );
            let evidence_refs = result
                .evidence_refs
                .iter()
                .cloned()
                .chain([
                    format!(
                        "cultcache://admitted-model-direction/{}",
                        request.request_id
                    ),
                    format!(
                        "cultcache://admitted-model-direction-result/{}",
                        result.result_id
                    ),
                    format!(
                        "cultcache://repo-model/projection/{}",
                        result.model_projection_digest
                    ),
                ])
                .collect::<Vec<_>>();
            let payload_sha256 = crate::repo_frontier_proposal_payload_sha256(
                &option.title,
                &option.summary,
                &option.summary,
                &result.uncertainties,
                &[],
                &evidence_refs,
                &[],
            )?;
            let proposal = RepoFrontierWorkProposal {
                schema_version: REPO_FRONTIER_WORK_PROPOSAL_SCHEMA_VERSION.into(),
                proposal_id: proposal_id.clone(),
                source_kind: crate::RepoFrontierProposalSourceKind::Imagination,
                source_actor: EPIPHANY_IMAGINATION_OWNER_ROLE.into(),
                source_ref: result.result_id.clone(),
                repository: repository.into(),
                workspace: body_binding.git_top_level.clone(),
                thread_id: request.thread_id.clone(),
                runtime_id: identity.runtime_id.clone(),
                payload_sha256: payload_sha256.clone(),
                title: option.title.clone(),
                body: option.summary.clone(),
                desired_outcome: option.summary.clone(),
                constraints: result.uncertainties.clone(),
                scope_hints: Vec::new(),
                evidence_refs,
                public_source_refs: Vec::new(),
                private_state_included: false,
                proposed_at: result.proposed_at.clone(),
                contract: REPO_FRONTIER_WORK_PROPOSAL_CONTRACT.into(),
            };
            validate_repo_frontier_work_proposal(&proposal)?;
            let binding = RepoFrontierAutonomousProposalBinding {
                schema_version: REPO_FRONTIER_AUTONOMOUS_PROPOSAL_BINDING_SCHEMA_VERSION.into(),
                binding_id: format!("autonomous-proposal-binding-{proposal_id}"),
                proposal_id: proposal_id.clone(),
                proposal_payload_sha256: payload_sha256.clone(),
                direction_request_id: request.request_id.clone(),
                direction_result_id: result.result_id.clone(),
                direction_result_sha256: result_sha256.clone(),
                model_projection_digest: result.model_projection_digest.clone(),
                model_source_documents: result.model_source_documents.clone(),
                option_ordinal: ordinal as u32,
                option_sha256,
                runtime_id: identity.runtime_id.clone(),
                thread_id: request.thread_id.clone(),
                workspace_id: route.workspace_id.clone(),
                body_binding_sha256: route.body_binding_sha256.clone(),
                created_at: selected_at.into(),
                contract: REPO_FRONTIER_AUTONOMOUS_PROPOSAL_BINDING_CONTRACT.into(),
                direction_worker_job_id: direction_worker.job_id.clone(),
                direction_worker_result_id: direction_worker.result_id.clone(),
                direction_worker_result_sha256: direction_worker_result_sha256.clone(),
                direction_worker_launch_sha256: direction_worker_launch_sha256.clone(),
            };
            let selection = RepoFrontierProposalModelingRequest {
                schema_version: REPO_FRONTIER_PROPOSAL_MODELING_REQUEST_SCHEMA_VERSION.into(),
                request_id: crate::proposal_modeling_request_id(
                    &identity.runtime_id,
                    &proposal_id,
                    &payload_sha256,
                ),
                proposal_id: proposal_id.clone(),
                proposal_payload_sha256: payload_sha256,
                runtime_id: identity.runtime_id.clone(),
                thread_id: request.thread_id.clone(),
                repository: repository.into(),
                workspace: body_binding.git_top_level.clone(),
                selected_at: selected_at.into(),
                contract: REPO_FRONTIER_PROPOSAL_MODELING_REQUEST_CONTRACT.into(),
            };
            validate_repo_frontier_proposal_modeling_request(&selection)?;
            let mut current = runtime_spine_cache(runtime_store)?;
            current.pull_all_backing_stores()?;
            let existing = (
                current.get::<RepoFrontierWorkProposal>(&proposal.proposal_id)?,
                current.get::<RepoFrontierAutonomousProposalBinding>(&binding.binding_id)?,
                current.get::<RepoFrontierProposalModelingRequest>(&selection.request_id)?,
            );
            let replay_selection_matches = existing.2.as_ref().is_some_and(|existing_selection| {
                let mut replay_selection = selection.clone();
                replay_selection.selected_at = existing_selection.selected_at.clone();
                existing_selection == &replay_selection
            });
            if let (Some(existing_proposal), Some(_), Some(existing_selection)) = &existing {
                validate_autonomous_proposal_binding(&current, existing_proposal)?;
                if existing_proposal == &proposal && replay_selection_matches {
                    promoted.push(existing_selection.clone());
                    continue;
                }
            }
            if existing.0.is_some() || existing.1.is_some() || existing.2.is_some() {
                return Err(anyhow!(
                    "autonomous proposal promotion companion collision for {}: proposalPresent={} bindingPresent={} selectionPresent={} proposalMatches={} selectionMatches={} existingProposalThread={} expectedRequestThread={}",
                    proposal.proposal_id,
                    existing.0.is_some(),
                    existing.1.is_some(),
                    existing.2.is_some(),
                    existing.0.as_ref() == Some(&proposal),
                    replay_selection_matches,
                    existing
                        .0
                        .as_ref()
                        .map_or("missing", |value| value.thread_id.as_str()),
                    request.thread_id,
                ));
            }
            let (proposal_envelope, _) = current.prepare_entry(&proposal.proposal_id, &proposal)?;
            let (binding_envelope, _) = current.prepare_entry(&binding.binding_id, &binding)?;
            let (selection_envelope, _) =
                current.prepare_entry(&selection.request_id, &selection)?;
            let mut expected = vec![
                current
                    .get_envelope::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
                    .ok_or_else(|| anyhow!("runtime identity envelope disappeared"))?,
                current
                    .get_envelope::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
                    .ok_or_else(|| anyhow!("thread envelope disappeared"))?,
                current
                    .get_envelope::<crate::RuntimeRepositoryBodyStoreBinding>(
                        crate::RUNTIME_BODY_STORE_BINDING_KEY,
                    )?
                    .ok_or_else(|| anyhow!("Body route envelope disappeared"))?,
                current
                    .get_envelope::<RuntimeRepositoryDomainBinding>(
                        RUNTIME_REPOSITORY_DOMAIN_BINDING_KEY,
                    )?
                    .ok_or_else(|| anyhow!("repository domain envelope disappeared"))?,
                current
                    .get_envelope::<crate::AdmittedModelDirectionConsiderationRequest>(
                        &request.request_id,
                    )?
                    .ok_or_else(|| anyhow!("direction request envelope disappeared"))?,
                current
                    .get_envelope::<crate::AdmittedModelDirectionConsiderationResult>(
                        &result.result_id,
                    )?
                    .ok_or_else(|| anyhow!("direction result envelope disappeared"))?,
                current
                    .get_envelope::<EpiphanyRuntimeWorkerLaunchRequest>(&direction_worker.job_id)?
                    .ok_or_else(|| anyhow!("direction launch envelope disappeared"))?,
                current
                    .get_envelope::<EpiphanyRuntimeRoleWorkerResult>(&direction_worker.job_id)?
                    .ok_or_else(|| anyhow!("direction worker envelope disappeared"))?,
            ];
            for source in &result.model_source_documents {
                let envelope = current
                    .snapshot_envelopes()
                    .into_iter()
                    .find(|envelope| {
                        envelope.r#type == source.document_type
                            && envelope.key == source.document_key
                    })
                    .ok_or_else(|| anyhow!("keyed model source envelope disappeared"))?;
                if crate::EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", &envelope)?
                    != *source
                {
                    return Err(anyhow!("keyed model source changed before promotion"));
                }
                expected.push(envelope);
            }
            let mut replacement = expected.clone();
            replacement.extend([proposal_envelope, binding_envelope, selection_envelope]);
            if !SingleFileMessagePackBackingStore::new(runtime_store)
                .compare_and_swap_batch(&expected, replacement)?
            {
                return Err(anyhow!(
                    "autonomous proposal promotion lost atomic insertion"
                ));
            }
            promoted.push(selection);
        }
    }
    Ok(promoted)
}

pub fn runtime_repo_frontier_proposal_modeling_request(
    store_path: impl AsRef<Path>,
    request_id: &str,
) -> Result<Option<RepoFrontierProposalModelingRequest>> {
    validate_non_empty(request_id, "repo frontier proposal Modeling request id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<RepoFrontierProposalModelingRequest>(request_id)
}

pub fn runtime_current_repo_model(
    store_path: impl AsRef<Path>,
) -> Result<Option<crate::EpiphanyMemoryGraphSnapshot>> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    if cache
        .get::<crate::EpiphanyRepoModelIdentityDocument>(crate::REPO_MODEL_IDENTITY_KEY)?
        .is_none()
    {
        return Ok(None);
    }
    Ok(Some(
        crate::repo_model_documents::assemble_repo_model_view_from_cache(&cache)?
            .memory_context_projection(),
    ))
}

fn current_keyed_repo_model(
    cache: &CultCache,
) -> Result<(crate::EpiphanyRepoModelView, crate::EpiphanyRepoModelBasis)> {
    let view = crate::repo_model_documents::assemble_repo_model_view_from_cache(cache)?;
    let basis = view.reasoning_basis();
    basis.validate_against_cache(cache)?;
    Ok((view, basis))
}

fn require_keyed_repo_model_basis(
    cache: &CultCache,
    projection_digest: &str,
    source_documents: &[crate::EpiphanyMindDocumentVersion],
) -> Result<crate::EpiphanyRepoModelView> {
    crate::EpiphanyRepoModelBasis {
        projection_digest: projection_digest.to_string(),
        source_documents: source_documents.to_vec(),
    }
    .validate_against_cache(cache)?;
    crate::repo_model_documents::assemble_repo_model_view_from_cache(cache)
}

fn keyed_repo_model_basis_envelopes(
    cache: &CultCache,
    basis: &crate::EpiphanyRepoModelBasis,
) -> Result<Vec<CultCacheEnvelope>> {
    basis.validate_against_cache(cache)?;
    basis
        .source_documents
        .iter()
        .map(|source| {
            cache
                .snapshot_envelopes()
                .into_iter()
                .find(|envelope| {
                    envelope.r#type == source.document_type && envelope.key == source.document_key
                })
                .ok_or_else(|| anyhow!("RepoModel basis source disappeared"))
        })
        .collect()
}

fn keyed_repo_model_basis_after_writes(
    current: &crate::EpiphanyRepoModelBasis,
    writes: &[CultCacheEnvelope],
) -> Result<crate::EpiphanyRepoModelBasis> {
    let mut sources = current
        .source_documents
        .iter()
        .cloned()
        .map(|source| {
            (
                (source.document_type.clone(), source.document_key.clone()),
                source,
            )
        })
        .collect::<BTreeMap<_, _>>();
    for write in writes {
        if crate::repo_model_documents::repo_model_write_key(write)?.is_some() {
            let source = crate::EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", write)?;
            sources.insert(
                (source.document_type.clone(), source.document_key.clone()),
                source,
            );
        }
    }
    let source_documents = sources.into_values().collect::<Vec<_>>();
    let projection_digest = format!(
        "sha256:{:x}",
        Sha256::digest(rmp_serde::to_vec_named(&source_documents)?)
    );
    let basis = crate::EpiphanyRepoModelBasis {
        projection_digest,
        source_documents,
    };
    basis.validate()?;
    Ok(basis)
}

pub fn runtime_modeling_semantic_projection_input(
    store_path: impl AsRef<Path>,
) -> Result<crate::MemorySemanticProjectionInput> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let binding = require_runtime_swarm_binding(&cache)?;
    let view = crate::repo_model_documents::assemble_repo_model_view_from_cache(&cache)?;
    let basis = view.reasoning_basis();
    basis.validate_against_cache(&cache)?;
    // The semantic subsystem still consumes its historical projection DTO.  It
    // is now a derived carrier only: keyed Mind documents are the authority,
    // and the projection has no reusable global revision.
    let mut snapshot = view.memory_context_projection();
    snapshot.model_revision = 1;
    snapshot.model_hash = crate::memory_graph_model_hash(&snapshot)?;
    let model_hash = crate::memory_graph_model_hash(&snapshot)?;
    let canonical_source_id = format!("epiphany.runtime/{}/repo-model", binding.runtime_id);
    let matches = cache
        .get_all::<crate::MemorySemanticProjectionObligation>()?
        .into_iter()
        .filter(|obligation| {
            obligation.swarm_id == binding.swarm_id
                && obligation.partition == "modeling"
                && obligation.canonical_source_id == canonical_source_id
                && obligation.graph_id == snapshot.graph_id
                && obligation.source_commit_id == basis.projection_digest
                && obligation.source_generation == 1
                && obligation.source_model_hash == model_hash
        })
        .collect::<Vec<_>>();
    if matches.len() != 1 {
        return Err(anyhow!(
            "Modeling projection requires exactly one obligation for current RepoModel"
        ));
    }
    let obligation = matches.into_iter().next().expect("one obligation");
    let expected = crate::repo_model_documents::derive_repo_model_semantic_projection_obligation(
        &view,
        &obligation.created_at,
    )?;
    if obligation != expected {
        return Err(anyhow!(
            "Modeling projection obligation does not match canonical RepoModel"
        ));
    }
    let mut authority_envelopes = keyed_repo_model_basis_envelopes(&cache, &basis)?;
    authority_envelopes.push(
        cache
            .get_envelope::<EpiphanyRuntimeSwarmBinding>(RUNTIME_SWARM_BINDING_KEY)?
            .ok_or_else(|| anyhow!("Modeling projection lost its swarm binding"))?,
    );
    authority_envelopes
        .sort_by(|left, right| (&left.r#type, &left.key).cmp(&(&right.r#type, &right.key)));
    Ok(crate::MemorySemanticProjectionInput {
        snapshot,
        authority: crate::memory_graph::MemorySemanticProjectionAuthoritySnapshot {
            head: crate::MemorySemanticProjectionSourceHead {
                swarm_id: obligation.swarm_id.clone(),
                partition: obligation.partition.clone(),
                canonical_source_id: obligation.canonical_source_id.clone(),
                source_commit_id: obligation.source_commit_id.clone(),
                graph_id: obligation.graph_id.clone(),
                source_generation: obligation.source_generation,
                source_model_hash: obligation.source_model_hash.clone(),
                canonical_content_set_hash: obligation.canonical_content_set_hash.clone(),
            },
            envelopes: authority_envelopes,
        },
        obligation,
    })
}

pub fn select_repo_frontier_work_proposal_for_modeling(
    store_path: impl AsRef<Path>,
    proposal_id: &str,
    selected_at: &str,
) -> Result<RepoFrontierProposalModelingRequest> {
    chrono::DateTime::parse_from_rfc3339(selected_at)
        .map_err(|_| anyhow!("proposal selection timestamp must be RFC3339"))?;
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let identity = require_identity(&cache)?;
    let proposal = cache
        .get::<RepoFrontierWorkProposal>(proposal_id)?
        .ok_or_else(|| anyhow!("proposal selection requires exact persisted proposal"))?;
    validate_repo_frontier_work_proposal(&proposal)?;
    if proposal.source_kind == crate::RepoFrontierProposalSourceKind::Imagination {
        validate_autonomous_proposal_binding(&cache, &proposal)?;
    }
    if proposal.runtime_id != identity.runtime_id {
        return Err(anyhow!("proposal selection provenance mismatch"));
    }
    let existing_requests = cache.get_all::<RepoFrontierProposalModelingRequest>()?;
    let request_id = crate::proposal_modeling_request_id(
        &proposal.runtime_id,
        &proposal.proposal_id,
        &proposal.payload_sha256,
    );
    let request = RepoFrontierProposalModelingRequest {
        schema_version: REPO_FRONTIER_PROPOSAL_MODELING_REQUEST_SCHEMA_VERSION.into(),
        request_id: request_id.clone(),
        proposal_id: proposal.proposal_id.clone(),
        proposal_payload_sha256: proposal.payload_sha256.clone(),
        runtime_id: proposal.runtime_id.clone(),
        thread_id: proposal.thread_id.clone(),
        repository: proposal.repository.clone(),
        workspace: proposal.workspace.clone(),
        selected_at: selected_at.into(),
        contract: REPO_FRONTIER_PROPOSAL_MODELING_REQUEST_CONTRACT.into(),
    };
    if let Some(existing) = existing_requests
        .into_iter()
        .find(|r| r.proposal_id == proposal_id)
    {
        validate_repo_frontier_proposal_modeling_request(&existing)?;
        return if existing == request {
            Ok(existing)
        } else {
            Err(anyhow!("proposal selection identity conflict"))
        };
    }
    match put_immutable_planning_entry(store_path, &request_id, &request) {
        Ok(()) => Ok(request),
        Err(error) => {
            let mut reloaded = runtime_spine_cache(store_path)?;
            reloaded.pull_all_backing_stores()?;
            if let Some(existing) =
                reloaded.get::<RepoFrontierProposalModelingRequest>(&request_id)?
            {
                if existing == request {
                    return Ok(existing);
                }
            }
            Err(error)
        }
    }
}

pub fn select_and_commit_repo_frontier_planning_request(
    runtime_store: impl AsRef<Path>,
    at: &str,
) -> Result<RepoFrontierPlanningRequest> {
    chrono::DateTime::parse_from_rfc3339(at)
        .map_err(|_| anyhow!("planning request timestamp must be RFC3339"))?;
    let runtime_store = runtime_store.as_ref();
    let mut cache = runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    let identity = require_identity(&cache)?;
    let thread = cache
        .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
        .ok_or_else(|| anyhow!("planning request requires authoritative thread state"))?;
    thread.state()?;
    let backing = SingleFileMessagePackBackingStore::new(runtime_store);
    let (view, basis) = current_keyed_repo_model(&cache)?;
    let model = view.memory_context_projection();
    let challenges = current_repo_model_claim_challenges(&cache, &model, &basis)?;
    let item = actionable_imagination_frontier_item(&model, &challenges)
        .ok_or_else(|| anyhow!("planning requires an actionable Imagination frontier"))?;
    let item_hash = repo_frontier_item_hash(item)?;
    let request_id =
        crate::frontier_planning_request_id(&identity.runtime_id, &item.id, &item_hash);
    if cache
        .get_all::<RepoFrontierPlanDecisionReceipt>()?
        .iter()
        .any(|decision| decision.planning_request_id == request_id)
    {
        return Err(anyhow!(
            "current frontier planning request already has a terminal Mind decision"
        ));
    }
    if let Some(existing) = cache.get::<RepoFrontierPlanningRequest>(&request_id)? {
        validate_actionable_repo_frontier_planning_request(&cache, &existing)?;
        return Ok(existing);
    }
    let request = RepoFrontierPlanningRequest {
        schema_version: REPO_FRONTIER_PLANNING_REQUEST_SCHEMA_VERSION.into(),
        request_id: request_id.clone(),
        model_projection_digest: basis.projection_digest.clone(),
        model_source_documents: basis.source_documents.clone(),
        frontier_item_id: item.id.clone(),
        frontier_item_hash: item_hash,
        selected_organ: "Imagination".into(),
        source_scope: item.source_scope.clone(),
        requested_at: at.into(),
        contract: REPO_FRONTIER_PLANNING_CONTRACT.into(),
        runtime_id: identity.runtime_id,
        thread_id: thread.thread_id,
    };
    let (envelope, _) = cache.prepare_entry(&request_id, &request)?;
    let expected = keyed_repo_model_basis_envelopes(&cache, &basis)?;
    let mut writes = expected.clone();
    writes.push(envelope);
    if !backing.compare_and_swap_batch(&expected, writes)? {
        let mut reloaded = runtime_spine_cache(runtime_store)?;
        reloaded.pull_all_backing_stores()?;
        if let Some(existing) = reloaded.get::<RepoFrontierPlanningRequest>(&request_id)? {
            let mut retry = request;
            retry.requested_at = existing.requested_at.clone();
            return if existing == retry {
                Ok(existing)
            } else {
                Err(anyhow!("planning request CAS collision"))
            };
        }
        return Err(anyhow!("planning request lost exact keyed-model CAS"));
    }
    Ok(request)
}

pub(crate) fn validate_actionable_repo_frontier_planning_request(
    cache: &CultCache,
    request: &RepoFrontierPlanningRequest,
) -> Result<()> {
    chrono::DateTime::parse_from_rfc3339(&request.requested_at)
        .map_err(|_| anyhow!("planning request timestamp must be RFC3339"))?;
    let identity = require_identity(cache)?;
    let thread = cache
        .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
        .ok_or_else(|| anyhow!("planning request requires authoritative thread state"))?;
    thread.state()?;
    let view = require_keyed_repo_model_basis(
        cache,
        &request.model_projection_digest,
        &request.model_source_documents,
    )?;
    let model = view.memory_context_projection();
    let basis = view.reasoning_basis();
    let challenges = current_repo_model_claim_challenges(cache, &model, &basis)?;
    let item = model
        .frontier
        .iter()
        .find(|item| item.id == request.frontier_item_id)
        .ok_or_else(|| anyhow!("planning request frontier disappeared"))?;
    if !imagination_frontier_item_is_actionable(&model, &challenges, item) {
        return Err(anyhow!("planning request frontier is no longer actionable"));
    }
    let item_hash = repo_frontier_item_hash(item)?;
    let expected_request_id =
        crate::frontier_planning_request_id(&identity.runtime_id, &item.id, &item_hash);
    if request.schema_version != REPO_FRONTIER_PLANNING_REQUEST_SCHEMA_VERSION
        || request.contract != REPO_FRONTIER_PLANNING_CONTRACT
        || request.request_id != expected_request_id
        || request.model_projection_digest != basis.projection_digest
        || request.model_source_documents != basis.source_documents
        || request.frontier_item_id != item.id
        || request.frontier_item_hash != item_hash
        || request.selected_organ != "Imagination"
        || request.source_scope != item.source_scope
        || request.runtime_id != identity.runtime_id
        || request.thread_id.is_empty()
    {
        return Err(anyhow!(
            "planning request does not exactly bind its actionable frontier and runtime"
        ));
    }
    Ok(())
}

pub fn commit_repo_frontier_plan_mind_request(
    runtime_store: impl AsRef<Path>,
    imagination_result_id: &str,
    requested_at: &str,
) -> Result<RepoFrontierPlanMindRequest> {
    chrono::DateTime::parse_from_rfc3339(requested_at)
        .map_err(|_| anyhow!("Mind request timestamp must be RFC3339"))?;
    let runtime_store = runtime_store.as_ref();
    let mut cache = runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    let results = cache
        .get_all::<EpiphanyRuntimeRoleWorkerResult>()?
        .into_iter()
        .filter(|result| result.result_id == imagination_result_id)
        .collect::<Vec<_>>();
    if results.len() != 1 {
        return Err(anyhow!(
            "Mind request requires exactly one immutable Imagination result"
        ));
    }
    let result = &results[0];
    put_runtime_role_worker_result(runtime_store, result)?;
    let planning_request_id = result
        .frontier_planning_request_id
        .as_deref()
        .ok_or_else(|| anyhow!("Mind request source lacks planning request echo"))?;
    let planning = cache
        .get::<RepoFrontierPlanningRequest>(planning_request_id)?
        .ok_or_else(|| anyhow!("Mind request source planning request disappeared"))?;
    let candidate = result
        .frontier_plan_candidate()?
        .ok_or_else(|| anyhow!("Mind request source candidate disappeared"))?;
    validate_actionable_repo_frontier_planning_request(&cache, &planning)?;
    validate_repo_frontier_plan_candidate_against_request(&cache, &candidate, &planning)?;
    let candidate_sha256 = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&candidate)?));
    let request_id = crate::frontier_plan_mind_request_id(
        &planning.runtime_id,
        &planning.request_id,
        &result.result_id,
        &candidate_sha256,
    );
    let request = RepoFrontierPlanMindRequest {
        schema_version: REPO_FRONTIER_PLAN_MIND_REQUEST_SCHEMA_VERSION.into(),
        request_id: request_id.clone(),
        planning_request_id: planning.request_id,
        imagination_result_id: result.result_id.clone(),
        imagination_job_id: result.job_id.clone(),
        candidate_id: candidate.candidate_id,
        candidate_sha256,
        runtime_id: planning.runtime_id,
        thread_id: planning.thread_id,
        requested_at: requested_at.into(),
        contract: REPO_FRONTIER_PLAN_MIND_REQUEST_CONTRACT.into(),
    };
    if let Some(existing) = cache.get::<RepoFrontierPlanMindRequest>(&request_id)? {
        let mut retry = request.clone();
        retry.requested_at = existing.requested_at.clone();
        return if existing == retry {
            Ok(existing)
        } else {
            Err(anyhow!("Mind request identity collision"))
        };
    }
    let (envelope, _) = cache.prepare_entry(&request_id, &request)?;
    let backing = SingleFileMessagePackBackingStore::new(runtime_store);
    let basis = crate::EpiphanyRepoModelBasis {
        projection_digest: planning.model_projection_digest.clone(),
        source_documents: planning.model_source_documents.clone(),
    };
    let expected = keyed_repo_model_basis_envelopes(&cache, &basis)?;
    let mut writes = expected.clone();
    writes.push(envelope);
    if !backing.compare_and_swap_batch(&expected, writes)? {
        return Err(anyhow!(
            "Mind request lost exact keyed-model CAS or immutable request claim"
        ));
    }
    Ok(request)
}

pub(crate) fn validate_repo_frontier_plan_mind_request(
    cache: &CultCache,
    request: &RepoFrontierPlanMindRequest,
) -> Result<(RepoFrontierPlanningRequest, RepoFrontierPlanCandidate)> {
    let (planning, candidate) = validate_repo_frontier_plan_mind_request_identity(cache, request)?;
    validate_actionable_repo_frontier_planning_request(cache, &planning)?;
    validate_repo_frontier_plan_candidate_against_request(cache, &candidate, &planning)?;
    Ok((planning, candidate))
}

fn validate_repo_frontier_plan_mind_request_identity(
    cache: &CultCache,
    request: &RepoFrontierPlanMindRequest,
) -> Result<(RepoFrontierPlanningRequest, RepoFrontierPlanCandidate)> {
    if request.schema_version != REPO_FRONTIER_PLAN_MIND_REQUEST_SCHEMA_VERSION
        || request.contract != REPO_FRONTIER_PLAN_MIND_REQUEST_CONTRACT
        || chrono::DateTime::parse_from_rfc3339(&request.requested_at).is_err()
    {
        return Err(anyhow!("invalid typed Mind request"));
    }
    let result = cache
        .get_all::<EpiphanyRuntimeRoleWorkerResult>()?
        .into_iter()
        .find(|r| r.result_id == request.imagination_result_id)
        .ok_or_else(|| anyhow!("Mind request source result disappeared"))?;
    let planning = cache
        .get::<RepoFrontierPlanningRequest>(&request.planning_request_id)?
        .ok_or_else(|| anyhow!("Mind request planning request disappeared"))?;
    let candidate = result
        .frontier_plan_candidate()?
        .ok_or_else(|| anyhow!("Mind request candidate disappeared"))?;
    let hash = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&candidate)?));
    if result.job_id != request.imagination_job_id
        || result.frontier_planning_request_id.as_deref()
            != Some(request.planning_request_id.as_str())
        || candidate.candidate_id != request.candidate_id
        || hash != request.candidate_sha256
        || request.runtime_id != planning.runtime_id
        || request.thread_id != planning.thread_id
    {
        return Err(anyhow!(
            "Mind request substituted immutable Imagination causal identity"
        ));
    }
    Ok((planning, candidate))
}

fn actionable_imagination_frontier_item<'a>(
    model: &'a crate::EpiphanyMemoryGraphSnapshot,
    challenges: &[RepoModelClaimChallenge],
) -> Option<&'a crate::RepoFrontierItem> {
    let mut eligible = model
        .frontier
        .iter()
        .filter(|item| imagination_frontier_item_is_actionable(model, challenges, item))
        .collect::<Vec<_>>();
    eligible.sort_by(|a, b| a.id.cmp(&b.id));
    eligible.into_iter().next()
}

fn imagination_frontier_item_is_actionable(
    model: &crate::EpiphanyMemoryGraphSnapshot,
    challenges: &[RepoModelClaimChallenge],
    item: &crate::RepoFrontierItem,
) -> bool {
    let terminal = |id: &str| {
        model
            .frontier
            .iter()
            .find(|item| item.id == id)
            .is_some_and(|item| {
                matches!(
                    item.status,
                    crate::RepoFrontierStatus::Resolved
                        | crate::RepoFrontierStatus::Retired
                        | crate::RepoFrontierStatus::Superseded
                )
            })
    };
    item.status == crate::RepoFrontierStatus::Active
        && item.recommended_next_organ == "Imagination"
        && !item.source_scope.is_empty()
        && safe_sorted_unique_paths(&item.source_scope)
        && frontier_target_claims_unchallenged(item, challenges)
        && item.dependency_item_ids.iter().all(|id| terminal(id))
}

fn frontier_target_claims_unchallenged(
    item: &crate::RepoFrontierItem,
    challenges: &[RepoModelClaimChallenge],
) -> bool {
    !challenges
        .iter()
        .any(|challenge| item.target_claim_ids.contains(&challenge.target_claim_id))
}

fn current_repo_model_claim_challenges(
    cache: &CultCache,
    model: &crate::EpiphanyMemoryGraphSnapshot,
    basis: &crate::EpiphanyRepoModelBasis,
) -> Result<Vec<RepoModelClaimChallenge>> {
    let mut current = Vec::new();
    for challenge in cache.get_all::<RepoModelClaimChallenge>()? {
        let Some(claim) = model
            .nodes
            .iter()
            .find(|node| node.id == challenge.target_claim_id)
        else {
            continue;
        };
        if format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(claim)?))
            == challenge.target_claim_sha256
        {
            validate_repo_model_claim_challenge_chain(cache, model, basis, &challenge, true)?;
            current.push(challenge);
        }
    }
    Ok(current)
}

fn validate_repo_model_claim_challenge_chain(
    cache: &CultCache,
    model: &crate::EpiphanyMemoryGraphSnapshot,
    basis: &crate::EpiphanyRepoModelBasis,
    challenge: &RepoModelClaimChallenge,
    require_current_model: bool,
) -> Result<()> {
    if challenge.schema_version != REPO_MODEL_CLAIM_CHALLENGE_SCHEMA_VERSION
        || challenge.contract != REPO_MODEL_CLAIM_CHALLENGE_CONTRACT
        || challenge.challenge_id.trim().is_empty()
        || challenge.finding.trim().is_empty()
        || challenge.uncertainty.trim().is_empty()
        || challenge.source_refs.is_empty()
        || challenge.evidence_ids.is_empty()
        || chrono::DateTime::parse_from_rfc3339(&challenge.challenged_at).is_err()
    {
        return Err(anyhow!("invalid repo model claim challenge"));
    }
    let packet = cache
        .get::<EyesEvidencePacket>(&challenge.eyes_evidence_packet_id)?
        .ok_or_else(|| anyhow!("claim challenge requires exact Eyes evidence packet"))?;
    if packet.schema_version != EYES_EVIDENCE_PACKET_SCHEMA_VERSION
        || packet.contract.trim().is_empty()
        || chrono::DateTime::parse_from_rfc3339(&packet.emitted_at).is_err()
        || packet.source_result_id != challenge.source_result_id
        || packet.source_job_id != challenge.source_job_id
        || packet.source_refs != challenge.source_refs
        || packet.evidence_ids != challenge.evidence_ids
        || format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&packet)?))
            != challenge.eyes_evidence_packet_sha256
    {
        return Err(anyhow!("claim challenge substituted Eyes evidence"));
    }
    if challenge.model_projection_digest != basis.projection_digest
        || challenge.model_source_documents != basis.source_documents
    {
        return Err(anyhow!("claim challenge keyed model basis mismatch"));
    }
    if require_current_model {
        basis.validate_against_cache(cache)?;
    }
    let claim = model
        .nodes
        .iter()
        .find(|node| node.id == challenge.target_claim_id)
        .ok_or_else(|| anyhow!("claim challenge target claim is missing"))?;
    if format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(claim)?))
        != challenge.target_claim_sha256
    {
        return Err(anyhow!("claim challenge target claim identity mismatch"));
    }
    Ok(())
}

pub fn commit_repo_model_claim_challenge(
    store_path: impl AsRef<Path>,
    challenge: &RepoModelClaimChallenge,
) -> Result<()> {
    if challenge.schema_version != REPO_MODEL_CLAIM_CHALLENGE_SCHEMA_VERSION
        || challenge.contract != REPO_MODEL_CLAIM_CHALLENGE_CONTRACT
        || challenge.challenge_id.trim().is_empty()
        || challenge.finding.trim().is_empty()
        || challenge.uncertainty.trim().is_empty()
        || challenge.source_refs.is_empty()
        || challenge.evidence_ids.is_empty()
        || chrono::DateTime::parse_from_rfc3339(&challenge.challenged_at).is_err()
    {
        return Err(anyhow!("invalid repo model claim challenge"));
    }
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    let envelopes = backing.pull_all()?;
    let (view, basis) = current_keyed_repo_model(&cache)?;
    let model = view.memory_context_projection();
    validate_repo_model_claim_challenge_chain(&cache, &model, &basis, challenge, true)?;
    if let Some(existing) = cache.get::<RepoModelClaimChallenge>(&challenge.challenge_id)? {
        return if existing == *challenge {
            Ok(())
        } else {
            Err(anyhow!("claim challenge ids are immutable"))
        };
    }
    let packet_envelope = envelopes
        .iter()
        .find(|entry| {
            entry.r#type == EYES_EVIDENCE_PACKET_TYPE
                && entry.key == challenge.eyes_evidence_packet_id
        })
        .cloned()
        .ok_or_else(|| anyhow!("claim challenge packet envelope is missing"))?;
    let (challenge_envelope, _) = cache.prepare_entry(&challenge.challenge_id, challenge)?;
    let mut expected = keyed_repo_model_basis_envelopes(&cache, &basis)?;
    expected.push(packet_envelope);
    let mut writes = expected.clone();
    writes.push(challenge_envelope);
    if !backing.compare_and_swap_batch(&expected, writes)? {
        let mut reloaded = runtime_spine_cache(store_path)?;
        reloaded.pull_all_backing_stores()?;
        return match reloaded.get::<RepoModelClaimChallenge>(&challenge.challenge_id)? {
            Some(existing) if existing == *challenge => Ok(()),
            Some(_) => Err(anyhow!("claim challenge immutable collision")),
            None => Err(anyhow!("claim challenge lost exact model/packet CAS")),
        };
    }
    Ok(())
}

pub fn commit_repo_model_claim_repair_request(
    store_path: impl AsRef<Path>,
    challenge_id: &str,
    requested_at: &str,
) -> Result<RepoModelClaimRepairRequest> {
    validate_non_empty(challenge_id, "claim repair challenge id")?;
    chrono::DateTime::parse_from_rfc3339(requested_at)
        .map_err(|_| anyhow!("claim repair request timestamp must be RFC3339"))?;
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    let envelopes = backing.pull_all()?;
    let (view, basis) = current_keyed_repo_model(&cache)?;
    let model = view.memory_context_projection();
    let challenge = cache
        .get::<RepoModelClaimChallenge>(challenge_id)?
        .ok_or_else(|| anyhow!("claim repair request requires exact challenge"))?;
    validate_repo_model_claim_challenge_chain(&cache, &model, &basis, &challenge, true)?;
    let packet = cache
        .get::<EyesEvidencePacket>(&challenge.eyes_evidence_packet_id)?
        .ok_or_else(|| anyhow!("claim repair request requires exact Eyes packet"))?;
    let identity = cache
        .get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("claim repair request requires runtime identity"))?;
    let thread = cache
        .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
        .ok_or_else(|| anyhow!("claim repair request requires authoritative thread"))?;
    let claim = model
        .nodes
        .iter()
        .find(|node| node.id == challenge.target_claim_id)
        .ok_or_else(|| anyhow!("claim repair target is no longer present"))?;
    if format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(claim)?))
        != challenge.target_claim_sha256
    {
        return Err(anyhow!(
            "claim repair challenge is already resolved by a changed claim"
        ));
    }
    let challenge_sha256 = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&challenge)?));
    let request_id = format!("repo-model-claim-repair-{challenge_id}");
    let mut affected_frontier = model
        .frontier
        .iter()
        .filter(|item| item.target_claim_ids.contains(&challenge.target_claim_id))
        .map(|item| {
            Ok(crate::RepoModelClaimRepairFrontierRef {
                frontier_item_id: item.id.clone(),
                frontier_item_sha256: format!(
                    "{:x}",
                    Sha256::digest(rmp_serde::to_vec_named(item)?)
                ),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    affected_frontier.sort_by(|a, b| a.frontier_item_id.cmp(&b.frontier_item_id));
    let request = RepoModelClaimRepairRequest {
        schema_version: REPO_MODEL_CLAIM_REPAIR_REQUEST_SCHEMA_VERSION.into(),
        request_id: request_id.clone(),
        challenge_id: challenge.challenge_id.clone(),
        challenge_sha256,
        eyes_evidence_packet_id: packet.packet_id.clone(),
        eyes_evidence_packet_sha256: challenge.eyes_evidence_packet_sha256.clone(),
        source_result_id: challenge.source_result_id.clone(),
        source_job_id: challenge.source_job_id.clone(),
        original_model_projection_digest: challenge.model_projection_digest.clone(),
        original_model_source_documents: challenge.model_source_documents.clone(),
        current_model_projection_digest: basis.projection_digest.clone(),
        current_model_source_documents: basis.source_documents.clone(),
        target_claim_id: challenge.target_claim_id.clone(),
        target_claim_sha256: challenge.target_claim_sha256.clone(),
        runtime_id: identity.runtime_id.clone(),
        thread_id: thread.thread_id.clone(),
        affected_frontier,
        requested_at: requested_at.to_string(),
        contract: REPO_MODEL_CLAIM_REPAIR_REQUEST_CONTRACT.into(),
    };
    validate_current_repo_model_claim_repair_request(&cache, &request)?;
    if let Some(existing) = cache.get::<RepoModelClaimRepairRequest>(&request_id)? {
        return if existing == request {
            Ok(existing)
        } else {
            Err(anyhow!("claim repair request identity collision"))
        };
    }
    let challenge_envelope = envelopes
        .iter()
        .find(|entry| {
            entry.r#type == "epiphany.eyes.repo_model_claim_challenge"
                && entry.key == challenge.challenge_id
        })
        .cloned()
        .ok_or_else(|| anyhow!("claim repair challenge envelope is missing"))?;
    let packet_envelope = envelopes
        .iter()
        .find(|entry| entry.r#type == EYES_EVIDENCE_PACKET_TYPE && entry.key == packet.packet_id)
        .cloned()
        .ok_or_else(|| anyhow!("claim repair packet envelope is missing"))?;
    let identity_envelope = envelopes
        .iter()
        .find(|entry| entry.r#type == RUNTIME_IDENTITY_TYPE && entry.key == RUNTIME_IDENTITY_KEY)
        .cloned()
        .ok_or_else(|| anyhow!("claim repair identity envelope is missing"))?;
    let thread_envelope = envelopes
        .iter()
        .find(|entry| entry.r#type == THREAD_STATE_TYPE && entry.key == crate::THREAD_STATE_KEY)
        .cloned()
        .ok_or_else(|| anyhow!("claim repair thread envelope is missing"))?;
    let (request_envelope, _) = cache.prepare_entry(&request_id, &request)?;
    let mut expected = keyed_repo_model_basis_envelopes(&cache, &basis)?;
    expected.extend([
        challenge_envelope.clone(),
        packet_envelope.clone(),
        identity_envelope.clone(),
        thread_envelope.clone(),
    ]);
    let mut replacement = expected.clone();
    replacement.push(request_envelope);
    if !backing.compare_and_swap_batch(&expected, replacement)? {
        let mut reloaded = runtime_spine_cache(store_path)?;
        reloaded.pull_all_backing_stores()?;
        return match reloaded.get::<RepoModelClaimRepairRequest>(&request_id)? {
            Some(existing) if existing == request => Ok(existing),
            Some(_) => Err(anyhow!("claim repair request immutable collision")),
            None => Err(anyhow!("claim repair request lost exact causal CAS")),
        };
    }
    Ok(request)
}

pub(crate) fn validate_current_repo_model_claim_repair_request(
    cache: &CultCache,
    request: &RepoModelClaimRepairRequest,
) -> Result<()> {
    chrono::DateTime::parse_from_rfc3339(&request.requested_at)
        .map_err(|_| anyhow!("claim repair request timestamp must be RFC3339"))?;
    let view = require_keyed_repo_model_basis(
        cache,
        &request.current_model_projection_digest,
        &request.current_model_source_documents,
    )?;
    let model = view.memory_context_projection();
    let basis = view.reasoning_basis();
    let challenge = cache
        .get::<RepoModelClaimChallenge>(&request.challenge_id)?
        .ok_or_else(|| anyhow!("claim repair request requires exact challenge"))?;
    validate_repo_model_claim_challenge_chain(cache, &model, &basis, &challenge, true)?;
    let packet = cache
        .get::<EyesEvidencePacket>(&challenge.eyes_evidence_packet_id)?
        .ok_or_else(|| anyhow!("claim repair request requires exact Eyes packet"))?;
    let identity = cache
        .get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("claim repair request requires runtime identity"))?;
    let thread = cache
        .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
        .ok_or_else(|| anyhow!("claim repair request requires authoritative thread"))?;
    let claim = model
        .nodes
        .iter()
        .find(|node| node.id == challenge.target_claim_id)
        .ok_or_else(|| anyhow!("claim repair target is no longer present"))?;
    if format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(claim)?))
        != challenge.target_claim_sha256
    {
        return Err(anyhow!(
            "claim repair challenge is already resolved by a changed claim"
        ));
    }
    let mut affected_frontier = model
        .frontier
        .iter()
        .filter(|item| item.target_claim_ids.contains(&challenge.target_claim_id))
        .map(|item| {
            Ok(RepoModelClaimRepairFrontierRef {
                frontier_item_id: item.id.clone(),
                frontier_item_sha256: format!(
                    "{:x}",
                    Sha256::digest(rmp_serde::to_vec_named(item)?)
                ),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    affected_frontier.sort_by(|a, b| a.frontier_item_id.cmp(&b.frontier_item_id));
    let expected = RepoModelClaimRepairRequest {
        schema_version: REPO_MODEL_CLAIM_REPAIR_REQUEST_SCHEMA_VERSION.into(),
        request_id: format!("repo-model-claim-repair-{}", challenge.challenge_id),
        challenge_id: challenge.challenge_id.clone(),
        challenge_sha256: format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&challenge)?)),
        eyes_evidence_packet_id: packet.packet_id.clone(),
        eyes_evidence_packet_sha256: challenge.eyes_evidence_packet_sha256.clone(),
        source_result_id: challenge.source_result_id.clone(),
        source_job_id: challenge.source_job_id.clone(),
        original_model_projection_digest: challenge.model_projection_digest.clone(),
        original_model_source_documents: challenge.model_source_documents.clone(),
        current_model_projection_digest: basis.projection_digest,
        current_model_source_documents: basis.source_documents,
        target_claim_id: challenge.target_claim_id.clone(),
        target_claim_sha256: challenge.target_claim_sha256.clone(),
        runtime_id: identity.runtime_id.clone(),
        thread_id: thread.thread_id.clone(),
        affected_frontier,
        requested_at: request.requested_at.clone(),
        contract: REPO_MODEL_CLAIM_REPAIR_REQUEST_CONTRACT.into(),
    };
    if *request != expected {
        return Err(anyhow!(
            "claim repair request does not match the current canonical causal chain"
        ));
    }
    Ok(())
}

fn validate_repo_frontier_plan_candidate_against_request(
    cache: &CultCache,
    candidate: &RepoFrontierPlanCandidate,
    request: &RepoFrontierPlanningRequest,
) -> Result<()> {
    if candidate.schema_version != REPO_FRONTIER_PLAN_CANDIDATE_SCHEMA_VERSION
        || candidate.contract != REPO_FRONTIER_PLANNING_CONTRACT
        || candidate.selected_fields_invalid()
        || candidate.planning_request_id != request.request_id
        || candidate.model_projection_digest != request.model_projection_digest
        || candidate.model_source_documents != request.model_source_documents
        || candidate.frontier_item_id != request.frontier_item_id
        || candidate.frontier_item_hash != request.frontier_item_hash
    {
        return Err(anyhow!(
            "frontier planning candidate substituted request identity or required cargo"
        ));
    }
    let expected_candidate_id = canonical_repo_frontier_plan_candidate_id(candidate)?;
    if candidate.candidate_id != expected_candidate_id {
        return Err(anyhow!("frontier planning candidate id is not canonical"));
    }
    let model = require_keyed_repo_model_basis(
        cache,
        &request.model_projection_digest,
        &request.model_source_documents,
    )?
    .memory_context_projection();
    let item = model
        .frontier
        .iter()
        .find(|item| item.id == request.frontier_item_id)
        .ok_or_else(|| anyhow!("frontier planning candidate frontier disappeared"))?;
    if format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(item)?)) != request.frontier_item_hash
        || item.source_scope != request.source_scope
        || !candidate.safe_paths.iter().all(|path| {
            request.source_scope.iter().any(|scope| {
                path == scope
                    || path.starts_with(&format!("{}/", scope.trim_end_matches(['/', '\\'])))
            })
        })
    {
        return Err(anyhow!(
            "frontier planning candidate exceeds exact frontier authority"
        ));
    }
    Ok(())
}

pub fn canonical_repo_frontier_plan_candidate_id(
    candidate: &RepoFrontierPlanCandidate,
) -> Result<String> {
    let semantic_bytes = rmp_serde::to_vec_named(&(
        &candidate.planning_request_id,
        &candidate.model_projection_digest,
        &candidate.model_source_documents,
        &candidate.frontier_item_id,
        &candidate.frontier_item_hash,
        &candidate.safe_paths,
        &candidate.action,
        &candidate.command,
        &candidate.checks,
        &candidate.stop_conditions,
        &candidate.rollback_steps,
        &candidate.commit_message,
        &candidate.proposed_at,
    ))?;
    Ok(format!(
        "repo-frontier-plan-candidate-{:x}",
        Sha256::digest(semantic_bytes)
    ))
}

pub fn commit_repo_frontier_plan_decision(
    runtime_store: impl AsRef<Path>,
    mind_result_id: &str,
) -> Result<RepoFrontierPlanDecisionReceipt> {
    commit_repo_frontier_plan_decision_inner(
        runtime_store,
        FrontierPlanDecisionSource::MindWorker(mind_result_id),
        None,
    )
}

enum FrontierPlanDecisionSource<'a> {
    MindWorker(&'a str),
    Operator(&'a crate::RepoFrontierPlanOperatorReview),
}

/// Read-only, bounded identities for candidates already routed to canonical
/// Mind review. Invalid, stale, or terminal candidates are not projected.
pub fn pending_repo_frontier_plan_reviews(
    runtime_store: impl AsRef<Path>,
    limit: usize,
) -> Result<Vec<crate::RepoFrontierPlanReviewSummary>> {
    if limit == 0 || limit > 25 {
        return Err(anyhow!(
            "Mind review projection limit must be within 1..=25"
        ));
    }
    let mut cache = runtime_spine_cache(runtime_store.as_ref())?;
    cache.pull_all_backing_stores()?;
    let terminal = cache
        .get_all::<RepoFrontierPlanDecisionReceipt>()?
        .into_iter()
        .map(|receipt| receipt.planning_request_id)
        .collect::<std::collections::BTreeSet<_>>();
    let mut summaries = Vec::new();
    for request in cache.get_all::<RepoFrontierPlanMindRequest>()? {
        let Ok((planning, candidate)) = validate_repo_frontier_plan_mind_request(&cache, &request)
        else {
            continue;
        };
        if terminal.contains(&planning.request_id) {
            continue;
        }
        summaries.push(crate::RepoFrontierPlanReviewSummary {
            mind_request_id: request.request_id,
            candidate_id: candidate.candidate_id,
            candidate_sha256: request.candidate_sha256,
            model_projection_digest: planning.model_projection_digest,
            model_source_documents: planning.model_source_documents,
            frontier_item_id: planning.frontier_item_id,
            requested_at: request.requested_at,
        });
    }
    summaries.sort_by(|a, b| {
        a.requested_at
            .cmp(&b.requested_at)
            .then_with(|| a.mind_request_id.cmp(&b.mind_request_id))
    });
    summaries.truncate(limit);
    Ok(summaries)
}

/// Canonical Mind commit path for an authenticated operator's exact review
/// request. The operator selects a disposition; Mind revalidates the complete
/// immutable candidate and current model before performing the same atomic
/// terminal transition used by a Mind worker.
pub fn commit_operator_repo_frontier_plan_review(
    runtime_store: impl AsRef<Path>,
    review: &crate::RepoFrontierPlanOperatorReview,
) -> Result<RepoFrontierPlanDecisionReceipt> {
    commit_repo_frontier_plan_decision_inner(
        runtime_store,
        FrontierPlanDecisionSource::Operator(review),
        None,
    )
}

/// Classifies only operator-visible precondition refusal. Store corruption,
/// decoding failure, and I/O remain errors and must never be fossilized as a
/// terminal Refused command result.
pub fn operator_repo_frontier_plan_review_is_current(
    runtime_store: impl AsRef<Path>,
    review: &crate::RepoFrontierPlanOperatorReview,
) -> Result<bool> {
    let mut cache = runtime_spine_cache(runtime_store.as_ref())?;
    cache.pull_all_backing_stores()?;
    let Some(request) = cache.get::<RepoFrontierPlanMindRequest>(&review.mind_request_id)? else {
        return Ok(false);
    };
    let (planning, candidate) =
        validate_repo_frontier_plan_mind_request_identity(&cache, &request)?;
    if review.candidate_id != candidate.candidate_id
        || review.candidate_sha256 != request.candidate_sha256
        || review.expected_model_projection_digest != planning.model_projection_digest
        || review.expected_model_source_documents != planning.model_source_documents
    {
        return Ok(false);
    }
    let terminal = cache
        .get_all::<RepoFrontierPlanDecisionReceipt>()?
        .into_iter()
        .filter(|receipt| receipt.planning_request_id == planning.request_id)
        .collect::<Vec<_>>();
    if !terminal.is_empty() {
        if terminal.len() != 1 {
            return Err(anyhow!(
                "Mind review candidate has multiple terminal decisions"
            ));
        }
        let receipt = &terminal[0];
        return Ok(receipt.decision == review.decision
            && receipt.candidate_id == review.candidate_id
            && receipt.candidate_sha256 == review.candidate_sha256
            && receipt.model_projection_digest == review.expected_model_projection_digest
            && receipt.model_source_documents == review.expected_model_source_documents
            && receipt.decision_source.as_ref()
                == Some(
                    &crate::RepoFrontierPlanDecisionSource::AuthenticatedOperatorReview {
                        command_id: review.command_id.clone(),
                        admission_id: review.admission_id.clone(),
                        packet_sha256: review.packet_sha256.clone(),
                        source_actor_id: review.source_actor_id.clone(),
                    },
                ));
    }
    if require_keyed_repo_model_basis(
        &cache,
        &planning.model_projection_digest,
        &planning.model_source_documents,
    )
    .is_err()
    {
        return Ok(false);
    }
    validate_repo_frontier_plan_candidate_against_request(&cache, &candidate, &planning)?;
    Ok(true)
}

fn commit_repo_frontier_plan_decision_inner(
    runtime_store: impl AsRef<Path>,
    source: FrontierPlanDecisionSource<'_>,
    pre_cas: Option<&(dyn Fn() + Sync)>,
) -> Result<RepoFrontierPlanDecisionReceipt> {
    let runtime_store = runtime_store.as_ref();
    let mut cache = runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    let (
        mind_request_id,
        decision,
        rationale,
        decided_at,
        decision_source,
        decision_context_id,
        operator_provenance,
    ) = match source {
        FrontierPlanDecisionSource::MindWorker(result_id) => {
            let result = cache
                .get_all::<EpiphanyRuntimeRoleWorkerResult>()?
                .into_iter()
                .find(|result| result.result_id == result_id)
                .ok_or_else(|| anyhow!("frontier plan decision lost its Mind result"))?;
            let typed = result
                .frontier_plan_mind_decision()?
                .ok_or_else(|| anyhow!("frontier plan decision requires a typed Mind decision"))?;
            (
                result
                    .frontier_plan_mind_request_id
                    .clone()
                    .ok_or_else(|| anyhow!("Mind result lacks its plan request echo"))?,
                typed.decision,
                typed.rationale,
                typed.decided_at,
                crate::RepoFrontierPlanDecisionSource::MindWorker {
                    result_id: result.result_id,
                    job_id: result.job_id,
                },
                Some(result.decision_context_id),
                None,
            )
        }
        FrontierPlanDecisionSource::Operator(review) => {
            let admitted = cache
                .get::<crate::LocalAdmittedOperatorCommand>(&review.command_id)?
                .ok_or_else(|| anyhow!("operator review lost its admitted command"))?;
            if admitted.admission_id != review.admission_id
                || admitted.packet_sha256 != review.packet_sha256
                || admitted.source_actor_id != review.source_actor_id
            {
                return Err(anyhow!("operator review provenance mismatch"));
            }
            let provenance = cache
                .get_envelope::<crate::LocalAdmittedOperatorCommand>(&review.command_id)?
                .ok_or_else(|| anyhow!("operator review provenance envelope disappeared"))?;
            (
                review.mind_request_id.clone(),
                review.decision,
                format!("Authenticated operator requested {:?}.", review.decision).to_lowercase(),
                review.decided_at.clone(),
                crate::RepoFrontierPlanDecisionSource::AuthenticatedOperatorReview {
                    command_id: review.command_id.clone(),
                    admission_id: review.admission_id.clone(),
                    packet_sha256: review.packet_sha256.clone(),
                    source_actor_id: review.source_actor_id.clone(),
                },
                None,
                Some(provenance),
            )
        }
    };
    chrono::DateTime::parse_from_rfc3339(&decided_at)
        .map_err(|_| anyhow!("frontier plan decision time must be RFC3339"))?;
    let mind_request = cache
        .get::<RepoFrontierPlanMindRequest>(&mind_request_id)?
        .ok_or_else(|| anyhow!("frontier plan decision requires its typed Mind request"))?;
    let (planning, candidate) = validate_repo_frontier_plan_mind_request(&cache, &mind_request)?;
    if let FrontierPlanDecisionSource::Operator(review) = source {
        if review.candidate_id != candidate.candidate_id
            || review.candidate_sha256 != mind_request.candidate_sha256
            || review.expected_model_projection_digest != planning.model_projection_digest
            || review.expected_model_source_documents != planning.model_source_documents
        {
            return Err(anyhow!(
                "operator review does not bind the exact keyed model"
            ));
        }
    }
    let candidate_sha256 = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&candidate)?));
    let decision_id = format!(
        "repo-frontier-plan-decision-{:x}",
        Sha256::digest(planning.request_id.as_bytes())
    );
    let receipt = RepoFrontierPlanDecisionReceipt {
        schema_version: REPO_FRONTIER_PLAN_DECISION_RECEIPT_SCHEMA_VERSION.into(),
        decision_id: decision_id.clone(),
        planning_request_id: planning.request_id.clone(),
        legacy_mind_worker_result_id: None,
        legacy_mind_worker_job_id: None,
        candidate_id: candidate.candidate_id.clone(),
        candidate_sha256,
        model_projection_digest: planning.model_projection_digest.clone(),
        model_source_documents: planning.model_source_documents.clone(),
        frontier_item_id: planning.frontier_item_id.clone(),
        frontier_item_hash: planning.frontier_item_hash.clone(),
        decision,
        rationale,
        decided_at: decided_at.clone(),
        contract: REPO_FRONTIER_PLAN_DECISION_CONTRACT.into(),
        decision_source: Some(decision_source),
    };
    if let Some(existing) = cache.get::<RepoFrontierPlanDecisionReceipt>(&decision_id)? {
        return if existing == receipt {
            Ok(existing)
        } else {
            Err(anyhow!("frontier plan decision identity collision"))
        };
    }

    let basis = crate::EpiphanyRepoModelBasis {
        projection_digest: planning.model_projection_digest.clone(),
        source_documents: planning.model_source_documents.clone(),
    };
    let mut strong_reads = keyed_repo_model_basis_envelopes(&cache, &basis)?;
    for envelope in [
        cache.get_envelope::<RepoFrontierPlanningRequest>(&planning.request_id)?,
        cache.get_envelope::<RepoFrontierPlanMindRequest>(&mind_request.request_id)?,
    ]
    .into_iter()
    .flatten()
    {
        strong_reads.push(envelope);
    }
    let mut writes = Vec::new();
    if decision == RepoFrontierPlanDecision::Adopt {
        let view = require_keyed_repo_model_basis(
            &cache,
            &planning.model_projection_digest,
            &planning.model_source_documents,
        )?;
        let mut item = view
            .frontier
            .into_iter()
            .find(|item| item.id == planning.frontier_item_id)
            .ok_or_else(|| anyhow!("frontier plan decision target disappeared"))?;
        if repo_frontier_item_hash(&item)? != planning.frontier_item_hash {
            return Err(anyhow!("frontier plan decision target changed"));
        }
        item.adopted_plan = Some(crate::RepoFrontierAdoptedPlan {
            planning_request_id: planning.request_id.clone(),
            result_id: mind_request.imagination_result_id.clone(),
            job_id: mind_request.imagination_job_id.clone(),
            candidate_id: candidate.candidate_id.clone(),
            candidate_sha256: mind_request.candidate_sha256.clone(),
            safe_paths: candidate.safe_paths,
            action: candidate.action,
            command: candidate.command,
            checks: candidate.checks,
            stop_conditions: candidate.stop_conditions,
            rollback_steps: candidate.rollback_steps,
            commit_message: candidate.commit_message,
            execution_amendment: None,
        });
        let proposal = crate::EpiphanyRepoModelMutationProposal::new(
            format!("repo-frontier-plan-adoption-{decision_id}"),
            mind_request.request_id.clone(),
            decision_id.clone(),
            vec![
                candidate.candidate_id.clone(),
                mind_request.imagination_result_id.clone(),
            ],
            crate::load_current_runtime_repository_body_basis(runtime_store)?,
            vec![crate::EpiphanyRepoModelMutationOperation::PutFrontier { item }],
        )?;
        let plan = crate::plan_repo_model_mutation(runtime_store, &proposal)?;
        strong_reads.extend(plan.strong_reads);
        writes.extend(plan.writes);
    }
    writes.push(cache.prepare_entry(&receipt.decision_id, &receipt)?.0);
    strong_reads.sort_by(|left, right| {
        left.r#type
            .cmp(&right.r#type)
            .then(left.key.cmp(&right.key))
    });
    strong_reads.dedup_by(|left, right| left.r#type == right.r#type && left.key == right.key);
    if let Some(pre_cas) = pre_cas {
        pre_cas();
    }
    let outcome = if let Some(context_id) = decision_context_id {
        crate::commit_mind_mutation(
            runtime_store,
            &context_id,
            "Mind.repo_frontier_plan_decision",
            strong_reads,
            writes,
            &decided_at,
        )?
    } else {
        crate::commit_operator_mind_mutation(
            runtime_store,
            operator_provenance
                .ok_or_else(|| anyhow!("operator plan decision lacks provenance"))?,
            "Mind.repo_frontier_plan_decision",
            strong_reads,
            writes,
            &decided_at,
        )?
    };
    match outcome {
        crate::EpiphanyMindCommitOutcome::Committed(_) => Ok(receipt),
        crate::EpiphanyMindCommitOutcome::Conflict { .. } => {
            let mut reloaded = runtime_spine_cache(runtime_store)?;
            reloaded.pull_all_backing_stores()?;
            match reloaded.get::<RepoFrontierPlanDecisionReceipt>(&decision_id)? {
                Some(existing) if existing == receipt => Ok(existing),
                _ => Err(anyhow!("frontier plan decision lost its exact keyed CAS")),
            }
        }
    }
}

impl RepoFrontierPlanCandidate {
    fn selected_fields_invalid(&self) -> bool {
        self.candidate_id.trim().is_empty()
            || self.safe_paths.is_empty()
            || !safe_sorted_unique_paths(&self.safe_paths)
            || self.action.trim().is_empty()
            || self.command.trim().is_empty()
            || self.checks.is_empty()
            || self.checks.iter().any(|v| v.trim().is_empty())
            || self.stop_conditions.is_empty()
            || self.stop_conditions.iter().any(|v| v.trim().is_empty())
            || self.rollback_steps.is_empty()
            || self.rollback_steps.iter().any(|v| v.trim().is_empty())
            || self.commit_message.trim().is_empty()
            || chrono::DateTime::parse_from_rfc3339(&self.proposed_at).is_err()
    }
}

pub fn select_and_commit_repo_frontier_route(
    runtime_store: impl AsRef<Path>,
    at: &str,
) -> Result<RepoFrontierRoute> {
    chrono::DateTime::parse_from_rfc3339(at)
        .map_err(|_| anyhow!("repo frontier route timestamp must be RFC3339"))?;
    let runtime_store = runtime_store.as_ref();
    let mut cache = runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let backing = SingleFileMessagePackBackingStore::new(runtime_store);
    let (view, basis) = current_keyed_repo_model(&cache)?;
    let current = view.memory_context_projection();
    let challenges = current_repo_model_claim_challenges(&cache, &current, &basis)?;
    let item = actionable_hands_frontier_item(&current, &challenges)
        .ok_or_else(|| anyhow!("current repo model has no eligible Hands frontier route"))?;
    if !safe_sorted_unique_paths(&item.source_scope) || item.source_scope.is_empty() {
        return Err(anyhow!(
            "Hands frontier route requires safe sorted source scope"
        ));
    }
    let item_hash = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(item)?));
    let route_seed = format!("{}:{}:{}", basis.projection_digest, item.id, item_hash);
    let route_id = format!(
        "repo-frontier-route-{:x}",
        Sha256::digest(route_seed.as_bytes())
    );
    let route = RepoFrontierRoute {
        schema_version: REPO_FRONTIER_ROUTE_SCHEMA_VERSION.to_string(),
        route_id: route_id.clone(),
        next_organ: RepoFrontierNextOrgan::Hands,
        model_projection_digest: basis.projection_digest.clone(),
        model_source_documents: basis.source_documents.clone(),
        frontier_item_id: item.id.clone(),
        frontier_item_hash: item_hash,
        migration_body: item.migration_body.clone(),
        question: item.question.clone(),
        gap: item.gap.clone(),
        target_claim_ids: item.target_claim_ids.clone(),
        source_scope: item
            .adopted_plan
            .as_ref()
            .map(|plan| plan.safe_paths.clone())
            .unwrap_or_else(|| item.source_scope.clone()),
        adopted_plan: item.adopted_plan.clone(),
        selected_at: at.to_string(),
        contract: REPO_FRONTIER_ROUTE_CONTRACT.to_string(),
    };
    if let Some(existing) = cache.get::<RepoFrontierRoute>(&route_id)? {
        let mut retry = route.clone();
        retry.selected_at = existing.selected_at.clone();
        return if existing == retry {
            Ok(existing)
        } else {
            Err(anyhow!(
                "repo frontier route deterministic identity collision"
            ))
        };
    }
    let (route_envelope, _) = cache.prepare_entry(&route_id, &route)?;
    let expected = keyed_repo_model_basis_envelopes(&cache, &basis)?;
    let mut writes = expected.clone();
    writes.push(route_envelope);
    if !backing.compare_and_swap_batch(&expected, writes)? {
        return Err(anyhow!(
            "repo frontier route lost current-model CAS or companion collision"
        ));
    }
    Ok(route)
}

fn actionable_hands_frontier_item<'a>(
    model: &'a crate::EpiphanyMemoryGraphSnapshot,
    challenges: &[RepoModelClaimChallenge],
) -> Option<&'a crate::RepoFrontierItem> {
    actionable_frontier_item_for_organ(model, challenges, "Hands", true)
}

fn actionable_frontier_item_for_organ<'a>(
    model: &'a crate::EpiphanyMemoryGraphSnapshot,
    challenges: &[RepoModelClaimChallenge],
    organ: &str,
    require_unchallenged_targets: bool,
) -> Option<&'a crate::RepoFrontierItem> {
    model.frontier.iter().find(|item| {
        frontier_item_is_actionable_for_organ(
            model,
            challenges,
            item,
            organ,
            require_unchallenged_targets,
        )
    })
}

fn frontier_item_is_actionable_for_organ(
    model: &crate::EpiphanyMemoryGraphSnapshot,
    challenges: &[RepoModelClaimChallenge],
    item: &crate::RepoFrontierItem,
    organ: &str,
    require_unchallenged_targets: bool,
) -> bool {
    let terminal = |status: crate::RepoFrontierStatus| {
        matches!(
            status,
            crate::RepoFrontierStatus::Resolved
                | crate::RepoFrontierStatus::Retired
                | crate::RepoFrontierStatus::Superseded
        )
    };
    item.status == crate::RepoFrontierStatus::Active
        && item.recommended_next_organ == organ
        && !item.source_scope.is_empty()
        && safe_sorted_unique_paths(&item.source_scope)
        && (!require_unchallenged_targets || frontier_target_claims_unchallenged(item, challenges))
        && item.dependency_item_ids.iter().all(|dependency_id| {
            model
                .frontier
                .iter()
                .find(|candidate| candidate.id == *dependency_id)
                .is_some_and(|dependency| terminal(dependency.status))
        })
}

/// Read-only Self signal. It is true only when the canonical runtime model is
/// admitted exactly once and contains an item the route committer can hand to
/// Hands. Status projection must use this instead of assuming that a clear
/// CRRC lane implies implementation authority.
pub fn runtime_has_actionable_hands_frontier(runtime_store: impl AsRef<Path>) -> Result<bool> {
    runtime_has_actionable_frontier_for_organ(runtime_store, "Hands")
}

/// Read-only Self signal for source gathering. It is true only when the
/// canonical runtime model is admitted exactly once and contains an Active,
/// dependency-ready Eyes frontier item.
pub fn runtime_has_actionable_eyes_frontier(runtime_store: impl AsRef<Path>) -> Result<bool> {
    runtime_has_actionable_frontier_for_organ(runtime_store, "Eyes")
}

pub fn select_and_commit_repo_frontier_research_request(
    runtime_store: impl AsRef<Path>,
    at: &str,
) -> Result<RepoFrontierResearchRequest> {
    chrono::DateTime::parse_from_rfc3339(at)
        .map_err(|_| anyhow!("research request timestamp must be RFC3339"))?;
    let runtime_store = runtime_store.as_ref();
    let backing = SingleFileMessagePackBackingStore::new(runtime_store);
    let _opening = backing.pull_all()?;
    let mut cache = runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    let identity = require_identity(&cache)?;
    let thread = cache
        .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
        .ok_or_else(|| anyhow!("frontier Research requires authoritative thread state"))?;
    thread.state()?;
    let (view, basis) = current_keyed_repo_model(&cache)?;
    let model = view.memory_context_projection();
    let challenges = current_repo_model_claim_challenges(&cache, &model, &basis)?;
    let launches = cache.get_all::<EpiphanyRuntimeWorkerLaunchRequest>()?;
    let packets = cache.get_all::<EyesEvidencePacket>()?;
    let item =
        match next_repo_frontier_research_work(&cache, &model, &challenges, &launches, &packets)? {
            Some(NextRepoFrontierResearchWork::Existing(request)) => return Ok(request),
            Some(NextRepoFrontierResearchWork::Unrequested(item)) => item,
            None => {
                return Err(anyhow!(
                    "current model has no uncovered actionable Eyes frontier"
                ));
            }
        };
    let request = repo_frontier_research_request_for_admitted_item(
        &identity.runtime_id,
        &thread.thread_id,
        &model,
        &basis,
        &item,
        at,
    )?;
    let request_id = request.request_id.clone();
    if let Some(existing) = cache.get::<RepoFrontierResearchRequest>(&request_id)? {
        let mut replay = request.clone();
        replay.requested_at = existing.requested_at.clone();
        // The frontier owns this request. The thread records where it was
        // created, but later coordinator incarnations must not fork or
        // invalidate the same frontier request.
        replay.thread_id = existing.thread_id.clone();
        return if existing == replay {
            Ok(existing)
        } else {
            Err(anyhow!("frontier Research request identity collision"))
        };
    }
    let (request_envelope, _) = cache.prepare_entry(&request_id, &request)?;
    let expected = keyed_repo_model_basis_envelopes(&cache, &basis)?;
    let mut writes = expected.clone();
    writes.push(request_envelope);
    if !backing.compare_and_swap_batch(&expected, writes)? {
        return Err(anyhow!("frontier Research request lost current-model CAS"));
    }
    Ok(request)
}

fn repo_frontier_item_hash(item: &crate::RepoFrontierItem) -> Result<String> {
    Ok(format!(
        "{:x}",
        Sha256::digest(rmp_serde::to_vec_named(item)?)
    ))
}

fn repo_frontier_research_request_for_admitted_item(
    runtime_id: &str,
    thread_id: &str,
    model: &crate::EpiphanyMemoryGraphSnapshot,
    basis: &crate::EpiphanyRepoModelBasis,
    item: &crate::RepoFrontierItem,
    requested_at: &str,
) -> Result<RepoFrontierResearchRequest> {
    if runtime_id.is_empty()
        || thread_id.is_empty()
        || basis.validate().is_err()
        || model
            .frontier
            .iter()
            .find(|candidate| candidate.id == item.id)
            != Some(item)
    {
        return Err(anyhow!(
            "frontier Research request requires its exact admitted frontier"
        ));
    }
    let item_hash = repo_frontier_item_hash(item)?;
    let public_source_refs = crate::ImmutableGithubSource::canonicalize_set(
        item.public_source_refs.iter().map(String::as_str),
    )?;
    if public_source_refs != item.public_source_refs {
        return Err(anyhow!(
            "frontier Research public source authority is not canonical"
        ));
    }
    Ok(RepoFrontierResearchRequest {
        schema_version: REPO_FRONTIER_RESEARCH_REQUEST_SCHEMA_VERSION.to_string(),
        request_id: crate::frontier_research_request_id(runtime_id, &item.id, &item_hash),
        model_projection_digest: basis.projection_digest.clone(),
        model_source_documents: basis.source_documents.clone(),
        frontier_item_id: item.id.clone(),
        frontier_item_hash: item_hash,
        source_scope: item.source_scope.clone(),
        requested_at: requested_at.to_string(),
        runtime_id: runtime_id.to_string(),
        thread_id: thread_id.to_string(),
        contract: REPO_FRONTIER_RESEARCH_REQUEST_CONTRACT.to_string(),
        public_source_refs,
    })
}

/// True only when the exact current Eyes frontier has not yet been covered by
/// an accepted Eyes packet from a worker launch bound to its typed request.
/// Historical Research acceptance is deliberately irrelevant.
pub fn runtime_has_uncovered_actionable_eyes_frontier(
    runtime_store: impl AsRef<Path>,
) -> Result<bool> {
    Ok(!matches!(
        runtime_repo_frontier_research_lifecycle(runtime_store)?.stage,
        RepoFrontierResearchLifecycleStage::Terminal
    ))
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RepoFrontierResearchLifecycleStage {
    Terminal,
    LaunchReady,
    WorkerRunning,
    ResultReady,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RepoFrontierResearchContinuationAction {
    LaunchResearch,
    ReviewResearchResult,
}

impl RepoFrontierResearchContinuationAction {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::LaunchResearch => "launchResearch",
            Self::ReviewResearchResult => "reviewResearchResult",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoFrontierResearchLifecycle {
    pub stage: RepoFrontierResearchLifecycleStage,
    pub frontier_item_id: Option<String>,
    pub request_id: Option<String>,
    pub worker_job_id: Option<String>,
}

impl RepoFrontierResearchLifecycle {
    pub fn continuation_action(&self) -> Option<RepoFrontierResearchContinuationAction> {
        match self.stage {
            RepoFrontierResearchLifecycleStage::LaunchReady => {
                Some(RepoFrontierResearchContinuationAction::LaunchResearch)
            }
            RepoFrontierResearchLifecycleStage::ResultReady => {
                Some(RepoFrontierResearchContinuationAction::ReviewResearchResult)
            }
            RepoFrontierResearchLifecycleStage::Terminal
            | RepoFrontierResearchLifecycleStage::WorkerRunning => None,
        }
    }
}

/// Projects the next exact actionable Eyes frontier through request, worker,
/// review, and accepted-packet lifecycle. This is the launch-currency owner:
/// an uncovered frontier is not launchable while its current attempt is still
/// running or awaiting review.
pub fn runtime_repo_frontier_research_lifecycle(
    runtime_store: impl AsRef<Path>,
) -> Result<RepoFrontierResearchLifecycle> {
    let runtime_store = runtime_store.as_ref();
    let mut cache = runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let launches = cache.get_all::<EpiphanyRuntimeWorkerLaunchRequest>()?;
    let packets = cache.get_all::<EyesEvidencePacket>()?;
    let jobs = cache.get_all::<EpiphanyRuntimeJob>()?;
    let job_results = cache.get_all::<EpiphanyRuntimeJobResult>()?;
    let role_results = cache.get_all::<EpiphanyRuntimeRoleWorkerResult>()?;
    let state = cache
        .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
        .map(|entry| entry.state())
        .transpose()?;
    let (view, basis) = current_keyed_repo_model(&cache)?;
    let model = view.memory_context_projection();
    let challenges = current_repo_model_claim_challenges(&cache, &model, &basis)?;

    let work = next_repo_frontier_research_work(&cache, &model, &challenges, &launches, &packets)?;
    let request = match work {
        Some(NextRepoFrontierResearchWork::Unrequested(item)) => {
            return Ok(RepoFrontierResearchLifecycle {
                stage: RepoFrontierResearchLifecycleStage::LaunchReady,
                frontier_item_id: Some(item.id.clone()),
                request_id: None,
                worker_job_id: None,
            });
        }
        Some(NextRepoFrontierResearchWork::Existing(request)) => request,
        None => {
            return Ok(RepoFrontierResearchLifecycle {
                stage: RepoFrontierResearchLifecycleStage::Terminal,
                frontier_item_id: None,
                request_id: None,
                worker_job_id: None,
            });
        }
    };
    let mut attempts = launches
        .iter()
        .filter(|launch| {
            launch.repo_frontier_research_request_id.as_deref() == Some(request.request_id.as_str())
        })
        .map(|launch| {
            let carried_request = frontier_research_request_for_launch(&cache, launch)?
                .ok_or_else(|| anyhow!("frontier Research launch lost its typed request"))?;
            if carried_request != request {
                return Err(anyhow!(
                    "frontier Research launch carries substituted request"
                ));
            }
            let job = jobs
                .iter()
                .find(|job| job.job_id == launch.job_id)
                .ok_or_else(|| anyhow!("frontier Research launch lost its runtime job"))?;
            Ok((launch, job))
        })
        .collect::<Result<Vec<_>>>()?;
    attempts.sort_by(|(_, left), (_, right)| {
        left.created_at
            .cmp(&right.created_at)
            .then_with(|| left.job_id.cmp(&right.job_id))
    });
    let Some((launch, job)) = attempts.last() else {
        return Ok(RepoFrontierResearchLifecycle {
            stage: RepoFrontierResearchLifecycleStage::LaunchReady,
            frontier_item_id: Some(request.frontier_item_id.clone()),
            request_id: Some(request.request_id.clone()),
            worker_job_id: None,
        });
    };
    let stage = match job.status {
        EpiphanyRuntimeJobStatus::Queued
        | EpiphanyRuntimeJobStatus::Running
        | EpiphanyRuntimeJobStatus::WaitingForReview => {
            RepoFrontierResearchLifecycleStage::WorkerRunning
        }
        EpiphanyRuntimeJobStatus::Completed
        | EpiphanyRuntimeJobStatus::Failed
        | EpiphanyRuntimeJobStatus::Cancelled => {
            let result_id = role_results
                .iter()
                .find(|result| result.job_id == job.job_id)
                .map(|result| result.result_id.as_str())
                .or_else(|| {
                    job_results
                        .iter()
                        .filter(|result| result.job_id == job.job_id)
                        .max_by(|left, right| {
                            left.completed_at
                                .cmp(&right.completed_at)
                                .then_with(|| left.result_id.cmp(&right.result_id))
                        })
                        .map(|result| result.result_id.as_str())
                })
                .ok_or_else(|| {
                    anyhow!("terminal frontier Research job lost its reviewable result")
                })?;
            let matching_receipts = state
                .as_ref()
                .map(|state| {
                    state
                        .acceptance_receipts
                        .iter()
                        .filter(|receipt| {
                            receipt.result_id == result_id
                                && receipt.job_id == job.job_id
                                && receipt.role_id == "research"
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if matching_receipts.len() > 1 {
                return Err(anyhow!(
                    "frontier Research result has multiple review authorities"
                ));
            }
            match matching_receipts.first() {
                Some(receipt)
                    if receipt.surface == "roleFailureReview" && receipt.status == "superseded" =>
                {
                    RepoFrontierResearchLifecycleStage::LaunchReady
                }
                Some(receipt)
                    if receipt.surface == "roleAccept" && receipt.status == "accepted" =>
                {
                    return Err(anyhow!(
                        "accepted frontier Research result lost its Eyes evidence packet"
                    ));
                }
                Some(_) => {
                    return Err(anyhow!(
                        "frontier Research result has conflicting review authority"
                    ));
                }
                None => RepoFrontierResearchLifecycleStage::ResultReady,
            }
        }
    };
    Ok(RepoFrontierResearchLifecycle {
        stage,
        frontier_item_id: Some(request.frontier_item_id.clone()),
        request_id: Some(request.request_id.clone()),
        worker_job_id: Some(launch.job_id.clone()),
    })
}

enum NextRepoFrontierResearchWork {
    Existing(RepoFrontierResearchRequest),
    Unrequested(crate::RepoFrontierItem),
}

fn next_repo_frontier_research_work(
    cache: &CultCache,
    model: &crate::EpiphanyMemoryGraphSnapshot,
    challenges: &[RepoModelClaimChallenge],
    launches: &[EpiphanyRuntimeWorkerLaunchRequest],
    packets: &[EyesEvidencePacket],
) -> Result<Option<NextRepoFrontierResearchWork>> {
    let existing_actionable = actionable_repo_frontier_research_requests(cache)?;
    let mut existing_uncovered = existing_actionable
        .iter()
        .cloned()
        .filter_map(|request| {
            match repo_frontier_research_request_is_covered(cache, &request, launches, packets) {
                Ok(false) => Some(Ok(request)),
                Ok(true) => None,
                Err(error) => Some(Err(error)),
            }
        })
        .collect::<Result<Vec<_>>>()?;
    existing_uncovered.sort_by(|left, right| {
        left.requested_at
            .cmp(&right.requested_at)
            .then_with(|| left.request_id.cmp(&right.request_id))
    });
    if let Some(request) = existing_uncovered.into_iter().next() {
        return Ok(Some(NextRepoFrontierResearchWork::Existing(request)));
    }
    for item in model.frontier.iter().filter(|item| {
        frontier_item_is_actionable_for_organ(model, challenges, item, "Eyes", false)
    }) {
        let item_hash = repo_frontier_item_hash(item)?;
        if !existing_actionable.iter().any(|request| {
            request.frontier_item_id == item.id && request.frontier_item_hash == item_hash
        }) {
            return Ok(Some(NextRepoFrontierResearchWork::Unrequested(
                item.clone(),
            )));
        }
    }
    Ok(None)
}

fn repo_frontier_research_request_is_covered(
    cache: &CultCache,
    request: &RepoFrontierResearchRequest,
    launches: &[EpiphanyRuntimeWorkerLaunchRequest],
    packets: &[EyesEvidencePacket],
) -> Result<bool> {
    let mut matching_jobs = BTreeSet::new();
    for launch in launches {
        if launch.repo_frontier_research_request_id.as_deref() != Some(&request.request_id) {
            continue;
        }
        let carried_request = frontier_research_request_for_launch(&cache, launch)?
            .ok_or_else(|| anyhow!("frontier Research launch lost its typed request"))?;
        if carried_request != *request {
            return Err(anyhow!(
                "frontier Research launch carries substituted request"
            ));
        }
        matching_jobs.insert(launch.job_id.as_str());
    }
    Ok(packets
        .iter()
        .any(|packet| matching_jobs.contains(packet.source_job_id.as_str())))
}

fn actionable_repo_frontier_research_requests(
    cache: &CultCache,
) -> Result<Vec<RepoFrontierResearchRequest>> {
    cache
        .get_all::<RepoFrontierResearchRequest>()?
        .into_iter()
        .filter_map(
            |request| match repo_frontier_research_request_is_actionable(cache, &request) {
                Ok(true) => Some(Ok(request)),
                Ok(false) => None,
                Err(error) => Some(Err(error)),
            },
        )
        .collect()
}

fn repo_frontier_research_request_is_actionable(
    cache: &CultCache,
    request: &RepoFrontierResearchRequest,
) -> Result<bool> {
    if request.schema_version != REPO_FRONTIER_RESEARCH_REQUEST_SCHEMA_VERSION
        || request.contract != REPO_FRONTIER_RESEARCH_REQUEST_CONTRACT
        || request.source_scope.is_empty()
        || !safe_sorted_unique_paths(&request.source_scope)
    {
        return Err(anyhow!("invalid frontier Research request"));
    }
    let identity = require_identity(cache)?;
    let view = match require_keyed_repo_model_basis(
        cache,
        &request.model_projection_digest,
        &request.model_source_documents,
    ) {
        Ok(view) => view,
        Err(_) => return Ok(false),
    };
    let model = view.memory_context_projection();
    let basis = view.reasoning_basis();
    if request.runtime_id != identity.runtime_id || request.thread_id.is_empty() {
        return Ok(false);
    }
    let challenges = current_repo_model_claim_challenges(cache, &model, &basis)?;
    let Some(item) = model
        .frontier
        .iter()
        .find(|item| item.id == request.frontier_item_id)
    else {
        return Ok(false);
    };
    if !frontier_item_is_actionable_for_organ(&model, &challenges, item, "Eyes", false) {
        return Ok(false);
    }
    let item_hash = repo_frontier_item_hash(item)?;
    let expected_public_source_refs = crate::ImmutableGithubSource::canonicalize_set(
        item.public_source_refs.iter().map(String::as_str),
    )?;
    if expected_public_source_refs != item.public_source_refs {
        return Err(anyhow!(
            "frontier Research public source authority is not canonical"
        ));
    }
    let expected_request_id =
        crate::frontier_research_request_id(&identity.runtime_id, &item.id, &item_hash);
    Ok(request.request_id == expected_request_id
        && request.frontier_item_id == item.id
        && request.frontier_item_hash == item_hash
        && request.source_scope == item.source_scope
        && request.public_source_refs == expected_public_source_refs)
}

/// Read-only Self signal for proposal planning. It uses the same eligibility
/// predicate as the planning-request committer, including unchallenged target
/// claims, so status cannot advertise authority that the commit path rejects.
pub fn runtime_has_actionable_imagination_frontier(
    runtime_store: impl AsRef<Path>,
) -> Result<bool> {
    let eligibility = runtime_repo_frontier_planning_eligibility(runtime_store)?;
    Ok(eligibility.current_model_count == 1
        && eligibility
            .candidates
            .iter()
            .any(|candidate| candidate.eligible))
}

/// Read-only Self signal explaining the exact canonical blockers on every
/// active Imagination frontier. It derives from the same admitted RepoModel,
/// current claim-challenge set, source-scope predicate, and dependency rule as
/// planning selection; it neither creates nor repairs authority.
pub fn runtime_repo_frontier_planning_eligibility(
    runtime_store: impl AsRef<Path>,
) -> Result<RepoFrontierPlanningEligibility> {
    let runtime_store = runtime_store.as_ref();
    let mut cache = runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let Ok((view, basis)) = current_keyed_repo_model(&cache) else {
        return Ok(RepoFrontierPlanningEligibility {
            current_model_count: 0,
            candidates: Vec::new(),
        });
    };
    let model = view.memory_context_projection();
    let model_count = 1;
    let challenges = current_repo_model_claim_challenges(&cache, &model, &basis)?;
    let terminal = |status: crate::RepoFrontierStatus| {
        matches!(
            status,
            crate::RepoFrontierStatus::Resolved
                | crate::RepoFrontierStatus::Retired
                | crate::RepoFrontierStatus::Superseded
        )
    };
    let mut candidates = model
        .frontier
        .iter()
        .filter(|item| {
            matches!(
                item.status,
                crate::RepoFrontierStatus::Active | crate::RepoFrontierStatus::Proposed
            ) && item
                .recommended_next_organ
                .eq_ignore_ascii_case("Imagination")
        })
        .map(|item| {
            let status_valid = item.status == crate::RepoFrontierStatus::Active;
            let recommended_next_organ_valid = item.recommended_next_organ == "Imagination";
            let source_scope_valid =
                !item.source_scope.is_empty() && safe_sorted_unique_paths(&item.source_scope);
            let mut challenged_target_claim_ids = challenges
                .iter()
                .filter(|challenge| item.target_claim_ids.contains(&challenge.target_claim_id))
                .map(|challenge| challenge.target_claim_id.clone())
                .collect::<Vec<_>>();
            challenged_target_claim_ids.sort();
            challenged_target_claim_ids.dedup();
            let mut unresolved_dependency_item_ids = item
                .dependency_item_ids
                .iter()
                .filter(|dependency_id| {
                    !model
                        .frontier
                        .iter()
                        .find(|candidate| candidate.id.as_str() == dependency_id.as_str())
                        .is_some_and(|dependency| terminal(dependency.status))
                })
                .cloned()
                .collect::<Vec<_>>();
            unresolved_dependency_item_ids.sort();
            unresolved_dependency_item_ids.dedup();
            RepoFrontierPlanningCandidateEligibility {
                frontier_item_id: item.id.clone(),
                eligible: model_count == 1
                    && status_valid
                    && recommended_next_organ_valid
                    && source_scope_valid
                    && challenged_target_claim_ids.is_empty()
                    && unresolved_dependency_item_ids.is_empty(),
                status_valid,
                recommended_next_organ_valid,
                source_scope_valid,
                challenged_target_claim_ids,
                unresolved_dependency_item_ids,
            }
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|a, b| a.frontier_item_id.cmp(&b.frontier_item_id));
    Ok(RepoFrontierPlanningEligibility {
        current_model_count: model_count,
        candidates,
    })
}

/// Read-only Self projection over the existing typed frontier-planning chain.
/// It never creates a request or decision; each mutating stage remains owned by
/// its established commit primitive.
pub fn runtime_repo_frontier_planning_lifecycle(
    runtime_store: impl AsRef<Path>,
) -> Result<RepoFrontierPlanningLifecycle> {
    let runtime_store = runtime_store.as_ref();
    let mut cache = runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let empty = |stage| RepoFrontierPlanningLifecycle {
        stage,
        planning_request_id: None,
        imagination_job_id: None,
        imagination_result_id: None,
        mind_request_id: None,
        mind_job_id: None,
        mind_result_id: None,
        decision_id: None,
    };
    let decisions = cache.get_all::<RepoFrontierPlanDecisionReceipt>()?;
    let mut active_requests = Vec::new();
    let mut terminal_current_requests = Vec::new();
    for request in cache.get_all::<RepoFrontierPlanningRequest>()? {
        if validate_actionable_repo_frontier_planning_request(&cache, &request).is_ok() {
            if let Some(decision) = decisions
                .iter()
                .find(|decision| decision.planning_request_id == request.request_id)
            {
                terminal_current_requests.push((request, decision));
            } else {
                active_requests.push(request);
            }
        }
    }
    active_requests.sort_by(|a, b| {
        a.requested_at
            .cmp(&b.requested_at)
            .then_with(|| a.request_id.cmp(&b.request_id))
    });
    if active_requests.len() > 1 {
        return Err(anyhow!(
            "Self found multiple nonterminal current frontier planning requests"
        ));
    }
    if terminal_current_requests.len() > 1 {
        return Err(anyhow!(
            "Self found multiple terminal decisions for current frontier planning authority"
        ));
    }
    let Some(request) = active_requests.pop() else {
        if let Some((request, decision)) = terminal_current_requests.pop() {
            return Ok(RepoFrontierPlanningLifecycle {
                decision_id: Some(decision.decision_id.clone()),
                planning_request_id: Some(request.request_id),
                ..empty(RepoFrontierPlanningLifecycleStage::Terminal)
            });
        }
        if runtime_has_actionable_imagination_frontier(runtime_store)? {
            return Ok(empty(RepoFrontierPlanningLifecycleStage::Ready));
        }
        let latest_terminal = decisions.iter().max_by(|a, b| {
            a.decided_at
                .cmp(&b.decided_at)
                .then_with(|| a.decision_id.cmp(&b.decision_id))
        });
        return Ok(if let Some(decision) = latest_terminal {
            RepoFrontierPlanningLifecycle {
                decision_id: Some(decision.decision_id.clone()),
                planning_request_id: Some(decision.planning_request_id.clone()),
                ..empty(RepoFrontierPlanningLifecycleStage::Terminal)
            }
        } else {
            empty(RepoFrontierPlanningLifecycleStage::Unavailable)
        });
    };
    let mut lifecycle = RepoFrontierPlanningLifecycle {
        stage: RepoFrontierPlanningLifecycleStage::ImaginationLaunchReady,
        planning_request_id: Some(request.request_id.clone()),
        imagination_job_id: None,
        imagination_result_id: None,
        mind_request_id: None,
        mind_job_id: None,
        mind_result_id: None,
        decision_id: None,
    };
    let mut imagination_launches = cache
        .get_all::<RepoFrontierPlanningLaunchBinding>()?
        .into_iter()
        .filter(|binding| binding.planning_request_id == request.request_id)
        .collect::<Vec<_>>();
    imagination_launches.sort_by_key(|binding| binding.attempt_ordinal);
    for (expected, binding) in imagination_launches.iter().enumerate() {
        if binding.attempt_ordinal != expected as u64 {
            return Err(anyhow!(
                "Self found noncontiguous frontier planning attempt identity"
            ));
        }
    }
    if let Some(binding) = imagination_launches.last() {
        lifecycle.imagination_job_id = Some(binding.job_id.clone());
    }
    let imagination_results = cache
        .get_all::<EpiphanyRuntimeRoleWorkerResult>()?
        .into_iter()
        .filter(|result| lifecycle.imagination_job_id.as_deref() == Some(result.job_id.as_str()))
        .collect::<Vec<_>>();
    if imagination_results.len() > 1 {
        return Err(anyhow!(
            "Self found multiple immutable Imagination results for one planning request"
        ));
    }
    let Some(imagination_result) = imagination_results.first() else {
        if lifecycle.imagination_job_id.is_some() {
            lifecycle.stage = RepoFrontierPlanningLifecycleStage::ImaginationRunning;
        }
        return Ok(lifecycle);
    };
    lifecycle.imagination_job_id = Some(imagination_result.job_id.clone());
    lifecycle.imagination_result_id = Some(imagination_result.result_id.clone());
    if imagination_result.frontier_planning_request_id.as_deref()
        != Some(request.request_id.as_str())
    {
        if !imagination_result
            .role_id
            .eq_ignore_ascii_case("imagination")
            || imagination_result
                .item_error
                .as_deref()
                .is_none_or(str::is_empty)
            || imagination_result.frontier_plan_candidate_msgpack.is_some()
        {
            return Err(anyhow!(
                "Self found malformed typed frontier planning failure"
            ));
        }
        let reviewed = cache
            .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
            .ok_or_else(|| anyhow!("planning failure review requires thread state"))?
            .state()?
            .acceptance_receipts
            .into_iter()
            .filter(|receipt| {
                receipt.result_id == imagination_result.result_id
                    && receipt.job_id == imagination_result.job_id
                    && receipt.binding_id == EPIPHANY_IMAGINATION_ROLE_BINDING_ID
                    && receipt.surface == "roleFailureReview"
                    && receipt.role_id == "imagination"
                    && receipt.status == "superseded"
            })
            .count();
        lifecycle.stage = if reviewed == 1 {
            RepoFrontierPlanningLifecycleStage::ImaginationLaunchReady
        } else if reviewed == 0 {
            RepoFrontierPlanningLifecycleStage::ImaginationFailed
        } else {
            return Err(anyhow!(
                "Self found conflicting frontier planning failure reviews"
            ));
        };
        return Ok(lifecycle);
    }
    let mind_requests = cache
        .get_all::<RepoFrontierPlanMindRequest>()?
        .into_iter()
        .filter(|mind| mind.imagination_result_id == imagination_result.result_id)
        .collect::<Vec<_>>();
    if mind_requests.len() > 1 {
        return Err(anyhow!(
            "Self found multiple Mind requests for one Imagination result"
        ));
    }
    let Some(mind_request) = mind_requests.first() else {
        lifecycle.stage = RepoFrontierPlanningLifecycleStage::ImaginationResultReady;
        return Ok(lifecycle);
    };
    validate_repo_frontier_plan_mind_request(&cache, mind_request)?;
    lifecycle.mind_request_id = Some(mind_request.request_id.clone());
    lifecycle.stage = RepoFrontierPlanningLifecycleStage::MindLaunchReady;
    let mut mind_launches = cache
        .get_all::<RepoFrontierPlanMindLaunchBinding>()?
        .into_iter()
        .filter(|binding| binding.mind_request_id == mind_request.request_id)
        .collect::<Vec<_>>();
    mind_launches.sort_by_key(|binding| binding.attempt_ordinal);
    for (expected, binding) in mind_launches.iter().enumerate() {
        if binding.attempt_ordinal != expected as u64 {
            return Err(anyhow!("Self found noncontiguous Mind attempt identity"));
        }
    }
    if let Some(binding) = mind_launches.last() {
        lifecycle.mind_job_id = Some(binding.job_id.clone());
    }
    let mind_results = cache
        .get_all::<EpiphanyRuntimeRoleWorkerResult>()?
        .into_iter()
        .filter(|result| lifecycle.mind_job_id.as_deref() == Some(result.job_id.as_str()))
        .collect::<Vec<_>>();
    if mind_results.len() > 1 {
        return Err(anyhow!(
            "Self found multiple immutable Mind results for one planning request"
        ));
    }
    let Some(mind_result) = mind_results.first() else {
        if lifecycle.mind_job_id.is_some() {
            lifecycle.stage = RepoFrontierPlanningLifecycleStage::MindRunning;
        }
        return Ok(lifecycle);
    };
    lifecycle.mind_job_id = Some(mind_result.job_id.clone());
    lifecycle.mind_result_id = Some(mind_result.result_id.clone());
    if mind_result.frontier_plan_mind_request_id.as_deref()
        != Some(mind_request.request_id.as_str())
    {
        if !mind_result
            .role_id
            .eq_ignore_ascii_case("mindAdmissionReview")
            || mind_result.item_error.as_deref().is_none_or(str::is_empty)
            || mind_result.frontier_plan_mind_decision_msgpack.is_some()
        {
            return Err(anyhow!("Self found malformed typed Mind failure"));
        }
        let reviewed = cache
            .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
            .ok_or_else(|| anyhow!("Mind failure review requires thread state"))?
            .state()?
            .acceptance_receipts
            .into_iter()
            .filter(|receipt| {
                receipt.result_id == mind_result.result_id
                    && receipt.job_id == mind_result.job_id
                    && receipt.binding_id == EPIPHANY_MIND_ROLE_BINDING_ID
                    && receipt.surface == "roleFailureReview"
                    && receipt.role_id == "mindAdmissionReview"
                    && receipt.status == "superseded"
            })
            .count();
        lifecycle.stage = if reviewed == 1 {
            RepoFrontierPlanningLifecycleStage::MindLaunchReady
        } else if reviewed == 0 {
            RepoFrontierPlanningLifecycleStage::MindFailed
        } else {
            return Err(anyhow!("Self found conflicting Mind failure reviews"));
        };
        return Ok(lifecycle);
    }
    lifecycle.stage = RepoFrontierPlanningLifecycleStage::MindResultReady;
    Ok(lifecycle)
}

fn runtime_has_actionable_frontier_for_organ(
    runtime_store: impl AsRef<Path>,
    organ: &str,
) -> Result<bool> {
    let runtime_store = runtime_store.as_ref();
    let mut cache = runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let Ok((view, basis)) = current_keyed_repo_model(&cache) else {
        return Ok(false);
    };
    let model = view.memory_context_projection();
    let challenges = current_repo_model_claim_challenges(&cache, &model, &basis)?;
    Ok(actionable_frontier_item_for_organ(&model, &challenges, organ, organ == "Hands").is_some())
}

pub fn put_runtime_reorient_worker_result(
    store_path: impl AsRef<Path>,
    result: &EpiphanyRuntimeReorientWorkerResult,
) -> Result<()> {
    validate_non_empty(&result.job_id, "reorient worker result job id")?;
    validate_non_empty(&result.result_id, "reorient worker result id")?;
    validate_non_empty(&result.mode, "reorient worker result mode")?;
    validate_non_empty(
        &result.decision_context_id,
        "reorient worker result decision context id",
    )?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_worker_decision_context(&cache, &result.decision_context_id, &result.job_id)?;
    cache.put(&result.job_id, result)?;
    Ok(())
}

fn require_worker_decision_context(
    cache: &cultcache_rs::CultCache,
    context_id: &str,
    worker_job_id: &str,
) -> Result<()> {
    #[cfg(test)]
    if context_id == "decision-context-fixture" {
        return Ok(());
    }
    let context = cache
        .get::<crate::EpiphanyDecisionContext>(context_id)?
        .ok_or_else(|| anyhow!("worker result decision context is absent"))?;
    if context.native_request()?.source_worker_job_id.as_deref() != Some(worker_job_id) {
        return Err(anyhow!(
            "worker result decision context belongs to another worker"
        ));
    }
    Ok(())
}

pub fn runtime_reorient_worker_result(
    store_path: impl AsRef<Path>,
    job_id: &str,
) -> Result<Option<EpiphanyRuntimeReorientWorkerResult>> {
    validate_non_empty(job_id, "reorient worker result job id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<EpiphanyRuntimeReorientWorkerResult>(job_id)
}

pub fn runtime_mind_gateway_review(
    store_path: impl AsRef<Path>,
    gateway_id: &str,
) -> Result<Option<MindGatewayReview>> {
    validate_non_empty(gateway_id, "Mind gateway review id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<MindGatewayReview>(gateway_id)
}

pub fn runtime_mind_state_commit_receipt(
    store_path: impl AsRef<Path>,
    receipt_id: &str,
) -> Result<Option<MindStateCommitReceipt>> {
    validate_non_empty(receipt_id, "Mind state commit receipt id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<MindStateCommitReceipt>(receipt_id)
}

// Eyes packets are accepted-research prerequisites and publish atomically with
// the coordinator state transition. Direct insertion is fixture-only.
#[cfg(test)]
pub(crate) fn put_eyes_evidence_packet(
    store_path: impl AsRef<Path>,
    packet: &EyesEvidencePacket,
) -> Result<()> {
    validate_non_empty(&packet.packet_id, "Eyes evidence packet id")?;
    validate_non_empty(
        &packet.source_result_id,
        "Eyes evidence packet source result",
    )?;
    validate_non_empty(&packet.source_job_id, "Eyes evidence packet source job")?;
    validate_non_empty(&packet.source_role_id, "Eyes evidence packet source role")?;
    validate_non_empty(&packet.emitted_at, "Eyes evidence packet timestamp")?;
    if packet.evidence_ids.is_empty() {
        return Err(anyhow!("Eyes evidence packet must reference evidence ids"));
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    cache.put(&packet.packet_id, packet)?;
    Ok(())
}

pub fn runtime_eyes_evidence_packet(
    store_path: impl AsRef<Path>,
    packet_id: &str,
) -> Result<Option<EyesEvidencePacket>> {
    validate_non_empty(packet_id, "Eyes evidence packet id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<EyesEvidencePacket>(packet_id)
}

pub fn put_substrate_gate_repo_access_grant_receipt(
    store_path: impl AsRef<Path>,
    receipt: &SubstrateGateRepoAccessGrantReceipt,
) -> Result<()> {
    validate_non_empty(
        &receipt.receipt_id,
        "Substrate Gate access grant receipt id",
    )?;
    validate_non_empty(
        &receipt.runtime_job_id,
        "Substrate Gate access grant runtime job",
    )?;
    validate_non_empty(&receipt.binding_id, "Substrate Gate access grant binding")?;
    validate_non_empty(&receipt.role, "Substrate Gate access grant role")?;
    validate_non_empty(
        &receipt.authority_scope,
        "Substrate Gate access grant authority scope",
    )?;
    validate_non_empty(&receipt.granted_at, "Substrate Gate access grant timestamp")?;
    if receipt.granted_operations.is_empty() {
        return Err(anyhow!(
            "Substrate Gate access grant must name granted operations"
        ));
    }
    if receipt.schema_version != SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION
        || chrono::DateTime::parse_from_rfc3339(&receipt.granted_at).is_err()
        || receipt.contract.trim().is_empty()
    {
        return Err(anyhow!("invalid Substrate Gate access grant contract"));
    }
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let (envelope, _) = cache.prepare_entry(&receipt.receipt_id, receipt)?;
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    if backing.compare_and_swap_batch(&[], vec![envelope])? {
        return Ok(());
    }
    let mut reloaded = runtime_spine_cache(store_path)?;
    reloaded.pull_all_backing_stores()?;
    match reloaded.get::<SubstrateGateRepoAccessGrantReceipt>(&receipt.receipt_id)? {
        Some(existing) if existing == *receipt => Ok(()),
        _ => Err(anyhow!("Substrate Gate grant ids are immutable")),
    }
}

pub fn runtime_substrate_gate_repo_access_grant_receipt(
    store_path: impl AsRef<Path>,
    receipt_id: &str,
) -> Result<Option<SubstrateGateRepoAccessGrantReceipt>> {
    validate_non_empty(receipt_id, "Substrate Gate access grant receipt id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<SubstrateGateRepoAccessGrantReceipt>(receipt_id)
}

pub fn put_hands_action_intent(
    store_path: impl AsRef<Path>,
    intent: &HandsActionIntent,
) -> Result<()> {
    validate_non_empty(&intent.intent_id, "Hands action intent id")?;
    validate_non_empty(&intent.runtime_job_id, "Hands action runtime job")?;
    validate_non_empty(&intent.binding_id, "Hands action binding")?;
    validate_non_empty(&intent.role, "Hands action role")?;
    validate_non_empty(&intent.authority_scope, "Hands action authority scope")?;
    validate_non_empty(&intent.requested_action, "Hands requested action")?;
    validate_non_empty(
        &intent.substrate_gate_grant_receipt_id,
        "Hands Substrate Gate grant receipt",
    )?;
    validate_non_empty(&intent.requested_at, "Hands action requested timestamp")?;
    if intent.requested_paths.is_empty() {
        return Err(anyhow!("Hands action intent must name requested paths"));
    }
    if intent.schema_version != HANDS_ACTION_INTENT_SCHEMA_VERSION
        || chrono::DateTime::parse_from_rfc3339(&intent.requested_at).is_err()
        || intent.contract.trim().is_empty()
    {
        return Err(anyhow!("invalid Hands action intent contract"));
    }
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let grant = cache
        .get::<SubstrateGateRepoAccessGrantReceipt>(&intent.substrate_gate_grant_receipt_id)?
        .ok_or_else(|| {
            anyhow!("Hands action intent requires its persisted Substrate Gate grant")
        })?;
    if grant.runtime_job_id != intent.runtime_job_id
        || grant.binding_id != intent.binding_id
        || grant.role != intent.role
        || grant.authority_scope != intent.authority_scope
        || !grant
            .granted_operations
            .iter()
            .any(|operation| operation == "read")
        || !intent.requested_paths.iter().all(|path| {
            grant.granted_paths.iter().any(|granted| {
                granted == "."
                    || path == granted
                    || path.starts_with(&format!("{}/", granted.trim_end_matches(['/', '\\'])))
            })
        })
    {
        return Err(anyhow!(
            "Hands action intent does not match its Substrate Gate grant scope"
        ));
    }
    let (envelope, _) = cache.prepare_entry(&intent.intent_id, intent)?;
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    if backing.compare_and_swap_batch(&[], vec![envelope])? {
        return Ok(());
    }
    let mut reloaded = runtime_spine_cache(store_path)?;
    reloaded.pull_all_backing_stores()?;
    match reloaded.get::<HandsActionIntent>(&intent.intent_id)? {
        Some(existing) if existing == *intent => Ok(()),
        _ => Err(anyhow!("Hands action intent ids are immutable")),
    }
}

pub fn runtime_hands_action_intent(
    store_path: impl AsRef<Path>,
    intent_id: &str,
) -> Result<Option<HandsActionIntent>> {
    validate_non_empty(intent_id, "Hands action intent id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<HandsActionIntent>(intent_id)
}

pub fn put_hands_action_review(
    store_path: impl AsRef<Path>,
    review: &HandsActionReview,
) -> Result<()> {
    validate_non_empty(&review.review_id, "Hands action review id")?;
    validate_non_empty(&review.intent_id, "Hands action review intent")?;
    validate_non_empty(&review.decision, "Hands action review decision")?;
    validate_non_empty(&review.reviewed_at, "Hands action review timestamp")?;
    if review.allowed_operations.is_empty() {
        return Err(anyhow!("Hands action review must name allowed operations"));
    }
    if review.schema_version != HANDS_ACTION_REVIEW_SCHEMA_VERSION
        || chrono::DateTime::parse_from_rfc3339(&review.reviewed_at).is_err()
        || review.contract.trim().is_empty()
    {
        return Err(anyhow!("invalid Hands action review contract"));
    }
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let (envelope, _) = cache.prepare_entry(&review.review_id, review)?;
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    if backing.compare_and_swap_batch(&[], vec![envelope])? {
        return Ok(());
    }
    let mut reloaded = runtime_spine_cache(store_path)?;
    reloaded.pull_all_backing_stores()?;
    match reloaded.get::<HandsActionReview>(&review.review_id)? {
        Some(existing) if existing == *review => Ok(()),
        _ => Err(anyhow!("Hands action review ids are immutable")),
    }
}

pub fn runtime_hands_action_review(
    store_path: impl AsRef<Path>,
    review_id: &str,
) -> Result<Option<HandsActionReview>> {
    validate_non_empty(review_id, "Hands action review id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<HandsActionReview>(review_id)
}

fn validate_repo_frontier_hands_authority_chain(
    cache: &CultCache,
    authority: &RepoFrontierHandsAuthority,
) -> Result<()> {
    let route = cache
        .get::<RepoFrontierRoute>(&authority.route_id)?
        .ok_or_else(|| anyhow!("Hands authority requires its persisted route"))?;
    let current = require_keyed_repo_model_basis(
        cache,
        &authority.model_projection_digest,
        &authority.model_source_documents,
    )?;
    let current_item = current
        .frontier
        .iter()
        .find(|item| item.id == route.frontier_item_id)
        .ok_or_else(|| anyhow!("Hands authority lost its model frontier"))?;
    let intent = cache
        .get::<HandsActionIntent>(&authority.hands_intent_id)?
        .ok_or_else(|| anyhow!("Hands authority requires its persisted intent"))?;
    let review = cache
        .get::<HandsActionReview>(&authority.hands_review_id)?
        .ok_or_else(|| anyhow!("Hands authority requires its persisted review"))?;
    let grant = cache
        .get::<SubstrateGateRepoAccessGrantReceipt>(&authority.substrate_grant_receipt_id)?
        .ok_or_else(|| anyhow!("Hands authority requires its persisted Substrate grant"))?;
    let within_scope = authority.requested_paths.iter().all(|path| {
        route.source_scope.iter().any(|scope| {
            path == scope || path.starts_with(&format!("{}/", scope.trim_end_matches(['/', '\\'])))
        })
    });
    let requested_operations: &[&str] = match intent.requested_action.as_str() {
        "patch" => &["patch"],
        "continueImplementation" => &["patch", "command", "commit"],
        _ => {
            return Err(anyhow!(
                "Hands authority names an unsupported requested action"
            ));
        }
    };
    let adopted_plan_binding_is_exact = match route.adopted_plan.as_ref() {
        Some(plan) => {
            intent.frontier_route_id == route.route_id
                && intent.plan_candidate_sha256 == plan.candidate_sha256
                && intent.plan_action == plan.effective_action()
        }
        None => {
            intent.frontier_route_id.is_empty()
                && intent.plan_candidate_sha256.is_empty()
                && intent.plan_action.is_empty()
        }
    };
    if route.schema_version != REPO_FRONTIER_ROUTE_SCHEMA_VERSION
        || route.contract != REPO_FRONTIER_ROUTE_CONTRACT
        || intent.schema_version != HANDS_ACTION_INTENT_SCHEMA_VERSION
        || review.schema_version != HANDS_ACTION_REVIEW_SCHEMA_VERSION
        || grant.schema_version != SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION
        || intent.contract.trim().is_empty()
        || review.contract.trim().is_empty()
        || grant.contract.trim().is_empty()
        || chrono::DateTime::parse_from_rfc3339(&intent.requested_at).is_err()
        || chrono::DateTime::parse_from_rfc3339(&review.reviewed_at).is_err()
        || chrono::DateTime::parse_from_rfc3339(&grant.granted_at).is_err()
        || route.next_organ != RepoFrontierNextOrgan::Hands
        || authority.model_projection_digest != route.model_projection_digest
        || authority.model_source_documents != route.model_source_documents
        || authority.frontier_item_id != route.frontier_item_id
        || authority.frontier_item_hash != route.frontier_item_hash
        || current_item.adopted_plan != route.adopted_plan
        || route
            .adopted_plan
            .as_ref()
            .is_some_and(|plan| route.source_scope != plan.safe_paths)
        || !adopted_plan_binding_is_exact
        || review.intent_id != intent.intent_id
        || review.decision != "approved"
        || !requested_operations.iter().all(|required| {
            review
                .allowed_operations
                .iter()
                .any(|operation| operation == required)
        })
        || intent.substrate_gate_grant_receipt_id != grant.receipt_id
        || grant.runtime_job_id != intent.runtime_job_id
        || grant.binding_id != intent.binding_id
        || grant.role != intent.role
        || grant.authority_scope != intent.authority_scope
        || !requested_operations.iter().all(|required| {
            grant
                .granted_operations
                .iter()
                .any(|operation| operation == required)
        })
        || authority.requested_paths != intent.requested_paths
        || authority.requested_paths != grant.granted_paths
        || !within_scope
    {
        return Err(anyhow!(
            "repo frontier Hands authority chain violates its full authority contract"
        ));
    }
    Ok(())
}

pub fn put_repo_frontier_hands_authority(
    store_path: impl AsRef<Path>,
    authority: &RepoFrontierHandsAuthority,
) -> Result<()> {
    let store_path = store_path.as_ref();
    if authority.schema_version != REPO_FRONTIER_HANDS_AUTHORITY_SCHEMA_VERSION
        || authority.contract != REPO_FRONTIER_HANDS_AUTHORITY_CONTRACT
        || chrono::DateTime::parse_from_rfc3339(&authority.granted_at).is_err()
        || !safe_sorted_unique_paths(&authority.requested_paths)
        || authority.requested_paths.is_empty()
    {
        return Err(anyhow!("invalid repo frontier Hands authority contract"));
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    validate_repo_frontier_hands_authority_chain(&cache, authority)?;
    let route = cache
        .get::<RepoFrontierRoute>(&authority.route_id)?
        .ok_or_else(|| anyhow!("repo frontier Hands authority requires its persisted route"))?;
    require_keyed_repo_model_basis(
        &cache,
        &authority.model_projection_digest,
        &authority.model_source_documents,
    )?;
    let intent = cache
        .get::<HandsActionIntent>(&authority.hands_intent_id)?
        .ok_or_else(|| anyhow!("repo frontier Hands authority requires its persisted intent"))?;
    let review = cache
        .get::<HandsActionReview>(&authority.hands_review_id)?
        .ok_or_else(|| anyhow!("repo frontier Hands authority requires its persisted review"))?;
    let grant = cache
        .get::<SubstrateGateRepoAccessGrantReceipt>(&authority.substrate_grant_receipt_id)?
        .ok_or_else(|| {
            anyhow!("repo frontier Hands authority requires its persisted Substrate grant")
        })?;
    let within_scope = authority.requested_paths.iter().all(|path| {
        route.source_scope.iter().any(|scope| {
            path == scope || path.starts_with(&format!("{}/", scope.trim_end_matches(['/', '\\'])))
        })
    });
    if route.schema_version != REPO_FRONTIER_ROUTE_SCHEMA_VERSION
        || route.contract != REPO_FRONTIER_ROUTE_CONTRACT
        || route.next_organ != RepoFrontierNextOrgan::Hands
        || authority.route_id != route.route_id
        || authority.model_projection_digest != route.model_projection_digest
        || authority.model_source_documents != route.model_source_documents
        || authority.frontier_item_id != route.frontier_item_id
        || authority.frontier_item_hash != route.frontier_item_hash
        || review.intent_id != intent.intent_id
        || review.decision != "approved"
        || intent.substrate_gate_grant_receipt_id != grant.receipt_id
        || grant.runtime_job_id != intent.runtime_job_id
        || grant.binding_id != intent.binding_id
        || grant.role != intent.role
        || grant.authority_scope != intent.authority_scope
        || authority.requested_paths != intent.requested_paths
        || authority.requested_paths != grant.granted_paths
        || !within_scope
    {
        return Err(anyhow!(
            "repo frontier Hands authority does not exactly bind route, model, intent, review, grant, and scope"
        ));
    }
    let (envelope, _) = cache.prepare_entry(&authority.authority_id, authority)?;
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    let basis = crate::EpiphanyRepoModelBasis {
        projection_digest: authority.model_projection_digest.clone(),
        source_documents: authority.model_source_documents.clone(),
    };
    let expected = keyed_repo_model_basis_envelopes(&cache, &basis)?;
    let mut writes = expected.clone();
    writes.push(envelope);
    if backing.compare_and_swap_batch(&expected, writes)? {
        return Ok(());
    }
    let mut reloaded = runtime_spine_cache(store_path)?;
    reloaded.pull_all_backing_stores()?;
    match reloaded.get::<RepoFrontierHandsAuthority>(&authority.authority_id)? {
        Some(existing) if existing == *authority => Ok(()),
        _ => Err(anyhow!("repo frontier Hands authority ids are immutable")),
    }
}

/// Atomically records Hands' inability to act under an exact route and lets
/// Mind retire that route's frontier. The historical route and authority stay
/// immutable, while the admitted model revision makes them structurally stale.
pub fn relinquish_repo_frontier_hands_route(
    store_path: impl AsRef<Path>,
    intent_id: &str,
    review_id: &str,
    refusal_receipt_id: &str,
    missing_required_paths: Vec<String>,
    summary: String,
    relinquished_at: String,
) -> Result<RepoFrontierRelinquishmentReceipt> {
    let store_path = store_path.as_ref();
    for (value, label) in [
        (intent_id, "Hands intent id"),
        (review_id, "Hands review id"),
        (refusal_receipt_id, "Hands refusal receipt id"),
        (&summary, "Hands refusal summary"),
    ] {
        validate_non_empty(value, label)?;
    }
    chrono::DateTime::parse_from_rfc3339(&relinquished_at)
        .map_err(|_| anyhow!("Hands refusal timestamp must be RFC3339"))?;
    if missing_required_paths.is_empty() || !safe_sorted_unique_paths(&missing_required_paths) {
        return Err(anyhow!(
            "Hands refusal paths must be non-empty and canonical"
        ));
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    if let Some(existing) = cache.get::<HandsActionRefusalReceipt>(refusal_receipt_id)? {
        let receipts = cache
            .get_all::<RepoFrontierRelinquishmentReceipt>()?
            .into_iter()
            .filter(|receipt| receipt.hands_refusal_receipt_id == existing.receipt_id)
            .collect::<Vec<_>>();
        return match receipts.as_slice() {
            [receipt]
                if existing.intent_id == intent_id
                    && existing.review_id == review_id
                    && existing.missing_required_paths == missing_required_paths
                    && existing.summary == summary
                    && existing.refused_at == relinquished_at =>
            {
                Ok(receipt.clone())
            }
            _ => Err(anyhow!(
                "Hands refusal replay is not the exact committed transition"
            )),
        };
    }
    let authorities = cache
        .get_all::<RepoFrontierHandsAuthority>()?
        .into_iter()
        .filter(|authority| {
            authority.hands_intent_id == intent_id && authority.hands_review_id == review_id
        })
        .collect::<Vec<_>>();
    let [authority] = authorities.as_slice() else {
        return Err(anyhow!("Hands refusal requires one exact route authority"));
    };
    validate_repo_frontier_hands_authority_chain(&cache, authority)?;
    let route = cache
        .get::<RepoFrontierRoute>(&authority.route_id)?
        .ok_or_else(|| anyhow!("Hands refusal lost its route"))?;
    if missing_required_paths.iter().all(|path| {
        route.source_scope.iter().any(|scope| {
            path == scope || path.starts_with(&format!("{}/", scope.trim_end_matches(['/', '\\'])))
        })
    }) {
        return Err(anyhow!(
            "Hands refusal names no missing path outside route scope"
        ));
    }
    if cache
        .get_all::<HandsPatchReceipt>()?
        .iter()
        .any(|receipt| receipt.intent_id == intent_id)
        || cache
            .get_all::<HandsCommandReceipt>()?
            .iter()
            .any(|receipt| receipt.intent_id == intent_id)
        || cache
            .get_all::<HandsCommitReceipt>()?
            .iter()
            .any(|receipt| receipt.intent_id == intent_id)
        || cache
            .get_all::<HandsPrReceipt>()?
            .iter()
            .any(|receipt| receipt.intent_id == intent_id)
    {
        return Err(anyhow!("Hands cannot relinquish after consequences exist"));
    }
    let basis = crate::EpiphanyRepoModelBasis {
        projection_digest: route.model_projection_digest.clone(),
        source_documents: route.model_source_documents.clone(),
    };
    let view =
        require_keyed_repo_model_basis(&cache, &basis.projection_digest, &basis.source_documents)?;
    let mut item = view
        .frontier
        .into_iter()
        .find(|item| item.id == route.frontier_item_id)
        .ok_or_else(|| anyhow!("Hands refusal frontier disappeared"))?;
    if repo_frontier_item_hash(&item)? != route.frontier_item_hash {
        return Err(anyhow!("Hands refusal frontier changed"));
    }
    item.status = crate::RepoFrontierStatus::Retired;
    item.retired_at = Some(relinquished_at.clone());
    let proposal = crate::EpiphanyRepoModelMutationProposal::new(
        format!("repo-frontier-relinquishment-{refusal_receipt_id}"),
        route.route_id.clone(),
        refusal_receipt_id.to_string(),
        vec![authority.authority_id.clone(), intent_id.to_string()],
        crate::load_current_runtime_repository_body_basis(store_path)?,
        vec![crate::EpiphanyRepoModelMutationOperation::PutFrontier { item }],
    )?;
    let plan = crate::plan_repo_model_mutation(store_path, &proposal)?;
    let admitted_basis = keyed_repo_model_basis_after_writes(&basis, &plan.writes)?;
    let refusal = HandsActionRefusalReceipt {
        schema_version: HANDS_ACTION_REFUSAL_RECEIPT_SCHEMA_VERSION.into(),
        receipt_id: refusal_receipt_id.to_string(),
        route_id: route.route_id.clone(),
        authority_id: authority.authority_id.clone(),
        intent_id: intent_id.to_string(),
        review_id: review_id.to_string(),
        substrate_gate_grant_receipt_id: authority.substrate_grant_receipt_id.clone(),
        model_projection_digest: basis.projection_digest.clone(),
        model_source_documents: basis.source_documents.clone(),
        frontier_item_id: route.frontier_item_id.clone(),
        frontier_item_hash: route.frontier_item_hash.clone(),
        missing_required_paths,
        summary,
        refused_at: relinquished_at.clone(),
        contract: HANDS_ACTION_REFUSAL_RECEIPT_CONTRACT.into(),
    };
    let receipt_id = format!(
        "repo-frontier-relinquishment-{:x}",
        Sha256::digest(format!("{}:{}", route.route_id, refusal.receipt_id).as_bytes())
    );
    let receipt = RepoFrontierRelinquishmentReceipt {
        schema_version: crate::REPO_FRONTIER_RELINQUISHMENT_RECEIPT_SCHEMA_VERSION.into(),
        receipt_id: receipt_id.clone(),
        hands_refusal_receipt_id: refusal.receipt_id.clone(),
        route_id: route.route_id,
        frontier_item_id: route.frontier_item_id,
        previous_model_projection_digest: basis.projection_digest,
        previous_model_source_documents: basis.source_documents,
        admitted_model_projection_digest: admitted_basis.projection_digest,
        admitted_model_source_documents: admitted_basis.source_documents,
        relinquished_at: relinquished_at.clone(),
        contract: crate::REPO_FRONTIER_RELINQUISHMENT_RECEIPT_CONTRACT.into(),
    };
    let provenance = cache.prepare_entry(&refusal.receipt_id, &refusal)?.0;
    let mut writes = plan.writes;
    writes.push(cache.prepare_entry(&receipt.receipt_id, &receipt)?.0);
    let outcome = crate::commit_typed_organ_mind_mutation(
        store_path,
        "Hands",
        provenance,
        "Mind.repo_frontier_relinquishment",
        plan.strong_reads,
        writes,
        &relinquished_at,
    )?;
    match outcome {
        crate::EpiphanyMindCommitOutcome::Committed(_) => Ok(receipt),
        crate::EpiphanyMindCommitOutcome::Conflict { .. } => {
            let mut reloaded = runtime_spine_cache(store_path)?;
            reloaded.pull_all_backing_stores()?;
            match reloaded.get::<RepoFrontierRelinquishmentReceipt>(&receipt_id)? {
                Some(existing) if existing == receipt => Ok(existing),
                _ => Err(anyhow!("Hands refusal lost its exact keyed-model CAS")),
            }
        }
    }
}

fn worker_result_has_keyed_mind_commit(
    cache: &CultCache,
    result: &EpiphanyRuntimeRoleWorkerResult,
) -> Result<bool> {
    require_worker_decision_context(cache, &result.decision_context_id, &result.job_id)?;
    Ok(cache
        .get_all::<crate::EpiphanyMindCommitReceipt>()?
        .into_iter()
        .any(|receipt| {
            matches!(
                receipt.authority,
                crate::EpiphanyMindCommitAuthority::ModelDecisionContext {
                    ref decision_context_id
                } if decision_context_id == &result.decision_context_id
            ) && receipt.invariant_owner.starts_with("Modeling.")
                && receipt
                    .writes
                    .iter()
                    .any(|write| write.document_type.starts_with("epiphany.mind.repo_model."))
        }))
}

/// Mind-owned repair for a route whose immutable adopted plan cannot bind the
/// already-observed consequence. The original plan and route remain intact;
/// only an authenticated, single-use execution amendment enters RepoModel.
pub fn amend_repo_frontier_execution(
    store_path: impl AsRef<Path>,
    amendment: crate::RepoFrontierExecutionAmendment,
) -> Result<RepoFrontierExecutionAmendmentReceipt> {
    let store_path = store_path.as_ref();
    chrono::DateTime::parse_from_rfc3339(&amendment.amended_at)
        .map_err(|_| anyhow!("execution amendment amended_at must be RFC3339"))?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    if let Some(existing) =
        cache.get::<RepoFrontierExecutionAmendmentReceipt>(&amendment.amendment_id)?
    {
        return Ok(existing);
    }
    let route = cache
        .get::<RepoFrontierRoute>(&amendment.replaces_route_id)?
        .ok_or_else(|| anyhow!("execution amendment requires the exact route"))?;
    let provenance = cache
        .get_envelope::<crate::LocalAdmittedOperatorCommand>(&amendment.command_id)?
        .ok_or_else(|| anyhow!("execution amendment lost its admitted operator command"))?;
    let admitted = cache
        .get::<crate::LocalAdmittedOperatorCommand>(&amendment.command_id)?
        .ok_or_else(|| anyhow!("execution amendment lost operator provenance"))?;
    if admitted.admission_id != amendment.admission_id
        || admitted.packet_sha256 != amendment.packet_sha256
        || admitted.source_actor_id != amendment.source_actor_id
    {
        return Err(anyhow!("execution amendment operator provenance mismatch"));
    }
    let basis = crate::EpiphanyRepoModelBasis {
        projection_digest: route.model_projection_digest.clone(),
        source_documents: route.model_source_documents.clone(),
    };
    let view =
        require_keyed_repo_model_basis(&cache, &basis.projection_digest, &basis.source_documents)?;
    let mut item = view
        .frontier
        .into_iter()
        .find(|item| item.id == route.frontier_item_id)
        .ok_or_else(|| anyhow!("execution amendment frontier disappeared"))?;
    let previous_frontier_item_hash = repo_frontier_item_hash(&item)?;
    if previous_frontier_item_hash != route.frontier_item_hash {
        return Err(anyhow!("execution amendment route is stale"));
    }
    let plan = item
        .adopted_plan
        .as_mut()
        .ok_or_else(|| anyhow!("execution amendment requires an adopted plan"))?;
    if plan.execution_amendment.is_some() {
        return Err(anyhow!(
            "execution amendment cannot replace another amendment"
        ));
    }
    plan.execution_amendment = Some(amendment.clone());
    let proposal = crate::EpiphanyRepoModelMutationProposal::new(
        format!(
            "repo-frontier-execution-amendment-{}",
            amendment.amendment_id
        ),
        amendment.command_id.clone(),
        amendment.amendment_id.clone(),
        vec![
            amendment.admission_id.clone(),
            amendment.packet_sha256.clone(),
        ],
        crate::load_current_runtime_repository_body_basis(store_path)?,
        vec![crate::EpiphanyRepoModelMutationOperation::PutFrontier { item }],
    )?;
    let mutation = crate::plan_repo_model_mutation(store_path, &proposal)?;
    let admitted_basis = keyed_repo_model_basis_after_writes(&basis, &mutation.writes)?;
    let receipt = RepoFrontierExecutionAmendmentReceipt {
        schema_version: REPO_FRONTIER_EXECUTION_AMENDMENT_RECEIPT_SCHEMA_VERSION.into(),
        receipt_id: amendment.amendment_id.clone(),
        amendment_id: amendment.amendment_id.clone(),
        replaced_route_id: route.route_id,
        frontier_item_id: route.frontier_item_id,
        previous_frontier_item_hash,
        previous_model_projection_digest: basis.projection_digest,
        previous_model_source_documents: basis.source_documents,
        admitted_model_projection_digest: admitted_basis.projection_digest,
        admitted_model_source_documents: admitted_basis.source_documents,
        source_actor_id: amendment.source_actor_id,
        command_id: amendment.command_id,
        admission_id: amendment.admission_id,
        packet_sha256: amendment.packet_sha256,
        replacement_action: amendment.action,
        replacement_command: amendment.command,
        rationale: amendment.rationale,
        amended_at: amendment.amended_at.clone(),
        contract: REPO_FRONTIER_EXECUTION_AMENDMENT_RECEIPT_CONTRACT.into(),
    };
    let mut writes = mutation.writes;
    writes.push(cache.prepare_entry(&receipt.receipt_id, &receipt)?.0);
    match crate::commit_operator_mind_mutation(
        store_path,
        provenance,
        "Mind.repo_frontier_execution_amendment",
        mutation.strong_reads,
        writes,
        &amendment.amended_at,
    )? {
        crate::EpiphanyMindCommitOutcome::Committed(_) => Ok(receipt),
        crate::EpiphanyMindCommitOutcome::Conflict { .. } => {
            let mut reloaded = runtime_spine_cache(store_path)?;
            reloaded.pull_all_backing_stores()?;
            match reloaded.get::<RepoFrontierExecutionAmendmentReceipt>(&receipt.receipt_id)? {
                Some(existing) if existing == receipt => Ok(existing),
                _ => Err(anyhow!(
                    "execution amendment lost its exact keyed-model CAS"
                )),
            }
        }
    }
}

pub fn put_repo_frontier_verification_request(
    store_path: impl AsRef<Path>,
    request: &RepoFrontierVerificationRequest,
) -> Result<()> {
    let store_path = store_path.as_ref();
    if request.schema_version != REPO_FRONTIER_VERIFICATION_REQUEST_SCHEMA_VERSION
        || request.contract != REPO_FRONTIER_VERIFICATION_REQUEST_CONTRACT
        || chrono::DateTime::parse_from_rfc3339(&request.requested_at).is_err()
        || request.request_id.trim().is_empty()
    {
        return Err(anyhow!(
            "invalid repo frontier verification request contract"
        ));
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let route = cache
        .get::<RepoFrontierRoute>(&request.route_id)?
        .ok_or_else(|| anyhow!("verification request requires its exact frontier route"))?;
    require_keyed_repo_model_basis(
        &cache,
        &request.model_projection_digest,
        &request.model_source_documents,
    )?;
    let authorities = cache
        .get_all::<RepoFrontierHandsAuthority>()?
        .into_iter()
        .filter(|value| {
            value.route_id == route.route_id && value.hands_intent_id == request.hands_intent_id
        })
        .collect::<Vec<_>>();
    if authorities.len() != 1 {
        return Err(anyhow!(
            "verification request requires exactly one Hands authority"
        ));
    }
    let authority = &authorities[0];
    validate_repo_frontier_hands_authority_chain(&cache, authority)?;
    let intent = cache
        .get::<HandsActionIntent>(&request.hands_intent_id)?
        .ok_or_else(|| anyhow!("verification request requires its Hands intent"))?;
    let review = cache
        .get::<HandsActionReview>(&request.hands_review_id)?
        .ok_or_else(|| anyhow!("verification request requires its Hands review"))?;
    let patch = cache
        .get::<HandsPatchReceipt>(&request.hands_patch_receipt_id)?
        .ok_or_else(|| anyhow!("verification request requires its exact patch receipt"))?;
    let command = cache
        .get::<HandsCommandReceipt>(&request.hands_command_receipt_id)?
        .ok_or_else(|| anyhow!("verification request requires its exact command receipt"))?;
    let commit = cache
        .get::<HandsCommitReceipt>(&request.hands_commit_receipt_id)?
        .ok_or_else(|| anyhow!("verification request requires its exact commit receipt"))?;
    let adopted_plan_mismatches = route.adopted_plan.as_ref().map_or_else(Vec::new, |plan| {
        let mut mismatches = Vec::new();
        if intent.frontier_route_id != route.route_id {
            mismatches.push("intent.plan.route");
        }
        if intent.plan_candidate_sha256 != plan.candidate_sha256 {
            mismatches.push("intent.plan.candidate");
        }
        if intent.plan_action != plan.effective_action() {
            mismatches.push("intent.plan.action");
        }
        if command.command != plan.effective_command() {
            mismatches.push("command.plan.command");
        }
        mismatches
    });
    if request.model_projection_digest != route.model_projection_digest
        || request.model_source_documents != route.model_source_documents
        || request.frontier_item_id != route.frontier_item_id
        || request.frontier_item_hash != route.frontier_item_hash
        || authority.hands_review_id != request.hands_review_id
        || authority.model_projection_digest != request.model_projection_digest
        || authority.model_source_documents != request.model_source_documents
        || authority.frontier_item_id != request.frontier_item_id
        || authority.frontier_item_hash != request.frontier_item_hash
        || review.intent_id != intent.intent_id
        || review.decision != "approved"
        || patch.intent_id != intent.intent_id
        || patch.review_id != review.review_id
        || patch.substrate_gate_grant_receipt_id != authority.substrate_grant_receipt_id
        || command.intent_id != intent.intent_id
        || command.review_id != review.review_id
        || command.substrate_gate_grant_receipt_id != authority.substrate_grant_receipt_id
        || commit.intent_id != intent.intent_id
        || commit.review_id != review.review_id
        || patch.runtime_job_id != intent.runtime_job_id
        || command.runtime_job_id != intent.runtime_job_id
        || commit.runtime_job_id != intent.runtime_job_id
        || patch.changed_paths != commit.changed_paths
        || !consequence_paths_within_authority(&patch.changed_paths, &authority.requested_paths)
        || !adopted_plan_mismatches.is_empty()
    {
        return Err(anyhow!(
            "verification request does not exactly bind route, model, Hands authority, and complete receipts{}",
            if adopted_plan_mismatches.is_empty() {
                String::new()
            } else {
                format!("; mismatches: {}", adopted_plan_mismatches.join(", "))
            }
        ));
    }
    let (envelope, _) = cache.prepare_entry(&request.request_id, request)?;
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    let basis = crate::EpiphanyRepoModelBasis {
        projection_digest: request.model_projection_digest.clone(),
        source_documents: request.model_source_documents.clone(),
    };
    let expected = keyed_repo_model_basis_envelopes(&cache, &basis)?;
    let mut writes = expected.clone();
    writes.push(envelope);
    if backing.compare_and_swap_batch(&expected, writes)? {
        return Ok(());
    }
    let mut reloaded = runtime_spine_cache(store_path)?;
    reloaded.pull_all_backing_stores()?;
    match reloaded.get::<RepoFrontierVerificationRequest>(&request.request_id)? {
        Some(existing) if existing == *request => Ok(()),
        _ => Err(anyhow!("verification request ids are immutable")),
    }
}

pub fn runtime_repo_frontier_verification_request(
    store_path: impl AsRef<Path>,
    request_id: &str,
) -> Result<Option<RepoFrontierVerificationRequest>> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<RepoFrontierVerificationRequest>(request_id)
}

pub fn runtime_repo_frontier_route(
    store_path: impl AsRef<Path>,
    route_id: &str,
) -> Result<Option<RepoFrontierRoute>> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<RepoFrontierRoute>(route_id)
}

pub fn runtime_repo_frontier_plan_decision(
    runtime_store: impl AsRef<Path>,
    decision_id: &str,
) -> Result<Option<RepoFrontierPlanDecisionReceipt>> {
    let mut cache = runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    Ok(cache
        .get_all::<RepoFrontierPlanDecisionReceipt>()?
        .into_iter()
        .find(|receipt| receipt.decision_id == decision_id))
}

pub fn runtime_repo_frontier_execution_amendment(
    store_path: impl AsRef<Path>,
    receipt_id: &str,
) -> Result<Option<RepoFrontierExecutionAmendmentReceipt>> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<RepoFrontierExecutionAmendmentReceipt>(receipt_id)
}

pub fn runtime_latest_repo_frontier_relinquishment(
    store_path: impl AsRef<Path>,
) -> Result<Option<RepoFrontierRelinquishmentReceipt>> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut receipts = cache.get_all::<RepoFrontierRelinquishmentReceipt>()?;
    receipts.sort_by(|left, right| {
        left.relinquished_at
            .cmp(&right.relinquished_at)
            .then_with(|| left.receipt_id.cmp(&right.receipt_id))
    });
    Ok(receipts.pop())
}

pub fn commit_repo_frontier_modeling_request(
    store_path: impl AsRef<Path>,
    acceptance: &epiphany_state_model::EpiphanyAcceptanceReceipt,
) -> Result<RepoFrontierModelingRequest> {
    if acceptance.role_id != "verification"
        || acceptance.surface != "roleAccept"
        || acceptance.status != "accepted"
        || acceptance.result_id.trim().is_empty()
        || acceptance.job_id.trim().is_empty()
        || chrono::DateTime::parse_from_rfc3339(&acceptance.accepted_at).is_err()
    {
        return Err(anyhow!(
            "frontier Modeling request requires one accepted Verification receipt"
        ));
    }
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let state = cache
        .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
        .ok_or_else(|| anyhow!("frontier Modeling request requires persisted coordinator state"))?
        .state()?;
    let persisted_acceptances = state
        .acceptance_receipts
        .iter()
        .filter(|candidate| candidate.id == acceptance.id)
        .collect::<Vec<_>>();
    if persisted_acceptances.len() != 1 || persisted_acceptances[0] != acceptance {
        return Err(anyhow!(
            "frontier Modeling request requires exactly one byte-exact persisted acceptance receipt"
        ));
    }
    let acceptance = persisted_acceptances[0];
    let results = cache
        .get_all::<EpiphanyRuntimeRoleWorkerResult>()?
        .into_iter()
        .filter(|result| result.result_id == acceptance.result_id)
        .collect::<Vec<_>>();
    if results.len() != 1 {
        return Err(anyhow!(
            "frontier Modeling request requires one immutable accepted Verification result"
        ));
    }
    let result = &results[0];
    let verdicts = cache
        .get_all::<SoulVerdictReceipt>()?
        .into_iter()
        .filter(|verdict| {
            verdict.source_result_id == acceptance.result_id
                && verdict.source_job_id == acceptance.job_id
        })
        .collect::<Vec<_>>();
    if verdicts.len() != 1 {
        return Err(anyhow!(
            "frontier Modeling request requires exactly one Soul verdict for the accepted result"
        ));
    }
    let verdict = &verdicts[0];
    let verification_request = cache
        .get::<RepoFrontierVerificationRequest>(&verdict.verification_request_id)?
        .ok_or_else(|| anyhow!("frontier Modeling request requires the exact Soul request"))?;
    let route = cache
        .get::<RepoFrontierRoute>(&verdict.frontier_route_id)?
        .ok_or_else(|| anyhow!("frontier Modeling request requires the exact frontier route"))?;
    let view = require_keyed_repo_model_basis(
        &cache,
        &route.model_projection_digest,
        &route.model_source_documents,
    )?;
    let item = view
        .frontier
        .iter()
        .find(|item| item.id == route.frontier_item_id)
        .ok_or_else(|| anyhow!("frontier Modeling request routed item is missing"))?;
    let item_hash = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(item)?));
    let mut result_evidence = result.evidence_ids.clone();
    let mut verdict_evidence = verdict.evidence_ids.clone();
    result_evidence.sort();
    result_evidence.dedup();
    verdict_evidence.sort();
    verdict_evidence.dedup();
    let disposition = match verdict.verdict.trim().to_ascii_lowercase().as_str() {
        "pass" => RepoFrontierVerdictDisposition::Resolved,
        "needs-review" | "needs-evidence" | "fail" => RepoFrontierVerdictDisposition::Blocked,
        _ => return Err(anyhow!("Soul verdict has no allowed frontier disposition")),
    };
    if result.schema_version != RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION
        || !result.role_id.eq_ignore_ascii_case("verification")
        || result.item_error.is_some()
        || result.job_id != acceptance.job_id
        || result.verification_request_id.as_deref()
            != Some(verification_request.request_id.as_str())
        || result.frontier_route_id.as_deref() != Some(route.route_id.as_str())
        || verdict.schema_version != SOUL_VERDICT_RECEIPT_SCHEMA_VERSION
        || verdict.verdict != result.verdict
        || verdict.summary != result.summary
        || verdict.risks != result.risks
        || verdict_evidence != result_evidence
        || verification_request.schema_version != REPO_FRONTIER_VERIFICATION_REQUEST_SCHEMA_VERSION
        || verification_request.contract != REPO_FRONTIER_VERIFICATION_REQUEST_CONTRACT
        || verification_request.route_id != route.route_id
        || verification_request.model_projection_digest != route.model_projection_digest
        || verification_request.model_source_documents != route.model_source_documents
        || verification_request.frontier_item_id != route.frontier_item_id
        || verification_request.frontier_item_hash != route.frontier_item_hash
        || item_hash != route.frontier_item_hash
        || item.status != crate::RepoFrontierStatus::Active
    {
        return Err(anyhow!(
            "frontier Modeling request does not exactly bind accepted result, Soul verdict, request, route, item, and current model"
        ));
    }
    let request_id = format!(
        "frontier-modeling-{:x}",
        Sha256::digest(
            format!(
                "{}:{}:{}:{}",
                acceptance.id, result.result_id, verdict.receipt_id, route.route_id
            )
            .as_bytes()
        )
    );
    let request = RepoFrontierModelingRequest {
        schema_version: REPO_FRONTIER_MODELING_REQUEST_SCHEMA_VERSION.to_string(),
        request_id: request_id.clone(),
        model_projection_digest: route.model_projection_digest.clone(),
        model_source_documents: route.model_source_documents.clone(),
        route_id: route.route_id.clone(),
        frontier_item_id: route.frontier_item_id.clone(),
        frontier_item_hash: route.frontier_item_hash.clone(),
        verification_request_id: verification_request.request_id.clone(),
        soul_verdict_receipt_id: verdict.receipt_id.clone(),
        verification_result_id: result.result_id.clone(),
        verification_job_id: result.job_id.clone(),
        verification_acceptance_receipt_id: acceptance.id.clone(),
        allowed_disposition: disposition,
        requested_at: acceptance.accepted_at.clone(),
        contract: REPO_FRONTIER_MODELING_REQUEST_CONTRACT.to_string(),
    };
    let (envelope, _) = cache.prepare_entry(&request_id, &request)?;
    let backing = SingleFileMessagePackBackingStore::new(store_path.as_ref());
    let basis = crate::EpiphanyRepoModelBasis {
        projection_digest: request.model_projection_digest.clone(),
        source_documents: request.model_source_documents.clone(),
    };
    let expected = keyed_repo_model_basis_envelopes(&cache, &basis)?;
    let mut writes = expected.clone();
    writes.push(envelope);
    if backing.compare_and_swap_batch(&expected, writes)? {
        return Ok(request);
    }
    let mut reloaded = runtime_spine_cache(store_path)?;
    reloaded.pull_all_backing_stores()?;
    match reloaded.get::<RepoFrontierModelingRequest>(&request_id)? {
        Some(existing) if existing == request => Ok(existing),
        _ => Err(anyhow!(
            "frontier Modeling request deterministic identity collision"
        )),
    }
}

fn consequence_paths_within_authority(
    changed_paths: &[String],
    authority_paths: &[String],
) -> bool {
    !changed_paths.is_empty()
        && changed_paths.iter().all(|path| {
            authority_paths.iter().any(|scope| {
                path == scope
                    || path.starts_with(&format!("{}/", scope.trim_end_matches(['/', '\\'])))
            })
        })
}

pub fn commit_repo_frontier_verification_request_for_chain(
    store_path: impl AsRef<Path>,
    chain: &RuntimeHandsReceiptChainSummary,
    requested_at: &str,
) -> Result<RepoFrontierVerificationRequest> {
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    let authorities = cache
        .get_all::<RepoFrontierHandsAuthority>()?
        .into_iter()
        .filter(|value| {
            value.hands_intent_id == chain.intent_id && value.hands_review_id == chain.review_id
        })
        .collect::<Vec<_>>();
    if authorities.len() != 1 {
        return Err(anyhow!(
            "complete Hands chain requires exactly one frontier authority before Soul launch"
        ));
    }
    let authority = &authorities[0];
    let request_id = format!(
        "frontier-verification-{}-{}",
        authority.route_id, chain.commit_receipt_id
    );
    let requested_at = cache
        .get::<RepoFrontierVerificationRequest>(&request_id)?
        .map(|existing| existing.requested_at)
        .unwrap_or_else(|| requested_at.to_string());
    let request = RepoFrontierVerificationRequest {
        schema_version: REPO_FRONTIER_VERIFICATION_REQUEST_SCHEMA_VERSION.to_string(),
        request_id,
        route_id: authority.route_id.clone(),
        model_projection_digest: authority.model_projection_digest.clone(),
        model_source_documents: authority.model_source_documents.clone(),
        frontier_item_id: authority.frontier_item_id.clone(),
        frontier_item_hash: authority.frontier_item_hash.clone(),
        hands_intent_id: chain.intent_id.clone(),
        hands_review_id: chain.review_id.clone(),
        hands_patch_receipt_id: chain.patch_receipt_id.clone(),
        hands_command_receipt_id: chain.command_receipt_id.clone(),
        hands_commit_receipt_id: chain.commit_receipt_id.clone(),
        requested_at,
        contract: REPO_FRONTIER_VERIFICATION_REQUEST_CONTRACT.to_string(),
    };
    put_repo_frontier_verification_request(store_path, &request)?;
    Ok(request)
}

fn safe_sorted_unique_paths(paths: &[String]) -> bool {
    paths.windows(2).all(|pair| pair[0] < pair[1])
        && paths.iter().all(|path| {
            !path.is_empty()
                && !Path::new(path).is_absolute()
                && !Path::new(path)
                    .components()
                    .any(|part| matches!(part, std::path::Component::ParentDir))
        })
}

fn validate_hands_consequence_grant(
    store_path: &Path,
    intent_id: &str,
    review_id: &str,
    runtime_job_id: &str,
    operation: &str,
    changed_paths: &[String],
    stated_grant_id: Option<&str>,
    stated_command: Option<&str>,
) -> Result<()> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let intent = cache
        .get::<HandsActionIntent>(intent_id)?
        .ok_or_else(|| anyhow!("Hands consequence requires its persisted intent"))?;
    let review = cache
        .get::<HandsActionReview>(review_id)?
        .ok_or_else(|| anyhow!("Hands consequence requires its persisted review"))?;
    let grant = cache
        .get::<SubstrateGateRepoAccessGrantReceipt>(&intent.substrate_gate_grant_receipt_id)?
        .ok_or_else(|| anyhow!("Hands consequence requires its persisted Substrate Gate grant"))?;
    let authorities = cache
        .get_all::<RepoFrontierHandsAuthority>()?
        .into_iter()
        .filter(|authority| authority.hands_intent_id == intent.intent_id)
        .collect::<Vec<_>>();
    if authorities.len() != 1 {
        return Err(anyhow!(
            "Hands consequence requires exactly one repo frontier authority for its intent"
        ));
    }
    let authority = &authorities[0];
    let route = cache
        .get::<RepoFrontierRoute>(&authority.route_id)?
        .ok_or_else(|| anyhow!("Hands consequence requires its persisted repo frontier route"))?;
    require_keyed_repo_model_basis(
        &cache,
        &authority.model_projection_digest,
        &authority.model_source_documents,
    )?;
    let paths_covered = changed_paths.iter().all(|path| {
        grant.granted_paths.iter().any(|granted| {
            granted == "."
                || path == granted
                || path.starts_with(&format!("{}/", granted.trim_end_matches(['/', '\\'])))
        })
    });
    if intent.runtime_job_id != runtime_job_id
        || review.intent_id != intent.intent_id
        || review.decision != "approved"
        || !review
            .allowed_operations
            .iter()
            .any(|allowed| allowed == operation)
        || grant.runtime_job_id != intent.runtime_job_id
        || grant.binding_id != intent.binding_id
        || grant.role != intent.role
        || grant.authority_scope != intent.authority_scope
        || !grant
            .granted_operations
            .iter()
            .any(|allowed| allowed == operation)
        || stated_grant_id.is_some_and(|id| id != grant.receipt_id)
        || stated_command.is_some_and(|command| {
            route
                .adopted_plan
                .as_ref()
                .is_some_and(|plan| command != plan.effective_command())
        })
        || !paths_covered
        || authority.schema_version != REPO_FRONTIER_HANDS_AUTHORITY_SCHEMA_VERSION
        || authority.contract != REPO_FRONTIER_HANDS_AUTHORITY_CONTRACT
        || authority.hands_review_id != review.review_id
        || authority.substrate_grant_receipt_id != grant.receipt_id
        || authority.requested_paths != intent.requested_paths
        || authority.route_id != route.route_id
        || authority.model_projection_digest != route.model_projection_digest
        || authority.model_source_documents != route.model_source_documents
        || authority.frontier_item_id != route.frontier_item_id
        || authority.frontier_item_hash != route.frontier_item_hash
        || !changed_paths.iter().all(|path| {
            authority.requested_paths.iter().any(|scope| {
                path == scope
                    || path.starts_with(&format!("{}/", scope.trim_end_matches(['/', '\\'])))
            })
        })
    {
        return Err(anyhow!(
            "Hands consequence does not match its approved review and Substrate Gate grant"
        ));
    }
    Ok(())
}

/// Revalidates the persisted Hands/Substrate authority chain before an actuator
/// performs a consequence. Receipt writers call the same primitive again after
/// the consequence; this preflight prevents a stale or substituted grant from
/// authorizing the consequence in the first place.
pub fn validate_hands_action_authority(
    store_path: impl AsRef<Path>,
    intent_id: &str,
    review_id: &str,
    runtime_job_id: &str,
    operation: &str,
    changed_paths: &[String],
    stated_grant_id: &str,
) -> Result<()> {
    validate_hands_consequence_grant(
        store_path.as_ref(),
        intent_id,
        review_id,
        runtime_job_id,
        operation,
        changed_paths,
        Some(stated_grant_id),
        None,
    )
}

pub fn put_hands_patch_receipt(
    store_path: impl AsRef<Path>,
    receipt: &HandsPatchReceipt,
) -> Result<()> {
    let store_path = store_path.as_ref();
    validate_non_empty(&receipt.receipt_id, "Hands patch receipt id")?;
    validate_non_empty(&receipt.intent_id, "Hands patch intent")?;
    validate_non_empty(&receipt.review_id, "Hands patch review")?;
    validate_non_empty(
        &receipt.substrate_gate_grant_receipt_id,
        "Hands patch Substrate Gate grant receipt",
    )?;
    validate_non_empty(&receipt.runtime_job_id, "Hands patch runtime job")?;
    validate_non_empty(&receipt.summary, "Hands patch summary")?;
    validate_non_empty(&receipt.emitted_at, "Hands patch timestamp")?;
    if receipt.changed_paths.is_empty() {
        return Err(anyhow!("Hands patch receipt must name changed paths"));
    }
    validate_hands_consequence_grant(
        store_path.as_ref(),
        &receipt.intent_id,
        &receipt.review_id,
        &receipt.runtime_job_id,
        "patch",
        &receipt.changed_paths,
        Some(&receipt.substrate_gate_grant_receipt_id),
        None,
    )?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let (envelope, _) = cache.prepare_entry(&receipt.receipt_id, receipt)?;
    if SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&[], vec![envelope])?
    {
        return Ok(());
    }
    let mut reloaded = runtime_spine_cache(store_path)?;
    reloaded.pull_all_backing_stores()?;
    match reloaded.get::<HandsPatchReceipt>(&receipt.receipt_id)? {
        Some(existing) if existing == *receipt => Ok(()),
        _ => Err(anyhow!("Hands patch receipt ids are immutable")),
    }
}

pub fn runtime_hands_patch_receipt(
    store_path: impl AsRef<Path>,
    receipt_id: &str,
) -> Result<Option<HandsPatchReceipt>> {
    validate_non_empty(receipt_id, "Hands patch receipt id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<HandsPatchReceipt>(receipt_id)
}

pub fn put_hands_command_receipt(
    store_path: impl AsRef<Path>,
    receipt: &HandsCommandReceipt,
) -> Result<()> {
    let store_path = store_path.as_ref();
    validate_non_empty(&receipt.receipt_id, "Hands command receipt id")?;
    validate_non_empty(&receipt.intent_id, "Hands command intent")?;
    validate_non_empty(&receipt.review_id, "Hands command review")?;
    validate_non_empty(
        &receipt.substrate_gate_grant_receipt_id,
        "Hands command Substrate Gate grant receipt",
    )?;
    validate_non_empty(&receipt.runtime_job_id, "Hands command runtime job")?;
    validate_non_empty(&receipt.command, "Hands command")?;
    validate_non_empty(&receipt.exit_code, "Hands command exit code")?;
    validate_non_empty(&receipt.emitted_at, "Hands command timestamp")?;
    validate_hands_consequence_grant(
        store_path.as_ref(),
        &receipt.intent_id,
        &receipt.review_id,
        &receipt.runtime_job_id,
        "command",
        &[],
        Some(&receipt.substrate_gate_grant_receipt_id),
        Some(&receipt.command),
    )?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let (envelope, _) = cache.prepare_entry(&receipt.receipt_id, receipt)?;
    if SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&[], vec![envelope])?
    {
        return Ok(());
    }
    let mut reloaded = runtime_spine_cache(store_path)?;
    reloaded.pull_all_backing_stores()?;
    match reloaded.get::<HandsCommandReceipt>(&receipt.receipt_id)? {
        Some(existing) if existing == *receipt => Ok(()),
        _ => Err(anyhow!("Hands command receipt ids are immutable")),
    }
}

pub fn runtime_hands_command_receipt(
    store_path: impl AsRef<Path>,
    receipt_id: &str,
) -> Result<Option<HandsCommandReceipt>> {
    validate_non_empty(receipt_id, "Hands command receipt id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<HandsCommandReceipt>(receipt_id)
}

pub fn put_hands_commit_receipt(
    store_path: impl AsRef<Path>,
    receipt: &HandsCommitReceipt,
) -> Result<()> {
    let store_path = store_path.as_ref();
    validate_non_empty(&receipt.receipt_id, "Hands commit receipt id")?;
    validate_non_empty(&receipt.intent_id, "Hands commit intent")?;
    validate_non_empty(&receipt.review_id, "Hands commit review")?;
    validate_non_empty(&receipt.runtime_job_id, "Hands commit runtime job")?;
    validate_non_empty(&receipt.commit_sha, "Hands commit sha")?;
    validate_non_empty(&receipt.branch, "Hands commit branch")?;
    validate_non_empty(&receipt.summary, "Hands commit summary")?;
    validate_non_empty(&receipt.emitted_at, "Hands commit timestamp")?;
    if receipt.changed_paths.is_empty() {
        return Err(anyhow!("Hands commit receipt must name changed paths"));
    }
    validate_hands_consequence_grant(
        store_path.as_ref(),
        &receipt.intent_id,
        &receipt.review_id,
        &receipt.runtime_job_id,
        "commit",
        &receipt.changed_paths,
        None,
        None,
    )?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let (envelope, _) = cache.prepare_entry(&receipt.receipt_id, receipt)?;
    if SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&[], vec![envelope])?
    {
        return Ok(());
    }
    let mut reloaded = runtime_spine_cache(store_path)?;
    reloaded.pull_all_backing_stores()?;
    match reloaded.get::<HandsCommitReceipt>(&receipt.receipt_id)? {
        Some(existing) if existing == *receipt => Ok(()),
        _ => Err(anyhow!("Hands commit receipt ids are immutable")),
    }
}

pub fn runtime_hands_commit_receipt(
    store_path: impl AsRef<Path>,
    receipt_id: &str,
) -> Result<Option<HandsCommitReceipt>> {
    validate_non_empty(receipt_id, "Hands commit receipt id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<HandsCommitReceipt>(receipt_id)
}

#[cfg(test)]
pub fn put_hands_pr_receipt(store_path: impl AsRef<Path>, receipt: &HandsPrReceipt) -> Result<()> {
    validate_non_empty(&receipt.receipt_id, "Hands PR receipt id")?;
    validate_non_empty(&receipt.intent_id, "Hands PR intent")?;
    validate_non_empty(&receipt.review_id, "Hands PR review")?;
    validate_non_empty(&receipt.runtime_job_id, "Hands PR runtime job")?;
    validate_non_empty(&receipt.commit_receipt_id, "Hands PR commit receipt")?;
    validate_non_empty(&receipt.commit_sha, "Hands PR commit sha")?;
    validate_non_empty(&receipt.branch, "Hands PR branch")?;
    validate_non_empty(&receipt.pull_request_url, "Hands PR url")?;
    validate_non_empty(&receipt.pull_request_number, "Hands PR number")?;
    validate_non_empty(&receipt.pull_request_title, "Hands PR title")?;
    validate_non_empty(
        &receipt.bifrost_publication_receipt_id,
        "Hands PR Bifrost publication receipt",
    )?;
    validate_non_empty(&receipt.summary, "Hands PR summary")?;
    validate_non_empty(&receipt.emitted_at, "Hands PR timestamp")?;
    if receipt.changed_paths.is_empty() {
        return Err(anyhow!("Hands PR receipt must name changed paths"));
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    cache.put(&receipt.receipt_id, receipt)?;
    Ok(())
}

pub fn runtime_hands_pr_receipt(
    store_path: impl AsRef<Path>,
    receipt_id: &str,
) -> Result<Option<HandsPrReceipt>> {
    validate_non_empty(receipt_id, "Hands PR receipt id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<HandsPrReceipt>(receipt_id)
}

pub fn runtime_hands_receipt_chain_after(
    store_path: impl AsRef<Path>,
    after_timestamp: &str,
) -> Result<bool> {
    Ok(runtime_latest_hands_receipt_chain_after(store_path, after_timestamp)?.is_some())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHandsReceiptChainSummary {
    pub patch_schema_version: String,
    pub patch_receipt_id: String,
    pub command_schema_version: String,
    pub command_receipt_id: String,
    pub commit_schema_version: String,
    pub commit_receipt_id: String,
    pub intent_id: String,
    pub review_id: String,
    pub runtime_job_id: String,
    pub substrate_gate_grant_receipt_id: String,
    pub changed_paths: Vec<String>,
    pub command: String,
    pub exit_code: String,
    pub stdout_artifact: String,
    pub stderr_artifact: String,
    pub commit_sha: String,
    pub branch: String,
    pub summary: String,
    pub emitted_at: String,
}

pub fn runtime_latest_hands_receipt_chain_after(
    store_path: impl AsRef<Path>,
    after_timestamp: &str,
) -> Result<Option<RuntimeHandsReceiptChainSummary>> {
    validate_non_empty(after_timestamp, "Hands receipt lower-bound timestamp")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let patches = cache.get_all::<HandsPatchReceipt>()?;
    let commands = cache.get_all::<HandsCommandReceipt>()?;
    let commits = cache.get_all::<HandsCommitReceipt>()?;

    let mut summaries = Vec::new();
    for commit in commits
        .iter()
        .filter(|commit| timestamp_after(&commit.emitted_at, after_timestamp))
    {
        let Some(patch) = patches
            .iter()
            .filter(|patch| {
                patch.intent_id == commit.intent_id
                    && patch.review_id == commit.review_id
                    && patch.runtime_job_id == commit.runtime_job_id
                    && timestamp_after(&patch.emitted_at, after_timestamp)
                    && patch.emitted_at <= commit.emitted_at
            })
            .max_by(|left, right| left.emitted_at.cmp(&right.emitted_at))
        else {
            continue;
        };
        let Some(command) = commands
            .iter()
            .filter(|command| {
                command.intent_id == commit.intent_id
                    && command.review_id == commit.review_id
                    && command.runtime_job_id == commit.runtime_job_id
                    && command.exit_code == "0"
                    && timestamp_after(&command.emitted_at, after_timestamp)
                    && command.emitted_at <= commit.emitted_at
            })
            .max_by(|left, right| left.emitted_at.cmp(&right.emitted_at))
        else {
            continue;
        };
        summaries.push(RuntimeHandsReceiptChainSummary {
            patch_schema_version: patch.schema_version.clone(),
            patch_receipt_id: patch.receipt_id.clone(),
            command_schema_version: command.schema_version.clone(),
            command_receipt_id: command.receipt_id.clone(),
            commit_schema_version: commit.schema_version.clone(),
            commit_receipt_id: commit.receipt_id.clone(),
            intent_id: commit.intent_id.clone(),
            review_id: commit.review_id.clone(),
            runtime_job_id: commit.runtime_job_id.clone(),
            substrate_gate_grant_receipt_id: command.substrate_gate_grant_receipt_id.clone(),
            changed_paths: commit.changed_paths.clone(),
            command: command.command.clone(),
            exit_code: command.exit_code.clone(),
            stdout_artifact: command.stdout_artifact.clone(),
            stderr_artifact: command.stderr_artifact.clone(),
            commit_sha: commit.commit_sha.clone(),
            branch: commit.branch.clone(),
            summary: commit.summary.clone(),
            emitted_at: commit.emitted_at.clone(),
        });
    }
    summaries.sort_by(|left, right| left.emitted_at.cmp(&right.emitted_at));
    Ok(summaries.pop())
}

/// A complete historical receipt chain becomes notification-only when its
/// route no longer describes the current RepoModel. Self uses this predicate
/// before routing the chain to Soul.
pub fn runtime_hands_receipt_chain_matches_current_model(
    store_path: impl AsRef<Path>,
    chain: &RuntimeHandsReceiptChainSummary,
) -> Result<bool> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let authorities = cache
        .get_all::<RepoFrontierHandsAuthority>()?
        .into_iter()
        .filter(|authority| authority.hands_intent_id == chain.intent_id)
        .collect::<Vec<_>>();
    if authorities.len() != 1 {
        return Err(anyhow!(
            "Hands receipt chain requires exactly one persisted frontier authority"
        ));
    }
    let authority = &authorities[0];
    let route = cache
        .get::<RepoFrontierRoute>(&authority.route_id)?
        .ok_or_else(|| anyhow!("Hands receipt chain lost its persisted route"))?;
    let current = require_keyed_repo_model_basis(
        &cache,
        &route.model_projection_digest,
        &route.model_source_documents,
    )
    .is_ok();
    Ok(authority.hands_review_id == chain.review_id
        && authority.substrate_grant_receipt_id == chain.substrate_gate_grant_receipt_id
        && authority.route_id == route.route_id
        && authority.model_projection_digest == route.model_projection_digest
        && authority.model_source_documents == route.model_source_documents
        && authority.frontier_item_id == route.frontier_item_id
        && authority.frontier_item_hash == route.frontier_item_hash
        && current)
}

// Soul verdicts are terminal acceptance evidence. Production creates them only
// as prerequisites inside the coordinator's atomic Mind acceptance
// transaction; this crate-private writer exists for focused spine fixtures and
// migration tests, not as an independent runtime actuator.
#[cfg(test)]
pub(crate) fn put_soul_verdict_receipt(
    store_path: impl AsRef<Path>,
    receipt: &SoulVerdictReceipt,
) -> Result<()> {
    let store_path = store_path.as_ref();
    validate_non_empty(&receipt.receipt_id, "Soul verdict receipt id")?;
    validate_non_empty(&receipt.source_result_id, "Soul verdict source result")?;
    validate_non_empty(&receipt.source_job_id, "Soul verdict source job")?;
    validate_non_empty(&receipt.verdict, "Soul verdict")?;
    validate_non_empty(&receipt.emitted_at, "Soul verdict timestamp")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let (envelope, _) = cache.prepare_entry(&receipt.receipt_id, receipt)?;
    if SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&[], vec![envelope])?
    {
        return Ok(());
    }
    let mut reloaded = runtime_spine_cache(store_path)?;
    reloaded.pull_all_backing_stores()?;
    match reloaded.get::<SoulVerdictReceipt>(&receipt.receipt_id)? {
        Some(existing) if existing == *receipt => Ok(()),
        _ => Err(anyhow!(
            "Soul verdict receipt id already belongs to different immutable evidence"
        )),
    }
}

pub fn runtime_soul_verdict_receipt(
    store_path: impl AsRef<Path>,
    receipt_id: &str,
) -> Result<Option<SoulVerdictReceipt>> {
    validate_non_empty(receipt_id, "Soul verdict receipt id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<SoulVerdictReceipt>(receipt_id)
}

// Continuity recovery receipts are accepted-reorientation prerequisites and
// publish atomically with the coordinator state transition. Direct insertion
// is fixture-only.
#[cfg(test)]
pub(crate) fn put_continuity_recovery_receipt(
    store_path: impl AsRef<Path>,
    receipt: &ContinuityRecoveryReceipt,
) -> Result<()> {
    validate_non_empty(&receipt.receipt_id, "Continuity recovery receipt id")?;
    validate_non_empty(
        &receipt.source_result_id,
        "Continuity recovery source result",
    )?;
    validate_non_empty(&receipt.source_job_id, "Continuity recovery source job")?;
    validate_non_empty(&receipt.binding_id, "Continuity recovery binding")?;
    validate_non_empty(&receipt.mode, "Continuity recovery mode")?;
    validate_non_empty(
        &receipt.checkpoint_still_valid,
        "Continuity recovery checkpoint validity",
    )?;
    validate_non_empty(&receipt.emitted_at, "Continuity recovery timestamp")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    cache.put(&receipt.receipt_id, receipt)?;
    Ok(())
}

pub fn runtime_continuity_recovery_receipt(
    store_path: impl AsRef<Path>,
    receipt_id: &str,
) -> Result<Option<ContinuityRecoveryReceipt>> {
    validate_non_empty(receipt_id, "Continuity recovery receipt id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<ContinuityRecoveryReceipt>(receipt_id)
}

pub fn put_coordinator_run_receipt(
    store_path: impl AsRef<Path>,
    receipt: &EpiphanyCoordinatorRunReceipt,
) -> Result<()> {
    validate_coordinator_run_receipt(receipt)?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    cache.put(&receipt.receipt_id, receipt)?;
    Ok(())
}

fn validate_coordinator_run_receipt(receipt: &EpiphanyCoordinatorRunReceipt) -> Result<()> {
    validate_non_empty(&receipt.receipt_id, "coordinator run receipt id")?;
    validate_non_empty(&receipt.session_id, "coordinator run receipt session id")?;
    validate_non_empty(&receipt.thread_id, "coordinator run receipt thread id")?;
    validate_non_empty(&receipt.mode, "coordinator run receipt mode")?;
    validate_non_empty(&receipt.status, "coordinator run receipt status")?;
    validate_non_empty(
        &receipt.final_action,
        "coordinator run receipt final action",
    )?;
    validate_non_empty(&receipt.created_at, "coordinator run receipt created at")?;
    chrono::DateTime::parse_from_rfc3339(&receipt.created_at)
        .map_err(|error| anyhow!("coordinator run receipt timestamp is invalid: {error}"))?;
    Ok(())
}

pub fn runtime_typed_request_attempt_exists(
    store_path: impl AsRef<Path>,
    request: RuntimeTypedRequestRef<'_>,
) -> Result<bool> {
    let request_id = request.request_id();
    validate_non_empty(request_id, "typed attempt request id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let claims = cache
        .get_all::<EpiphanyRuntimeWorkerProcessClaim>()?
        .into_iter()
        .map(|claim| {
            Ok((
                claim.job_id.clone(),
                WorkerProcessStatus::parse(&claim.status)?,
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    Ok(cache
        .get_all::<EpiphanyRuntimeWorkerLaunchRequest>()?
        .iter()
        .filter(|launch| {
            claims
                .get(&launch.job_id)
                .is_none_or(|status| !status.allows_retry())
        })
        .any(|launch| request.matches_launch(launch)))
}

pub fn archive_failed_runtime_worker_attempt(
    store_path: impl AsRef<Path>,
    job_id: &str,
    live_resident_request_ids: &BTreeSet<String>,
    archived_at: &str,
) -> Result<EpiphanyArchivedRuntimeWorkerAttempt> {
    archive_runtime_worker_attempt(
        store_path,
        job_id,
        live_resident_request_ids,
        archived_at,
        false,
        || Ok(()),
    )
}

pub fn archive_fulfilled_runtime_worker_attempt(
    store_path: impl AsRef<Path>,
    job_id: &str,
    live_resident_request_ids: &BTreeSet<String>,
    archived_at: &str,
) -> Result<EpiphanyArchivedRuntimeWorkerAttempt> {
    archive_runtime_worker_attempt(
        store_path,
        job_id,
        live_resident_request_ids,
        archived_at,
        true,
        || Ok(()),
    )
}

fn validate_archivable_typed_worker_launch(
    cache: &CultCache,
    launch: &EpiphanyRuntimeWorkerLaunchRequest,
    request_kind: &str,
    request_id: &str,
) -> Result<()> {
    if launch.schema_version != RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION
        || launch.job_id.trim().is_empty()
    {
        return Err(anyhow!(
            "worker attempt archive found invalid immutable launch"
        ));
    }
    let document = launch.launch_document()?;
    let identity = require_identity(cache)?;
    let launch_sha256 = format!("{:x}", Sha256::digest(&launch.launch_document_msgpack));
    match request_kind {
        "proposal-modeling" => {
            validate_proposal_modeling_launch_carrier(
                &launch.role,
                &launch.binding_id,
                Some(request_id),
                &document,
            )?;
            validate_repository_body_launch_carrier(&launch.role, &document)?;
            let request = cache
                .get::<RepoFrontierProposalModelingRequest>(request_id)?
                .ok_or_else(|| anyhow!("archived proposal Modeling launch lost its request"))?;
            validate_repo_frontier_proposal_modeling_request(&request)?;
            let proposal = cache
                .get::<RepoFrontierWorkProposal>(&request.proposal_id)?
                .ok_or_else(|| anyhow!("archived proposal Modeling launch lost its proposal"))?;
            validate_repo_frontier_work_proposal(&proposal)?;
            let bindings = cache
                .get_all::<RepoFrontierProposalModelingLaunchBinding>()?
                .into_iter()
                .filter(|binding| binding.job_id == launch.job_id)
                .collect::<Vec<_>>();
            if bindings.len() != 1 {
                return Err(anyhow!(
                    "archived proposal Modeling launch requires one binding"
                ));
            }
            let binding = &bindings[0];
            let projection = match &document {
                EpiphanyWorkerLaunchDocument::Role(document) => {
                    document.proposal_modeling_context.as_ref()
                }
                EpiphanyWorkerLaunchDocument::Reorient(_) => None,
            }
            .ok_or_else(|| anyhow!("archived proposal Modeling launch lost its context"))?;
            if request.runtime_id != identity.runtime_id
                || request.proposal_payload_sha256 != proposal.payload_sha256
                || request.repository != proposal.repository
                || request.workspace != proposal.workspace
                || binding.schema_version
                    != REPO_FRONTIER_PROPOSAL_MODELING_LAUNCH_BINDING_SCHEMA_VERSION
                || binding.contract != REPO_FRONTIER_PROPOSAL_MODELING_LAUNCH_BINDING_CONTRACT
                || binding.binding_record_id
                    != format!("repo-frontier-proposal-modeling-launch-{}", launch.job_id)
                || binding.proposal_modeling_request_id != request.request_id
                || binding.proposal_id != proposal.proposal_id
                || binding.proposal_payload_sha256 != proposal.payload_sha256
                || binding.binding_id != EPIPHANY_MODELING_ROLE_BINDING_ID
                || binding.runtime_id != request.runtime_id
                || binding.thread_id != request.thread_id
                || binding.worker_launch_document_sha256 != launch_sha256
                || projection.request_id != request.request_id
                || projection.proposal_id != proposal.proposal_id
                || projection.proposal_payload_sha256 != proposal.payload_sha256
                || projection.runtime_id != request.runtime_id
                || projection.thread_id != request.thread_id
                || projection.repository != request.repository
                || projection.workspace != request.workspace
            {
                return Err(anyhow!(
                    "archived proposal Modeling launch provenance mismatch"
                ));
            }
        }
        "imagination-consideration" => {
            validate_imagination_consideration_launch_carrier(
                &launch.role,
                &launch.binding_id,
                Some(request_id),
                &document,
            )?;
            let request = cache
                .get::<crate::ImaginationConsiderationRequest>(request_id)?
                .ok_or_else(|| anyhow!("archived Imagination launch lost its request"))?;
            if request.schema_version != crate::IMAGINATION_CONSIDERATION_REQUEST_SCHEMA_VERSION
                || request.contract != crate::IMAGINATION_CONSIDERATION_REQUEST_CONTRACT
                || request.request_id != request_id
                || request.private_state_included
                || chrono::DateTime::parse_from_rfc3339(&request.requested_at).is_err()
            {
                return Err(anyhow!("archived Imagination launch request is invalid"));
            }
            let bindings = cache
                .get_all::<crate::ImaginationConsiderationLaunchBinding>()?
                .into_iter()
                .filter(|binding| binding.job_id == launch.job_id)
                .collect::<Vec<_>>();
            if bindings.len() != 1 {
                return Err(anyhow!("archived Imagination launch requires one binding"));
            }
            let binding = &bindings[0];
            let projection = match &document {
                EpiphanyWorkerLaunchDocument::Role(document) => {
                    document.imagination_consideration_context.as_ref()
                }
                EpiphanyWorkerLaunchDocument::Reorient(_) => None,
            }
            .ok_or_else(|| anyhow!("archived Imagination launch lost its context"))?;
            if request.runtime_id != identity.runtime_id
                || binding.request_id != request.request_id
                || binding.binding_id != EPIPHANY_IMAGINATION_ROLE_BINDING_ID
                || binding.runtime_id != request.runtime_id
                || binding.thread_id != request.thread_id
                || binding.worker_launch_document_sha256 != launch_sha256
                || projection.request != request
                || projection.model.reasoning_basis()
                    != (crate::EpiphanyRepoModelBasis {
                        projection_digest: request.model_projection_digest.clone(),
                        source_documents: request.model_source_documents.clone(),
                    })
            {
                return Err(anyhow!("archived Imagination launch provenance mismatch"));
            }
        }
        "admitted-model-direction" => {
            let request = cache
                .get::<crate::AdmittedModelDirectionConsiderationRequest>(request_id)?
                .ok_or_else(|| anyhow!("archived model-direction launch lost its request"))?;
            if request.schema_version
                != crate::ADMITTED_MODEL_DIRECTION_CONSIDERATION_REQUEST_SCHEMA_VERSION
                || request.contract
                    != crate::ADMITTED_MODEL_DIRECTION_CONSIDERATION_REQUEST_CONTRACT
                || request.request_id != request_id
                || request.private_state_included
                || chrono::DateTime::parse_from_rfc3339(&request.requested_at).is_err()
            {
                return Err(anyhow!(
                    "archived model-direction launch request is invalid"
                ));
            }
            let projection = match &document {
                EpiphanyWorkerLaunchDocument::Role(document) => document
                    .admitted_model_direction_consideration_context
                    .as_ref(),
                EpiphanyWorkerLaunchDocument::Reorient(_) => None,
            }
            .ok_or_else(|| anyhow!("archived model-direction launch lost its context"))?;
            if launch.role != EPIPHANY_IMAGINATION_OWNER_ROLE
                || launch.binding_id != EPIPHANY_IMAGINATION_ROLE_BINDING_ID
                || request.runtime_id != identity.runtime_id
                || projection.request != request
                || projection.model.reasoning_basis()
                    != (crate::EpiphanyRepoModelBasis {
                        projection_digest: request.model_projection_digest.clone(),
                        source_documents: request.model_source_documents.clone(),
                    })
            {
                return Err(anyhow!(
                    "archived model-direction launch provenance mismatch"
                ));
            }
        }
        _ => {
            return Err(anyhow!(
                "worker attempt archive found unsupported typed request"
            ));
        }
    }
    Ok(())
}

fn archive_runtime_worker_attempt<F>(
    store_path: impl AsRef<Path>,
    job_id: &str,
    live_resident_request_ids: &BTreeSet<String>,
    archived_at: &str,
    fulfilled: bool,
    before_commit: F,
) -> Result<EpiphanyArchivedRuntimeWorkerAttempt>
where
    F: FnOnce() -> Result<()>,
{
    validate_non_empty(job_id, "archived worker attempt job id")?;
    chrono::DateTime::parse_from_rfc3339(archived_at)
        .map_err(|error| anyhow!("worker attempt archive timestamp is invalid: {error}"))?;
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    if let Some(existing) = cache.get::<EpiphanyArchivedRuntimeWorkerAttempt>(job_id)? {
        if existing.schema_version != ARCHIVED_RUNTIME_WORKER_ATTEMPT_SCHEMA_VERSION
            || existing.archive_id != job_id
            || existing.job_id != job_id
            || existing.result_id.is_some() != fulfilled
            || !existing.retired_chain_digest.starts_with("sha256:")
        {
            return Err(anyhow!("archived worker attempt tombstone is invalid"));
        }
        if cache
            .get::<EpiphanyRuntimeWorkerLaunchRequest>(job_id)?
            .is_some()
            || cache
                .get::<EpiphanyRuntimeWorkerProcessClaim>(&worker_process_claim_id(job_id))?
                .is_some()
            || cache
                .get::<EpiphanyRuntimeRoleWorkerResult>(job_id)?
                .is_some()
        {
            return Err(anyhow!(
                "archived worker attempt retained live attempt authority"
            ));
        }
        return Ok(existing);
    }
    let launch = cache
        .get::<EpiphanyRuntimeWorkerLaunchRequest>(job_id)?
        .ok_or_else(|| anyhow!("worker attempt archive requires its immutable launch"))?;
    let request = launch.typed_request_ref()?.ok_or_else(|| {
        anyhow!("worker attempt archive requires exactly one supported typed request")
    })?;
    let request_kind = request.kind();
    let request_id = request.request_id();
    validate_archivable_typed_worker_launch(&cache, &launch, request_kind, request_id)?;
    if live_resident_request_ids.contains(request_id) {
        return Err(anyhow!(
            "worker attempt archive refuses resident-live typed request"
        ));
    }
    let role_result = cache.get::<EpiphanyRuntimeRoleWorkerResult>(job_id)?;
    if fulfilled != role_result.is_some() {
        return Err(anyhow!(
            "worker attempt archive terminal result shape disagrees with requested archive kind"
        ));
    }
    let claim_id = worker_process_claim_id(job_id);
    let claim = cache
        .get::<EpiphanyRuntimeWorkerProcessClaim>(&claim_id)?
        .ok_or_else(|| anyhow!("worker attempt archive requires its process claim"))?;
    let status = crate::WorkerProcessStatus::parse(&claim.status)?;
    let valid_claim = if fulfilled {
        status.is_fulfilled_terminal()
    } else {
        status.is_failed_terminal()
    };
    if !valid_claim || claim.terminal_at.is_none() || claim.terminal_authority_id.is_none() {
        return Err(anyhow!(
            "failed worker attempt archive requires exact terminal process authority"
        ));
    }
    let job = cache
        .get::<EpiphanyRuntimeJob>(job_id)?
        .ok_or_else(|| anyhow!("worker attempt archive requires its runtime job"))?;
    let valid_job = if fulfilled {
        job.status == EpiphanyRuntimeJobStatus::Completed
    } else {
        matches!(
            job.status,
            EpiphanyRuntimeJobStatus::Failed | EpiphanyRuntimeJobStatus::Cancelled
        )
    };
    if !valid_job {
        return Err(anyhow!(
            "worker attempt archive requires matching terminal runtime job"
        ));
    }
    let archived_result_id = if fulfilled {
        let request_ref = match request_kind {
            "proposal-modeling" => RuntimeTypedRequestRef::ProposalModeling(request_id),
            "imagination-consideration" => {
                RuntimeTypedRequestRef::ImaginationConsideration(request_id)
            }
            "admitted-model-direction" => {
                RuntimeTypedRequestRef::AdmittedModelDirection(request_id)
            }
            _ => unreachable!("typed request kind was selected above"),
        };
        let evidence =
            runtime_typed_request_fulfillment(store_path, request_ref)?.ok_or_else(|| {
                anyhow!("fulfilled worker attempt archive lost authenticated fulfillment")
            })?;
        if evidence.job_id != job_id
            || claim.terminal_authority_id.as_deref() != Some(evidence.result_id.as_str())
        {
            return Err(anyhow!(
                "fulfilled worker attempt archive substituted terminal authority"
            ));
        }
        if request_kind == "proposal-modeling"
            && !worker_result_has_keyed_mind_commit(
                &cache,
                role_result
                    .as_ref()
                    .expect("fulfilled result checked above"),
            )?
        {
            return Err(anyhow!(
                "proposal Modeling attempt remains live until Mind admission owns its result"
            ));
        }
        Some(evidence.result_id)
    } else {
        None
    };
    let snapshot = cache.snapshot_envelopes();
    let proposal_bindings = cache
        .get_all::<RepoFrontierProposalModelingLaunchBinding>()?
        .into_iter()
        .filter(|item| item.job_id == job_id)
        .map(|item| item.binding_record_id)
        .collect::<BTreeSet<_>>();
    let imagination_bindings = cache
        .get_all::<crate::ImaginationConsiderationLaunchBinding>()?
        .into_iter()
        .filter(|item| item.job_id == job_id)
        .map(|item| item.binding_record_id)
        .collect::<BTreeSet<_>>();
    let worker_job_results = cache
        .get_all::<EpiphanyRuntimeJobResult>()?
        .into_iter()
        .filter(|item| item.job_id == job_id)
        .collect::<Vec<_>>();
    let job_results = worker_job_results
        .iter()
        .map(|item| item.result_id.clone())
        .collect::<BTreeSet<_>>();
    let role_decision_context_id = cache
        .get::<EpiphanyRuntimeRoleWorkerResult>(job_id)?
        .map(|result| result.decision_context_id);
    let mut decision_context_ids = worker_job_results
        .iter()
        .filter_map(|result| result.decision_context_id.clone())
        .chain(role_decision_context_id)
        .collect::<BTreeSet<_>>();
    if decision_context_ids.len() > 1 {
        return Err(anyhow!(
            "worker attempt archive found conflicting decision contexts"
        ));
    }
    let decision_context_id = decision_context_ids.pop_first();
    if fulfilled && decision_context_id.is_none() {
        return Err(anyhow!(
            "fulfilled worker attempt archive requires its decision context"
        ));
    }
    if let Some(context_id) = decision_context_id.as_deref() {
        require_worker_decision_context(&cache, context_id, job_id)?;
    }
    let events = cache
        .get_all::<EpiphanyRuntimeEvent>()?
        .into_iter()
        .filter(|item| item.job_id.as_deref() == Some(job_id))
        .map(|item| item.event_id)
        .collect::<BTreeSet<_>>();
    let mut deletions = snapshot
        .iter()
        .filter(|entry| {
            (entry.r#type == EpiphanyRuntimeWorkerLaunchRequest::TYPE && entry.key == job_id)
                || (entry.r#type == EpiphanyRuntimeWorkerProcessClaim::TYPE
                    && entry.key == claim_id)
                || (entry.r#type == EpiphanyRuntimeRoleWorkerResult::TYPE
                    && fulfilled
                    && entry.key == job_id)
                || (entry.r#type == EpiphanyRuntimeJob::TYPE && entry.key == job_id)
                || (entry.r#type == EpiphanyRuntimeJobResult::TYPE
                    && job_results.contains(&entry.key))
                || (entry.r#type == EpiphanyRuntimeEvent::TYPE && events.contains(&entry.key))
                || (entry.r#type == RepoFrontierProposalModelingLaunchBinding::TYPE
                    && proposal_bindings.contains(&entry.key))
                || (entry.r#type == crate::ImaginationConsiderationLaunchBinding::TYPE
                    && imagination_bindings.contains(&entry.key))
        })
        .cloned()
        .collect::<Vec<_>>();
    deletions.sort_by(|a, b| a.r#type.cmp(&b.r#type).then(a.key.cmp(&b.key)));
    if !deletions
        .iter()
        .any(|e| e.r#type == EpiphanyRuntimeWorkerLaunchRequest::TYPE && e.key == job_id)
        || !deletions
            .iter()
            .any(|e| e.r#type == EpiphanyRuntimeWorkerProcessClaim::TYPE && e.key == claim_id)
        || !deletions
            .iter()
            .any(|e| e.r#type == EpiphanyRuntimeJob::TYPE && e.key == job_id)
    {
        return Err(anyhow!("worker attempt archive lost its exact core family"));
    }
    let mut counts = BTreeMap::new();
    let mut digest = Sha256::new();
    digest.update(b"epiphany-archived-worker-attempt-root");
    for entry in &deletions {
        *counts.entry(entry.r#type.clone()).or_default() += 1;
        for bytes in [
            entry.r#type.as_bytes(),
            entry.key.as_bytes(),
            entry.payload.as_slice(),
        ] {
            digest.update((bytes.len() as u64).to_le_bytes());
            digest.update(bytes);
        }
    }
    let tombstone = EpiphanyArchivedRuntimeWorkerAttempt {
        schema_version: ARCHIVED_RUNTIME_WORKER_ATTEMPT_SCHEMA_VERSION.into(),
        archive_id: job_id.into(),
        job_id: job_id.into(),
        request_kind: request_kind.into(),
        request_id: request_id.into(),
        terminal_process_status: claim.status,
        result_id: archived_result_id,
        archived_at: archived_at.into(),
        retired_type_counts: counts,
        retired_envelope_count: deletions.len() as u64,
        retired_chain_digest: format!("sha256:{:x}", digest.finalize()),
        decision_context_id,
    };
    let replacement = cache.prepare_entry(job_id, &tombstone)?.0;
    before_commit()?;
    if !runtime_spine_backing_store(store_path)?.replace_and_delete_if_snapshot_unchanged(
        &snapshot,
        vec![replacement],
        &deletions,
    )? {
        return Err(anyhow!(
            "worker attempt archive lost its full snapshot fence"
        ));
    }
    Ok(tombstone)
}

pub fn retain_failed_runtime_worker_attempts(
    store_path: impl AsRef<Path>,
    retain_recent: usize,
    live_resident_request_ids: &BTreeSet<String>,
    archived_at: &str,
) -> Result<Vec<EpiphanyArchivedRuntimeWorkerAttempt>> {
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let launches = cache
        .get_all::<EpiphanyRuntimeWorkerLaunchRequest>()?
        .into_iter()
        .map(|launch| (launch.job_id.clone(), launch))
        .collect::<BTreeMap<_, _>>();
    let typed_request_ids = launches
        .values()
        .map(|launch| {
            Ok((
                launch.job_id.clone(),
                launch
                    .typed_request_ref()?
                    .map(|request| request.request_id().to_string()),
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let jobs = cache
        .get_all::<EpiphanyRuntimeJob>()?
        .into_iter()
        .map(|job| (job.job_id.clone(), job))
        .collect::<BTreeMap<_, _>>();
    let claims = cache
        .get_all::<EpiphanyRuntimeWorkerProcessClaim>()?
        .into_iter()
        .map(|claim| Ok((crate::WorkerProcessStatus::parse(&claim.status)?, claim)))
        .collect::<Result<Vec<_>>>()?;
    let mut candidates = claims
        .into_iter()
        .filter(|(status, _)| status.is_failed_terminal())
        .filter_map(|(_, claim)| {
            let job = jobs.get(&claim.job_id)?;
            if !matches!(
                job.status,
                EpiphanyRuntimeJobStatus::Failed | EpiphanyRuntimeJobStatus::Cancelled
            ) {
                return None;
            }
            let request_id = typed_request_ids.get(&claim.job_id)?.as_deref()?;
            (!live_resident_request_ids.contains(request_id)).then_some(claim)
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|a, b| {
        b.terminal_at
            .cmp(&a.terminal_at)
            .then(b.job_id.cmp(&a.job_id))
    });
    candidates
        .into_iter()
        .skip(retain_recent.max(1))
        .map(|claim| {
            archive_failed_runtime_worker_attempt(
                store_path,
                &claim.job_id,
                live_resident_request_ids,
                archived_at,
            )
        })
        .collect()
}

pub fn retain_fulfilled_runtime_worker_attempts(
    store_path: impl AsRef<Path>,
    retain_recent: usize,
    live_resident_request_ids: &BTreeSet<String>,
    archived_at: &str,
) -> Result<Vec<EpiphanyArchivedRuntimeWorkerAttempt>> {
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let launches = cache
        .get_all::<EpiphanyRuntimeWorkerLaunchRequest>()?
        .into_iter()
        .map(|launch| (launch.job_id.clone(), launch))
        .collect::<BTreeMap<_, _>>();
    let typed_request_ids = launches
        .values()
        .map(|launch| {
            Ok((
                launch.job_id.clone(),
                launch
                    .typed_request_ref()?
                    .map(|request| request.request_id().to_string()),
            ))
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    let jobs = cache
        .get_all::<EpiphanyRuntimeJob>()?
        .into_iter()
        .map(|job| (job.job_id.clone(), job))
        .collect::<BTreeMap<_, _>>();
    let role_results = cache
        .get_all::<EpiphanyRuntimeRoleWorkerResult>()?
        .into_iter()
        .map(|result| (result.job_id.clone(), result))
        .collect::<BTreeMap<_, _>>();
    let claims = cache
        .get_all::<EpiphanyRuntimeWorkerProcessClaim>()?
        .into_iter()
        .map(|claim| Ok((crate::WorkerProcessStatus::parse(&claim.status)?, claim)))
        .collect::<Result<Vec<_>>>()?;
    let mut candidates = claims
        .into_iter()
        .filter(|(status, _)| status.is_fulfilled_terminal())
        .filter_map(|(_, claim)| {
            let launch = launches.get(&claim.job_id)?;
            let job = jobs.get(&claim.job_id)?;
            let result = role_results.get(&claim.job_id)?;
            if job.status != EpiphanyRuntimeJobStatus::Completed
                || (launch.proposal_modeling_request_id.is_some()
                    && !worker_result_has_keyed_mind_commit(&cache, result).ok()?)
            {
                return None;
            }
            let request_id = typed_request_ids.get(&claim.job_id)?.as_deref()?;
            (!live_resident_request_ids.contains(request_id)).then_some(claim)
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|a, b| {
        b.terminal_at
            .cmp(&a.terminal_at)
            .then(b.job_id.cmp(&a.job_id))
    });
    candidates
        .into_iter()
        .skip(retain_recent.max(1))
        .map(|claim| {
            archive_fulfilled_runtime_worker_attempt(
                store_path,
                &claim.job_id,
                live_resident_request_ids,
                archived_at,
            )
        })
        .collect()
}

fn coordinator_completion_event(receipt: &EpiphanyCoordinatorRunReceipt) -> EpiphanyRuntimeEvent {
    EpiphanyRuntimeEvent {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        event_id: format!("event-session-completed-{}", receipt.session_id),
        occurred_at: receipt.created_at.clone(),
        event_type: "session.completed".to_string(),
        source: "epiphany-mvp-coordinator".to_string(),
        session_id: Some(receipt.session_id.clone()),
        job_id: None,
        summary: format!(
            "Coordinator run {:?} terminalized with status {:?}.",
            receipt.receipt_id, receipt.status
        ),
        metadata: BTreeMap::new(),
    }
}

pub const COORDINATOR_DEATH_RECOVERY_SCHEMA_VERSION: &str =
    "epiphany.coordinator_death_recovery.v0";

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.coordinator_death_recovery.v0",
    schema = "EpiphanyCoordinatorDeathRecovery"
)]
pub struct EpiphanyCoordinatorDeathRecovery {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub recovery_id: String,
    #[cultcache(key = 2)]
    pub session_id: String,
    #[cultcache(key = 3)]
    pub thread_id: String,
    #[cultcache(key = 4)]
    pub resident_grant_id: String,
    #[cultcache(key = 5)]
    pub resident_launch_digest: String,
    #[cultcache(key = 6)]
    pub process_id: u32,
    #[cultcache(key = 7)]
    pub process_creation_token: u64,
    #[cultcache(key = 8)]
    pub process_executable_path: String,
    #[cultcache(key = 9)]
    pub resident_started_at_millis: u64,
    #[cultcache(key = 10)]
    pub observation: String,
    #[cultcache(key = 11)]
    pub recovered_at: String,
    #[cultcache(key = 12, default)]
    pub private_state_exposed: bool,
    #[cultcache(key = 13, default)]
    pub exit_code: Option<i32>,
}

pub fn coordinator_run_session_id(
    thread_id: &str,
    resident_launch_digest: Option<&str>,
) -> Result<String> {
    validate_non_empty(thread_id, "coordinator run thread id")?;
    match resident_launch_digest {
        None => Ok(format!("coordinator-{thread_id}")),
        Some(digest) => {
            let hex = digest.strip_prefix("sha256:").ok_or_else(|| {
                anyhow!("resident coordinator session requires a SHA-256 launch digest")
            })?;
            if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(anyhow!(
                    "resident coordinator session launch digest is invalid"
                ));
            }
            Ok(format!(
                "coordinator-{thread_id}-resident-{}",
                hex.to_ascii_lowercase()
            ))
        }
    }
}

pub fn open_coordinator_run(
    store_path: impl AsRef<Path>,
    session_id: &str,
    thread_id: &str,
    resident_launch_digest: Option<&str>,
    objective: &str,
    started_at: &str,
) -> Result<(EpiphanyRuntimeSession, EpiphanyRuntimeEvent)> {
    open_coordinator_run_with_before_commit(
        store_path,
        session_id,
        thread_id,
        resident_launch_digest,
        objective,
        started_at,
        || Ok(()),
    )
}

fn open_coordinator_run_with_before_commit<F>(
    store_path: impl AsRef<Path>,
    session_id: &str,
    thread_id: &str,
    resident_launch_digest: Option<&str>,
    objective: &str,
    started_at: &str,
    before_commit: F,
) -> Result<(EpiphanyRuntimeSession, EpiphanyRuntimeEvent)>
where
    F: FnOnce() -> Result<()>,
{
    validate_non_empty(session_id, "coordinator run session id")?;
    validate_non_empty(thread_id, "coordinator run thread id")?;
    validate_non_empty(objective, "coordinator run objective")?;
    chrono::DateTime::parse_from_rfc3339(started_at)
        .map_err(|error| anyhow!("coordinator run start timestamp is invalid: {error}"))?;
    if session_id != coordinator_run_session_id(thread_id, resident_launch_digest)? {
        return Err(anyhow!(
            "coordinator run session id is not bound to its thread"
        ));
    }
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let event_id = format!("event-coordinator-started-{session_id}");
    if cache.get::<EpiphanyRuntimeSession>(session_id)?.is_some()
        || cache
            .get::<EpiphanyArchivedRuntimeSession>(session_id)?
            .is_some()
        || cache.get::<EpiphanyRuntimeEvent>(&event_id)?.is_some()
        || cache
            .get_all::<EpiphanyCoordinatorRunReceipt>()?
            .iter()
            .any(|receipt| {
                receipt.session_id == session_id
                    || (resident_launch_digest.is_none() && receipt.thread_id == thread_id)
            })
        || cache
            .get_all::<EpiphanyCoordinatorDeathRecovery>()?
            .iter()
            .any(|recovery| recovery.session_id == session_id)
    {
        return Err(anyhow!(
            "coordinator run session or thread authority already exists"
        ));
    }
    let session = EpiphanyRuntimeSession {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        session_id: session_id.to_string(),
        objective: objective.to_string(),
        status: EpiphanyRuntimeSessionStatus::Active,
        created_at: started_at.to_string(),
        updated_at: started_at.to_string(),
        coordinator_note: "Coordinator owns native runtime receipts before process exit."
            .to_string(),
        metadata: BTreeMap::new(),
    };
    let event = EpiphanyRuntimeEvent {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        event_id,
        occurred_at: started_at.to_string(),
        event_type: "coordinator.started".to_string(),
        source: "epiphany-mvp-coordinator".to_string(),
        session_id: Some(session_id.to_string()),
        job_id: None,
        summary: "Native coordinator session opened.".to_string(),
        metadata: BTreeMap::new(),
    };
    let snapshot = cache.snapshot_envelopes();
    let replacements = vec![
        cache.prepare_entry(session_id, &session)?.0,
        cache.prepare_entry(&event.event_id, &event)?.0,
    ];
    before_commit()?;
    if !runtime_spine_backing_store(store_path)?
        .replace_and_append_if_snapshot_unchanged(&snapshot, replacements)?
    {
        return Err(anyhow!(
            "coordinator run opening lost its full snapshot fence"
        ));
    }
    Ok((session, event))
}

pub fn finalize_coordinator_run(
    store_path: impl AsRef<Path>,
    receipt: &EpiphanyCoordinatorRunReceipt,
) -> Result<EpiphanyRuntimeSession> {
    finalize_coordinator_run_with_before_commit(store_path, receipt, || Ok(()))
}

fn coordinator_death_recovery_event(
    recovery: &EpiphanyCoordinatorDeathRecovery,
) -> EpiphanyRuntimeEvent {
    EpiphanyRuntimeEvent {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        event_id: format!("event-coordinator-death-recovered-{}", recovery.session_id),
        occurred_at: recovery.recovered_at.clone(),
        event_type: "coordinator.death-recovered".to_string(),
        source: "epiphany-continuity".to_string(),
        session_id: Some(recovery.session_id.clone()),
        job_id: None,
        summary: format!(
            "Continuity terminalized coordinator session after exact process observation {:?}{}.",
            recovery.observation,
            recovery
                .exit_code
                .map(|code| format!(" with exit code {code}"))
                .unwrap_or_default()
        ),
        metadata: BTreeMap::new(),
    }
}

pub(crate) fn recover_coordinator_run_after_exact_process_death(
    store_path: impl AsRef<Path>,
    recovery: &EpiphanyCoordinatorDeathRecovery,
    expected_objective: &str,
) -> Result<EpiphanyRuntimeSession> {
    recover_coordinator_run_after_exact_process_death_with_before_commit(
        store_path,
        recovery,
        expected_objective,
        || Ok(()),
    )
}

pub(crate) fn coordinator_run_incarnation_is_absent(
    store_path: impl AsRef<Path>,
    thread_id: &str,
    resident_launch_digest: &str,
) -> Result<bool> {
    let session_id = coordinator_run_session_id(thread_id, Some(resident_launch_digest))?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    Ok(cache.get::<EpiphanyRuntimeSession>(&session_id)?.is_none()
        && cache
            .get::<EpiphanyArchivedRuntimeSession>(&session_id)?
            .is_none()
        && !cache
            .get_all::<EpiphanyRuntimeEvent>()?
            .iter()
            .any(|event| event.session_id.as_deref() == Some(session_id.as_str()))
        && !cache
            .get_all::<EpiphanyRuntimeJob>()?
            .iter()
            .any(|job| job.session_id == session_id)
        && !cache
            .get_all::<EpiphanyCoordinatorRunReceipt>()?
            .iter()
            .any(|receipt| receipt.session_id == session_id)
        && !cache
            .get_all::<EpiphanyCoordinatorDeathRecovery>()?
            .iter()
            .any(|recovery| recovery.session_id == session_id))
}

fn recover_coordinator_run_after_exact_process_death_with_before_commit<F>(
    store_path: impl AsRef<Path>,
    recovery: &EpiphanyCoordinatorDeathRecovery,
    expected_objective: &str,
    before_commit: F,
) -> Result<EpiphanyRuntimeSession>
where
    F: FnOnce() -> Result<()>,
{
    validate_non_empty(expected_objective, "recovered coordinator objective")?;
    if recovery.schema_version != COORDINATOR_DEATH_RECOVERY_SCHEMA_VERSION
        || recovery.recovery_id != format!("coordinator-death-recovery-{}", recovery.session_id)
        || recovery.session_id
            != coordinator_run_session_id(
                &recovery.thread_id,
                Some(&recovery.resident_launch_digest),
            )?
        || recovery.resident_grant_id.trim().is_empty()
        || !recovery.resident_launch_digest.starts_with("sha256:")
        || recovery.process_id == 0
        || recovery.process_creation_token == 0
        || recovery.process_executable_path.trim().is_empty()
        || recovery.resident_started_at_millis == 0
        || !matches!(recovery.observation.as_str(), "exited" | "missing")
        || (recovery.observation == "missing" && recovery.exit_code.is_some())
        || recovery.private_state_exposed
    {
        return Err(anyhow!("coordinator death recovery contract is invalid"));
    }
    chrono::DateTime::parse_from_rfc3339(&recovery.recovered_at)
        .map_err(|error| anyhow!("coordinator death recovery timestamp is invalid: {error}"))?;
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let mut session = cache
        .get::<EpiphanyRuntimeSession>(&recovery.session_id)?
        .ok_or_else(|| anyhow!("coordinator death recovery session is absent"))?;
    let expected_event = coordinator_death_recovery_event(recovery);
    if session.status == EpiphanyRuntimeSessionStatus::Completed {
        let existing_recovery =
            cache.get::<EpiphanyCoordinatorDeathRecovery>(&recovery.recovery_id)?;
        let existing_event = cache.get::<EpiphanyRuntimeEvent>(&expected_event.event_id)?;
        if existing_recovery.as_ref() == Some(recovery)
            && existing_event.as_ref() == Some(&expected_event)
            && session.objective == expected_objective
            && session.updated_at == recovery.recovered_at
            && session.coordinator_note == expected_event.summary
        {
            return Ok(session);
        }
        return Err(anyhow!(
            "completed coordinator session does not match exact death recovery"
        ));
    }
    if session.status != EpiphanyRuntimeSessionStatus::Active
        || session.objective != expected_objective
    {
        return Err(anyhow!(
            "coordinator death recovery requires the exact active session objective"
        ));
    }
    let start_event_id = format!("event-coordinator-started-{}", recovery.session_id);
    let start_event = cache
        .get::<EpiphanyRuntimeEvent>(&start_event_id)?
        .ok_or_else(|| anyhow!("coordinator death recovery lost its start event"))?;
    let session_started_at = chrono::DateTime::parse_from_rfc3339(&session.created_at)
        .map_err(|error| anyhow!("coordinator session start timestamp is invalid: {error}"))?;
    let recovered_at = chrono::DateTime::parse_from_rfc3339(&recovery.recovered_at)
        .map_err(|error| anyhow!("coordinator death recovery timestamp is invalid: {error}"))?;
    if (session_started_at.timestamp_millis().max(0) as u64) < recovery.resident_started_at_millis
        || recovered_at < session_started_at
        || start_event.occurred_at != session.created_at
        || start_event.event_type != "coordinator.started"
        || start_event.source != "epiphany-mvp-coordinator"
        || start_event.session_id.as_deref() != Some(recovery.session_id.as_str())
        || start_event.job_id.is_some()
        || cache
            .get_all::<EpiphanyRuntimeJob>()?
            .iter()
            .any(|job| job.session_id == recovery.session_id)
        || cache
            .get_all::<EpiphanyCoordinatorRunReceipt>()?
            .iter()
            .any(|receipt| receipt.session_id == recovery.session_id)
        || cache
            .get_all::<EpiphanyCoordinatorDeathRecovery>()?
            .iter()
            .any(|existing| existing.session_id == recovery.session_id)
        || cache
            .get::<EpiphanyRuntimeEvent>(&expected_event.event_id)?
            .is_some()
    {
        return Err(anyhow!(
            "coordinator death recovery found substituted or competing authority"
        ));
    }
    let snapshot = cache.snapshot_envelopes();
    session.status = EpiphanyRuntimeSessionStatus::Completed;
    session.updated_at = recovery.recovered_at.clone();
    session.coordinator_note = expected_event.summary.clone();
    let replacements = vec![
        cache.prepare_entry(&session.session_id, &session)?.0,
        cache.prepare_entry(&recovery.recovery_id, recovery)?.0,
        cache
            .prepare_entry(&expected_event.event_id, &expected_event)?
            .0,
    ];
    before_commit()?;
    if !runtime_spine_backing_store(store_path)?
        .replace_and_append_if_snapshot_unchanged(&snapshot, replacements)?
    {
        return Err(anyhow!(
            "coordinator death recovery lost its full snapshot fence"
        ));
    }
    Ok(session)
}

fn finalize_coordinator_run_with_before_commit<F>(
    store_path: impl AsRef<Path>,
    receipt: &EpiphanyCoordinatorRunReceipt,
    before_commit: F,
) -> Result<EpiphanyRuntimeSession>
where
    F: FnOnce() -> Result<()>,
{
    validate_coordinator_run_receipt(receipt)?;
    if receipt.session_id
        != coordinator_run_session_id(
            &receipt.thread_id,
            receipt.resident_launch_digest.as_deref(),
        )?
    {
        return Err(anyhow!(
            "coordinator run receipt session is not bound to its run incarnation"
        ));
    }
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let mut session = cache
        .get::<EpiphanyRuntimeSession>(&receipt.session_id)?
        .ok_or_else(|| {
            anyhow!(
                "coordinator run receipt session {:?} does not exist",
                receipt.session_id
            )
        })?;
    let expected_event = coordinator_completion_event(receipt);
    let completion_event_id = expected_event.event_id.clone();
    let completion_summary = expected_event.summary.clone();
    if session.status == EpiphanyRuntimeSessionStatus::Completed {
        let existing_receipt = cache.get::<EpiphanyCoordinatorRunReceipt>(&receipt.receipt_id)?;
        let existing_event = cache.get::<EpiphanyRuntimeEvent>(&completion_event_id)?;
        if existing_receipt.as_ref() == Some(receipt)
            && existing_event.as_ref() == Some(&expected_event)
            && session.updated_at == receipt.created_at
            && session.coordinator_note == completion_summary
        {
            return Ok(session);
        }
        return Err(anyhow!(
            "completed coordinator session does not match the terminal run transaction"
        ));
    }
    if session.status != EpiphanyRuntimeSessionStatus::Active {
        return Err(anyhow!(
            "coordinator run session {:?} is not active",
            receipt.session_id
        ));
    }
    if cache
        .get_all::<EpiphanyRuntimeJob>()?
        .iter()
        .any(|job| job.session_id == receipt.session_id)
    {
        return Err(anyhow!(
            "coordinator run finalization refuses a session with runtime jobs"
        ));
    }
    if cache
        .get_all::<EpiphanyCoordinatorRunReceipt>()?
        .iter()
        .any(|existing| existing.session_id == receipt.session_id)
        || cache
            .get::<EpiphanyRuntimeEvent>(&completion_event_id)?
            .is_some()
    {
        return Err(anyhow!(
            "coordinator run session already has partial terminal authority"
        ));
    }
    let expected_session =
        cache.get_required_envelope::<EpiphanyRuntimeSession>(&receipt.session_id)?;
    session.status = EpiphanyRuntimeSessionStatus::Completed;
    session.updated_at = receipt.created_at.clone();
    session.coordinator_note = completion_summary;
    let replacements = vec![
        cache.prepare_entry(&session.session_id, &session)?.0,
        cache.prepare_entry(&receipt.receipt_id, receipt)?.0,
        cache
            .prepare_entry(&completion_event_id, &expected_event)?
            .0,
    ];
    before_commit()?;
    if !runtime_spine_backing_store(store_path)?
        .compare_and_swap_batch(&[expected_session], replacements)?
    {
        return Err(anyhow!(
            "coordinator run finalization lost its exact session transaction"
        ));
    }
    Ok(session)
}

pub fn coordinator_run_receipts(
    store_path: impl AsRef<Path>,
) -> Result<Vec<EpiphanyCoordinatorRunReceipt>> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    cache.get_all::<EpiphanyCoordinatorRunReceipt>()
}

pub fn retain_coordinator_run_receipts(
    store_path: impl AsRef<Path>,
    retain_recent: usize,
    preserve_receipt_ids: &BTreeSet<String>,
    retained_at: &str,
) -> Result<Option<EpiphanyCoordinatorRunReceiptRetentionHead>> {
    chrono::DateTime::parse_from_rfc3339(retained_at)
        .map_err(|error| anyhow!("coordinator receipt retention timestamp is invalid: {error}"))?;
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let mut receipts = cache.get_all::<EpiphanyCoordinatorRunReceipt>()?;
    let session_bound_receipt_ids = cache
        .get_all::<EpiphanyRuntimeSession>()?
        .into_iter()
        .flat_map(|session| {
            receipts
                .iter()
                .filter(move |receipt| receipt.session_id == session.session_id)
                .map(|receipt| receipt.receipt_id.clone())
        })
        .collect::<BTreeSet<_>>();
    let created_at_by_receipt = receipts
        .iter()
        .map(|receipt| {
            chrono::DateTime::parse_from_rfc3339(&receipt.created_at)
                .map(|created_at| (receipt.receipt_id.clone(), created_at))
                .map_err(|error| {
                    anyhow!(
                        "coordinator receipt {:?} has invalid created_at: {error}",
                        receipt.receipt_id
                    )
                })
        })
        .collect::<Result<BTreeMap<_, _>>>()?;
    receipts.sort_by(|left, right| {
        created_at_by_receipt[&left.receipt_id]
            .cmp(&created_at_by_receipt[&right.receipt_id])
            .then(left.receipt_id.cmp(&right.receipt_id))
    });
    let keep_recent = retain_recent.max(1);
    let recent_ids = receipts
        .iter()
        .rev()
        .take(keep_recent)
        .map(|receipt| receipt.receipt_id.as_str())
        .collect::<BTreeSet<_>>();
    let retired = receipts
        .iter()
        .filter(|receipt| {
            !recent_ids.contains(receipt.receipt_id.as_str())
                && !preserve_receipt_ids.contains(&receipt.receipt_id)
                && !session_bound_receipt_ids.contains(&receipt.receipt_id)
        })
        .collect::<Vec<_>>();
    if retired.is_empty() {
        return Ok(None);
    }

    let prior = cache.get::<EpiphanyCoordinatorRunReceiptRetentionHead>(
        COORDINATOR_RUN_RECEIPT_RETENTION_HEAD_KEY,
    )?;
    if let Some(head) = &prior {
        if head.schema_version != COORDINATOR_RUN_RECEIPT_RETENTION_HEAD_SCHEMA_VERSION
            || head.private_state_exposed
            || !head.retired_chain_digest.starts_with("sha256:")
        {
            return Err(anyhow!("coordinator run receipt retention head is invalid"));
        }
    }
    let snapshot = cache.snapshot_envelopes();
    let mut deletions = retired
        .iter()
        .map(|receipt| {
            snapshot
                .iter()
                .find(|entry| {
                    entry.r#type == EpiphanyCoordinatorRunReceipt::TYPE
                        && entry.key == receipt.receipt_id
                })
                .cloned()
                .ok_or_else(|| anyhow!("coordinator receipt lost its exact envelope"))
        })
        .collect::<Result<Vec<_>>>()?;
    deletions.sort_by(|left, right| {
        left.r#type
            .cmp(&right.r#type)
            .then(left.key.cmp(&right.key))
    });
    let mut status_counts = prior
        .as_ref()
        .map(|head| head.retired_status_counts.clone())
        .unwrap_or_default();
    for receipt in &retired {
        *status_counts.entry(receipt.status.clone()).or_default() += 1;
    }
    let mut digest = Sha256::new();
    let prior_digest = prior
        .as_ref()
        .map(|head| head.retired_chain_digest.as_str())
        .unwrap_or("coordinator-run-receipt-retention-root");
    digest.update((prior_digest.len() as u64).to_le_bytes());
    digest.update(prior_digest.as_bytes());
    for entry in &deletions {
        for bytes in [
            entry.r#type.as_bytes(),
            entry.key.as_bytes(),
            entry.payload.as_slice(),
        ] {
            digest.update((bytes.len() as u64).to_le_bytes());
            digest.update(bytes);
        }
    }
    let head = EpiphanyCoordinatorRunReceiptRetentionHead {
        schema_version: COORDINATOR_RUN_RECEIPT_RETENTION_HEAD_SCHEMA_VERSION.into(),
        revision: prior.as_ref().map_or(1, |head| head.revision + 1),
        retired_receipt_count: prior.as_ref().map_or(retired.len() as u64, |head| {
            head.retired_receipt_count + retired.len() as u64
        }),
        retired_status_counts: status_counts,
        retired_chain_digest: format!("sha256:{:x}", digest.finalize()),
        retained_at: retained_at.into(),
        private_state_exposed: false,
    };
    let (replacement, _) =
        cache.prepare_entry(COORDINATOR_RUN_RECEIPT_RETENTION_HEAD_KEY, &head)?;
    if !runtime_spine_backing_store(store_path)?.replace_and_delete_if_snapshot_unchanged(
        &snapshot,
        vec![replacement],
        &deletions,
    )? {
        return Err(anyhow!(
            "coordinator run receipt retention lost its snapshot fence"
        ));
    }
    Ok(Some(head))
}

pub fn complete_runtime_job(
    store_path: impl AsRef<Path>,
    options: RuntimeSpineJobResultOptions,
) -> Result<EpiphanyRuntimeJobResult> {
    validate_non_empty(&options.result_id, "result id")?;
    validate_non_empty(&options.job_id, "job id")?;
    validate_non_empty(&options.completed_at, "completed at")?;
    validate_non_empty(&options.verdict, "verdict")?;
    validate_non_empty(&options.summary, "summary")?;
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let mut job = cache
        .get::<EpiphanyRuntimeJob>(&options.job_id)?
        .ok_or_else(|| anyhow!("runtime job {:?} does not exist", options.job_id))?;
    if matches!(
        job.status,
        EpiphanyRuntimeJobStatus::Completed
            | EpiphanyRuntimeJobStatus::Failed
            | EpiphanyRuntimeJobStatus::Cancelled
    ) {
        return Err(anyhow!(
            "runtime job {:?} is already terminal",
            options.job_id
        ));
    }
    if cache
        .get::<EpiphanyRuntimeJobResult>(&options.result_id)?
        .is_some()
    {
        return Err(anyhow!(
            "runtime job result {:?} already exists",
            options.result_id
        ));
    }
    let event_id = format!("event-job-completed-{}", options.job_id);
    if cache.get::<EpiphanyRuntimeEvent>(&event_id)?.is_some() {
        return Err(anyhow!(
            "runtime job completion event {:?} already exists",
            event_id
        ));
    }
    let snapshot = cache.snapshot_envelopes();
    let job_envelope = snapshot
        .iter()
        .find(|entry| entry.r#type == EpiphanyRuntimeJob::TYPE && entry.key == options.job_id)
        .cloned()
        .ok_or_else(|| anyhow!("runtime job lost its exact envelope"))?;
    let terminal_status = terminal_status_for_verdict(&options.verdict);
    job.status = terminal_status;
    job.updated_at = options.completed_at.clone();
    job.summary = options.summary.clone();
    job.artifact_refs = merge_refs(&job.artifact_refs, &options.artifact_refs);
    let result = EpiphanyRuntimeJobResult {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        result_id: options.result_id.clone(),
        job_id: options.job_id.clone(),
        session_id: job.session_id.clone(),
        role: job.role.clone(),
        verdict: options.verdict,
        summary: options.summary,
        completed_at: options.completed_at.clone(),
        next_safe_move: options.next_safe_move,
        evidence_refs: options.evidence_refs,
        artifact_refs: options.artifact_refs,
        metadata: BTreeMap::new(),
        decision_context_id: options.decision_context_id,
    };
    let event = EpiphanyRuntimeEvent {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        event_id,
        occurred_at: options.completed_at,
        event_type: "job.completed".to_string(),
        source: "runtime-spine".to_string(),
        session_id: Some(result.session_id.clone()),
        job_id: Some(options.job_id),
        summary: format!(
            "Native runtime job completed with verdict {}.",
            result.verdict
        ),
        metadata: BTreeMap::from([("resultId".to_string(), result.result_id.clone())]),
    };
    let mut expected = vec![job_envelope];
    let mut writes = vec![
        cache.prepare_entry(&job.job_id, &job)?.0,
        cache.prepare_entry(&result.result_id, &result)?.0,
        cache.prepare_entry(&event.event_id, &event)?.0,
    ];
    let claim_id = worker_process_claim_id(&result.job_id);
    if let Some(claim) = cache.get::<EpiphanyRuntimeWorkerProcessClaim>(&claim_id)? {
        match crate::WorkerProcessStatus::parse(&claim.status)? {
            crate::WorkerProcessStatus::Active => {
                let claim_envelope = snapshot
                    .iter()
                    .find(|entry| {
                        entry.r#type == EpiphanyRuntimeWorkerProcessClaim::TYPE
                            && entry.key == claim_id
                    })
                    .cloned()
                    .ok_or_else(|| anyhow!("runtime worker completion lost its claim envelope"))?;
                let mut terminal = claim;
                if let Some(reorient) =
                    cache.get::<EpiphanyRuntimeReorientWorkerResult>(&result.job_id)?
                {
                    terminal.status = crate::WorkerProcessStatus::TerminalResult.as_str().into();
                    terminal.terminal_authority_id = Some(reorient.result_id);
                } else {
                    terminal.status = crate::WorkerProcessStatus::TerminalFailure.as_str().into();
                    terminal.terminal_authority_id = Some(result.result_id.clone());
                }
                terminal.terminal_at = Some(result.completed_at.clone());
                expected.push(claim_envelope);
                writes.push(cache.prepare_entry(&claim_id, &terminal)?.0);
            }
            crate::WorkerProcessStatus::TerminalResult => {}
            crate::WorkerProcessStatus::Claimed => {
                return Err(anyhow!(
                    "runtime worker job cannot complete before activation"
                ));
            }
            status => {
                return Err(anyhow!(
                    "runtime worker job completion found process status {:?}",
                    status.as_str()
                ));
            }
        }
    }
    if !runtime_spine_backing_store(store_path.as_ref())?
        .compare_and_swap_batch(&expected, writes)?
    {
        return Err(anyhow!("runtime job completion lost its exact snapshot"));
    }
    Ok(result)
}

pub fn append_runtime_event(
    store_path: impl AsRef<Path>,
    options: RuntimeSpineEventOptions,
) -> Result<EpiphanyRuntimeEvent> {
    validate_non_empty(&options.event_id, "event id")?;
    validate_non_empty(&options.occurred_at, "occurred at")?;
    validate_non_empty(&options.event_type, "event type")?;
    validate_non_empty(&options.source, "source")?;
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    if cache
        .get::<EpiphanyRuntimeEvent>(&options.event_id)?
        .is_some()
    {
        return Err(anyhow!(
            "runtime event {:?} already exists",
            options.event_id
        ));
    }
    let event = EpiphanyRuntimeEvent {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        event_id: options.event_id.clone(),
        occurred_at: options.occurred_at,
        event_type: options.event_type,
        source: options.source,
        session_id: options.session_id,
        job_id: options.job_id,
        summary: options.summary,
        metadata: BTreeMap::new(),
    };
    cache.put(&options.event_id, &event)?;
    Ok(event)
}

pub fn runtime_spine_status(store_path: impl AsRef<Path>) -> Result<EpiphanyRuntimeSpineStatus> {
    let store_path = store_path.as_ref();
    if !store_path.exists() {
        return Ok(EpiphanyRuntimeSpineStatus {
            store: store_path.display().to_string(),
            present: false,
            runtime_id: None,
            display_name: None,
            sessions: 0,
            active_sessions: 0,
            jobs: 0,
            open_jobs: 0,
            job_results: 0,
            events: 0,
            tool_invocation_intents: 0,
            tool_invocation_receipts: 0,
            pending_tool_invocations: 0,
            supported_document_types: Vec::new(),
        });
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache
        .pull_all_backing_stores()
        .with_context(|| format!("failed to read runtime spine {}", store_path.display()))?;
    let identity = cache.get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?;
    let sessions = cache.get_all::<EpiphanyRuntimeSession>()?;
    let jobs = cache.get_all::<EpiphanyRuntimeJob>()?;
    let job_results = cache.get_all::<EpiphanyRuntimeJobResult>()?;
    let events = cache.get_all::<EpiphanyRuntimeEvent>()?;
    let tool_intents = cache.get_all::<EpiphanyToolInvocationIntent>()?;
    let tool_receipts = cache.get_all::<EpiphanyToolInvocationReceipt>()?;
    let receipt_intent_ids = tool_receipts
        .iter()
        .map(|receipt| receipt.intent_id.as_str())
        .collect::<BTreeSet<_>>();
    let active_sessions = sessions
        .iter()
        .filter(|session| {
            matches!(
                session.status,
                EpiphanyRuntimeSessionStatus::Active
                    | EpiphanyRuntimeSessionStatus::WaitingForReview
            )
        })
        .count();
    let open_jobs = jobs
        .iter()
        .filter(|job| {
            matches!(
                job.status,
                EpiphanyRuntimeJobStatus::Queued
                    | EpiphanyRuntimeJobStatus::Running
                    | EpiphanyRuntimeJobStatus::WaitingForReview
            )
        })
        .count();
    Ok(EpiphanyRuntimeSpineStatus {
        store: store_path.display().to_string(),
        present: identity.is_some(),
        runtime_id: identity.as_ref().map(|item| item.runtime_id.clone()),
        display_name: identity.as_ref().map(|item| item.display_name.clone()),
        sessions: sessions.len(),
        active_sessions,
        jobs: jobs.len(),
        open_jobs,
        job_results: job_results.len(),
        events: events.len(),
        tool_invocation_intents: tool_intents.len(),
        tool_invocation_receipts: tool_receipts.len(),
        pending_tool_invocations: tool_intents
            .iter()
            .filter(|intent| !receipt_intent_ids.contains(intent.intent_id.as_str()))
            .count(),
        supported_document_types: identity
            .is_some()
            .then(runtime_registered_document_types)
            .unwrap_or_default(),
    })
}

pub fn runtime_tool_invocation_statuses(
    store_path: impl AsRef<Path>,
) -> Result<Vec<EpiphanyToolInvocationStatus>> {
    let store_path = store_path.as_ref();
    if !store_path.exists() {
        return Ok(Vec::new());
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache
        .pull_all_backing_stores()
        .with_context(|| format!("failed to read runtime spine {}", store_path.display()))?;
    let mut receipts = cache
        .get_all::<EpiphanyToolInvocationReceipt>()?
        .into_iter()
        .map(|receipt| (receipt.intent_id.clone(), receipt))
        .collect::<BTreeMap<_, _>>();
    let mut statuses = cache
        .get_all::<EpiphanyToolInvocationIntent>()?
        .into_iter()
        .map(|intent| {
            let receipt = receipts.remove(&intent.intent_id);
            EpiphanyToolInvocationStatus {
                intent_id: intent.intent_id,
                adapter: intent.adapter,
                server: intent.server,
                tool_name: intent.tool_name,
                call_id: intent.call_id,
                model_request_id: intent.model_request_id,
                caller: intent.caller,
                reason: intent.reason,
                created_at: intent.created_at,
                status: receipt
                    .as_ref()
                    .map(|receipt| receipt.status.clone())
                    .unwrap_or_else(|| "pending".to_string()),
                receipt_id: receipt.as_ref().map(|receipt| receipt.receipt_id.clone()),
                completed_at: receipt.as_ref().map(|receipt| receipt.completed_at.clone()),
                error: receipt.and_then(|receipt| receipt.error),
            }
        })
        .collect::<Vec<_>>();
    statuses.sort_by(|left, right| {
        left.created_at
            .cmp(&right.created_at)
            .then_with(|| left.intent_id.cmp(&right.intent_id))
    });
    Ok(statuses)
}

pub fn runtime_hello_frame(store_path: impl AsRef<Path>) -> Result<Vec<u8>> {
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    let identity = require_identity(&cache)?;
    let message = CultNetMessage::Hello {
        runtime_id: identity.runtime_id,
        runtime_kind: identity.runtime_kind,
        agent_id: Some("self".to_string()),
        role: Some("coordinator".to_string()),
        display_name: Some(identity.display_name),
        supported_document_types: Some(runtime_registered_document_types()),
        supported_mutation_contracts: Some(epiphany_mutation_contracts()),
        supported_message_versions: Some(vec![
            "cultnet.hello.v0".to_string(),
            "cultnet.document_put.v0".to_string(),
            "cultnet.snapshot_request.v0".to_string(),
            "cultnet.snapshot_response.v0".to_string(),
            "cultnet.schema_catalog_request.v0".to_string(),
            "cultnet.schema_catalog_response.v0".to_string(),
        ]),
        transport_profiles: None,
        supports_schema_catalog: Some(true),
    };
    let payload = encode_cultnet_message_to_vec(&message, CultNetWireContract::CultNetSchemaV0)?;
    encode_frame(&payload)
}

pub fn write_runtime_hello_frame(
    store_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<usize> {
    let output_path = output_path.as_ref();
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let frame = runtime_hello_frame(store_path)?;
    fs::write(output_path, &frame)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(frame.len())
}

pub fn epiphany_schema_registry() -> Result<CultNetSchemaRegistry> {
    let mut registry = builtin_schema_registry()?;
    let schema_root = epiphany_schema_root();
    let index_path = schema_root.join("index.json");
    let raw_index = fs::read_to_string(&index_path)
        .with_context(|| format!("failed to read {}", index_path.display()))?;
    let index: EpiphanyCultNetSchemaIndex = serde_json::from_str(&raw_index)
        .with_context(|| format!("failed to parse {}", index_path.display()))?;
    if index.schema_version.trim().is_empty() {
        return Err(anyhow!(
            "CultNet schema index at {} is missing schemaVersion",
            index_path.display()
        ));
    }

    for entry in index.schemas {
        let schema_path = schema_root.join(&entry.path);
        let schema_json = fs::read_to_string(&schema_path)
            .with_context(|| format!("failed to read {}", schema_path.display()))?;
        registry.register(CultNetSchemaRegistration {
            schema_id: entry.schema_id,
            kind: entry.kind,
            wire_contracts: entry.wire_contracts,
            schema_version: entry.schema_version,
            document_type: entry.document_type,
            title: entry.title,
            schema_json: Some(schema_json),
        })?;
    }

    Ok(registry)
}

pub fn runtime_schema_catalog_response(
    message_id: impl Into<String>,
    include_schema_json: bool,
    schema_ids: Option<Vec<String>>,
    kinds: Option<Vec<CultNetSchemaKind>>,
) -> Result<CultNetMessage> {
    let registry = epiphany_schema_registry()?;
    registry.create_catalog_response(&CultNetMessage::SchemaCatalogRequest {
        message_id: message_id.into(),
        include_schema_json: Some(include_schema_json),
        schema_ids,
        kinds,
    })
}

pub fn write_runtime_schema_catalog_json(
    output_path: impl AsRef<Path>,
    include_schema_json: bool,
) -> Result<usize> {
    let output_path = output_path.as_ref();
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let response = runtime_schema_catalog_response(
        "runtime-spine-schema-catalog".to_string(),
        include_schema_json,
        None,
        None,
    )?;
    let body = serde_json::to_vec_pretty(&response)?;
    fs::write(output_path, &body)
        .with_context(|| format!("failed to write {}", output_path.display()))?;
    Ok(body.len())
}

fn epiphany_schema_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("epiphany-core has no parent repo root")
        .join(CULTNET_SCHEMA_INDEX_RELATIVE)
        .parent()
        .expect("cultnet schema index has no parent directory")
        .to_path_buf()
}

fn require_identity(cache: &CultCache) -> Result<EpiphanyRuntimeIdentity> {
    cache
        .get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("runtime spine is missing identity; run init first"))
}

fn require_runtime_identity_not_archived(
    cache: &CultCache,
    identity_kind: &str,
    identity: &str,
) -> Result<()> {
    let collision = cache
        .get_all::<EpiphanyArchivedRuntimeSession>()?
        .into_iter()
        .find(|archive| match identity_kind {
            "session" => archive.session_id == identity,
            "job" => archive.job_ids.iter().any(|item| item == identity),
            "model-request" => archive
                .model_request_ids
                .iter()
                .any(|item| item == identity),
            "tool-intent" => archive.tool_intent_ids.iter().any(|item| item == identity),
            _ => false,
        });
    if let Some(archive) = collision {
        return Err(anyhow!(
            "runtime {identity_kind} identity {identity:?} was retired by archive {:?}",
            archive.archive_id
        ));
    }
    Ok(())
}

pub fn runtime_registered_document_types() -> Vec<String> {
    let mut document_types = Vec::new();
    for contract in epiphany_mutation_contracts() {
        if !document_types.contains(&contract.document_type) {
            document_types.push(contract.document_type);
        }
    }
    document_types
}

fn mutation_contract(
    document_type: impl Into<String>,
    payload_schema_version: impl Into<String>,
    operations: Vec<CultNetDocumentOperation>,
    authority: CultNetMutationAuthority,
    intent_document_types: Vec<&str>,
    receipt_document_types: Vec<&str>,
    notes: Vec<&str>,
) -> CultNetDocumentMutationContract {
    CultNetDocumentMutationContract {
        document_type: document_type.into(),
        payload_schema_version: Some(payload_schema_version.into()),
        operations,
        authority,
        intent_document_types: (!intent_document_types.is_empty()).then(|| {
            intent_document_types
                .into_iter()
                .map(str::to_string)
                .collect()
        }),
        receipt_document_types: (!receipt_document_types.is_empty()).then(|| {
            receipt_document_types
                .into_iter()
                .map(str::to_string)
                .collect()
        }),
        notes: (!notes.is_empty()).then(|| notes.into_iter().map(str::to_string).collect()),
    }
}

fn epiphany_mutation_contracts() -> Vec<CultNetDocumentMutationContract> {
    vec![
        mutation_contract(
            crate::EpiphanyRepoModelIdentityDocument::TYPE,
            crate::EpiphanyRepoModelIdentityDocument::SCHEMA_NAME,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![crate::EpiphanyMindCommitReceipt::TYPE],
            vec!["Keyed RepoModel identity is written only by the local Mind commit path."],
        ),
        mutation_contract(
            crate::EpiphanyRepoModelDomainDocument::TYPE,
            crate::EpiphanyRepoModelDomainDocument::SCHEMA_NAME,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![crate::EpiphanyMindCommitReceipt::TYPE],
            vec!["Keyed RepoModel domains are written only by the local Mind commit path."],
        ),
        mutation_contract(
            crate::EpiphanyRepoModelNodeDocument::TYPE,
            crate::EpiphanyRepoModelNodeDocument::SCHEMA_NAME,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![crate::EpiphanyMindCommitReceipt::TYPE],
            vec!["Keyed RepoModel nodes are written only by the local Mind commit path."],
        ),
        mutation_contract(
            crate::EpiphanyRepoModelEdgeDocument::TYPE,
            crate::EpiphanyRepoModelEdgeDocument::SCHEMA_NAME,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![crate::EpiphanyMindCommitReceipt::TYPE],
            vec!["Keyed RepoModel edges are written only by the local Mind commit path."],
        ),
        mutation_contract(
            crate::EpiphanyRepoModelSummaryDocument::TYPE,
            crate::EpiphanyRepoModelSummaryDocument::SCHEMA_NAME,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![crate::EpiphanyMindCommitReceipt::TYPE],
            vec!["Keyed RepoModel summaries are written only by the local Mind commit path."],
        ),
        mutation_contract(
            crate::EpiphanyRepoModelFrontierDocument::TYPE,
            crate::EpiphanyRepoModelFrontierDocument::SCHEMA_NAME,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![crate::EpiphanyMindCommitReceipt::TYPE],
            vec!["Keyed RepoModel frontier items are written only by the local Mind commit path."],
        ),
        mutation_contract(
            crate::EpiphanyRepoModelLifecycleReceiptDocument::TYPE,
            crate::EpiphanyRepoModelLifecycleReceiptDocument::SCHEMA_NAME,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![crate::EpiphanyMindCommitReceipt::TYPE],
            vec![
                "Keyed RepoModel lifecycle receipts are written only by the local Mind commit path.",
            ],
        ),
        mutation_contract(
            crate::EpiphanyRepoModelClaimObligationsDocument::TYPE,
            crate::EpiphanyRepoModelClaimObligationsDocument::SCHEMA_NAME,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![crate::EpiphanyMindCommitReceipt::TYPE],
            vec!["Keyed RepoModel claim obligations are derived and committed by local Mind."],
        ),
        mutation_contract(
            crate::AtlasSurfaceOffer::TYPE,
            crate::AtlasSurfaceOffer::SCHEMA_NAME,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![crate::ATLAS_SURFACE_OFFER_WRITE_INTENT_SCHEMA],
            vec![crate::EpiphanyMindCommitReceipt::TYPE],
            vec![
                "Provider Modeling owns local surface offers through typed Atlas planners and Mind CAS.",
            ],
        ),
        mutation_contract(
            crate::AtlasDependencyClaim::TYPE,
            crate::AtlasDependencyClaim::SCHEMA_NAME,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![crate::ATLAS_DEPENDENCY_CLAIM_WRITE_INTENT_SCHEMA],
            vec![crate::EpiphanyMindCommitReceipt::TYPE],
            vec![
                "Consumer Modeling owns local dependency claims through typed Atlas planners and Mind CAS.",
            ],
        ),
        mutation_contract(
            crate::AtlasDependencyVerification::TYPE,
            crate::AtlasDependencyVerification::SCHEMA_NAME,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![crate::ATLAS_DEPENDENCY_VERIFICATION_WRITE_INTENT_SCHEMA],
            vec![crate::EpiphanyMindCommitReceipt::TYPE],
            vec![
                "Soul owns exact local claim/offer verification through its dedicated Atlas planner.",
            ],
        ),
        mutation_contract(
            crate::AtlasDependencyImpact::TYPE,
            crate::AtlasDependencyImpact::SCHEMA_NAME,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![crate::ATLAS_DEPENDENCY_IMPACT_WRITE_INTENT_SCHEMA],
            vec![crate::EpiphanyMindCommitReceipt::TYPE],
            vec![
                "Consumer Self owns local dependency impacts through its dedicated Atlas planner.",
            ],
        ),
        mutation_contract(
            crate::USER_OBJECTIVE_INTAKE_TYPE,
            crate::USER_OBJECTIVE_INTAKE_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::IntentSubmit,
            ],
            CultNetMutationAuthority::Coordinator,
            vec![crate::USER_OBJECTIVE_INTAKE_TYPE],
            vec![crate::EpiphanyMindCommitReceipt::TYPE],
            vec![
                "The human owns the initial objective assertion; Self atomically records it with the keyed Mind objective and a typed operator-provenance commit receipt.",
                "Thread identity is provenance and does not own objective causality.",
                "Objective replacement requires a separate reviewed adoption flow.",
            ],
        ),
        mutation_contract(
            RUNTIME_IDENTITY_TYPE,
            RUNTIME_SPINE_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["Runtime identity is advertised by the coordinator, not remotely mutated."],
        ),
        mutation_contract(
            RUNTIME_SESSION_TYPE,
            RUNTIME_SPINE_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::IntentSubmit,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::Coordinator,
            vec!["epiphany.runtime.session_intent.v0"],
            vec!["epiphany.swarm_control_receipt.v0"],
            vec!["Sessions change through coordinator-reviewed typed intents."],
        ),
        mutation_contract(
            RUNTIME_JOB_TYPE,
            RUNTIME_SPINE_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::IntentSubmit,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::Coordinator,
            vec!["epiphany.heartbeat_pump_intent.v0"],
            vec!["epiphany.swarm_control_receipt.v0"],
            vec![
                "Heartbeat activation owns agent work; external callers submit intents and watch receipts.",
            ],
        ),
        mutation_contract(
            RUNTIME_WORKER_LAUNCH_REQUEST_TYPE,
            RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::IntentSubmit,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::Coordinator,
            vec![RUNTIME_WORKER_LAUNCH_REQUEST_TYPE],
            vec![RUNTIME_JOB_TYPE],
            vec![
                "Worker launch requests are typed task-intent documents; runtime jobs are lifecycle receipts, not the source of work intent.",
                "Core/coordinator policy owns the launch yes/no; the Epiphany-Codex bridge translates between CultNet-shaped intent and Codex JSON only.",
                "Codex-hosted executors may gather host facts and perform side effects after the verdict, with readable receipts.",
            ],
        ),
        mutation_contract(
            RUNTIME_ROLE_WORKER_RESULT_TYPE,
            RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Role worker results preserve the typed finding payload; generic runtime job results are lifecycle receipts.",
            ],
        ),
        mutation_contract(
            RUNTIME_WORKER_PROCESS_CLAIM_TYPE,
            RUNTIME_WORKER_PROCESS_CLAIM_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "The worker claims its native process before model/tool work; the spawning coordinator alone presents the one-use activation preimage.",
                "Terminal result and exact process death replace the same claim; absence and age are not terminal authority.",
            ],
        ),
        mutation_contract(
            ARCHIVED_RUNTIME_WORKER_ATTEMPT_TYPE,
            ARCHIVED_RUNTIME_WORKER_ATTEMPT_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Runtime spine atomically replaces one exact terminal typed worker family with this per-attempt tombstone only after resident liveness clears.",
                "A terminal-result tombstone preserves authenticated fulfillment identity; it does not own producer semantic companions or Mind admission.",
            ],
        ),
        mutation_contract(
            RUNTIME_REORIENT_WORKER_RESULT_TYPE,
            RUNTIME_REORIENT_WORKER_RESULT_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Reorient worker results preserve continuity findings separately from generic runtime lifecycle receipts.",
            ],
        ),
        mutation_contract(
            RUNTIME_JOB_RESULT_TYPE,
            RUNTIME_SPINE_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Job results are evidence records; review and acceptance are separate typed flows.",
            ],
        ),
        mutation_contract(
            MIND_THOUGHT_TYPE,
            MIND_THOUGHT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::IntentSubmit,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::Coordinator,
            vec![MIND_THOUGHT_TYPE],
            vec![MIND_GATEWAY_REVIEW_TYPE, MIND_STATE_REJECTION_RECEIPT_TYPE],
            vec![
                "Sub-agent output enters Epiphany as thought, not durable state authority.",
                "The Mind contract is the gateway between worker output and persistent state.",
            ],
        ),
        mutation_contract(
            MIND_STATE_EFFECT_PROPOSAL_TYPE,
            MIND_STATE_EFFECT_PROPOSAL_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::IntentSubmit,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::Coordinator,
            vec![MIND_STATE_EFFECT_PROPOSAL_TYPE],
            vec![
                MIND_GATEWAY_REVIEW_TYPE,
                MIND_STATE_COMMIT_RECEIPT_TYPE,
                MIND_STATE_REJECTION_RECEIPT_TYPE,
            ],
            vec![
                "Mind is the persistent state guardian: role acceptance, reorientation acceptance, Persona Interpreter effects, selfPatch, evidence, scratch, checkpoints, graph changes, and objective changes share this gate.",
                "Workers and public Verse ingress propose effects; Mind accepts, refuses, or holds them before any durable state mutation.",
            ],
        ),
        mutation_contract(
            MIND_GATEWAY_REVIEW_TYPE,
            MIND_GATEWAY_REVIEW_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Mind reviews are durable receipts explaining accepted, refused, or held state effects.",
            ],
        ),
        mutation_contract(
            MIND_STATE_COMMIT_RECEIPT_TYPE,
            MIND_STATE_COMMIT_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "A commit receipt is proof that Mind, not the worker, admitted a proposed effect into durable state.",
            ],
        ),
        mutation_contract(
            MIND_STATE_REJECTION_RECEIPT_TYPE,
            MIND_STATE_REJECTION_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "A rejection receipt preserves why a thought or state effect was refused without mutating the Mind.",
            ],
        ),
        mutation_contract(
            MIND_VERSE_ADOPTION_RECEIPT_TYPE,
            MIND_VERSE_ADOPTION_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Foreign or public Verse material is thought weather until local Mind emits an adoption receipt.",
                "The global Verse never receives private state authority by being interesting.",
            ],
        ),
        mutation_contract(
            SUBSTRATE_GATE_REPO_ACCESS_REQUEST_TYPE,
            SUBSTRATE_GATE_REPO_ACCESS_REQUEST_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::IntentSubmit,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::Coordinator,
            vec![SUBSTRATE_GATE_REPO_ACCESS_REQUEST_TYPE],
            vec![
                SUBSTRATE_GATE_REPO_ACCESS_REVIEW_TYPE,
                SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE,
                SUBSTRATE_GATE_REPO_ACCESS_REFUSAL_RECEIPT_TYPE,
            ],
            vec![
                "Substrate Gate is the repository access protocol: reads, indexing, edits, commands, and bridge operations must be requested through this contract.",
                "Hands mutates only after a scoped Substrate Gate grant; Eyes inspects only after a scoped Substrate Gate read/index grant.",
            ],
        ),
        mutation_contract(
            SUBSTRATE_GATE_REPO_ACCESS_REVIEW_TYPE,
            SUBSTRATE_GATE_REPO_ACCESS_REVIEW_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Substrate Gate reviews explain granted/refused repo paths, operations, commands, and bridge surfaces.",
            ],
        ),
        mutation_contract(
            SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE,
            SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["A Substrate Gate grant receipt scopes a permitted repo touch."],
        ),
        mutation_contract(
            SUBSTRATE_GATE_REPO_ACCESS_REFUSAL_RECEIPT_TYPE,
            SUBSTRATE_GATE_REPO_ACCESS_REFUSAL_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["A Substrate Gate refusal receipt preserves why repo access was denied."],
        ),
        mutation_contract(
            SUBSTRATE_GATE_REPO_SNAPSHOT_RECEIPT_TYPE,
            SUBSTRATE_GATE_REPO_SNAPSHOT_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["Repo snapshots are evidence projections from Substrate-Gate-scoped access."],
        ),
        mutation_contract(
            SUBSTRATE_GATE_REPO_MUTATION_RECEIPT_TYPE,
            SUBSTRATE_GATE_REPO_MUTATION_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Repo mutation receipts prove Substrate Gate granted the substrate touch before Hands changed files or ran repo-affecting commands.",
            ],
        ),
        mutation_contract(
            EYES_EVIDENCE_REQUEST_TYPE,
            EYES_EVIDENCE_REQUEST_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::IntentSubmit,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::Coordinator,
            vec![EYES_EVIDENCE_REQUEST_TYPE],
            vec![
                EYES_EVIDENCE_REVIEW_TYPE,
                EYES_SOURCE_LOOKUP_RECEIPT_TYPE,
                EYES_EVIDENCE_PACKET_TYPE,
                EYES_EVIDENCE_REFUSAL_RECEIPT_TYPE,
            ],
            vec![
                "Eyes is the evidence ingress guardian: source-grounded claims, provenance, uncertainty, and evidence packets enter through this contract.",
                "Substrate Gate grants substrate access; Eyes decides what was actually inspected and what other organs may cite.",
            ],
        ),
        mutation_contract(
            EYES_EVIDENCE_REVIEW_TYPE,
            EYES_EVIDENCE_REVIEW_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["Eyes reviews explain whether a claim is source-grounded, uncertain, or refused."],
        ),
        mutation_contract(
            EYES_SOURCE_LOOKUP_RECEIPT_TYPE,
            EYES_SOURCE_LOOKUP_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Source lookup receipts prove what was searched or inspected under a Substrate Gate grant.",
            ],
        ),
        mutation_contract(
            EYES_EVIDENCE_PACKET_TYPE,
            EYES_EVIDENCE_PACKET_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Evidence packets carry provenance, uncertainty, and source refs for the other organs.",
            ],
        ),
        mutation_contract(
            EYES_EVIDENCE_REFUSAL_RECEIPT_TYPE,
            EYES_EVIDENCE_REFUSAL_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["Evidence refusal receipts preserve why Eyes would not certify a claim."],
        ),
        mutation_contract(
            HANDS_ACTION_INTENT_TYPE,
            HANDS_ACTION_INTENT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::IntentSubmit,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::Coordinator,
            vec![HANDS_ACTION_INTENT_TYPE],
            vec![
                HANDS_ACTION_REVIEW_TYPE,
                HANDS_COMMAND_RECEIPT_TYPE,
                HANDS_PATCH_RECEIPT_TYPE,
                HANDS_COMMIT_RECEIPT_TYPE,
                HANDS_PR_RECEIPT_TYPE,
                HANDS_ROLLBACK_RECEIPT_TYPE,
                HANDS_ACTION_REFUSAL_RECEIPT_TYPE,
            ],
            vec![
                "Hands is the action organ: commands, patches, commits, PRs, and rollbacks enter as bounded action intents.",
                "Substrate Gate grants substrate access before Hands mutates; Soul verifies consequences after.",
            ],
        ),
        mutation_contract(
            HANDS_ACTION_REVIEW_TYPE,
            HANDS_ACTION_REVIEW_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["Hands reviews explain allowed, refused, sequenced, or delegated action."],
        ),
        mutation_contract(
            HANDS_COMMAND_RECEIPT_TYPE,
            HANDS_COMMAND_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["Command receipts prove what command ran and under which Substrate Gate grant."],
        ),
        mutation_contract(
            HANDS_PATCH_RECEIPT_TYPE,
            HANDS_PATCH_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["Patch receipts prove file mutations and the scoped grant that permitted them."],
        ),
        mutation_contract(
            HANDS_COMMIT_RECEIPT_TYPE,
            HANDS_COMMIT_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![HANDS_PR_RECEIPT_TYPE],
            vec!["Commit receipts preserve publication consequences after verification."],
        ),
        mutation_contract(
            HANDS_PR_RECEIPT_TYPE,
            HANDS_PR_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["PR receipts preserve outward publication consequences for operator review."],
        ),
        mutation_contract(
            HANDS_ROLLBACK_RECEIPT_TYPE,
            HANDS_ROLLBACK_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["Rollback receipts prove failed action was unwound instead of hidden."],
        ),
        mutation_contract(
            HANDS_ACTION_REFUSAL_RECEIPT_TYPE,
            HANDS_ACTION_REFUSAL_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["Hands refusal receipts preserve why an action intent was denied."],
        ),
        mutation_contract(
            crate::REPO_FRONTIER_RELINQUISHMENT_RECEIPT_TYPE,
            crate::REPO_FRONTIER_RELINQUISHMENT_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Mind relinquishment receipts prove that an exact Hands refusal retired route authority without a repository consequence.",
            ],
        ),
        mutation_contract(
            SOUL_VERIFICATION_REQUEST_TYPE,
            SOUL_VERIFICATION_REQUEST_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::IntentSubmit,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::Coordinator,
            vec![SOUL_VERIFICATION_REQUEST_TYPE],
            vec![
                SOUL_INVARIANT_CHECK_TYPE,
                SOUL_VERDICT_RECEIPT_TYPE,
                SOUL_REGRESSION_RECEIPT_TYPE,
                SOUL_REVIEW_RECEIPT_TYPE,
                SOUL_VERIFICATION_REFUSAL_RECEIPT_TYPE,
            ],
            vec![
                "Soul is the verification organ: invariants, tests, review, falsification, and refusal enter here.",
                "Soul verdicts inform Mind admission; they do not mutate repo or state by themselves.",
            ],
        ),
        mutation_contract(
            SOUL_INVARIANT_CHECK_TYPE,
            SOUL_INVARIANT_CHECK_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Invariant checks identify which promise was tested and whether old paths can still violate it.",
            ],
        ),
        mutation_contract(
            SOUL_VERDICT_RECEIPT_TYPE,
            SOUL_VERDICT_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["Verdict receipts are proof of sanctity or proof of failure."],
        ),
        mutation_contract(
            SOUL_REGRESSION_RECEIPT_TYPE,
            SOUL_REGRESSION_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Regression receipts preserve violated invariants and surviving obsolete authorities.",
            ],
        ),
        mutation_contract(
            SOUL_REVIEW_RECEIPT_TYPE,
            SOUL_REVIEW_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["Review receipts preserve risks, missing tests, and falsification notes."],
        ),
        mutation_contract(
            SOUL_VERIFICATION_REFUSAL_RECEIPT_TYPE,
            SOUL_VERIFICATION_REFUSAL_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Soul refusal receipts preserve why a verification request could not honestly be performed.",
            ],
        ),
        mutation_contract(
            CONTINUITY_PACKET_TYPE,
            CONTINUITY_PACKET_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::IntentSubmit,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::Coordinator,
            vec![CONTINUITY_PACKET_TYPE],
            vec![
                CONTINUITY_COMPACTION_CHECKPOINT_TYPE,
                CONTINUITY_SLEEP_DISTILLATION_TYPE,
                CONTINUITY_RECOVERY_RECEIPT_TYPE,
                CONTINUITY_STALE_TURN_REPAIR_TYPE,
                CONTINUITY_REFUSAL_RECEIPT_TYPE,
            ],
            vec![
                "Continuity is deterministic protocol machinery: compaction, sleep, recovery, stale-turn repair, and handoff packets enter here.",
                "Continuity preserves survival across rupture; Mind admits durable state.",
            ],
        ),
        mutation_contract(
            CONTINUITY_COMPACTION_CHECKPOINT_TYPE,
            CONTINUITY_COMPACTION_CHECKPOINT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["Compaction checkpoints preserve hot context before rupture."],
        ),
        mutation_contract(
            CONTINUITY_SLEEP_DISTILLATION_TYPE,
            CONTINUITY_SLEEP_DISTILLATION_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["Sleep distillation receipts separate durable lessons from rumination residue."],
        ),
        mutation_contract(
            CONTINUITY_RECOVERY_RECEIPT_TYPE,
            CONTINUITY_RECOVERY_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["Recovery receipts explain what survived and what must be regathered."],
        ),
        mutation_contract(
            CONTINUITY_STALE_TURN_REPAIR_TYPE,
            CONTINUITY_STALE_TURN_REPAIR_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Stale-turn repair receipts close abandoned work without pretending it completed.",
            ],
        ),
        mutation_contract(
            CONTINUITY_REFUSAL_RECEIPT_TYPE,
            CONTINUITY_REFUSAL_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Continuity refusal receipts preserve why a continuity packet could not be trusted.",
            ],
        ),
        mutation_contract(
            RUNTIME_EVENT_TYPE,
            RUNTIME_SPINE_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["Runtime events are append-only projections for inspection."],
        ),
        mutation_contract(
            COORDINATOR_RUN_RECEIPT_TYPE,
            COORDINATOR_RUN_RECEIPT_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::ReceiptWatch],
            CultNetMutationAuthority::Coordinator,
            vec![],
            vec![],
            vec![
                "Coordinator run receipts are typed summaries of local plan/run decisions; artifact JSON is display evidence, not the only durable account.",
            ],
        ),
        mutation_contract(
            RUNTIME_MODEL_EXECUTION_BINDING_TYPE,
            RUNTIME_MODEL_EXECUTION_BINDING_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Runtime spine atomically binds one native/provider model request pair to its owning session and job before transport begins.",
                "Retention must reject unbound model rows rather than infer ownership from request names or conversation ids.",
            ],
        ),
        mutation_contract(
            RUNTIME_TOOL_EXECUTION_BINDING_TYPE,
            RUNTIME_TOOL_EXECUTION_BINDING_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Runtime spine atomically binds a tool intent to its session and job before the tool runtime may execute it.",
                "Model-derived tool intents must inherit the exact owning model execution; direct intents require explicit runtime ownership.",
            ],
        ),
        mutation_contract(
            ARCHIVED_RUNTIME_SESSION_TYPE,
            ARCHIVED_RUNTIME_SESSION_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Runtime spine alone archives an exact completed model-session generation under a full snapshot fence.",
                "The tombstone preserves retired identities and digest evidence, blocks ID reuse, and cannot satisfy execution authority.",
            ],
        ),
        mutation_contract(
            MODEL_ADAPTER_STATUS_TYPE,
            MODEL_ADAPTER_STATUS_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Model adapter status is provider-neutral; OpenAI/Codex is one current provider behind this boundary.",
            ],
        ),
        mutation_contract(
            MODEL_REQUEST_TYPE,
            MODEL_REQUEST_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::IntentSubmit,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::Coordinator,
            vec![MODEL_REQUEST_TYPE],
            vec![MODEL_STREAM_EVENT_TYPE, MODEL_RECEIPT_TYPE],
            vec![
                "Model turns enter through typed provider-neutral Epiphany request documents and return typed stream events/receipts.",
                "Provider adapters may authenticate and transport; they must not own Epiphany state, prompt authority, or scheduling.",
            ],
        ),
        mutation_contract(
            MODEL_STREAM_EVENT_TYPE,
            MODEL_STREAM_EVENT_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::ReceiptWatch],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec!["Model stream events are receipts from a typed model request."],
        ),
        mutation_contract(
            MODEL_RECEIPT_TYPE,
            MODEL_RECEIPT_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::ReceiptWatch],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Terminal model receipts carry provider response id, usage, and transport evidence.",
            ],
        ),
        mutation_contract(
            TOOL_CAPABILITY_TYPE,
            TOOL_CAPABILITY_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Tool capability documents describe adapter-discovered tools without making raw MCP discovery JSON authoritative.",
            ],
        ),
        mutation_contract(
            TOOL_INVOCATION_INTENT_TYPE,
            TOOL_INVOCATION_INTENT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::IntentSubmit,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::Coordinator,
            vec![TOOL_INVOCATION_INTENT_TYPE],
            vec![TOOL_INVOCATION_RECEIPT_TYPE],
            vec![
                "Tool calls enter Epiphany as typed invocation intents; MCP JSON remains protocol-edge cargo.",
            ],
        ),
        mutation_contract(
            TOOL_INVOCATION_RECEIPT_TYPE,
            TOOL_INVOCATION_RECEIPT_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::ReceiptWatch],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Tool invocation receipts seal parsed results, errors, and raw-result artifact refs before scheduler or state admission.",
            ],
        ),
        mutation_contract(
            OPENAI_ADAPTER_STATUS_TYPE,
            OPENAI_ADAPTER_STATUS_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "OpenAI adapter status is provider-specific evidence behind the model adapter boundary.",
            ],
        ),
        mutation_contract(
            OPENAI_MODEL_REQUEST_TYPE,
            OPENAI_MODEL_REQUEST_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![OPENAI_MODEL_STREAM_EVENT_TYPE, OPENAI_MODEL_RECEIPT_TYPE],
            vec![
                "OpenAI model requests are adapter projection evidence, not the provider-neutral request authority.",
            ],
        ),
        mutation_contract(
            OPENAI_MODEL_STREAM_EVENT_TYPE,
            OPENAI_MODEL_STREAM_EVENT_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::ReceiptWatch],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "OpenAI stream events are provider-specific receipts mirrored from model stream events.",
            ],
        ),
        mutation_contract(
            OPENAI_MODEL_RECEIPT_TYPE,
            OPENAI_MODEL_RECEIPT_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::ReceiptWatch],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "OpenAI terminal receipts are provider-specific evidence behind the model receipt.",
            ],
        ),
        mutation_contract(
            AGENT_MEMORY_TYPE,
            AGENT_MEMORY_PAYLOAD_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::IntentSubmit,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::Coordinator,
            vec!["epiphany.agent_memory_intent.v0"],
            vec!["epiphany.swarm_control_receipt.v0"],
            vec![
                "Sub-agents request memory mutations; the coordinator carries the typed intent, and Mind accepts, rejects, or explains durable-state admission.",
            ],
        ),
        mutation_contract(
            HEARTBEAT_STATE_TYPE,
            HEARTBEAT_STATE_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::IntentSubmit,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::Coordinator,
            vec![
                "epiphany.heartbeat_pump_intent.v0",
                "epiphany.heartbeat_heat_intent.v0",
                "epiphany.circadian_rhythm_intent.v0",
            ],
            vec!["epiphany.swarm_control_receipt.v0"],
            vec![
                "Aquarium controls heartbeat and circadian rhythm through typed intents, not blind state replacement.",
                "Initiative heat is heartbeat policy: global, group, role, and agent tempo changes enter through the heartbeat heat intent.",
            ],
        ),
        mutation_contract(
            STATE_LEDGER_STORE_TYPE,
            STATE_LEDGER_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "The ledger is inspected as durable memory; writes are mediated by role-specific state flows.",
            ],
        ),
        mutation_contract(
            THREAD_STATE_TYPE,
            THREAD_STATE_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "The mirrored thread state is the typed repo/control-plane state source; Codex rollout is a compatibility source, not the network contract.",
            ],
        ),
        mutation_contract(
            EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_TYPE,
            EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::DocumentPut,
            ],
            CultNetMutationAuthority::LocalUser,
            vec![],
            vec![],
            vec![
                "Operator snapshots are bounded typed receipts derived from operator-safe status/run artifacts.",
                "Raw Codex app-server JSON remains an edge artifact; this CultMesh document is the native Epiphany status receipt.",
            ],
        ),
        mutation_contract(
            EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_TYPE,
            EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::DocumentPut,
            ],
            CultNetMutationAuthority::LocalUser,
            vec![],
            vec![EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_TYPE],
            vec![
                "Operator run intents record explicit local wrapper requests before status/plan/smoke/run actions execute.",
                "This is not a scheduler queue; it is the typed consent/trace surface for local operator action.",
            ],
        ),
        mutation_contract(
            EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_TYPE,
            EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::DocumentPut,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::LocalUser,
            vec![],
            vec![],
            vec![
                "Operator run receipts record completed local wrapper actions and evidence artifact references.",
                "Referenced artifacts remain evidence; the receipt is the native completion contract.",
            ],
        ),
    ]
}

fn validate_non_empty(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{field} must be non-empty"));
    }
    Ok(())
}

fn timestamp_after(value: &str, lower_bound: &str) -> bool {
    !value.trim().is_empty() && value > lower_bound
}

fn worker_launch_document_kind(document: &EpiphanyWorkerLaunchDocument) -> &'static str {
    document.document_kind()
}

fn encode_worker_launch_document(document: &EpiphanyWorkerLaunchDocument) -> Result<Vec<u8>> {
    rmp_serde::to_vec_named(document).context("failed to encode worker launch document MessagePack")
}

fn validate_launch_organ_contract(
    contract: &EpiphanyLaunchOrganContract,
    authority_scope: &str,
    document_kind: &str,
    output_contract_id: &str,
) -> Result<()> {
    validate_non_empty(
        &contract.schema_version,
        "epiphany launch organ contract schema_version",
    )?;
    if contract.authority_scope != authority_scope {
        return Err(anyhow!(
            "epiphany launch organ contract authority_scope must match the launch request"
        ));
    }
    if contract.document_kind != document_kind {
        return Err(anyhow!(
            "epiphany launch organ contract document_kind must match the typed launch document"
        ));
    }
    if contract.output_contract_id != output_contract_id {
        return Err(anyhow!(
            "epiphany launch organ contract output_contract_id must match the launch request"
        ));
    }
    validate_non_empty(
        &contract.owner_organ,
        "epiphany launch organ contract owner_organ",
    )?;
    if contract.dependencies.is_empty() {
        return Err(anyhow!(
            "epiphany launch organ contract must carry organ dependencies"
        ));
    }
    if contract.required_receipt_document_types.is_empty() {
        return Err(anyhow!(
            "epiphany launch organ contract must carry required receipt document types"
        ));
    }
    if contract.receipt_proof_profiles.is_empty() {
        return Err(anyhow!(
            "epiphany launch organ contract must carry effect-specific receipt proof profiles"
        ));
    }
    Ok(())
}

fn decode_optional_msgpack<T>(payload: Option<&[u8]>, label: &str) -> Result<Option<T>>
where
    T: for<'de> Deserialize<'de>,
{
    payload
        .map(|payload| {
            rmp_serde::from_slice(payload).with_context(|| format!("failed to decode {label}"))
        })
        .transpose()
}

fn validate_heartbeat_launch_options(
    state: &EpiphanyThreadState,
    options: &RuntimeSpineHeartbeatLaunchPlanOptions,
) -> Result<()> {
    validate_non_empty(&options.binding_id, "epiphany job launch binding_id")?;
    if matches!(
        options.binding_id.as_str(),
        "retrieval-index" | "graph-remap" | "verification"
    ) {
        return Err(anyhow!(
            "epiphany job launch binding_id {:?} is reserved for a derived built-in slot",
            options.binding_id
        ));
    }
    if options.kind != EpiphanyJobKind::Specialist {
        return Err(anyhow!(
            "epiphany job launch currently supports only specialist heartbeat turns"
        ));
    }
    validate_non_empty(&options.scope, "epiphany job launch scope")?;
    validate_non_empty(&options.owner_role, "epiphany job launch owner_role")?;
    validate_non_empty(
        &options.authority_scope,
        "epiphany job launch authority_scope",
    )?;
    validate_non_empty(&options.instruction, "epiphany job launch instruction")?;
    validate_non_empty(
        options.launch_document.thread_id(),
        "epiphany job launch document thread id",
    )?;
    validate_non_empty(
        &options.output_contract_id,
        "epiphany job launch output_contract_id",
    )?;
    if options.output_contract_id != options.launch_document.output_contract_id() {
        return Err(anyhow!(
            "epiphany job launch output_contract_id must match the typed launch document"
        ));
    }
    validate_launch_organ_contract(
        &options.organ_launch_contract,
        &options.authority_scope,
        options.launch_document.document_kind(),
        &options.output_contract_id,
    )?;
    if let Some(max_runtime_seconds) = options.max_runtime_seconds
        && max_runtime_seconds == 0
    {
        return Err(anyhow!(
            "epiphany job launch max_runtime_seconds must be >= 1"
        ));
    }
    let existing_binding = state
        .job_bindings
        .iter()
        .find(|binding| binding.id == options.binding_id);
    let latest_runtime_link = state.runtime_links.iter().find(|link| {
        link.binding_id == options.binding_id && !link.runtime_job_id.trim().is_empty()
    });
    if latest_runtime_link.is_some_and(|link| link.runtime_result_id.is_none())
        && existing_binding.is_none_or(|binding| binding.blocking_reason.is_none())
    {
        return Err(anyhow!(
            "epiphany job binding {:?} is already bound to an active heartbeat turn; interrupt it before launching a replacement",
            options.binding_id
        ));
    }
    Ok(())
}

fn terminal_status_for_verdict(verdict: &str) -> EpiphanyRuntimeJobStatus {
    if matches!(
        verdict,
        "failed" | "fail" | "error" | "blocked" | "cancelled" | "canceled"
    ) {
        EpiphanyRuntimeJobStatus::Failed
    } else {
        EpiphanyRuntimeJobStatus::Completed
    }
}

fn merge_refs(existing: &[String], incoming: &[String]) -> Vec<String> {
    let mut merged = existing.to_vec();
    for item in incoming {
        if !merged.contains(item) {
            merged.push(item.clone());
        }
    }
    merged
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn bind_test_runtime_swarm(store: &Path, swarm_id: &str) -> Result<()> {
        let agent_store = store.with_extension("test-agent-memory.cc");
        crate::ensure_agent_memory_swarm_identity(&agent_store, swarm_id)?;
        bind_runtime_to_agent_memory_swarm(store, &agent_store, "2026-08-14T00:00:01Z")?;
        Ok(())
    }

    pub(crate) fn bind_test_repository_body(store: &Path, workspace_id: &str) -> Result<()> {
        if crate::runtime_repository_body_store_binding(store)?.is_some() {
            return Ok(());
        }
        let repo = store.with_extension(format!("{workspace_id}.body-repo"));
        std::fs::create_dir_all(&repo)?;
        let init = std::process::Command::new("git")
            .args(["init", "--quiet"])
            .current_dir(&repo)
            .output()?;
        if !init.status.success() {
            return Err(anyhow!("test repository git init failed"));
        }
        std::fs::write(repo.join("body-seed.txt"), workspace_id.as_bytes())?;
        let add = std::process::Command::new("git")
            .args(["add", "body-seed.txt"])
            .current_dir(&repo)
            .output()?;
        if !add.status.success() {
            return Err(anyhow!("test repository git add failed"));
        }
        let body_store = store.with_extension(format!("{workspace_id}.body.cc"));
        crate::bind_repository_body(&repo, &body_store, store, workspace_id)?;
        Ok(())
    }

    #[test]
    fn runtime_registry_has_no_aggregate_repo_model_authority() -> Result<()> {
        let registered = runtime_registered_document_types();
        assert!(
            !registered
                .iter()
                .any(|kind| kind == "epiphany.memory_graph")
        );
        assert!(
            !registered
                .iter()
                .any(|kind| kind.contains("repo_model_admission"))
        );
        assert!(
            registered
                .iter()
                .any(|kind| kind == crate::EpiphanyRepoModelIdentityDocument::TYPE)
        );
        Ok(())
    }
}
