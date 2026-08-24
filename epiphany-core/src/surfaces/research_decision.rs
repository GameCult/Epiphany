use anyhow::{Result, anyhow};
use crate::state_model::EpiphanyEvidenceRecord;
use crate::state_model::EpiphanyInvestigationCheckpoint;
use crate::state_model::EpiphanyObservation;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// The complete semantic cargo authored by one terminal Research pass.
///
/// This is deliberately not a generic Mind patch. Research may report exact
/// evidence, observations grounded in that evidence, and an optional
/// investigation checkpoint. The Research admission owner decides which keyed
/// Mind mutations follow from this decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EpiphanyResearchDecision {
    pub observations: Vec<EpiphanyObservation>,
    pub evidence: Vec<EpiphanyEvidenceRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub investigation_checkpoint: Option<EpiphanyInvestigationCheckpoint>,
}

impl EpiphanyResearchDecision {
    pub fn validate(&self) -> Result<()> {
        if self.evidence.is_empty() {
            return Err(anyhow!(
                "Research decision requires at least one evidence record"
            ));
        }
        if self.observations.is_empty() {
            return Err(anyhow!(
                "Research decision requires at least one observation"
            ));
        }

        let mut evidence_ids = BTreeSet::new();
        for evidence in &self.evidence {
            if evidence.id.trim().is_empty() {
                return Err(anyhow!("Research evidence identity cannot be empty"));
            }
            if !evidence_ids.insert(evidence.id.as_str()) {
                return Err(anyhow!(
                    "Research decision contains duplicate evidence identity {:?}",
                    evidence.id
                ));
            }
        }

        let mut observation_ids = BTreeSet::new();
        for observation in &self.observations {
            if observation.id.trim().is_empty() {
                return Err(anyhow!("Research observation identity cannot be empty"));
            }
            if !observation_ids.insert(observation.id.as_str()) {
                return Err(anyhow!(
                    "Research decision contains duplicate observation identity {:?}",
                    observation.id
                ));
            }
            if observation.evidence_ids.is_empty() {
                return Err(anyhow!(
                    "Research observation {:?} must cite at least one evidence identity",
                    observation.id
                ));
            }
            if observation
                .evidence_ids
                .iter()
                .any(|id| !evidence_ids.contains(id.as_str()))
            {
                return Err(anyhow!(
                    "Research observation {:?} cites evidence outside its exact decision",
                    observation.id
                ));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence(id: &str) -> EpiphanyEvidenceRecord {
        EpiphanyEvidenceRecord {
            id: id.into(),
            kind: "source".into(),
            status: "ok".into(),
            summary: "source proof".into(),
            code_refs: Vec::new(),
        }
    }

    fn observation(id: &str, evidence_ids: &[&str]) -> EpiphanyObservation {
        EpiphanyObservation {
            id: id.into(),
            summary: "observed proof".into(),
            source_kind: "research".into(),
            status: "ok".into(),
            code_refs: Vec::new(),
            evidence_ids: evidence_ids.iter().map(|id| (*id).into()).collect(),
        }
    }

    #[test]
    fn exact_evidence_closure_owns_research_decision_validity() {
        let valid = EpiphanyResearchDecision {
            evidence: vec![evidence("evidence-a")],
            observations: vec![observation("observation-a", &["evidence-a"])],
            investigation_checkpoint: None,
        };
        assert!(valid.validate().is_ok());

        let mut foreign = valid.clone();
        foreign.observations[0].evidence_ids = vec!["evidence-foreign".into()];
        assert!(foreign.validate().is_err());

        let mut duplicate = valid;
        duplicate.evidence.push(evidence("evidence-a"));
        assert!(duplicate.validate().is_err());
    }
}
