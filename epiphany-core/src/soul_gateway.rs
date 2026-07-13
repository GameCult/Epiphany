use crate::EpiphanyRoleFindingInterpretation;
use cultcache_rs::DatabaseEntry;
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
pub const REPO_FRONTIER_VERIFICATION_REQUEST_TYPE: &str =
    "epiphany.soul.repo_frontier_verification_request";
pub const REPO_FRONTIER_VERIFICATION_REQUEST_SCHEMA_VERSION: &str =
    "epiphany.soul.repo_frontier_verification_request.v0";
pub const REPO_FRONTIER_VERIFICATION_REQUEST_CONTRACT: &str =
    "epiphany.repo_frontier_verification_request.v0";
pub const SOUL_INVARIANT_CHECK_SCHEMA_VERSION: &str = "epiphany.soul.invariant_check.v0";
pub const SOUL_VERDICT_RECEIPT_SCHEMA_VERSION: &str = "epiphany.soul.verdict_receipt.v1";
pub const SOUL_REGRESSION_RECEIPT_SCHEMA_VERSION: &str = "epiphany.soul.regression_receipt.v0";
pub const SOUL_REVIEW_RECEIPT_SCHEMA_VERSION: &str = "epiphany.soul.review_receipt.v0";
pub const SOUL_VERIFICATION_REFUSAL_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.soul.verification_refusal_receipt.v0";

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
    pub hands_patch_receipt_id: String,
    #[cultcache(key = 10)]
    pub hands_command_receipt_id: String,
    #[cultcache(key = 11)]
    pub hands_commit_receipt_id: String,
    #[cultcache(key = 12)]
    pub requested_at: String,
    #[cultcache(key = 13)]
    pub contract: String,
}

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

pub fn soul_verdict_receipt_from_verification_finding(
    receipt_id: String,
    finding: &EpiphanyRoleFindingInterpretation,
    emitted_at: String,
) -> SoulVerdictReceipt {
    SoulVerdictReceipt {
        schema_version: SOUL_VERDICT_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id,
        source_result_id: finding.runtime_result_id.clone().unwrap_or_default(),
        source_job_id: finding.runtime_job_id.clone().unwrap_or_default(),
        verdict: finding.verdict.clone().unwrap_or_else(|| "unknown".to_string()),
        summary: finding.summary.clone().unwrap_or_default(),
        evidence_ids: finding.evidence_ids.clone(),
        risks: finding.risks.clone(),
        emitted_at,
        contract: "Soul verdict emitted from a reviewed Verification lane finding; it is proof of verification judgment before Mind admission.".to_string(),
        verification_request_id: finding.verification_request_id.clone().unwrap_or_default(),
        frontier_route_id: finding.frontier_route_id.clone().unwrap_or_default(),
    }
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

    #[test]
    fn verification_finding_builds_soul_verdict_receipt() {
        let finding = EpiphanyRoleFindingInterpretation {
            verdict: Some("passed".to_string()),
            summary: Some("Checks passed.".to_string()),
            next_safe_move: Some("Proceed.".to_string()),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: vec!["src/lib.rs".to_string()],
            frontier_node_ids: Vec::new(),
            evidence_ids: vec!["ev-check".to_string()],
            artifact_refs: Vec::new(),
            runtime_result_id: Some("result-1".to_string()),
            runtime_job_id: Some("job-1".to_string()),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            state_patch: None,
            repo_model_patch: None,
            self_patch: None,
            self_persistence: None,
            job_error: None,
            item_error: None,
            verification_request_id: Some("verification-request-1".to_string()),
            frontier_route_id: Some("frontier-route-1".to_string()),
        };
        let receipt = soul_verdict_receipt_from_verification_finding(
            "soul-verdict-1".to_string(),
            &finding,
            "2026-05-30T00:00:00Z".to_string(),
        );
        assert_eq!(receipt.verdict, "passed");
        assert!(receipt.evidence_ids.contains(&"ev-check".to_string()));
        assert_eq!(receipt.verification_request_id, "verification-request-1");
        assert_eq!(receipt.frontier_route_id, "frontier-route-1");
    }
}
