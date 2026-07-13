use crate::MindGatewayDecision;
use cultcache_rs::DatabaseEntry;
use serde::{Deserialize, Serialize};

pub const REPO_MODEL_ADMISSION_REVIEW_TYPE: &str = "epiphany.mind.repo_model_admission_review";
pub const REPO_MODEL_ADMISSION_REVIEW_SCHEMA_VERSION: &str =
    "epiphany.mind.repo_model_admission_review.v0";
pub const REPO_MODEL_ADMISSION_RECEIPT_TYPE: &str = "epiphany.mind.repo_model_admission_receipt";
pub const REPO_MODEL_ADMISSION_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.mind.repo_model_admission_receipt.v1";
pub const REPO_MODEL_MIGRATION_RECEIPT_TYPE: &str = "epiphany.mind.repo_model_migration_receipt";
pub const REPO_MODEL_MIGRATION_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.mind.repo_model_migration_receipt.v0";
pub const REPO_MODEL_ADMISSION_CONTRACT: &str = "epiphany.repo_model_admission.v1";
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
