use serde::Deserialize;
use serde::Serialize;

pub const SOUL_VERIFICATION_REQUEST_TYPE: &str = "epiphany.soul.verification_request";
pub const SOUL_INVARIANT_CHECK_TYPE: &str = "epiphany.soul.invariant_check";
pub const SOUL_VERDICT_RECEIPT_TYPE: &str = "epiphany.soul.verdict_receipt";
pub const SOUL_REGRESSION_RECEIPT_TYPE: &str = "epiphany.soul.regression_receipt";
pub const SOUL_REVIEW_RECEIPT_TYPE: &str = "epiphany.soul.review_receipt";
pub const SOUL_VERIFICATION_REFUSAL_RECEIPT_TYPE: &str =
    "epiphany.soul.verification_refusal_receipt";
pub const SOUL_VERIFICATION_REQUEST_SCHEMA_VERSION: &str = "epiphany.soul.verification_request.v0";
pub const SOUL_INVARIANT_CHECK_SCHEMA_VERSION: &str = "epiphany.soul.invariant_check.v0";
pub const SOUL_VERDICT_RECEIPT_SCHEMA_VERSION: &str = "epiphany.soul.verdict_receipt.v0";
pub const SOUL_REGRESSION_RECEIPT_SCHEMA_VERSION: &str = "epiphany.soul.regression_receipt.v0";
pub const SOUL_REVIEW_RECEIPT_SCHEMA_VERSION: &str = "epiphany.soul.review_receipt.v0";
pub const SOUL_VERIFICATION_REFUSAL_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.soul.verification_refusal_receipt.v0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SoulCultNetContract {
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

pub fn default_soul_cultnet_contracts() -> Vec<SoulCultNetContract> {
    vec![
        SoulCultNetContract {
            contract_id: "epiphany.soul.verification.review".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: SOUL_VERIFICATION_REQUEST_TYPE.to_string(),
            payload_schema_version: SOUL_VERIFICATION_REQUEST_SCHEMA_VERSION.to_string(),
            authority: "soul".to_string(),
            operations: vec![
                "intentSubmit".to_string(),
                "receiptWatch".to_string(),
                "snapshot".to_string(),
            ],
            intent_document_types: vec![SOUL_VERIFICATION_REQUEST_TYPE.to_string()],
            receipt_document_types: vec![
                SOUL_INVARIANT_CHECK_TYPE.to_string(),
                SOUL_VERDICT_RECEIPT_TYPE.to_string(),
                SOUL_REGRESSION_RECEIPT_TYPE.to_string(),
                SOUL_REVIEW_RECEIPT_TYPE.to_string(),
                SOUL_VERIFICATION_REFUSAL_RECEIPT_TYPE.to_string(),
            ],
            notes: vec![
                "Soul is the verification organ: invariants, tests, review, falsification, and refusal enter here.".to_string(),
                "Soul does not mutate state or repo by itself; Hands acts, Eyes grounds evidence, and Mind admits durable state after Soul's verdict.".to_string(),
            ],
        },
        SoulCultNetContract {
            contract_id: "epiphany.soul.invariant_check.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: SOUL_INVARIANT_CHECK_TYPE.to_string(),
            payload_schema_version: SOUL_INVARIANT_CHECK_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Invariant checks say which promise was tested, at which layer, and whether the old path can still violate it.".to_string(),
            ],
        },
        SoulCultNetContract {
            contract_id: "epiphany.soul.verdict.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: SOUL_VERDICT_RECEIPT_TYPE.to_string(),
            payload_schema_version: SOUL_VERDICT_RECEIPT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Verdict receipts are proof of sanctity or proof of failure; they are not cosmetic test summaries.".to_string(),
            ],
        },
        SoulCultNetContract {
            contract_id: "epiphany.soul.regression.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: SOUL_REGRESSION_RECEIPT_TYPE.to_string(),
            payload_schema_version: SOUL_REGRESSION_RECEIPT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Regression receipts preserve violated invariants and the old state path that still had authority.".to_string(),
            ],
        },
        SoulCultNetContract {
            contract_id: "epiphany.soul.review.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: SOUL_REVIEW_RECEIPT_TYPE.to_string(),
            payload_schema_version: SOUL_REVIEW_RECEIPT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Review receipts preserve risks, missing tests, and falsification notes for Mind before state admission.".to_string(),
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn soul_contracts_make_internal_verse_the_verification_gate() {
        let contracts = default_soul_cultnet_contracts();
        let verification = contracts
            .iter()
            .find(|contract| contract.contract_id == "epiphany.soul.verification.review")
            .expect("soul verification review contract");

        assert_eq!(verification.verse_id, "epiphany-internal");
        assert_eq!(verification.authority, "soul");
        assert!(
            verification
                .notes
                .iter()
                .any(|note| note.contains("verification organ"))
        );
        assert!(
            verification
                .receipt_document_types
                .contains(&SOUL_VERDICT_RECEIPT_TYPE.to_string())
        );
    }
}
