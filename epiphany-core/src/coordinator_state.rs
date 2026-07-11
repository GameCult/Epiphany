use crate::EpiphanyStateUpdate;
use crate::EpiphanyStateUpdatedField;
use crate::EpiphanyThreadStateEntry;
use crate::THREAD_STATE_KEY;
use crate::apply_epiphany_state_update;
use crate::coordinator_acceptance_cache;
use crate::epiphany_state_update_validation_errors;
use crate::read_accepted_coordinator_state;
use anyhow::Result;
use anyhow::anyhow;
use epiphany_state_model::EpiphanyThreadState;
use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub struct EpiphanyCoordinatorStateApplied {
    pub revision: u64,
    pub changed_fields: Vec<EpiphanyStateUpdatedField>,
    pub state: EpiphanyThreadState,
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
    let changed_fields = changed_fields(&update);
    let state = apply_coordinator_state_update_to_state(
        &read_coordinator_state(store)?.unwrap_or_default(),
        update,
        reference_turn_id,
    )?;
    let mut cache = coordinator_acceptance_cache(store)?;
    let entry = EpiphanyThreadStateEntry::from_state(thread_id, &state)?;
    cache.put(THREAD_STATE_KEY, &entry)?;
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
}
