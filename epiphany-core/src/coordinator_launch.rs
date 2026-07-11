use crate::*;
use anyhow::{Result, anyhow};
use epiphany_state_model::{EpiphanyRuntimeLink, EpiphanyThreadState};
use std::path::Path;

#[derive(Debug, Clone)]
pub struct EpiphanyCoordinatorJobLaunchPlan {
    pub launcher_job_id: String,
    pub backend_job_id: String,
    pub planning_state: EpiphanyThreadState,
    pub heartbeat_plan: RuntimeSpineHeartbeatLaunchPlan,
    pub state_update: EpiphanyStateUpdate,
}

pub fn plan_coordinator_job_launch(
    state: &EpiphanyThreadState,
    request: &EpiphanyJobLaunchRequest,
    runtime_store: &Path,
    launcher_job_id: String,
    backend_job_id: String,
) -> Result<EpiphanyCoordinatorJobLaunchPlan> {
    if let Some(expected) = request.expected_revision
        && expected != state.revision
    {
        return Err(anyhow!(
            "epiphany state revision mismatch: expected {expected}, found {}",
            state.revision
        ));
    }
    let completed_prior_link =
        terminal_runtime_link_for_binding(state, &request.binding_id, runtime_store)?;
    let mut planning_state = state.clone();
    if let Some(link) = completed_prior_link.clone() {
        planning_state.runtime_links.insert(0, link);
    }
    let heartbeat_plan = plan_runtime_spine_heartbeat_launch(
        &planning_state,
        RuntimeSpineHeartbeatLaunchPlanOptions {
            binding_id: request.binding_id.clone(),
            kind: request.kind,
            scope: request.scope.clone(),
            owner_role: request.owner_role.clone(),
            authority_scope: request.authority_scope.clone(),
            linked_subgoal_ids: request.linked_subgoal_ids.clone(),
            linked_graph_node_ids: request.linked_graph_node_ids.clone(),
            instruction: request.instruction.clone(),
            launch_document: request.launch_document.clone(),
            output_contract_id: request.output_contract_id.clone(),
            organ_launch_contract: request.organ_launch_contract.clone(),
            max_runtime_seconds: request.max_runtime_seconds,
            runtime_job_id: backend_job_id.clone(),
        },
    )?;
    let job_bindings = replace_or_append_epiphany_job_binding(
        state.job_bindings.clone(),
        heartbeat_plan.binding.clone(),
    );
    let mut runtime_links = vec![heartbeat_plan.runtime_link.clone()];
    if let Some(link) = completed_prior_link {
        runtime_links.push(link);
    }
    let state_update = EpiphanyStateUpdate {
        expected_revision: request.expected_revision,
        job_bindings: Some(job_bindings),
        runtime_links,
        ..Default::default()
    };
    let errors = epiphany_state_update_validation_errors(state, &state_update);
    if !errors.is_empty() {
        return Err(anyhow!(
            "invalid Epiphany job launch patch: {}",
            errors.join("; ")
        ));
    }
    Ok(EpiphanyCoordinatorJobLaunchPlan {
        launcher_job_id,
        backend_job_id,
        planning_state,
        heartbeat_plan,
        state_update,
    })
}

fn terminal_runtime_link_for_binding(
    state: &EpiphanyThreadState,
    binding_id: &str,
    runtime_store: &Path,
) -> Result<Option<EpiphanyRuntimeLink>> {
    let Some(link) = state
        .runtime_links
        .iter()
        .find(|link| link.binding_id == binding_id && !link.runtime_job_id.trim().is_empty())
    else {
        return Ok(None);
    };
    if link.runtime_result_id.is_some() {
        return Ok(None);
    }
    let Some(snapshot) = runtime_job_snapshot(runtime_store, &link.runtime_job_id)? else {
        return Ok(None);
    };
    let Some(result) = snapshot.result else {
        return Ok(None);
    };
    let mut terminal = link.clone();
    terminal.id = format!("{}-{}", link.id, result.result_id);
    terminal.surface = "runtimeResult".to_string();
    terminal.runtime_result_id = Some(result.result_id);
    Ok(Some(terminal))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stale_revision_refuses_launch_before_runtime_writes() {
        let state = EpiphanyThreadState {
            revision: 2,
            ..Default::default()
        };
        let request = build_epiphany_role_launch_request(
            "thread-1",
            EpiphanyRoleResultRoleId::Modeling,
            Some(1),
            Some(60),
            &state,
        )
        .expect("role launch request");
        let error = plan_coordinator_job_launch(
            &state,
            &request,
            Path::new("missing.cc"),
            "launcher".to_string(),
            "backend".to_string(),
        )
        .expect_err("stale launch must fail");
        assert!(error.to_string().contains("revision mismatch"));
    }
}
