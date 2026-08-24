use cultcache_rs::DatabaseEntry;

pub const HANDS_COMMAND_RECEIPT_TYPE: &str = "epiphany.hands.command_receipt";
pub const HANDS_PATCH_RECEIPT_TYPE: &str = "epiphany.hands.patch_receipt";
pub const HANDS_COMMIT_RECEIPT_TYPE: &str = "epiphany.hands.commit_receipt";
pub const HANDS_ACTION_INTENT_SCHEMA_VERSION: &str = "epiphany.hands.action_intent.v1";
pub const HANDS_ACTION_REVIEW_SCHEMA_VERSION: &str = "epiphany.hands.action_review.v0";
pub const HANDS_COMMAND_RECEIPT_SCHEMA_VERSION: &str = "epiphany.hands.command_receipt.v0";
pub const HANDS_PATCH_RECEIPT_SCHEMA_VERSION: &str = "epiphany.hands.patch_receipt.v0";
pub const HANDS_COMMIT_RECEIPT_SCHEMA_VERSION: &str = "epiphany.hands.commit_receipt.v0";

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.hands.action_intent", schema = "HandsActionIntent")]
pub struct HandsActionIntent {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub intent_id: String,
    #[cultcache(key = 2)]
    pub runtime_job_id: String,
    #[cultcache(key = 3)]
    pub binding_id: String,
    #[cultcache(key = 4)]
    pub role: String,
    #[cultcache(key = 5)]
    pub authority_scope: String,
    #[cultcache(key = 6)]
    pub requested_action: String,
    #[cultcache(key = 7)]
    pub requested_paths: Vec<String>,
    #[cultcache(key = 8)]
    pub substrate_gate_grant_receipt_id: String,
    #[cultcache(key = 9)]
    pub requested_at: String,
    #[cultcache(key = 10)]
    pub contract: String,
    #[cultcache(key = 11, default)]
    pub frontier_route_id: String,
    #[cultcache(key = 12, default)]
    pub plan_candidate_sha256: String,
    #[cultcache(key = 13, default)]
    pub plan_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.hands.action_review", schema = "HandsActionReview")]
pub struct HandsActionReview {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub review_id: String,
    #[cultcache(key = 2)]
    pub intent_id: String,
    #[cultcache(key = 3)]
    pub decision: String,
    #[cultcache(key = 4)]
    pub allowed_operations: Vec<String>,
    #[cultcache(key = 5)]
    pub required_receipts: Vec<String>,
    #[cultcache(key = 6)]
    pub reasons: Vec<String>,
    #[cultcache(key = 7)]
    pub reviewed_at: String,
    #[cultcache(key = 8)]
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.hands.patch_receipt", schema = "HandsPatchReceipt")]
pub struct HandsPatchReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub intent_id: String,
    #[cultcache(key = 3)]
    pub review_id: String,
    #[cultcache(key = 4)]
    pub substrate_gate_grant_receipt_id: String,
    #[cultcache(key = 5)]
    pub runtime_job_id: String,
    #[cultcache(key = 6)]
    pub changed_paths: Vec<String>,
    #[cultcache(key = 7)]
    pub summary: String,
    #[cultcache(key = 8)]
    pub emitted_at: String,
    #[cultcache(key = 9)]
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.hands.command_receipt",
    schema = "HandsCommandReceipt"
)]
pub struct HandsCommandReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub intent_id: String,
    #[cultcache(key = 3)]
    pub review_id: String,
    #[cultcache(key = 4)]
    pub substrate_gate_grant_receipt_id: String,
    #[cultcache(key = 5)]
    pub runtime_job_id: String,
    #[cultcache(key = 6)]
    pub command: String,
    #[cultcache(key = 7)]
    pub exit_code: String,
    #[cultcache(key = 8)]
    pub stdout_artifact: String,
    #[cultcache(key = 9)]
    pub stderr_artifact: String,
    #[cultcache(key = 10)]
    pub summary: String,
    #[cultcache(key = 11)]
    pub emitted_at: String,
    #[cultcache(key = 12)]
    pub contract: String,
}

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.hands.commit_receipt", schema = "HandsCommitReceipt")]
pub struct HandsCommitReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub intent_id: String,
    #[cultcache(key = 3)]
    pub review_id: String,
    #[cultcache(key = 4)]
    pub runtime_job_id: String,
    #[cultcache(key = 5)]
    pub commit_sha: String,
    #[cultcache(key = 6)]
    pub branch: String,
    #[cultcache(key = 7)]
    pub changed_paths: Vec<String>,
    #[cultcache(key = 8)]
    pub summary: String,
    #[cultcache(key = 9)]
    pub emitted_at: String,
    #[cultcache(key = 10)]
    pub contract: String,
}

pub fn hands_action_review_for_intent(
    review_id: String,
    intent: &HandsActionIntent,
    decision: String,
    allowed_operations: Vec<String>,
    reasons: Vec<String>,
    reviewed_at: String,
) -> HandsActionReview {
    HandsActionReview {
        schema_version: HANDS_ACTION_REVIEW_SCHEMA_VERSION.to_string(),
        review_id,
        intent_id: intent.intent_id.clone(),
        decision,
        allowed_operations,
        required_receipts: vec![HANDS_PATCH_RECEIPT_TYPE.to_string()],
        reasons,
        reviewed_at,
        contract: "Hands review is the execution decision for a bounded action intent; it depends on Substrate Gate access and does not admit durable Mind state.".to_string(),
    }
}

pub fn hands_patch_receipt_for_review(
    receipt_id: String,
    intent: &HandsActionIntent,
    review: &HandsActionReview,
    changed_paths: Vec<String>,
    summary: String,
    emitted_at: String,
) -> HandsPatchReceipt {
    HandsPatchReceipt {
        schema_version: HANDS_PATCH_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id,
        intent_id: intent.intent_id.clone(),
        review_id: review.review_id.clone(),
        substrate_gate_grant_receipt_id: intent.substrate_gate_grant_receipt_id.clone(),
        runtime_job_id: intent.runtime_job_id.clone(),
        changed_paths,
        summary,
        emitted_at,
        contract: "Hands patch receipt proves which files changed under the reviewed action and named Substrate Gate grant; Soul and Mind still decide verification and durable admission.".to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn hands_command_receipt_for_review(
    receipt_id: String,
    intent: &HandsActionIntent,
    review: &HandsActionReview,
    command: String,
    exit_code: String,
    stdout_artifact: String,
    stderr_artifact: String,
    summary: String,
    emitted_at: String,
) -> HandsCommandReceipt {
    HandsCommandReceipt {
        schema_version: HANDS_COMMAND_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id,
        intent_id: intent.intent_id.clone(),
        review_id: review.review_id.clone(),
        substrate_gate_grant_receipt_id: intent.substrate_gate_grant_receipt_id.clone(),
        runtime_job_id: intent.runtime_job_id.clone(),
        command,
        exit_code,
        stdout_artifact,
        stderr_artifact,
        summary,
        emitted_at,
        contract: "Hands command receipt proves which command ran, where output evidence lives, and which reviewed action plus Substrate Gate grant authorized it.".to_string(),
    }
}

pub fn hands_commit_receipt_for_review(
    receipt_id: String,
    intent: &HandsActionIntent,
    review: &HandsActionReview,
    commit_sha: String,
    branch: String,
    changed_paths: Vec<String>,
    summary: String,
    emitted_at: String,
) -> HandsCommitReceipt {
    HandsCommitReceipt {
        schema_version: HANDS_COMMIT_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id,
        intent_id: intent.intent_id.clone(),
        review_id: review.review_id.clone(),
        runtime_job_id: intent.runtime_job_id.clone(),
        commit_sha,
        branch,
        changed_paths,
        summary,
        emitted_at,
        contract: "Hands commit receipt proves a repository commit consequence after a reviewed action; it is still subject to Soul verification and Mind admission.".to_string(),
    }
}
