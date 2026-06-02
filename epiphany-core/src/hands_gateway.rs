use cultcache_rs::DatabaseEntry;
use serde::Deserialize;
use serde::Serialize;

pub const HANDS_ACTION_INTENT_TYPE: &str = "epiphany.hands.action_intent";
pub const HANDS_ACTION_REVIEW_TYPE: &str = "epiphany.hands.action_review";
pub const HANDS_COMMAND_RECEIPT_TYPE: &str = "epiphany.hands.command_receipt";
pub const HANDS_PATCH_RECEIPT_TYPE: &str = "epiphany.hands.patch_receipt";
pub const HANDS_COMMIT_RECEIPT_TYPE: &str = "epiphany.hands.commit_receipt";
pub const HANDS_PR_RECEIPT_TYPE: &str = "epiphany.hands.pr_receipt";
pub const HANDS_ROLLBACK_RECEIPT_TYPE: &str = "epiphany.hands.rollback_receipt";
pub const HANDS_ACTION_REFUSAL_RECEIPT_TYPE: &str = "epiphany.hands.action_refusal_receipt";
pub const HANDS_ACTION_INTENT_SCHEMA_VERSION: &str = "epiphany.hands.action_intent.v0";
pub const HANDS_ACTION_REVIEW_SCHEMA_VERSION: &str = "epiphany.hands.action_review.v0";
pub const HANDS_COMMAND_RECEIPT_SCHEMA_VERSION: &str = "epiphany.hands.command_receipt.v0";
pub const HANDS_PATCH_RECEIPT_SCHEMA_VERSION: &str = "epiphany.hands.patch_receipt.v0";
pub const HANDS_COMMIT_RECEIPT_SCHEMA_VERSION: &str = "epiphany.hands.commit_receipt.v0";
pub const HANDS_PR_RECEIPT_SCHEMA_VERSION: &str = "epiphany.hands.pr_receipt.v0";
pub const HANDS_ROLLBACK_RECEIPT_SCHEMA_VERSION: &str = "epiphany.hands.rollback_receipt.v0";
pub const HANDS_ACTION_REFUSAL_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.hands.action_refusal_receipt.v0";

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HandsCultNetContract {
    pub contract_id: String,
    pub verse_id: String,
    pub document_type: String,
    pub payload_schema_version: String,
    pub authority: String,
    pub operations: Vec<String>,
    pub intent_document_types: Vec<String>,
    pub receipt_document_types: Vec<String>,
    pub notes: Vec<String>,
}

pub fn default_hands_cultnet_contracts() -> Vec<HandsCultNetContract> {
    vec![
        HandsCultNetContract {
            contract_id: "epiphany.hands.action.review".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: HANDS_ACTION_INTENT_TYPE.to_string(),
            payload_schema_version: HANDS_ACTION_INTENT_SCHEMA_VERSION.to_string(),
            authority: "hands".to_string(),
            operations: vec![
                "intentSubmit".to_string(),
                "receiptWatch".to_string(),
                "snapshot".to_string(),
            ],
            intent_document_types: vec![HANDS_ACTION_INTENT_TYPE.to_string()],
            receipt_document_types: vec![
                HANDS_ACTION_REVIEW_TYPE.to_string(),
                HANDS_COMMAND_RECEIPT_TYPE.to_string(),
                HANDS_PATCH_RECEIPT_TYPE.to_string(),
                HANDS_COMMIT_RECEIPT_TYPE.to_string(),
                HANDS_PR_RECEIPT_TYPE.to_string(),
                HANDS_ROLLBACK_RECEIPT_TYPE.to_string(),
                HANDS_ACTION_REFUSAL_RECEIPT_TYPE.to_string(),
            ],
            notes: vec![
                "Hands is the action organ: commands, patches, commits, PRs, and rollbacks enter here as bounded action intents.".to_string(),
                "Hands does not grant repo access; Substrate Gate grants substrate access before Hands mutates, and Soul verifies the result after.".to_string(),
            ],
        },
        HandsCultNetContract {
            contract_id: "epiphany.hands.action.review_receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: HANDS_ACTION_REVIEW_TYPE.to_string(),
            payload_schema_version: HANDS_ACTION_REVIEW_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Hands reviews explain what action was allowed, refused, sequenced, or delegated after Substrate Gate access was checked.".to_string(),
            ],
        },
        HandsCultNetContract {
            contract_id: "epiphany.hands.command.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: HANDS_COMMAND_RECEIPT_TYPE.to_string(),
            payload_schema_version: HANDS_COMMAND_RECEIPT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Command receipts prove what command ran, under which Substrate Gate grant, and where output artifacts live.".to_string(),
            ],
        },
        HandsCultNetContract {
            contract_id: "epiphany.hands.patch.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: HANDS_PATCH_RECEIPT_TYPE.to_string(),
            payload_schema_version: HANDS_PATCH_RECEIPT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Patch receipts prove which files changed and which Substrate Gate grant made the edit legal.".to_string(),
            ],
        },
        HandsCultNetContract {
            contract_id: "epiphany.hands.commit_and_pr.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: HANDS_COMMIT_RECEIPT_TYPE.to_string(),
            payload_schema_version: HANDS_COMMIT_RECEIPT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: vec![HANDS_PR_RECEIPT_TYPE.to_string()],
            notes: vec![
                "Commit and PR receipts preserve publication consequences after Soul verification and operator policy allow them.".to_string(),
            ],
        },
        HandsCultNetContract {
            contract_id: "epiphany.hands.rollback.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: HANDS_ROLLBACK_RECEIPT_TYPE.to_string(),
            payload_schema_version: HANDS_ROLLBACK_RECEIPT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Rollback receipts prove a failed or refused action was unwound instead of hidden by later convergence.".to_string(),
            ],
        },
    ]
}

pub fn hands_action_review_for_intent(
    review_id: String,
    intent: &HandsActionIntent,
    decision: String,
    allowed_operations: Vec<String>,
    reasons: Vec<String>,
    reviewed_at: String,
) -> HandsActionReview {
    let mut required_receipts = Vec::new();
    for operation in &allowed_operations {
        match operation.trim() {
            "patch" => push_unique(&mut required_receipts, HANDS_PATCH_RECEIPT_TYPE),
            "command" => push_unique(&mut required_receipts, HANDS_COMMAND_RECEIPT_TYPE),
            "commit" => push_unique(&mut required_receipts, HANDS_COMMIT_RECEIPT_TYPE),
            _ => {}
        }
    }
    if required_receipts.is_empty() {
        required_receipts.push(HANDS_PATCH_RECEIPT_TYPE.to_string());
    }
    HandsActionReview {
        schema_version: HANDS_ACTION_REVIEW_SCHEMA_VERSION.to_string(),
        review_id,
        intent_id: intent.intent_id.clone(),
        decision,
        allowed_operations,
        required_receipts,
        reasons,
        reviewed_at,
        contract: "Hands review is the execution decision for a bounded action intent; it depends on Substrate Gate access and does not admit durable Mind state.".to_string(),
    }
}

fn push_unique(target: &mut Vec<String>, value: &str) {
    if !target.iter().any(|existing| existing == value) {
        target.push(value.to_string());
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
        contract: "Hands commit receipt proves source publication consequences after a reviewed action; it is still subject to Soul verification and Mind admission.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hands_contracts_make_internal_verse_the_action_gate() {
        let contracts = default_hands_cultnet_contracts();
        let action = contracts
            .iter()
            .find(|contract| contract.contract_id == "epiphany.hands.action.review")
            .expect("hands action review contract");

        assert_eq!(action.verse_id, "epiphany-internal");
        assert_eq!(action.authority, "hands");
        assert!(
            action
                .notes
                .iter()
                .any(|note| note.contains("action organ"))
        );
        assert!(
            action
                .receipt_document_types
                .contains(&HANDS_PATCH_RECEIPT_TYPE.to_string())
        );
    }

    #[test]
    fn hands_action_review_and_patch_receipt_preserve_the_chain() {
        let intent = HandsActionIntent {
            schema_version: HANDS_ACTION_INTENT_SCHEMA_VERSION.to_string(),
            intent_id: "hands-intent-1".to_string(),
            runtime_job_id: "job-1".to_string(),
            binding_id: "implementation-worker".to_string(),
            role: "epiphany-hands".to_string(),
            authority_scope: "epiphany.role.implementation".to_string(),
            requested_action: "patch".to_string(),
            requested_paths: vec!["src/lib.rs".to_string()],
            substrate_gate_grant_receipt_id: "substrate-grant-1".to_string(),
            requested_at: "2026-06-02T00:00:00Z".to_string(),
            contract: "Hands action intent is bounded by Substrate Gate.".to_string(),
        };
        let review = hands_action_review_for_intent(
            "hands-review-1".to_string(),
            &intent,
            "approved".to_string(),
            vec!["patch".to_string()],
            vec!["Substrate Gate grant is present.".to_string()],
            "2026-06-02T00:01:00Z".to_string(),
        );
        let receipt = hands_patch_receipt_for_review(
            "hands-patch-1".to_string(),
            &intent,
            &review,
            vec!["src/lib.rs".to_string()],
            "Applied focused patch.".to_string(),
            "2026-06-02T00:02:00Z".to_string(),
        );

        assert_eq!(review.intent_id, intent.intent_id);
        assert_eq!(receipt.review_id, review.review_id);
        assert_eq!(receipt.substrate_gate_grant_receipt_id, "substrate-grant-1");
        let command = hands_command_receipt_for_review(
            "hands-command-1".to_string(),
            &intent,
            &review,
            "cargo test".to_string(),
            "0".to_string(),
            "artifacts/stdout.log".to_string(),
            "artifacts/stderr.log".to_string(),
            "Focused command passed.".to_string(),
            "2026-06-02T00:03:00Z".to_string(),
        );
        let commit = hands_commit_receipt_for_review(
            "hands-commit-1".to_string(),
            &intent,
            &review,
            "abc123".to_string(),
            "main".to_string(),
            vec!["src/lib.rs".to_string()],
            "Committed focused patch.".to_string(),
            "2026-06-02T00:04:00Z".to_string(),
        );
        assert_eq!(command.review_id, review.review_id);
        assert_eq!(commit.intent_id, intent.intent_id);
        assert!(
            review
                .required_receipts
                .contains(&HANDS_PATCH_RECEIPT_TYPE.to_string())
        );
    }

    #[test]
    fn hands_action_review_names_receipts_for_allowed_operations() {
        let intent = HandsActionIntent {
            schema_version: HANDS_ACTION_INTENT_SCHEMA_VERSION.to_string(),
            intent_id: "hands-intent-2".to_string(),
            runtime_job_id: "job-2".to_string(),
            binding_id: "implementation-worker".to_string(),
            role: "epiphany-hands".to_string(),
            authority_scope: "epiphany.role.implementation".to_string(),
            requested_action: "branch-turn".to_string(),
            requested_paths: vec!["src/lib.rs".to_string()],
            substrate_gate_grant_receipt_id: "substrate-grant-2".to_string(),
            requested_at: "2026-06-02T00:00:00Z".to_string(),
            contract: "Hands action intent is bounded by Substrate Gate.".to_string(),
        };
        let review = hands_action_review_for_intent(
            "hands-review-2".to_string(),
            &intent,
            "approved".to_string(),
            vec![
                "patch".to_string(),
                "command".to_string(),
                "commit".to_string(),
            ],
            vec!["Branch turn evidence is required.".to_string()],
            "2026-06-02T00:01:00Z".to_string(),
        );

        assert!(
            review
                .required_receipts
                .contains(&HANDS_PATCH_RECEIPT_TYPE.to_string())
        );
        assert!(
            review
                .required_receipts
                .contains(&HANDS_COMMAND_RECEIPT_TYPE.to_string())
        );
        assert!(
            review
                .required_receipts
                .contains(&HANDS_COMMIT_RECEIPT_TYPE.to_string())
        );
    }
}
