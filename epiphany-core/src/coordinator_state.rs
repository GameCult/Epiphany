use crate::EpiphanyStateUpdate;
use crate::EpiphanyStateUpdatedField;
use crate::apply_epiphany_state_update;
use crate::coordinator_state_transaction::commit_coordinator_state_transaction;
use crate::coordinator_state_transaction::open_coordinator_state_transaction;
use crate::epiphany_state_update_validation_errors;
use crate::read_accepted_coordinator_state;
use anyhow::Result;
use anyhow::anyhow;
use epiphany_state_model::{EpiphanyEvidenceRecord, EpiphanyThreadState};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct EpiphanyCoordinatorStateApplied {
    pub revision: u64,
    pub changed_fields: Vec<EpiphanyStateUpdatedField>,
    pub state: EpiphanyThreadState,
}

pub const SUPERVISOR_MODELING_ACCEPTANCE_CORRECTION_REASON: &str =
    "missing-typed-future-frontier";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanySupervisorAcceptanceCorrectionReceipt {
    pub correction_id: String,
    pub thread_id: String,
    pub source_actor_id: String,
    pub prior_receipt_id: String,
    pub prior_receipt_sha256: String,
    pub result_id: String,
    pub job_id: String,
    pub binding_id: String,
    pub role_id: String,
    pub reason: String,
    pub corrected_at: String,
    pub prior_revision: u64,
    pub corrected_revision: u64,
}

pub fn apply_supervisor_modeling_acceptance_correction(
    store: &Path,
    thread_id: &str,
    expected_revision: u64,
    prior_receipt_id: &str,
    result_id: &str,
    source_actor_id: &str,
    reason: &str,
    corrected_at: &str,
) -> Result<EpiphanySupervisorAcceptanceCorrectionReceipt> {
    for (name, value) in [
        ("thread_id", thread_id),
        ("prior_receipt_id", prior_receipt_id),
        ("result_id", result_id),
        ("source_actor_id", source_actor_id),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("supervisor acceptance correction requires {name}"));
        }
    }
    if reason != SUPERVISOR_MODELING_ACCEPTANCE_CORRECTION_REASON {
        return Err(anyhow!(
            "unsupported supervisor Modeling acceptance correction reason"
        ));
    }
    chrono::DateTime::parse_from_rfc3339(corrected_at)
        .map_err(|_| anyhow!("supervisor acceptance correction timestamp must be RFC3339"))?;

    let current = read_coordinator_state(store)?
        .ok_or_else(|| anyhow!("supervisor acceptance correction requires coordinator state"))?;
    if current.revision != expected_revision {
        return Err(anyhow!(
            "supervisor acceptance correction revision mismatch: expected {expected_revision}, found {}",
            current.revision
        ));
    }
    let receipt_index = current
        .acceptance_receipts
        .iter()
        .position(|receipt| receipt.id == prior_receipt_id)
        .ok_or_else(|| anyhow!("supervisor acceptance correction receipt is absent"))?;
    let prior = current.acceptance_receipts[receipt_index].clone();
    if prior.result_id != result_id
        || prior.role_id != "modeling"
        || prior.surface != "roleAccept"
        || prior.status != "accepted"
    {
        return Err(anyhow!(
            "supervisor acceptance correction requires the exact accepted Modeling receipt"
        ));
    }
    let prior_receipt_sha256 = format!(
        "{:x}",
        Sha256::digest(rmp_serde::to_vec_named(&prior)?)
    );
    let semantic = format!(
        "{thread_id}\0{source_actor_id}\0{prior_receipt_id}\0{prior_receipt_sha256}\0{result_id}\0{reason}\0{corrected_at}\0{expected_revision}"
    );
    let correction_id = format!(
        "supervisor-acceptance-correction-{:x}",
        Sha256::digest(semantic.as_bytes())
    );
    let mut next = current.clone();
    next.acceptance_receipts.remove(receipt_index);
    next.recent_evidence.insert(
        0,
        EpiphanyEvidenceRecord {
            id: correction_id.clone(),
            kind: "supervisor-runtime-repair".into(),
            status: "ok".into(),
            summary: format!(
                "Superseded routing effect of accepted Modeling receipt {prior_receipt_id} ({prior_receipt_sha256}) for result {result_id}; reason={reason}. Immutable result and admitted RepoModel remain evidence."
            ),
            code_refs: Vec::new(),
        },
    );
    next.revision = next.revision.saturating_add(1);
    next.last_updated_turn_id = Some(correction_id.clone());
    let mut transaction = open_coordinator_state_transaction(store, &current)?;
    commit_coordinator_state_transaction(&mut transaction, thread_id, &next, Vec::new(), Vec::new())?;
    Ok(EpiphanySupervisorAcceptanceCorrectionReceipt {
        correction_id,
        thread_id: thread_id.to_string(),
        source_actor_id: source_actor_id.to_string(),
        prior_receipt_id: prior_receipt_id.to_string(),
        prior_receipt_sha256,
        result_id: result_id.to_string(),
        job_id: prior.job_id,
        binding_id: prior.binding_id,
        role_id: prior.role_id,
        reason: reason.to_string(),
        corrected_at: corrected_at.to_string(),
        prior_revision: current.revision,
        corrected_revision: next.revision,
    })
}

pub fn read_coordinator_state(store: &Path) -> Result<Option<EpiphanyThreadState>> {
    read_accepted_coordinator_state(store)
}

pub fn apply_coordinator_state_update(
    store: &Path,
    thread_id: &str,
    update: EpiphanyStateUpdate,
    reference_turn_id: Option<String>,
) -> Result<EpiphanyCoordinatorStateApplied> {
    let current = read_coordinator_state(store)?.unwrap_or_default();
    apply_coordinator_state_update_from_state(store, thread_id, &current, update, reference_turn_id)
}

pub fn apply_coordinator_state_update_from_state(
    store: &Path,
    thread_id: &str,
    current_state: &EpiphanyThreadState,
    update: EpiphanyStateUpdate,
    reference_turn_id: Option<String>,
) -> Result<EpiphanyCoordinatorStateApplied> {
    let mut cache = open_coordinator_state_transaction(store, current_state)?;
    let changed_fields = changed_fields(&update);
    let state = apply_coordinator_state_update_to_state(current_state, update, reference_turn_id)?;
    commit_coordinator_state_transaction(&mut cache, thread_id, &state, Vec::new(), Vec::new())?;
    Ok(EpiphanyCoordinatorStateApplied {
        revision: state.revision,
        changed_fields,
        state,
    })
}

pub fn apply_coordinator_state_update_to_state(
    current_state: &EpiphanyThreadState,
    update: EpiphanyStateUpdate,
    reference_turn_id: Option<String>,
) -> Result<EpiphanyThreadState> {
    if update.is_empty() {
        return Err(anyhow!(
            "epiphany update patch must contain at least one mutation"
        ));
    }
    if let Some(expected_revision) = update.expected_revision
        && expected_revision != current_state.revision
    {
        return Err(anyhow!(
            "epiphany state revision mismatch: expected {expected_revision}, found {}",
            current_state.revision
        ));
    }
    let errors = epiphany_state_update_validation_errors(current_state, &update);
    if !errors.is_empty() {
        return Err(anyhow!(
            "invalid epiphany update patch: {}",
            errors.join("; ")
        ));
    }
    let mut next_state = current_state.clone();
    apply_epiphany_state_update(&mut next_state, update, reference_turn_id);
    Ok(next_state)
}

pub(crate) fn changed_fields(update: &EpiphanyStateUpdate) -> Vec<EpiphanyStateUpdatedField> {
    let mut fields = Vec::new();
    macro_rules! field {
        ($condition:expr, $field:ident) => {
            if $condition {
                fields.push(EpiphanyStateUpdatedField::$field);
            }
        };
    }
    field!(update.objective.is_some(), Objective);
    field!(update.active_subgoal_id.is_some(), ActiveSubgoalId);
    field!(update.subgoals.is_some(), Subgoals);
    field!(update.invariants.is_some(), Invariants);
    field!(update.graphs.is_some(), Graphs);
    field!(update.graph_frontier.is_some(), GraphFrontier);
    field!(update.graph_checkpoint.is_some(), GraphCheckpoint);
    field!(update.scratch.is_some(), Scratch);
    field!(
        update.investigation_checkpoint.is_some(),
        InvestigationCheckpoint
    );
    field!(update.job_bindings.is_some(), JobBindings);
    field!(!update.acceptance_receipts.is_empty(), AcceptanceReceipts);
    field!(!update.runtime_links.is_empty(), RuntimeLinks);
    field!(!update.observations.is_empty(), Observations);
    field!(!update.evidence.is_empty(), Evidence);
    field!(update.churn.is_some(), Churn);
    field!(update.mode.is_some(), Mode);
    field!(update.planning.is_some(), Planning);
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_owner_writes_and_refuses_stale_revision() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("thread-state.msgpack");
        let first = apply_coordinator_state_update(
            &store,
            "session-1",
            EpiphanyStateUpdate {
                expected_revision: Some(0),
                objective: Some("Canonical objective".to_string()),
                ..Default::default()
            },
            None,
        )?;
        assert_eq!(first.revision, 1);
        assert!(
            apply_coordinator_state_update(
                &store,
                "session-1",
                EpiphanyStateUpdate {
                    expected_revision: Some(0),
                    objective: Some("Stale overwrite".to_string()),
                    ..Default::default()
                },
                None
            )
            .is_err()
        );
        assert_eq!(
            read_coordinator_state(&store)?
                .unwrap()
                .objective
                .as_deref(),
            Some("Canonical objective")
        );
        Ok(())
    }

    #[test]
    fn imported_host_state_seeds_once_then_cannot_override_native_truth() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("imported-state.msgpack");
        let imported = EpiphanyThreadState {
            revision: 4,
            objective: Some("Imported rollout state".to_string()),
            ..Default::default()
        };
        let applied = apply_coordinator_state_update_from_state(
            &store,
            "thread-1",
            &imported,
            EpiphanyStateUpdate {
                expected_revision: Some(4),
                objective: Some("Native authority".to_string()),
                ..Default::default()
            },
            None,
        )?;
        assert_eq!(applied.revision, 5);
        assert!(
            apply_coordinator_state_update_from_state(
                &store,
                "thread-1",
                &imported,
                EpiphanyStateUpdate {
                    expected_revision: None,
                    objective: Some("Stale host overwrite".to_string()),
                    ..Default::default()
                },
                None,
            )
            .is_err()
        );
        assert_eq!(
            read_coordinator_state(&store)?
                .unwrap()
                .objective
                .as_deref(),
            Some("Native authority")
        );
        Ok(())
    }

    #[test]
    fn supervisor_correction_removes_only_legacy_modeling_routing_authority() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("supervisor-correction.cc");
        let accepted = epiphany_state_model::EpiphanyAcceptanceReceipt {
            id: "accept-modeling-legacy".into(),
            result_id: "result-modeling-legacy".into(),
            job_id: "job-modeling-legacy".into(),
            binding_id: "epiphany-role-modeling".into(),
            surface: "roleAccept".into(),
            role_id: "modeling".into(),
            status: "accepted".into(),
            accepted_at: "2026-08-08T15:00:00Z".into(),
            summary: Some("Legacy result omitted its now-required frontier.".into()),
            ..Default::default()
        };
        let seeded = apply_coordinator_state_update(
            &store,
            "thread-repair",
            EpiphanyStateUpdate {
                expected_revision: Some(0),
                objective: Some("Preserve the admitted Body map.".into()),
                acceptance_receipts: vec![accepted.clone()],
                ..Default::default()
            },
            None,
        )?;
        let before_hostile = std::fs::read(&store)?;
        assert!(
            apply_supervisor_modeling_acceptance_correction(
                &store,
                "thread-repair",
                seeded.revision,
                &accepted.id,
                &accepted.result_id,
                "operator-enginseer",
                "invent-a-frontier",
                "2026-08-08T15:01:00Z",
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&store)?, before_hostile);

        let correction = apply_supervisor_modeling_acceptance_correction(
            &store,
            "thread-repair",
            seeded.revision,
            &accepted.id,
            &accepted.result_id,
            "operator-enginseer",
            SUPERVISOR_MODELING_ACCEPTANCE_CORRECTION_REASON,
            "2026-08-08T15:01:00Z",
        )?;
        let repaired = read_coordinator_state(&store)?.expect("repaired state");
        assert_eq!(correction.corrected_revision, seeded.revision + 1);
        assert!(repaired.acceptance_receipts.is_empty());
        assert_eq!(repaired.objective.as_deref(), seeded.state.objective.as_deref());
        assert_eq!(repaired.recent_evidence[0].id, correction.correction_id);
        assert!(
            repaired.recent_evidence[0]
                .summary
                .contains(&correction.prior_receipt_sha256)
        );
        Ok(())
    }
}
