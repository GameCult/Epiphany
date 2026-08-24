//! Typed repository-frontier authority chain.
//!
//! The minimal admitted path is deliberately linear:
//!
//! 1. `RepoFrontierWorkProposal` is inert evidence. Modeling may turn it into
//!    an admitted RepoModel frontier item; the proposal itself grants nothing.
//! 2. Self selects exactly one eligible Imagination item and commits a
//!    `RepoFrontierPlanningRequest`. Imagination returns a
//!    `RepoFrontierPlanCandidate` whose safe paths may only narrow the item's
//!    repository scope.
//! 3. Self commits a `RepoFrontierPlanMindRequest`. Mind alone adopts, refuses,
//!    or holds the candidate through `RepoFrontierPlanDecisionReceipt`.
//! 4. Only an adopted decision may be embedded in `RepoFrontierRoute` by
//!    `runtime_spine::select_and_commit_repo_frontier_route`. Route selection
//!    binds the exact keyed frontier and decision documents in one atomic
//!    compare-and-swap operation.
//! 5. Hands remains powerless until the coordinator pairs that route with a
//!    reviewed Hands intent, Substrate Gate receipt, and
//!    `RepoFrontierHandsAuthority`. Hands receipts describe consequences; they
//!    do not admit durable Mind state.
//! 6. Soul verifies the exact route and consequence. The concrete family
//!    admission owner commits its keyed documents and receipt through Mind CAS;
//!    no worker, Hands receipt, coordinator display, or event is a second owner.
//!    Continuity receipts preserve recovery facts. Coordinator status and
//!    CultMesh surfaces only project admitted outcomes.
//!
//! If a required identity or authority is absent at any step, the chain stops.
//! Do not infer it from display state, repair it after execution, or invent a
//! parallel route document.

use anyhow::Result;
use cultcache_rs::DatabaseEntry;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const RUNTIME_REPOSITORY_DOMAIN_BINDING_KEY: &str = "runtime-repository-domain-binding";

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.evidence.repo_frontier_work_proposal",
    schema = "RepoFrontierWorkProposal"
)]
pub struct RepoFrontierWorkProposal {
    #[cultcache(key = 1)]
    pub proposal_id: String,
    #[cultcache(key = 2)]
    pub payload_sha256: String,
    #[cultcache(key = 3)]
    pub title: String,
    #[cultcache(key = 4)]
    pub body: String,
    #[cultcache(key = 5)]
    pub constraints: Vec<String>,
    #[cultcache(key = 6)]
    pub evidence_refs: Vec<String>,
}

pub fn repo_frontier_proposal_payload_sha256(
    title: &str,
    body: &str,
    constraints: &[String],
    evidence_refs: &[String],
) -> Result<String> {
    let content = rmp_serde::to_vec_named(&(title, body, constraints, evidence_refs))?;
    Ok(format!("{:x}", Sha256::digest(content)))
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.runtime.repository_domain_binding",
    schema = "RuntimeRepositoryDomainBinding"
)]
pub struct RuntimeRepositoryDomainBinding {
    #[cultcache(key = 0)]
    pub repository_full_name: String,
    #[cultcache(key = 1)]
    pub body_binding_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.coordinator.repo_frontier_proposal_modeling_request",
    schema = "RepoFrontierProposalModelingRequest"
)]
pub struct RepoFrontierProposalModelingRequest {
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
    #[cultcache(key = 10)]
    pub direction_result_id: String,
    #[cultcache(key = 11)]
    pub direction_option_ordinal: u32,
    #[cultcache(key = 12)]
    pub direction_worker_job_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.self.repo_frontier_planning_request",
    schema = "RepoFrontierPlanningRequest"
)]
pub struct RepoFrontierPlanningRequest {
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
    pub repository_scope: Vec<String>,
    #[cultcache(key = 8)]
    pub requested_at: String,
    #[cultcache(key = 10)]
    pub runtime_id: String,
    /// Exact current documents that own whether this planning work remains
    /// actionable. The complete model sources above are immutable audit cargo.
    #[cultcache(key = 12)]
    pub frontier_authority_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    /// Per-claim guards close the otherwise invisible race between planning
    /// eligibility and a newly admitted external-evidence challenge.
    #[cultcache(key = 13)]
    pub claim_obligation_documents: Vec<crate::EpiphanyMindDocumentVersion>,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.self.repo_frontier_research_request",
    schema = "RepoFrontierResearchRequest"
)]
pub struct RepoFrontierResearchRequest {
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
    pub repository_scope: Vec<String>,
    #[cultcache(key = 7)]
    pub requested_at: String,
    #[cultcache(key = 8)]
    pub runtime_id: String,
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

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.self.repo_frontier_plan_mind_request",
    schema = "RepoFrontierPlanMindRequest"
)]
pub struct RepoFrontierPlanMindRequest {
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
    #[cultcache(key = 9)]
    pub requested_at: String,
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
    #[cultcache(key = 1)]
    pub decision_id: String,
    #[cultcache(key = 2)]
    pub planning_request_id: String,
    #[cultcache(key = 3)]
    pub candidate_id: String,
    #[cultcache(key = 4)]
    pub candidate_sha256: String,
    #[cultcache(key = 5)]
    pub model_projection_digest: String,
    #[cultcache(key = 6)]
    pub model_source_documents: Vec<crate::EpiphanyMindDocumentVersion>,
    #[cultcache(key = 7)]
    pub frontier_item_id: String,
    #[cultcache(key = 8)]
    pub frontier_item_hash: String,
    #[cultcache(key = 9)]
    pub decision: RepoFrontierPlanDecision,
    #[cultcache(key = 10)]
    pub rationale: String,
    #[cultcache(key = 11)]
    pub decided_at: String,
    #[cultcache(key = 13)]
    pub decision_source: RepoFrontierPlanDecisionSource,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum RepoFrontierPlanDecisionSource {
    MindWorker { result_id: String, job_id: String },
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
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.self.repo_frontier_planning_failure_review",
    schema = "RepoFrontierPlanningFailureReview"
)]
pub struct RepoFrontierPlanningFailureReview {
    #[cultcache(key = 1)]
    pub review_id: String,
    #[cultcache(key = 2)]
    pub planning_request_id: String,
    #[cultcache(key = 3)]
    pub pass_kind: String,
    #[cultcache(key = 4)]
    pub job_id: String,
    #[cultcache(key = 5)]
    pub result_id: String,
    #[cultcache(key = 6)]
    pub disposition: String,
    #[cultcache(key = 7)]
    pub reviewed_at: String,
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
    pub authorized_paths: Vec<String>,
    #[cultcache(key = 12, default)]
    pub adopted_plan: Option<crate::RepoFrontierAdoptedPlan>,
    #[cultcache(key = 13)]
    pub selected_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.hands.repo_frontier_authority",
    schema = "RepoFrontierHandsAuthority"
)]
pub struct RepoFrontierHandsAuthority {
    #[cultcache(key = 1)]
    pub authority_id: String,
    #[cultcache(key = 2)]
    pub route_id: String,
    #[cultcache(key = 3)]
    pub hands_intent_id: String,
    #[cultcache(key = 4)]
    pub substrate_grant_receipt_id: String,
}
