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
}
