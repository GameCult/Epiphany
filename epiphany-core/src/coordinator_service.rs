use crate::EpiphanyCoordinatorReorientResultSnapshot;
use crate::EpiphanyCoordinatorRoleResultSnapshot;
use crate::EpiphanyCoordinatorStateApplied;
use crate::EpiphanyRoleResultRoleId;
use crate::EpiphanyStateUpdate;
use crate::apply_coordinator_state_update;
use crate::read_coordinator_state;
use crate::read_reorient_result_snapshot;
use crate::read_role_result_snapshot;
use anyhow::Result;
use epiphany_state_model::EpiphanyThreadState;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct EpiphanyCoordinatorService {
    thread_state_store: PathBuf,
    runtime_spine_store: PathBuf,
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
        read_coordinator_state(&self.thread_state_store)
    }

    pub fn role_result(
        &self,
        role_id: EpiphanyRoleResultRoleId,
        binding_id: &str,
    ) -> Result<EpiphanyCoordinatorRoleResultSnapshot> {
        let state = self.state()?;
        Ok(read_role_result_snapshot(
            state.as_ref(),
            Some(&self.runtime_spine_store),
            role_id,
            binding_id,
        ))
    }

    pub fn reorient_result(
        &self,
        binding_id: &str,
    ) -> Result<EpiphanyCoordinatorReorientResultSnapshot> {
        let state = self.state()?;
        Ok(read_reorient_result_snapshot(
            state.as_ref(),
            Some(&self.runtime_spine_store),
            binding_id,
        ))
    }

    pub fn apply_state_update(
        &self,
        thread_id: &str,
        update: EpiphanyStateUpdate,
        reference_turn_id: Option<String>,
    ) -> Result<EpiphanyCoordinatorStateApplied> {
        apply_coordinator_state_update(
            &self.thread_state_store,
            thread_id,
            update,
            reference_turn_id,
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn facade_contains_no_coordinator_policy() {
        let source = include_str!("coordinator_service.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        assert!(
            production.lines().count() < 90,
            "coordinator facade regrew into a host brain"
        );
        for forbidden in [
            "epiphany_state_update_validation_errors",
            "runtime_worker_launch_request",
            "mind_review_role_acceptance",
            "build_role_acceptance_bundle",
            "serde_json",
            "thread/epiphany/",
        ] {
            assert!(
                !production.contains(forbidden),
                "coordinator facade contains policy/protocol marker {forbidden:?}"
            );
        }
    }
}
