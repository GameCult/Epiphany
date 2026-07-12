use crate::EpiphanyWorkerLaunchDocument;
use crate::agent_memory::AGENT_MEMORY_TYPE;
use crate::continuity_gateway::ContinuityRecoveryReceipt;
use crate::continuity_gateway::*;
use crate::cultmesh_integration::EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION;
use crate::cultmesh_integration::EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_TYPE;
use crate::cultmesh_integration::EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION;
use crate::cultmesh_integration::EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_TYPE;
use crate::cultmesh_integration::EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_SCHEMA_VERSION;
use crate::cultmesh_integration::EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_TYPE;
use crate::cultmesh_integration::EPIPHANY_CULTMESH_OPERATOR_STATUS_SCHEMA_VERSION;
use crate::cultmesh_integration::EPIPHANY_CULTMESH_OPERATOR_STATUS_TYPE;
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
use crate::hands_gateway::*;
use crate::heartbeat_state::HEARTBEAT_STATE_SCHEMA_VERSION;
use crate::heartbeat_state::HEARTBEAT_STATE_TYPE;
use crate::memory_graph::MEMORY_GRAPH_SCHEMA_VERSION;
use crate::memory_graph::MEMORY_GRAPH_TYPE;
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
use crate::mind_gateway::MindGatewayDecision;
use crate::mind_gateway::MindGatewayReview;
use crate::mind_gateway::MindStateCommitReceipt;
use crate::modeling_gateway::REPO_WORK_MAP_ENTRY_SCHEMA_VERSION;
use crate::modeling_gateway::REPO_WORK_MAP_ENTRY_TYPE;
use crate::modeling_gateway::REPO_WORK_MODELING_FINDING_SCHEMA_VERSION;
use crate::modeling_gateway::REPO_WORK_MODELING_FINDING_TYPE;
use crate::modeling_gateway::REPO_WORK_MODELING_REQUEST_SCHEMA_VERSION;
use crate::modeling_gateway::REPO_WORK_MODELING_REQUEST_TYPE;
use crate::modeling_gateway::REPO_WORK_MODELING_ROUTE_SCHEMA_VERSION;
use crate::modeling_gateway::REPO_WORK_MODELING_ROUTE_TYPE;
use crate::modeling_gateway::RepoWorkMapEntry;
use crate::modeling_gateway::RepoWorkModelingFinding;
use crate::modeling_gateway::RepoWorkModelingRequest;
use crate::modeling_gateway::RepoWorkModelingRoute;
use crate::organ_dependencies::EpiphanyLaunchOrganContract;
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
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use cultcache_rs::CultCache;
use cultcache_rs::CultCacheEnvelope;
use cultcache_rs::DatabaseEntry;
use cultcache_rs::SingleFileMessagePackBackingStore;
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
use epiphany_openai_adapter::EpiphanyOpenAiAdapterStatus;
use epiphany_openai_adapter::EpiphanyOpenAiModelReceipt;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
use epiphany_openai_adapter::EpiphanyOpenAiStreamEvent;
use epiphany_state_model::EpiphanyJobBinding;
use epiphany_state_model::EpiphanyJobKind;
use epiphany_state_model::EpiphanyRuntimeLink;
use epiphany_state_model::EpiphanyThreadState;
use epiphany_tool_adapter::EpiphanyToolCapability;
use epiphany_tool_adapter::EpiphanyToolInvocationIntent;
use epiphany_tool_adapter::EpiphanyToolInvocationReceipt;
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub const RUNTIME_IDENTITY_TYPE: &str = "epiphany.runtime.identity";
pub const RUNTIME_SESSION_TYPE: &str = "epiphany.runtime.session";
pub const RUNTIME_JOB_TYPE: &str = "epiphany.runtime.job";
pub const RUNTIME_WORKER_LAUNCH_REQUEST_TYPE: &str = "epiphany.runtime.worker_launch_request";
pub const RUNTIME_ROLE_WORKER_RESULT_TYPE: &str = "epiphany.runtime.role_worker_result";
pub const RUNTIME_REORIENT_WORKER_RESULT_TYPE: &str = "epiphany.runtime.reorient_worker_result";
pub const RUNTIME_JOB_RESULT_TYPE: &str = "epiphany.runtime.job_result";
pub const RUNTIME_EVENT_TYPE: &str = "epiphany.runtime.event";
pub const COORDINATOR_RUN_RECEIPT_TYPE: &str = "epiphany.coordinator_run_receipt.v0";
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
pub const SURFACE_SCENE_TYPE: &str = "epiphany.surface.scene";
pub const SURFACE_FRESHNESS_TYPE: &str = "epiphany.surface.freshness";
pub const SURFACE_CONTEXT_TYPE: &str = "epiphany.surface.context";
pub const SURFACE_GRAPH_QUERY_TYPE: &str = "epiphany.surface.graph_query";
pub const SURFACE_PRESSURE_TYPE: &str = "epiphany.surface.pressure";
pub const SURFACE_REORIENT_TYPE: &str = "epiphany.surface.reorient";
pub const SURFACE_CRRC_TYPE: &str = "epiphany.surface.crrc";
pub const SURFACE_JOBS_TYPE: &str = "epiphany.surface.jobs";
pub const SURFACE_ROLES_TYPE: &str = "epiphany.surface.roles";
pub const SURFACE_ROLE_RESULT_TYPE: &str = "epiphany.surface.role_result";
pub const SURFACE_REORIENT_RESULT_TYPE: &str = "epiphany.surface.reorient_result";
pub const SURFACE_PLANNING_TYPE: &str = "epiphany.surface.planning";
pub const SURFACE_COORDINATOR_TYPE: &str = "epiphany.surface.coordinator";
pub const SURFACE_PERSONA_TYPE: &str = "epiphany.surface.persona";
pub const SURFACE_VOID_MEMORY_TYPE: &str = "epiphany.surface.void_memory";
pub const SURFACE_REPO_INITIALIZATION_TYPE: &str = "epiphany.surface.repo_initialization";
pub const SURFACE_REPO_BIRTH_RUNNER_TYPE: &str = "epiphany.surface.repo_birth_runner";
pub const RUNTIME_IDENTITY_KEY: &str = "self";
pub const RUNTIME_SPINE_SCHEMA_VERSION: &str = "epiphany.runtime_spine.v0";
pub const RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION: &str =
    "epiphany.runtime.worker_launch_request.v0";
pub const RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION: &str =
    "epiphany.runtime.role_worker_result.v0";
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
pub const SCENE_SURFACE_SCHEMA_VERSION: &str = "epiphany.scene_surface.v0";
pub const FRESHNESS_SURFACE_SCHEMA_VERSION: &str = "epiphany.freshness_surface.v0";
pub const CONTEXT_SURFACE_SCHEMA_VERSION: &str = "epiphany.context_surface.v0";
pub const GRAPH_QUERY_SURFACE_SCHEMA_VERSION: &str = "epiphany.graph_query_surface.v0";
pub const PRESSURE_SURFACE_SCHEMA_VERSION: &str = "epiphany.pressure_surface.v0";
pub const REORIENT_SURFACE_SCHEMA_VERSION: &str = "epiphany.reorient_surface.v0";
pub const CRRC_SURFACE_SCHEMA_VERSION: &str = "epiphany.crrc_surface.v0";
pub const JOBS_SURFACE_SCHEMA_VERSION: &str = "epiphany.jobs_surface.v0";
pub const ROLES_SURFACE_SCHEMA_VERSION: &str = "epiphany.roles_surface.v0";
pub const ROLE_RESULT_SURFACE_SCHEMA_VERSION: &str = "epiphany.role_result_surface.v0";
pub const REORIENT_RESULT_SURFACE_SCHEMA_VERSION: &str = "epiphany.reorient_result_surface.v0";
pub const PLANNING_SURFACE_SCHEMA_VERSION: &str = "epiphany.planning_surface.v0";
pub const COORDINATOR_SURFACE_SCHEMA_VERSION: &str = "epiphany.coordinator_surface.v0";
pub const PERSONA_SURFACE_SCHEMA_VERSION: &str = "epiphany.persona_surface.v0";
pub const VOID_MEMORY_SURFACE_SCHEMA_VERSION: &str = "epiphany.void_memory_surface.v0";
pub const REPO_INITIALIZATION_SURFACE_SCHEMA_VERSION: &str =
    "epiphany.repo_initialization_surface.v0";
pub const REPO_BIRTH_RUNNER_SURFACE_SCHEMA_VERSION: &str = "epiphany.repo_birth_runner_surface.v0";
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
}

impl EpiphanyRuntimeWorkerLaunchRequest {
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
    cache.register_entry_type::<crate::EpiphanyThreadStateEntry>()?;
    cache.register_entry_type::<EpiphanyRuntimeIdentity>()?;
    cache.register_entry_type::<EpiphanyRuntimeSession>()?;
    cache.register_entry_type::<EpiphanyRuntimeJob>()?;
    cache.register_entry_type::<EpiphanyRuntimeWorkerLaunchRequest>()?;
    cache.register_entry_type::<EpiphanyRuntimeRoleWorkerResult>()?;
    cache.register_entry_type::<EpiphanyRuntimeReorientWorkerResult>()?;
    cache.register_entry_type::<EpiphanyRuntimeJobResult>()?;
    cache.register_entry_type::<EpiphanyRuntimeEvent>()?;
    cache.register_entry_type::<EpiphanyCoordinatorRunReceipt>()?;
    cache.register_entry_type::<MindGatewayReview>()?;
    cache.register_entry_type::<MindStateCommitReceipt>()?;
    cache.register_entry_type::<RepoWorkModelingFinding>()?;
    cache.register_entry_type::<RepoWorkModelingRequest>()?;
    cache.register_entry_type::<RepoWorkModelingRoute>()?;
    cache.register_entry_type::<RepoWorkMapEntry>()?;
    cache.register_entry_type::<EyesEvidencePacket>()?;
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
    cache.register_entry_type::<EpiphanyToolCapability>()?;
    cache.register_entry_type::<EpiphanyToolInvocationIntent>()?;
    cache.register_entry_type::<EpiphanyToolInvocationReceipt>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(
        store_path.to_path_buf(),
    ));
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
    cache.put(RUNTIME_IDENTITY_KEY, &identity)?;
    Ok(identity)
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

pub fn put_runtime_role_worker_result(
    store_path: impl AsRef<Path>,
    result: &EpiphanyRuntimeRoleWorkerResult,
) -> Result<()> {
    validate_non_empty(&result.job_id, "role worker result job id")?;
    validate_non_empty(&result.result_id, "role worker result id")?;
    validate_non_empty(&result.role_id, "role worker result role id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.put(&result.job_id, result)?;
    Ok(())
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

pub fn put_runtime_reorient_worker_result(
    store_path: impl AsRef<Path>,
    result: &EpiphanyRuntimeReorientWorkerResult,
) -> Result<()> {
    validate_non_empty(&result.job_id, "reorient worker result job id")?;
    validate_non_empty(&result.result_id, "reorient worker result id")?;
    validate_non_empty(&result.mode, "reorient worker result mode")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.put(&result.job_id, result)?;
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

pub fn put_mind_gateway_review(
    store_path: impl AsRef<Path>,
    review: &MindGatewayReview,
) -> Result<()> {
    validate_non_empty(&review.gateway_id, "Mind gateway review id")?;
    validate_non_empty(&review.source_kind, "Mind gateway review source kind")?;
    validate_non_empty(&review.source_role_id, "Mind gateway review source role")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    cache.put(&review.gateway_id, review)?;
    Ok(())
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

pub fn put_mind_state_commit_receipt(
    store_path: impl AsRef<Path>,
    receipt: &MindStateCommitReceipt,
) -> Result<()> {
    validate_non_empty(&receipt.receipt_id, "Mind state commit receipt id")?;
    validate_non_empty(&receipt.gateway_id, "Mind state commit gateway id")?;
    validate_non_empty(&receipt.source_kind, "Mind state commit source kind")?;
    validate_non_empty(&receipt.source_role_id, "Mind state commit source role")?;
    validate_non_empty(&receipt.committed_at, "Mind state commit timestamp")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    cache.put(&receipt.receipt_id, receipt)?;
    Ok(())
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

pub fn put_eyes_evidence_packet(
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
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    cache.put(&receipt.receipt_id, receipt)?;
    Ok(())
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
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    cache.put(&intent.intent_id, intent)?;
    Ok(())
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
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    cache.put(&review.review_id, review)?;
    Ok(())
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

pub fn put_hands_patch_receipt(
    store_path: impl AsRef<Path>,
    receipt: &HandsPatchReceipt,
) -> Result<()> {
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
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    cache.put(&receipt.receipt_id, receipt)?;
    Ok(())
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
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    cache.put(&receipt.receipt_id, receipt)?;
    Ok(())
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
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    cache.put(&receipt.receipt_id, receipt)?;
    Ok(())
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

pub fn put_soul_verdict_receipt(
    store_path: impl AsRef<Path>,
    receipt: &SoulVerdictReceipt,
) -> Result<()> {
    validate_non_empty(&receipt.receipt_id, "Soul verdict receipt id")?;
    validate_non_empty(&receipt.source_result_id, "Soul verdict source result")?;
    validate_non_empty(&receipt.source_job_id, "Soul verdict source job")?;
    validate_non_empty(&receipt.verdict, "Soul verdict")?;
    validate_non_empty(&receipt.emitted_at, "Soul verdict timestamp")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    if let Some(existing) = cache.get::<SoulVerdictReceipt>(&receipt.receipt_id)? {
        if existing == *receipt {
            return Ok(());
        }
        return Err(anyhow!(
            "Soul verdict receipt id already belongs to different immutable evidence"
        ));
    }
    cache.put(&receipt.receipt_id, receipt)?;
    Ok(())
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

pub fn put_repo_work_modeling_finding(
    store_path: impl AsRef<Path>,
    finding: &RepoWorkModelingFinding,
) -> Result<()> {
    validate_non_empty(&finding.receipt_id, "Modeling finding receipt id")?;
    validate_non_empty(&finding.item, "Modeling finding item")?;
    validate_non_empty(&finding.model_ref, "Modeling finding model ref")?;
    validate_non_empty(
        &finding.soul_verdict_receipt_id,
        "Modeling finding Soul verdict receipt id",
    )?;
    validate_non_empty(&finding.verdict, "Modeling finding verdict")?;
    validate_non_empty(&finding.finding, "Modeling finding text")?;
    validate_non_empty(&finding.commit_sha, "Modeling finding commit sha")?;
    validate_non_empty(&finding.emitted_at, "Modeling finding timestamp")?;
    if finding.schema_version != REPO_WORK_MODELING_FINDING_SCHEMA_VERSION {
        return Err(anyhow!("unsupported Modeling finding schema"));
    }
    if finding.private_state_exposed {
        return Err(anyhow!("Modeling finding cannot expose private state"));
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let request = cache
        .get::<RepoWorkModelingRequest>(&finding.request_id)?
        .ok_or_else(|| anyhow!("Modeling finding requires its typed request"))?;
    let soul = cache
        .get::<SoulVerdictReceipt>(&finding.soul_verdict_receipt_id)?
        .ok_or_else(|| anyhow!("Modeling finding requires its Soul verdict receipt"))?;
    if soul.verdict.trim().to_ascii_lowercase() != "passed" {
        return Err(anyhow!(
            "Modeling finding requires a passing Soul verdict receipt"
        ));
    }
    if request.item != finding.item
        || request.soul_verdict_receipt_id != finding.soul_verdict_receipt_id
        || request.commit_sha != finding.commit_sha
        || request.changed_paths != finding.changed_paths
    {
        return Err(anyhow!(
            "Modeling finding does not answer its typed request"
        ));
    }
    if let Some(existing) = cache.get::<RepoWorkModelingFinding>(&finding.receipt_id)? {
        if existing == *finding {
            return Ok(());
        }
        return Err(anyhow!(
            "Modeling finding receipt id already belongs to different immutable evidence"
        ));
    }
    cache.put(&finding.receipt_id, finding)?;
    Ok(())
}

pub fn put_repo_work_modeling_request(
    store_path: impl AsRef<Path>,
    request: &RepoWorkModelingRequest,
) -> Result<()> {
    if request.schema_version != REPO_WORK_MODELING_REQUEST_SCHEMA_VERSION {
        return Err(anyhow!("unsupported Modeling request schema"));
    }
    validate_non_empty(&request.request_id, "Modeling request id")?;
    validate_non_empty(&request.item, "Modeling request item")?;
    validate_non_empty(&request.requester, "Modeling request requester")?;
    validate_non_empty(
        &request.soul_verdict_receipt_id,
        "Modeling request Soul verdict receipt id",
    )?;
    validate_non_empty(&request.commit_sha, "Modeling request commit sha")?;
    validate_non_empty(&request.instruction, "Modeling request instruction")?;
    validate_non_empty(&request.requested_at, "Modeling request timestamp")?;
    if request.requester != "self" || request.private_state_exposed {
        return Err(anyhow!(
            "repo-work Modeling requests must be Self-routed and private-state sealed"
        ));
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let soul = cache
        .get::<SoulVerdictReceipt>(&request.soul_verdict_receipt_id)?
        .ok_or_else(|| anyhow!("Modeling request requires its Soul verdict receipt"))?;
    if soul.verdict.trim().to_ascii_lowercase() != "passed" {
        return Err(anyhow!("Modeling request requires a passing Soul verdict"));
    }
    if let Some(existing) = cache.get::<RepoWorkModelingRequest>(&request.request_id)? {
        if existing == *request {
            return Ok(());
        }
        return Err(anyhow!(
            "Modeling request id already belongs to different immutable work"
        ));
    }
    cache.put(&request.request_id, request)?;
    Ok(())
}

pub fn runtime_repo_work_modeling_request(
    store_path: impl AsRef<Path>,
    request_id: &str,
) -> Result<Option<RepoWorkModelingRequest>> {
    validate_non_empty(request_id, "Modeling request id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<RepoWorkModelingRequest>(request_id)
}

pub fn commit_initial_repo_work_modeling_route(
    store_path: impl AsRef<Path>,
    request: &RepoWorkModelingRequest,
    route: &RepoWorkModelingRoute,
) -> Result<RepoWorkModelingRoute> {
    if request.schema_version != REPO_WORK_MODELING_REQUEST_SCHEMA_VERSION {
        return Err(anyhow!("unsupported Modeling request schema"));
    }
    validate_non_empty(&request.request_id, "Modeling request id")?;
    validate_non_empty(&request.item, "Modeling request item")?;
    validate_non_empty(
        &request.soul_verdict_receipt_id,
        "Modeling request Soul verdict receipt id",
    )?;
    validate_non_empty(&request.commit_sha, "Modeling request commit sha")?;
    validate_non_empty(&request.instruction, "Modeling request instruction")?;
    validate_non_empty(&request.requested_at, "Modeling request timestamp")?;
    if request.requester != "self" || request.private_state_exposed {
        return Err(anyhow!(
            "initial Modeling request must be Self-routed and private-state sealed"
        ));
    }
    if route.schema_version != REPO_WORK_MODELING_ROUTE_SCHEMA_VERSION {
        return Err(anyhow!("unsupported Modeling route schema"));
    }
    validate_non_empty(&route.route_id, "Modeling route id")?;
    validate_non_empty(&route.item, "Modeling route item")?;
    validate_non_empty(&route.request_id, "Modeling route request id")?;
    validate_non_empty(&route.authority_owner, "Modeling route authority owner")?;
    validate_non_empty(
        &route.authority_witness_id,
        "Modeling route authority witness",
    )?;
    validate_non_empty(&route.updated_at, "Modeling route timestamp")?;
    if route.generation != 0
        || !route.previous_finding_receipt_id.is_empty()
        || route.authority_owner != "soul"
        || route.authority_witness_id != request.soul_verdict_receipt_id
        || route.item != request.item
        || route.request_id != request.request_id
        || route.private_state_exposed
    {
        return Err(anyhow!(
            "initial Modeling route must be generation zero, Soul-backed, request-matched, and private-state sealed"
        ));
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let soul = cache
        .get::<SoulVerdictReceipt>(&request.soul_verdict_receipt_id)?
        .ok_or_else(|| anyhow!("initial Modeling route requires its Soul verdict"))?;
    if soul.verdict.trim().to_ascii_lowercase() != "passed" {
        return Err(anyhow!(
            "initial Modeling route requires passing Soul truth"
        ));
    }
    if let Some(existing) = cache.get::<RepoWorkModelingRoute>(&route.route_id)? {
        let stored_request = cache
            .get::<RepoWorkModelingRequest>(&existing.request_id)?
            .ok_or_else(|| anyhow!("existing Modeling route lost its request"))?;
        if existing == *route && stored_request == *request {
            return Ok(existing);
        }
        return Err(anyhow!(
            "initial Modeling route already belongs to different routing truth"
        ));
    }
    if cache
        .get::<RepoWorkModelingRequest>(&request.request_id)?
        .is_some()
    {
        return Err(anyhow!(
            "Modeling request exists without its atomic initial route"
        ));
    }
    let (request_envelope, _) = cache.prepare_entry(&request.request_id, request)?;
    let (route_envelope, _) = cache.prepare_entry(&route.route_id, route)?;
    cache.put_prepared_batch(vec![request_envelope, route_envelope])?;
    Ok(route.clone())
}

pub fn runtime_repo_work_modeling_route(
    store_path: impl AsRef<Path>,
    route_id: &str,
) -> Result<Option<RepoWorkModelingRoute>> {
    validate_non_empty(route_id, "Modeling route id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<RepoWorkModelingRoute>(route_id)
}

pub fn advance_repo_work_modeling_route(
    store_path: impl AsRef<Path>,
    request: &RepoWorkModelingRequest,
    route: &RepoWorkModelingRoute,
    mind_review: &MindGatewayReview,
) -> Result<RepoWorkModelingRoute> {
    if request.schema_version != REPO_WORK_MODELING_REQUEST_SCHEMA_VERSION
        || route.schema_version != REPO_WORK_MODELING_ROUTE_SCHEMA_VERSION
    {
        return Err(anyhow!("unsupported Modeling route generation schema"));
    }
    validate_non_empty(&request.request_id, "next Modeling request id")?;
    validate_non_empty(&request.item, "next Modeling request item")?;
    validate_non_empty(
        &request.soul_verdict_receipt_id,
        "next Modeling request Soul verdict",
    )?;
    validate_non_empty(&request.commit_sha, "next Modeling request commit")?;
    validate_non_empty(&request.instruction, "next Modeling request instruction")?;
    validate_non_empty(&request.requested_at, "next Modeling request timestamp")?;
    validate_non_empty(&route.route_id, "Modeling route id")?;
    validate_non_empty(
        &route.previous_finding_receipt_id,
        "previous Modeling finding receipt id",
    )?;
    validate_non_empty(&route.updated_at, "next Modeling route timestamp")?;
    if request.requester != "mind"
        || request.private_state_exposed
        || route.private_state_exposed
        || route.authority_owner != "mind"
        || route.authority_witness_id != mind_review.gateway_id
        || route.request_id != request.request_id
        || route.item != request.item
        || mind_review.decision != MindGatewayDecision::Accept
        || mind_review.source_kind != "repo_work_modeling_revision"
        || !mind_review
            .allowed_effects
            .iter()
            .any(|effect| effect == "repoWork.modelingRoute")
    {
        return Err(anyhow!(
            "next Modeling generation requires a private-state-sealed Mind acceptance that owns only the route transition"
        ));
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let current = cache
        .get::<RepoWorkModelingRoute>(&route.route_id)?
        .ok_or_else(|| anyhow!("cannot advance a missing Modeling route"))?;
    let current_request = cache
        .get::<RepoWorkModelingRequest>(&current.request_id)?
        .ok_or_else(|| anyhow!("current Modeling route lost its request"))?;
    let previous_finding = cache
        .get::<RepoWorkModelingFinding>(&route.previous_finding_receipt_id)?
        .ok_or_else(|| anyhow!("next Modeling generation requires the previous finding"))?;
    if route.generation != current.generation.saturating_add(1)
        || previous_finding.request_id != current.request_id
        || previous_finding.item != current.item
        || previous_finding
            .verdict
            .trim()
            .eq_ignore_ascii_case("passed")
        || request.item != current_request.item
        || request.soul_verdict_receipt_id != current_request.soul_verdict_receipt_id
        || request.commit_sha != current_request.commit_sha
        || request.changed_paths != current_request.changed_paths
    {
        return Err(anyhow!(
            "next Modeling generation must follow one non-passing current finding and preserve the Soul-verified consequence"
        ));
    }
    if cache
        .get::<RepoWorkModelingRequest>(&request.request_id)?
        .is_some()
    {
        return Err(anyhow!("next Modeling request id already exists"));
    }
    let (request_envelope, _) = cache.prepare_entry(&request.request_id, request)?;
    let (review_envelope, _) = cache.prepare_entry(&mind_review.gateway_id, mind_review)?;
    let (route_envelope, _) = cache.prepare_entry(&route.route_id, route)?;
    cache.put_prepared_batch(vec![request_envelope, review_envelope, route_envelope])?;
    Ok(route.clone())
}

pub fn runtime_repo_work_modeling_finding(
    store_path: impl AsRef<Path>,
    receipt_id: &str,
) -> Result<Option<RepoWorkModelingFinding>> {
    validate_non_empty(receipt_id, "Modeling finding receipt id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<RepoWorkModelingFinding>(receipt_id)
}

pub fn commit_repo_work_map_admission(
    store_path: impl AsRef<Path>,
    entry: &RepoWorkMapEntry,
    mind_review: &MindGatewayReview,
    mind_commit: &MindStateCommitReceipt,
) -> Result<RepoWorkMapEntry> {
    if entry.schema_version != REPO_WORK_MAP_ENTRY_SCHEMA_VERSION {
        return Err(anyhow!("unsupported repo-work map entry schema"));
    }
    validate_non_empty(&entry.map_entry_id, "repo-work map entry id")?;
    validate_non_empty(&entry.item, "repo-work map item")?;
    validate_non_empty(
        &entry.modeling_finding_receipt_id,
        "repo-work map Modeling finding receipt id",
    )?;
    if !entry.durable_state_admitted || entry.private_state_exposed {
        return Err(anyhow!(
            "repo-work map admission requires admitted, private-state-sealed state"
        ));
    }
    if entry.mind_gateway_review_id != mind_review.gateway_id
        || entry.mind_state_commit_receipt_id != mind_commit.receipt_id
        || mind_commit.gateway_id != mind_review.gateway_id
    {
        return Err(anyhow!(
            "repo-work map entry and Mind witnesses do not share one identity"
        ));
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let modeling = cache
        .get::<RepoWorkModelingFinding>(&entry.modeling_finding_receipt_id)?
        .ok_or_else(|| anyhow!("repo-work map admission requires typed Modeling finding"))?;
    if modeling.item != entry.item
        || modeling.soul_verdict_receipt_id != entry.soul_verdict_receipt_id
        || modeling.commit_sha != entry.commit_sha
        || modeling.changed_paths != entry.changed_paths
        || modeling.summary != entry.modeling_summary
    {
        return Err(anyhow!(
            "repo-work map entry does not match its typed Modeling finding"
        ));
    }
    if let Some(existing) = cache.get::<RepoWorkMapEntry>(&entry.map_entry_id)? {
        let mut candidate = entry.clone();
        candidate.admitted_at = existing.admitted_at.clone();
        if existing != candidate {
            return Err(anyhow!(
                "repo-work map entry id already belongs to different admitted state"
            ));
        }
        let stored_review = cache
            .get::<MindGatewayReview>(&existing.mind_gateway_review_id)?
            .ok_or_else(|| anyhow!("admitted repo-work map is missing its Mind review"))?;
        let stored_commit = cache
            .get::<MindStateCommitReceipt>(&existing.mind_state_commit_receipt_id)?
            .ok_or_else(|| anyhow!("admitted repo-work map is missing its Mind commit"))?;
        if stored_review.gateway_id != existing.mind_gateway_review_id
            || stored_commit.gateway_id != stored_review.gateway_id
        {
            return Err(anyhow!(
                "admitted repo-work map has incoherent Mind witnesses"
            ));
        }
        return Ok(existing);
    }
    let (review_envelope, _) = cache.prepare_entry(&mind_review.gateway_id, mind_review)?;
    let (commit_envelope, _) = cache.prepare_entry(&mind_commit.receipt_id, mind_commit)?;
    let (map_envelope, _) = cache.prepare_entry(&entry.map_entry_id, entry)?;
    cache.put_prepared_batch(vec![review_envelope, commit_envelope, map_envelope])?;
    Ok(entry.clone())
}

pub fn runtime_repo_work_map_entry(
    store_path: impl AsRef<Path>,
    map_entry_id: &str,
) -> Result<Option<RepoWorkMapEntry>> {
    validate_non_empty(map_entry_id, "repo-work map entry id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<RepoWorkMapEntry>(map_entry_id)
}

pub fn put_continuity_recovery_receipt(
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
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    cache.put(&receipt.receipt_id, receipt)?;
    Ok(())
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
    };
    cache.put(&job.job_id, &job)?;
    cache.put(&result.result_id, &result)?;
    let event = EpiphanyRuntimeEvent {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        event_id: format!("event-job-completed-{}", options.job_id),
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
    cache.put(&event.event_id, &event)?;
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
            .map(|item| item.supported_document_types)
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
        supported_document_types: Some(identity.supported_document_types),
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

fn read_only_surface_contract(
    document_type: impl Into<String>,
    payload_schema_version: impl Into<String>,
    notes: Vec<&str>,
) -> CultNetDocumentMutationContract {
    mutation_contract(
        document_type,
        payload_schema_version,
        vec![CultNetDocumentOperation::Snapshot],
        CultNetMutationAuthority::ReadOnly,
        vec![],
        vec![],
        notes,
    )
}

fn coordinator_surface_contract(
    document_type: impl Into<String>,
    payload_schema_version: impl Into<String>,
    intent_document_types: Vec<&str>,
    receipt_document_types: Vec<&str>,
    notes: Vec<&str>,
) -> CultNetDocumentMutationContract {
    mutation_contract(
        document_type,
        payload_schema_version,
        vec![
            CultNetDocumentOperation::Snapshot,
            CultNetDocumentOperation::IntentSubmit,
            CultNetDocumentOperation::ReceiptWatch,
        ],
        CultNetMutationAuthority::Coordinator,
        intent_document_types,
        receipt_document_types,
        notes,
    )
}

fn epiphany_mutation_contracts() -> Vec<CultNetDocumentMutationContract> {
    vec![
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
            REPO_WORK_MODELING_REQUEST_TYPE,
            REPO_WORK_MODELING_REQUEST_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::IntentSubmit,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::Coordinator,
            vec![REPO_WORK_MODELING_REQUEST_TYPE],
            vec![REPO_WORK_MODELING_FINDING_TYPE],
            vec![
                "Self routes Soul-verified repo consequence to Modeling through this request.",
                "The request grants no authority to author the Modeling result.",
            ],
        ),
        mutation_contract(
            REPO_WORK_MODELING_FINDING_TYPE,
            REPO_WORK_MODELING_FINDING_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Modeling findings interpret a Soul-verified repo consequence before Mind admits a map update.",
                "Schedulers and raw CLI fields cannot substitute for this persisted receipt.",
            ],
        ),
        mutation_contract(
            REPO_WORK_MODELING_ROUTE_TYPE,
            REPO_WORK_MODELING_ROUTE_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::IntentSubmit,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::Coordinator,
            vec![REPO_WORK_MODELING_REQUEST_TYPE, MIND_GATEWAY_REVIEW_TYPE],
            vec![REPO_WORK_MODELING_ROUTE_TYPE],
            vec![
                "The route is the sole current-generation owner for repo-work Modeling requests.",
                "Generation zero is Soul-backed; later generations require a Mind review and preserve the previous finding.",
                "Filesystem closure artifacts and Self are projections/routers, not route writers.",
            ],
        ),
        mutation_contract(
            REPO_WORK_MAP_ENTRY_TYPE,
            REPO_WORK_MAP_ENTRY_SCHEMA_VERSION,
            vec![
                CultNetDocumentOperation::Snapshot,
                CultNetDocumentOperation::ReceiptWatch,
            ],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Repo-work map entries are Mind-admitted durable state committed atomically with their Mind witnesses.",
                "CultMesh rows are projections of this entry, not a second map owner.",
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
            MEMORY_GRAPH_TYPE,
            MEMORY_GRAPH_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "The unified memory graph is typed durable state; Qdrant embeddings are rebuildable cache, not canonical memory.",
            ],
        ),
        mutation_contract(
            EPIPHANY_CULTMESH_OPERATOR_STATUS_TYPE,
            EPIPHANY_CULTMESH_OPERATOR_STATUS_SCHEMA_VERSION,
            vec![CultNetDocumentOperation::Snapshot],
            CultNetMutationAuthority::ReadOnly,
            vec![],
            vec![],
            vec![
                "Native operator status is a CultMesh document; Codex app-server status is bridge/display compatibility.",
                "This surface names the Codex bridge role, Epiphany authority role, prompt-authority boundary, and quarantined optional bridges.",
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
        read_only_surface_contract(
            SURFACE_SCENE_TYPE,
            SCENE_SURFACE_SCHEMA_VERSION,
            vec![
                "Operator-safe scene reflection over typed Epiphany state.",
                "Aquarium should read this before offering live coordination actions.",
            ],
        ),
        read_only_surface_contract(
            SURFACE_FRESHNESS_TYPE,
            FRESHNESS_SURFACE_SCHEMA_VERSION,
            vec![
                "Freshness reflection over retrieval, watcher, and graph staleness signals.",
                "Use this to visualize retrieval and graph staleness without inventing a hidden refresh daemon.",
            ],
        ),
        read_only_surface_contract(
            SURFACE_CONTEXT_TYPE,
            CONTEXT_SURFACE_SCHEMA_VERSION,
            vec![
                "Targeted graph, frontier, checkpoint, observation, and evidence context shard over typed Epiphany state.",
                "Aquarium should inspect bounded state shards here instead of scraping state blobs by superstition.",
            ],
        ),
        read_only_surface_contract(
            SURFACE_GRAPH_QUERY_TYPE,
            GRAPH_QUERY_SURFACE_SCHEMA_VERSION,
            vec![
                "Bounded graph traversal over typed architecture/dataflow graph state.",
                "Use this for architecture/dataflow inspection and frontier neighborhoods without mutating state.",
            ],
        ),
        read_only_surface_contract(
            SURFACE_PRESSURE_TYPE,
            PRESSURE_SURFACE_SCHEMA_VERSION,
            vec![
                "Current context pressure and compaction posture derived from typed pressure inputs.",
                "This is a read-only warning surface, not a backdoor to force state mutation.",
            ],
        ),
        coordinator_surface_contract(
            SURFACE_REORIENT_TYPE,
            REORIENT_SURFACE_SCHEMA_VERSION,
            vec![
                "epiphany.reorient_launch_intent.v0",
                "epiphany.reorient_accept_intent.v0",
            ],
            vec![
                "epiphany.swarm_control_receipt.v0",
                "epiphany.reorient_result_surface.v0",
            ],
            vec![
                "Reorientation policy is the typed resume/regather verdict surface.",
                "Launch and acceptance stay review-gated through explicit typed intents.",
                "The transport may carry a reorient launch intent and receipt, but the resume/regather verdict belongs to core.",
            ],
        ),
        read_only_surface_contract(
            SURFACE_CRRC_TYPE,
            CRRC_SURFACE_SCHEMA_VERSION,
            vec![
                "CRRC recommendation surface over continuity, pressure, and reorientation signals.",
                "Use this to understand continuity pressure without letting CRRC seize authority.",
            ],
        ),
        coordinator_surface_contract(
            SURFACE_JOBS_TYPE,
            JOBS_SURFACE_SCHEMA_VERSION,
            vec![
                "epiphany.job_launch_intent.v0",
                "epiphany.job_interrupt_intent.v0",
            ],
            vec!["epiphany.swarm_control_receipt.v0"],
            vec![
                "Job reflection over typed job bindings and runtime-spine lifecycle receipts.",
                "Heartbeat/runtime-spine owns activation; callers submit typed intents and watch receipts.",
                "Launch receipts must name decisionOwner, transportRole, and any hostExecutorRole so transport never grows an unreadable second opinion.",
            ],
        ),
        coordinator_surface_contract(
            SURFACE_ROLES_TYPE,
            ROLES_SURFACE_SCHEMA_VERSION,
            vec!["epiphany.role_launch_intent.v0"],
            vec!["epiphany.swarm_control_receipt.v0"],
            vec![
                "Role ownership surface for fixed Epiphany lanes and launch affordances.",
                "Treat this as the discoverable lane catalog for Aquarium.",
            ],
        ),
        coordinator_surface_contract(
            SURFACE_ROLE_RESULT_TYPE,
            ROLE_RESULT_SURFACE_SCHEMA_VERSION,
            vec!["epiphany.role_accept_intent.v0"],
            vec!["epiphany.swarm_control_receipt.v0"],
            vec![
                "Role findings are typed review surfaces accepted through explicit role acceptance intents.",
                "Semantic findings remain explicitly review-gated.",
            ],
        ),
        coordinator_surface_contract(
            SURFACE_REORIENT_RESULT_TYPE,
            REORIENT_RESULT_SURFACE_SCHEMA_VERSION,
            vec!["epiphany.reorient_accept_intent.v0"],
            vec!["epiphany.swarm_control_receipt.v0"],
            vec![
                "Completed reorientation findings are typed review surfaces accepted through explicit reorientation acceptance intents.",
            ],
        ),
        coordinator_surface_contract(
            SURFACE_PLANNING_TYPE,
            PLANNING_SURFACE_SCHEMA_VERSION,
            vec![
                "epiphany.planning_update_intent.v0",
                "epiphany.objective_adoption_intent.v0",
            ],
            vec!["epiphany.swarm_control_receipt.v0"],
            vec![
                "Planning projection over captures, backlog, roadmap streams, and Objective Drafts.",
                "Backlog, captures, and Objective Drafts are planning state until explicit adoption.",
            ],
        ),
        read_only_surface_contract(
            SURFACE_COORDINATOR_TYPE,
            COORDINATOR_SURFACE_SCHEMA_VERSION,
            vec![
                "Fixed-lane recommendation surface derived from typed role, pressure, reorientation, and result signals.",
                "Aquarium should treat this as the primary action oracle, not invent its own scheduler.",
            ],
        ),
        coordinator_surface_contract(
            SURFACE_PERSONA_TYPE,
            PERSONA_SURFACE_SCHEMA_VERSION,
            vec![
                "epiphany.persona_bubble_intent.v0",
                "epiphany.character_turn_intent.v0",
                "epiphany.discord_persona_post_intent.v0",
                "epiphany.reddit_persona_post_intent.v0",
            ],
            vec![
                "epiphany.persona_bubble.v0",
                "epiphany.persona_chat.v0",
                "epiphany.character_turn_packet.v0",
            ],
            vec![
                "Persona bubble, draft, Discord, and Reddit public mouth affordances are projected from typed Persona and character-loop artifacts; public crossings route through Bifrost.",
                "Humans talk to Persona; sealed inner thoughts stay behind the projection boundary.",
            ],
        ),
        read_only_surface_contract(
            SURFACE_VOID_MEMORY_TYPE,
            VOID_MEMORY_SURFACE_SCHEMA_VERSION,
            vec![
                "Void-derived memory status/search/context availability is projected from the typed Void memory bridge.",
                "This is an inspection surface for the memory organs, not a license to bypass typed Epiphany state.",
            ],
        ),
        coordinator_surface_contract(
            SURFACE_REPO_INITIALIZATION_TYPE,
            REPO_INITIALIZATION_SURFACE_SCHEMA_VERSION,
            vec![
                "epiphany.repo_startup_intent.v0",
                "epiphany.repo_initialization_accept_intent.v0",
            ],
            vec![
                "epiphany.repo_personality_artifacts.v0",
                "epiphany.repo_initialization_record.v0",
            ],
            vec![
                "Repo birth/startup status is projected from typed repo-personality startup and accept-init receipts.",
                "Birth specialists are startup-only and remain outside the heartbeat lane system.",
            ],
        ),
        coordinator_surface_contract(
            SURFACE_REPO_BIRTH_RUNNER_TYPE,
            REPO_BIRTH_RUNNER_SURFACE_SCHEMA_VERSION,
            vec!["epiphany.repo_birth_run_intent.v0"],
            vec![
                "epiphany.repo_birth_runner.v0",
                "epiphany.repo_initialization_record.v0",
            ],
            vec![
                "Startup-only birth runner plan/run affordances are projected from typed birth-runner receipts.",
                "Aquarium should review birth artifacts and accept them explicitly instead of growing a hidden wizard.",
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
mod tests {
    use super::*;
    use cultnet_rs::CultNetWireContract;
    use cultnet_rs::decode_cultnet_message_from_slice;
    use tempfile::tempdir;

    #[test]
    fn runtime_spine_initializes_sessions_events_and_status() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.msgpack");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "epiphany-test".to_string(),
                display_name: "Epiphany Test".to_string(),
                created_at: "2026-05-06T00:00:00Z".to_string(),
            },
        )?;
        create_runtime_session(
            &store,
            RuntimeSpineSessionOptions {
                session_id: "session-1".to_string(),
                objective: "Build the spine.".to_string(),
                created_at: "2026-05-06T00:01:00Z".to_string(),
                coordinator_note: "Native first.".to_string(),
            },
        )?;
        append_runtime_event(
            &store,
            RuntimeSpineEventOptions {
                event_id: "event-1".to_string(),
                occurred_at: "2026-05-06T00:02:00Z".to_string(),
                event_type: "session.started".to_string(),
                source: "test".to_string(),
                session_id: Some("session-1".to_string()),
                job_id: None,
                summary: "Session started.".to_string(),
            },
        )?;
        put_coordinator_run_receipt(
            &store,
            &EpiphanyCoordinatorRunReceipt {
                schema_version: COORDINATOR_RUN_RECEIPT_SCHEMA_VERSION.to_string(),
                receipt_id: "coordinator-receipt-1".to_string(),
                session_id: "session-1".to_string(),
                thread_id: "thread-1".to_string(),
                mode: "plan".to_string(),
                status: "planned".to_string(),
                final_action: "launchModeling".to_string(),
                final_reason: Some("Modeling should run.".to_string()),
                step_count: 1,
                created_at: "2026-05-06T00:03:00Z".to_string(),
                model_provider: Some("openai-codex".to_string()),
                runtime_store: store.display().to_string(),
                artifact_refs: vec!["coordinator-summary.json".to_string()],
                sealed_artifact_refs: vec!["epiphany-transcript.jsonl".to_string()],
                metadata: BTreeMap::new(),
            },
        )?;

        let status = runtime_spine_status(&store)?;
        assert!(status.present);
        assert_eq!(status.runtime_id.as_deref(), Some("epiphany-test"));
        assert_eq!(status.sessions, 1);
        assert_eq!(status.active_sessions, 1);
        assert_eq!(status.events, 1);
        let mut cache = runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        assert!(
            cache
                .get::<EpiphanyCoordinatorRunReceipt>("coordinator-receipt-1")?
                .is_some()
        );
        Ok(())
    }

    #[test]
    fn runtime_spine_derives_tool_invocation_statuses() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.msgpack");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "epiphany-test".to_string(),
                display_name: "Epiphany Test".to_string(),
                created_at: "2026-05-06T00:00:00Z".to_string(),
            },
        )?;
        let mut cache = runtime_spine_cache(&store)?;
        cache.put(
            "intent:done",
            &EpiphanyToolInvocationIntent::new(
                "done",
                "codex-mcp",
                "smoke-server",
                "smoke_tool",
                "{}",
                "model-request-1",
                "Test completed tool call.",
                "2026-05-06T00:01:00Z",
            ),
        )?;
        cache.put(
            "intent:pending",
            &EpiphanyToolInvocationIntent::new(
                "pending",
                "codex-mcp",
                "smoke-server",
                "waiting_tool",
                "{}",
                "model-request-2",
                "Test pending tool call.",
                "2026-05-06T00:02:00Z",
            ),
        )?;
        let mut failed_receipt = EpiphanyToolInvocationReceipt::new(
            "receipt-done",
            "done",
            "codex-mcp",
            "smoke-server",
            "smoke_tool",
            "failed",
            "2026-05-06T00:03:00Z",
        );
        failed_receipt.error = Some("smoke server absent".to_string());
        cache.put("receipt:done", &failed_receipt)?;

        let status = runtime_spine_status(&store)?;
        assert_eq!(status.tool_invocation_intents, 2);
        assert_eq!(status.tool_invocation_receipts, 1);
        assert_eq!(status.pending_tool_invocations, 1);

        let invocations = runtime_tool_invocation_statuses(&store)?;
        assert_eq!(invocations.len(), 2);
        assert_eq!(invocations[0].intent_id, "done");
        assert_eq!(invocations[0].status, "failed");
        assert_eq!(invocations[0].error.as_deref(), Some("smoke server absent"));
        assert_eq!(invocations[1].intent_id, "pending");
        assert_eq!(invocations[1].status, "pending");
        assert!(invocations[1].receipt_id.is_none());
        Ok(())
    }

    #[test]
    fn runtime_spine_opens_and_completes_native_jobs() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.msgpack");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "epiphany-test".to_string(),
                display_name: "Epiphany Test".to_string(),
                created_at: "2026-05-06T00:00:00Z".to_string(),
            },
        )?;
        create_runtime_session(
            &store,
            RuntimeSpineSessionOptions {
                session_id: "session-1".to_string(),
                objective: "Build the job artery.".to_string(),
                created_at: "2026-05-06T00:01:00Z".to_string(),
                coordinator_note: "Native jobs.".to_string(),
            },
        )?;
        let job = create_runtime_job(
            &store,
            RuntimeSpineJobOptions {
                job_id: "job-1".to_string(),
                session_id: "session-1".to_string(),
                role: "modeling".to_string(),
                created_at: "2026-05-06T00:02:00Z".to_string(),
                summary: "Model the target.".to_string(),
                artifact_refs: vec!["artifact:modeling-plan".to_string()],
            },
        )?;
        assert_eq!(job.status, EpiphanyRuntimeJobStatus::Queued);
        let result = complete_runtime_job(
            &store,
            RuntimeSpineJobResultOptions {
                result_id: "result-1".to_string(),
                job_id: "job-1".to_string(),
                completed_at: "2026-05-06T00:03:00Z".to_string(),
                verdict: "pass".to_string(),
                summary: "Model is ready.".to_string(),
                next_safe_move: "Launch verification.".to_string(),
                evidence_refs: vec!["evidence:model".to_string()],
                artifact_refs: vec!["artifact:model".to_string()],
            },
        )?;
        assert_eq!(result.role, "modeling");
        let status = runtime_spine_status(&store)?;
        assert_eq!(status.jobs, 1);
        assert_eq!(status.open_jobs, 0);
        assert_eq!(status.job_results, 1);
        assert_eq!(status.events, 2);
        let snapshot =
            runtime_job_snapshot(&store, "job-1")?.expect("completed job snapshot should exist");
        assert_eq!(snapshot.job.status, EpiphanyRuntimeJobStatus::Completed);
        assert_eq!(
            snapshot
                .result
                .as_ref()
                .map(|result| result.result_id.as_str()),
            Some("result-1")
        );
        Ok(())
    }

    #[test]
    fn runtime_spine_persists_mind_gateway_review_receipts() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.msgpack");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "epiphany-test".to_string(),
                display_name: "Epiphany Test".to_string(),
                created_at: "2026-05-06T00:00:00Z".to_string(),
            },
        )?;
        let review = MindGatewayReview {
            schema_version: crate::MIND_GATEWAY_REVIEW_SCHEMA_VERSION.to_string(),
            gateway_id: "mind-role-modeling-job-1".to_string(),
            source_kind: "roleWorkerResult".to_string(),
            source_role_id: "modeling".to_string(),
            decision: crate::mind_gateway::MindGatewayDecision::Accept,
            allowed_effects: vec!["statePatch".to_string()],
            refused_effects: Vec::new(),
            reasons: Vec::new(),
            contract: "Mind review is persisted before state admission.".to_string(),
        };

        put_mind_gateway_review(&store, &review)?;
        let commit = crate::mind_state_commit_receipt(
            "mind-commit-1".to_string(),
            &review,
            42,
            vec!["GraphCheckpoint".to_string()],
            "2026-05-06T00:04:00Z".to_string(),
        );
        put_mind_state_commit_receipt(&store, &commit)?;
        let eyes_packet = EyesEvidencePacket {
            schema_version: crate::EYES_EVIDENCE_PACKET_SCHEMA_VERSION.to_string(),
            packet_id: "eyes-packet-1".to_string(),
            source_result_id: "result-1".to_string(),
            source_job_id: "job-1".to_string(),
            source_role_id: "research".to_string(),
            evidence_ids: vec!["ev-1".to_string()],
            observation_ids: vec!["obs-1".to_string()],
            source_refs: vec!["src/lib.rs:1".to_string()],
            summary: "Evidence packet.".to_string(),
            uncertainty: "none declared".to_string(),
            emitted_at: "2026-05-06T00:05:00Z".to_string(),
            contract: "Eyes packet persists as runtime-spine proof.".to_string(),
        };
        put_eyes_evidence_packet(&store, &eyes_packet)?;
        let substrate_grant = SubstrateGateRepoAccessGrantReceipt {
            schema_version: crate::SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION
                .to_string(),
            receipt_id: "substrate-grant-1".to_string(),
            runtime_job_id: "job-1".to_string(),
            binding_id: "research-source-gather-worker".to_string(),
            role: "epiphany-eyes".to_string(),
            authority_scope: "epiphany.role.research".to_string(),
            granted_operations: vec!["read".to_string(), "snapshot".to_string()],
            granted_paths: vec![".".to_string()],
            granted_at: "2026-05-06T00:06:00Z".to_string(),
            contract: "Substrate Gate grant persists as runtime-spine proof.".to_string(),
        };
        put_substrate_gate_repo_access_grant_receipt(&store, &substrate_grant)?;
        let soul_verdict = SoulVerdictReceipt {
            schema_version: crate::SOUL_VERDICT_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "soul-verdict-1".to_string(),
            source_result_id: "result-verify-1".to_string(),
            source_job_id: "job-verify-1".to_string(),
            verdict: "passed".to_string(),
            summary: "Verification passed.".to_string(),
            evidence_ids: vec!["ev-check".to_string()],
            risks: Vec::new(),
            emitted_at: "2026-05-06T00:07:00Z".to_string(),
            contract: "Soul verdict persists as runtime-spine proof.".to_string(),
        };
        put_soul_verdict_receipt(&store, &soul_verdict)?;
        let continuity_recovery = ContinuityRecoveryReceipt {
            schema_version: crate::CONTINUITY_RECOVERY_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "continuity-recovery-1".to_string(),
            source_result_id: "result-reorient-1".to_string(),
            source_job_id: "job-reorient-1".to_string(),
            binding_id: "reorientation-worker".to_string(),
            mode: "resume".to_string(),
            checkpoint_still_valid: "true".to_string(),
            summary: "Checkpoint survives.".to_string(),
            next_safe_move: "Continue.".to_string(),
            files_inspected: vec!["state/map.yaml".to_string()],
            emitted_at: "2026-05-06T00:08:00Z".to_string(),
            contract: "Continuity recovery persists as runtime-spine proof.".to_string(),
        };
        put_continuity_recovery_receipt(&store, &continuity_recovery)?;

        let stored = runtime_mind_gateway_review(&store, "mind-role-modeling-job-1")?
            .expect("Mind review receipt should persist");
        assert_eq!(stored, review);
        let stored_commit = runtime_mind_state_commit_receipt(&store, "mind-commit-1")?
            .expect("Mind state commit receipt should persist");
        assert_eq!(stored_commit, commit);
        let stored_packet = runtime_eyes_evidence_packet(&store, "eyes-packet-1")?
            .expect("Eyes evidence packet should persist");
        assert_eq!(stored_packet, eyes_packet);
        let stored_grant =
            runtime_substrate_gate_repo_access_grant_receipt(&store, "substrate-grant-1")?
                .expect("Substrate Gate grant should persist");
        assert_eq!(stored_grant, substrate_grant);
        let hands_intent = HandsActionIntent {
            schema_version: crate::HANDS_ACTION_INTENT_SCHEMA_VERSION.to_string(),
            intent_id: "hands-intent-1".to_string(),
            runtime_job_id: "job-implementation-1".to_string(),
            binding_id: "implementation-worker".to_string(),
            role: "epiphany-hands".to_string(),
            authority_scope: "epiphany.role.implementation".to_string(),
            requested_action: "patch".to_string(),
            requested_paths: vec!["src/lib.rs".to_string()],
            substrate_gate_grant_receipt_id: "substrate-grant-1".to_string(),
            requested_at: "2026-05-06T00:06:30Z".to_string(),
            contract: "Hands action intent persists as runtime-spine proof.".to_string(),
        };
        put_hands_action_intent(&store, &hands_intent)?;
        let hands_review = crate::hands_action_review_for_intent(
            "hands-review-1".to_string(),
            &hands_intent,
            "approved".to_string(),
            vec!["patch".to_string()],
            vec!["Substrate Gate grant is present.".to_string()],
            "2026-05-06T00:06:40Z".to_string(),
        );
        put_hands_action_review(&store, &hands_review)?;
        let hands_patch = crate::hands_patch_receipt_for_review(
            "hands-patch-1".to_string(),
            &hands_intent,
            &hands_review,
            vec!["src/lib.rs".to_string()],
            "Applied focused patch.".to_string(),
            "2026-05-06T00:06:50Z".to_string(),
        );
        put_hands_patch_receipt(&store, &hands_patch)?;
        let hands_command = crate::hands_command_receipt_for_review(
            "hands-command-1".to_string(),
            &hands_intent,
            &hands_review,
            "cargo test".to_string(),
            "0".to_string(),
            "artifacts/stdout.log".to_string(),
            "artifacts/stderr.log".to_string(),
            "Focused command passed.".to_string(),
            "2026-05-06T00:07:00Z".to_string(),
        );
        put_hands_command_receipt(&store, &hands_command)?;
        let hands_commit = crate::hands_commit_receipt_for_review(
            "hands-commit-1".to_string(),
            &hands_intent,
            &hands_review,
            "abc123".to_string(),
            "main".to_string(),
            vec!["src/lib.rs".to_string()],
            "Committed focused patch.".to_string(),
            "2026-05-06T00:07:10Z".to_string(),
        );
        put_hands_commit_receipt(&store, &hands_commit)?;
        let hands_pr = crate::hands_pr_receipt_for_review(
            "hands-pr-1".to_string(),
            &hands_intent,
            &hands_review,
            &hands_commit,
            "https://github.com/GameCult/EpiphanyAgent/pull/1".to_string(),
            "1".to_string(),
            "Publish focused patch".to_string(),
            "bifrost-publication-receipt-1".to_string(),
            "Published focused patch as pull request.".to_string(),
            "2026-05-06T00:07:20Z".to_string(),
        );
        put_hands_pr_receipt(&store, &hands_pr)?;
        let stored_intent = runtime_hands_action_intent(&store, "hands-intent-1")?
            .expect("Hands action intent should persist");
        assert_eq!(stored_intent, hands_intent);
        let stored_review = runtime_hands_action_review(&store, "hands-review-1")?
            .expect("Hands action review should persist");
        assert_eq!(stored_review, hands_review);
        let stored_patch = runtime_hands_patch_receipt(&store, "hands-patch-1")?
            .expect("Hands patch receipt should persist");
        assert_eq!(stored_patch, hands_patch);
        let stored_command = runtime_hands_command_receipt(&store, "hands-command-1")?
            .expect("Hands command receipt should persist");
        assert_eq!(stored_command, hands_command);
        let stored_commit = runtime_hands_commit_receipt(&store, "hands-commit-1")?
            .expect("Hands commit receipt should persist");
        assert_eq!(stored_commit, hands_commit);
        let stored_pr = runtime_hands_pr_receipt(&store, "hands-pr-1")?
            .expect("Hands PR receipt should persist");
        assert_eq!(stored_pr, hands_pr);
        assert!(runtime_hands_receipt_chain_after(
            &store,
            "2026-05-06T00:06:45Z"
        )?);
        let hands_chain = runtime_latest_hands_receipt_chain_after(&store, "2026-05-06T00:06:45Z")?
            .expect("Hands receipt chain should summarize");
        assert_eq!(hands_chain.patch_receipt_id, "hands-patch-1");
        assert_eq!(hands_chain.command_receipt_id, "hands-command-1");
        assert_eq!(hands_chain.commit_receipt_id, "hands-commit-1");
        assert_eq!(hands_chain.exit_code, "0");
        assert!(!runtime_hands_receipt_chain_after(
            &store,
            "2026-05-06T00:07:15Z"
        )?);
        let stored_verdict = runtime_soul_verdict_receipt(&store, "soul-verdict-1")?
            .expect("Soul verdict should persist");
        assert_eq!(stored_verdict, soul_verdict);
        let stored_recovery = runtime_continuity_recovery_receipt(&store, "continuity-recovery-1")?
            .expect("Continuity recovery should persist");
        assert_eq!(stored_recovery, continuity_recovery);
        Ok(())
    }

    #[test]
    fn latest_hands_chain_uses_latest_same_gate_receipts_before_commit() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.msgpack");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "epiphany-test".to_string(),
                display_name: "Epiphany Test".to_string(),
                created_at: "2026-06-13T00:00:00Z".to_string(),
            },
        )?;
        let intent = HandsActionIntent {
            schema_version: crate::HANDS_ACTION_INTENT_SCHEMA_VERSION.to_string(),
            intent_id: "hands-intent-reused".to_string(),
            runtime_job_id: "hands-job-reused".to_string(),
            binding_id: "implementation-worker".to_string(),
            role: "epiphany-hands".to_string(),
            authority_scope: "epiphany.role.implementation".to_string(),
            requested_action: "continueImplementation".to_string(),
            requested_paths: vec![".".to_string()],
            substrate_gate_grant_receipt_id: "substrate-grant-reused".to_string(),
            requested_at: "2026-06-13T00:00:01Z".to_string(),
            contract: "Test reused Hands gate.".to_string(),
        };
        put_hands_action_intent(&store, &intent)?;
        let review = crate::hands_action_review_for_intent(
            "hands-review-reused".to_string(),
            &intent,
            "approved".to_string(),
            vec![
                "patch".to_string(),
                "command".to_string(),
                "commit".to_string(),
            ],
            vec!["test reused gate".to_string()],
            "2026-06-13T00:00:02Z".to_string(),
        );
        put_hands_action_review(&store, &review)?;

        put_hands_patch_receipt(
            &store,
            &crate::hands_patch_receipt_for_review(
                "hands-patch-old".to_string(),
                &intent,
                &review,
                vec!["old.rs".to_string()],
                "old patch".to_string(),
                "2026-06-13T00:00:03Z".to_string(),
            ),
        )?;
        put_hands_command_receipt(
            &store,
            &crate::hands_command_receipt_for_review(
                "hands-command-old".to_string(),
                &intent,
                &review,
                "cargo test old".to_string(),
                "0".to_string(),
                "old-stdout.log".to_string(),
                "old-stderr.log".to_string(),
                "old command".to_string(),
                "2026-06-13T00:00:04Z".to_string(),
            ),
        )?;
        put_hands_commit_receipt(
            &store,
            &crate::hands_commit_receipt_for_review(
                "hands-commit-old".to_string(),
                &intent,
                &review,
                "oldsha".to_string(),
                "codex/test".to_string(),
                vec!["old.rs".to_string()],
                "old commit".to_string(),
                "2026-06-13T00:00:05Z".to_string(),
            ),
        )?;
        put_hands_patch_receipt(
            &store,
            &crate::hands_patch_receipt_for_review(
                "hands-patch-new".to_string(),
                &intent,
                &review,
                vec!["new.rs".to_string()],
                "new patch".to_string(),
                "2026-06-13T00:00:06Z".to_string(),
            ),
        )?;
        put_hands_command_receipt(
            &store,
            &crate::hands_command_receipt_for_review(
                "hands-command-new".to_string(),
                &intent,
                &review,
                "cargo test new".to_string(),
                "0".to_string(),
                "new-stdout.log".to_string(),
                "new-stderr.log".to_string(),
                "new command".to_string(),
                "2026-06-13T00:00:07Z".to_string(),
            ),
        )?;
        put_hands_commit_receipt(
            &store,
            &crate::hands_commit_receipt_for_review(
                "hands-commit-new".to_string(),
                &intent,
                &review,
                "newsha".to_string(),
                "codex/test".to_string(),
                vec!["new.rs".to_string()],
                "new commit".to_string(),
                "2026-06-13T00:00:08Z".to_string(),
            ),
        )?;

        let chain = runtime_latest_hands_receipt_chain_after(&store, "2026-06-13T00:00:02Z")?
            .expect("latest same-gate Hands chain");
        assert_eq!(chain.patch_receipt_id, "hands-patch-new");
        assert_eq!(chain.command_receipt_id, "hands-command-new");
        assert_eq!(chain.commit_receipt_id, "hands-commit-new");
        assert_eq!(chain.command, "cargo test new");
        assert_eq!(chain.stdout_artifact, "new-stdout.log");
        assert_eq!(chain.commit_sha, "newsha");
        assert_eq!(chain.changed_paths, vec!["new.rs".to_string()]);
        Ok(())
    }

    #[test]
    fn runtime_spine_opens_heartbeat_job_from_single_typed_call() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.msgpack");

        let job = open_runtime_spine_heartbeat_job(
            &store,
            RuntimeSpineHeartbeatJobOptions {
                runtime_id: "epiphany-test".to_string(),
                display_name: "Epiphany Test".to_string(),
                session_id: "epiphany-main".to_string(),
                objective: "Run heartbeat worker.".to_string(),
                coordinator_note: "Test launch.".to_string(),
                job_id: "heartbeat-job-1".to_string(),
                role: "modeling".to_string(),
                binding_id: "modeling-checkpoint-worker".to_string(),
                authority_scope: "modeling".to_string(),
                instruction: "Inspect the checkpoint and return typed role findings.".to_string(),
                launch_document: EpiphanyWorkerLaunchDocument::Role(
                    crate::EpiphanyRoleWorkerLaunchDocument {
                        thread_id: "thread-1".to_string(),
                        role_id: "modeling".to_string(),
                        state_revision: 7,
                        objective: Some("Run heartbeat worker.".to_string()),
                        dynamic_prompt_context: None,
                        active_subgoal_id: None,
                        active_subgoals: Vec::new(),
                        active_graph_node_ids: vec!["node-model".to_string()],
                        investigation_checkpoint: None,
                        scratch: None,
                        invariants: Vec::new(),
                        graphs: None,
                        recent_evidence: Vec::new(),
                        recent_observations: Vec::new(),
                        graph_frontier: None,
                        graph_checkpoint: None,
                        planning: None,
                        churn: None,
                    },
                ),
                output_contract_id: crate::ROLE_WORKER_OUTPUT_CONTRACT_ID.to_string(),
                organ_launch_contract: crate::default_launch_organ_contract(
                    "modeling",
                    "role",
                    crate::ROLE_WORKER_OUTPUT_CONTRACT_ID,
                ),
                created_at: "2026-05-06T00:02:00Z".to_string(),
            },
        )?;

        assert_eq!(job.job_id, "heartbeat-job-1");
        assert_eq!(job.session_id, "epiphany-main");
        assert_eq!(job.role, "modeling");
        let status = runtime_spine_status(&store)?;
        assert_eq!(status.sessions, 1);
        assert_eq!(status.jobs, 1);
        assert_eq!(status.open_jobs, 1);
        assert_eq!(status.events, 1);
        let mut cache = runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let launch_request = cache
            .get::<EpiphanyRuntimeWorkerLaunchRequest>("heartbeat-job-1")?
            .expect("typed worker launch request should be durable");
        assert_eq!(launch_request.job_id, "heartbeat-job-1");
        assert_eq!(launch_request.binding_id, "modeling-checkpoint-worker");
        assert_eq!(launch_request.role, "modeling");
        assert_eq!(
            launch_request.output_contract_id,
            crate::ROLE_WORKER_OUTPUT_CONTRACT_ID
        );
        assert_eq!(launch_request.document_kind, "role");
        assert!(!launch_request.launch_document_msgpack.is_empty());
        assert_eq!(launch_request.launch_document()?.thread_id(), "thread-1");
        assert_eq!(launch_request.organ_launch_contract.document_kind, "role");
        assert!(
            launch_request
                .organ_launch_contract
                .required_receipt_document_types
                .contains(&crate::MIND_GATEWAY_REVIEW_TYPE.to_string())
        );
        assert!(
            launch_request
                .organ_launch_contract
                .receipt_proof_profiles
                .iter()
                .any(|profile| profile.effect_kind
                    == crate::EpiphanyReceiptEffectKind::StateAdmission
                    && profile
                        .required_before_promotion_document_types
                        .contains(&crate::MIND_GATEWAY_REVIEW_TYPE.to_string()))
        );
        Ok(())
    }

    #[test]
    fn heartbeat_launch_plan_leaves_lifecycle_to_runtime_links() {
        let launch_plan = plan_runtime_spine_heartbeat_launch(
            &EpiphanyThreadState::default(),
            RuntimeSpineHeartbeatLaunchPlanOptions {
                binding_id: "modeling-checkpoint-worker".to_string(),
                kind: EpiphanyJobKind::Specialist,
                scope: "role-scoped modeling/checkpoint maintenance".to_string(),
                owner_role: "epiphany-modeler".to_string(),
                authority_scope: "epiphany.role.modeling".to_string(),
                linked_subgoal_ids: vec!["phase-6".to_string()],
                linked_graph_node_ids: vec!["runtime-spine".to_string()],
                instruction: "Model the target before implementation.".to_string(),
                launch_document: EpiphanyWorkerLaunchDocument::Role(
                    crate::EpiphanyRoleWorkerLaunchDocument {
                        thread_id: "thread-1".to_string(),
                        role_id: "modeling".to_string(),
                        state_revision: 7,
                        objective: Some("keep state typed".to_string()),
                        dynamic_prompt_context: None,
                        active_subgoal_id: None,
                        active_subgoals: Vec::new(),
                        active_graph_node_ids: vec!["runtime-spine".to_string()],
                        investigation_checkpoint: None,
                        scratch: None,
                        invariants: Vec::new(),
                        graphs: None,
                        recent_evidence: Vec::new(),
                        recent_observations: Vec::new(),
                        graph_frontier: None,
                        graph_checkpoint: None,
                        planning: None,
                        churn: None,
                    },
                ),
                output_contract_id: "epiphany.worker.role_result.v0".to_string(),
                organ_launch_contract: crate::default_launch_organ_contract(
                    "epiphany.role.modeling",
                    "role",
                    "epiphany.worker.role_result.v0",
                ),
                max_runtime_seconds: Some(60),
                runtime_job_id: "turn-1".to_string(),
            },
        )
        .expect("launch planning should build binding and runtime link");

        assert_eq!(
            launch_plan.binding.authority_scope.as_deref(),
            Some("epiphany.role.modeling")
        );
        assert_eq!(
            launch_plan.runtime_link.id,
            "runtime-link-modeling-checkpoint-worker-turn-1"
        );
        assert_eq!(launch_plan.runtime_link.runtime_job_id, "turn-1");
        assert_eq!(launch_plan.runtime_link.runtime_result_id, None);
        assert_eq!(launch_plan.runtime_link.role_id, "epiphany-modeler");
    }

    #[test]
    fn heartbeat_launch_allows_replacement_after_terminal_runtime_link() {
        let mut state = EpiphanyThreadState::default();
        state.runtime_links.push(EpiphanyRuntimeLink {
            id: "runtime-link-modeling-checkpoint-worker-old".to_string(),
            binding_id: "modeling-checkpoint-worker".to_string(),
            surface: "runtimeResult".to_string(),
            role_id: "epiphany-modeler".to_string(),
            authority_scope: "epiphany.role.modeling".to_string(),
            runtime_job_id: "old-turn".to_string(),
            runtime_result_id: Some("result-old-turn".to_string()),
            linked_subgoal_ids: Vec::new(),
            linked_graph_node_ids: Vec::new(),
        });
        state.runtime_links.push(EpiphanyRuntimeLink {
            id: "runtime-link-modeling-checkpoint-worker-stale-active".to_string(),
            binding_id: "modeling-checkpoint-worker".to_string(),
            surface: "jobLaunch".to_string(),
            role_id: "epiphany-modeler".to_string(),
            authority_scope: "epiphany.role.modeling".to_string(),
            runtime_job_id: "stale-turn".to_string(),
            runtime_result_id: None,
            linked_subgoal_ids: Vec::new(),
            linked_graph_node_ids: Vec::new(),
        });

        let launch_plan = plan_runtime_spine_heartbeat_launch(
            &state,
            RuntimeSpineHeartbeatLaunchPlanOptions {
                binding_id: "modeling-checkpoint-worker".to_string(),
                kind: EpiphanyJobKind::Specialist,
                scope: "role-scoped modeling/checkpoint maintenance".to_string(),
                owner_role: "epiphany-modeler".to_string(),
                authority_scope: "epiphany.role.modeling".to_string(),
                linked_subgoal_ids: Vec::new(),
                linked_graph_node_ids: Vec::new(),
                instruction: "Model the target before implementation.".to_string(),
                launch_document: EpiphanyWorkerLaunchDocument::Role(
                    crate::EpiphanyRoleWorkerLaunchDocument {
                        thread_id: "thread-1".to_string(),
                        role_id: "modeling".to_string(),
                        state_revision: 7,
                        objective: None,
                        dynamic_prompt_context: None,
                        active_subgoal_id: None,
                        active_subgoals: Vec::new(),
                        active_graph_node_ids: Vec::new(),
                        investigation_checkpoint: None,
                        scratch: None,
                        invariants: Vec::new(),
                        graphs: None,
                        recent_evidence: Vec::new(),
                        recent_observations: Vec::new(),
                        graph_frontier: None,
                        graph_checkpoint: None,
                        planning: None,
                        churn: None,
                    },
                ),
                output_contract_id: crate::ROLE_WORKER_OUTPUT_CONTRACT_ID.to_string(),
                organ_launch_contract: crate::default_launch_organ_contract(
                    "epiphany.role.modeling",
                    "role",
                    crate::ROLE_WORKER_OUTPUT_CONTRACT_ID,
                ),
                max_runtime_seconds: Some(60),
                runtime_job_id: "new-turn".to_string(),
            },
        )
        .expect("terminal runtime links should not block replacement launch");

        assert_eq!(launch_plan.runtime_link.runtime_job_id, "new-turn");
    }

    #[test]
    fn runtime_spine_emits_cultnet_hello_frame() -> Result<()> {
        let temp = tempdir()?;
        let store = temp.path().join("runtime.msgpack");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "epiphany-test".to_string(),
                display_name: "Epiphany Test".to_string(),
                created_at: "2026-05-06T00:00:00Z".to_string(),
            },
        )?;
        let frame = runtime_hello_frame(&store)?;
        let payload_len = u32::from_be_bytes([frame[0], frame[1], frame[2], frame[3]]) as usize;
        let message =
            decode_cultnet_message_from_slice(&frame[4..], CultNetWireContract::CultNetSchemaV0)?;
        assert_eq!(payload_len, frame.len() - 4);
        match message {
            CultNetMessage::Hello {
                runtime_id,
                runtime_kind,
                supported_document_types,
                supported_mutation_contracts,
                ..
            } => {
                assert_eq!(runtime_id, "epiphany-test");
                assert_eq!(runtime_kind, "epiphany.native");
                assert!(
                    supported_document_types
                        .unwrap()
                        .contains(&RUNTIME_JOB_RESULT_TYPE.to_string())
                );
                let contracts = supported_mutation_contracts.unwrap();
                let heartbeat_contract = contracts
                    .iter()
                    .find(|contract| contract.document_type == HEARTBEAT_STATE_TYPE)
                    .expect("heartbeat state should advertise mutation contract");
                assert_eq!(
                    heartbeat_contract.authority,
                    CultNetMutationAuthority::Coordinator
                );
                assert!(
                    heartbeat_contract
                        .operations
                        .contains(&CultNetDocumentOperation::IntentSubmit)
                );
                assert!(
                    heartbeat_contract
                        .intent_document_types
                        .as_ref()
                        .is_some_and(|items| items
                            .iter()
                            .any(|item| item == "epiphany.heartbeat_heat_intent.v0"))
                );
                let mind_state_contract = contracts
                    .iter()
                    .find(|contract| contract.document_type == MIND_STATE_EFFECT_PROPOSAL_TYPE)
                    .expect("Mind state-effect proposal should advertise a mutation contract");
                assert_eq!(
                    mind_state_contract.authority,
                    CultNetMutationAuthority::Coordinator
                );
                assert!(
                    mind_state_contract
                        .receipt_document_types
                        .as_ref()
                        .is_some_and(|items| items
                            .iter()
                            .any(|item| item == MIND_STATE_COMMIT_RECEIPT_TYPE))
                );
                assert!(mind_state_contract.notes.as_ref().is_some_and(|notes| {
                    notes
                        .iter()
                        .any(|note| note.contains("persistent state guardian"))
                }));
                let substrate_gate_repo_contract = contracts
                    .iter()
                    .find(|contract| {
                        contract.document_type == SUBSTRATE_GATE_REPO_ACCESS_REQUEST_TYPE
                    })
                    .expect("Substrate Gate repo access should advertise a mutation contract");
                assert_eq!(
                    substrate_gate_repo_contract.authority,
                    CultNetMutationAuthority::Coordinator
                );
                assert!(
                    substrate_gate_repo_contract
                        .receipt_document_types
                        .as_ref()
                        .is_some_and(|items| items
                            .iter()
                            .any(|item| item == SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE))
                );
                assert!(
                    substrate_gate_repo_contract
                        .notes
                        .as_ref()
                        .is_some_and(|notes| {
                            notes
                                .iter()
                                .any(|note| note.contains("repository access protocol"))
                        })
                );
                let eyes_evidence_contract = contracts
                    .iter()
                    .find(|contract| contract.document_type == EYES_EVIDENCE_REQUEST_TYPE)
                    .expect("Eyes evidence request should advertise a mutation contract");
                assert_eq!(
                    eyes_evidence_contract.authority,
                    CultNetMutationAuthority::Coordinator
                );
                assert!(
                    eyes_evidence_contract
                        .receipt_document_types
                        .as_ref()
                        .is_some_and(|items| items
                            .iter()
                            .any(|item| item == EYES_EVIDENCE_PACKET_TYPE))
                );
                assert!(eyes_evidence_contract.notes.as_ref().is_some_and(|notes| {
                    notes
                        .iter()
                        .any(|note| note.contains("evidence ingress guardian"))
                }));
                let coordinator_contract = contracts
                    .iter()
                    .find(|contract| contract.document_type == SURFACE_COORDINATOR_TYPE)
                    .expect("coordinator surface should advertise a read-only contract");
                assert_eq!(
                    coordinator_contract.authority,
                    CultNetMutationAuthority::ReadOnly
                );
                let persona_contract = contracts
                    .iter()
                    .find(|contract| contract.document_type == SURFACE_PERSONA_TYPE)
                    .expect("Persona surface should advertise an interactive contract");
                assert_eq!(
                    persona_contract.authority,
                    CultNetMutationAuthority::Coordinator
                );
                assert!(
                    persona_contract
                        .receipt_document_types
                        .as_ref()
                        .is_some_and(|items| items
                            .iter()
                            .any(|item| item == "epiphany.persona_bubble.v0"))
                );
                let operator_status_contract = contracts
                    .iter()
                    .find(|contract| {
                        contract.document_type == EPIPHANY_CULTMESH_OPERATOR_STATUS_TYPE
                    })
                    .expect("operator status should advertise a read-only CultMesh document");
                assert_eq!(
                    operator_status_contract.authority,
                    CultNetMutationAuthority::ReadOnly
                );
                assert!(
                    operator_status_contract
                        .notes
                        .as_ref()
                        .is_some_and(|notes| {
                            notes
                                .iter()
                                .any(|note| note.contains("Codex app-server status is bridge"))
                        })
                );
                let operator_snapshot_contract = contracts
                    .iter()
                    .find(|contract| {
                        contract.document_type == EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_TYPE
                    })
                    .expect("operator snapshot should advertise a local typed receipt document");
                assert_eq!(
                    operator_snapshot_contract.authority,
                    CultNetMutationAuthority::LocalUser
                );
                assert!(
                    operator_snapshot_contract
                        .operations
                        .contains(&CultNetDocumentOperation::DocumentPut)
                );
                let operator_run_intent_contract = contracts
                    .iter()
                    .find(|contract| {
                        contract.document_type == EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_TYPE
                    })
                    .expect("operator run intent should advertise a local typed action document");
                assert_eq!(
                    operator_run_intent_contract.authority,
                    CultNetMutationAuthority::LocalUser
                );
                assert!(
                    operator_run_intent_contract
                        .receipt_document_types
                        .as_ref()
                        .is_some_and(|items| items
                            .iter()
                            .any(|item| item == EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_TYPE))
                );
                let operator_run_receipt_contract = contracts
                    .iter()
                    .find(|contract| {
                        contract.document_type == EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_TYPE
                    })
                    .expect("operator run receipt should advertise a local typed receipt document");
                assert!(
                    operator_run_receipt_contract
                        .operations
                        .contains(&CultNetDocumentOperation::ReceiptWatch)
                );
                assert!(
                    contracts
                        .iter()
                        .all(|contract| contract.document_type != "epiphany.surface.rider_bridge")
                );
                assert!(
                    contracts
                        .iter()
                        .all(|contract| contract.document_type != "epiphany.surface.unity_bridge")
                );
                let coordinator_run_receipt_contract = contracts
                    .iter()
                    .find(|contract| contract.document_type == COORDINATOR_RUN_RECEIPT_TYPE)
                    .expect("coordinator run receipt should advertise typed receipt contract");
                assert_eq!(
                    coordinator_run_receipt_contract.authority,
                    CultNetMutationAuthority::Coordinator
                );
                assert!(
                    coordinator_run_receipt_contract
                        .operations
                        .contains(&CultNetDocumentOperation::ReceiptWatch)
                );
                let birth_contract = contracts
                    .iter()
                    .find(|contract| contract.document_type == SURFACE_REPO_BIRTH_RUNNER_TYPE)
                    .expect("repo birth runner surface should advertise startup actions");
                assert!(
                    birth_contract
                        .receipt_document_types
                        .as_ref()
                        .is_some_and(|items| items
                            .iter()
                            .any(|item| item == "epiphany.repo_birth_runner.v0"))
                );
                let model_contract = contracts
                    .iter()
                    .find(|contract| contract.document_type == MODEL_REQUEST_TYPE)
                    .expect("model adapter should advertise provider-neutral request contract");
                assert_eq!(
                    model_contract.authority,
                    CultNetMutationAuthority::Coordinator
                );
                assert!(
                    model_contract
                        .receipt_document_types
                        .as_ref()
                        .is_some_and(|items| items.iter().any(|item| item == MODEL_RECEIPT_TYPE))
                );
                let openai_contract = contracts
                    .iter()
                    .find(|contract| contract.document_type == OPENAI_MODEL_REQUEST_TYPE)
                    .expect("OpenAI adapter should advertise typed provider evidence contract");
                assert_eq!(
                    openai_contract.authority,
                    CultNetMutationAuthority::ReadOnly
                );
                assert!(openai_contract.notes.as_ref().is_some_and(|notes| {
                    notes.iter().any(|note| note.contains("adapter projection"))
                }));
                let tool_intent_contract = contracts
                    .iter()
                    .find(|contract| contract.document_type == TOOL_INVOCATION_INTENT_TYPE)
                    .expect("tool invocation intent should advertise typed adapter contract");
                assert_eq!(
                    tool_intent_contract.authority,
                    CultNetMutationAuthority::Coordinator
                );
                assert!(
                    tool_intent_contract
                        .receipt_document_types
                        .as_ref()
                        .is_some_and(|items| items
                            .iter()
                            .any(|item| item == TOOL_INVOCATION_RECEIPT_TYPE))
                );
                let memory_graph_contract = contracts
                    .iter()
                    .find(|contract| contract.document_type == MEMORY_GRAPH_TYPE)
                    .expect("memory graph should advertise a read-only contract");
                assert_eq!(
                    memory_graph_contract.authority,
                    CultNetMutationAuthority::ReadOnly
                );
                let thread_state_contract = contracts
                    .iter()
                    .find(|contract| contract.document_type == THREAD_STATE_TYPE)
                    .expect("thread state should advertise a read-only contract");
                assert_eq!(
                    thread_state_contract.authority,
                    CultNetMutationAuthority::ReadOnly
                );
            }
            other => panic!("expected hello, got {other:?}"),
        }
        Ok(())
    }

    #[test]
    fn runtime_spine_schema_catalog_includes_surface_and_control_receipts() -> Result<()> {
        let registry = epiphany_schema_registry()?;
        let schemas = registry.list(&cultnet_rs::CultNetSchemaCatalogOptions {
            include_schema_json: false,
            schema_ids: None,
            kinds: None,
        });
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some(SURFACE_SCENE_TYPE)
                && schema.schema_version.as_deref() == Some(SCENE_SURFACE_SCHEMA_VERSION)
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some(SURFACE_COORDINATOR_TYPE)
                && schema.schema_version.as_deref() == Some(COORDINATOR_SURFACE_SCHEMA_VERSION)
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some("epiphany.role_launch_intent.v0")
                && schema.schema_version.as_deref() == Some("epiphany.role_launch_intent.v0")
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some("epiphany.heartbeat_initiative_heat.v0")
                && schema.schema_version.as_deref() == Some("epiphany.heartbeat_initiative_heat.v0")
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some("epiphany.heartbeat_heat_intent.v0")
                && schema.schema_version.as_deref() == Some("epiphany.heartbeat_heat_intent.v0")
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some("epiphany.swarm_control_receipt.v0")
                && schema.schema_version.as_deref() == Some("epiphany.swarm_control_receipt.v0")
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some(COORDINATOR_RUN_RECEIPT_TYPE)
                && schema.schema_version.as_deref() == Some(COORDINATOR_RUN_RECEIPT_SCHEMA_VERSION)
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some(MODEL_REQUEST_TYPE)
                && schema.schema_version.as_deref() == Some(MODEL_REQUEST_SCHEMA_VERSION)
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some(MODEL_RECEIPT_TYPE)
                && schema.schema_version.as_deref() == Some(MODEL_RECEIPT_SCHEMA_VERSION)
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some(TOOL_INVOCATION_INTENT_TYPE)
                && schema.schema_version.as_deref() == Some(TOOL_INVOCATION_INTENT_SCHEMA_VERSION)
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some(TOOL_INVOCATION_RECEIPT_TYPE)
                && schema.schema_version.as_deref() == Some(TOOL_INVOCATION_RECEIPT_SCHEMA_VERSION)
        }));
        let receipt_schema_path =
            epiphany_schema_root().join("epiphany.swarm-control-receipt.schema.json");
        let receipt_schema: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(receipt_schema_path)?)?;
        let required = receipt_schema["required"]
            .as_array()
            .expect("receipt schema should list required fields");
        assert!(required.iter().any(|field| field == "decisionOwner"));
        assert!(required.iter().any(|field| field == "transportRole"));
        assert!(
            receipt_schema["properties"]["decisionOwner"]["description"]
                .as_str()
                .is_some_and(|description| description.contains("epiphany-core"))
        );
        assert!(
            receipt_schema["properties"]["transportRole"]["description"]
                .as_str()
                .is_some_and(|description| description.contains("must not make"))
        );
        assert!(
            receipt_schema["properties"]["hostExecutorRole"]["description"]
                .as_str()
                .is_some_and(|description| description.contains("gathered facts"))
        );
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some(OPENAI_MODEL_REQUEST_TYPE)
                && schema.schema_version.as_deref() == Some(OPENAI_MODEL_REQUEST_SCHEMA_VERSION)
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some(OPENAI_MODEL_RECEIPT_TYPE)
                && schema.schema_version.as_deref() == Some(OPENAI_MODEL_RECEIPT_SCHEMA_VERSION)
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some(MEMORY_GRAPH_TYPE)
                && schema.schema_version.as_deref() == Some(MEMORY_GRAPH_SCHEMA_VERSION)
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some(THREAD_STATE_TYPE)
                && schema.schema_version.as_deref() == Some(THREAD_STATE_SCHEMA_VERSION)
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some(EPIPHANY_CULTMESH_OPERATOR_STATUS_TYPE)
                && schema.schema_version.as_deref()
                    == Some(EPIPHANY_CULTMESH_OPERATOR_STATUS_SCHEMA_VERSION)
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some(EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_TYPE)
                && schema.schema_version.as_deref()
                    == Some(EPIPHANY_CULTMESH_OPERATOR_SNAPSHOT_SCHEMA_VERSION)
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some(EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_TYPE)
                && schema.schema_version.as_deref()
                    == Some(EPIPHANY_CULTMESH_OPERATOR_RUN_INTENT_SCHEMA_VERSION)
        }));
        assert!(schemas.iter().any(|schema| {
            schema.document_type.as_deref() == Some(EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_TYPE)
                && schema.schema_version.as_deref()
                    == Some(EPIPHANY_CULTMESH_OPERATOR_RUN_RECEIPT_SCHEMA_VERSION)
        }));
        assert!(
            !schemas.iter().any(|schema| {
                schema.document_type.as_deref() == Some("epiphany.surface.rider_bridge")
            }),
            "Rider bridge schema is quarantined and should not be advertised in the active catalog"
        );
        assert!(
            !schemas.iter().any(|schema| {
                schema.document_type.as_deref() == Some("epiphany.surface.unity_bridge")
            }),
            "Unity bridge schema is quarantined and should not be advertised in the active catalog"
        );
        Ok(())
    }
}
