use crate::EpiphanyResearchDecision;
use crate::EpiphanyRoleFindingInterpretation;
use cultcache_rs::DatabaseEntry;

pub const EYES_SOURCE_LOOKUP_RECEIPT_TYPE: &str = "epiphany.eyes.source_lookup_receipt";
pub const EYES_EVIDENCE_PACKET_TYPE: &str = "epiphany.eyes.evidence_packet";
pub const EYES_SOURCE_LOOKUP_RECEIPT_SCHEMA_VERSION: &str =
    "epiphany.eyes.source_lookup_receipt.v0";
pub const EYES_EVIDENCE_PACKET_SCHEMA_VERSION: &str = "epiphany.eyes.evidence_packet.v2";

#[derive(Debug, Clone, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.eyes.source_lookup_receipt",
    schema = "EyesSourceLookupReceipt"
)]
pub struct EyesSourceLookupReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub source_job_id: String,
    #[cultcache(key = 3)]
    pub substrate_grant_receipt_id: String,
    #[cultcache(key = 4)]
    pub tool_intent_id: String,
    #[cultcache(key = 5)]
    pub tool_receipt_id: String,
    #[cultcache(key = 6)]
    pub provider: String,
    #[cultcache(key = 7)]
    pub repository: String,
    #[cultcache(key = 8)]
    pub revision: String,
    #[cultcache(key = 9)]
    pub path: String,
    #[cultcache(key = 10)]
    pub source_ref: String,
    #[cultcache(key = 11)]
    pub content_sha256: String,
    #[cultcache(key = 12)]
    pub byte_count: u64,
    #[cultcache(key = 13)]
    pub observed_at: String,
    #[cultcache(key = 14)]
    pub contract: String,
}

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
    pub evidence_ids: Vec<String>,
    #[cultcache(key = 5)]
    pub observation_ids: Vec<String>,
    #[cultcache(key = 6)]
    pub source_refs: Vec<String>,
    #[cultcache(key = 7)]
    pub summary: String,
    #[cultcache(key = 8)]
    pub uncertainty: String,
    #[cultcache(key = 9)]
    pub emitted_at: String,
    #[cultcache(key = 10)]
    pub contract: String,
    #[cultcache(key = 11, default)]
    pub source_lookup_receipt_ids: Vec<String>,
    #[cultcache(key = 12)]
    pub research_request_id: String,
    #[cultcache(key = 13)]
    pub decision_context_id: String,
}

pub fn eyes_evidence_packet_from_research_finding(
    packet_id: String,
    research_request_id: String,
    decision_context_id: String,
    finding: &EpiphanyRoleFindingInterpretation,
    decision: &EpiphanyResearchDecision,
    source_lookups: &[EyesSourceLookupReceipt],
    emitted_at: String,
) -> EyesEvidencePacket {
    let evidence_ids = decision
        .evidence
        .iter()
        .filter_map(|evidence| non_empty_string(&evidence.id))
        .collect::<Vec<_>>();
    let observation_ids = decision
        .observations
        .iter()
        .filter_map(|observation| non_empty_string(&observation.id))
        .collect::<Vec<_>>();
    let mut source_refs = Vec::new();
    for evidence in &decision.evidence {
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
    for lookup in source_lookups {
        push_unique(&mut source_refs, lookup.source_ref.clone());
    }
    EyesEvidencePacket {
        schema_version: EYES_EVIDENCE_PACKET_SCHEMA_VERSION.to_string(),
        packet_id,
        source_result_id: finding.runtime_result_id.clone().unwrap_or_default(),
        source_job_id: finding.runtime_job_id.clone().unwrap_or_default(),
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
        source_lookup_receipt_ids: source_lookups
            .iter()
            .map(|lookup| lookup.receipt_id.clone())
            .collect(),
        research_request_id,
        decision_context_id,
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
