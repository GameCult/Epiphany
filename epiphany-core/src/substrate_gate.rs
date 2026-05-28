use serde::Deserialize;
use serde::Serialize;

pub const SUBSTRATE_GATE_REPO_ACCESS_REQUEST_TYPE: &str =
    "epiphany.substrate_gate.repo_access_request";
pub const SUBSTRATE_GATE_REPO_ACCESS_REVIEW_TYPE: &str =
    "epiphany.substrate_gate.repo_access_review";
pub const SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE: &str =
    "epiphany.substrate_gate.repo_access_grant_receipt";
pub const SUBSTRATE_GATE_REPO_ACCESS_REFUSAL_RECEIPT_TYPE: &str =
    "epiphany.substrate_gate.repo_access_refusal_receipt";
pub const SUBSTRATE_GATE_REPO_SNAPSHOT_RECEIPT_TYPE: &str =
    "epiphany.substrate_gate.repo_snapshot_receipt";
pub const SUBSTRATE_GATE_REPO_MUTATION_RECEIPT_TYPE: &str =
    "epiphany.substrate_gate.repo_mutation_receipt";
pub const SUBSTRATE_GATE_REPO_ACCESS_REQUEST_SCHEMA_VERSION: &str =
    "epiphany.substrate_gate.repo_access_request.v0";
pub const SUBSTRATE_GATE_REPO_ACCESS_REVIEW_SCHEMA_VERSION: &str =
    "epiphany.substrate_gate.repo_access_review.v0";
pub const SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.substrate_gate.repo_access_grant_receipt.v0";
pub const SUBSTRATE_GATE_REPO_ACCESS_REFUSAL_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.substrate_gate.repo_access_refusal_receipt.v0";
pub const SUBSTRATE_GATE_REPO_SNAPSHOT_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.substrate_gate.repo_snapshot_receipt.v0";
pub const SUBSTRATE_GATE_REPO_MUTATION_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.substrate_gate.repo_mutation_receipt.v0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubstrateGateCultNetContract {
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

pub fn default_substrate_gate_cultnet_contracts() -> Vec<SubstrateGateCultNetContract> {
    vec![
        SubstrateGateCultNetContract {
            contract_id: "epiphany.substrate_gate.repo_access.review".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: SUBSTRATE_GATE_REPO_ACCESS_REQUEST_TYPE.to_string(),
            payload_schema_version: SUBSTRATE_GATE_REPO_ACCESS_REQUEST_SCHEMA_VERSION.to_string(),
            authority: "substrateGate".to_string(),
            operations: vec![
                "intentSubmit".to_string(),
                "receiptWatch".to_string(),
                "snapshot".to_string(),
            ],
            intent_document_types: vec![SUBSTRATE_GATE_REPO_ACCESS_REQUEST_TYPE.to_string()],
            receipt_document_types: vec![
                SUBSTRATE_GATE_REPO_ACCESS_REVIEW_TYPE.to_string(),
                SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE.to_string(),
                SUBSTRATE_GATE_REPO_ACCESS_REFUSAL_RECEIPT_TYPE.to_string(),
            ],
            notes: vec![
                "Substrate Gate is the repo access protocol: workers request repository reads, indexing, commands, edits, and bridge operations here.".to_string(),
                "Hands may mutate files only after Substrate Gate grants scoped access; Eyes may inspect only through scoped Substrate Gate read/index grants.".to_string(),
            ],
        },
        SubstrateGateCultNetContract {
            contract_id: "epiphany.substrate_gate.repo_access.review_receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: SUBSTRATE_GATE_REPO_ACCESS_REVIEW_TYPE.to_string(),
            payload_schema_version: SUBSTRATE_GATE_REPO_ACCESS_REVIEW_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Substrate Gate reviews explain which repo paths, operations, commands, and bridge surfaces were granted or refused.".to_string(),
            ],
        },
        SubstrateGateCultNetContract {
            contract_id: "epiphany.substrate_gate.repo_snapshot.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: SUBSTRATE_GATE_REPO_SNAPSHOT_RECEIPT_TYPE.to_string(),
            payload_schema_version: SUBSTRATE_GATE_REPO_SNAPSHOT_RECEIPT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Repo snapshots are evidence projections from Substrate Gate access; they do not grant future access by existing.".to_string(),
            ],
        },
        SubstrateGateCultNetContract {
            contract_id: "epiphany.substrate_gate.repo_mutation.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: SUBSTRATE_GATE_REPO_MUTATION_RECEIPT_TYPE.to_string(),
            payload_schema_version: SUBSTRATE_GATE_REPO_MUTATION_RECEIPT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Repo mutation receipts prove Substrate Gate granted the scoped substrate touch before Hands changed files or ran repo-affecting commands.".to_string(),
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substrate_gate_contracts_make_internal_verse_the_repo_access_gate() {
        let contracts = default_substrate_gate_cultnet_contracts();
        let repo_access = contracts
            .iter()
            .find(|contract| contract.contract_id == "epiphany.substrate_gate.repo_access.review")
            .expect("repo access review contract");

        assert_eq!(repo_access.verse_id, "epiphany-internal");
        assert_eq!(repo_access.authority, "substrateGate");
        assert!(
            repo_access
                .notes
                .iter()
                .any(|note| note.contains("repo access protocol"))
        );
        assert!(
            repo_access
                .receipt_document_types
                .contains(&SUBSTRATE_GATE_REPO_ACCESS_GRANT_RECEIPT_TYPE.to_string())
        );
    }
}
