use crate::{
    DECISION_CONTEXT_SCHEMA_VERSION, DECISION_CONTEXT_TYPE, MIND_COMMIT_RECEIPT_SCHEMA_VERSION,
    MIND_COMMIT_RECEIPT_TYPE, REASONING_BASIS_SCHEMA_VERSION, REASONING_BASIS_TYPE,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MindCultNetContract {
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

pub fn default_mind_cultnet_contracts() -> Vec<MindCultNetContract> {
    vec![
        MindCultNetContract {
            contract_id: "epiphany.mind.reasoning_basis.snapshot".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: REASONING_BASIS_TYPE.to_string(),
            payload_schema_version: REASONING_BASIS_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "A sealed reasoning basis records the exact typed Mind projection and source document versions supplied to one model pass.".to_string(),
                "CultMesh may project the basis but cannot create, amend, or admit it.".to_string(),
            ],
        },
        MindCultNetContract {
            contract_id: "epiphany.mind.decision_context.snapshot".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: DECISION_CONTEXT_TYPE.to_string(),
            payload_schema_version: DECISION_CONTEXT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "A sealed decision context binds one reasoning basis to the exact terminal native request, internally derived provider request, and governed tool observations actually supplied.".to_string(),
                "A decision context is audit evidence, not permission to mutate arbitrary Mind fields.".to_string(),
            ],
        },
        MindCultNetContract {
            contract_id: "epiphany.mind.commit_receipt.snapshot".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: MIND_COMMIT_RECEIPT_TYPE.to_string(),
            payload_schema_version: MIND_COMMIT_RECEIPT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "A Mind commit receipt names the concrete invariant owner, exact authority, strong reads, and writes committed by one batch CAS.".to_string(),
                "Mutation remains local to concrete invariant owners; CultMesh exposes the receipt without becoming a generic state-effect gateway.".to_string(),
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mind_contracts_publish_only_current_decision_audit_authorities() {
        let contracts = default_mind_cultnet_contracts();
        assert_eq!(contracts.len(), 3);
        assert!(contracts.iter().all(|contract| {
            contract.verse_id == "epiphany-internal"
                && contract.authority == "readOnly"
                && contract.intent_document_types.is_empty()
        }));
        assert!(contracts.iter().any(|contract| {
            contract.document_type == REASONING_BASIS_TYPE
                && contract.payload_schema_version == REASONING_BASIS_SCHEMA_VERSION
        }));
        assert!(contracts.iter().any(|contract| {
            contract.document_type == DECISION_CONTEXT_TYPE
                && contract.payload_schema_version == DECISION_CONTEXT_SCHEMA_VERSION
        }));
        assert!(contracts.iter().any(|contract| {
            contract.document_type == MIND_COMMIT_RECEIPT_TYPE
                && contract.payload_schema_version == MIND_COMMIT_RECEIPT_SCHEMA_VERSION
        }));
    }
}
