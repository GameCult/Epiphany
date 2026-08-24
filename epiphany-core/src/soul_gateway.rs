use cultcache_rs::DatabaseEntry;

pub const REPO_FRONTIER_VERIFICATION_REQUEST_SCHEMA_VERSION: &str =
    "epiphany.soul.repo_frontier_verification_request.v2";
pub const REPO_FRONTIER_VERIFICATION_REQUEST_CONTRACT: &str =
    "epiphany.repo_frontier_verification_request.v2";
pub const SOUL_VERDICT_RECEIPT_SCHEMA_VERSION: &str = "epiphany.soul.verdict_receipt.v1";

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.soul.verdict_receipt", schema = "SoulVerdictReceipt")]
pub struct SoulVerdictReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub source_result_id: String,
    #[cultcache(key = 3)]
    pub source_job_id: String,
    #[cultcache(key = 4)]
    pub verdict: String,
    #[cultcache(key = 5)]
    pub summary: String,
    #[cultcache(key = 6)]
    pub evidence_ids: Vec<String>,
    #[cultcache(key = 7)]
    pub risks: Vec<String>,
    #[cultcache(key = 8)]
    pub emitted_at: String,
    #[cultcache(key = 9)]
    pub contract: String,
    #[cultcache(key = 10, default)]
    pub verification_request_id: String,
    #[cultcache(key = 11, default)]
    pub frontier_route_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.soul.repo_frontier_verification_request",
    schema = "RepoFrontierVerificationRequest"
)]
pub struct RepoFrontierVerificationRequest {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub request_id: String,
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
    pub hands_patch_receipt_id: String,
    #[cultcache(key = 10)]
    pub hands_command_receipt_id: String,
    #[cultcache(key = 11)]
    pub hands_commit_receipt_id: String,
    #[cultcache(key = 12)]
    pub requested_at: String,
    #[cultcache(key = 13)]
    pub contract: String,
    #[cultcache(key = 14)]
    pub frontier_authority_documents: Vec<crate::EpiphanyMindDocumentVersion>,
}
