use crate::EpiphanyWorkerLaunchDocument;
use crate::RepoFrontierPlanMindContextProjection;
use crate::agent_launch::{
    EPIPHANY_IMAGINATION_OWNER_ROLE, EPIPHANY_IMAGINATION_ROLE_BINDING_ID,
    EPIPHANY_MIND_OWNER_ROLE, EPIPHANY_MIND_ROLE_BINDING_ID, EPIPHANY_MODELING_OWNER_ROLE,
    EPIPHANY_MODELING_ROLE_BINDING_ID,
};
use crate::eyes_gateway::EyesEvidencePacket;
use crate::eyes_gateway::EyesSourceLookupReceipt;
use crate::hands_gateway::*;
use crate::repo_model_gateway::{
    RUNTIME_REPOSITORY_DOMAIN_BINDING_KEY, RepoFrontierHandsAuthority, RepoFrontierModelingRequest,
    RepoFrontierNextOrgan, RepoFrontierPlanCandidate, RepoFrontierPlanDecision,
    RepoFrontierPlanDecisionReceipt, RepoFrontierPlanMindDecision, RepoFrontierPlanMindRequest,
    RepoFrontierPlanningFailureReview, RepoFrontierPlanningLifecycle,
    RepoFrontierPlanningLifecycleStage, RepoFrontierPlanningRequest,
    RepoFrontierProposalModelingRequest, RepoFrontierResearchRequest, RepoFrontierRoute,
    RepoFrontierVerdictDisposition, RepoFrontierWorkProposal, RuntimeRepositoryDomainBinding,
};
use crate::runtime_store_backend::{
    RuntimeSpineBackingStore as SingleFileMessagePackBackingStore, runtime_spine_backing_store,
};
use crate::soul_gateway::SoulVerdictReceipt;
use crate::soul_gateway::*;
use crate::substrate_gate::SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION;
use crate::substrate_gate::SubstrateGateRepoAccessGrantReceipt;
use crate::{RuntimeTypedRequestRef, WorkerProcessStatus};
use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use cultcache_rs::CacheBackingStore;
use cultcache_rs::CultCache;
use cultcache_rs::CultCacheEnvelope;
use cultcache_rs::DatabaseEntry;
use epiphany_model_adapter::EpiphanyModelReceipt;
use epiphany_model_adapter::EpiphanyModelRequest;
use epiphany_model_adapter::EpiphanyModelStreamEvent;
use epiphany_model_adapter::EpiphanyModelStreamPayload;
use epiphany_openai_adapter::EpiphanyOpenAiModelRequest;
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
use std::path::Path;
use std::path::PathBuf;

pub const RUNTIME_IDENTITY_TYPE: &str = "epiphany.runtime.identity";
pub const COORDINATOR_RUN_RECEIPT_TYPE: &str = "epiphany.coordinator_run_receipt.v1";
pub const RUNTIME_IDENTITY_KEY: &str = "self";
pub const RUNTIME_SWARM_BINDING_KEY: &str = "runtime-swarm-binding";
pub const RUNTIME_SWARM_BINDING_SCHEMA_VERSION: &str = "epiphany.runtime.swarm_binding.v1";
pub const RUNTIME_SPINE_SCHEMA_VERSION: &str = "epiphany.runtime_spine.v34";
pub const EPIPHANY_RUNTIME_ROOT_SESSION_ID: &str = "epiphany-main";
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
    pub created_at: String,
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
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(type = "epiphany.runtime.job", schema = "EpiphanyRuntimeJob")]
pub struct EpiphanyRuntimeJob {
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
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.model_execution_binding",
    schema = "EpiphanyRuntimeModelExecutionBinding"
)]
pub struct EpiphanyRuntimeModelExecutionBinding {
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
struct EpiphanyArchivedRuntimeSession {
    #[cultcache(key = 1)]
    session_id: String,
    #[cultcache(key = 2)]
    job_ids: Vec<String>,
    #[cultcache(key = 3)]
    model_request_ids: Vec<String>,
    #[cultcache(key = 4)]
    tool_intent_ids: Vec<String>,
    #[cultcache(key = 5)]
    retired_chain_digest: String,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.worker_launch_request",
    schema = "EpiphanyRuntimeWorkerLaunchRequest"
)]
pub struct EpiphanyRuntimeWorkerLaunchRequest {
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
    #[cultcache(key = 11, default)]
    pub proposal_modeling_request_id: Option<String>,
    #[cultcache(key = 12, default)]
    pub repo_frontier_verification_request_id: Option<String>,
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
    #[cultcache(key = 19, default)]
    pub repo_frontier_research_request_id: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.worker_process_claim.v0",
    schema = "EpiphanyRuntimeWorkerProcessClaim"
)]
pub struct EpiphanyRuntimeWorkerProcessClaim {
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

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.archived_worker_attempt.v2",
    schema = "EpiphanyArchivedRuntimeWorkerAttempt"
)]
pub struct EpiphanyArchivedRuntimeWorkerAttempt {
    #[cultcache(key = 1)]
    pub job_id: String,
    #[cultcache(key = 2)]
    pub request_kind: String,
    #[cultcache(key = 3)]
    pub request_id: String,
    #[cultcache(key = 4)]
    pub terminal_process_status: String,
    #[cultcache(key = 5)]
    pub retired_chain_digest: String,
    #[cultcache(key = 6)]
    pub decision: Option<EpiphanyArchivedRuntimeWorkerDecision>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EpiphanyArchivedRuntimeWorkerDecision {
    pub decision_context_id: String,
    pub role_result: Option<EpiphanyRuntimeRoleWorkerResult>,
    pub job_results: Vec<EpiphanyRuntimeJobResult>,
}

impl EpiphanyArchivedRuntimeWorkerAttempt {
    pub fn decision_context_id(&self) -> Option<&str> {
        self.decision
            .as_ref()
            .map(|decision| decision.decision_context_id.as_str())
    }

    pub fn fulfilled_result_id(&self) -> Option<&str> {
        self.decision
            .as_ref()
            .and_then(|decision| decision.role_result.as_ref())
            .map(|result| result.result_id.as_str())
    }

    pub(crate) fn validate_decision_record(&self, fulfilled: bool) -> Result<()> {
        let Some(decision) = &self.decision else {
            if fulfilled {
                return Err(anyhow!(
                    "fulfilled worker archive lost its structured decision"
                ));
            }
            return Ok(());
        };
        validate_non_empty(
            &decision.decision_context_id,
            "archived worker decision context id",
        )?;
        if decision.job_results.is_empty()
            || decision.job_results.iter().any(|result| {
                result.job_id != self.job_id
                    || result.decision_context_id.as_deref()
                        != Some(decision.decision_context_id.as_str())
            })
        {
            return Err(anyhow!(
                "archived worker decision lost its exact generic result family"
            ));
        }
        match &decision.role_result {
            Some(result)
                if fulfilled
                    && result.job_id == self.job_id
                    && result.decision_context_id == decision.decision_context_id =>
            {
                Ok(())
            }
            None if !fulfilled => Ok(()),
            _ => Err(anyhow!(
                "archived worker decision terminal result shape is invalid"
            )),
        }
    }
}

fn worker_process_claim_id(job_id: &str) -> String {
    format!("runtime-worker-process-{job_id}")
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoFrontierVerdictModelingLaunchAuthority {
    pub request: RepoFrontierModelingRequest,
    pub frontier_item: crate::RepoFrontierItem,
    pub soul_verdict: SoulVerdictReceipt,
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
    pub research_decision_msgpack: Option<Vec<u8>>,
    #[cultcache(key = 17, default)]
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
    pub repo_frontier_research_request_id: Option<String>,
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
    pub fn research_decision(&self) -> Result<Option<crate::EpiphanyResearchDecision>> {
        decode_optional_msgpack(
            self.research_decision_msgpack.as_deref(),
            "role worker researchDecision",
        )
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
#[cultcache(
    type = "epiphany.coordinator_run_receipt.v1",
    schema = "EpiphanyCoordinatorRunReceipt"
)]
pub struct EpiphanyCoordinatorRunReceipt {
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
    pub created_at: String,
    #[cultcache(key = 9, default)]
    pub resident_grant_id: Option<String>,
    #[cultcache(key = 10, default)]
    pub resident_launch_digest: Option<String>,
    #[cultcache(key = 11, default)]
    pub resident_policy_digest: Option<String>,
    #[cultcache(key = 12, default)]
    pub resident_argv_digest: Option<String>,
    #[cultcache(key = 13, default)]
    pub resident_objective_digest: Option<String>,
    #[cultcache(key = 14, default)]
    pub resident_release_commit: Option<String>,
    #[cultcache(key = 15, default)]
    pub resident_release_manifest_digest: Option<String>,
    #[cultcache(key = 16, default)]
    pub resident_executable_digest: Option<String>,
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
pub struct ModelPassFailureTerminalOptions {
    pub decision_context_id: String,
    pub failure_kind: String,
    pub summary: String,
    pub failed_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeSpineJobOptions {
    pub job_id: String,
    pub session_id: String,
    pub role: String,
    pub created_at: String,
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
    pub proposal_modeling_request_id: Option<String>,
    pub frontier_planning_request_id: Option<String>,
    pub frontier_plan_mind_request_id: Option<String>,
    pub imagination_consideration_request_id: Option<String>,
    pub admitted_model_direction_consideration_request_id: Option<String>,
    pub repo_frontier_modeling_request_id: Option<String>,
    pub repo_frontier_research_request_id: Option<String>,
    pub repo_frontier_verification_request_id: Option<String>,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EpiphanyRuntimeJobSnapshot {
    pub job: EpiphanyRuntimeJob,
    pub result: Option<EpiphanyRuntimeJobResult>,
}

pub fn runtime_spine_cache(store_path: impl AsRef<Path>) -> Result<CultCache> {
    let store_path = store_path.as_ref();
    let backing_store = runtime_spine_backing_store(store_path)?;
    validate_runtime_store_epoch(&backing_store.pull_all()?)?;
    let mut cache = runtime_spine_schema_cache()?;
    cache.add_generic_backing_store(backing_store);
    Ok(cache)
}

fn runtime_spine_schema_cache() -> Result<CultCache> {
    let mut cache = CultCache::new();
    crate::mind_documents::register_mind_document_types(&mut cache)?;
    cache.register_entry_type::<crate::UserObjectiveIntake>()?;
    cache.register_entry_type::<EpiphanyRuntimeIdentity>()?;
    cache.register_entry_type::<EpiphanyRuntimeSwarmBinding>()?;
    cache.register_entry_type::<crate::AtlasDependencyVerificationWriteIntent>()?;
    cache.register_entry_type::<crate::AtlasDependencyImpactWriteIntent>()?;
    cache.register_entry_type::<EpiphanyRuntimeSession>()?;
    cache.register_entry_type::<EpiphanyRuntimeJob>()?;
    cache.register_entry_type::<EpiphanyRuntimeModelExecutionBinding>()?;
    cache.register_entry_type::<crate::EpiphanyReasoningBasis>()?;
    cache.register_entry_type::<crate::EpiphanyDecisionContext>()?;
    cache.register_entry_type::<crate::EpiphanyModelPassFailure>()?;
    cache.register_entry_type::<crate::EpiphanyMindCommitReceipt>()?;
    cache.register_entry_type::<EpiphanyRuntimeToolExecutionBinding>()?;
    cache.register_entry_type::<EpiphanyArchivedRuntimeSession>()?;
    cache.register_entry_type::<EpiphanyRuntimeWorkerLaunchRequest>()?;
    cache.register_entry_type::<EpiphanyRuntimeWorkerProcessClaim>()?;
    cache.register_entry_type::<EpiphanyArchivedRuntimeWorkerAttempt>()?;
    cache.register_entry_type::<EpiphanyRuntimeRoleWorkerResult>()?;
    cache.register_entry_type::<crate::RuntimeRepositoryBodyStoreBinding>()?;
    cache.register_entry_type::<RepoFrontierRoute>()?;
    cache.register_entry_type::<RepoFrontierHandsAuthority>()?;
    cache.register_entry_type::<RepoFrontierModelingRequest>()?;
    cache.register_entry_type::<RepoFrontierWorkProposal>()?;
    cache.register_entry_type::<RuntimeRepositoryDomainBinding>()?;
    cache.register_entry_type::<RepoFrontierProposalModelingRequest>()?;
    cache.register_entry_type::<RepoFrontierPlanningRequest>()?;
    cache.register_entry_type::<RepoFrontierResearchRequest>()?;
    cache.register_entry_type::<RepoFrontierPlanningFailureReview>()?;
    cache.register_entry_type::<crate::ImaginationConsiderationRequest>()?;
    cache.register_entry_type::<crate::ImaginationConsiderationCandidate>()?;
    cache.register_entry_type::<crate::AdmittedModelDirectionConsiderationRequest>()?;
    cache.register_entry_type::<crate::AdmittedModelDirectionConsiderationResult>()?;
    cache.register_entry_type::<RepoFrontierPlanCandidate>()?;
    cache.register_entry_type::<RepoFrontierPlanMindRequest>()?;
    cache.register_entry_type::<crate::EpiphanyReorientationRequest>()?;
    cache.register_entry_type::<crate::EpiphanyMindReorientationDecisionDocument>()?;
    cache.register_entry_type::<crate::EpiphanyMindReorientationPassFailureDocument>()?;
    cache.register_entry_type::<RepoFrontierVerificationRequest>()?;
    cache.register_entry_type::<EpiphanyRuntimeReorientWorkerResult>()?;
    cache.register_entry_type::<EpiphanyRuntimeJobResult>()?;
    cache.register_entry_type::<EpiphanyCoordinatorRunReceipt>()?;
    cache.register_entry_type::<EpiphanyCoordinatorDeathRecovery>()?;
    cache.register_entry_type::<EyesEvidencePacket>()?;
    cache.register_entry_type::<EyesSourceLookupReceipt>()?;
    cache.register_entry_type::<SubstrateGateRepoAccessGrantReceipt>()?;
    cache.register_entry_type::<HandsActionIntent>()?;
    cache.register_entry_type::<HandsActionReview>()?;
    cache.register_entry_type::<HandsPatchReceipt>()?;
    cache.register_entry_type::<HandsCommandReceipt>()?;
    cache.register_entry_type::<HandsCommitReceipt>()?;
    cache.register_entry_type::<SoulVerdictReceipt>()?;
    cache.register_entry_type::<EpiphanyOpenAiModelRequest>()?;
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
    cache.register_entry_type::<EpiphanyToolInvocationIntent>()?;
    cache.register_entry_type::<EpiphanyToolInvocationReceipt>()?;
    Ok(cache)
}

fn validate_runtime_store_epoch(envelopes: &[CultCacheEnvelope]) -> Result<()> {
    if envelopes.is_empty() {
        return Ok(());
    }
    let mind_identities = envelopes
        .iter()
        .filter(|envelope| envelope.r#type == crate::EpiphanyMindIdentity::TYPE)
        .collect::<Vec<_>>();
    let runtime_identities = envelopes
        .iter()
        .filter(|envelope| envelope.r#type == EpiphanyRuntimeIdentity::TYPE)
        .collect::<Vec<_>>();
    if mind_identities.is_empty() && runtime_identities.is_empty() {
        return Ok(());
    }
    if mind_identities.len() != 1 || runtime_identities.len() != 1 {
        return Err(anyhow!(
            "claimed runtime Mind store does not have one current schema identity pair"
        ));
    }
    let mind_envelope = mind_identities[0];
    let runtime_envelope = runtime_identities[0];
    let mind: crate::EpiphanyMindIdentity = rmp_serde::from_slice(&mind_envelope.payload)
        .map_err(|error| anyhow!("runtime Mind schema identity is invalid: {error}"))?;
    let runtime: EpiphanyRuntimeIdentity = rmp_serde::from_slice(&runtime_envelope.payload)
        .map_err(|error| anyhow!("runtime schema identity is invalid: {error}"))?;
    if mind_envelope.key != crate::MIND_SCHEMA_EPOCH
        || mind.schema_epoch != crate::MIND_SCHEMA_EPOCH
        || runtime_envelope.key != RUNTIME_IDENTITY_KEY
        || runtime.schema_version != RUNTIME_SPINE_SCHEMA_VERSION
        || mind.runtime_id != runtime.runtime_id
    {
        return Err(anyhow!(
            "nonempty runtime Mind store belongs to an unsupported writable schema epoch"
        ));
    }
    Ok(())
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
    if let Some(existing) = existing {
        let existing_mind = existing_mind.expect("identity pair was checked above");
        if existing_mind.runtime_id != existing.runtime_id {
            return Err(anyhow!("runtime and Mind identities disagree"));
        }
        return Ok(existing);
    }
    let identity = EpiphanyRuntimeIdentity {
        schema_version: RUNTIME_SPINE_SCHEMA_VERSION.to_string(),
        runtime_id: options.runtime_id,
        display_name: options.display_name,
        created_at: options.created_at,
    };
    let mind_identity = crate::EpiphanyMindIdentity {
        schema_epoch: crate::MIND_SCHEMA_EPOCH.to_string(),
        runtime_id: identity.runtime_id.clone(),
    };
    debug_assert!(existing_mind.is_none());
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

pub fn bind_runtime_to_swarm(
    runtime_store: impl AsRef<Path>,
    swarm_id: &str,
    bound_at: &str,
) -> Result<EpiphanyRuntimeSwarmBinding> {
    chrono::DateTime::parse_from_rfc3339(bound_at)
        .map_err(|_| anyhow!("runtime swarm binding timestamp must be RFC3339"))?;
    if swarm_id.trim().is_empty() {
        return Err(anyhow!("runtime swarm binding requires swarm id"));
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
        swarm_id: swarm_id.to_string(),
        source_identity_type: RUNTIME_IDENTITY_TYPE.to_string(),
        source_identity_key: RUNTIME_IDENTITY_KEY.to_string(),
        source_identity_sha256: format!(
            "{:x}",
            Sha256::digest(rmp_serde::to_vec_named(&identity)?)
        ),
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
        session_id: options.session_id.clone(),
        objective: options.objective,
        status: EpiphanyRuntimeSessionStatus::Active,
        created_at: options.created_at.clone(),
        updated_at: options.created_at,
        coordinator_note: options.coordinator_note,
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
    session.status = EpiphanyRuntimeSessionStatus::Completed;
    session.updated_at = options.completed_at.clone();
    session.coordinator_note = options.summary.clone();
    cache.put(&session.session_id, &session)?;
    Ok(session)
}

/// Atomically makes one failed model pass auditable and closes the exact
/// runtime session that carried it. The sealed decision context is the owner;
/// provider events and assistant deltas are not required for replay.
pub fn terminalize_model_pass_failure_session(
    store_path: impl AsRef<Path>,
    options: ModelPassFailureTerminalOptions,
) -> Result<crate::EpiphanyModelPassFailure> {
    for (value, label) in [
        (&options.decision_context_id, "decision context id"),
        (&options.failure_kind, "failure kind"),
        (&options.summary, "failure summary"),
        (&options.failed_at, "failure time"),
    ] {
        validate_non_empty(value, label)?;
    }
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    let context = cache
        .get::<crate::EpiphanyDecisionContext>(&options.decision_context_id)?
        .ok_or_else(|| anyhow!("model pass failure lost its decision context"))?;
    let basis = cache
        .get::<crate::EpiphanyReasoningBasis>(&context.basis_id)?
        .ok_or_else(|| anyhow!("model pass failure lost its reasoning basis"))?;
    let binding = validate_runtime_model_execution_binding(&cache, &context.terminal_request_id)?;
    if binding.session_id == EPIPHANY_RUNTIME_ROOT_SESSION_ID {
        return Err(anyhow!(
            "model pass failure cannot close the runtime root session"
        ));
    }
    let failure = crate::EpiphanyModelPassFailure::new(
        &basis,
        &context,
        binding.session_id.clone(),
        binding.job_id.clone(),
        options.failure_kind.clone(),
        options.summary.clone(),
        options.failed_at.clone(),
    )?;
    if let Some(existing) = cache.get::<crate::EpiphanyModelPassFailure>(&failure.failure_id)? {
        existing.validate(&basis, &context)?;
        if existing != failure {
            return Err(anyhow!("model pass failure identity collision"));
        }
        let session = cache
            .get::<EpiphanyRuntimeSession>(&binding.session_id)?
            .ok_or_else(|| anyhow!("model pass failure lost its runtime session"))?;
        if session.status != EpiphanyRuntimeSessionStatus::Completed {
            return Err(anyhow!(
                "model pass failure exists without terminal runtime session"
            ));
        }
        return Ok(existing);
    }
    let session_envelope = cache
        .get_envelope::<EpiphanyRuntimeSession>(&binding.session_id)?
        .ok_or_else(|| anyhow!("model pass failure runtime session is absent"))?;
    let mut session = cache
        .get::<EpiphanyRuntimeSession>(&binding.session_id)?
        .ok_or_else(|| anyhow!("model pass failure runtime session is absent"))?;
    if session.status != EpiphanyRuntimeSessionStatus::Active {
        return Err(anyhow!("model pass failure runtime session is not active"));
    }
    let model_job_envelope = cache
        .get_envelope::<EpiphanyRuntimeJob>(&binding.job_id)?
        .ok_or_else(|| anyhow!("model pass failure runtime job is absent"))?;
    let mut model_job = cache
        .get::<EpiphanyRuntimeJob>(&binding.job_id)?
        .ok_or_else(|| anyhow!("model pass failure runtime job is absent"))?;
    if model_job.session_id != binding.session_id {
        return Err(anyhow!(
            "model pass failure runtime job is outside its exact session"
        ));
    }
    let model_job_is_live = matches!(
        model_job.status,
        EpiphanyRuntimeJobStatus::Queued
            | EpiphanyRuntimeJobStatus::Running
            | EpiphanyRuntimeJobStatus::WaitingForReview
    );
    let model_job_results = cache
        .get_all::<EpiphanyRuntimeJobResult>()?
        .into_iter()
        .filter(|result| result.job_id == binding.job_id)
        .collect::<Vec<_>>();
    if model_job_is_live && !model_job_results.is_empty() {
        return Err(anyhow!("live model pass job already has a terminal result"));
    }
    if !model_job_is_live
        && (model_job_results.len() != 1 || model_job_results[0].decision_context_id.is_some())
    {
        return Err(anyhow!(
            "terminal model transport job lost its non-authoritative result"
        ));
    }
    let open_job_ids = cache
        .get_all::<EpiphanyRuntimeJob>()?
        .into_iter()
        .filter(|job| {
            job.session_id == binding.session_id
                && (!model_job_is_live || job.job_id != binding.job_id)
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
            "model pass failure session still has open jobs: {}",
            open_job_ids.join(", ")
        ));
    }
    session.status = EpiphanyRuntimeSessionStatus::Completed;
    session.updated_at = options.failed_at.clone();
    session.coordinator_note = options.summary.clone();
    let mut expected = vec![session_envelope];
    let mut replacements = vec![
        cache.prepare_entry(&failure.failure_id, &failure)?.0,
        cache.prepare_entry(&session.session_id, &session)?.0,
    ];
    if model_job_is_live {
        model_job.status = EpiphanyRuntimeJobStatus::Failed;
        model_job.updated_at = options.failed_at.clone();
        let model_result = EpiphanyRuntimeJobResult {
            result_id: format!("result-model-pass-failure-{}", failure.failure_id),
            job_id: binding.job_id.clone(),
            session_id: binding.session_id.clone(),
            role: model_job.role.clone(),
            verdict: "failed".to_string(),
            summary: options.summary.clone(),
            completed_at: options.failed_at.clone(),
            next_safe_move:
                "Inspect the sealed decision context and typed model-pass failure before retrying."
                    .to_string(),
            evidence_refs: Vec::new(),
            artifact_refs: Vec::new(),
            metadata: BTreeMap::new(),
            decision_context_id: None,
        };
        expected.push(model_job_envelope);
        replacements.push(cache.prepare_entry(&model_job.job_id, &model_job)?.0);
        replacements.push(
            cache
                .prepare_entry(&model_result.result_id, &model_result)?
                .0,
        );
    }
    if !runtime_spine_backing_store(store_path)?.compare_and_swap_batch(&expected, replacements)? {
        return terminalize_model_pass_failure_session(store_path, options);
    }
    Ok(failure)
}

pub fn model_pass_failure_for_request(
    store_path: impl AsRef<Path>,
    model_request_id: &str,
) -> Result<Option<crate::EpiphanyModelPassFailure>> {
    validate_non_empty(model_request_id, "model request id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let mut matches = cache
        .get_all::<crate::EpiphanyModelPassFailure>()?
        .into_iter()
        .filter(|failure| failure.model_request_id == model_request_id);
    let failure = matches.next();
    if matches.next().is_some() {
        return Err(anyhow!(
            "model request has multiple terminal failure records"
        ));
    }
    let Some(failure) = failure else {
        return Ok(None);
    };
    let context = cache
        .get::<crate::EpiphanyDecisionContext>(&failure.decision_context_id)?
        .ok_or_else(|| anyhow!("model pass failure lost its decision context"))?;
    let basis = cache
        .get::<crate::EpiphanyReasoningBasis>(&failure.reasoning_basis_id)?
        .ok_or_else(|| anyhow!("model pass failure lost its reasoning basis"))?;
    failure.validate(&basis, &context)?;
    let binding = validate_runtime_model_execution_binding(&cache, &failure.model_request_id)?;
    if failure.runtime_session_id != binding.session_id || failure.runtime_job_id != binding.job_id
    {
        return Err(anyhow!(
            "model pass failure disagrees with its exact runtime binding"
        ));
    }
    Ok(Some(failure))
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
        job_id: options.job_id.clone(),
        session_id: options.session_id.clone(),
        role: options.role,
        status: EpiphanyRuntimeJobStatus::Queued,
        created_at: options.created_at.clone(),
        updated_at: options.created_at.clone(),
    };
    cache.put(&options.job_id, &job)?;
    Ok(job)
}

pub fn open_runtime_model_execution(
    store_path: impl AsRef<Path>,
    session_options: RuntimeSpineSessionOptions,
    job_options: RuntimeSpineJobOptions,
    model_request: &EpiphanyModelRequest,
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
    let provider_request = epiphany_openai_adapter::request_from_native(model_request);

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
            session_id: session_options.session_id.clone(),
            objective: session_options.objective.clone(),
            status: EpiphanyRuntimeSessionStatus::Active,
            created_at: session_options.created_at.clone(),
            updated_at: session_options.created_at.clone(),
            coordinator_note: session_options.coordinator_note.clone(),
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
        job_id: job_options.job_id.clone(),
        session_id: session.session_id.clone(),
        role: job_options.role,
        status: EpiphanyRuntimeJobStatus::Queued,
        created_at: job_options.created_at.clone(),
        updated_at: job_options.created_at.clone(),
    };
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
        cache.prepare_entry(&binding_id, &binding)?.0,
        cache
            .prepare_entry(&model_request.request_id, model_request)?
            .0,
        cache
            .prepare_entry(&provider_request.request_id, &provider_request)?
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

pub(crate) fn validate_runtime_model_execution_binding(
    cache: &CultCache,
    request_id: &str,
) -> Result<EpiphanyRuntimeModelExecutionBinding> {
    validate_non_empty(request_id, "model execution request id")?;
    let binding = cache
        .get::<EpiphanyRuntimeModelExecutionBinding>(request_id)?
        .ok_or_else(|| anyhow!("model execution request {request_id:?} is unbound"))?;
    let native = cache
        .get::<EpiphanyModelRequest>(request_id)?
        .ok_or_else(|| anyhow!("model execution binding {request_id:?} lost its native request"))?;
    let provider = cache
        .get::<EpiphanyOpenAiModelRequest>(request_id)?
        .ok_or_else(|| {
            anyhow!("model execution binding {request_id:?} lost its provider request")
        })?;
    let session = cache
        .get::<EpiphanyRuntimeSession>(&binding.session_id)?
        .ok_or_else(|| anyhow!("model execution binding {request_id:?} lost its session"))?;
    let job = cache
        .get::<EpiphanyRuntimeJob>(&binding.job_id)?
        .ok_or_else(|| anyhow!("model execution binding {request_id:?} lost its job"))?;
    if binding.binding_id != request_id
        || binding.request_id != request_id
        || binding.provider != native.provider
        || binding.source_worker_job_id != native.source_worker_job_id
        || binding.reasoning_basis_id != native.reasoning_basis_id
        || chrono::DateTime::parse_from_rfc3339(&binding.bound_at).is_err()
        || provider != epiphany_openai_adapter::request_from_native(&native)
        || session.status == EpiphanyRuntimeSessionStatus::Archived
        || job.session_id != binding.session_id
    {
        return Err(anyhow!(
            "model execution binding {request_id:?} is not one exact request family"
        ));
    }
    if let Some(worker_job_id) = binding.source_worker_job_id.as_deref() {
        let basis_id = binding
            .reasoning_basis_id
            .as_deref()
            .ok_or_else(|| anyhow!("decision-bearing model execution lost its reasoning basis"))?;
        let basis = cache
            .get::<crate::EpiphanyReasoningBasis>(basis_id)?
            .ok_or_else(|| anyhow!("model execution binding lost its reasoning basis"))?;
        let launch = cache
            .get::<EpiphanyRuntimeWorkerLaunchRequest>(worker_job_id)?
            .ok_or_else(|| anyhow!("model execution binding lost its source worker launch"))?;
        let worker_job = cache
            .get::<EpiphanyRuntimeJob>(worker_job_id)?
            .ok_or_else(|| anyhow!("model execution binding lost its source worker job"))?;
        if basis.pass_id != worker_job_id
            || launch.job_id != worker_job_id
            || worker_job.role != launch.role
        {
            return Err(anyhow!(
                "model execution binding {request_id:?} has foreign worker authority"
            ));
        }
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
    validate_runtime_tool_execution_binding(&cache, intent_id)
}

pub(crate) fn validate_runtime_tool_execution_binding(
    cache: &CultCache,
    intent_id: &str,
) -> Result<EpiphanyRuntimeToolExecutionBinding> {
    validate_non_empty(intent_id, "tool execution intent id")?;
    let binding = cache
        .get::<EpiphanyRuntimeToolExecutionBinding>(intent_id)?
        .ok_or_else(|| anyhow!("tool execution intent {intent_id:?} is unbound"))?;
    if binding.binding_id != intent_id
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

pub(crate) fn validate_terminal_tool_execution_family(
    binding: &EpiphanyRuntimeToolExecutionBinding,
    intent: &EpiphanyToolInvocationIntent,
    receipt: &EpiphanyToolInvocationReceipt,
) -> Result<()> {
    if binding.binding_id != intent.intent_id
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

pub fn retain_completed_runtime_sessions(
    store_path: impl AsRef<Path>,
    retain_recent: usize,
) -> Result<()> {
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;

    let jobs = cache.get_all::<EpiphanyRuntimeJob>()?;
    let bindings = cache.get_all::<EpiphanyRuntimeModelExecutionBinding>()?;
    let mut candidates = cache
        .get_all::<EpiphanyRuntimeSession>()?
        .into_iter()
        .filter(|session| session.status == EpiphanyRuntimeSessionStatus::Completed)
        .filter(|session| {
            let session_jobs = jobs
                .iter()
                .filter(|job| job.session_id == session.session_id)
                .collect::<Vec<_>>();
            !session_jobs.is_empty()
                && session_jobs.iter().all(|job| {
                    job.role == "openai-model-adapter"
                        && bindings.iter().any(|binding| binding.job_id == job.job_id)
                })
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| right.session_id.cmp(&left.session_id))
    });

    for session in candidates.into_iter().skip(retain_recent.max(1)) {
        archive_completed_model_session(store_path, &session.session_id)?;
    }
    Ok(())
}

fn archive_completed_model_session(
    store_path: impl AsRef<Path>,
    session_id: &str,
) -> Result<EpiphanyArchivedRuntimeSession> {
    validate_non_empty(session_id, "archived runtime session id")?;
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    if let Some(existing) = cache.get::<EpiphanyArchivedRuntimeSession>(session_id)? {
        if existing.session_id != session_id
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
            || provider_request != epiphany_openai_adapter::request_from_native(&native_request)
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
        if native_terminals.len() != 1
            || native_events.last().map(|event| event.sequence)
                != native_terminals.first().map(|event| event.sequence)
        {
            return Err(anyhow!(
                "runtime session archive requires one terminal model stream event"
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
        if cache
            .get::<EpiphanyModelReceipt>(&binding.request_id)?
            .is_some()
        {
            retired_identities.insert((
                EpiphanyModelReceipt::TYPE.to_string(),
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
    let mut digest = Sha256::new();
    digest.update(b"epiphany-runtime-archived-session-root");
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
    let archive = EpiphanyArchivedRuntimeSession {
        session_id: session_id.to_string(),
        job_ids: job_ids.into_iter().collect(),
        model_request_ids: model_request_ids.into_iter().collect(),
        tool_intent_ids: tool_intent_ids.into_iter().collect(),
        retired_chain_digest: format!("sha256:{:x}", digest.finalize()),
    };
    let (replacement, _) = cache.prepare_entry(session_id, &archive)?;
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

pub fn prepare_runtime_spine_heartbeat_job(
    cache: &CultCache,
    options: RuntimeSpineHeartbeatJobOptions,
) -> Result<Vec<CultCacheEnvelope>> {
    validate_non_empty(&options.runtime_id, "runtime id")?;
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
    validate_frontier_planning_launch_carrier(
        &options.role,
        &options.binding_id,
        options.frontier_planning_request_id.as_deref(),
        &options.launch_document,
    )?;
    validate_frontier_plan_mind_launch_carrier(
        &options.role,
        &options.binding_id,
        options.frontier_plan_mind_request_id.as_deref(),
        &options.launch_document,
    )?;
    validate_frontier_research_launch_carrier(
        &options.role,
        &options.binding_id,
        options.repo_frontier_research_request_id.as_deref(),
        &options.launch_document,
    )?;
    validate_frontier_verification_launch_carrier(
        &options.role,
        &options.binding_id,
        options.repo_frontier_verification_request_id.as_deref(),
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
        &options.launch_document,
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
    validate_non_empty(&options.created_at, "created at")?;

    let identity = cache
        .get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("runtime job preparation requires runtime identity"))?;
    let identity_envelope = cache
        .get_envelope::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("runtime job preparation lost runtime identity envelope"))?;
    if identity.runtime_id != options.runtime_id {
        return Err(anyhow!(
            "runtime job preparation cannot substitute runtime identity"
        ));
    }
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
            session_id: options.session_id.clone(),
            objective: options.objective,
            status: EpiphanyRuntimeSessionStatus::Active,
            created_at: options.created_at.clone(),
            updated_at: options.created_at.clone(),
            coordinator_note: options.coordinator_note,
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
        job_id: options.job_id.clone(),
        session_id: options.session_id.clone(),
        role: options.role.clone(),
        status: EpiphanyRuntimeJobStatus::Queued,
        created_at: options.created_at.clone(),
        updated_at: options.created_at.clone(),
    };
    let request = EpiphanyRuntimeWorkerLaunchRequest {
        job_id: options.job_id.clone(),
        binding_id: options.binding_id,
        role: options.role,
        authority_scope: options.authority_scope,
        instruction: options.instruction,
        output_contract_id: options.output_contract_id,
        document_kind: worker_launch_document_kind(&options.launch_document).to_string(),
        launch_document_msgpack: encode_worker_launch_document(&options.launch_document)?,
        metadata: BTreeMap::new(),
        proposal_modeling_request_id: options.proposal_modeling_request_id,
        frontier_planning_request_id: options.frontier_planning_request_id,
        frontier_plan_mind_request_id: options.frontier_plan_mind_request_id,
        imagination_consideration_request_id: options.imagination_consideration_request_id,
        admitted_model_direction_consideration_request_id: options
            .admitted_model_direction_consideration_request_id,
        repo_frontier_modeling_request_id: options.repo_frontier_modeling_request_id,
        repo_frontier_research_request_id: options.repo_frontier_research_request_id,
        repo_frontier_verification_request_id: options.repo_frontier_verification_request_id,
    };
    let session_envelope =
        match cache.get_envelope::<EpiphanyRuntimeSession>(&session.session_id)? {
            Some(existing) => existing,
            None => cache.prepare_entry(&session.session_id, &session)?.0,
        };
    let envelopes = vec![
        identity_envelope,
        session_envelope,
        cache.prepare_entry(&job.job_id, &job)?.0,
        cache.prepare_entry(&request.job_id, &request)?.0,
    ];
    Ok(envelopes)
}

fn validate_repo_frontier_verdict_modeling_launch_authority(
    role: &str,
    request_id: Option<&str>,
    launch_document: &EpiphanyWorkerLaunchDocument,
) -> Result<()> {
    let authority = match launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => {
            document.frontier_verdict_modeling_context.as_ref()
        }
        EpiphanyWorkerLaunchDocument::Reorient(_) => None,
    };
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
            if authority.request.soul_verdict_receipt_id != authority.soul_verdict.receipt_id {
                return Err(anyhow!(
                    "verdict-bound Modeling authority does not carry its exact Soul verdict"
                ));
            }
            Ok(())
        }
    }
}

fn validate_repo_frontier_verdict_modeling_mutation(
    cache: &CultCache,
    launch_document: &EpiphanyWorkerLaunchDocument,
    proposal: &crate::EpiphanyRepoModelMutationProposal,
) -> Result<()> {
    let authority = match launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => document
            .frontier_verdict_modeling_context
            .as_ref()
            .ok_or_else(|| anyhow!("frontier verdict Modeling result lost its typed context"))?,
        EpiphanyWorkerLaunchDocument::Reorient(_) => {
            return Err(anyhow!(
                "frontier verdict Modeling authority cannot cross a reorientation launch"
            ));
        }
    };
    validate_repo_frontier_modeling_request(cache, &authority.request)?;
    let persisted_verdict = cache
        .get::<SoulVerdictReceipt>(&authority.soul_verdict.receipt_id)?
        .ok_or_else(|| anyhow!("frontier verdict Modeling context lost its Soul verdict"))?;
    let current_item = cache
        .get::<crate::EpiphanyRepoModelFrontierDocument>(&authority.frontier_item.id)?
        .ok_or_else(|| anyhow!("frontier verdict Modeling context lost its frontier item"))?
        .value()?;
    if persisted_verdict != authority.soul_verdict || current_item != authority.frontier_item {
        return Err(anyhow!(
            "frontier verdict Modeling context does not bind current typed authority"
        ));
    }
    let operations = proposal.operations()?;
    let [crate::EpiphanyRepoModelMutationOperation::PutFrontier { item }] = operations.as_slice()
    else {
        return Err(anyhow!(
            "frontier verdict Modeling result must revise exactly one frontier identity"
        ));
    };
    let expected_status = match authority.request.allowed_disposition {
        RepoFrontierVerdictDisposition::Resolved => crate::RepoFrontierStatus::Resolved,
        RepoFrontierVerdictDisposition::Blocked => crate::RepoFrontierStatus::Blocked,
    };
    let mut expected_evidence = authority.frontier_item.evidence_refs.clone();
    expected_evidence.push(authority.request.verification_request_id.clone());
    expected_evidence.push(authority.request.soul_verdict_receipt_id.clone());
    expected_evidence.sort();
    expected_evidence.dedup();
    if item.id != authority.frontier_item.id
        || item.migration_body != authority.frontier_item.migration_body
        || item.question != authority.frontier_item.question
        || item.target_claim_ids != authority.frontier_item.target_claim_ids
        || item.repository_scope != authority.frontier_item.repository_scope
        || item.recommended_next_organ != authority.frontier_item.recommended_next_organ
        || item.adopted_plan != authority.frontier_item.adopted_plan
        || item.dependency_item_ids != authority.frontier_item.dependency_item_ids
        || item.public_source_refs != authority.frontier_item.public_source_refs
        || item.created_at != authority.frontier_item.created_at
        || item.retired_at != authority.frontier_item.retired_at
        || item.superseded_by != authority.frontier_item.superseded_by
        || item.status != expected_status
        || item.evidence_refs != expected_evidence
        || item.gap.trim().is_empty()
        || item
            .updated_at
            .as_deref()
            .is_none_or(|value| chrono::DateTime::parse_from_rfc3339(value).is_err())
    {
        return Err(anyhow!(
            "frontier verdict Modeling result exceeded its exact routed mutation authority"
        ));
    }
    Ok(())
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

fn validate_frontier_plan_mind_launch_carrier(
    role: &str,
    binding_id: &str,
    request_id: Option<&str>,
    launch_document: &EpiphanyWorkerLaunchDocument,
) -> Result<()> {
    let projection = match launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => {
            document.frontier_plan_mind_context.as_ref()
        }
        EpiphanyWorkerLaunchDocument::Reorient(_) => None,
    };
    let Some(request_id) = request_id else {
        if projection.is_some() {
            return Err(anyhow!(
                "frontier plan Mind context requires its typed request id"
            ));
        }
        return Ok(());
    };
    validate_non_empty(request_id, "frontier plan Mind request id")?;
    if role != EPIPHANY_MIND_OWNER_ROLE || binding_id != EPIPHANY_MIND_ROLE_BINDING_ID {
        return Err(anyhow!(
            "frontier plan Mind request may only be transported by the Mind role launch"
        ));
    }
    let projection = projection
        .ok_or_else(|| anyhow!("frontier plan Mind request requires its typed context"))?;
    if projection.request.request_id != request_id {
        return Err(anyhow!("frontier plan Mind context/request mismatch"));
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

fn validate_frontier_verification_launch_carrier(
    role: &str,
    binding_id: &str,
    verification_request_id: Option<&str>,
    launch_document: &EpiphanyWorkerLaunchDocument,
) -> Result<()> {
    let projection = match launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => {
            document.frontier_verification_context.as_ref()
        }
        EpiphanyWorkerLaunchDocument::Reorient(_) => None,
    };
    let Some(request_id) = verification_request_id else {
        if projection.is_some() {
            return Err(anyhow!(
                "frontier Verification context requires its typed request id"
            ));
        }
        return Ok(());
    };
    validate_non_empty(request_id, "frontier Verification request id")?;
    if role != crate::EPIPHANY_VERIFICATION_OWNER_ROLE
        || binding_id != crate::EPIPHANY_VERIFICATION_ROLE_BINDING_ID
    {
        return Err(anyhow!(
            "frontier Verification request may only be transported by the Verification role launch"
        ));
    }
    let projection = projection
        .ok_or_else(|| anyhow!("frontier Verification request requires its typed context"))?;
    if projection.request.request_id != request_id
        || projection.schema_version != crate::REPO_FRONTIER_VERIFICATION_CONTEXT_SCHEMA_VERSION
        || projection.contract != crate::REPO_FRONTIER_VERIFICATION_CONTEXT_CONTRACT
    {
        return Err(anyhow!("frontier Verification context/request mismatch"));
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

pub(crate) fn frontier_research_request_for_launch(
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
    validate_repo_frontier_research_request(&request)?;
    if request.request_id != request_id || request.runtime_id != runtime.runtime_id {
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

pub(crate) fn frontier_verification_request_for_launch(
    cache: &CultCache,
    launch: &EpiphanyRuntimeWorkerLaunchRequest,
) -> Result<Option<RepoFrontierVerificationRequest>> {
    let document: EpiphanyWorkerLaunchDocument =
        rmp_serde::from_slice(&launch.launch_document_msgpack)?;
    validate_frontier_verification_launch_carrier(
        &launch.role,
        &launch.binding_id,
        launch.repo_frontier_verification_request_id.as_deref(),
        &document,
    )?;
    let Some(request_id) = launch.repo_frontier_verification_request_id.as_deref() else {
        return Ok(None);
    };
    let request = cache
        .get::<RepoFrontierVerificationRequest>(request_id)?
        .ok_or_else(|| anyhow!("frontier Verification launch lost its typed request"))?;
    validate_repo_frontier_verification_request_intrinsic(&request)?;
    let carried = match document {
        EpiphanyWorkerLaunchDocument::Role(document) => document.frontier_verification_context,
        EpiphanyWorkerLaunchDocument::Reorient(_) => None,
    }
    .ok_or_else(|| anyhow!("frontier Verification launch lost its typed context"))?;
    if carried.request != request
        || carried.route.route_id != request.route_id
        || carried.hands_authority.route_id != request.route_id
        || carried.hands_authority.hands_intent_id != request.hands_intent_id
        || carried.hands_authority.hands_review_id != request.hands_review_id
        || carried.hands_intent.intent_id != request.hands_intent_id
        || carried.hands_review.review_id != request.hands_review_id
        || carried.patch_receipt.receipt_id != request.hands_patch_receipt_id
        || carried.command_receipt.receipt_id != request.hands_command_receipt_id
        || carried.commit_receipt.receipt_id != request.hands_commit_receipt_id
    {
        return Err(anyhow!(
            "frontier Verification launch context diverges from its request and Hands receipts"
        ));
    }
    Ok(Some(request))
}

pub(crate) fn runtime_authenticated_public_source_lookups_for_worker(
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

fn terminal_worker_request_id(cache: &CultCache, worker_job_id: &str) -> Result<Option<String>> {
    let bindings = cache
        .get_all::<EpiphanyRuntimeModelExecutionBinding>()?
        .into_iter()
        .filter(|binding| binding.source_worker_job_id.as_deref() == Some(worker_job_id))
        .map(|binding| {
            let validated = validate_runtime_model_execution_binding(cache, &binding.request_id)?;
            let job = cache
                .get::<EpiphanyRuntimeJob>(&validated.job_id)?
                .ok_or_else(|| anyhow!("worker model execution lost its runtime job"))?;
            let request = cache
                .get::<EpiphanyModelRequest>(&validated.request_id)?
                .ok_or_else(|| anyhow!("worker model execution lost its native request"))?;
            Ok((validated, job, request))
        })
        .collect::<Result<Vec<_>>>()?;
    if bindings.is_empty() {
        return Ok(None);
    }

    let live = bindings
        .iter()
        .filter(|(_, job, _)| {
            matches!(
                job.status,
                EpiphanyRuntimeJobStatus::Queued
                    | EpiphanyRuntimeJobStatus::Running
                    | EpiphanyRuntimeJobStatus::WaitingForReview
            )
        })
        .collect::<Vec<_>>();
    if live.len() > 1 {
        return Err(anyhow!(
            "worker attempt has multiple live model request authorities"
        ));
    }
    if let Some((binding, _, _)) = live.first() {
        return Ok(Some(binding.request_id.clone()));
    }

    // Tool continuations are self-contained requests whose ordered input is a
    // strict extension of the request that preceded them. This finds the
    // unique terminal request from request bytes, without granting timestamps
    // or provider event order causal authority.
    let terminal = bindings
        .iter()
        .filter(|(_, _, candidate)| {
            bindings.iter().all(|(_, _, other)| {
                candidate.request_id == other.request_id
                    || (candidate.input.len() > other.input.len()
                        && candidate.input.starts_with(&other.input))
            })
        })
        .collect::<Vec<_>>();
    match terminal.as_slice() {
        [(binding, _, _)] => Ok(Some(binding.request_id.clone())),
        [] => Err(anyhow!(
            "worker model request family has no unique terminal self-contained request"
        )),
        _ => Err(anyhow!(
            "worker model request family has multiple terminal request authorities"
        )),
    }
}

fn complete_persisted_worker_outcome(
    store_path: &Path,
    job_id: &str,
    completed_at: &str,
) -> Result<Option<EpiphanyRuntimeJobResult>> {
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let role = cache.get::<EpiphanyRuntimeRoleWorkerResult>(job_id)?;
    let reorient = cache.get::<EpiphanyRuntimeReorientWorkerResult>(job_id)?;
    match (role, reorient) {
        (Some(_), Some(_)) => Err(anyhow!(
            "worker attempt has both role and reorientation terminal outcomes"
        )),
        (Some(result), None) => complete_runtime_job(
            store_path,
            RuntimeSpineJobResultOptions {
                result_id: result.result_id,
                job_id: job_id.to_string(),
                completed_at: completed_at.to_string(),
                verdict: result.verdict,
                summary: result.summary,
                next_safe_move: result.next_safe_move,
                evidence_refs: result.evidence_ids,
                artifact_refs: result.artifact_refs,
                decision_context_id: Some(result.decision_context_id),
            },
        )
        .map(Some),
        (None, Some(result)) => complete_runtime_job(
            store_path,
            RuntimeSpineJobResultOptions {
                result_id: result.result_id,
                job_id: job_id.to_string(),
                completed_at: completed_at.to_string(),
                verdict: result.mode,
                summary: result.summary,
                next_safe_move: result.next_safe_move,
                evidence_refs: result.evidence_ids,
                artifact_refs: result.artifact_refs,
                decision_context_id: Some(result.decision_context_id),
            },
        )
        .map(Some),
        (None, None) => Ok(None),
    }
}

fn commit_runtime_worker_process_death(
    store_path: impl AsRef<Path>,
    job_id: &str,
    terminal_authority_id: &str,
    terminal_at: &str,
    decision_context_id: Option<&str>,
) -> Result<EpiphanyRuntimeJobResult> {
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
        let result_id = format!("result-worker-death-{job_id}");
        let result = cache
            .get::<EpiphanyRuntimeJobResult>(&result_id)?
            .ok_or_else(|| anyhow!("worker death claim lost its terminal job result"))?;
        let job = cache
            .get::<EpiphanyRuntimeJob>(job_id)?
            .ok_or_else(|| anyhow!("worker death claim lost its runtime job"))?;
        if job.status != EpiphanyRuntimeJobStatus::Failed
            || result.job_id != job_id
            || result.decision_context_id.as_deref() != decision_context_id
        {
            return Err(anyhow!(
                "worker death replay found split terminal authority"
            ));
        }
        return Ok(result);
    }
    if !status.is_live() {
        return Err(anyhow!(
            "worker death recovery found terminal process authority"
        ));
    }
    if cache
        .get::<EpiphanyRuntimeRoleWorkerResult>(job_id)?
        .is_some()
        || cache
            .get::<EpiphanyRuntimeReorientWorkerResult>(job_id)?
            .is_some()
    {
        return Err(anyhow!("worker death recovery races a terminal outcome"));
    }
    let mut job = cache
        .get::<EpiphanyRuntimeJob>(job_id)?
        .ok_or_else(|| anyhow!("worker death recovery requires its runtime job"))?;
    if !matches!(
        job.status,
        EpiphanyRuntimeJobStatus::Queued
            | EpiphanyRuntimeJobStatus::Running
            | EpiphanyRuntimeJobStatus::WaitingForReview
    ) {
        return Err(anyhow!(
            "worker death recovery found a terminal runtime job"
        ));
    }
    let existing_results = cache
        .get_all::<EpiphanyRuntimeJobResult>()?
        .into_iter()
        .filter(|result| result.job_id == job_id)
        .collect::<Vec<_>>();
    if !existing_results.is_empty() {
        return Err(anyhow!("live worker death recovery found a job result"));
    }
    let job_envelope = cache
        .get_envelope::<EpiphanyRuntimeJob>(job_id)?
        .ok_or_else(|| anyhow!("worker death recovery lost its job envelope"))?;
    let current_envelope = cache
        .get_envelope::<EpiphanyRuntimeWorkerProcessClaim>(&claim_id)?
        .ok_or_else(|| anyhow!("worker death recovery lost its claim envelope"))?;
    let mut next = current;
    next.status = WorkerProcessStatus::TerminalDeath.as_str().into();
    next.terminal_at = Some(terminal_at.into());
    next.terminal_authority_id = Some(terminal_authority_id.into());
    let summary = "Runtime Continuity proved the exact worker process terminal before a structured outcome was admitted.".to_string();
    job.status = EpiphanyRuntimeJobStatus::Failed;
    job.updated_at = terminal_at.to_string();
    let result = EpiphanyRuntimeJobResult {
        result_id: format!("result-worker-death-{job_id}"),
        job_id: job_id.to_string(),
        session_id: job.session_id.clone(),
        role: job.role.clone(),
        verdict: "failed".to_string(),
        summary: summary.clone(),
        completed_at: terminal_at.to_string(),
        next_safe_move: "Derive a fresh attempt from current typed work; never rebase the abandoned model output.".to_string(),
        evidence_refs: Vec::new(),
        artifact_refs: Vec::new(),
        metadata: BTreeMap::new(),
        decision_context_id: decision_context_id.map(str::to_string),
    };
    if cache
        .get::<EpiphanyRuntimeJobResult>(&result.result_id)?
        .is_some()
    {
        return Err(anyhow!("worker death terminal identities already exist"));
    }
    let mut expected = vec![current_envelope, job_envelope];
    let mut unchanged_strong_reads = Vec::new();
    if let Some(context_id) = decision_context_id {
        let context = cache
            .get::<crate::EpiphanyDecisionContext>(context_id)?
            .ok_or_else(|| anyhow!("model-backed worker death lost its decision context"))?;
        if context.native_request()?.source_worker_job_id.as_deref() != Some(job_id) {
            return Err(anyhow!(
                "worker death decision context belongs to another worker"
            ));
        }
        let failures = cache
            .get_all::<crate::EpiphanyModelPassFailure>()?
            .into_iter()
            .filter(|failure| {
                failure.decision_context_id == context_id && failure.pass_id == job_id
            })
            .collect::<Vec<_>>();
        if failures.len() != 1 {
            return Err(anyhow!(
                "model-backed worker death requires one exact typed pass failure"
            ));
        }
        let context_envelope = cache
            .get_envelope::<crate::EpiphanyDecisionContext>(context_id)?
            .ok_or_else(|| anyhow!("worker death lost its context envelope"))?;
        let failure_envelope = cache
            .get_envelope::<crate::EpiphanyModelPassFailure>(&failures[0].failure_id)?
            .ok_or_else(|| anyhow!("worker death lost its pass-failure envelope"))?;
        expected.extend([context_envelope.clone(), failure_envelope.clone()]);
        unchanged_strong_reads.extend([context_envelope, failure_envelope]);
    } else if cache
        .get_all::<EpiphanyRuntimeModelExecutionBinding>()?
        .iter()
        .any(|binding| binding.source_worker_job_id.as_deref() == Some(job_id))
    {
        return Err(anyhow!(
            "model-backed worker death cannot terminalize without its exact context"
        ));
    }
    let mut writes = vec![
        cache.prepare_entry(&claim_id, &next)?.0,
        cache.prepare_entry(job_id, &job)?.0,
        cache.prepare_entry(&result.result_id, &result)?.0,
    ];
    writes.extend(unchanged_strong_reads);
    if !SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&expected, writes)?
    {
        return commit_runtime_worker_process_death(
            store_path,
            job_id,
            terminal_authority_id,
            terminal_at,
            decision_context_id,
        );
    }
    Ok(result)
}

pub(crate) fn terminalize_dead_runtime_worker_attempt(
    store_path: impl AsRef<Path>,
    job_id: &str,
    terminal_authority_id: &str,
    terminal_at: &str,
) -> Result<EpiphanyRuntimeJobResult> {
    let store_path = store_path.as_ref();
    if let Some(result) = complete_persisted_worker_outcome(store_path, job_id, terminal_at)? {
        return Ok(result);
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let terminal_request_id = terminal_worker_request_id(&cache, job_id)?;
    drop(cache);
    let decision_context_id = if let Some(request_id) = terminal_request_id {
        if let Some(failure) = model_pass_failure_for_request(store_path, &request_id)? {
            Some(failure.decision_context_id)
        } else {
            let context = crate::seal_model_decision_context(store_path, &request_id)?;
            let failure = terminalize_model_pass_failure_session(
                store_path,
                ModelPassFailureTerminalOptions {
                    decision_context_id: context.context_id,
                    failure_kind: "worker_process_death".to_string(),
                    summary: "Runtime Continuity proved the exact model worker process terminal before a structured outcome was admitted.".to_string(),
                    failed_at: terminal_at.to_string(),
                },
            )?;
            Some(failure.decision_context_id)
        }
    } else {
        None
    };
    commit_runtime_worker_process_death(
        store_path,
        job_id,
        terminal_authority_id,
        terminal_at,
        decision_context_id.as_deref(),
    )
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
    let is_modeling = result.role_id.eq_ignore_ascii_case("modeling");
    let modeling_binding_count = [
        result.repo_frontier_modeling_request_id.is_some(),
        result.proposal_modeling_request_id.is_some(),
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
            || result.proposal_modeling_request_id != worker_launch.proposal_modeling_request_id)
    {
        return Err(anyhow!(
            "Modeling result must exactly preserve its runtime-owned request authority"
        ));
    }
    let is_frontier_research = worker_launch.repo_frontier_research_request_id.is_some();
    if is_frontier_research
        != result
            .repo_frontier_research_request_id
            .as_ref()
            .is_some_and(|id| !id.trim().is_empty())
        || result.repo_frontier_research_request_id
            != worker_launch.repo_frontier_research_request_id
        || (is_frontier_research && !result.role_id.eq_ignore_ascii_case("research"))
        || (!is_frontier_research && result.repo_frontier_research_request_id.is_some())
    {
        return Err(anyhow!(
            "Research result must exactly preserve its runtime-owned frontier request"
        ));
    }
    if is_frontier_research {
        let request = frontier_research_request_for_launch(&cache, &worker_launch)?
            .ok_or_else(|| anyhow!("Research result lost its typed request"))?;
        if result.repo_frontier_research_request_id.as_deref() != Some(request.request_id.as_str())
        {
            return Err(anyhow!("Research result substituted its typed request"));
        }
        result
            .research_decision()?
            .ok_or_else(|| anyhow!("Research result requires its typed decision"))?
            .validate()?;
    } else if result.research_decision_msgpack.is_some() {
        return Err(anyhow!(
            "only an exact frontier Research result may carry a Research decision"
        ));
    }
    let is_frontier_verification = worker_launch
        .repo_frontier_verification_request_id
        .is_some();
    if is_verification != is_frontier_verification {
        return Err(anyhow!(
            "Verification result and launch must share one exact frontier request"
        ));
    }
    if is_frontier_verification {
        let request = frontier_verification_request_for_launch(&cache, &worker_launch)?
            .ok_or_else(|| anyhow!("Verification result lost its typed request"))?;
        if result.verification_request_id.as_deref() != Some(request.request_id.as_str())
            || result.frontier_route_id.as_deref() != Some(request.route_id.as_str())
        {
            return Err(anyhow!(
                "Verification result substituted its exact request or route"
            ));
        }
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
        if (worker_launch.repo_frontier_modeling_request_id.is_some()
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
        if worker_launch.repo_frontier_modeling_request_id.is_some() {
            validate_repo_frontier_verdict_modeling_mutation(&cache, &launch_document, &proposal)?;
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
        if result.research_decision_msgpack.is_some()
            || result.repo_model_mutation_proposal_msgpack.is_some()
            || result.verification_request_id.is_some()
            || result.frontier_route_id.is_some()
            || result.repo_frontier_modeling_request_id.is_some()
            || result.proposal_modeling_request_id.is_some()
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
        crate::current_work::frontier_planning_attempt_ordinal(
            &request.request_id,
            &result.job_id,
        )?;
        if worker_launch.job_id != result.job_id
            || worker_launch.role != EPIPHANY_IMAGINATION_OWNER_ROLE
            || worker_launch.binding_id != EPIPHANY_IMAGINATION_ROLE_BINDING_ID
            || worker_launch.frontier_planning_request_id.as_deref()
                != Some(request.request_id.as_str())
            || worker_launch.proposal_modeling_request_id.is_some()
            || launch_document.thread_id() != result.job_id
            || projection != Some(&expected_projection)
        {
            return Err(anyhow!(
                "frontier planning result does not exactly bind request, immutable launch, and candidate"
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
            || result.research_decision_msgpack.is_some()
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
            || result.research_decision_msgpack.is_some()
            || result.repo_model_mutation_proposal_msgpack.is_some()
            || result.verification_request_id.is_some()
            || result.frontier_route_id.is_some()
            || result.repo_frontier_modeling_request_id.is_some()
            || result.proposal_modeling_request_id.is_some()
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
        crate::current_work::consideration_attempt_ordinal(request_id, &result.job_id)?;
        if worker.job_id != result.job_id
            || worker.role != EPIPHANY_IMAGINATION_OWNER_ROLE
            || worker.binding_id != EPIPHANY_IMAGINATION_ROLE_BINDING_ID
            || worker.imagination_consideration_request_id.as_deref() != Some(request_id)
            || document.thread_id() != result.job_id
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
            || result.research_decision_msgpack.is_some()
            || result.repo_model_mutation_proposal_msgpack.is_some()
            || result.frontier_planning_request_id.is_some()
            || result.frontier_plan_candidate_msgpack.is_some()
            || result.verification_request_id.is_some()
            || result.frontier_route_id.is_some()
            || result.repo_frontier_modeling_request_id.is_some()
            || result.proposal_modeling_request_id.is_some()
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
        let launch = cache
            .get::<EpiphanyRuntimeWorkerLaunchRequest>(&result.job_id)?
            .ok_or_else(|| anyhow!("Mind result launch disappeared"))?;
        let document = launch.launch_document()?;
        let projection = match &document {
            EpiphanyWorkerLaunchDocument::Role(d) => d.frontier_plan_mind_context.as_ref(),
            _ => None,
        };
        let expected = RepoFrontierPlanMindContextProjection::new(&request, &planning, &candidate);
        crate::current_work::frontier_plan_mind_attempt_ordinal(
            &request.request_id,
            &result.job_id,
        )?;
        if launch.job_id != result.job_id
            || launch.role != EPIPHANY_MIND_OWNER_ROLE
            || launch.binding_id != EPIPHANY_MIND_ROLE_BINDING_ID
            || launch.frontier_plan_mind_request_id.as_deref() != Some(request.request_id.as_str())
            || document.thread_id() != result.job_id
            || projection != Some(&expected)
        {
            return Err(anyhow!(
                "Mind result does not exactly bind request, immutable launch, and candidate"
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
pub(crate) struct RuntimeTypedFulfillmentEvidence {
    pub job_id: String,
    pub result_id: String,
}

struct ValidatedProposalModelingWorkerFulfillment {
    proposal: RepoFrontierWorkProposal,
    mutation_operations: Vec<crate::EpiphanyRepoModelMutationOperation>,
}

fn validated_proposal_modeling_worker_fulfillment(
    cache: &CultCache,
    result: &EpiphanyRuntimeRoleWorkerResult,
) -> Result<ValidatedProposalModelingWorkerFulfillment> {
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
    validate_autonomous_proposal_origin_request(cache, &proposal)?;
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
        &proposal.constraints,
        &proposal.evidence_refs,
    )?;
    let attempt_ordinal =
        crate::current_work::proposal_modeling_attempt_ordinal(request_id, &result.job_id)?;
    let prior_admission_refusals = crate::current_work::proposal_modeling_prior_admission_refusals(
        cache,
        request_id,
        attempt_ordinal,
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
            title: proposal.title.clone(),
            body: proposal.body.clone(),
            constraints: proposal.constraints.clone(),
            evidence_refs: proposal.evidence_refs.clone(),
            model_projection_digest: projection.model_projection_digest.clone(),
            model_source_documents: projection.model_source_documents.clone(),
            prior_admission_refusals: prior_admission_refusals.clone(),
        };
        projection == &expected
    });
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
        ("launch.job", launch.job_id != result.job_id),
        (
            "launch.binding",
            launch.binding_id != EPIPHANY_MODELING_ROLE_BINDING_ID,
        ),
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
    Ok(ValidatedProposalModelingWorkerFulfillment {
        proposal,
        mutation_operations,
    })
}

pub(crate) fn validate_proposal_modeling_worker_fulfillment(
    cache: &CultCache,
    result: &EpiphanyRuntimeRoleWorkerResult,
) -> Result<()> {
    validated_proposal_modeling_worker_fulfillment(cache, result)?;
    Ok(())
}

pub(crate) fn validate_proposal_modeling_worker_admission(
    cache: &CultCache,
    result: &EpiphanyRuntimeRoleWorkerResult,
) -> Result<()> {
    let validated = validated_proposal_modeling_worker_fulfillment(cache, result)?;
    let proposal = validated.proposal;
    let mutation_operations = validated.mutation_operations;
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
        || !crate::memory_graph::frontier_item_has_routeable_repository_scope(upserts[0])
        || !upserts[0].public_source_refs.is_empty()
        || upserts[0].status != crate::RepoFrontierStatus::Active
        || !matches!(
            upserts[0].recommended_next_organ.as_str(),
            "Eyes" | "Imagination"
        )
        || upserts[0].adopted_plan.is_some()
    {
        return Err(anyhow!(
            "proposal Modeling fulfillment result is not one safe proposal-citing routeable frontier"
        ));
    }
    Ok(())
}

pub(crate) fn runtime_typed_request_fulfillment(
    store_path: impl AsRef<Path>,
    request: RuntimeTypedRequestRef<'_>,
) -> Result<Option<RuntimeTypedFulfillmentEvidence>> {
    let store_path = store_path.as_ref();
    let request_id = request.request_id();
    validate_non_empty(request_id, "typed fulfillment request id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let admission_refusals = cache
        .get_all::<crate::EpiphanyAgentPassAdmissionRefusal>()?
        .into_iter()
        .map(|refusal| {
            refusal.validate()?;
            Ok(refusal)
        })
        .collect::<Result<Vec<_>>>()?;
    let archived_matches = cache
        .get_all::<EpiphanyArchivedRuntimeWorkerAttempt>()?
        .into_iter()
        .filter(|attempt| {
            attempt.request_id == request_id
                && attempt.fulfilled_result_id().is_some()
                && !admission_refusals.iter().any(|refusal| {
                    refusal.pass_family.request_kind() == request.kind()
                        && refusal.request_id == request_id
                        && refusal.job_id == attempt.job_id
                        && attempt.fulfilled_result_id() == Some(refusal.result_id.as_str())
                        && attempt.decision_context_id()
                            == Some(refusal.decision_context_id.as_str())
                })
        })
        .collect::<Vec<_>>();
    if archived_matches.len() > 1 {
        return Err(anyhow!(
            "typed fulfillment request has multiple archived terminal claimants"
        ));
    }
    if let Some(attempt) = archived_matches.first() {
        if attempt.request_kind != request.kind()
            || !crate::WorkerProcessStatus::parse(&attempt.terminal_process_status)?
                .is_fulfilled_terminal()
            || !attempt.retired_chain_digest.starts_with("sha256:")
        {
            return Err(anyhow!("archived typed fulfillment tombstone is invalid"));
        }
        attempt.validate_decision_record(true)?;
        let archived_role_result = attempt
            .decision
            .as_ref()
            .and_then(|decision| decision.role_result.as_ref())
            .ok_or_else(|| anyhow!("archived typed fulfillment lost its structured result"))?;
        if !request.matches_result(archived_role_result) {
            return Err(anyhow!(
                "archived typed fulfillment substituted its structured result"
            ));
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
                .fulfilled_result_id()
                .map(str::to_string)
                .expect("validated archived result"),
        }));
    }
    let matches = cache
        .get_all::<EpiphanyRuntimeRoleWorkerResult>()?
        .into_iter()
        .filter(|result| {
            request.matches_result(result)
                && !admission_refusals.iter().any(|refusal| {
                    refusal.pass_family.request_kind() == request.kind()
                        && refusal.request_id == request_id
                        && refusal.job_id == result.job_id
                        && refusal.result_id == result.result_id
                        && refusal.decision_context_id == result.decision_context_id
                })
        })
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
        RuntimeTypedRequestRef::FrontierVerdictModeling(_) => {}
        RuntimeTypedRequestRef::FrontierResearch(_) => {}
        RuntimeTypedRequestRef::FrontierVerification(_) => {}
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
    }))
}

pub(crate) fn validate_repo_frontier_work_proposal(
    proposal: &RepoFrontierWorkProposal,
) -> Result<()> {
    if proposal.proposal_id.trim().is_empty()
        || proposal.title.trim().is_empty()
        || proposal.body.trim().is_empty()
    {
        return Err(anyhow!("invalid inert repo frontier work proposal"));
    }
    let expected_payload_sha256 = crate::repo_frontier_proposal_payload_sha256(
        &proposal.title,
        &proposal.body,
        &proposal.constraints,
        &proposal.evidence_refs,
    )?;
    if proposal.payload_sha256 != expected_payload_sha256 {
        return Err(anyhow!("proposal content hash mismatch"));
    }
    Ok(())
}

fn validate_autonomous_proposal_origin_request(
    cache: &CultCache,
    proposal: &RepoFrontierWorkProposal,
) -> Result<RepoFrontierProposalModelingRequest> {
    let identity = require_identity(cache)?;
    let modeling_request_id = crate::proposal_modeling_request_id(
        &identity.runtime_id,
        &proposal.proposal_id,
        &proposal.payload_sha256,
    );
    let modeling_request = cache
        .get::<RepoFrontierProposalModelingRequest>(&modeling_request_id)?
        .ok_or_else(|| anyhow!("Imagination proposal lacks its Modeling request"))?;
    validate_repo_frontier_proposal_modeling_request(&modeling_request)?;
    let result = cache
        .get::<crate::AdmittedModelDirectionConsiderationResult>(
            &modeling_request.direction_result_id,
        )?
        .ok_or_else(|| anyhow!("autonomous proposal request lost its direction result"))?;
    let request = cache
        .get::<crate::AdmittedModelDirectionConsiderationRequest>(&result.request_id)?
        .ok_or_else(|| anyhow!("autonomous proposal request lost its direction request"))?;
    let worker_result = cache
        .get::<EpiphanyRuntimeRoleWorkerResult>(&modeling_request.direction_worker_job_id)?
        .ok_or_else(|| anyhow!("autonomous proposal request lost its Imagination worker result"))?;
    let worker_launch = cache
        .get::<EpiphanyRuntimeWorkerLaunchRequest>(&modeling_request.direction_worker_job_id)?
        .ok_or_else(|| anyhow!("autonomous proposal request lost its Imagination worker launch"))?;
    crate::validate_admitted_model_direction_consideration_request(&request)?;
    crate::validate_admitted_model_direction_consideration_result(&request, &result)?;
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
        .get(modeling_request.direction_option_ordinal as usize)
        .ok_or_else(|| anyhow!("autonomous proposal request names a missing option"))?;
    let route = cache
        .get::<crate::RuntimeRepositoryBodyStoreBinding>(crate::RUNTIME_BODY_STORE_BINDING_KEY)?
        .ok_or_else(|| anyhow!("autonomous proposal requires repository Body binding"))?;
    let domain = cache
        .get::<RuntimeRepositoryDomainBinding>(RUNTIME_REPOSITORY_DOMAIN_BINDING_KEY)?
        .ok_or_else(|| anyhow!("autonomous proposal requires repository domain binding"))?;
    let chain_checks = [
        (
            "worker job",
            worker_result.job_id == modeling_request.direction_worker_job_id,
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
                    &modeling_request.direction_worker_job_id,
                ),
        ),
        (
            "launch role",
            worker_launch
                .role
                .eq_ignore_ascii_case(EPIPHANY_IMAGINATION_OWNER_ROLE),
        ),
        (
            "launch job",
            worker_launch.job_id == modeling_request.direction_worker_job_id,
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
    if modeling_request.direction_result_id != result.result_id
        || modeling_request.proposal_id != proposal.proposal_id
        || modeling_request.proposal_payload_sha256 != proposal.payload_sha256
        || modeling_request.runtime_id != request.runtime_id
        || modeling_request.thread_id != request.thread_id
        || domain.repository_full_name != modeling_request.repository
        || domain.body_binding_sha256 != route.body_binding_sha256
        || proposal.title != option.title
        || proposal.body != option.summary
    {
        return Err(anyhow!("autonomous proposal origin binding mismatch"));
    }
    Ok(modeling_request)
}

pub(crate) fn validate_autonomous_proposal_origin(
    cache: &CultCache,
    proposal: &RepoFrontierWorkProposal,
) -> Result<()> {
    let modeling_request = validate_autonomous_proposal_origin_request(cache, proposal)?;
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
        || modeling_request.workspace != body_binding.git_top_level
    {
        return Err(anyhow!("autonomous proposal Body binding mismatch"));
    }
    Ok(())
}

pub(crate) fn validate_repo_frontier_proposal_modeling_request(
    request: &RepoFrontierProposalModelingRequest,
) -> Result<()> {
    if request.request_id.trim().is_empty()
        || request.proposal_id.trim().is_empty()
        || request.proposal_payload_sha256.trim().is_empty()
        || request.runtime_id.trim().is_empty()
        || request.thread_id.trim().is_empty()
        || request.repository.trim().is_empty()
        || request.workspace.trim().is_empty()
        || request.direction_result_id.trim().is_empty()
        || request.direction_worker_job_id.trim().is_empty()
        || request.request_id
            != crate::proposal_modeling_request_id(
                &request.runtime_id,
                &request.proposal_id,
                &request.proposal_payload_sha256,
            )
        || chrono::DateTime::parse_from_rfc3339(&request.selected_at).is_err()
    {
        return Err(anyhow!(
            "invalid coordinator repo frontier proposal Modeling request"
        ));
    }
    Ok(())
}

pub fn bind_runtime_repository_domain(
    runtime_store: impl AsRef<Path>,
    repository_full_name: &str,
) -> Result<RuntimeRepositoryDomainBinding> {
    if !repository_full_name.starts_with("GameCult/")
        || repository_full_name["GameCult/".len()..].trim().is_empty()
        || repository_full_name.chars().any(char::is_whitespace)
    {
        return Err(anyhow!(
            "runtime repository domain requires a canonical GameCult name"
        ));
    }
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
        repository_full_name: repository_full_name.into(),
        body_binding_sha256: route.body_binding_sha256.clone(),
    };
    if let Some(existing) =
        cache.get::<RuntimeRepositoryDomainBinding>(RUNTIME_REPOSITORY_DOMAIN_BINDING_KEY)?
    {
        return if binding == existing {
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

pub(crate) fn promote_autonomous_direction_options_for_modeling(
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
        || domain.repository_full_name != repository
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
                &result.uncertainties,
                &evidence_refs,
            )?;
            let proposal = RepoFrontierWorkProposal {
                proposal_id: proposal_id.clone(),
                payload_sha256: payload_sha256.clone(),
                title: option.title.clone(),
                body: option.summary.clone(),
                constraints: result.uncertainties.clone(),
                evidence_refs,
            };
            validate_repo_frontier_work_proposal(&proposal)?;
            let selection = RepoFrontierProposalModelingRequest {
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
                direction_result_id: result.result_id.clone(),
                direction_option_ordinal: ordinal as u32,
                direction_worker_job_id: direction_worker.job_id.clone(),
            };
            validate_repo_frontier_proposal_modeling_request(&selection)?;
            let mut current = runtime_spine_cache(runtime_store)?;
            current.pull_all_backing_stores()?;
            let existing = (
                current.get::<RepoFrontierWorkProposal>(&proposal.proposal_id)?,
                current.get::<RepoFrontierProposalModelingRequest>(&selection.request_id)?,
            );
            let replay_selection_matches = existing.1.as_ref().is_some_and(|existing_selection| {
                let mut replay_selection = selection.clone();
                replay_selection.selected_at = existing_selection.selected_at.clone();
                existing_selection == &replay_selection
            });
            if let (Some(existing_proposal), Some(existing_selection)) = &existing {
                validate_autonomous_proposal_origin(&current, existing_proposal)?;
                if existing_proposal == &proposal && replay_selection_matches {
                    promoted.push(existing_selection.clone());
                    continue;
                }
            }
            if existing.0.is_some() || existing.1.is_some() {
                return Err(anyhow!(
                    "autonomous proposal promotion companion collision for {}: proposalPresent={} selectionPresent={} proposalMatches={} selectionMatches={}",
                    proposal.proposal_id,
                    existing.0.is_some(),
                    existing.1.is_some(),
                    existing.0.as_ref() == Some(&proposal),
                    replay_selection_matches,
                ));
            }
            let (proposal_envelope, _) = current.prepare_entry(&proposal.proposal_id, &proposal)?;
            let (selection_envelope, _) =
                current.prepare_entry(&selection.request_id, &selection)?;
            let mut expected = vec![
                current
                    .get_envelope::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
                    .ok_or_else(|| anyhow!("runtime identity envelope disappeared"))?,
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
            replacement.extend([proposal_envelope, selection_envelope]);
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
    let backing = SingleFileMessagePackBackingStore::new(runtime_store);
    let (view, basis) = current_keyed_repo_model(&cache)?;
    let model = view.memory_context_projection();
    let item = actionable_imagination_frontier_item(&model)
        .ok_or_else(|| anyhow!("planning requires an actionable Imagination frontier"))?;
    let item_hash = repo_frontier_item_hash(item)?;
    let frontier_authority_documents = frontier_authority_documents(&cache, item)?;
    let claim_obligation_documents = planning_claim_obligation_documents(&cache, item)?;
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
        request_id: request_id.clone(),
        model_projection_digest: basis.projection_digest.clone(),
        model_source_documents: basis.source_documents.clone(),
        frontier_item_id: item.id.clone(),
        frontier_item_hash: item_hash,
        selected_organ: "Imagination".into(),
        repository_scope: item.repository_scope.clone(),
        requested_at: at.into(),
        runtime_id: identity.runtime_id,
        frontier_authority_documents,
        claim_obligation_documents,
    };
    let (envelope, _) = cache.prepare_entry(&request_id, &request)?;
    let mut expected = frontier_authority_envelopes(&cache, &request.frontier_authority_documents)?;
    expected.extend(planning_claim_obligation_envelopes(
        &cache,
        &request.claim_obligation_documents,
    )?);
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
        return Err(anyhow!("planning request lost exact frontier CAS"));
    }
    Ok(request)
}

fn validate_repo_frontier_planning_request(
    request: &RepoFrontierPlanningRequest,
) -> Result<crate::RepoFrontierItem> {
    chrono::DateTime::parse_from_rfc3339(&request.requested_at)
        .map_err(|_| anyhow!("planning request timestamp must be RFC3339"))?;
    let basis = crate::EpiphanyRepoModelBasis {
        projection_digest: request.model_projection_digest.clone(),
        source_documents: request.model_source_documents.clone(),
    };
    basis.validate()?;
    if request.runtime_id.trim().is_empty()
        || request.frontier_item_id.trim().is_empty()
        || request.frontier_item_hash.trim().is_empty()
        || request.selected_organ != "Imagination"
        || request.repository_scope.is_empty()
        || !crate::memory_graph::repo_paths_are_canonical_and_safe(&request.repository_scope)
        || request.frontier_authority_documents.is_empty()
        || request.frontier_authority_documents.iter().any(|document| {
            document.validate().is_err()
                || document.store_id != "epiphany-mind"
                || document.document_type != crate::EpiphanyRepoModelFrontierDocument::TYPE
                || !request.model_source_documents.contains(document)
        })
        || !request.frontier_authority_documents.windows(2).all(|pair| {
            (
                pair[0].document_type.as_str(),
                pair[0].document_key.as_str(),
            ) < (
                pair[1].document_type.as_str(),
                pair[1].document_key.as_str(),
            )
        })
        || request.claim_obligation_documents.iter().any(|document| {
            document.validate().is_err()
                || document.store_id != "epiphany-mind"
                || document.document_type != crate::EpiphanyRepoModelClaimObligationsDocument::TYPE
                || !request.model_source_documents.contains(document)
        })
        || !request.claim_obligation_documents.windows(2).all(|pair| {
            (
                pair[0].document_type.as_str(),
                pair[0].document_key.as_str(),
            ) < (
                pair[1].document_type.as_str(),
                pair[1].document_key.as_str(),
            )
        })
    {
        return Err(anyhow!("invalid frontier planning request"));
    }
    let frontier_source = request
        .frontier_authority_documents
        .iter()
        .find(|document| document.document_key == request.frontier_item_id)
        .ok_or_else(|| anyhow!("planning request lost its owning frontier document"))?;
    let item = rmp_serde::from_slice::<crate::EpiphanyRepoModelFrontierDocument>(
        &frontier_source.payload_msgpack,
    )?
    .value()?;
    let mut expected_authority_ids = vec![item.id.as_str()];
    expected_authority_ids.extend(item.dependency_item_ids.iter().map(String::as_str));
    expected_authority_ids.sort_unstable();
    expected_authority_ids.dedup();
    let expected_request_id = crate::frontier_planning_request_id(
        &request.runtime_id,
        &item.id,
        &repo_frontier_item_hash(&item)?,
    );
    let mut expected_claim_ids = item
        .target_claim_ids
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    expected_claim_ids.sort_unstable();
    expected_claim_ids.dedup();
    if request
        .frontier_authority_documents
        .iter()
        .map(|document| document.document_key.as_str())
        .collect::<Vec<_>>()
        != expected_authority_ids
        || request
            .claim_obligation_documents
            .iter()
            .map(|document| document.document_key.as_str())
            .collect::<Vec<_>>()
            != expected_claim_ids
        || request.request_id != expected_request_id
        || request.frontier_item_id != item.id
        || request.frontier_item_hash != repo_frontier_item_hash(&item)?
        || request.repository_scope != item.repository_scope
    {
        return Err(anyhow!(
            "frontier planning request diverges from its sealed frontier authority"
        ));
    }
    Ok(item)
}

pub(crate) fn validate_actionable_repo_frontier_planning_request(
    cache: &CultCache,
    request: &RepoFrontierPlanningRequest,
) -> Result<()> {
    let sealed_item = validate_repo_frontier_planning_request(request)?;
    let identity = require_identity(cache)?;
    if request.runtime_id != identity.runtime_id {
        return Err(anyhow!("planning request belongs to another runtime"));
    }
    frontier_authority_envelopes(cache, &request.frontier_authority_documents)?;
    planning_claim_obligation_envelopes(cache, &request.claim_obligation_documents)?;
    let current_item = cache
        .get::<crate::EpiphanyRepoModelFrontierDocument>(&request.frontier_item_id)?
        .ok_or_else(|| anyhow!("planning request frontier disappeared"))?
        .value()?;
    if current_item != sealed_item
        || frontier_authority_documents(cache, &current_item)?
            != request.frontier_authority_documents
        || planning_claim_obligation_documents(cache, &current_item)?
            != request.claim_obligation_documents
    {
        return Err(anyhow!("planning request frontier authority changed"));
    }
    let view = crate::repo_model_documents::assemble_repo_model_view_from_cache(cache)?;
    let model = view.memory_context_projection();
    if !imagination_frontier_item_is_actionable(&model, &current_item) {
        return Err(anyhow!("planning request frontier is no longer actionable"));
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
        request_id: request_id.clone(),
        planning_request_id: planning.request_id.clone(),
        imagination_result_id: result.result_id.clone(),
        imagination_job_id: result.job_id.clone(),
        candidate_id: candidate.candidate_id,
        candidate_sha256,
        runtime_id: planning.runtime_id.clone(),
        requested_at: requested_at.into(),
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
    let mut expected =
        frontier_authority_envelopes(&cache, &planning.frontier_authority_documents)?;
    expected.extend(planning_claim_obligation_envelopes(
        &cache,
        &planning.claim_obligation_documents,
    )?);
    expected.push(
        cache
            .get_envelope::<RepoFrontierPlanningRequest>(&planning.request_id)?
            .ok_or_else(|| anyhow!("Mind request lost its Planning request envelope"))?,
    );
    expected.push(
        cache
            .get_envelope::<EpiphanyRuntimeRoleWorkerResult>(&result.job_id)?
            .ok_or_else(|| anyhow!("Mind request lost its Imagination result envelope"))?,
    );
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
    if chrono::DateTime::parse_from_rfc3339(&request.requested_at).is_err() {
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
    {
        return Err(anyhow!(
            "Mind request substituted immutable Imagination causal identity"
        ));
    }
    Ok((planning, candidate))
}

fn actionable_imagination_frontier_item(
    model: &crate::EpiphanyMemoryGraphSnapshot,
) -> Option<&crate::RepoFrontierItem> {
    let mut eligible = model
        .frontier
        .iter()
        .filter(|item| imagination_frontier_item_is_actionable(model, item))
        .collect::<Vec<_>>();
    eligible.sort_by(|a, b| a.id.cmp(&b.id));
    eligible.into_iter().next()
}

fn imagination_frontier_item_is_actionable(
    model: &crate::EpiphanyMemoryGraphSnapshot,
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
        && crate::memory_graph::frontier_item_has_routeable_repository_scope(item)
        && item.dependency_item_ids.iter().all(|id| terminal(id))
}

fn validate_repo_frontier_plan_candidate_against_request(
    cache: &CultCache,
    candidate: &RepoFrontierPlanCandidate,
    request: &RepoFrontierPlanningRequest,
) -> Result<()> {
    if candidate.selected_fields_invalid()
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
    let item = validate_repo_frontier_planning_request(request)?;
    frontier_authority_envelopes(cache, &request.frontier_authority_documents)?;
    if repo_frontier_item_hash(&item)? != request.frontier_item_hash
        || item.repository_scope != request.repository_scope
        || !candidate.safe_paths.iter().all(|path| {
            request.repository_scope.iter().any(|scope| {
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
}

fn commit_repo_frontier_plan_decision_inner(
    runtime_store: impl AsRef<Path>,
    source: FrontierPlanDecisionSource<'_>,
    pre_cas: Option<&(dyn Fn() + Sync)>,
) -> Result<RepoFrontierPlanDecisionReceipt> {
    let runtime_store = runtime_store.as_ref();
    let mut cache = runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    let (mind_request_id, decision, rationale, decided_at, decision_source, decision_context_id) =
        match source {
            FrontierPlanDecisionSource::MindWorker(result_id) => {
                let result = cache
                    .get_all::<EpiphanyRuntimeRoleWorkerResult>()?
                    .into_iter()
                    .find(|result| result.result_id == result_id)
                    .ok_or_else(|| anyhow!("frontier plan decision lost its Mind result"))?;
                let typed = result.frontier_plan_mind_decision()?.ok_or_else(|| {
                    anyhow!("frontier plan decision requires a typed Mind decision")
                })?;
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
                )
            }
        };
    chrono::DateTime::parse_from_rfc3339(&decided_at)
        .map_err(|_| anyhow!("frontier plan decision time must be RFC3339"))?;
    let mind_request = cache
        .get::<RepoFrontierPlanMindRequest>(&mind_request_id)?
        .ok_or_else(|| anyhow!("frontier plan decision requires its typed Mind request"))?;
    let (planning, candidate) =
        validate_repo_frontier_plan_mind_request_identity(&cache, &mind_request)?;
    let candidate_sha256 = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&candidate)?));
    let decision_id = format!(
        "repo-frontier-plan-decision-{:x}",
        Sha256::digest(planning.request_id.as_bytes())
    );
    let receipt = RepoFrontierPlanDecisionReceipt {
        decision_id: decision_id.clone(),
        planning_request_id: planning.request_id.clone(),
        candidate_id: candidate.candidate_id.clone(),
        candidate_sha256,
        model_projection_digest: planning.model_projection_digest.clone(),
        model_source_documents: planning.model_source_documents.clone(),
        frontier_item_id: planning.frontier_item_id.clone(),
        frontier_item_hash: planning.frontier_item_hash.clone(),
        decision,
        rationale,
        decided_at: decided_at.clone(),
        decision_source: decision_source.clone(),
    };
    if let Some(existing) = cache.get::<RepoFrontierPlanDecisionReceipt>(&decision_id)? {
        if existing != receipt {
            return Err(anyhow!("frontier plan decision identity collision"));
        }
        if existing.decision == RepoFrontierPlanDecision::Adopt {
            let item = cache
                .get::<crate::EpiphanyRepoModelFrontierDocument>(&planning.frontier_item_id)?
                .ok_or_else(|| anyhow!("replayed plan decision lost its frontier"))?
                .value()?;
            let adopted = item
                .adopted_plan
                .as_ref()
                .ok_or_else(|| anyhow!("replayed plan decision lost its adoption"))?;
            if adopted.planning_request_id != planning.request_id
                || adopted.result_id != mind_request.imagination_result_id
                || adopted.candidate_id != candidate.candidate_id
                || adopted.candidate_sha256 != mind_request.candidate_sha256
            {
                return Err(anyhow!("replayed plan decision adoption was substituted"));
            }
        }
        return Ok(existing);
    }

    validate_actionable_repo_frontier_planning_request(&cache, &planning)?;
    validate_repo_frontier_plan_candidate_against_request(&cache, &candidate, &planning)?;

    let mut strong_reads =
        frontier_authority_envelopes(&cache, &planning.frontier_authority_documents)?;
    strong_reads.extend(planning_claim_obligation_envelopes(
        &cache,
        &planning.claim_obligation_documents,
    )?);
    for envelope in [
        cache.get_envelope::<RepoFrontierPlanningRequest>(&planning.request_id)?,
        cache.get_envelope::<RepoFrontierPlanMindRequest>(&mind_request.request_id)?,
        cache.get_envelope::<EpiphanyRuntimeRoleWorkerResult>(&mind_request.imagination_job_id)?,
    ]
    .into_iter()
    .flatten()
    {
        strong_reads.push(envelope);
    }
    let crate::RepoFrontierPlanDecisionSource::MindWorker { job_id, .. } = &decision_source;
    strong_reads.push(
        cache
            .get_envelope::<EpiphanyRuntimeRoleWorkerResult>(job_id)?
            .ok_or_else(|| anyhow!("frontier plan decision lost its Mind result envelope"))?,
    );
    let mut writes = Vec::new();
    if decision == RepoFrontierPlanDecision::Adopt {
        let item = cache
            .get::<crate::EpiphanyRepoModelFrontierDocument>(&planning.frontier_item_id)?
            .ok_or_else(|| anyhow!("frontier plan decision target disappeared"))?;
        let mut item = item.value()?;
        if repo_frontier_item_hash(&item)? != planning.frontier_item_hash {
            return Err(anyhow!("frontier plan decision target changed"));
        }
        item.adopted_plan = Some(crate::RepoFrontierAdoptedPlan {
            planning_request_id: planning.request_id.clone(),
            result_id: mind_request.imagination_result_id.clone(),
            job_id: mind_request.imagination_job_id.clone(),
            candidate_id: candidate.candidate_id.clone(),
            candidate_sha256: mind_request.candidate_sha256.clone(),
            safe_paths: candidate.safe_paths.clone(),
            action: candidate.action.clone(),
            command: candidate.command.clone(),
            checks: candidate.checks.clone(),
            stop_conditions: candidate.stop_conditions.clone(),
            rollback_steps: candidate.rollback_steps.clone(),
            commit_message: candidate.commit_message.clone(),
        });
        writes.push(
            cache
                .prepare_entry(
                    &planning.frontier_item_id,
                    &crate::EpiphanyRepoModelFrontierDocument::new(&item)?,
                )?
                .0,
        );
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
    let context_id = decision_context_id
        .ok_or_else(|| anyhow!("Mind plan decision lacks its decision context"))?;
    let outcome = crate::commit_mind_mutation(
        runtime_store,
        &context_id,
        "Mind.repo_frontier_plan_decision",
        strong_reads,
        writes,
        &decided_at,
    )?;
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
            || !crate::memory_graph::repo_paths_are_canonical_and_safe(&self.safe_paths)
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
    let item = actionable_hands_frontier_item(&current)
        .ok_or_else(|| anyhow!("current repo model has no eligible Hands frontier route"))?;
    if !crate::memory_graph::frontier_item_has_routeable_repository_scope(item) {
        return Err(anyhow!(
            "Hands frontier route requires canonical repository scope"
        ));
    }
    let item_hash = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&item)?));
    let route_seed = format!("{}:{}:{}", basis.projection_digest, item.id, item_hash);
    let route_id = format!(
        "repo-frontier-route-{:x}",
        Sha256::digest(route_seed.as_bytes())
    );
    let route = RepoFrontierRoute {
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
        authorized_paths: item
            .adopted_plan
            .as_ref()
            .map(|plan| plan.safe_paths.clone())
            .unwrap_or_else(|| item.repository_scope.clone()),
        adopted_plan: item.adopted_plan.clone(),
        selected_at: at.to_string(),
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

fn actionable_hands_frontier_item(
    model: &crate::EpiphanyMemoryGraphSnapshot,
) -> Option<&crate::RepoFrontierItem> {
    actionable_frontier_item_for_organ(model, "Hands")
}

fn actionable_frontier_item_for_organ<'a>(
    model: &'a crate::EpiphanyMemoryGraphSnapshot,
    organ: &str,
) -> Option<&'a crate::RepoFrontierItem> {
    model
        .frontier
        .iter()
        .find(|item| frontier_item_is_actionable_for_organ(model, item, organ))
}

fn frontier_item_is_actionable_for_organ(
    model: &crate::EpiphanyMemoryGraphSnapshot,
    item: &crate::RepoFrontierItem,
    organ: &str,
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
        && crate::memory_graph::frontier_item_has_routeable_repository_scope(item)
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
pub(crate) fn runtime_has_actionable_hands_frontier(
    runtime_store: impl AsRef<Path>,
) -> Result<bool> {
    runtime_has_actionable_frontier_for_organ(runtime_store, "Hands")
}

fn frontier_authority_documents(
    cache: &CultCache,
    item: &crate::RepoFrontierItem,
) -> Result<Vec<crate::EpiphanyMindDocumentVersion>> {
    let mut ids = vec![item.id.as_str()];
    ids.extend(item.dependency_item_ids.iter().map(String::as_str));
    ids.sort_unstable();
    ids.dedup();
    let mut documents = ids
        .into_iter()
        .map(|id| {
            let envelope = cache
                .get_envelope::<crate::EpiphanyRepoModelFrontierDocument>(id)?
                .ok_or_else(|| anyhow!("frontier authority document {id:?} is absent"))?;
            crate::EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", &envelope)
        })
        .collect::<Result<Vec<_>>>()?;
    documents.sort_by(|left, right| {
        left.document_type
            .cmp(&right.document_type)
            .then(left.document_key.cmp(&right.document_key))
    });
    Ok(documents)
}

fn frontier_authority_envelopes(
    cache: &CultCache,
    documents: &[crate::EpiphanyMindDocumentVersion],
) -> Result<Vec<CultCacheEnvelope>> {
    let snapshot = cache.snapshot_envelopes();
    documents
        .iter()
        .map(|document| {
            if document.store_id != "epiphany-mind"
                || document.document_type != crate::EpiphanyRepoModelFrontierDocument::TYPE
            {
                return Err(anyhow!(
                    "frontier authority contains a non-frontier Mind document"
                ));
            }
            let envelope = snapshot
                .iter()
                .find(|envelope| {
                    envelope.r#type == document.document_type
                        && envelope.key == document.document_key
                })
                .ok_or_else(|| anyhow!("frontier authority document is absent"))?;
            if crate::EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", envelope)?
                != *document
            {
                return Err(anyhow!("frontier authority document changed"));
            }
            Ok(envelope.clone())
        })
        .collect()
}

fn planning_claim_obligation_documents(
    cache: &CultCache,
    item: &crate::RepoFrontierItem,
) -> Result<Vec<crate::EpiphanyMindDocumentVersion>> {
    let mut claim_ids = item.target_claim_ids.clone();
    claim_ids.sort();
    claim_ids.dedup();
    let mut documents = claim_ids
        .into_iter()
        .map(|claim_id| {
            let envelope = cache
                .get_envelope::<crate::EpiphanyRepoModelClaimObligationsDocument>(&claim_id)?
                .ok_or_else(|| anyhow!("planning claim obligation {claim_id:?} is absent"))?;
            crate::EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", &envelope)
        })
        .collect::<Result<Vec<_>>>()?;
    documents.sort_by(|left, right| {
        left.document_type
            .cmp(&right.document_type)
            .then(left.document_key.cmp(&right.document_key))
    });
    Ok(documents)
}

fn planning_claim_obligation_envelopes(
    cache: &CultCache,
    documents: &[crate::EpiphanyMindDocumentVersion],
) -> Result<Vec<CultCacheEnvelope>> {
    let snapshot = cache.snapshot_envelopes();
    documents
        .iter()
        .map(|document| {
            if document.store_id != "epiphany-mind"
                || document.document_type != crate::EpiphanyRepoModelClaimObligationsDocument::TYPE
            {
                return Err(anyhow!(
                    "planning claim authority contains a non-obligation Mind document"
                ));
            }
            let envelope = snapshot
                .iter()
                .find(|envelope| {
                    envelope.r#type == document.document_type
                        && envelope.key == document.document_key
                })
                .ok_or_else(|| anyhow!("planning claim obligation is absent"))?;
            if crate::EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", envelope)?
                != *document
            {
                return Err(anyhow!("planning claim obligation changed"));
            }
            Ok(envelope.clone())
        })
        .collect()
}

fn validate_repo_frontier_research_request(
    request: &RepoFrontierResearchRequest,
) -> Result<crate::RepoFrontierItem> {
    if request.runtime_id.is_empty()
        || chrono::DateTime::parse_from_rfc3339(&request.requested_at).is_err()
        || request.frontier_item_id.is_empty()
        || request.frontier_item_hash.is_empty()
        || request.repository_scope.is_empty()
        || !crate::memory_graph::repo_paths_are_canonical_and_safe(&request.repository_scope)
        || request.frontier_authority_documents.is_empty()
    {
        return Err(anyhow!("invalid frontier Research request"));
    }
    let basis = crate::EpiphanyRepoModelBasis {
        projection_digest: request.model_projection_digest.clone(),
        source_documents: request.model_source_documents.clone(),
    };
    basis.validate()?;
    if request.frontier_authority_documents.iter().any(|document| {
        document.validate().is_err()
            || document.store_id != "epiphany-mind"
            || document.document_type != crate::EpiphanyRepoModelFrontierDocument::TYPE
            || !request.model_source_documents.contains(document)
    }) || !request.frontier_authority_documents.windows(2).all(|pair| {
        (
            pair[0].document_type.as_str(),
            pair[0].document_key.as_str(),
        ) < (
            pair[1].document_type.as_str(),
            pair[1].document_key.as_str(),
        )
    }) {
        return Err(anyhow!(
            "Research frontier authority is not one canonical subset of its sealed model projection"
        ));
    }
    let frontier_source = request
        .frontier_authority_documents
        .iter()
        .find(|document| document.document_key == request.frontier_item_id)
        .ok_or_else(|| anyhow!("Research request lost its owning frontier document"))?;
    let frontier = rmp_serde::from_slice::<crate::EpiphanyRepoModelFrontierDocument>(
        &frontier_source.payload_msgpack,
    )?
    .value()?;
    let mut expected_authority_ids = vec![frontier.id.as_str()];
    expected_authority_ids.extend(frontier.dependency_item_ids.iter().map(String::as_str));
    expected_authority_ids.sort_unstable();
    expected_authority_ids.dedup();
    if request
        .frontier_authority_documents
        .iter()
        .map(|document| document.document_key.as_str())
        .collect::<Vec<_>>()
        != expected_authority_ids
        || request.frontier_item_id != frontier.id
        || request.frontier_item_hash != repo_frontier_item_hash(&frontier)?
        || request.repository_scope != frontier.repository_scope
        || request.public_source_refs
            != crate::ImmutableGithubSource::canonicalize_set(
                frontier.public_source_refs.iter().map(String::as_str),
            )?
        || request.public_source_refs != frontier.public_source_refs
        || request.request_id
            != crate::frontier_research_request_id(
                &request.runtime_id,
                &request.frontier_item_id,
                &request.frontier_item_hash,
            )
    {
        return Err(anyhow!(
            "frontier Research request diverges from its sealed frontier authority"
        ));
    }
    Ok(frontier)
}

fn current_repo_frontier_research_request(
    cache: &CultCache,
    request: &RepoFrontierResearchRequest,
) -> Result<Option<crate::RepoFrontierItem>> {
    let sealed_frontier = validate_repo_frontier_research_request(request)?;
    let identity = require_identity(cache)?;
    if request.runtime_id != identity.runtime_id {
        return Ok(None);
    }
    if frontier_authority_envelopes(cache, &request.frontier_authority_documents).is_err() {
        return Ok(None);
    }
    let frontier = cache
        .get::<crate::EpiphanyRepoModelFrontierDocument>(&request.frontier_item_id)?
        .ok_or_else(|| anyhow!("Research request frontier document is absent"))?
        .value()?;
    let expected_authority = frontier_authority_documents(cache, &frontier)?;
    if expected_authority != request.frontier_authority_documents {
        return Ok(None);
    }
    let dependencies_ready = frontier.dependency_item_ids.iter().all(|dependency_id| {
        cache
            .get::<crate::EpiphanyRepoModelFrontierDocument>(dependency_id)
            .ok()
            .flatten()
            .and_then(|document| document.value().ok())
            .is_some_and(|dependency| {
                matches!(
                    dependency.status,
                    crate::RepoFrontierStatus::Resolved
                        | crate::RepoFrontierStatus::Retired
                        | crate::RepoFrontierStatus::Superseded
                )
            })
    });
    Ok((frontier == sealed_frontier
        && frontier.status == crate::RepoFrontierStatus::Active
        && frontier.recommended_next_organ == "Eyes"
        && dependencies_ready)
        .then_some(frontier))
}

pub(crate) fn select_and_commit_repo_frontier_research_request(
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
    let (view, basis) = current_keyed_repo_model(&cache)?;
    let model = view.memory_context_projection();
    let launches = cache.get_all::<EpiphanyRuntimeWorkerLaunchRequest>()?;
    let packets = cache.get_all::<EyesEvidencePacket>()?;
    let item = match next_repo_frontier_research_work(&cache, &model, &launches, &packets)? {
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
        &cache,
        &model,
        &basis,
        &item,
        at,
    )?;
    let request_id = request.request_id.clone();
    if let Some(existing) = cache.get::<RepoFrontierResearchRequest>(&request_id)? {
        let mut replay = request.clone();
        replay.requested_at = existing.requested_at.clone();
        return if existing == replay {
            Ok(existing)
        } else {
            Err(anyhow!("frontier Research request identity collision"))
        };
    }
    let (request_envelope, _) = cache.prepare_entry(&request_id, &request)?;
    let expected = frontier_authority_envelopes(&cache, &request.frontier_authority_documents)?;
    let mut writes = expected.clone();
    writes.push(request_envelope);
    if !backing.compare_and_swap_batch(&expected, writes)? {
        return Err(anyhow!(
            "frontier Research request lost exact frontier authority CAS"
        ));
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
    cache: &CultCache,
    model: &crate::EpiphanyMemoryGraphSnapshot,
    basis: &crate::EpiphanyRepoModelBasis,
    item: &crate::RepoFrontierItem,
    requested_at: &str,
) -> Result<RepoFrontierResearchRequest> {
    if runtime_id.is_empty()
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
    let frontier_authority_documents = frontier_authority_documents(cache, item)?;
    let public_source_refs = crate::ImmutableGithubSource::canonicalize_set(
        item.public_source_refs.iter().map(String::as_str),
    )?;
    if public_source_refs != item.public_source_refs {
        return Err(anyhow!(
            "frontier Research public source authority is not canonical"
        ));
    }
    Ok(RepoFrontierResearchRequest {
        request_id: crate::frontier_research_request_id(runtime_id, &item.id, &item_hash),
        model_projection_digest: basis.projection_digest.clone(),
        model_source_documents: basis.source_documents.clone(),
        frontier_authority_documents,
        frontier_item_id: item.id.clone(),
        frontier_item_hash: item_hash,
        repository_scope: item.repository_scope.clone(),
        requested_at: requested_at.to_string(),
        runtime_id: runtime_id.to_string(),
        public_source_refs,
    })
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
pub(crate) fn runtime_repo_frontier_research_lifecycle(
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
    let (view, _basis) = current_keyed_repo_model(&cache)?;
    let model = view.memory_context_projection();

    let work = next_repo_frontier_research_work(&cache, &model, &launches, &packets)?;
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
        EpiphanyRuntimeJobStatus::Completed => {
            let result = role_results
                .iter()
                .find(|result| result.job_id == job.job_id)
                .ok_or_else(|| anyhow!("completed frontier Research job lost its typed result"))?;
            if result.repo_frontier_research_request_id.as_deref()
                != Some(request.request_id.as_str())
                || !result.role_id.eq_ignore_ascii_case("research")
            {
                return Err(anyhow!(
                    "frontier Research result crossed request-family authority"
                ));
            }
            if result.item_error.is_some() {
                RepoFrontierResearchLifecycleStage::LaunchReady
            } else {
                RepoFrontierResearchLifecycleStage::ResultReady
            }
        }
        EpiphanyRuntimeJobStatus::Failed | EpiphanyRuntimeJobStatus::Cancelled => {
            let terminal = job_results
                .iter()
                .filter(|result| result.job_id == job.job_id)
                .count();
            if terminal != 1 {
                return Err(anyhow!(
                    "failed frontier Research attempt lost its exact terminal result"
                ));
            }
            RepoFrontierResearchLifecycleStage::LaunchReady
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
    existing_uncovered.sort_by(|left, right| left.request_id.cmp(&right.request_id));
    if let Some(request) = existing_uncovered.into_iter().next() {
        return Ok(Some(NextRepoFrontierResearchWork::Existing(request)));
    }
    for item in model
        .frontier
        .iter()
        .filter(|item| frontier_item_is_actionable_for_organ(model, item, "Eyes"))
    {
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
    for packet in packets
        .iter()
        .filter(|packet| packet.research_request_id == request.request_id)
    {
        let result = cache.get::<EpiphanyRuntimeRoleWorkerResult>(&packet.source_job_id)?;
        let archived = cache.get::<EpiphanyArchivedRuntimeWorkerAttempt>(&packet.source_job_id)?;
        let exact_live = result.as_ref().is_some_and(|result| {
            matching_jobs.contains(result.job_id.as_str())
                && result.result_id == packet.source_result_id
                && result.decision_context_id == packet.decision_context_id
                && result.repo_frontier_research_request_id.as_deref()
                    == Some(request.request_id.as_str())
        });
        let exact_archived = archived.as_ref().is_some_and(|attempt| {
            attempt.request_kind == "frontier-research"
                && attempt.request_id == request.request_id
                && attempt.fulfilled_result_id() == Some(packet.source_result_id.as_str())
                && attempt.decision_context_id() == Some(packet.decision_context_id.as_str())
                && attempt.decision.as_ref().is_some_and(|decision| {
                    decision.role_result.as_ref().is_some_and(|result| {
                        result.repo_frontier_research_request_id.as_deref()
                            == Some(request.request_id.as_str())
                    })
                })
        });
        if exact_live || exact_archived {
            return Ok(true);
        }
    }
    Ok(false)
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
    if request.repository_scope.is_empty()
        || !crate::memory_graph::repo_paths_are_canonical_and_safe(&request.repository_scope)
    {
        return Err(anyhow!("invalid frontier Research request"));
    }
    Ok(current_repo_frontier_research_request(cache, request)?.is_some())
}

fn repo_frontier_planning_failure_review_id(result_id: &str) -> String {
    format!("repo-frontier-planning-failure-review-{result_id}")
}

pub(crate) fn repo_frontier_planning_failure_review(
    cache: &CultCache,
    planning_request_id: &str,
    pass_kind: &str,
    result: &EpiphanyRuntimeRoleWorkerResult,
) -> Result<Option<RepoFrontierPlanningFailureReview>> {
    let review_id = repo_frontier_planning_failure_review_id(&result.result_id);
    let Some(review) = cache.get::<RepoFrontierPlanningFailureReview>(&review_id)? else {
        return Ok(None);
    };
    if review.review_id != review_id
        || review.planning_request_id != planning_request_id
        || review.pass_kind != pass_kind
        || review.job_id != result.job_id
        || review.result_id != result.result_id
        || review.disposition != "superseded"
        || chrono::DateTime::parse_from_rfc3339(&review.reviewed_at).is_err()
    {
        return Err(anyhow!("frontier planning failure review is corrupt"));
    }
    Ok(Some(review))
}

pub fn review_repo_frontier_planning_failure(
    runtime_store: impl AsRef<Path>,
    job_id: &str,
    reviewed_at: &str,
) -> Result<RepoFrontierPlanningFailureReview> {
    chrono::DateTime::parse_from_rfc3339(reviewed_at)
        .map_err(|_| anyhow!("planning failure review time must be RFC3339"))?;
    let runtime_store = runtime_store.as_ref();
    let lifecycle = runtime_repo_frontier_planning_lifecycle(runtime_store)?;
    let (pass_kind, expected_job_id) = match lifecycle.stage {
        RepoFrontierPlanningLifecycleStage::ImaginationFailed => {
            ("imagination", lifecycle.imagination_job_id.as_deref())
        }
        RepoFrontierPlanningLifecycleStage::MindFailed => {
            ("mind", lifecycle.mind_job_id.as_deref())
        }
        _ => return Err(anyhow!("frontier planning has no reviewable failed pass")),
    };
    if expected_job_id != Some(job_id) {
        return Err(anyhow!("planning failure review job is not current"));
    }
    let planning_request_id = lifecycle
        .planning_request_id
        .ok_or_else(|| anyhow!("planning failure review lost its request"))?;
    let mut cache = runtime_spine_cache(runtime_store)?;
    cache.pull_all_backing_stores()?;
    let result = cache
        .get::<EpiphanyRuntimeRoleWorkerResult>(job_id)?
        .ok_or_else(|| anyhow!("planning failure review lost its typed result"))?;
    let review = RepoFrontierPlanningFailureReview {
        review_id: repo_frontier_planning_failure_review_id(&result.result_id),
        planning_request_id: planning_request_id.clone(),
        pass_kind: pass_kind.into(),
        job_id: job_id.into(),
        result_id: result.result_id.clone(),
        disposition: "superseded".into(),
        reviewed_at: reviewed_at.into(),
    };
    if let Some(existing) =
        repo_frontier_planning_failure_review(&cache, &planning_request_id, pass_kind, &result)?
    {
        return if existing == review {
            Ok(existing)
        } else {
            Err(anyhow!("planning failure review identity collision"))
        };
    }
    let snapshot = cache.snapshot_envelopes();
    let mut expected = Vec::new();
    for (document_type, document_key) in [
        (
            RepoFrontierPlanningRequest::TYPE,
            planning_request_id.as_str(),
        ),
        (EpiphanyRuntimeWorkerLaunchRequest::TYPE, job_id),
        (EpiphanyRuntimeJob::TYPE, job_id),
        (EpiphanyRuntimeRoleWorkerResult::TYPE, job_id),
    ] {
        expected.push(
            snapshot
                .iter()
                .find(|envelope| envelope.r#type == document_type && envelope.key == document_key)
                .cloned()
                .ok_or_else(|| anyhow!("planning failure review lost a strong source"))?,
        );
    }
    if pass_kind == "mind" {
        let mind_request_id = lifecycle
            .mind_request_id
            .ok_or_else(|| anyhow!("Mind failure review lost its request"))?;
        expected.push(
            cache
                .get_envelope::<RepoFrontierPlanMindRequest>(&mind_request_id)?
                .ok_or_else(|| anyhow!("Mind failure review request envelope is missing"))?,
        );
    }
    let envelope = cache.prepare_entry(&review.review_id, &review)?.0;
    let mut writes = expected.clone();
    writes.push(envelope);
    let backing = SingleFileMessagePackBackingStore::new(runtime_store);
    if backing.compare_and_swap_batch(&expected, writes)? {
        return Ok(review);
    }
    let mut reloaded = runtime_spine_cache(runtime_store)?;
    reloaded.pull_all_backing_stores()?;
    match reloaded.get::<RepoFrontierPlanningFailureReview>(&review.review_id)? {
        Some(existing) if existing == review => Ok(existing),
        _ => Err(anyhow!("planning failure review lost exact-envelope CAS")),
    }
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
        if runtime_has_actionable_frontier_for_organ(runtime_store, "Imagination")? {
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
        .get_all::<EpiphanyRuntimeWorkerLaunchRequest>()?
        .into_iter()
        .filter(|launch| {
            launch.frontier_planning_request_id.as_deref() == Some(request.request_id.as_str())
        })
        .map(|launch| {
            Ok((
                crate::current_work::frontier_planning_attempt_ordinal(
                    &request.request_id,
                    &launch.job_id,
                )?,
                launch,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    imagination_launches.sort_by_key(|(ordinal, _)| *ordinal);
    for (expected, (ordinal, _)) in imagination_launches.iter().enumerate() {
        if *ordinal != expected {
            return Err(anyhow!(
                "Self found noncontiguous frontier planning attempt identity"
            ));
        }
    }
    if let Some((_, launch)) = imagination_launches.last() {
        lifecycle.imagination_job_id = Some(launch.job_id.clone());
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
        let reviewed = repo_frontier_planning_failure_review(
            &cache,
            &request.request_id,
            "imagination",
            imagination_result,
        )?;
        lifecycle.stage = if reviewed.is_some() {
            RepoFrontierPlanningLifecycleStage::ImaginationLaunchReady
        } else {
            RepoFrontierPlanningLifecycleStage::ImaginationFailed
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
        .get_all::<EpiphanyRuntimeWorkerLaunchRequest>()?
        .into_iter()
        .filter(|launch| {
            launch.frontier_plan_mind_request_id.as_deref()
                == Some(mind_request.request_id.as_str())
        })
        .map(|launch| {
            Ok((
                crate::current_work::frontier_plan_mind_attempt_ordinal(
                    &mind_request.request_id,
                    &launch.job_id,
                )?,
                launch,
            ))
        })
        .collect::<Result<Vec<_>>>()?;
    mind_launches.sort_by_key(|(ordinal, _)| *ordinal);
    for (expected, (ordinal, _)) in mind_launches.iter().enumerate() {
        if *ordinal != expected {
            return Err(anyhow!("Self found noncontiguous Mind attempt identity"));
        }
    }
    if let Some((_, launch)) = mind_launches.last() {
        lifecycle.mind_job_id = Some(launch.job_id.clone());
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
        let reviewed = repo_frontier_planning_failure_review(
            &cache,
            &request.request_id,
            "mind",
            mind_result,
        )?;
        lifecycle.stage = if reviewed.is_some() {
            RepoFrontierPlanningLifecycleStage::MindLaunchReady
        } else {
            RepoFrontierPlanningLifecycleStage::MindFailed
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
    let Ok((view, _basis)) = current_keyed_repo_model(&cache) else {
        return Ok(false);
    };
    let model = view.memory_context_projection();
    Ok(actionable_frontier_item_for_organ(&model, organ).is_some())
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

pub(crate) fn runtime_reorient_worker_result(
    store_path: impl AsRef<Path>,
    job_id: &str,
) -> Result<Option<EpiphanyRuntimeReorientWorkerResult>> {
    validate_non_empty(job_id, "reorient worker result job id")?;
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    cache.get::<EpiphanyRuntimeReorientWorkerResult>(job_id)
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

fn validate_repo_frontier_hands_authority_chain(
    cache: &CultCache,
    authority: &RepoFrontierHandsAuthority,
) -> Result<()> {
    let route = cache
        .get::<RepoFrontierRoute>(&authority.route_id)?
        .ok_or_else(|| anyhow!("Hands authority requires its persisted route"))?;
    let frontier_source = route
        .model_source_documents
        .iter()
        .find(|source| {
            source.document_type == crate::EpiphanyRepoModelFrontierDocument::TYPE
                && source.document_key == route.frontier_item_id
        })
        .ok_or_else(|| anyhow!("Hands authority route lost its exact frontier version"))?;
    let current_frontier_envelope = cache
        .get_envelope::<crate::EpiphanyRepoModelFrontierDocument>(&route.frontier_item_id)?
        .ok_or_else(|| anyhow!("Hands authority lost its model frontier"))?;
    if crate::EpiphanyMindDocumentVersion::from_envelope(
        "epiphany-mind",
        &current_frontier_envelope,
    )? != *frontier_source
    {
        return Err(anyhow!(
            "Hands authority frontier version is no longer current"
        ));
    }
    let current_item: crate::EpiphanyRepoModelFrontierDocument =
        rmp_serde::from_slice(&current_frontier_envelope.payload)?;
    let current_item = current_item.value()?;
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
        route.authorized_paths.iter().any(|scope| {
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
    if intent.schema_version != HANDS_ACTION_INTENT_SCHEMA_VERSION
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
            .is_some_and(|plan| route.authorized_paths != plan.safe_paths)
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
    if chrono::DateTime::parse_from_rfc3339(&authority.granted_at).is_err()
        || !crate::memory_graph::repo_paths_are_canonical_and_safe(&authority.requested_paths)
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
        route.authorized_paths.iter().any(|scope| {
            path == scope || path.starts_with(&format!("{}/", scope.trim_end_matches(['/', '\\'])))
        })
    });
    if route.next_organ != RepoFrontierNextOrgan::Hands
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
    let expected = vec![
        cache
            .get_envelope::<RepoFrontierRoute>(&route.route_id)?
            .ok_or_else(|| anyhow!("repo frontier Hands authority lost its route envelope"))?,
        cache
            .get_envelope::<HandsActionIntent>(&intent.intent_id)?
            .ok_or_else(|| anyhow!("repo frontier Hands authority lost its intent envelope"))?,
        cache
            .get_envelope::<HandsActionReview>(&review.review_id)?
            .ok_or_else(|| anyhow!("repo frontier Hands authority lost its review envelope"))?,
        cache
            .get_envelope::<SubstrateGateRepoAccessGrantReceipt>(&grant.receipt_id)?
            .ok_or_else(|| anyhow!("repo frontier Hands authority lost its grant envelope"))?,
        cache
            .get_envelope::<crate::EpiphanyRepoModelFrontierDocument>(&authority.frontier_item_id)?
            .ok_or_else(|| anyhow!("repo frontier Hands authority lost its frontier envelope"))?,
    ];
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
            ) && if result.role_id.eq_ignore_ascii_case("modeling") {
                receipt.invariant_owner.starts_with("Modeling.")
                    && receipt
                        .writes
                        .iter()
                        .any(|write| write.document_type.starts_with("epiphany.mind.repo_model."))
            } else if result.role_id.eq_ignore_ascii_case("research") {
                receipt.invariant_owner == "Eyes.frontier_research"
                    && receipt.writes.iter().any(|write| {
                        matches!(
                            write.document_type.as_str(),
                            crate::EpiphanyMindEvidenceDocument::TYPE
                                | crate::EpiphanyMindObservationDocument::TYPE
                        )
                    })
            } else if result.role_id.eq_ignore_ascii_case("verification") {
                receipt.invariant_owner == "Soul.verification"
                    && receipt.writes.iter().any(|write| {
                        write.document_type == crate::EpiphanyMindVerificationAuditDocument::TYPE
                    })
            } else {
                false
            }
        }))
}

pub(crate) fn validate_repo_frontier_verification_request_intrinsic(
    request: &RepoFrontierVerificationRequest,
) -> Result<crate::RepoFrontierItem> {
    if chrono::DateTime::parse_from_rfc3339(&request.requested_at).is_err()
        || [
            request.request_id.as_str(),
            request.route_id.as_str(),
            request.frontier_item_id.as_str(),
            request.frontier_item_hash.as_str(),
            request.hands_intent_id.as_str(),
            request.hands_review_id.as_str(),
            request.hands_patch_receipt_id.as_str(),
            request.hands_command_receipt_id.as_str(),
            request.hands_commit_receipt_id.as_str(),
        ]
        .into_iter()
        .any(str::is_empty)
        || request.frontier_authority_documents.len() != 1
    {
        return Err(anyhow!(
            "invalid repo frontier verification request contract"
        ));
    }
    crate::EpiphanyRepoModelBasis {
        projection_digest: request.model_projection_digest.clone(),
        source_documents: request.model_source_documents.clone(),
    }
    .validate()?;
    let source = &request.frontier_authority_documents[0];
    source.validate()?;
    if source.store_id != "epiphany-mind"
        || source.document_type != crate::EpiphanyRepoModelFrontierDocument::TYPE
        || source.document_key != request.frontier_item_id
    {
        return Err(anyhow!(
            "Verification request frontier authority has the wrong identity"
        ));
    }
    let frontier: crate::EpiphanyRepoModelFrontierDocument =
        rmp_serde::from_slice(&source.payload_msgpack)?;
    let frontier = frontier.value()?;
    let frontier_hash = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&frontier)?));
    if frontier.id != request.frontier_item_id || frontier_hash != request.frontier_item_hash {
        return Err(anyhow!(
            "Verification request frontier authority does not bind its exact item"
        ));
    }
    Ok(frontier)
}

pub(crate) fn verification_frontier_is_current(
    cache: &CultCache,
    request: &RepoFrontierVerificationRequest,
) -> Result<bool> {
    validate_repo_frontier_verification_request_intrinsic(request)?;
    let source = &request.frontier_authority_documents[0];
    Ok(cache
        .snapshot_envelopes()
        .iter()
        .find(|envelope| {
            envelope.r#type == source.document_type && envelope.key == source.document_key
        })
        .map(|envelope| {
            crate::EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", envelope)
                .map(|current| current == *source)
        })
        .transpose()?
        .unwrap_or(false))
}

pub(crate) fn repo_frontier_verification_context(
    cache: &CultCache,
    request: &RepoFrontierVerificationRequest,
) -> Result<crate::RepoFrontierVerificationContextProjection> {
    repo_frontier_verification_context_with_commit(cache, request, None)
}

fn repo_frontier_verification_context_with_commit(
    cache: &CultCache,
    request: &RepoFrontierVerificationRequest,
    prospective_commit: Option<&HandsCommitReceipt>,
) -> Result<crate::RepoFrontierVerificationContextProjection> {
    validate_repo_frontier_verification_request_intrinsic(request)?;
    if !verification_frontier_is_current(cache, request)? {
        return Err(anyhow!(
            "Verification context cannot be projected from stale frontier authority"
        ));
    }
    let route = cache
        .get::<RepoFrontierRoute>(&request.route_id)?
        .ok_or_else(|| anyhow!("Verification context lost its route"))?;
    let authorities = cache
        .get_all::<RepoFrontierHandsAuthority>()?
        .into_iter()
        .filter(|authority| {
            authority.route_id == request.route_id
                && authority.hands_intent_id == request.hands_intent_id
                && authority.hands_review_id == request.hands_review_id
        })
        .collect::<Vec<_>>();
    let [hands_authority] = authorities.as_slice() else {
        return Err(anyhow!(
            "Verification context requires one exact Hands authority"
        ));
    };
    validate_repo_frontier_hands_authority_chain(cache, hands_authority)?;
    let hands_intent = cache
        .get::<HandsActionIntent>(&request.hands_intent_id)?
        .ok_or_else(|| anyhow!("Verification context lost its Hands intent"))?;
    let hands_review = cache
        .get::<HandsActionReview>(&request.hands_review_id)?
        .ok_or_else(|| anyhow!("Verification context lost its Hands review"))?;
    let patch_receipt = cache
        .get::<HandsPatchReceipt>(&request.hands_patch_receipt_id)?
        .ok_or_else(|| anyhow!("Verification context lost its patch receipt"))?;
    let command_receipt = cache
        .get::<HandsCommandReceipt>(&request.hands_command_receipt_id)?
        .ok_or_else(|| anyhow!("Verification context lost its command receipt"))?;
    let commit_receipt = match prospective_commit {
        Some(receipt)
            if receipt.receipt_id == request.hands_commit_receipt_id
                && receipt.intent_id == request.hands_intent_id
                && receipt.review_id == request.hands_review_id =>
        {
            receipt.clone()
        }
        Some(_) => {
            return Err(anyhow!(
                "prospective Hands commit does not exactly bind its Verification request"
            ));
        }
        None => cache
            .get::<HandsCommitReceipt>(&request.hands_commit_receipt_id)?
            .ok_or_else(|| anyhow!("Verification context lost its commit receipt"))?,
    };
    Ok(crate::RepoFrontierVerificationContextProjection {
        schema_version: crate::REPO_FRONTIER_VERIFICATION_CONTEXT_SCHEMA_VERSION.into(),
        request: request.clone(),
        route,
        hands_authority: hands_authority.clone(),
        hands_intent,
        hands_review,
        patch_receipt,
        command_receipt,
        commit_receipt,
        contract: crate::REPO_FRONTIER_VERIFICATION_CONTEXT_CONTRACT.into(),
    })
}

pub(crate) fn derive_repo_frontier_modeling_request(
    cache: &CultCache,
    verdict: &SoulVerdictReceipt,
) -> Result<RepoFrontierModelingRequest> {
    let identity = require_identity(cache)?;
    if verdict.receipt_id.trim().is_empty()
        || verdict.source_result_id.trim().is_empty()
        || verdict.source_job_id.trim().is_empty()
        || chrono::DateTime::parse_from_rfc3339(&verdict.emitted_at).is_err()
    {
        return Err(anyhow!(
            "frontier Modeling request requires one typed Soul verdict"
        ));
    }
    let result = cache
        .get::<EpiphanyRuntimeRoleWorkerResult>(&verdict.source_job_id)?
        .ok_or_else(|| anyhow!("frontier Modeling request requires its Verification result"))?;
    let verification_request = cache
        .get::<RepoFrontierVerificationRequest>(&verdict.verification_request_id)?
        .ok_or_else(|| anyhow!("frontier Modeling request requires the exact Soul request"))?;
    let route = cache
        .get::<RepoFrontierRoute>(&verdict.frontier_route_id)?
        .ok_or_else(|| anyhow!("frontier Modeling request requires the exact frontier route"))?;
    let item_envelope = cache
        .get_envelope::<crate::EpiphanyRepoModelFrontierDocument>(&route.frontier_item_id)?
        .ok_or_else(|| anyhow!("frontier Modeling request routed item is missing"))?;
    let item =
        rmp_serde::from_slice::<crate::EpiphanyRepoModelFrontierDocument>(&item_envelope.payload)?
            .value()?;
    let item_version =
        crate::EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", &item_envelope)?;
    let item_hash = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&item)?));
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
    if !result.role_id.eq_ignore_ascii_case("verification")
        || result.item_error.is_some()
        || result.result_id != verdict.source_result_id
        || result.job_id != verdict.source_job_id
        || result.verification_request_id.as_deref()
            != Some(verification_request.request_id.as_str())
        || result.frontier_route_id.as_deref() != Some(route.route_id.as_str())
        || verdict.verdict != result.verdict
        || verdict.summary != result.summary
        || verdict.risks != result.risks
        || verdict_evidence != result_evidence
        || verification_request.route_id != route.route_id
        || verification_request.model_projection_digest != route.model_projection_digest
        || verification_request.model_source_documents != route.model_source_documents
        || verification_request.frontier_item_id != route.frontier_item_id
        || verification_request.frontier_item_hash != route.frontier_item_hash
        || !route.model_source_documents.contains(&item_version)
        || item_hash != route.frontier_item_hash
        || item.status != crate::RepoFrontierStatus::Active
    {
        return Err(anyhow!(
            "frontier Modeling request does not exactly bind accepted result, Soul verdict, request, route, item, and current model"
        ));
    }
    let request_id = crate::causal_work_identity::frontier_verdict_modeling_request_id(
        &identity.runtime_id,
        &verdict.receipt_id,
        &result.result_id,
        &route.route_id,
    );
    let request = RepoFrontierModelingRequest {
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
        allowed_disposition: disposition,
        requested_at: verdict.emitted_at.clone(),
    };
    Ok(request)
}

pub(crate) fn validate_repo_frontier_modeling_request(
    cache: &CultCache,
    request: &RepoFrontierModelingRequest,
) -> Result<()> {
    let verdict = cache
        .get::<SoulVerdictReceipt>(&request.soul_verdict_receipt_id)?
        .ok_or_else(|| anyhow!("frontier Modeling request lost its Soul verdict"))?;
    if derive_repo_frontier_modeling_request(cache, &verdict)? != *request {
        return Err(anyhow!(
            "frontier Modeling request is not the canonical typed consequence of its Soul verdict"
        ));
    }
    Ok(())
}

fn derive_repo_frontier_verification_request_for_chain(
    cache: &CultCache,
    patch: &HandsPatchReceipt,
    command: &HandsCommandReceipt,
    commit: &HandsCommitReceipt,
) -> Result<RepoFrontierVerificationRequest> {
    let authorities = cache
        .get_all::<RepoFrontierHandsAuthority>()?
        .into_iter()
        .filter(|value| {
            value.hands_intent_id == commit.intent_id && value.hands_review_id == commit.review_id
        })
        .collect::<Vec<_>>();
    if authorities.len() != 1 {
        return Err(anyhow!(
            "complete Hands chain requires exactly one frontier authority before Soul launch"
        ));
    }
    let authority = &authorities[0];
    let frontier_envelope = cache
        .get_envelope::<crate::EpiphanyRepoModelFrontierDocument>(&authority.frontier_item_id)?
        .ok_or_else(|| anyhow!("Hands authority lost its exact frontier document"))?;
    let frontier_authority_documents = vec![crate::EpiphanyMindDocumentVersion::from_envelope(
        "epiphany-mind",
        &frontier_envelope,
    )?];
    let request_id = format!(
        "frontier-verification-{}-{}",
        authority.route_id, commit.receipt_id
    );
    let requested_at = cache
        .get::<RepoFrontierVerificationRequest>(&request_id)?
        .map(|existing| existing.requested_at)
        .unwrap_or_else(|| commit.emitted_at.clone());
    let request = RepoFrontierVerificationRequest {
        request_id,
        route_id: authority.route_id.clone(),
        model_projection_digest: authority.model_projection_digest.clone(),
        model_source_documents: authority.model_source_documents.clone(),
        frontier_item_id: authority.frontier_item_id.clone(),
        frontier_item_hash: authority.frontier_item_hash.clone(),
        hands_intent_id: commit.intent_id.clone(),
        hands_review_id: commit.review_id.clone(),
        hands_patch_receipt_id: patch.receipt_id.clone(),
        hands_command_receipt_id: command.receipt_id.clone(),
        hands_commit_receipt_id: commit.receipt_id.clone(),
        requested_at,
        frontier_authority_documents,
    };
    Ok(request)
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
    let patch = cache
        .get_all::<HandsPatchReceipt>()?
        .into_iter()
        .filter(|patch| {
            patch.intent_id == receipt.intent_id
                && patch.review_id == receipt.review_id
                && patch.runtime_job_id == receipt.runtime_job_id
                && patch.emitted_at <= receipt.emitted_at
        })
        .max_by(|left, right| left.emitted_at.cmp(&right.emitted_at))
        .ok_or_else(|| anyhow!("Hands commit requires its exact patch receipt"))?;
    let command = cache
        .get_all::<HandsCommandReceipt>()?
        .into_iter()
        .filter(|command| {
            command.intent_id == receipt.intent_id
                && command.review_id == receipt.review_id
                && command.runtime_job_id == receipt.runtime_job_id
                && command.exit_code == "0"
                && command.emitted_at <= receipt.emitted_at
        })
        .max_by(|left, right| left.emitted_at.cmp(&right.emitted_at))
        .ok_or_else(|| anyhow!("Hands commit requires its successful command receipt"))?;
    let request =
        derive_repo_frontier_verification_request_for_chain(&cache, &patch, &command, receipt)?;
    let context = repo_frontier_verification_context_with_commit(&cache, &request, Some(receipt))?;
    let snapshot = cache.snapshot_envelopes();
    let mut expected = Vec::new();
    for (document_type, document_key) in [
        (RepoFrontierRoute::TYPE, request.route_id.as_str()),
        (
            RepoFrontierHandsAuthority::TYPE,
            context.hands_authority.authority_id.as_str(),
        ),
        (HandsActionIntent::TYPE, request.hands_intent_id.as_str()),
        (HandsActionReview::TYPE, request.hands_review_id.as_str()),
        (
            HandsPatchReceipt::TYPE,
            request.hands_patch_receipt_id.as_str(),
        ),
        (
            HandsCommandReceipt::TYPE,
            request.hands_command_receipt_id.as_str(),
        ),
        (
            crate::EpiphanyRepoModelFrontierDocument::TYPE,
            request.frontier_item_id.as_str(),
        ),
    ] {
        expected.push(
            snapshot
                .iter()
                .find(|value| value.r#type == document_type && value.key == document_key)
                .cloned()
                .ok_or_else(|| anyhow!("Hands commit lost exact Verification authority"))?,
        );
    }
    let mut writes = expected.clone();
    writes.push(cache.prepare_entry(&receipt.receipt_id, receipt)?.0);
    writes.push(cache.prepare_entry(&request.request_id, &request)?.0);
    if SingleFileMessagePackBackingStore::new(store_path)
        .compare_and_swap_batch(&expected, writes)?
    {
        return Ok(());
    }
    let mut reloaded = runtime_spine_cache(store_path)?;
    reloaded.pull_all_backing_stores()?;
    match (
        reloaded.get::<HandsCommitReceipt>(&receipt.receipt_id)?,
        reloaded.get::<RepoFrontierVerificationRequest>(&request.request_id)?,
    ) {
        (Some(existing), Some(existing_request))
            if existing == *receipt && existing_request == request =>
        {
            Ok(())
        }
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

fn validate_coordinator_run_receipt(receipt: &EpiphanyCoordinatorRunReceipt) -> Result<()> {
    validate_non_empty(&receipt.receipt_id, "coordinator run receipt id")?;
    validate_non_empty(&receipt.session_id, "coordinator run receipt session id")?;
    validate_non_empty(&receipt.thread_id, "coordinator run receipt thread id")?;
    if !matches!(receipt.mode.as_str(), "plan" | "execute") {
        return Err(anyhow!("coordinator run receipt mode is invalid"));
    }
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

pub(crate) fn runtime_typed_request_attempt_exists(
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

fn validate_archivable_typed_worker_launch(
    cache: &CultCache,
    launch: &EpiphanyRuntimeWorkerLaunchRequest,
    request_kind: &str,
    request_id: &str,
) -> Result<()> {
    if launch.job_id.trim().is_empty() {
        return Err(anyhow!(
            "worker attempt archive found invalid immutable launch"
        ));
    }
    let document = launch.launch_document()?;
    let identity = require_identity(cache)?;
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
            validate_autonomous_proposal_origin_request(cache, &proposal)?;
            let projection = match &document {
                EpiphanyWorkerLaunchDocument::Role(document) => {
                    document.proposal_modeling_context.as_ref()
                }
                EpiphanyWorkerLaunchDocument::Reorient(_) => None,
            }
            .ok_or_else(|| anyhow!("archived proposal Modeling launch lost its context"))?;
            if request.runtime_id != identity.runtime_id
                || request.proposal_payload_sha256 != proposal.payload_sha256
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
        "frontier-verdict-modeling" => {
            validate_repo_frontier_verdict_modeling_launch_authority(
                &launch.role,
                Some(request_id),
                &document,
            )?;
            validate_repository_body_launch_carrier(&launch.role, &document)?;
            let request = cache
                .get::<RepoFrontierModelingRequest>(request_id)?
                .ok_or_else(|| {
                    anyhow!("archived frontier verdict Modeling launch lost its request")
                })?;
            let authority = match &document {
                EpiphanyWorkerLaunchDocument::Role(document) => {
                    document.frontier_verdict_modeling_context.as_ref()
                }
                EpiphanyWorkerLaunchDocument::Reorient(_) => None,
            }
            .ok_or_else(|| anyhow!("archived frontier verdict Modeling launch lost its context"))?;
            let verdict = cache
                .get::<SoulVerdictReceipt>(&request.soul_verdict_receipt_id)?
                .ok_or_else(|| {
                    anyhow!("archived frontier verdict Modeling launch lost its Soul verdict")
                })?;
            let item_hash = format!(
                "{:x}",
                Sha256::digest(rmp_serde::to_vec_named(&authority.frontier_item)?)
            );
            let expected_request_id =
                crate::causal_work_identity::frontier_verdict_modeling_request_id(
                    &identity.runtime_id,
                    &request.soul_verdict_receipt_id,
                    &request.verification_result_id,
                    &request.route_id,
                );
            if authority.request != request
                || authority.soul_verdict != verdict
                || item_hash != request.frontier_item_hash
                || request.request_id != expected_request_id
            {
                return Err(anyhow!(
                    "archived frontier verdict Modeling launch provenance mismatch"
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
            let projection = match &document {
                EpiphanyWorkerLaunchDocument::Role(document) => {
                    document.imagination_consideration_context.as_ref()
                }
                EpiphanyWorkerLaunchDocument::Reorient(_) => None,
            }
            .ok_or_else(|| anyhow!("archived Imagination launch lost its context"))?;
            if request.runtime_id != identity.runtime_id
                || launch.job_id.trim().is_empty()
                || document.thread_id() != launch.job_id
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

fn archive_runtime_worker_attempt(
    store_path: impl AsRef<Path>,
    job_id: &str,
    live_resident_request_ids: &BTreeSet<String>,
    fulfilled: bool,
) -> Result<EpiphanyArchivedRuntimeWorkerAttempt> {
    validate_non_empty(job_id, "archived worker attempt job id")?;
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    require_identity(&cache)?;
    if let Some(existing) = cache.get::<EpiphanyArchivedRuntimeWorkerAttempt>(job_id)? {
        if existing.job_id != job_id || !existing.retired_chain_digest.starts_with("sha256:") {
            return Err(anyhow!("archived worker attempt tombstone is invalid"));
        }
        existing.validate_decision_record(fulfilled)?;
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
    if fulfilled {
        let request_ref = match request_kind {
            "proposal-modeling" => RuntimeTypedRequestRef::ProposalModeling(request_id),
            "frontier-verdict-modeling" => {
                RuntimeTypedRequestRef::FrontierVerdictModeling(request_id)
            }
            "frontier-research" => RuntimeTypedRequestRef::FrontierResearch(request_id),
            "frontier-verification" => RuntimeTypedRequestRef::FrontierVerification(request_id),
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
        if matches!(
            request_kind,
            "proposal-modeling"
                | "frontier-verdict-modeling"
                | "frontier-research"
                | "frontier-verification"
        ) && !worker_result_has_keyed_mind_commit(
            &cache,
            role_result
                .as_ref()
                .expect("fulfilled result checked above"),
        )? {
            return Err(anyhow!(
                "proposal Modeling attempt remains live until Mind admission owns its result"
            ));
        }
    }
    let snapshot = cache.snapshot_envelopes();
    let worker_job_results = cache
        .get_all::<EpiphanyRuntimeJobResult>()?
        .into_iter()
        .filter(|item| item.job_id == job_id)
        .collect::<Vec<_>>();
    let job_results = worker_job_results
        .iter()
        .map(|item| item.result_id.clone())
        .collect::<BTreeSet<_>>();
    let role_decision_context_id = role_result
        .as_ref()
        .map(|result| result.decision_context_id.clone());
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
    let mut archived_job_results = worker_job_results.clone();
    archived_job_results.sort_by(|left, right| left.result_id.cmp(&right.result_id));
    let decision = decision_context_id.map(|context_id| EpiphanyArchivedRuntimeWorkerDecision {
        decision_context_id: context_id,
        role_result: role_result.clone(),
        job_results: archived_job_results,
    });
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
    let mut digest = Sha256::new();
    digest.update(b"epiphany-archived-worker-attempt-root");
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
    let tombstone = EpiphanyArchivedRuntimeWorkerAttempt {
        job_id: job_id.into(),
        request_kind: request_kind.into(),
        request_id: request_id.into(),
        terminal_process_status: claim.status,
        retired_chain_digest: format!("sha256:{:x}", digest.finalize()),
        decision,
    };
    tombstone.validate_decision_record(fulfilled)?;
    let replacement = cache.prepare_entry(job_id, &tombstone)?.0;
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
) -> Result<()> {
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
    for claim in candidates.into_iter().skip(retain_recent.max(1)) {
        archive_runtime_worker_attempt(
            store_path,
            &claim.job_id,
            live_resident_request_ids,
            false,
        )?;
    }
    Ok(())
}

pub fn retain_fulfilled_runtime_worker_attempts(
    store_path: impl AsRef<Path>,
    retain_recent: usize,
    live_resident_request_ids: &BTreeSet<String>,
) -> Result<()> {
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
    for claim in candidates.into_iter().skip(retain_recent.max(1)) {
        archive_runtime_worker_attempt(store_path, &claim.job_id, live_resident_request_ids, true)?;
    }
    Ok(())
}

fn coordinator_completion_summary(receipt: &EpiphanyCoordinatorRunReceipt) -> String {
    format!(
        "Coordinator run {:?} terminalized with status {:?}.",
        receipt.receipt_id, receipt.status
    )
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.coordinator_death_recovery.v0",
    schema = "EpiphanyCoordinatorDeathRecovery"
)]
pub struct EpiphanyCoordinatorDeathRecovery {
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
) -> Result<EpiphanyRuntimeSession> {
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
    if cache.get::<EpiphanyRuntimeSession>(session_id)?.is_some()
        || cache
            .get::<EpiphanyArchivedRuntimeSession>(session_id)?
            .is_some()
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
        session_id: session_id.to_string(),
        objective: objective.to_string(),
        status: EpiphanyRuntimeSessionStatus::Active,
        created_at: started_at.to_string(),
        updated_at: started_at.to_string(),
        coordinator_note: "Coordinator owns native runtime receipts before process exit."
            .to_string(),
    };
    let snapshot = cache.snapshot_envelopes();
    let replacements = vec![cache.prepare_entry(session_id, &session)?.0];
    if !runtime_spine_backing_store(store_path)?
        .replace_and_append_if_snapshot_unchanged(&snapshot, replacements)?
    {
        return Err(anyhow!(
            "coordinator run opening lost its full snapshot fence"
        ));
    }
    Ok(session)
}

fn coordinator_death_recovery_summary(recovery: &EpiphanyCoordinatorDeathRecovery) -> String {
    format!(
        "Continuity terminalized coordinator session after exact process observation {:?}{}.",
        recovery.observation,
        recovery
            .exit_code
            .map(|code| format!(" with exit code {code}"))
            .unwrap_or_default()
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

pub(crate) fn recover_coordinator_run_after_exact_process_death(
    store_path: impl AsRef<Path>,
    recovery: &EpiphanyCoordinatorDeathRecovery,
    expected_objective: &str,
) -> Result<EpiphanyRuntimeSession> {
    validate_non_empty(expected_objective, "recovered coordinator objective")?;
    if recovery.recovery_id != format!("coordinator-death-recovery-{}", recovery.session_id)
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
    let completion_summary = coordinator_death_recovery_summary(recovery);
    if session.status == EpiphanyRuntimeSessionStatus::Completed {
        let existing_recovery =
            cache.get::<EpiphanyCoordinatorDeathRecovery>(&recovery.recovery_id)?;
        if existing_recovery.as_ref() == Some(recovery)
            && session.objective == expected_objective
            && session.updated_at == recovery.recovered_at
            && session.coordinator_note == completion_summary
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
    let session_started_at = chrono::DateTime::parse_from_rfc3339(&session.created_at)
        .map_err(|error| anyhow!("coordinator session start timestamp is invalid: {error}"))?;
    let recovered_at = chrono::DateTime::parse_from_rfc3339(&recovery.recovered_at)
        .map_err(|error| anyhow!("coordinator death recovery timestamp is invalid: {error}"))?;
    if (session_started_at.timestamp_millis().max(0) as u64) < recovery.resident_started_at_millis
        || recovered_at < session_started_at
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
    {
        return Err(anyhow!(
            "coordinator death recovery found substituted or competing authority"
        ));
    }
    let snapshot = cache.snapshot_envelopes();
    session.status = EpiphanyRuntimeSessionStatus::Completed;
    session.updated_at = recovery.recovered_at.clone();
    session.coordinator_note = completion_summary;
    let replacements = vec![
        cache.prepare_entry(&session.session_id, &session)?.0,
        cache.prepare_entry(&recovery.recovery_id, recovery)?.0,
    ];
    if !runtime_spine_backing_store(store_path)?
        .replace_and_append_if_snapshot_unchanged(&snapshot, replacements)?
    {
        return Err(anyhow!(
            "coordinator death recovery lost its full snapshot fence"
        ));
    }
    Ok(session)
}

pub fn finalize_coordinator_run(
    store_path: impl AsRef<Path>,
    receipt: &EpiphanyCoordinatorRunReceipt,
) -> Result<EpiphanyRuntimeSession> {
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
    let completion_summary = coordinator_completion_summary(receipt);
    if session.status == EpiphanyRuntimeSessionStatus::Completed {
        let existing_receipt = cache.get::<EpiphanyCoordinatorRunReceipt>(&receipt.receipt_id)?;
        if existing_receipt.as_ref() == Some(receipt)
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
    ];
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
    let snapshot = cache.snapshot_envelopes();
    let job_envelope = snapshot
        .iter()
        .find(|entry| entry.r#type == EpiphanyRuntimeJob::TYPE && entry.key == options.job_id)
        .cloned()
        .ok_or_else(|| anyhow!("runtime job lost its exact envelope"))?;
    let terminal_status = terminal_status_for_verdict(&options.verdict);
    job.status = terminal_status;
    job.updated_at = options.completed_at.clone();
    let result = EpiphanyRuntimeJobResult {
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
    let mut expected = vec![job_envelope];
    let mut writes = vec![
        cache.prepare_entry(&job.job_id, &job)?.0,
        cache.prepare_entry(&result.result_id, &result)?.0,
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
                let role = cache.get::<EpiphanyRuntimeRoleWorkerResult>(&result.job_id)?;
                let reorient = cache.get::<EpiphanyRuntimeReorientWorkerResult>(&result.job_id)?;
                if role.is_some() && reorient.is_some() {
                    return Err(anyhow!(
                        "runtime worker job has both role and reorientation terminal outcomes"
                    ));
                }
                if let Some(terminal_result_id) = role
                    .map(|result| result.result_id)
                    .or_else(|| reorient.map(|result| result.result_id))
                {
                    terminal.status = crate::WorkerProcessStatus::TerminalResult.as_str().into();
                    terminal.terminal_authority_id = Some(terminal_result_id);
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

pub fn runtime_identity(store_path: impl AsRef<Path>) -> Result<Option<EpiphanyRuntimeIdentity>> {
    let store_path = store_path.as_ref();
    if !store_path.exists() {
        return Ok(None);
    }
    let mut cache = runtime_spine_cache(store_path)?;
    cache
        .pull_all_backing_stores()
        .with_context(|| format!("failed to read runtime spine {}", store_path.display()))?;
    cache.get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)
}

pub fn runtime_registered_document_types() -> Result<Vec<String>> {
    Ok(runtime_spine_schema_cache()?.registered_entry_types())
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
            archive.session_id
        ));
    }
    Ok(())
}

fn validate_non_empty(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{field} must be non-empty"));
    }
    Ok(())
}

fn worker_launch_document_kind(document: &EpiphanyWorkerLaunchDocument) -> &'static str {
    document.document_kind()
}

fn encode_worker_launch_document(document: &EpiphanyWorkerLaunchDocument) -> Result<Vec<u8>> {
    rmp_serde::to_vec_named(document).context("failed to encode worker launch document MessagePack")
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

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) fn bind_test_runtime_swarm(store: &Path, swarm_id: &str) -> Result<()> {
        bind_runtime_to_swarm(store, swarm_id, "2026-08-14T00:00:01Z")?;
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
    fn current_runtime_refuses_old_writable_epoch_without_mutation() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("historical.cc");
        let mut historical = CultCache::new();
        historical.register_entry_type::<crate::EpiphanyMindIdentity>()?;
        historical.register_entry_type::<EpiphanyRuntimeIdentity>()?;
        historical.add_generic_backing_store(runtime_spine_backing_store(&store)?);
        historical.put(
            "epiphany.mind.epoch.v1",
            &crate::EpiphanyMindIdentity {
                schema_epoch: "epiphany.mind.epoch.v1".into(),
                runtime_id: "historical-runtime".into(),
            },
        )?;
        historical.put(
            RUNTIME_IDENTITY_KEY,
            &EpiphanyRuntimeIdentity {
                schema_version: "epiphany.runtime_spine.v0".into(),
                runtime_id: "historical-runtime".into(),
                display_name: "Historical runtime".into(),
                created_at: "2026-08-17T00:00:00Z".into(),
            },
        )?;
        let before = runtime_spine_backing_store(&store)?.pull_all()?;
        assert!(runtime_spine_cache(&store).is_err());
        assert!(
            initialize_runtime_spine(
                &store,
                RuntimeSpineInitOptions {
                    runtime_id: "historical-runtime".into(),
                    display_name: "Must not migrate".into(),
                    created_at: "2026-08-18T00:00:00Z".into(),
                },
            )
            .is_err()
        );
        assert_eq!(runtime_spine_backing_store(&store)?.pull_all()?, before);

        let archive_v0_store = temp.path().join("archive-v0-runtime.cc");
        let mut archive_v0 = CultCache::new();
        archive_v0.register_entry_type::<crate::EpiphanyMindIdentity>()?;
        archive_v0.register_entry_type::<EpiphanyRuntimeIdentity>()?;
        archive_v0.add_generic_backing_store(runtime_spine_backing_store(&archive_v0_store)?);
        archive_v0.put(
            crate::MIND_SCHEMA_EPOCH,
            &crate::EpiphanyMindIdentity {
                schema_epoch: crate::MIND_SCHEMA_EPOCH.into(),
                runtime_id: "archive-v0-runtime".into(),
            },
        )?;
        archive_v0.put(
            RUNTIME_IDENTITY_KEY,
            &EpiphanyRuntimeIdentity {
                schema_version: "epiphany.runtime_spine.v1".into(),
                runtime_id: "archive-v0-runtime".into(),
                display_name: "Archive v0 runtime".into(),
                created_at: "2026-08-18T00:00:00Z".into(),
            },
        )?;
        let archive_v0_before = runtime_spine_backing_store(&archive_v0_store)?.pull_all()?;
        assert!(runtime_spine_cache(&archive_v0_store).is_err());
        assert_eq!(
            runtime_spine_backing_store(&archive_v0_store)?.pull_all()?,
            archive_v0_before
        );

        let responses_only_store = temp.path().join("responses-only-runtime.cc");
        let mut responses_only = CultCache::new();
        responses_only.register_entry_type::<crate::EpiphanyMindIdentity>()?;
        responses_only.register_entry_type::<EpiphanyRuntimeIdentity>()?;
        responses_only
            .add_generic_backing_store(runtime_spine_backing_store(&responses_only_store)?);
        responses_only.put(
            crate::MIND_SCHEMA_EPOCH,
            &crate::EpiphanyMindIdentity {
                schema_epoch: crate::MIND_SCHEMA_EPOCH.into(),
                runtime_id: "responses-only-runtime".into(),
            },
        )?;
        responses_only.put(
            RUNTIME_IDENTITY_KEY,
            &EpiphanyRuntimeIdentity {
                schema_version: "epiphany.runtime_spine.v2".into(),
                runtime_id: "responses-only-runtime".into(),
                display_name: "Responses-only runtime".into(),
                created_at: "2026-08-22T00:00:00Z".into(),
            },
        )?;
        let responses_only_before =
            runtime_spine_backing_store(&responses_only_store)?.pull_all()?;
        assert!(runtime_spine_cache(&responses_only_store).is_err());
        assert_eq!(
            runtime_spine_backing_store(&responses_only_store)?.pull_all()?,
            responses_only_before
        );
        Ok(())
    }
}
