use crate::MindGatewayDecision;
use cultcache_rs::DatabaseEntry;
use serde::{Deserialize, Serialize};

pub const REPO_MODEL_ADMISSION_REVIEW_TYPE: &str = "epiphany.mind.repo_model_admission_review";
pub const REPO_MODEL_ADMISSION_REVIEW_SCHEMA_VERSION: &str =
    "epiphany.mind.repo_model_admission_review.v0";
pub const REPO_MODEL_ADMISSION_RECEIPT_TYPE: &str = "epiphany.mind.repo_model_admission_receipt";
pub const REPO_MODEL_ADMISSION_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.mind.repo_model_admission_receipt.v3";
pub const REPO_MODEL_MIGRATION_RECEIPT_TYPE: &str = "epiphany.mind.repo_model_migration_receipt";
pub const REPO_MODEL_MIGRATION_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.mind.repo_model_migration_receipt.v0";
pub const REPO_MODEL_ADMISSION_CONTRACT: &str = "epiphany.repo_model_admission.v3";
pub const REPO_MODEL_MIGRATION_CONTRACT: &str = "epiphany.repo_model_migration.v0";
pub const REPO_FRONTIER_ROUTE_TYPE: &str = "epiphany.self.repo_frontier_route";
pub const REPO_FRONTIER_ROUTE_SCHEMA_VERSION: &str = "epiphany.self.repo_frontier_route.v0";
pub const REPO_FRONTIER_HANDS_AUTHORITY_TYPE: &str = "epiphany.hands.repo_frontier_authority";
pub const REPO_FRONTIER_HANDS_AUTHORITY_SCHEMA_VERSION: &str =
    "epiphany.hands.repo_frontier_authority.v0";
pub const REPO_FRONTIER_ROUTE_CONTRACT: &str = "epiphany.repo_frontier_route.v0";
pub const REPO_FRONTIER_HANDS_AUTHORITY_CONTRACT: &str =
    "epiphany.repo_frontier_hands_authority.v0";
pub const REPO_FRONTIER_MODELING_REQUEST_TYPE: &str =
    "epiphany.modeling.repo_frontier_verdict_request";
pub const REPO_FRONTIER_MODELING_REQUEST_SCHEMA_VERSION: &str =
    "epiphany.modeling.repo_frontier_verdict_request.v0";
pub const REPO_FRONTIER_MODELING_REQUEST_CONTRACT: &str =
    "epiphany.repo_frontier_verdict_modeling_request.v0";
pub const REPO_FRONTIER_WORK_PROPOSAL_SCHEMA_VERSION: &str =
    "epiphany.repo_frontier_work_proposal.v0";
pub const REPO_FRONTIER_PLANNING_REQUEST_SCHEMA_VERSION: &str =
    "epiphany.self.repo_frontier_planning_request.v0";
pub const REPO_FRONTIER_PLAN_CANDIDATE_SCHEMA_VERSION: &str =
    "epiphany.imagination.repo_frontier_plan_candidate.v0";
pub const REPO_FRONTIER_PLAN_ADOPTION_SCHEMA_VERSION: &str =
    "epiphany.mind.repo_frontier_plan_adoption.v0";
pub const REPO_FRONTIER_PLANNING_CONTRACT: &str = "epiphany.repo_frontier_planning.v0";
pub const REPO_FRONTIER_WORK_PROPOSAL_CONTRACT: &str =
    "epiphany.repo_frontier_work_proposal.inert.v0";
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
pub const REPO_MODEL_CLAIM_REPAIR_REQUEST_SCHEMA_VERSION: &str =
    "epiphany.modeling.repo_model_claim_repair_request.v0";
pub const REPO_MODEL_CLAIM_REPAIR_REQUEST_CONTRACT: &str =
    "epiphany.repo_model_claim_repair_request.v0";
pub const REPO_MODEL_CLAIM_REPAIR_LAUNCH_BINDING_SCHEMA_VERSION: &str =
    "epiphany.coordinator.repo_model_claim_repair_launch_binding.v0";
pub const REPO_MODEL_CLAIM_REPAIR_LAUNCH_BINDING_CONTRACT: &str =
    "epiphany.repo_model_claim_repair_launch_binding.v0";

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
    pub model_revision: u64,
    #[cultcache(key = 7)]
    pub model_hash: String,
    #[cultcache(key = 8)]
    pub admission_receipt_id: String,
    #[cultcache(key = 9)]
    pub target_claim_id: String,
    #[cultcache(key = 10)]
    pub target_claim_sha256: String,
    #[cultcache(key = 11)]
    pub disposition: RepoModelClaimChallengeDisposition,
    #[cultcache(key = 12)]
    pub finding: String,
    #[cultcache(key = 13)]
    pub uncertainty: String,
    #[cultcache(key = 14)]
    pub source_refs: Vec<String>,
    #[cultcache(key = 15)]
    pub evidence_ids: Vec<String>,
    #[cultcache(key = 16)]
    pub challenged_at: String,
    #[cultcache(key = 17)]
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.modeling.repo_model_claim_repair_request",
    schema = "RepoModelClaimRepairRequest"
)]
pub struct RepoModelClaimRepairRequest {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub challenge_id: String,
    #[cultcache(key = 3)]
    pub challenge_sha256: String,
    #[cultcache(key = 4)]
    pub eyes_evidence_packet_id: String,
    #[cultcache(key = 5)]
    pub eyes_evidence_packet_sha256: String,
    #[cultcache(key = 6)]
    pub source_result_id: String,
    #[cultcache(key = 7)]
    pub source_job_id: String,
    #[cultcache(key = 8)]
    pub original_admission_receipt_id: String,
    #[cultcache(key = 9)]
    pub current_admission_receipt_id: String,
    #[cultcache(key = 10)]
    pub model_revision: u64,
    #[cultcache(key = 11)]
    pub model_hash: String,
    #[cultcache(key = 12)]
    pub target_claim_id: String,
    #[cultcache(key = 13)]
    pub target_claim_sha256: String,
    #[cultcache(key = 14)]
    pub runtime_id: String,
    #[cultcache(key = 15)]
    pub thread_id: String,
    #[cultcache(key = 16)]
    pub affected_frontier: Vec<RepoModelClaimRepairFrontierRef>,
    #[cultcache(key = 17)]
    pub requested_at: String,
    #[cultcache(key = 18)]
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.coordinator.repo_model_claim_repair_launch_binding",
    schema = "RepoModelClaimRepairLaunchBinding"
)]
pub struct RepoModelClaimRepairLaunchBinding {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub binding_record_id: String,
    #[cultcache(key = 2)]
    pub repair_request_id: String,
    #[cultcache(key = 3)]
    pub challenge_id: String,
    #[cultcache(key = 4)]
    pub challenge_sha256: String,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoModelClaimRepairFrontierRef {
    pub frontier_item_id: String,
    pub frontier_item_sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RepoFrontierProposalSourceKind {
    User,
    Persona,
    Bifrost,
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
    pub model_revision: u64,
    #[cultcache(key = 3)]
    pub model_hash: String,
    #[cultcache(key = 4)]
    pub admission_receipt_id: String,
    #[cultcache(key = 5)]
    pub frontier_item_id: String,
    #[cultcache(key = 6)]
    pub frontier_item_hash: String,
    #[cultcache(key = 7)]
    pub selected_organ: String,
    #[cultcache(key = 8)]
    pub source_scope: Vec<String>,
    #[cultcache(key = 9)]
    pub requested_at: String,
    #[cultcache(key = 10)]
    pub contract: String,
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
    pub model_revision: u64,
    #[cultcache(key = 4)]
    pub model_hash: String,
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

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.repo_frontier_plan_adoption",
    schema = "RepoFrontierPlanAdoption"
)]
pub struct RepoFrontierPlanAdoption {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub adoption_id: String,
    #[cultcache(key = 2)]
    pub candidate_id: String,
    #[cultcache(key = 3)]
    pub candidate_sha256: String,
    #[cultcache(key = 4)]
    pub planning_request_id: String,
    #[cultcache(key = 5)]
    pub model_revision: u64,
    #[cultcache(key = 6)]
    pub model_hash: String,
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
    #[cultcache(key = 12)]
    pub contract: String,
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
    pub model_revision: u64,
    #[cultcache(key = 3)]
    pub model_hash: String,
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
    pub verification_acceptance_receipt_id: String,
    #[cultcache(key = 12)]
    pub allowed_disposition: RepoFrontierVerdictDisposition,
    #[cultcache(key = 13)]
    pub requested_at: String,
    #[cultcache(key = 14)]
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
    pub model_revision: u64,
    #[cultcache(key = 4)]
    pub model_hash: String,
    #[cultcache(key = 5)]
    pub admission_receipt_id: String,
    #[cultcache(key = 6)]
    pub frontier_item_id: String,
    #[cultcache(key = 7)]
    pub frontier_item_hash: String,
    #[cultcache(key = 8)]
    pub migration_body: String,
    #[cultcache(key = 9)]
    pub question: String,
    #[cultcache(key = 10)]
    pub gap: String,
    #[cultcache(key = 11)]
    pub target_claim_ids: Vec<String>,
    #[cultcache(key = 12)]
    pub source_scope: Vec<String>,
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
    pub model_revision: u64,
    #[cultcache(key = 4)]
    pub model_hash: String,
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
    type = "epiphany.mind.repo_model_admission_review",
    schema = "RepoModelAdmissionReview"
)]
pub struct RepoModelAdmissionReview {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub review_id: String,
    #[cultcache(key = 2)]
    pub result_id: String,
    #[cultcache(key = 3)]
    pub job_id: String,
    #[cultcache(key = 4)]
    pub patch_id: String,
    #[cultcache(key = 5)]
    pub patch_sha256: String,
    #[cultcache(key = 6)]
    pub base_revision: u64,
    #[cultcache(key = 7)]
    pub base_hash: String,
    #[cultcache(key = 8)]
    pub decision: MindGatewayDecision,
    #[cultcache(key = 9)]
    pub evidence_ids: Vec<String>,
    #[cultcache(key = 10)]
    pub reviewed_at: String,
    #[cultcache(key = 11)]
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.repo_model_admission_receipt",
    schema = "RepoModelAdmissionReceipt"
)]
pub struct RepoModelAdmissionReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub review_id: String,
    #[cultcache(key = 3)]
    pub result_id: String,
    #[cultcache(key = 4)]
    pub patch_id: String,
    #[cultcache(key = 5)]
    pub patch_sha256: String,
    #[cultcache(key = 6)]
    pub previous_revision: u64,
    #[cultcache(key = 7)]
    pub previous_hash: String,
    #[cultcache(key = 8)]
    pub admitted_revision: u64,
    #[cultcache(key = 9)]
    pub admitted_hash: String,
    #[cultcache(key = 10)]
    pub admitted_at: String,
    #[cultcache(key = 11)]
    pub contract: String,
    #[cultcache(key = 12)]
    pub purpose: epiphany_state_model::RepoModelPatchPurpose,
    #[cultcache(key = 13, default)]
    pub frontier_route_id: String,
    #[cultcache(key = 14, default)]
    pub verification_request_id: String,
    #[cultcache(key = 15, default)]
    pub soul_verdict_receipt_id: String,
    #[cultcache(key = 16, default)]
    pub frontier_modeling_request_id: String,
    #[cultcache(key = 17, default)]
    pub proposal_modeling_request_id: String,
    #[cultcache(key = 18, default)]
    pub claim_repair_request_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.repo_model_migration_receipt",
    schema = "RepoModelMigrationReceipt"
)]
pub struct RepoModelMigrationReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub source_store: String,
    #[cultcache(key = 3)]
    pub source_graph_id: String,
    #[cultcache(key = 4)]
    pub imported_revision: u64,
    #[cultcache(key = 5)]
    pub imported_hash: String,
    #[cultcache(key = 6)]
    pub imported_at: String,
    #[cultcache(key = 7)]
    pub contract: String,
}
