use serde::Deserialize;
use serde::Serialize;

pub const EYES_EVIDENCE_REQUEST_TYPE: &str = "epiphany.eyes.evidence_request";
pub const EYES_EVIDENCE_REVIEW_TYPE: &str = "epiphany.eyes.evidence_review";
pub const EYES_SOURCE_LOOKUP_RECEIPT_TYPE: &str = "epiphany.eyes.source_lookup_receipt";
pub const EYES_EVIDENCE_PACKET_TYPE: &str = "epiphany.eyes.evidence_packet";
pub const EYES_EVIDENCE_REFUSAL_RECEIPT_TYPE: &str = "epiphany.eyes.evidence_refusal_receipt";
pub const EYES_EVIDENCE_REQUEST_SCHEMA_VERSION: &str = "epiphany.eyes.evidence_request.v0";
pub const EYES_EVIDENCE_REVIEW_SCHEMA_VERSION: &str = "epiphany.eyes.evidence_review.v0";
pub const EYES_SOURCE_LOOKUP_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.eyes.source_lookup_receipt.v0";
pub const EYES_EVIDENCE_PACKET_SCHEMA_VERSION: &str = "epiphany.eyes.evidence_packet.v0";
pub const EYES_EVIDENCE_REFUSAL_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.eyes.evidence_refusal_receipt.v0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EyesCultNetContract {
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

pub fn default_eyes_cultnet_contracts() -> Vec<EyesCultNetContract> {
    vec![
        EyesCultNetContract {
            contract_id: "epiphany.eyes.evidence.review".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: EYES_EVIDENCE_REQUEST_TYPE.to_string(),
            payload_schema_version: EYES_EVIDENCE_REQUEST_SCHEMA_VERSION.to_string(),
            authority: "eyes".to_string(),
            operations: vec![
                "intentSubmit".to_string(),
                "receiptWatch".to_string(),
                "snapshot".to_string(),
            ],
            intent_document_types: vec![EYES_EVIDENCE_REQUEST_TYPE.to_string()],
            receipt_document_types: vec![
                EYES_EVIDENCE_REVIEW_TYPE.to_string(),
                EYES_SOURCE_LOOKUP_RECEIPT_TYPE.to_string(),
                EYES_EVIDENCE_PACKET_TYPE.to_string(),
                EYES_EVIDENCE_REFUSAL_RECEIPT_TYPE.to_string(),
            ],
            notes: vec![
                "Eyes is the evidence ingress guardian: organs request source-grounded lookup, provenance, uncertainty, and evidence packets here.".to_string(),
                "Body grants substrate access; Eyes turns inspected material into citable evidence packets.".to_string(),
            ],
        },
        EyesCultNetContract {
            contract_id: "epiphany.eyes.evidence.review_receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: EYES_EVIDENCE_REVIEW_TYPE.to_string(),
            payload_schema_version: EYES_EVIDENCE_REVIEW_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Eyes reviews explain whether a claim has source grounding, needs more looking, or should be refused.".to_string(),
            ],
        },
        EyesCultNetContract {
            contract_id: "epiphany.eyes.source_lookup.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: EYES_SOURCE_LOOKUP_RECEIPT_TYPE.to_string(),
            payload_schema_version: EYES_SOURCE_LOOKUP_RECEIPT_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Source lookup receipts prove what was searched or inspected, under which Body grant, before another organ cites it.".to_string(),
            ],
        },
        EyesCultNetContract {
            contract_id: "epiphany.eyes.evidence_packet.receipts".to_string(),
            verse_id: "epiphany-internal".to_string(),
            document_type: EYES_EVIDENCE_PACKET_TYPE.to_string(),
            payload_schema_version: EYES_EVIDENCE_PACKET_SCHEMA_VERSION.to_string(),
            authority: "readOnly".to_string(),
            operations: vec!["snapshot".to_string(), "receiptWatch".to_string()],
            intent_document_types: Vec::new(),
            receipt_document_types: Vec::new(),
            notes: vec![
                "Evidence packets carry provenance, uncertainty, and source refs for Imagination, Mind, Hands, Soul, Face, Self, Body, and Life.".to_string(),
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eyes_contracts_make_internal_verse_the_evidence_gate() {
        let contracts = default_eyes_cultnet_contracts();
        let evidence = contracts
            .iter()
            .find(|contract| contract.contract_id == "epiphany.eyes.evidence.review")
            .expect("eyes evidence review contract");

        assert_eq!(evidence.verse_id, "epiphany-internal");
        assert_eq!(evidence.authority, "eyes");
        assert!(
            evidence
                .notes
                .iter()
                .any(|note| note.contains("evidence ingress guardian"))
        );
        assert!(
            evidence
                .receipt_document_types
                .contains(&EYES_EVIDENCE_PACKET_TYPE.to_string())
        );
    }
}
