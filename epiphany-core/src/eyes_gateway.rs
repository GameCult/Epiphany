use crate::EpiphanyRoleFindingInterpretation;
use crate::EpiphanyRoleStatePatchDocument;
use cultcache_rs::DatabaseEntry;
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

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(type = "epiphany.eyes.evidence_packet", schema = "EyesEvidencePacket")]
pub struct EyesEvidencePacket {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub packet_id: String,
    #[cultcache(key = 2)]
    pub source_result_id: String,
    #[cultcache(key = 3)]
    pub source_job_id: String,
    #[cultcache(key = 4)]
    pub source_role_id: String,
    #[cultcache(key = 5)]
    pub evidence_ids: Vec<String>,
    #[cultcache(key = 6)]
    pub observation_ids: Vec<String>,
    #[cultcache(key = 7)]
    pub source_refs: Vec<String>,
    #[cultcache(key = 8)]
    pub summary: String,
    #[cultcache(key = 9)]
    pub uncertainty: String,
    #[cultcache(key = 10)]
    pub emitted_at: String,
    #[cultcache(key = 11)]
    pub contract: String,
}

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
                "Substrate Gate grants substrate access; Eyes turns inspected material into citable evidence packets.".to_string(),
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
                "Source lookup receipts prove what was searched or inspected, under which Substrate Gate grant, before another organ cites it.".to_string(),
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
                "Evidence packets carry provenance, uncertainty, and source refs for Imagination, Mind, Hands, Soul, Persona, Self, Modeling, and Continuity protocols.".to_string(),
            ],
        },
    ]
}

pub fn eyes_evidence_packet_from_research_finding(
    packet_id: String,
    finding: &EpiphanyRoleFindingInterpretation,
    patch: &EpiphanyRoleStatePatchDocument,
    emitted_at: String,
) -> EyesEvidencePacket {
    let evidence_ids = patch
        .evidence
        .iter()
        .filter_map(|evidence| non_empty_string(&evidence.id))
        .collect::<Vec<_>>();
    let observation_ids = patch
        .observations
        .iter()
        .filter_map(|observation| non_empty_string(&observation.id))
        .collect::<Vec<_>>();
    let mut source_refs = Vec::new();
    for evidence in &patch.evidence {
        for code_ref in &evidence.code_refs {
            let mut rendered = code_ref.path.display().to_string();
            if let Some(start_line) = code_ref.start_line {
                rendered.push(':');
                rendered.push_str(&start_line.to_string());
            }
            if let Some(end_line) = code_ref.end_line
                && Some(end_line) != code_ref.start_line
            {
                rendered.push('-');
                rendered.push_str(&end_line.to_string());
            }
            if let Some(symbol) = code_ref.symbol.as_deref().filter(|value| !value.is_empty()) {
                rendered.push('#');
                rendered.push_str(symbol);
            }
            push_unique(&mut source_refs, rendered);
        }
    }
    EyesEvidencePacket {
        schema_version: EYES_EVIDENCE_PACKET_SCHEMA_VERSION.to_string(),
        packet_id,
        source_result_id: finding.runtime_result_id.clone().unwrap_or_default(),
        source_job_id: finding.runtime_job_id.clone().unwrap_or_default(),
        source_role_id: "research".to_string(),
        evidence_ids,
        observation_ids,
        source_refs,
        summary: finding.summary.clone().unwrap_or_default(),
        uncertainty: if finding.evidence_gaps.is_empty() && finding.risks.is_empty() {
            "no declared research gaps or risks".to_string()
        } else {
            finding
                .evidence_gaps
                .iter()
                .chain(finding.risks.iter())
                .cloned()
                .collect::<Vec<_>>()
                .join("; ")
        },
        emitted_at,
        contract: "Eyes packet emitted from a reviewed Research lane finding; it makes the source-gathering evidence claim citable before Mind admission.".to_string(),
    }
}

fn non_empty_string(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn push_unique(out: &mut Vec<String>, value: String) {
    if !out.contains(&value) {
        out.push(value);
    }
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

    #[test]
    fn research_finding_builds_evidence_packet() {
        let finding = EpiphanyRoleFindingInterpretation {
            verdict: Some("evidence-ready".to_string()),
            summary: Some("Found source proof.".to_string()),
            next_safe_move: Some("Review evidence.".to_string()),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: vec!["src/lib.rs".to_string()],
            frontier_node_ids: Vec::new(),
            evidence_ids: vec!["ev-source".to_string()],
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
            verification_request_id: None,
            frontier_route_id: None,
        };
        let mut patch = EpiphanyRoleStatePatchDocument::default();
        patch
            .evidence
            .push(epiphany_state_model::EpiphanyEvidenceRecord {
                id: "ev-source".to_string(),
                kind: "source".to_string(),
                status: "ok".to_string(),
                summary: "Source proof.".to_string(),
                code_refs: vec![epiphany_state_model::EpiphanyCodeRef {
                    path: "src/lib.rs".into(),
                    start_line: Some(12),
                    end_line: None,
                    symbol: Some("thing".to_string()),
                    note: None,
                }],
            });
        patch
            .observations
            .push(epiphany_state_model::EpiphanyObservation {
                id: "obs-source".to_string(),
                summary: "Observed source proof.".to_string(),
                source_kind: "research".to_string(),
                status: "ok".to_string(),
                code_refs: Vec::new(),
                evidence_ids: vec!["ev-source".to_string()],
            });

        let packet = eyes_evidence_packet_from_research_finding(
            "eyes-packet-1".to_string(),
            &finding,
            &patch,
            "2026-05-30T00:00:00Z".to_string(),
        );
        assert_eq!(packet.source_role_id, "research");
        assert!(packet.evidence_ids.contains(&"ev-source".to_string()));
        assert!(packet.observation_ids.contains(&"obs-source".to_string()));
        assert!(
            packet
                .source_refs
                .iter()
                .any(|item| item.contains("src/lib.rs"))
        );
    }
}
