use crate::*;
use anyhow::{Result, anyhow};
use cultcache_rs::{CultCache, CultCacheEnvelope};
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

pub fn commit_coordinator_job_launch(
    store: &Path,
    thread_id: &str,
    current_state: &EpiphanyThreadState,
    request: &EpiphanyJobLaunchRequest,
    plan: &EpiphanyCoordinatorJobLaunchPlan,
    created_at: String,
) -> Result<EpiphanyJobLaunchResult> {
    if let Some(persisted) = read_accepted_coordinator_state(store)?
        && persisted != *current_state
    {
        return Err(anyhow!(
            "authoritative coordinator state changed before launch commit"
        ));
    }
    let mut cache = coordinator_acceptance_cache(store)?;
    commit_coordinator_job_launch_in_cache(
        &mut cache,
        thread_id,
        current_state,
        request,
        plan,
        created_at,
        Vec::new(),
    )
}

fn commit_coordinator_job_launch_in_cache(
    cache: &mut CultCache,
    thread_id: &str,
    current_state: &EpiphanyThreadState,
    request: &EpiphanyJobLaunchRequest,
    plan: &EpiphanyCoordinatorJobLaunchPlan,
    created_at: String,
    injected_envelopes: Vec<CultCacheEnvelope>,
) -> Result<EpiphanyJobLaunchResult> {
    let next_state =
        apply_coordinator_state_update_to_state(current_state, plan.state_update.clone(), None)?;
    let prepared = prepare_runtime_spine_heartbeat_job(
        &cache,
        RuntimeSpineHeartbeatJobOptions {
            runtime_id: "epiphany-local".to_string(),
            display_name: "Epiphany Local".to_string(),
            session_id: "epiphany-main".to_string(),
            objective: plan
                .planning_state
                .objective
                .clone()
                .filter(|objective| !objective.trim().is_empty())
                .unwrap_or_else(|| "Epiphany heartbeat activation".to_string()),
            coordinator_note: "Native coordinator launch transaction opened this session."
                .to_string(),
            job_id: plan.backend_job_id.clone(),
            role: request.owner_role.clone(),
            binding_id: request.binding_id.clone(),
            authority_scope: request.authority_scope.clone(),
            instruction: request.instruction.clone(),
            launch_document: request.launch_document.clone(),
            output_contract_id: request.output_contract_id.clone(),
            organ_launch_contract: request.organ_launch_contract.clone(),
            created_at: created_at.clone(),
        },
    )?;
    let mut batch = prepared.envelopes;
    if request.binding_id == EPIPHANY_RESEARCH_ROLE_BINDING_ID {
        let grant = substrate_gate_repo_access_grant_for_launch(
            format!("substrate-grant-{}", plan.backend_job_id),
            plan.backend_job_id.clone(),
            request,
            created_at,
        );
        batch.push(cache.prepare_entry(&grant.receipt_id, &grant)?.0);
    }
    let state_entry = EpiphanyThreadStateEntry::from_state(thread_id, &next_state)?;
    batch.push(cache.prepare_entry(THREAD_STATE_KEY, &state_entry)?.0);
    batch.extend(injected_envelopes);
    cache.put_prepared_batch(batch)?;
    Ok(EpiphanyJobLaunchResult {
        epiphany_state: next_state,
        binding_id: request.binding_id.clone(),
        launcher_job_id: plan.launcher_job_id.clone(),
        backend_job_id: plan.backend_job_id.clone(),
    })
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

    fn research_launch(state: &EpiphanyThreadState) -> EpiphanyJobLaunchRequest {
        build_epiphany_role_launch_request(
            "thread-1",
            EpiphanyRoleResultRoleId::Research,
            Some(state.revision),
            Some(60),
            state,
        )
        .expect("research launch request")
    }

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

    #[test]
    fn launch_documents_state_and_grant_share_one_snapshot() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("launch.cc");
        let state = EpiphanyThreadState::default();
        let request = research_launch(&state);
        let plan = plan_coordinator_job_launch(
            &state,
            &request,
            &store,
            "launcher-1".to_string(),
            "backend-1".to_string(),
        )?;
        let result = commit_coordinator_job_launch(
            &store,
            "thread-1",
            &state,
            &request,
            &plan,
            "2026-07-11T00:00:00Z".to_string(),
        )?;
        assert_eq!(result.epiphany_state.revision, 1);
        let cache = coordinator_acceptance_cache(&store)?;
        assert!(
            cache
                .get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
                .is_some()
        );
        assert!(
            cache
                .get::<EpiphanyRuntimeSession>("epiphany-main")?
                .is_some()
        );
        assert!(cache.get::<EpiphanyRuntimeJob>("backend-1")?.is_some());
        assert!(
            cache
                .get::<EpiphanyRuntimeWorkerLaunchRequest>("backend-1")?
                .is_some()
        );
        assert!(
            cache
                .get::<EpiphanyRuntimeEvent>("event-job-opened-backend-1")?
                .is_some()
        );
        assert!(
            cache
                .get::<SubstrateGateRepoAccessGrantReceipt>("substrate-grant-backend-1")?
                .is_some()
        );
        assert_eq!(
            cache
                .get_required::<EpiphanyThreadStateEntry>(THREAD_STATE_KEY)?
                .state()?,
            result.epiphany_state
        );
        Ok(())
    }

    #[test]
    fn refused_batch_preserves_prior_snapshot_without_partial_launch() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("launch-refused.cc");
        let state = EpiphanyThreadState::default();
        let mut cache = coordinator_acceptance_cache(&store)?;
        let initial = EpiphanyThreadStateEntry::from_state("thread-1", &state)?;
        cache.put(THREAD_STATE_KEY, &initial)?;
        let request = research_launch(&state);
        let plan = plan_coordinator_job_launch(
            &state,
            &request,
            &store,
            "launcher-refused".to_string(),
            "backend-refused".to_string(),
        )?;
        let poison = CultCacheEnvelope {
            key: "injected-refusal".to_string(),
            r#type: "unregistered.injected_failure".to_string(),
            payload: Vec::new(),
            stored_at: "2026-07-11T00:00:00Z".to_string(),
            schema_id: None,
        };
        assert!(
            commit_coordinator_job_launch_in_cache(
                &mut cache,
                "thread-1",
                &state,
                &request,
                &plan,
                "2026-07-11T00:00:00Z".to_string(),
                vec![poison],
            )
            .is_err()
        );
        let cache = coordinator_acceptance_cache(&store)?;
        assert_eq!(
            cache
                .get_required::<EpiphanyThreadStateEntry>(THREAD_STATE_KEY)?
                .state()?,
            state
        );
        assert!(
            cache
                .get::<EpiphanyRuntimeJob>("backend-refused")?
                .is_none()
        );
        assert!(
            cache
                .get::<EpiphanyRuntimeWorkerLaunchRequest>("backend-refused")?
                .is_none()
        );
        assert!(
            cache
                .get::<SubstrateGateRepoAccessGrantReceipt>("substrate-grant-backend-refused")?
                .is_none()
        );
        Ok(())
    }
}
