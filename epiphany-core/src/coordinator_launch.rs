use crate::*;
use anyhow::{Result, anyhow};
use cultcache_rs::{CultCache, CultCacheEnvelope};
use epiphany_state_model::{EpiphanyRuntimeLink, EpiphanyThreadState};
use sha2::{Digest, Sha256};
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
    let mut cache =
        coordinator_state_transaction::open_coordinator_state_transaction(store, current_state)?;
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
    let runtime_identity = cache
        .get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("coordinator launch requires runtime identity"))?;
    let next_state =
        apply_coordinator_state_update_to_state(current_state, plan.state_update.clone(), None)?;
    let mut effective_launch_document = request.launch_document.clone();
    let proposal_launch = if let Some(request_id) = request.proposal_modeling_request_id.as_deref()
    {
        let (selection, proposal, identity) =
            validate_proposal_modeling_launch(cache, current_state, request, request_id)?;
        let model = cache
            .get::<crate::EpiphanyMemoryGraphEntry>(crate::MEMORY_GRAPH_KEY)?
            .ok_or_else(|| anyhow!("proposal Modeling launch requires canonical RepoModel"))?
            .snapshot()?;
        let projection = build_proposal_modeling_context_projection(&selection, &proposal, &model)?;
        match &mut effective_launch_document {
            EpiphanyWorkerLaunchDocument::Role(document) => {
                document.proposal_modeling_context = Some(projection);
            }
            EpiphanyWorkerLaunchDocument::Reorient(_) => {
                return Err(anyhow!(
                    "reorient launch cannot carry proposal Modeling context"
                ));
            }
        }
        let bytes = rmp_serde::to_vec_named(&effective_launch_document)?;
        Some((
            selection,
            proposal,
            identity,
            format!("{:x}", Sha256::digest(bytes)),
        ))
    } else {
        None
    };
    let prepared = prepare_runtime_spine_heartbeat_job(
        &cache,
        RuntimeSpineHeartbeatJobOptions {
            runtime_id: runtime_identity.runtime_id,
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
            launch_document: effective_launch_document,
            output_contract_id: request.output_contract_id.clone(),
            organ_launch_contract: request.organ_launch_contract.clone(),
            proposal_modeling_request_id: request.proposal_modeling_request_id.clone(),
            created_at: created_at.clone(),
        },
    )?;
    let mut batch = prepared.envelopes;
    if let Some((selection, proposal, identity, worker_launch_document_sha256)) = proposal_launch {
        let launch_binding = RepoFrontierProposalModelingLaunchBinding {
            schema_version: REPO_FRONTIER_PROPOSAL_MODELING_LAUNCH_BINDING_SCHEMA_VERSION.into(),
            binding_record_id: format!(
                "repo-frontier-proposal-modeling-launch-{}",
                plan.backend_job_id
            ),
            proposal_modeling_request_id: selection.request_id,
            proposal_id: proposal.proposal_id,
            proposal_payload_sha256: proposal.payload_sha256,
            job_id: plan.backend_job_id.clone(),
            binding_id: request.binding_id.clone(),
            runtime_id: identity.runtime_id,
            thread_id: selection.thread_id.clone(),
            launched_at: created_at.clone(),
            worker_launch_document_sha256,
            contract: REPO_FRONTIER_PROPOSAL_MODELING_LAUNCH_BINDING_CONTRACT.into(),
        };
        if cache
            .get::<RepoFrontierProposalModelingLaunchBinding>(&launch_binding.binding_record_id)?
            .is_some()
        {
            return Err(anyhow!(
                "proposal Modeling launch binding already exists for backend job"
            ));
        }
        batch.push(
            cache
                .prepare_entry(&launch_binding.binding_record_id, &launch_binding)?
                .0,
        );
    }
    if request.binding_id == EPIPHANY_RESEARCH_ROLE_BINDING_ID {
        let grant = substrate_gate_repo_access_grant_for_launch(
            format!("substrate-grant-{}", plan.backend_job_id),
            plan.backend_job_id.clone(),
            request,
            created_at,
        );
        batch.push(cache.prepare_entry(&grant.receipt_id, &grant)?.0);
    }
    batch.extend(injected_envelopes);
    coordinator_state_transaction::commit_coordinator_state_transaction(
        cache,
        thread_id,
        &next_state,
        batch,
    )?;
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
    let caller_projection = match &request.launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => document.proposal_modeling_context.as_ref(),
        EpiphanyWorkerLaunchDocument::Reorient(_) => None,
    };
    if caller_projection.is_some() {
        return Err(anyhow!(
            "caller-prepopulated proposal Modeling context is forbidden; coordinator commit owns projection"
        ));
    }
    if let Some(expected) = request.expected_revision
        && expected != state.revision
    {
        return Err(anyhow!(
            "epiphany state revision mismatch: expected {expected}, found {}",
            state.revision
        ));
    }
    if let Some(request_id) = request.proposal_modeling_request_id.as_deref() {
        let mut cache = runtime_spine_cache(runtime_store)?;
        cache.pull_all_backing_stores()?;
        validate_proposal_modeling_launch(&cache, state, request, request_id)?;
    } else if request.owner_role == EPIPHANY_MODELING_OWNER_ROLE {
        // Ordinary Modeling launches remain valid, but carry no proposal authority.
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

fn build_proposal_modeling_context_projection(
    request: &RepoFrontierProposalModelingRequest,
    proposal: &RepoFrontierWorkProposal,
    model: &crate::EpiphanyMemoryGraphSnapshot,
) -> Result<RepoFrontierProposalModelingContextProjection> {
    Ok(RepoFrontierProposalModelingContextProjection {
        schema_version: REPO_FRONTIER_PROPOSAL_MODELING_CONTEXT_SCHEMA_VERSION.into(),
        contract: REPO_FRONTIER_PROPOSAL_MODELING_CONTEXT_CONTRACT.into(),
        request_id: request.request_id.clone(),
        proposal_id: proposal.proposal_id.clone(),
        proposal_payload_sha256: proposal.payload_sha256.clone(),
        runtime_id: request.runtime_id.clone(),
        thread_id: request.thread_id.clone(),
        repository: request.repository.clone(),
        workspace: request.workspace.clone(),
        source_kind: proposal.source_kind,
        source_actor: proposal.source_actor.clone(),
        source_ref: proposal.source_ref.clone(),
        title: proposal.title.clone(),
        body: proposal.body.clone(),
        desired_outcome: proposal.desired_outcome.clone(),
        constraints: proposal.constraints.clone(),
        scope_hints: proposal.scope_hints.clone(),
        evidence_refs: proposal.evidence_refs.clone(),
        private_state_included: proposal.private_state_included,
        model_revision: model.model_revision,
        model_hash: crate::memory_graph_model_hash(model)?,
    })
}

fn validate_proposal_modeling_launch(
    cache: &CultCache,
    state: &EpiphanyThreadState,
    launch: &EpiphanyJobLaunchRequest,
    request_id: &str,
) -> Result<(
    RepoFrontierProposalModelingRequest,
    RepoFrontierWorkProposal,
    EpiphanyRuntimeIdentity,
)> {
    if match &launch.launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => {
            document.proposal_modeling_context.is_some()
        }
        EpiphanyWorkerLaunchDocument::Reorient(_) => false,
    } {
        return Err(anyhow!("caller cannot author proposal Modeling context"));
    }
    if launch.owner_role != EPIPHANY_MODELING_OWNER_ROLE
        || launch.binding_id != EPIPHANY_MODELING_ROLE_BINDING_ID
    {
        return Err(anyhow!(
            "proposal selection may only be carried by the Modeling role launch"
        ));
    }
    let selection = cache
        .get::<RepoFrontierProposalModelingRequest>(request_id)?
        .ok_or_else(|| {
            anyhow!("proposal Modeling selection request {request_id:?} does not exist")
        })?;
    crate::runtime_spine::validate_repo_frontier_proposal_modeling_request(&selection)?;
    let proposal = cache
        .get::<RepoFrontierWorkProposal>(&selection.proposal_id)?
        .ok_or_else(|| anyhow!("proposal Modeling selection references a missing proposal"))?;
    crate::runtime_spine::validate_repo_frontier_work_proposal(&proposal)?;
    if cache
        .get_all::<RepoFrontierProposalModelingLaunchBinding>()?
        .iter()
        .any(|binding| binding.proposal_modeling_request_id == request_id)
    {
        return Err(anyhow!(
            "proposal Modeling selection request is already bound to a launch"
        ));
    }
    let identity = cache
        .get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("proposal Modeling launch requires runtime identity"))?;
    let persisted_state = cache
        .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
        .ok_or_else(|| anyhow!("proposal Modeling launch requires authoritative thread state"))?;
    let persisted_state_value = persisted_state.state()?;
    let content = rmp_serde::to_vec_named(&(
        &proposal.title,
        &proposal.body,
        &proposal.desired_outcome,
        &proposal.constraints,
        &proposal.scope_hints,
        &proposal.evidence_refs,
    ))?;
    if selection.schema_version != REPO_FRONTIER_PROPOSAL_MODELING_REQUEST_SCHEMA_VERSION
        || selection.contract != REPO_FRONTIER_PROPOSAL_MODELING_REQUEST_CONTRACT
        || selection.request_id != request_id
        || selection.proposal_payload_sha256 != proposal.payload_sha256
        || proposal.payload_sha256 != format!("{:x}", Sha256::digest(content))
        || selection.runtime_id != identity.runtime_id
        || selection.runtime_id != proposal.runtime_id
        || selection.thread_id != persisted_state.thread_id
        || selection.thread_id != proposal.thread_id
        || persisted_state_value != *state
        || selection.repository != proposal.repository
        || selection.workspace != proposal.workspace
    {
        return Err(anyhow!(
            "proposal Modeling launch provenance binding mismatch"
        ));
    }
    Ok((selection, proposal, identity))
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

    fn proposal_launch_fixture(
        root: &Path,
        suffix: &str,
    ) -> Result<(
        std::path::PathBuf,
        EpiphanyThreadState,
        EpiphanyJobLaunchRequest,
        crate::RepoFrontierProposalModelingRequest,
    )> {
        let store = root.join(format!("proposal-launch-{suffix}.cc"));
        let runtime_id = format!("proposal-launch-runtime-{suffix}");
        let thread_id = format!("proposal-launch-thread-{suffix}");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: runtime_id.clone(),
                display_name: "Proposal launch fixture".into(),
                created_at: "2026-07-13T05:00:00Z".into(),
            },
        )?;
        let state = EpiphanyThreadState::default();
        let cache = coordinator_acceptance_cache(&store)?;
        cache.put(
            THREAD_STATE_KEY,
            &EpiphanyThreadStateEntry::from_state(&thread_id, &state)?,
        )?;
        crate::ensure_runtime_repo_model(
            &store,
            root.join(format!("proposal-launch-{suffix}.legacy.cc")),
            &crate::EpiphanyMemoryGraphSnapshot {
                schema_version: Some(crate::MEMORY_GRAPH_SCHEMA_VERSION.to_string()),
                graph_id: format!("proposal-launch-model-{suffix}"),
                domains: vec![crate::EpiphanyMemoryDomain {
                    id: "repo".to_string(),
                    profile: crate::EpiphanyMemoryProfile::RepoArchitecture,
                    title: "Repository".to_string(),
                    lifecycle: crate::EpiphanyMemoryLifecycle::Accepted,
                    ..Default::default()
                }],
                nodes: vec![crate::EpiphanyMemoryNode {
                    id: format!("proposal-launch-claim-{suffix}"),
                    domain_id: "repo".to_string(),
                    profile: crate::EpiphanyMemoryProfile::RepoArchitecture,
                    kind: crate::EpiphanyMemoryNodeKind::RuntimeContract,
                    title: "Proposal launch".to_string(),
                    claim: "Coordinator launch owns the proposal context projection.".to_string(),
                    question: "Does every selected proposal reach the exact bound worker?"
                        .to_string(),
                    action_implication: "Inject and seal the projection in the launch transaction."
                        .to_string(),
                    source_hashes: vec!["anchor:missing".to_string()],
                    lifecycle: crate::EpiphanyMemoryLifecycle::Accepted,
                    ..Default::default()
                }],
                ..Default::default()
            },
            "2026-07-13T05:00:00Z",
        )?;
        let proposal_id = format!("proposal-launch-{suffix}");
        crate::intake_user_repo_frontier_proposal(
            &store,
            crate::RepoFrontierUserProposalInput {
                proposal_id: proposal_id.clone(),
                source_actor: "fixture-user".into(),
                source_ref: format!("fixture:{suffix}"),
                repository: "EpiphanyAgent".into(),
                workspace: "E:/Projects/EpiphanyAgent".into(),
                thread_id: thread_id.clone(),
                runtime_id,
                title: "Launch selected Modeling".into(),
                body: "Bind one backend job".into(),
                desired_outcome: "One immutable launch binding".into(),
                constraints: vec!["No duplicate backend".into()],
                scope_hints: vec!["epiphany-core/src".into()],
                evidence_refs: vec![format!("fixture:{suffix}")],
                private_state_included: false,
                proposed_at: "2026-07-13T05:00:01Z".into(),
            },
        )?;
        let selection = crate::select_repo_frontier_work_proposal_for_modeling(
            &store,
            &proposal_id,
            "2026-07-13T05:00:02Z",
        )?;
        let mut request = build_epiphany_role_launch_request(
            &thread_id,
            EpiphanyRoleResultRoleId::Modeling,
            Some(state.revision),
            Some(60),
            &state,
        )
        .map_err(|error| anyhow!(error))?;
        request.proposal_modeling_request_id = Some(selection.request_id.clone());
        Ok((store, state, request, selection))
    }

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
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "coordinator-launch-test".to_string(),
                display_name: "Coordinator Launch Test".to_string(),
                created_at: "2026-07-10T23:59:59Z".to_string(),
            },
        )?;
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

    #[test]
    fn proposal_launch_refuses_corrupt_selection_and_second_backend_without_writes() -> Result<()> {
        let root = tempfile::tempdir()?;
        let (store, state, request, mut selection) =
            proposal_launch_fixture(root.path(), "corrupt")?;
        selection.contract = "corrupt".into();
        let mut cache = coordinator_acceptance_cache(&store)?;
        cache.put(&selection.request_id, &selection)?;
        let before = std::fs::read(&store)?;
        assert!(
            plan_coordinator_job_launch(
                &state,
                &request,
                &store,
                "launcher-corrupt".into(),
                "backend-corrupt".into(),
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&store)?, before);

        let (store, state, request, selection) = proposal_launch_fixture(root.path(), "duplicate")?;
        let first = plan_coordinator_job_launch(
            &state,
            &request,
            &store,
            "launcher-first".into(),
            "backend-first".into(),
        )?;
        let committed = commit_coordinator_job_launch(
            &store,
            &selection.thread_id,
            &state,
            &request,
            &first,
            "2026-07-13T05:00:03Z".into(),
        )?;
        let persisted_launch = runtime_worker_launch_request(&store, "backend-first")?
            .expect("coordinator persisted worker launch");
        let persisted_document = persisted_launch.launch_document()?;
        let projection = match persisted_document {
            EpiphanyWorkerLaunchDocument::Role(document) => document
                .proposal_modeling_context
                .expect("coordinator injected typed proposal context"),
            EpiphanyWorkerLaunchDocument::Reorient(_) => {
                panic!("proposal selection cannot produce reorient launch")
            }
        };
        assert_eq!(projection.request_id, selection.request_id);
        assert_eq!(projection.proposal_id, selection.proposal_id);
        assert_eq!(
            projection.proposal_payload_sha256,
            selection.proposal_payload_sha256
        );
        let mut cache = coordinator_acceptance_cache(&store)?;
        let binding = cache
            .get::<RepoFrontierProposalModelingLaunchBinding>(
                "repo-frontier-proposal-modeling-launch-backend-first",
            )?
            .expect("coordinator persisted proposal launch binding");
        assert_eq!(
            binding.worker_launch_document_sha256,
            format!(
                "{:x}",
                Sha256::digest(&persisted_launch.launch_document_msgpack)
            )
        );
        let mut second_request = build_epiphany_role_launch_request(
            &selection.thread_id,
            EpiphanyRoleResultRoleId::Modeling,
            Some(committed.epiphany_state.revision),
            Some(60),
            &committed.epiphany_state,
        )
        .map_err(|error| anyhow!(error))?;
        second_request.proposal_modeling_request_id = request.proposal_modeling_request_id.clone();
        let before = std::fs::read(&store)?;
        assert!(
            plan_coordinator_job_launch(
                &committed.epiphany_state,
                &second_request,
                &store,
                "launcher-second".into(),
                "backend-second".into(),
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&store)?, before);
        Ok(())
    }
}
