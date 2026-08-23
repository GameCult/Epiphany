use epiphany_state_model::EpiphanyChurnState;
use epiphany_state_model::EpiphanyEvidenceRecord;
use epiphany_state_model::EpiphanyGraphCheckpoint;
use epiphany_state_model::EpiphanyGraphFrontier;
use epiphany_state_model::EpiphanyGraphs;
use epiphany_state_model::EpiphanyInvariant;
use epiphany_state_model::EpiphanyInvestigationCheckpoint;
use epiphany_state_model::EpiphanyObservation;
use epiphany_state_model::EpiphanyPlanningState;
use epiphany_state_model::EpiphanyScratchPad;
use epiphany_state_model::EpiphanySubgoal;
use serde::Deserialize;
use serde::Serialize;

pub const ROLE_WORKER_OUTPUT_CONTRACT_ID: &str = "epiphany.worker.role_result.v4";
pub const REORIENT_WORKER_OUTPUT_CONTRACT_ID: &str = "epiphany.worker.reorient_result.v0";
pub const REPO_FRONTIER_PROPOSAL_MODELING_CONTEXT_SCHEMA_VERSION: &str =
    "epiphany.worker.repo_frontier_proposal_modeling_context.v4";
pub const REPO_FRONTIER_PROPOSAL_MODELING_CONTEXT_CONTRACT: &str =
    "epiphany.repo_frontier_proposal_modeling_context.v4";
pub const REPO_FRONTIER_PLANNING_CONTEXT_SCHEMA_VERSION: &str =
    "epiphany.worker.repo_frontier_planning_context.v2";
pub const REPO_FRONTIER_PLANNING_CONTEXT_CONTRACT: &str =
    "epiphany.repo_frontier_planning_context.v2";
pub const REPO_FRONTIER_RESEARCH_CONTEXT_SCHEMA_VERSION: &str =
    "epiphany.worker.repo_frontier_research_context.v3";
pub const REPO_FRONTIER_RESEARCH_CONTEXT_CONTRACT: &str =
    "epiphany.repo_frontier_research_context.v3";
pub const REPO_FRONTIER_VERIFICATION_CONTEXT_SCHEMA_VERSION: &str =
    "epiphany.worker.repo_frontier_verification_context.v1";
pub const REPO_FRONTIER_VERIFICATION_CONTEXT_CONTRACT: &str =
    "epiphany.repo_frontier_verification_context.v1";
pub const REPO_FRONTIER_PLAN_MIND_CONTEXT_SCHEMA_VERSION: &str =
    "epiphany.worker.repo_frontier_plan_mind_context.v0";
pub const REPO_FRONTIER_PLAN_MIND_CONTEXT_CONTRACT: &str =
    "epiphany.repo_frontier_plan_mind_context.v0";
pub const IMAGINATION_CONSIDERATION_CONTEXT_SCHEMA_VERSION: &str =
    "epiphany.worker.imagination_consideration_context.v0";
pub const IMAGINATION_CONSIDERATION_CONTEXT_CONTRACT: &str =
    "epiphany.imagination_consideration_context.v0";
pub const ADMITTED_MODEL_DIRECTION_CONSIDERATION_CONTEXT_SCHEMA_VERSION: &str =
    "epiphany.worker.admitted_model_direction_consideration_context.v0";
pub const ADMITTED_MODEL_DIRECTION_CONSIDERATION_CONTEXT_CONTRACT: &str =
    "epiphany.admitted_model_direction_consideration_context.v0";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "documentKind")]
pub enum EpiphanyWorkerLaunchDocument {
    Role(EpiphanyRoleWorkerLaunchDocument),
    Reorient(EpiphanyReorientWorkerLaunchDocument),
}

impl EpiphanyWorkerLaunchDocument {
    pub fn output_contract_id(&self) -> &'static str {
        match self {
            Self::Role(_) => ROLE_WORKER_OUTPUT_CONTRACT_ID,
            Self::Reorient(_) => REORIENT_WORKER_OUTPUT_CONTRACT_ID,
        }
    }

    pub fn document_kind(&self) -> &'static str {
        match self {
            Self::Role(_) => "role",
            Self::Reorient(_) => "reorient",
        }
    }

    pub fn thread_id(&self) -> &str {
        match self {
            Self::Role(document) => &document.thread_id,
            Self::Reorient(document) => &document.creation_thread_id,
        }
    }

    pub fn dynamic_prompt_context(&self) -> Option<&str> {
        match self {
            Self::Role(document) => document.dynamic_prompt_context.as_deref(),
            Self::Reorient(_) => None,
        }
        .map(str::trim)
        .filter(|context| !context.is_empty())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyRoleWorkerLaunchDocument {
    pub thread_id: String,
    pub role_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub objective: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamic_prompt_context: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repository_body_observation_basis: Option<crate::RepositoryBodyObservationBasis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proposal_modeling_context: Option<RepoFrontierProposalModelingContextProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frontier_verdict_modeling_context:
        Option<crate::RepoFrontierVerdictModelingLaunchAuthority>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frontier_planning_context: Option<RepoFrontierPlanningContextProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frontier_research_context: Option<RepoFrontierResearchContextProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frontier_verification_context: Option<RepoFrontierVerificationContextProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frontier_plan_mind_context: Option<RepoFrontierPlanMindContextProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub imagination_consideration_context: Option<ImaginationConsiderationContextProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub admitted_model_direction_consideration_context:
        Option<AdmittedModelDirectionConsiderationContextProjection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_subgoal_id: Option<String>,
    #[serde(default)]
    pub active_subgoals: Vec<EpiphanySubgoal>,
    #[serde(default)]
    pub active_graph_node_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub investigation_checkpoint: Option<EpiphanyInvestigationCheckpoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scratch: Option<EpiphanyScratchPad>,
    #[serde(default)]
    pub invariants: Vec<EpiphanyInvariant>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graphs: Option<EpiphanyGraphs>,
    #[serde(default)]
    pub recent_evidence: Vec<EpiphanyEvidenceRecord>,
    #[serde(default)]
    pub recent_observations: Vec<EpiphanyObservation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_frontier: Option<EpiphanyGraphFrontier>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_checkpoint: Option<EpiphanyGraphCheckpoint>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub planning: Option<EpiphanyPlanningState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub churn: Option<EpiphanyChurnState>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoFrontierResearchContextProjection {
    pub schema_version: String,
    pub request_id: String,
    pub model_projection_digest: String,
    pub model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    pub frontier_authority_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    pub frontier_item_id: String,
    pub frontier_item_hash: String,
    pub repository_scope: Vec<String>,
    #[serde(default)]
    pub public_source_refs: Vec<String>,
    pub contract: String,
}

impl From<&crate::RepoFrontierResearchRequest> for RepoFrontierResearchContextProjection {
    fn from(request: &crate::RepoFrontierResearchRequest) -> Self {
        Self {
            schema_version: REPO_FRONTIER_RESEARCH_CONTEXT_SCHEMA_VERSION.to_string(),
            request_id: request.request_id.clone(),
            model_projection_digest: request.model_projection_digest.clone(),
            model_source_documents: request.model_source_documents.clone(),
            frontier_authority_documents: request.frontier_authority_documents.clone(),
            frontier_item_id: request.frontier_item_id.clone(),
            frontier_item_hash: request.frontier_item_hash.clone(),
            repository_scope: request.repository_scope.clone(),
            public_source_refs: request.public_source_refs.clone(),
            contract: REPO_FRONTIER_RESEARCH_CONTEXT_CONTRACT.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyReorientWorkerLaunchDocument {
    pub schema_version: String,
    pub request_id: String,
    pub creation_thread_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoFrontierProposalModelingContextProjection {
    pub schema_version: String,
    pub contract: String,
    pub request_id: String,
    pub proposal_id: String,
    pub proposal_payload_sha256: String,
    pub runtime_id: String,
    pub thread_id: String,
    pub repository: String,
    pub workspace: String,
    pub title: String,
    pub body: String,
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    pub model_projection_digest: String,
    pub model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    pub prior_admission_refusals: Vec<crate::EpiphanyAgentPassAdmissionRefusal>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoFrontierPlanningContextProjection {
    pub schema_version: String,
    pub contract: String,
    pub request_id: String,
    pub model_projection_digest: String,
    pub model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    pub frontier_item_id: String,
    pub frontier_item_hash: String,
    pub selected_organ: String,
    #[serde(default)]
    pub repository_scope: Vec<String>,
    pub requested_at: String,
    pub runtime_id: String,
    pub frontier_authority_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    pub claim_obligation_documents: Vec<crate::EpiphanyMindDocumentVersion>,
}

impl RepoFrontierPlanningContextProjection {
    pub(crate) fn from_request(request: &crate::RepoFrontierPlanningRequest) -> Self {
        Self {
            schema_version: REPO_FRONTIER_PLANNING_CONTEXT_SCHEMA_VERSION.into(),
            contract: REPO_FRONTIER_PLANNING_CONTEXT_CONTRACT.into(),
            request_id: request.request_id.clone(),
            model_projection_digest: request.model_projection_digest.clone(),
            model_source_documents: request.model_source_documents.clone(),
            frontier_item_id: request.frontier_item_id.clone(),
            frontier_item_hash: request.frontier_item_hash.clone(),
            selected_organ: request.selected_organ.clone(),
            repository_scope: request.repository_scope.clone(),
            requested_at: request.requested_at.clone(),
            runtime_id: request.runtime_id.clone(),
            frontier_authority_documents: request.frontier_authority_documents.clone(),
            claim_obligation_documents: request.claim_obligation_documents.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImaginationConsiderationContextProjection {
    pub schema_version: String,
    pub contract: String,
    pub request: crate::ImaginationConsiderationRequest,
    pub model: crate::EpiphanyRepoModelView,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoFrontierVerificationContextProjection {
    pub schema_version: String,
    pub request: crate::RepoFrontierVerificationRequest,
    pub route: crate::RepoFrontierRoute,
    pub hands_authority: crate::RepoFrontierHandsAuthority,
    pub hands_intent: crate::HandsActionIntent,
    pub hands_review: crate::HandsActionReview,
    pub patch_receipt: crate::HandsPatchReceipt,
    pub command_receipt: crate::HandsCommandReceipt,
    pub commit_receipt: crate::HandsCommitReceipt,
    pub contract: String,
}

impl ImaginationConsiderationContextProjection {
    pub(crate) fn new(
        request: &crate::ImaginationConsiderationRequest,
        model: &crate::EpiphanyRepoModelView,
    ) -> Self {
        Self {
            schema_version: IMAGINATION_CONSIDERATION_CONTEXT_SCHEMA_VERSION.into(),
            contract: IMAGINATION_CONSIDERATION_CONTEXT_CONTRACT.into(),
            request: request.clone(),
            model: model.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdmittedModelDirectionConsiderationContextProjection {
    pub schema_version: String,
    pub contract: String,
    pub request: crate::AdmittedModelDirectionConsiderationRequest,
    pub model: crate::EpiphanyRepoModelView,
}

impl AdmittedModelDirectionConsiderationContextProjection {
    pub(crate) fn new(
        request: &crate::AdmittedModelDirectionConsiderationRequest,
        model: &crate::EpiphanyRepoModelView,
    ) -> Self {
        Self {
            schema_version: ADMITTED_MODEL_DIRECTION_CONSIDERATION_CONTEXT_SCHEMA_VERSION.into(),
            contract: ADMITTED_MODEL_DIRECTION_CONSIDERATION_CONTEXT_CONTRACT.into(),
            request: request.clone(),
            model: model.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoFrontierPlanMindContextProjection {
    pub schema_version: String,
    pub contract: String,
    pub request: crate::RepoFrontierPlanMindRequest,
    pub planning_request: crate::RepoFrontierPlanningRequest,
    pub candidate: crate::RepoFrontierPlanCandidate,
}

impl RepoFrontierPlanMindContextProjection {
    pub(crate) fn new(
        request: &crate::RepoFrontierPlanMindRequest,
        planning_request: &crate::RepoFrontierPlanningRequest,
        candidate: &crate::RepoFrontierPlanCandidate,
    ) -> Self {
        Self {
            schema_version: REPO_FRONTIER_PLAN_MIND_CONTEXT_SCHEMA_VERSION.into(),
            contract: REPO_FRONTIER_PLAN_MIND_CONTEXT_CONTRACT.into(),
            request: request.clone(),
            planning_request: planning_request.clone(),
            candidate: candidate.clone(),
        }
    }
}
