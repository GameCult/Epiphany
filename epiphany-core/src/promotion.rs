use codex_protocol::protocol::EpiphanyEvidenceRecord;
use codex_protocol::protocol::EpiphanyObservation;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct EpiphanyPromotionInput {
    pub has_state_replacements: bool,
    pub observations: Vec<EpiphanyObservation>,
    pub evidence: Vec<EpiphanyEvidenceRecord>,
    pub verifier_evidence: EpiphanyEvidenceRecord,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyPromotionDecision {
    pub accepted: bool,
    pub reasons: Vec<String>,
}

pub fn evaluate_promotion(input: EpiphanyPromotionInput) -> EpiphanyPromotionDecision {
    let mut reasons = Vec::new();

    if !input.has_state_replacements && input.observations.is_empty() && input.evidence.is_empty() {
        reasons.push("promotion patch must contain at least one mutation".to_string());
    }

    validate_evidence_record(&input.verifier_evidence, "verifierEvidence", &mut reasons);
    if !is_accepting_status(&input.verifier_evidence.status) {
        reasons.push(format!(
            "verifierEvidence.status must be accepting, got {:?}",
            input.verifier_evidence.status
        ));
    }

    let mut evidence_ids = HashSet::new();
    evidence_ids.insert(input.verifier_evidence.id.as_str());
    for evidence in &input.evidence {
        validate_evidence_record(evidence, "patch.evidence", &mut reasons);
        if !evidence_ids.insert(evidence.id.as_str()) {
            reasons.push(format!("duplicate evidence id {:?}", evidence.id));
        }
    }

    let mut observation_ids = HashSet::new();
    for observation in &input.observations {
        validate_observation(observation, &evidence_ids, &mut reasons);
        if !observation_ids.insert(observation.id.as_str()) {
            reasons.push(format!("duplicate observation id {:?}", observation.id));
        }
    }

    EpiphanyPromotionDecision {
        accepted: reasons.is_empty(),
        reasons,
    }
}

fn validate_evidence_record(
    evidence: &EpiphanyEvidenceRecord,
    label: &str,
    reasons: &mut Vec<String>,
) {
    require_nonempty(&evidence.id, &format!("{label}.id"), reasons);
    require_nonempty(&evidence.kind, &format!("{label}.kind"), reasons);
    require_nonempty(&evidence.status, &format!("{label}.status"), reasons);
    require_nonempty(&evidence.summary, &format!("{label}.summary"), reasons);
}

fn validate_observation(
    observation: &EpiphanyObservation,
    evidence_ids: &HashSet<&str>,
    reasons: &mut Vec<String>,
) {
    require_nonempty(&observation.id, "patch.observations.id", reasons);
    require_nonempty(&observation.summary, "patch.observations.summary", reasons);
    require_nonempty(
        &observation.source_kind,
        "patch.observations.source_kind",
        reasons,
    );
    require_nonempty(&observation.status, "patch.observations.status", reasons);
    if observation.evidence_ids.is_empty() {
        reasons.push(format!(
            "observation {:?} must cite at least one evidence id",
            observation.id
        ));
    }
    for evidence_id in &observation.evidence_ids {
        if !evidence_ids.contains(evidence_id.as_str()) {
            reasons.push(format!(
                "observation {:?} cites missing evidence id {:?}",
                observation.id, evidence_id
            ));
        }
    }
}

fn require_nonempty(value: &str, label: &str, reasons: &mut Vec<String>) {
    if value.trim().is_empty() {
        reasons.push(format!("{label} must not be empty"));
    }
}

fn is_accepting_status(status: &str) -> bool {
    matches!(
        status.trim().to_ascii_lowercase().as_str(),
        "ok" | "accepted" | "verified" | "pass" | "passed"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn verifier(status: &str) -> EpiphanyEvidenceRecord {
        EpiphanyEvidenceRecord {
            id: "ev-verifier".to_string(),
            kind: "verification".to_string(),
            status: status.to_string(),
            summary: "Verifier accepted the promotion".to_string(),
            code_refs: Vec::new(),
        }
    }

    #[test]
    fn evaluate_promotion_accepts_verified_evidence_linked_observation() {
        let decision = evaluate_promotion(EpiphanyPromotionInput {
            has_state_replacements: false,
            observations: vec![EpiphanyObservation {
                id: "obs-1".to_string(),
                summary: "Smoke passed".to_string(),
                source_kind: "smoke".to_string(),
                status: "ok".to_string(),
                code_refs: Vec::new(),
                evidence_ids: vec!["ev-1".to_string()],
            }],
            evidence: vec![EpiphanyEvidenceRecord {
                id: "ev-1".to_string(),
                kind: "verification".to_string(),
                status: "ok".to_string(),
                summary: "Smoke returned revision 1".to_string(),
                code_refs: Vec::new(),
            }],
            verifier_evidence: verifier("ok"),
        });

        assert!(decision.accepted);
        assert!(decision.reasons.is_empty());
    }

    #[test]
    fn evaluate_promotion_rejects_non_accepting_verifier_status() {
        let decision = evaluate_promotion(EpiphanyPromotionInput {
            has_state_replacements: true,
            observations: Vec::new(),
            evidence: Vec::new(),
            verifier_evidence: verifier("failed"),
        });

        assert!(!decision.accepted);
        assert!(
            decision
                .reasons
                .iter()
                .any(|reason| reason.contains("verifierEvidence.status"))
        );
    }

    #[test]
    fn evaluate_promotion_rejects_observation_without_existing_evidence() {
        let decision = evaluate_promotion(EpiphanyPromotionInput {
            has_state_replacements: false,
            observations: vec![EpiphanyObservation {
                id: "obs-1".to_string(),
                summary: "Smoke passed".to_string(),
                source_kind: "smoke".to_string(),
                status: "ok".to_string(),
                code_refs: Vec::new(),
                evidence_ids: vec!["ev-missing".to_string()],
            }],
            evidence: Vec::new(),
            verifier_evidence: verifier("ok"),
        });

        assert!(!decision.accepted);
        assert!(
            decision
                .reasons
                .iter()
                .any(|reason| reason.contains("missing evidence id"))
        );
    }
}
