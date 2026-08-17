//! Typed repository-frontier authority chain.
//!
//! The minimal admitted path is deliberately linear:
//!
//! 1. `RepoFrontierWorkProposal` is inert evidence. Modeling may turn it into
//!    an admitted RepoModel frontier item; the proposal itself grants nothing.
//! 2. Self selects exactly one eligible Imagination item and commits a
//!    `RepoFrontierPlanningRequest`. Imagination returns a
//!    `RepoFrontierPlanCandidate` whose safe paths may only narrow the item's
//!    source scope.
//! 3. Self commits a `RepoFrontierPlanMindRequest`. Mind alone adopts, refuses,
//!    or holds the candidate through `RepoFrontierPlanDecisionReceipt`.
//! 4. Only an adopted decision may be embedded in `RepoFrontierRoute` by
//!    `runtime_spine::select_and_commit_repo_frontier_route`. Route selection
//!    binds the current admitted model revision, admission receipt, frontier
//!    hash, and source scope in one compare-and-swap operation.
//! 5. Hands remains powerless until the coordinator pairs that route with a
//!    reviewed Hands intent, Substrate Gate receipt, and
//!    `RepoFrontierHandsAuthority`. Hands receipts describe consequences; they
//!    do not admit durable Mind state.
//! 6. Soul verifies the exact route and consequence. Coordinator acceptance
//!    applies reviewed state effects through the atomic coordinator state
//!    transaction; no worker, Hands receipt, or status projection is a second
//!    admission owner.
//! 7. Hands refusal terminates in `RepoFrontierRelinquishmentReceipt`, while
//!    Continuity receipts preserve recovery facts. `epiphany-mvp-status` and
//!    CultMesh surfaces only project these admitted outcomes.
//!
//! If a required identity or authority is absent at any step, the chain stops.
//! Do not infer it from display state, repair it after execution, or invent a
//! parallel route document.

use anyhow::Result;
use cultcache_rs::DatabaseEntry;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const REPO_FRONTIER_ROUTE_TYPE: &str = "epiphany.self.repo_frontier_route";
pub const REPO_FRONTIER_ROUTE_SCHEMA_VERSION: &str = "epiphany.self.repo_frontier_route.v2";
pub const REPO_FRONTIER_HANDS_AUTHORITY_TYPE: &str = "epiphany.hands.repo_frontier_authority";
pub const REPO_FRONTIER_HANDS_AUTHORITY_SCHEMA_VERSION: &str =
    "epiphany.hands.repo_frontier_authority.v0";
pub const REPO_FRONTIER_ROUTE_CONTRACT: &str = "epiphany.repo_frontier_route.v2";
pub const REPO_FRONTIER_HANDS_AUTHORITY_CONTRACT: &str =
    "epiphany.repo_frontier_hands_authority.v0";
pub const REPO_FRONTIER_RELINQUISHMENT_RECEIPT_TYPE: &str =
    "epiphany.mind.repo_frontier_relinquishment_receipt";
pub const REPO_FRONTIER_RELINQUISHMENT_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.mind.repo_frontier_relinquishment_receipt.v0";
pub const REPO_FRONTIER_RELINQUISHMENT_RECEIPT_CONTRACT: &str =
    "epiphany.repo_frontier_relinquishment.v0";
pub const REPO_FRONTIER_EXECUTION_AMENDMENT_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.mind.repo_frontier_execution_amendment_receipt.v0";
pub const REPO_FRONTIER_EXECUTION_AMENDMENT_RECEIPT_CONTRACT: &str =
    "epiphany.repo_frontier_execution_amendment.v0";
pub const REPO_FRONTIER_MODELING_REQUEST_TYPE: &str =
    "epiphany.modeling.repo_frontier_verdict_request";
pub const REPO_FRONTIER_MODELING_REQUEST_SCHEMA_VERSION: &str =
    "epiphany.modeling.repo_frontier_verdict_request.v1";
pub const REPO_FRONTIER_MODELING_REQUEST_CONTRACT: &str =
    "epiphany.repo_frontier_verdict_modeling_request.v1";
pub const REPO_FRONTIER_WORK_PROPOSAL_SCHEMA_VERSION: &str =
    "epiphany.repo_frontier_work_proposal.v0";
pub const REPO_FRONTIER_PLANNING_REQUEST_SCHEMA_VERSION: &str =
    "epiphany.self.repo_frontier_planning_request.v2";
pub const REPO_FRONTIER_PLAN_CANDIDATE_SCHEMA_VERSION: &str =
    "epiphany.imagination.repo_frontier_plan_candidate.v0";
pub const REPO_FRONTIER_PLAN_DECISION_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.mind.repo_frontier_plan_decision_receipt.v1";
pub const LEGACY_REPO_FRONTIER_PLAN_DECISION_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.mind.repo_frontier_plan_decision_receipt.v0";
pub const REPO_FRONTIER_PLAN_DECISION_CONTRACT: &str = "epiphany.repo_frontier_plan_decision.v0";
pub const REPO_FRONTIER_PLANNING_CONTRACT: &str = "epiphany.repo_frontier_planning.v2";
pub const REPO_FRONTIER_RESEARCH_REQUEST_SCHEMA_VERSION: &str =
    "epiphany.self.repo_frontier_research_request.v3";
pub const REPO_FRONTIER_RESEARCH_REQUEST_CONTRACT: &str =
    "epiphany.repo_frontier_research_request.v3";
pub const REPO_FRONTIER_WORK_PROPOSAL_CONTRACT: &str =
    "epiphany.repo_frontier_work_proposal.inert.v0";
pub const REPO_FRONTIER_AUTONOMOUS_PROPOSAL_BINDING_SCHEMA_VERSION: &str =
    "epiphany.self.repo_frontier_autonomous_proposal_binding.v1";
pub const REPO_FRONTIER_AUTONOMOUS_PROPOSAL_BINDING_CONTRACT: &str =
    "epiphany.repo_frontier_autonomous_proposal_binding.v1";
pub const RUNTIME_REPOSITORY_DOMAIN_BINDING_KEY: &str = "runtime-repository-domain-binding";
pub const RUNTIME_REPOSITORY_DOMAIN_BINDING_SCHEMA_VERSION: &str =
    "epiphany.runtime.repository_domain_binding.v0";
pub const RUNTIME_REPOSITORY_DOMAIN_BINDING_CONTRACT: &str = "deployment configuration binds one organizational repository name to one exact authenticated repository Body; Self may consume but not relabel it";
pub const REPO_FRONTIER_PROPOSAL_MODELING_REQUEST_SCHEMA_VERSION: &str =
    "epiphany.coordinator.repo_frontier_proposal_modeling_request.v0";
pub const REPO_FRONTIER_PROPOSAL_MODELING_REQUEST_CONTRACT: &str =
    "epiphany.repo_frontier_proposal_modeling_request.v0";
pub const REPO_FRONTIER_PROPOSAL_MODELING_LAUNCH_BINDING_SCHEMA_VERSION: &str =
    "epiphany.coordinator.repo_frontier_proposal_modeling_launch_binding.v1";
pub const REPO_FRONTIER_PROPOSAL_MODELING_LAUNCH_BINDING_CONTRACT: &str =
    "epiphany.repo_frontier_proposal_modeling_launch_binding.v1";
pub const REPO_MODEL_CLAIM_CHALLENGE_SCHEMA_VERSION: &str =
    "epiphany.eyes.repo_model_claim_challenge.v0";
pub const REPO_MODEL_CLAIM_CHALLENGE_CONTRACT: &str = "epiphany.repo_model_claim_challenge.v0";
pub const REPO_FRONTIER_PLANNING_LAUNCH_BINDING_SCHEMA_VERSION: &str =
    "epiphany.coordinator.repo_frontier_planning_launch_binding.v0";
pub const REPO_FRONTIER_PLANNING_LAUNCH_BINDING_CONTRACT: &str =
    "epiphany.repo_frontier_planning_launch_binding.v0";
pub const REPO_FRONTIER_PLAN_MIND_REQUEST_SCHEMA_VERSION: &str =
    "epiphany.self.repo_frontier_plan_mind_request.v0";
pub const REPO_FRONTIER_PLAN_MIND_REQUEST_CONTRACT: &str =
    "epiphany.repo_frontier_plan_mind_request.v0";
pub const REPO_FRONTIER_PLAN_MIND_LAUNCH_BINDING_SCHEMA_VERSION: &str =
    "epiphany.coordinator.repo_frontier_plan_mind_launch_binding.v0";
pub const REPO_FRONTIER_PLAN_MIND_LAUNCH_BINDING_CONTRACT: &str =
    "epiphany.repo_frontier_plan_mind_launch_binding.v0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoModelClaimChallengeDisposition {
    Contradicted,
    Stale,
    EvidenceInsufficient,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.eyes.repo_model_claim_challenge",
    schema = "RepoModelClaimChallenge"
)]
pub struct RepoModelClaimChallenge {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub challenge_id: String,
    #[cultcache(key = 2)]
    pub eyes_evidence_packet_id: String,
    #[cultcache(key = 3)]
    pub eyes_evidence_packet_sha256: String,
    #[cultcache(key = 4)]
    pub source_result_id: String,
    #[cultcache(key = 5)]
    pub source_job_id: String,
    #[cultcache(key = 6)]
    pub model_projection_digest: String,
    #[cultcache(key = 7)]
    pub model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    #[cultcache(key = 8)]
    pub target_claim_id: String,
    #[cultcache(key = 9)]
    pub target_claim_sha256: String,
    #[cultcache(key = 10)]
    pub disposition: RepoModelClaimChallengeDisposition,
    #[cultcache(key = 11)]
    pub finding: String,
    #[cultcache(key = 12)]
    pub uncertainty: String,
    #[cultcache(key = 13)]
    pub source_refs: Vec<String>,
    #[cultcache(key = 14)]
    pub evidence_ids: Vec<String>,
    #[cultcache(key = 15)]
    pub challenged_at: String,
    #[cultcache(key = 16)]
    pub contract: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoFrontierProposalSourceKind {
    User,
    Persona,
    Bifrost,
    Imagination,
}

#[derive(Debug, Clone)]
pub struct RepoFrontierUserProposalInput {
    pub proposal_id: String,
    pub source_actor: String,
    pub source_ref: String,
    pub repository: String,
    pub workspace: String,
    pub thread_id: String,
    pub runtime_id: String,
    pub title: String,
    pub body: String,
    pub desired_outcome: String,
    pub constraints: Vec<String>,
    pub scope_hints: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub public_source_refs: Vec<String>,
    pub proposed_at: String,
    pub private_state_included: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.evidence.repo_frontier_work_proposal",
    schema = "RepoFrontierWorkProposal"
)]
pub struct RepoFrontierWorkProposal {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub proposal_id: String,
    #[cultcache(key = 2)]
    pub source_kind: RepoFrontierProposalSourceKind,
    #[cultcache(key = 3)]
    pub source_actor: String,
    #[cultcache(key = 4)]
    pub source_ref: String,
    #[cultcache(key = 5)]
    pub repository: String,
    #[cultcache(key = 6)]
    pub workspace: String,
    #[cultcache(key = 7)]
    pub thread_id: String,
    #[cultcache(key = 8)]
    pub runtime_id: String,
    #[cultcache(key = 9)]
    pub payload_sha256: String,
    #[cultcache(key = 10)]
    pub title: String,
    #[cultcache(key = 11)]
    pub body: String,
    #[cultcache(key = 12)]
    pub desired_outcome: String,
    #[cultcache(key = 13)]
    pub constraints: Vec<String>,
    #[cultcache(key = 14)]
    pub scope_hints: Vec<String>,
    #[cultcache(key = 15)]
    pub evidence_refs: Vec<String>,
    #[cultcache(key = 16)]
    pub private_state_included: bool,
    #[cultcache(key = 17)]
    pub proposed_at: String,
    #[cultcache(key = 18)]
    pub contract: String,
    #[cultcache(key = 19, default)]
    pub public_source_refs: Vec<String>,
}

pub fn repo_frontier_proposal_payload_sha256(
    title: &str,
    body: &str,
    desired_outcome: &str,
    constraints: &[String],
    scope_hints: &[String],
    evidence_refs: &[String],
    public_source_refs: &[String],
) -> Result<String> {
    let content = if public_source_refs.is_empty() {
        // Preserve the exact identity of pre-public-source v0 proposals.
        rmp_serde::to_vec_named(&(
            title,
            body,
            desired_outcome,
            constraints,
            scope_hints,
            evidence_refs,
        ))?
    } else {
        rmp_serde::to_vec_named(&(
            title,
            body,
            desired_outcome,
            constraints,
            scope_hints,
            evidence_refs,
            public_source_refs,
        ))?
    };
    Ok(format!("{:x}", Sha256::digest(content)))
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.self.repo_frontier_autonomous_proposal_binding",
    schema = "RepoFrontierAutonomousProposalBinding"
)]
pub struct RepoFrontierAutonomousProposalBinding {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub binding_id: String,
    #[cultcache(key = 2)]
    pub proposal_id: String,
    #[cultcache(key = 3)]
    pub proposal_payload_sha256: String,
    #[cultcache(key = 4)]
    pub direction_request_id: String,
    #[cultcache(key = 5)]
    pub direction_result_id: String,
    #[cultcache(key = 6)]
    pub direction_result_sha256: String,
    #[cultcache(key = 7)]
    pub model_projection_digest: String,
    #[cultcache(key = 8)]
    pub model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    #[cultcache(key = 10)]
    pub option_ordinal: u32,
    #[cultcache(key = 11)]
    pub option_sha256: String,
    #[cultcache(key = 12)]
    pub runtime_id: String,
    #[cultcache(key = 13)]
    pub thread_id: String,
    #[cultcache(key = 14)]
    pub workspace_id: String,
    #[cultcache(key = 15)]
    pub body_binding_sha256: String,
    #[cultcache(key = 16)]
    pub created_at: String,
    #[cultcache(key = 17)]
    pub contract: String,
    #[cultcache(key = 18)]
    pub direction_worker_job_id: String,
    #[cultcache(key = 19)]
    pub direction_worker_result_id: String,
    #[cultcache(key = 20)]
    pub direction_worker_result_sha256: String,
    #[cultcache(key = 21)]
    pub direction_worker_launch_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.repository_domain_binding",
    schema = "RuntimeRepositoryDomainBinding"
)]
pub struct RuntimeRepositoryDomainBinding {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub binding_id: String,
    #[cultcache(key = 2)]
    pub repository_full_name: String,
    #[cultcache(key = 3)]
    pub runtime_id: String,
    #[cultcache(key = 4)]
    pub swarm_id: String,
    #[cultcache(key = 5)]
    pub workspace_id: String,
    #[cultcache(key = 6)]
    pub body_binding_sha256: String,
    #[cultcache(key = 7)]
    pub bound_at: String,
    #[cultcache(key = 8)]
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.coordinator.repo_frontier_proposal_modeling_request",
    schema = "RepoFrontierProposalModelingRequest"
)]
pub struct RepoFrontierProposalModelingRequest {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub proposal_id: String,
    #[cultcache(key = 3)]
    pub proposal_payload_sha256: String,
    #[cultcache(key = 4)]
    pub runtime_id: String,
    #[cultcache(key = 5)]
    pub thread_id: String,
    #[cultcache(key = 6)]
    pub repository: String,
    #[cultcache(key = 7)]
    pub workspace: String,
    #[cultcache(key = 8)]
    pub selected_at: String,
    #[cultcache(key = 9)]
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.coordinator.repo_frontier_proposal_modeling_launch_binding",
    schema = "RepoFrontierProposalModelingLaunchBinding"
)]
pub struct RepoFrontierProposalModelingLaunchBinding {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub binding_record_id: String,
    #[cultcache(key = 2)]
    pub proposal_modeling_request_id: String,
    #[cultcache(key = 3)]
    pub proposal_id: String,
    #[cultcache(key = 4)]
    pub proposal_payload_sha256: String,
    #[cultcache(key = 5)]
    pub job_id: String,
    #[cultcache(key = 6)]
    pub binding_id: String,
    #[cultcache(key = 7)]
    pub runtime_id: String,
    #[cultcache(key = 8)]
    pub thread_id: String,
    #[cultcache(key = 9)]
    pub launched_at: String,
    #[cultcache(key = 10)]
    pub worker_launch_document_sha256: String,
    #[cultcache(key = 11)]
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.self.repo_frontier_planning_request",
    schema = "RepoFrontierPlanningRequest"
)]
pub struct RepoFrontierPlanningRequest {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub model_projection_digest: String,
    #[cultcache(key = 3)]
    pub model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    #[cultcache(key = 4)]
    pub frontier_item_id: String,
    #[cultcache(key = 5)]
    pub frontier_item_hash: String,
    #[cultcache(key = 6)]
    pub selected_organ: String,
    #[cultcache(key = 7)]
    pub source_scope: Vec<String>,
    #[cultcache(key = 8)]
    pub requested_at: String,
    #[cultcache(key = 9)]
    pub contract: String,
    #[cultcache(key = 10)]
    pub runtime_id: String,
    #[cultcache(key = 11)]
    pub thread_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.coordinator.repo_frontier_planning_launch_binding",
    schema = "RepoFrontierPlanningLaunchBinding"
)]
pub struct RepoFrontierPlanningLaunchBinding {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub binding_record_id: String,
    #[cultcache(key = 2)]
    pub planning_request_id: String,
    #[cultcache(key = 3)]
    pub job_id: String,
    #[cultcache(key = 4)]
    pub binding_id: String,
    #[cultcache(key = 5)]
    pub runtime_id: String,
    #[cultcache(key = 6)]
    pub thread_id: String,
    #[cultcache(key = 7)]
    pub launched_at: String,
    #[cultcache(key = 8)]
    pub worker_launch_document_sha256: String,
    #[cultcache(key = 9)]
    pub contract: String,
    #[cultcache(key = 10, default)]
    pub attempt_ordinal: u64,
    #[cultcache(key = 11, default)]
    pub superseded_failure_result_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.self.repo_frontier_research_request",
    schema = "RepoFrontierResearchRequest"
)]
pub struct RepoFrontierResearchRequest {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub model_projection_digest: String,
    #[cultcache(key = 3)]
    pub model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    #[cultcache(key = 4)]
    pub frontier_item_id: String,
    #[cultcache(key = 5)]
    pub frontier_item_hash: String,
    #[cultcache(key = 6)]
    pub source_scope: Vec<String>,
    #[cultcache(key = 7)]
    pub requested_at: String,
    #[cultcache(key = 8)]
    pub runtime_id: String,
    #[cultcache(key = 10)]
    pub contract: String,
    /// Immutable public source identities already owned by the admitted
    /// frontier. This is dedicated retrieval authority, not worker-authored
    /// evidence or a search hint.
    #[cultcache(key = 11, default)]
    pub public_source_refs: Vec<String>,
    /// Exact current documents that own whether this work remains actionable.
    /// The complete model sources above are immutable audit cargo; unrelated
    /// keyed Mind changes must not stale this request.
    #[cultcache(key = 12)]
    pub frontier_authority_documents: Vec<crate::EpiphanyMindDocumentVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.imagination.repo_frontier_plan_candidate",
    schema = "RepoFrontierPlanCandidate"
)]
pub struct RepoFrontierPlanCandidate {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub candidate_id: String,
    #[cultcache(key = 2)]
    pub planning_request_id: String,
    #[cultcache(key = 3)]
    pub model_projection_digest: String,
    #[cultcache(key = 4)]
    pub model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    #[cultcache(key = 5)]
    pub frontier_item_id: String,
    #[cultcache(key = 6)]
    pub frontier_item_hash: String,
    #[cultcache(key = 7)]
    pub safe_paths: Vec<String>,
    #[cultcache(key = 8)]
    pub action: String,
    #[cultcache(key = 9)]
    pub command: String,
    #[cultcache(key = 10)]
    pub checks: Vec<String>,
    #[cultcache(key = 11)]
    pub stop_conditions: Vec<String>,
    #[cultcache(key = 12)]
    pub rollback_steps: Vec<String>,
    #[cultcache(key = 13)]
    pub commit_message: String,
    #[cultcache(key = 14)]
    pub proposed_at: String,
    #[cultcache(key = 15)]
    pub contract: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoFrontierPlanDecision {
    Adopt,
    Refuse,
    Hold,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum RepoFrontierPlanningLifecycleStage {
    Unavailable,
    Ready,
    ImaginationLaunchReady,
    ImaginationRunning,
    ImaginationFailed,
    ImaginationResultReady,
    MindLaunchReady,
    MindRunning,
    MindFailed,
    MindResultReady,
    Terminal,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoFrontierPlanningLifecycle {
    pub stage: RepoFrontierPlanningLifecycleStage,
    pub planning_request_id: Option<String>,
    pub imagination_job_id: Option<String>,
    pub imagination_result_id: Option<String>,
    pub mind_request_id: Option<String>,
    pub mind_job_id: Option<String>,
    pub mind_result_id: Option<String>,
    pub decision_id: Option<String>,
}

/// Read-only Self projection explaining why an unresolved Imagination frontier
/// is or is not eligible for the canonical planning-request commit primitive.
/// This carries no planning authority and persists no substitute state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoFrontierPlanningCandidateEligibility {
    pub frontier_item_id: String,
    pub eligible: bool,
    pub status_valid: bool,
    pub recommended_next_organ_valid: bool,
    pub source_scope_valid: bool,
    pub challenged_target_claim_ids: Vec<String>,
    pub unresolved_dependency_item_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoFrontierPlanningEligibility {
    pub current_model_count: usize,
    pub candidates: Vec<RepoFrontierPlanningCandidateEligibility>,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.self.repo_frontier_plan_mind_request",
    schema = "RepoFrontierPlanMindRequest"
)]
pub struct RepoFrontierPlanMindRequest {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub planning_request_id: String,
    #[cultcache(key = 3)]
    pub imagination_result_id: String,
    #[cultcache(key = 4)]
    pub imagination_job_id: String,
    #[cultcache(key = 5)]
    pub candidate_id: String,
    #[cultcache(key = 6)]
    pub candidate_sha256: String,
    #[cultcache(key = 7)]
    pub runtime_id: String,
    #[cultcache(key = 8)]
    pub thread_id: String,
    #[cultcache(key = 9)]
    pub requested_at: String,
    #[cultcache(key = 10)]
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.coordinator.repo_frontier_plan_mind_launch_binding",
    schema = "RepoFrontierPlanMindLaunchBinding"
)]
pub struct RepoFrontierPlanMindLaunchBinding {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub binding_record_id: String,
    #[cultcache(key = 2)]
    pub mind_request_id: String,
    #[cultcache(key = 3)]
    pub job_id: String,
    #[cultcache(key = 4)]
    pub binding_id: String,
    #[cultcache(key = 5)]
    pub runtime_id: String,
    #[cultcache(key = 6)]
    pub thread_id: String,
    #[cultcache(key = 7)]
    pub launched_at: String,
    #[cultcache(key = 8)]
    pub worker_launch_document_sha256: String,
    #[cultcache(key = 9)]
    pub contract: String,
    #[cultcache(key = 10, default)]
    pub attempt_ordinal: u64,
    #[cultcache(key = 11, default)]
    pub superseded_failure_result_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoFrontierPlanMindDecision {
    pub mind_request_id: String,
    pub planning_request_id: String,
    pub imagination_result_id: String,
    pub candidate_id: String,
    pub candidate_sha256: String,
    pub decision: RepoFrontierPlanDecision,
    pub rationale: String,
    pub decided_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.repo_frontier_plan_decision_receipt",
    schema = "RepoFrontierPlanDecisionReceipt"
)]
pub struct RepoFrontierPlanDecisionReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub decision_id: String,
    #[cultcache(key = 2)]
    pub planning_request_id: String,
    #[cultcache(key = 3)]
    pub legacy_mind_worker_result_id: Option<String>,
    #[cultcache(key = 4)]
    pub legacy_mind_worker_job_id: Option<String>,
    #[cultcache(key = 5)]
    pub candidate_id: String,
    #[cultcache(key = 6)]
    pub candidate_sha256: String,
    #[cultcache(key = 7)]
    pub model_projection_digest: String,
    #[cultcache(key = 8)]
    pub model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    #[cultcache(key = 9)]
    pub frontier_item_id: String,
    #[cultcache(key = 10)]
    pub frontier_item_hash: String,
    #[cultcache(key = 11)]
    pub decision: RepoFrontierPlanDecision,
    #[cultcache(key = 12)]
    pub rationale: String,
    #[cultcache(key = 13)]
    pub decided_at: String,
    #[cultcache(key = 14)]
    pub contract: String,
    #[cultcache(key = 15, default)]
    pub decision_source: Option<RepoFrontierPlanDecisionSource>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RepoFrontierPlanDecisionSource {
    MindWorker {
        result_id: String,
        job_id: String,
    },
    AuthenticatedOperatorReview {
        command_id: String,
        admission_id: String,
        packet_sha256: String,
        source_actor_id: String,
    },
}

/// Operator-safe identity projection of one current Mind review candidate.
/// Proposal text, commands, paths, and private state deliberately remain in
/// the canonical runtime store owned by Mind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepoFrontierPlanReviewSummary {
    pub mind_request_id: String,
    pub candidate_id: String,
    pub candidate_sha256: String,
    pub model_projection_digest: String,
    pub model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    pub frontier_item_id: String,
    pub requested_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RepoFrontierPlanOperatorReview {
    pub command_id: String,
    pub admission_id: String,
    pub packet_sha256: String,
    pub source_actor_id: String,
    pub mind_request_id: String,
    pub candidate_id: String,
    pub candidate_sha256: String,
    pub expected_model_projection_digest: String,
    pub expected_model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    pub decision: RepoFrontierPlanDecision,
    pub decided_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoFrontierVerdictDisposition {
    Resolved,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.modeling.repo_frontier_verdict_request",
    schema = "RepoFrontierModelingRequest"
)]
pub struct RepoFrontierModelingRequest {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub model_projection_digest: String,
    #[cultcache(key = 3)]
    pub model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    #[cultcache(key = 4)]
    pub route_id: String,
    #[cultcache(key = 5)]
    pub frontier_item_id: String,
    #[cultcache(key = 6)]
    pub frontier_item_hash: String,
    #[cultcache(key = 7)]
    pub verification_request_id: String,
    #[cultcache(key = 8)]
    pub soul_verdict_receipt_id: String,
    #[cultcache(key = 9)]
    pub verification_result_id: String,
    #[cultcache(key = 10)]
    pub verification_job_id: String,
    #[cultcache(key = 11)]
    pub allowed_disposition: RepoFrontierVerdictDisposition,
    #[cultcache(key = 12)]
    pub requested_at: String,
    #[cultcache(key = 13)]
    pub contract: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoFrontierNextOrgan {
    Hands,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.self.repo_frontier_route",
    schema = "RepoFrontierRoute"
)]
pub struct RepoFrontierRoute {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub route_id: String,
    #[cultcache(key = 2)]
    pub next_organ: RepoFrontierNextOrgan,
    #[cultcache(key = 3)]
    pub model_projection_digest: String,
    #[cultcache(key = 4)]
    pub model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    #[cultcache(key = 5)]
    pub frontier_item_id: String,
    #[cultcache(key = 6)]
    pub frontier_item_hash: String,
    #[cultcache(key = 7)]
    pub migration_body: String,
    #[cultcache(key = 8)]
    pub question: String,
    #[cultcache(key = 9)]
    pub gap: String,
    #[cultcache(key = 10)]
    pub target_claim_ids: Vec<String>,
    #[cultcache(key = 11)]
    pub source_scope: Vec<String>,
    #[cultcache(key = 12, default)]
    pub adopted_plan: Option<epiphany_state_model::RepoFrontierAdoptedPlan>,
    #[cultcache(key = 13)]
    pub selected_at: String,
    #[cultcache(key = 14)]
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.hands.repo_frontier_authority",
    schema = "RepoFrontierHandsAuthority"
)]
pub struct RepoFrontierHandsAuthority {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub authority_id: String,
    #[cultcache(key = 2)]
    pub route_id: String,
    #[cultcache(key = 3)]
    pub model_projection_digest: String,
    #[cultcache(key = 4)]
    pub model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    #[cultcache(key = 5)]
    pub frontier_item_id: String,
    #[cultcache(key = 6)]
    pub frontier_item_hash: String,
    #[cultcache(key = 7)]
    pub hands_intent_id: String,
    #[cultcache(key = 8)]
    pub hands_review_id: String,
    #[cultcache(key = 9)]
    pub substrate_grant_receipt_id: String,
    #[cultcache(key = 10)]
    pub requested_paths: Vec<String>,
    #[cultcache(key = 11)]
    pub granted_at: String,
    #[cultcache(key = 12)]
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.repo_frontier_relinquishment_receipt",
    schema = "RepoFrontierRelinquishmentReceipt"
)]
pub struct RepoFrontierRelinquishmentReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub hands_refusal_receipt_id: String,
    #[cultcache(key = 3)]
    pub route_id: String,
    #[cultcache(key = 4)]
    pub frontier_item_id: String,
    #[cultcache(key = 5)]
    pub previous_model_projection_digest: String,
    #[cultcache(key = 6)]
    pub previous_model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    #[cultcache(key = 7)]
    pub admitted_model_projection_digest: String,
    #[cultcache(key = 8)]
    pub admitted_model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    #[cultcache(key = 9)]
    pub relinquished_at: String,
    #[cultcache(key = 10)]
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.repo_frontier_execution_amendment_receipt",
    schema = "RepoFrontierExecutionAmendmentReceipt"
)]
pub struct RepoFrontierExecutionAmendmentReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub amendment_id: String,
    #[cultcache(key = 3)]
    pub replaced_route_id: String,
    #[cultcache(key = 4)]
    pub frontier_item_id: String,
    #[cultcache(key = 5)]
    pub previous_frontier_item_hash: String,
    #[cultcache(key = 6)]
    pub previous_model_projection_digest: String,
    #[cultcache(key = 7)]
    pub previous_model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    #[cultcache(key = 8)]
    pub admitted_model_projection_digest: String,
    #[cultcache(key = 9)]
    pub admitted_model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    #[cultcache(key = 10)]
    pub source_actor_id: String,
    #[cultcache(key = 11)]
    pub command_id: String,
    #[cultcache(key = 12)]
    pub admission_id: String,
    #[cultcache(key = 13)]
    pub packet_sha256: String,
    #[cultcache(key = 14)]
    pub replacement_action: String,
    #[cultcache(key = 15)]
    pub replacement_command: String,
    #[cultcache(key = 16)]
    pub rationale: String,
    #[cultcache(key = 17)]
    pub amended_at: String,
    #[cultcache(key = 18)]
    pub contract: String,
}
