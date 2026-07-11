use crate::EpiphanyCoordinatorReorientResultSnapshot;
use crate::EpiphanyCoordinatorRoleResultSnapshot;
use crate::EpiphanyCoordinatorStateApplied;
use crate::EpiphanyJobInterruptRequest;
use crate::EpiphanyJobInterruptResult;
use crate::EpiphanyJobLaunchRequest;
use crate::EpiphanyJobLaunchResult;
use crate::EpiphanyNativeReorientAcceptance;
use crate::EpiphanyNativeRoleAcceptance;
use crate::EpiphanyRoleResultRoleId;
use crate::EpiphanyStateUpdate;
use crate::accept_coordinator_reorient_finding;
use crate::accept_coordinator_role_finding;
use crate::apply_coordinator_state_update;
use crate::apply_coordinator_state_update_from_state;
use crate::commit_coordinator_job_launch;
use crate::interrupt_coordinator_job;
use crate::plan_coordinator_job_launch;
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

    pub fn apply_state_update_from(
        &self,
        thread_id: &str,
        current_state: &EpiphanyThreadState,
        update: EpiphanyStateUpdate,
        reference_turn_id: Option<String>,
    ) -> Result<EpiphanyCoordinatorStateApplied> {
        apply_coordinator_state_update_from_state(
            &self.thread_state_store,
            thread_id,
            current_state,
            update,
            reference_turn_id,
        )
    }

    pub fn launch_job(
        &self,
        thread_id: &str,
        state: &EpiphanyThreadState,
        request: &EpiphanyJobLaunchRequest,
        launcher_job_id: String,
        backend_job_id: String,
        created_at: String,
    ) -> Result<EpiphanyJobLaunchResult> {
        let plan = plan_coordinator_job_launch(
            state,
            request,
            &self.runtime_spine_store,
            launcher_job_id,
            backend_job_id,
        )?;
        commit_coordinator_job_launch(
            &self.runtime_spine_store,
            thread_id,
            state,
            request,
            &plan,
            created_at,
        )
    }

    pub fn accept_role(
        &self,
        thread_id: &str,
        state: &EpiphanyThreadState,
        role_id: EpiphanyRoleResultRoleId,
        binding_id: &str,
        expected_revision: Option<u64>,
        reference_turn_id: Option<String>,
        accepted_at: String,
        nonce: &str,
    ) -> Result<EpiphanyNativeRoleAcceptance> {
        accept_coordinator_role_finding(
            &self.runtime_spine_store,
            thread_id,
            state,
            role_id,
            binding_id,
            expected_revision,
            reference_turn_id,
            accepted_at,
            nonce,
        )
    }

    pub fn accept_reorient(
        &self,
        thread_id: &str,
        state: &EpiphanyThreadState,
        binding_id: &str,
        expected_revision: Option<u64>,
        reference_turn_id: Option<String>,
        accepted_at: String,
        nonce: &str,
        update_scratch: bool,
        update_investigation_checkpoint: bool,
    ) -> Result<EpiphanyNativeReorientAcceptance> {
        accept_coordinator_reorient_finding(
            &self.runtime_spine_store,
            thread_id,
            state,
            binding_id,
            expected_revision,
            reference_turn_id,
            accepted_at,
            nonce,
            update_scratch,
            update_investigation_checkpoint,
        )
    }

    pub fn interrupt_job(
        &self,
        thread_id: &str,
        state: &EpiphanyThreadState,
        request: EpiphanyJobInterruptRequest,
    ) -> Result<EpiphanyJobInterruptResult> {
        interrupt_coordinator_job(&self.thread_state_store, thread_id, state, request)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn facade_contains_no_coordinator_policy() {
        let source = include_str!("coordinator_service.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        assert!(
            production.lines().count() < 210,
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
