use crate::EpiphanyStateUpdate;
use crate::EpiphanyStateUpdatedField;
use crate::apply_epiphany_state_update;
use crate::epiphany_state_update_validation_errors;
use crate::load_thread_state;
use crate::write_thread_state;
use anyhow::Result;
use anyhow::anyhow;
use epiphany_state_model::EpiphanyThreadState;
use std::path::Path;
use std::path::PathBuf;

/// The Epiphany-native owner of coordinator state.
///
/// Codex compatibility surfaces may translate requests into calls on this
/// service, but they do not receive a persistence hook or retain an independent
/// state opinion.
#[derive(Debug, Clone)]
pub struct EpiphanyCoordinatorService {
    thread_state_store: PathBuf,
    runtime_spine_store: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EpiphanyCoordinatorStateApplied {
    pub revision: u64,
    pub changed_fields: Vec<EpiphanyStateUpdatedField>,
    pub state: EpiphanyThreadState,
}

impl EpiphanyCoordinatorService {
    pub fn new(
        thread_state_store: impl Into<PathBuf>,
        runtime_spine_store: impl Into<PathBuf>,
    ) -> Self {
        Self {
            thread_state_store: thread_state_store.into(),
            runtime_spine_store: runtime_spine_store.into(),
        }
    }

    pub fn thread_state_store(&self) -> &Path {
        &self.thread_state_store
    }

    pub fn runtime_spine_store(&self) -> &Path {
        &self.runtime_spine_store
    }

    pub fn state(&self) -> Result<Option<EpiphanyThreadState>> {
        load_thread_state(&self.thread_state_store)
    }

    pub fn apply_state_update(
        &self,
        thread_id: &str,
        update: EpiphanyStateUpdate,
        reference_turn_id: Option<String>,
    ) -> Result<EpiphanyCoordinatorStateApplied> {
        let changed_fields = changed_fields(&update);
        let state = apply_coordinator_state_update_to_state(
            &self.state()?.unwrap_or_default(),
            update,
            reference_turn_id,
        )?;
        write_thread_state(&self.thread_state_store, thread_id, &state)?;
        Ok(EpiphanyCoordinatorStateApplied {
            revision: state.revision,
            changed_fields,
            state,
        })
    }
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

    let validation_errors = epiphany_state_update_validation_errors(current_state, &update);
    if !validation_errors.is_empty() {
        return Err(anyhow!(
            "invalid epiphany update patch: {}",
            validation_errors.join("; ")
        ));
    }

    let mut next_state = current_state.clone();
    apply_epiphany_state_update(&mut next_state, update, reference_turn_id);
    Ok(next_state)
}

fn changed_fields(update: &EpiphanyStateUpdate) -> Vec<EpiphanyStateUpdatedField> {
    let mut fields = Vec::new();
    if update.objective.is_some() {
        fields.push(EpiphanyStateUpdatedField::Objective);
    }
    if update.active_subgoal_id.is_some() {
        fields.push(EpiphanyStateUpdatedField::ActiveSubgoalId);
    }
    if update.subgoals.is_some() {
        fields.push(EpiphanyStateUpdatedField::Subgoals);
    }
    if update.invariants.is_some() {
        fields.push(EpiphanyStateUpdatedField::Invariants);
    }
    if update.graphs.is_some() {
        fields.push(EpiphanyStateUpdatedField::Graphs);
    }
    if update.graph_frontier.is_some() {
        fields.push(EpiphanyStateUpdatedField::GraphFrontier);
    }
    if update.graph_checkpoint.is_some() {
        fields.push(EpiphanyStateUpdatedField::GraphCheckpoint);
    }
    if update.scratch.is_some() {
        fields.push(EpiphanyStateUpdatedField::Scratch);
    }
    if update.investigation_checkpoint.is_some() {
        fields.push(EpiphanyStateUpdatedField::InvestigationCheckpoint);
    }
    if update.job_bindings.is_some() {
        fields.push(EpiphanyStateUpdatedField::JobBindings);
    }
    if !update.acceptance_receipts.is_empty() {
        fields.push(EpiphanyStateUpdatedField::AcceptanceReceipts);
    }
    if !update.runtime_links.is_empty() {
        fields.push(EpiphanyStateUpdatedField::RuntimeLinks);
    }
    if !update.observations.is_empty() {
        fields.push(EpiphanyStateUpdatedField::Observations);
    }
    if !update.evidence.is_empty() {
        fields.push(EpiphanyStateUpdatedField::Evidence);
    }
    if update.churn.is_some() {
        fields.push(EpiphanyStateUpdatedField::Churn);
    }
    if update.mode.is_some() {
        fields.push(EpiphanyStateUpdatedField::Mode);
    }
    if update.planning.is_some() {
        fields.push(EpiphanyStateUpdatedField::Planning);
    }
    fields
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_service_owns_revision_gated_thread_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let service = EpiphanyCoordinatorService::new(
            temp.path().join("thread-state.msgpack"),
            temp.path().join("runtime-spine.msgpack"),
        );

        let applied = service.apply_state_update(
            "coordinator-session-1",
            EpiphanyStateUpdate {
                expected_revision: Some(0),
                objective: Some("Remove Codex from coordinator authority.".to_string()),
                ..Default::default()
            },
            Some("operator-turn-1".to_string()),
        )?;

        assert_eq!(applied.revision, 1);
        assert_eq!(
            applied.changed_fields,
            vec![EpiphanyStateUpdatedField::Objective]
        );
        assert_eq!(
            service.state()?.and_then(|state| state.objective),
            Some("Remove Codex from coordinator authority.".to_string())
        );
        Ok(())
    }

    #[test]
    fn rejected_revision_cannot_mutate_or_repair_native_state() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let service = EpiphanyCoordinatorService::new(
            temp.path().join("thread-state.msgpack"),
            temp.path().join("runtime-spine.msgpack"),
        );
        service.apply_state_update(
            "coordinator-session-1",
            EpiphanyStateUpdate {
                expected_revision: Some(0),
                objective: Some("Canonical objective".to_string()),
                ..Default::default()
            },
            None,
        )?;

        let error = service
            .apply_state_update(
                "coordinator-session-1",
                EpiphanyStateUpdate {
                    expected_revision: Some(0),
                    objective: Some("Stale overwrite".to_string()),
                    ..Default::default()
                },
                None,
            )
            .expect_err("stale revision must be refused");

        assert!(error.to_string().contains("revision mismatch"));
        let state = service.state()?.expect("canonical state must remain");
        assert_eq!(state.revision, 1);
        assert_eq!(state.objective.as_deref(), Some("Canonical objective"));
        Ok(())
    }
}
