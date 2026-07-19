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
    let body_basis = if request.owner_role == EPIPHANY_MODELING_OWNER_ROLE {
        Some(observe_runtime_repository_body_basis(store)?)
    } else {
        None
    };
    let mut cache =
        coordinator_state_transaction::open_coordinator_state_transaction(store, current_state)?;
    commit_coordinator_job_launch_in_cache(
        &mut cache,
        thread_id,
        current_state,
        request,
        plan,
        created_at,
        body_basis,
        Vec::new(),
    )
}

fn commit_coordinator_job_launch_in_cache(
    cache: &mut coordinator_state_transaction::CoordinatorStateTransaction,
    thread_id: &str,
    current_state: &EpiphanyThreadState,
    request: &EpiphanyJobLaunchRequest,
    plan: &EpiphanyCoordinatorJobLaunchPlan,
    created_at: String,
    repository_body_observation_basis: Option<RepositoryBodyObservationBasis>,
    injected_envelopes: Vec<CultCacheEnvelope>,
) -> Result<EpiphanyJobLaunchResult> {
    if thread_id != request.launch_document.thread_id() {
        return Err(anyhow!(
            "coordinator commit thread id must match typed launch document"
        ));
    }
    let runtime_identity = cache
        .get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("coordinator launch requires runtime identity"))?;
    let next_state =
        apply_coordinator_state_update_to_state(current_state, plan.state_update.clone(), None)?;
    let mut effective_launch_document = request.launch_document.clone();
    match &mut effective_launch_document {
        EpiphanyWorkerLaunchDocument::Role(document)
            if request.owner_role == EPIPHANY_MODELING_OWNER_ROLE =>
        {
            if document.repository_body_observation_basis.is_some() {
                return Err(anyhow!(
                    "caller-prepopulated repository Body basis is forbidden"
                ));
            }
            document.repository_body_observation_basis =
                Some(repository_body_observation_basis.ok_or_else(|| {
                    anyhow!("Modeling launch commit requires a repository Body basis")
                })?);
        }
        EpiphanyWorkerLaunchDocument::Role(document) => {
            if document.repository_body_observation_basis.is_some()
                || repository_body_observation_basis.is_some()
            {
                return Err(anyhow!(
                    "non-Modeling launch cannot carry a repository Body basis"
                ));
            }
        }
        EpiphanyWorkerLaunchDocument::Reorient(_) => {
            if repository_body_observation_basis.is_some() {
                return Err(anyhow!(
                    "reorient launch cannot carry a repository Body basis"
                ));
            }
        }
    }
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
    let claim_repair_launch = if let Some(request_id) = request.claim_repair_request_id.as_deref() {
        let (repair, challenge, identity) =
            validate_claim_repair_launch(cache, current_state, request, request_id)?;
        let projection = RepoModelClaimRepairContextProjection::from_request(&repair);
        match &mut effective_launch_document {
            EpiphanyWorkerLaunchDocument::Role(document) => {
                if document.proposal_modeling_context.is_some() {
                    return Err(anyhow!(
                        "claim repair and proposal contexts are mutually exclusive"
                    ));
                }
                document.claim_repair_context = Some(projection);
            }
            EpiphanyWorkerLaunchDocument::Reorient(_) => {
                return Err(anyhow!("reorient launch cannot carry claim repair context"));
            }
        }
        let bytes = rmp_serde::to_vec_named(&effective_launch_document)?;
        Some((
            repair,
            challenge,
            identity,
            format!("{:x}", Sha256::digest(bytes)),
        ))
    } else {
        None
    };
    let frontier_planning_launch =
        if let Some(request_id) = request.frontier_planning_request_id.as_deref() {
            let (planning, identity) =
                validate_frontier_planning_launch(cache, current_state, request, request_id)?;
            let projection = RepoFrontierPlanningContextProjection::from_request(&planning);
            match &mut effective_launch_document {
                EpiphanyWorkerLaunchDocument::Role(document) => {
                    if document.proposal_modeling_context.is_some()
                        || document.claim_repair_context.is_some()
                    {
                        return Err(anyhow!(
                            "frontier planning context is exclusive of Modeling authority contexts"
                        ));
                    }
                    document.frontier_planning_context = Some(projection);
                }
                EpiphanyWorkerLaunchDocument::Reorient(_) => {
                    return Err(anyhow!(
                        "reorient launch cannot carry frontier planning context"
                    ));
                }
            }
            let bytes = rmp_serde::to_vec_named(&effective_launch_document)?;
            Some((planning, identity, format!("{:x}", Sha256::digest(bytes))))
        } else {
            None
        };
    let imagination_consideration_launch = if let Some(request_id) =
        request.imagination_consideration_request_id.as_deref()
    {
        let (consideration, identity) =
            validate_imagination_consideration_launch(cache, current_state, request, request_id)?;
        let model = cache
            .get::<EpiphanyMemoryGraphEntry>(MEMORY_GRAPH_KEY)?
            .ok_or_else(|| anyhow!("consideration launch requires current Modeling map"))?
            .snapshot()?;
        if model.model_revision != consideration.model_revision
            || memory_graph_model_hash(&model)? != consideration.model_hash
        {
            return Err(anyhow!("consideration launch map/request mismatch"));
        }
        let projection = ImaginationConsiderationContextProjection::new(&consideration, &model);
        match &mut effective_launch_document {
            EpiphanyWorkerLaunchDocument::Role(document) => {
                if document.proposal_modeling_context.is_some()
                    || document.claim_repair_context.is_some()
                    || document.frontier_planning_context.is_some()
                    || document.frontier_plan_mind_context.is_some()
                {
                    return Err(anyhow!("consideration context is exclusive"));
                }
                document.objective = None;
                document.dynamic_prompt_context =
                    Some(render_imagination_consideration_prompt(&consideration)?);
                document.imagination_consideration_context = Some(projection);
            }
            EpiphanyWorkerLaunchDocument::Reorient(_) => {
                return Err(anyhow!("reorient cannot carry consideration context"));
            }
        }
        let hash = format!(
            "{:x}",
            Sha256::digest(rmp_serde::to_vec_named(&effective_launch_document)?)
        );
        Some((consideration, identity, hash))
    } else {
        None
    };
    let admitted_model_direction_consideration_launch = if let Some(request_id) = request
        .admitted_model_direction_consideration_request_id
        .as_deref()
    {
        let consideration = validate_admitted_model_direction_consideration_launch(
            cache,
            current_state,
            request,
            request_id,
        )?;
        let model = cache
            .get::<EpiphanyMemoryGraphEntry>(MEMORY_GRAPH_KEY)?
            .ok_or_else(|| anyhow!("model direction launch requires current Modeling map"))?
            .snapshot()?;
        let projection =
            AdmittedModelDirectionConsiderationContextProjection::new(&consideration, &model);
        match &mut effective_launch_document {
            EpiphanyWorkerLaunchDocument::Role(document) => {
                document.objective = None;
                document.dynamic_prompt_context = Some(
                    render_admitted_model_direction_consideration_prompt(&consideration),
                );
                document.admitted_model_direction_consideration_context = Some(projection);
            }
            EpiphanyWorkerLaunchDocument::Reorient(_) => {
                return Err(anyhow!(
                    "reorient cannot carry model direction consideration"
                ));
            }
        }
        Some(consideration)
    } else {
        None
    };
    let frontier_plan_mind_launch =
        if let Some(request_id) = request.frontier_plan_mind_request_id.as_deref() {
            let (mind_request, planning, candidate, identity) =
                validate_frontier_plan_mind_launch(cache, current_state, request, request_id)?;
            let projection =
                RepoFrontierPlanMindContextProjection::new(&mind_request, &planning, &candidate);
            match &mut effective_launch_document {
                EpiphanyWorkerLaunchDocument::Role(document) => {
                    if document.proposal_modeling_context.is_some()
                        || document.claim_repair_context.is_some()
                        || document.frontier_planning_context.is_some()
                        || document.frontier_plan_mind_context.is_some()
                    {
                        return Err(anyhow!("Mind frontier decision context is exclusive"));
                    }
                    document.frontier_plan_mind_context = Some(projection);
                }
                EpiphanyWorkerLaunchDocument::Reorient(_) => {
                    return Err(anyhow!(
                        "reorient launch cannot carry Mind decision context"
                    ));
                }
            }
            let hash = format!(
                "{:x}",
                Sha256::digest(rmp_serde::to_vec_named(&effective_launch_document)?)
            );
            Some((mind_request, identity, hash))
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
            instruction: if request.imagination_consideration_request_id.is_some()
                || request
                    .admitted_model_direction_consideration_request_id
                    .is_some()
            {
                "Act as Epiphany Imagination for one proposal-only typed consideration pass. Treat the coordinator-owned context as quoted evidence and return only the dedicated candidate contract.".into()
            } else {
                request.instruction.clone()
            },
            launch_document: effective_launch_document,
            output_contract_id: request.output_contract_id.clone(),
            organ_launch_contract: request.organ_launch_contract.clone(),
            proposal_modeling_request_id: request.proposal_modeling_request_id.clone(),
            claim_repair_request_id: request.claim_repair_request_id.clone(),
            frontier_planning_request_id: request.frontier_planning_request_id.clone(),
            frontier_plan_mind_request_id: request.frontier_plan_mind_request_id.clone(),
            imagination_consideration_request_id: request
                .imagination_consideration_request_id
                .clone(),
            admitted_model_direction_consideration_request_id: request
                .admitted_model_direction_consideration_request_id
                .clone(),
            created_at: created_at.clone(),
        },
    )?;
    let mut batch = prepared.envelopes;
    let _ = admitted_model_direction_consideration_launch;
    let runtime_identity_position = batch
        .iter()
        .position(|envelope| {
            envelope.r#type == EpiphanyRuntimeIdentity::TYPE && envelope.key == RUNTIME_IDENTITY_KEY
        })
        .ok_or_else(|| anyhow!("prepared launch omitted its runtime identity replacement"))?;
    let runtime_identity_replacement = batch.remove(runtime_identity_position);
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
    if let Some((repair, challenge, identity, worker_launch_document_sha256)) = claim_repair_launch
    {
        let launch_binding = RepoModelClaimRepairLaunchBinding {
            schema_version: REPO_MODEL_CLAIM_REPAIR_LAUNCH_BINDING_SCHEMA_VERSION.into(),
            binding_record_id: format!("repo-model-claim-repair-launch-{}", repair.request_id),
            repair_request_id: repair.request_id,
            challenge_id: challenge.challenge_id,
            challenge_sha256: repair.challenge_sha256,
            job_id: plan.backend_job_id.clone(),
            binding_id: request.binding_id.clone(),
            runtime_id: identity.runtime_id,
            thread_id: repair.thread_id,
            launched_at: created_at.clone(),
            worker_launch_document_sha256,
            contract: REPO_MODEL_CLAIM_REPAIR_LAUNCH_BINDING_CONTRACT.into(),
        };
        if cache
            .get::<RepoModelClaimRepairLaunchBinding>(&launch_binding.binding_record_id)?
            .is_some()
        {
            return Err(anyhow!(
                "claim repair launch binding already exists for backend job"
            ));
        }
        batch.push(
            cache
                .prepare_entry(&launch_binding.binding_record_id, &launch_binding)?
                .0,
        );
    }
    if let Some((planning, identity, worker_launch_document_sha256)) = frontier_planning_launch {
        let launch_binding = RepoFrontierPlanningLaunchBinding {
            schema_version: REPO_FRONTIER_PLANNING_LAUNCH_BINDING_SCHEMA_VERSION.into(),
            binding_record_id: format!("repo-frontier-planning-launch-{}", planning.request_id),
            planning_request_id: planning.request_id,
            job_id: plan.backend_job_id.clone(),
            binding_id: request.binding_id.clone(),
            runtime_id: identity.runtime_id,
            thread_id: planning.thread_id,
            launched_at: created_at.clone(),
            worker_launch_document_sha256,
            contract: REPO_FRONTIER_PLANNING_LAUNCH_BINDING_CONTRACT.into(),
        };
        if cache
            .get::<RepoFrontierPlanningLaunchBinding>(&launch_binding.binding_record_id)?
            .is_some()
        {
            return Err(anyhow!(
                "frontier planning request is already bound to a launch"
            ));
        }
        batch.push(
            cache
                .prepare_entry(&launch_binding.binding_record_id, &launch_binding)?
                .0,
        );
    }
    if let Some((consideration, identity, worker_launch_document_sha256)) =
        imagination_consideration_launch
    {
        let binding = ImaginationConsiderationLaunchBinding {
            schema_version: IMAGINATION_CONSIDERATION_LAUNCH_BINDING_SCHEMA_VERSION.into(),
            binding_record_id: format!(
                "imagination-consideration-launch-{}",
                consideration.request_id
            ),
            request_id: consideration.request_id,
            job_id: plan.backend_job_id.clone(),
            binding_id: request.binding_id.clone(),
            runtime_id: identity.runtime_id,
            thread_id: consideration.thread_id,
            launched_at: created_at.clone(),
            worker_launch_document_sha256,
        };
        if cache
            .get::<ImaginationConsiderationLaunchBinding>(&binding.binding_record_id)?
            .is_some()
        {
            return Err(anyhow!(
                "consideration request is already bound to a launch"
            ));
        }
        batch.push(cache.prepare_entry(&binding.binding_record_id, &binding)?.0);
    }
    if let Some((mind_request, identity, worker_launch_document_sha256)) = frontier_plan_mind_launch
    {
        let binding = RepoFrontierPlanMindLaunchBinding {
            schema_version: REPO_FRONTIER_PLAN_MIND_LAUNCH_BINDING_SCHEMA_VERSION.into(),
            binding_record_id: format!(
                "repo-frontier-plan-mind-launch-{}",
                mind_request.request_id
            ),
            mind_request_id: mind_request.request_id,
            job_id: plan.backend_job_id.clone(),
            binding_id: request.binding_id.clone(),
            runtime_id: identity.runtime_id,
            thread_id: mind_request.thread_id,
            launched_at: created_at.clone(),
            worker_launch_document_sha256,
            contract: REPO_FRONTIER_PLAN_MIND_LAUNCH_BINDING_CONTRACT.into(),
        };
        if cache
            .get::<RepoFrontierPlanMindLaunchBinding>(&binding.binding_record_id)?
            .is_some()
        {
            return Err(anyhow!("Mind request is already bound to a launch"));
        }
        batch.push(cache.prepare_entry(&binding.binding_record_id, &binding)?.0);
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
        vec![runtime_identity_replacement],
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
    let (
        caller_proposal_projection,
        caller_repair_projection,
        caller_planning_projection,
        caller_mind_projection,
        caller_body_basis,
    ) = match &request.launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => (
            document.proposal_modeling_context.as_ref(),
            document.claim_repair_context.as_ref(),
            document.frontier_planning_context.as_ref(),
            document.frontier_plan_mind_context.as_ref(),
            document.repository_body_observation_basis.as_ref(),
        ),
        EpiphanyWorkerLaunchDocument::Reorient(_) => (None, None, None, None, None),
    };
    if caller_body_basis.is_some() {
        return Err(anyhow!(
            "caller-prepopulated repository Body basis is forbidden; coordinator commit owns observation"
        ));
    }
    if caller_proposal_projection.is_some() {
        return Err(anyhow!(
            "caller-prepopulated proposal Modeling context is forbidden; coordinator commit owns projection"
        ));
    }
    if caller_repair_projection.is_some() {
        return Err(anyhow!(
            "caller-prepopulated claim repair context is forbidden; coordinator commit owns projection"
        ));
    }
    if caller_planning_projection.is_some() {
        return Err(anyhow!(
            "caller-prepopulated frontier planning context is forbidden; coordinator commit owns projection"
        ));
    }
    if caller_mind_projection.is_some() {
        return Err(anyhow!(
            "caller-prepopulated Mind decision context is forbidden; coordinator commit owns projection"
        ));
    }
    if [
        request.proposal_modeling_request_id.is_some(),
        request.claim_repair_request_id.is_some(),
        request.frontier_planning_request_id.is_some(),
        request.frontier_plan_mind_request_id.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count()
        > 1
    {
        return Err(anyhow!(
            "specialized authority launches are mutually exclusive"
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
    } else if let Some(request_id) = request.claim_repair_request_id.as_deref() {
        let mut cache = runtime_spine_cache(runtime_store)?;
        cache.pull_all_backing_stores()?;
        validate_claim_repair_launch(&cache, state, request, request_id)?;
    } else if let Some(request_id) = request.frontier_planning_request_id.as_deref() {
        let mut cache = runtime_spine_cache(runtime_store)?;
        cache.pull_all_backing_stores()?;
        validate_frontier_planning_launch(&cache, state, request, request_id)?;
    } else if let Some(request_id) = request.frontier_plan_mind_request_id.as_deref() {
        let mut cache = runtime_spine_cache(runtime_store)?;
        cache.pull_all_backing_stores()?;
        validate_frontier_plan_mind_launch(&cache, state, request, request_id)?;
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

fn validate_frontier_plan_mind_launch(
    cache: &CultCache,
    state: &EpiphanyThreadState,
    launch: &EpiphanyJobLaunchRequest,
    request_id: &str,
) -> Result<(
    RepoFrontierPlanMindRequest,
    RepoFrontierPlanningRequest,
    RepoFrontierPlanCandidate,
    EpiphanyRuntimeIdentity,
)> {
    if launch.owner_role != EPIPHANY_MIND_OWNER_ROLE
        || launch.binding_id != EPIPHANY_MIND_ROLE_BINDING_ID
    {
        return Err(anyhow!(
            "frontier plan decision may only be carried by the Mind role launch"
        ));
    }
    let request = cache
        .get::<RepoFrontierPlanMindRequest>(request_id)?
        .ok_or_else(|| anyhow!("Mind request does not exist"))?;
    let (planning, candidate) =
        crate::runtime_spine::validate_repo_frontier_plan_mind_request(cache, &request)?;
    let identity = cache
        .get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("Mind launch requires runtime identity"))?;
    let persisted = cache
        .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
        .ok_or_else(|| anyhow!("Mind launch requires thread state"))?;
    if persisted.state()? != *state
        || request.runtime_id != identity.runtime_id
        || request.thread_id != persisted.thread_id
        || launch.launch_document.thread_id() != request.thread_id
    {
        return Err(anyhow!("Mind launch provenance mismatch"));
    }
    if cache
        .get_all::<RepoFrontierPlanMindLaunchBinding>()?
        .iter()
        .any(|b| b.mind_request_id == request_id)
    {
        return Err(anyhow!("Mind request already bound"));
    }
    Ok((request, planning, candidate, identity))
}

fn validate_frontier_planning_launch(
    cache: &CultCache,
    state: &EpiphanyThreadState,
    launch: &EpiphanyJobLaunchRequest,
    request_id: &str,
) -> Result<(RepoFrontierPlanningRequest, EpiphanyRuntimeIdentity)> {
    if match &launch.launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => {
            document.frontier_planning_context.is_some()
        }
        EpiphanyWorkerLaunchDocument::Reorient(_) => false,
    } {
        return Err(anyhow!("caller cannot author frontier planning context"));
    }
    if launch.owner_role != EPIPHANY_IMAGINATION_OWNER_ROLE
        || launch.binding_id != EPIPHANY_IMAGINATION_ROLE_BINDING_ID
    {
        return Err(anyhow!(
            "frontier planning may only be carried by the Imagination role launch"
        ));
    }
    let planning = cache
        .get::<RepoFrontierPlanningRequest>(request_id)?
        .ok_or_else(|| anyhow!("frontier planning request {request_id:?} does not exist"))?;
    crate::runtime_spine::validate_current_repo_frontier_planning_request(cache, &planning)?;
    let identity = cache
        .get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("frontier planning launch requires runtime identity"))?;
    let persisted_state = cache
        .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
        .ok_or_else(|| anyhow!("frontier planning launch requires authoritative thread state"))?;
    let persisted_state_value = persisted_state.state()?;
    if planning.request_id != request_id
        || planning.runtime_id != identity.runtime_id
        || planning.thread_id != persisted_state.thread_id
        || persisted_state_value != *state
        || launch.launch_document.thread_id() != planning.thread_id
    {
        return Err(anyhow!(
            "frontier planning launch provenance binding mismatch"
        ));
    }
    if cache
        .get_all::<RepoFrontierPlanningLaunchBinding>()?
        .iter()
        .any(|binding| binding.planning_request_id == request_id)
    {
        return Err(anyhow!(
            "frontier planning request is already bound to a launch"
        ));
    }
    Ok((planning, identity))
}

fn validate_imagination_consideration_launch(
    cache: &CultCache,
    state: &EpiphanyThreadState,
    launch: &EpiphanyJobLaunchRequest,
    request_id: &str,
) -> Result<(ImaginationConsiderationRequest, EpiphanyRuntimeIdentity)> {
    if match &launch.launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => {
            document.imagination_consideration_context.is_some()
        }
        EpiphanyWorkerLaunchDocument::Reorient(_) => false,
    } {
        return Err(anyhow!("caller cannot author consideration context"));
    }
    if launch.owner_role != EPIPHANY_IMAGINATION_OWNER_ROLE
        || launch.binding_id != EPIPHANY_IMAGINATION_ROLE_BINDING_ID
        || launch.frontier_planning_request_id.is_some()
    {
        return Err(anyhow!(
            "consideration requires an exclusive Imagination launch"
        ));
    }
    let request = cache
        .get::<ImaginationConsiderationRequest>(request_id)?
        .ok_or_else(|| anyhow!("consideration request does not exist"))?;
    validate_current_imagination_consideration_request(cache, &request)?;
    let identity = cache
        .get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("consideration requires runtime identity"))?;
    let persisted = cache
        .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
        .ok_or_else(|| anyhow!("consideration requires thread state"))?;
    if request.runtime_id != identity.runtime_id
        || request.thread_id != persisted.thread_id
        || persisted.state()? != *state
        || launch.launch_document.thread_id() != request.thread_id
    {
        return Err(anyhow!("consideration launch provenance mismatch"));
    }
    if cache
        .get_all::<ImaginationConsiderationLaunchBinding>()?
        .iter()
        .any(|binding| binding.request_id == request_id)
    {
        return Err(anyhow!("consideration request already bound"));
    }
    Ok((request, identity))
}

fn validate_admitted_model_direction_consideration_launch(
    cache: &CultCache,
    state: &EpiphanyThreadState,
    launch: &EpiphanyJobLaunchRequest,
    request_id: &str,
) -> Result<AdmittedModelDirectionConsiderationRequest> {
    if match &launch.launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => document
            .admitted_model_direction_consideration_context
            .is_some(),
        EpiphanyWorkerLaunchDocument::Reorient(_) => false,
    } {
        return Err(anyhow!("caller cannot author model direction context"));
    }
    if launch.owner_role != EPIPHANY_IMAGINATION_OWNER_ROLE
        || launch.binding_id != EPIPHANY_IMAGINATION_ROLE_BINDING_ID
        || launch.imagination_consideration_request_id.is_some()
        || launch.frontier_planning_request_id.is_some()
        || launch.frontier_plan_mind_request_id.is_some()
        || launch.claim_repair_request_id.is_some()
        || launch.proposal_modeling_request_id.is_some()
    {
        return Err(anyhow!(
            "model direction consideration requires an exclusive Imagination launch"
        ));
    }
    let request = cache
        .get::<AdmittedModelDirectionConsiderationRequest>(request_id)?
        .ok_or_else(|| anyhow!("model direction consideration request does not exist"))?;
    validate_current_admitted_model_direction_consideration_request(cache, &request)?;
    let identity = cache
        .get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("model direction consideration requires runtime identity"))?;
    let persisted = cache
        .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
        .ok_or_else(|| anyhow!("model direction consideration requires thread state"))?;
    if request.runtime_id != identity.runtime_id
        || request.thread_id != persisted.thread_id
        || persisted.state()? != *state
        || launch.launch_document.thread_id() != request.thread_id
    {
        return Err(anyhow!("model direction consideration provenance mismatch"));
    }
    if cache
        .get_all::<EpiphanyRuntimeWorkerLaunchRequest>()?
        .iter()
        .any(|worker| {
            worker
                .admitted_model_direction_consideration_request_id
                .as_deref()
                == Some(request_id)
        })
    {
        return Err(anyhow!("model direction consideration already bound"));
    }
    Ok(request)
}

fn validate_claim_repair_launch(
    cache: &CultCache,
    state: &EpiphanyThreadState,
    launch: &EpiphanyJobLaunchRequest,
    request_id: &str,
) -> Result<(
    RepoModelClaimRepairRequest,
    RepoModelClaimChallenge,
    EpiphanyRuntimeIdentity,
)> {
    if launch.owner_role != EPIPHANY_MODELING_OWNER_ROLE
        || launch.binding_id != EPIPHANY_MODELING_ROLE_BINDING_ID
    {
        return Err(anyhow!(
            "claim repair may only be carried by the Modeling role launch"
        ));
    }
    let repair = cache
        .get::<RepoModelClaimRepairRequest>(request_id)?
        .ok_or_else(|| anyhow!("claim repair request {request_id:?} does not exist"))?;
    let challenge = cache
        .get::<RepoModelClaimChallenge>(&repair.challenge_id)?
        .ok_or_else(|| anyhow!("claim repair launch references a missing challenge"))?;
    let identity = cache
        .get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("claim repair launch requires runtime identity"))?;
    let persisted_state = cache
        .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
        .ok_or_else(|| anyhow!("claim repair launch requires authoritative thread state"))?;
    let persisted_state_value = persisted_state.state()?;
    crate::runtime_spine::validate_current_repo_model_claim_repair_request(cache, &repair)?;
    if repair.request_id != request_id
        || repair.runtime_id != identity.runtime_id
        || repair.thread_id != persisted_state.thread_id
        || persisted_state_value != *state
        || launch.launch_document.thread_id() != repair.thread_id
    {
        return Err(anyhow!("claim repair launch provenance binding mismatch"));
    }
    if cache
        .get_all::<RepoModelClaimRepairLaunchBinding>()?
        .iter()
        .any(|binding| binding.repair_request_id == request_id)
    {
        return Err(anyhow!("claim repair request is already bound to a launch"));
    }
    Ok((repair, challenge, identity))
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
    if proposal.source_kind == crate::RepoFrontierProposalSourceKind::Imagination {
        crate::runtime_spine::validate_autonomous_proposal_binding(cache, &proposal)?;
    }
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
pub(crate) mod tests {
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
        crate::runtime_spine::tests::bind_test_runtime_swarm(
            &store,
            &format!("proposal-launch-swarm-{suffix}"),
        )?;
        crate::runtime_spine::tests::bind_test_repository_body(
            &store,
            &format!("proposal-launch-workspace-{suffix}"),
        )?;
        let state = EpiphanyThreadState::default();
        let mut cache = coordinator_acceptance_cache(&store)?;
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

    pub(crate) fn claim_repair_launch_fixture(
        root: &Path,
        suffix: &str,
    ) -> Result<(
        std::path::PathBuf,
        EpiphanyThreadState,
        EpiphanyJobLaunchRequest,
        RepoModelClaimRepairRequest,
    )> {
        let (store, challenge) =
            crate::runtime_spine::tests::claim_challenge_fixture(root, suffix, "Hands")?;
        crate::commit_repo_model_claim_challenge(&store, &challenge)?;
        let repair = crate::commit_repo_model_claim_repair_request(
            &store,
            &challenge.challenge_id,
            "2026-07-14T09:00:02Z",
        )?;
        let mut cache = coordinator_acceptance_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let mut state = cache
            .get_required::<EpiphanyThreadStateEntry>(THREAD_STATE_KEY)?
            .state()?;
        for link in &mut state.runtime_links {
            if link.binding_id == EPIPHANY_MODELING_ROLE_BINDING_ID
                && link.runtime_result_id.is_none()
            {
                link.runtime_result_id = Some(format!("prior-modeling-result-{suffix}"));
            }
        }
        cache.put(
            THREAD_STATE_KEY,
            &EpiphanyThreadStateEntry::from_state(&repair.thread_id, &state)?,
        )?;
        let mut launch = build_epiphany_role_launch_request(
            &repair.thread_id,
            EpiphanyRoleResultRoleId::Modeling,
            Some(state.revision),
            Some(60),
            &state,
        )
        .map_err(|error| anyhow!(error))?;
        launch.claim_repair_request_id = Some(repair.request_id.clone());
        Ok((store, state, launch, repair))
    }

    fn frontier_planning_launch_fixture(
        root: &Path,
        suffix: &str,
    ) -> Result<(
        std::path::PathBuf,
        EpiphanyThreadState,
        EpiphanyJobLaunchRequest,
        RepoFrontierPlanningRequest,
    )> {
        let (store, _) =
            crate::runtime_spine::tests::claim_challenge_fixture(root, suffix, "Imagination")?;
        let planning = crate::select_and_commit_repo_frontier_planning_request(
            &store,
            "2026-07-15T09:00:02Z",
        )?;
        let mut cache = coordinator_acceptance_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let mut state = cache
            .get_required::<EpiphanyThreadStateEntry>(THREAD_STATE_KEY)?
            .state()?;
        for link in &mut state.runtime_links {
            if link.binding_id == EPIPHANY_IMAGINATION_ROLE_BINDING_ID
                && link.runtime_result_id.is_none()
            {
                link.runtime_result_id = Some(format!("prior-imagination-result-{suffix}"));
            }
        }
        cache.put(
            THREAD_STATE_KEY,
            &EpiphanyThreadStateEntry::from_state(&planning.thread_id, &state)?,
        )?;
        let mut launch = build_epiphany_role_launch_request(
            &planning.thread_id,
            EpiphanyRoleResultRoleId::Imagination,
            Some(state.revision),
            Some(60),
            &state,
        )
        .map_err(|error| anyhow!(error))?;
        launch.frontier_planning_request_id = Some(planning.request_id.clone());
        Ok((store, state, launch, planning))
    }

    fn frontier_planning_result(
        planning: &RepoFrontierPlanningRequest,
        job_id: &str,
        proposed_at: &str,
    ) -> Result<EpiphanyRuntimeRoleWorkerResult> {
        let mut candidate = RepoFrontierPlanCandidate {
            schema_version: REPO_FRONTIER_PLAN_CANDIDATE_SCHEMA_VERSION.into(),
            candidate_id: String::new(),
            planning_request_id: planning.request_id.clone(),
            model_revision: planning.model_revision,
            model_hash: planning.model_hash.clone(),
            frontier_item_id: planning.frontier_item_id.clone(),
            frontier_item_hash: planning.frontier_item_hash.clone(),
            safe_paths: vec![planning.source_scope[0].clone()],
            action: "Implement the bounded frontier plan.".into(),
            command: "cargo test --lib".into(),
            checks: vec!["Focused tests pass.".into()],
            stop_conditions: vec!["Authority scope changes.".into()],
            rollback_steps: vec!["Revert the bounded commit.".into()],
            commit_message: "Implement bounded frontier plan".into(),
            proposed_at: proposed_at.into(),
            contract: REPO_FRONTIER_PLANNING_CONTRACT.into(),
        };
        candidate.candidate_id = canonical_repo_frontier_plan_candidate_id(&candidate)?;
        Ok(EpiphanyRuntimeRoleWorkerResult {
            schema_version: RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION.into(),
            repository_body_observation_basis: None,
            result_id: format!("frontier-planning-result-{job_id}"),
            job_id: job_id.into(),
            role_id: "imagination".into(),
            verdict: "draft-ready".into(),
            summary: "Produced one bounded candidate.".into(),
            next_safe_move: "Mind admission.".into(),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: planning.source_scope.clone(),
            frontier_node_ids: vec![planning.frontier_item_id.clone()],
            evidence_ids: vec![planning.admission_receipt_id.clone()],
            artifact_refs: Vec::new(),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            state_patch_msgpack: None,
            self_patch_msgpack: None,
            item_error: None,
            metadata: std::collections::BTreeMap::new(),
            repo_model_patch_msgpack: None,
            verification_request_id: None,
            frontier_route_id: None,
            repo_frontier_modeling_request_id: None,
            proposal_modeling_request_id: None,
            claim_repair_request_id: None,
            frontier_planning_request_id: Some(planning.request_id.clone()),
            frontier_plan_candidate_msgpack: Some(rmp_serde::to_vec_named(&candidate)?),
            frontier_plan_mind_request_id: None,
            frontier_plan_mind_decision_msgpack: None,
            imagination_consideration_request_id: None,
            imagination_consideration_candidate_msgpack: None,
            admitted_model_direction_consideration_request_id: None,
            admitted_model_direction_consideration_result_msgpack: None,
        })
    }

    fn consideration_launch_fixture(
        root: &Path,
        suffix: &str,
    ) -> Result<(
        std::path::PathBuf,
        EpiphanyThreadState,
        EpiphanyJobLaunchRequest,
        ImaginationConsiderationRequest,
    )> {
        let (store, state, mut launch, planning) = frontier_planning_launch_fixture(root, suffix)?;
        launch.frontier_planning_request_id = None;
        let request = ImaginationConsiderationRequest {
            schema_version: IMAGINATION_CONSIDERATION_REQUEST_SCHEMA_VERSION.into(),
            request_id: format!("consideration-{suffix}"),
            feedback_id: format!("feedback-{suffix}"),
            feedback_admission_id: format!("admission-{suffix}"),
            feedback_packet_sha256: format!("sha256-feedback-{suffix}"),
            source_room_id: "discord://room".into(),
            source_visibility: "organization".into(),
            data_classification: "organization_feedback".into(),
            source_provider_identity_id: "bifrost-test".into(),
            runtime_id: planning.runtime_id.clone(),
            thread_id: planning.thread_id.clone(),
            repository: "GameCult/Epiphany".into(),
            persona_id: "epiphany".into(),
            model_revision: planning.model_revision,
            model_hash: planning.model_hash.clone(),
            model_admission_receipt_id: planning.admission_receipt_id.clone(),
            routing_policy_id: "feedback-consideration-v0".into(),
            question:
                ImaginationConsiderationQuestion::CompareWithCurrentBodyAndSuggestCoherentOptions,
            quoted_evidence: QuotedPersonaFeedbackEvidence {
                feedback_text: "Please improve map legibility".into(),
                source_discussion_refs: vec!["discord://message".into()],
                source_room_id: "discord://room".into(),
                source_visibility: "organization".into(),
                data_classification: "organization_feedback".into(),
                source_actor_id: "actor".into(),
                source_provider: "bifrost".into(),
            },
            requested_at: "2026-07-18T00:00:00Z".into(),
            contract: IMAGINATION_CONSIDERATION_REQUEST_CONTRACT.into(),
            private_state_included: false,
        };
        let mut cache = coordinator_acceptance_cache(&store)?;
        cache.pull_all_backing_stores()?;
        cache.put(&request.request_id, &request)?;
        launch.imagination_consideration_request_id = Some(request.request_id.clone());
        Ok((store, state, launch, request))
    }

    fn consideration_result(
        request: &ImaginationConsiderationRequest,
        job_id: &str,
    ) -> Result<EpiphanyRuntimeRoleWorkerResult> {
        let candidate = ImaginationConsiderationCandidate {
            schema_version: IMAGINATION_CONSIDERATION_CANDIDATE_SCHEMA_VERSION.into(),
            candidate_id: imagination_consideration_candidate_id_for_launch(
                &request.request_id,
                job_id,
            ),
            request_id: request.request_id.clone(),
            feedback_id: request.feedback_id.clone(),
            feedback_packet_sha256: request.feedback_packet_sha256.clone(),
            source_room_id: request.quoted_evidence.source_room_id.clone(),
            source_visibility: request.quoted_evidence.source_visibility.clone(),
            data_classification: request.quoted_evidence.data_classification.clone(),
            model_revision: request.model_revision,
            model_hash: request.model_hash.clone(),
            disposition: ImaginationConsiderationDisposition::Suggest,
            title: "Improve map legibility".into(),
            summary: "One reviewable option".into(),
            rationale: "Compared feedback with current Body map".into(),
            option_drafts: vec![ImaginationOptionDraft {
                title: "Clarify owner labels".into(),
                summary: "Ask Modeling to review".into(),
            }],
            uncertainties: vec![],
            evidence_refs: request.quoted_evidence.source_discussion_refs.clone(),
            recommended_review_route: ImaginationConsiderationReviewRoute::ModelingReview,
            proposed_at: "2026-07-18T00:01:00Z".into(),
            contract: IMAGINATION_CONSIDERATION_CANDIDATE_CONTRACT.into(),
        };
        Ok(EpiphanyRuntimeRoleWorkerResult {
            schema_version: RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION.into(),
            repository_body_observation_basis: None,
            result_id: format!("consideration-result-{job_id}"),
            job_id: job_id.into(),
            role_id: "imagination".into(),
            verdict: "proposal-only".into(),
            summary: "Considered quoted feedback".into(),
            next_safe_move: "Optional Self review request".into(),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: vec![],
            frontier_node_ids: vec![],
            evidence_ids: vec![request.feedback_admission_id.clone()],
            artifact_refs: vec![],
            open_questions: vec![],
            evidence_gaps: vec![],
            risks: vec![],
            state_patch_msgpack: None,
            self_patch_msgpack: None,
            item_error: None,
            metadata: std::collections::BTreeMap::new(),
            repo_model_patch_msgpack: None,
            verification_request_id: None,
            frontier_route_id: None,
            repo_frontier_modeling_request_id: None,
            proposal_modeling_request_id: None,
            claim_repair_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_candidate_msgpack: None,
            frontier_plan_mind_request_id: None,
            frontier_plan_mind_decision_msgpack: None,
            imagination_consideration_request_id: Some(request.request_id.clone()),
            imagination_consideration_candidate_msgpack: Some(rmp_serde::to_vec_named(&candidate)?),
            admitted_model_direction_consideration_request_id: None,
            admitted_model_direction_consideration_result_msgpack: None,
        })
    }

    fn launch_frontier_mind_result(
        store: &Path,
        imagination_result: &EpiphanyRuntimeRoleWorkerResult,
        decision: RepoFrontierPlanDecision,
        suffix: &str,
    ) -> Result<EpiphanyRuntimeRoleWorkerResult> {
        let mind_request = crate::commit_repo_frontier_plan_mind_request(
            store,
            &imagination_result.result_id,
            "2026-07-15T09:00:05Z",
        )
        .map_err(|error| anyhow!("commit frontier Mind request: {error}"))?;
        let mut cache = coordinator_acceptance_cache(store)?;
        cache.pull_all_backing_stores()?;
        let state = cache
            .get_required::<EpiphanyThreadStateEntry>(THREAD_STATE_KEY)?
            .state()?;
        let launch = crate::build_epiphany_frontier_plan_mind_launch_request(
            &mind_request.thread_id,
            Some(state.revision),
            Some(60),
            &state,
            mind_request.request_id.clone(),
        )
        .map_err(|error| anyhow!(error))?;
        let job_id = format!("backend-mind-decision-{suffix}");
        let plan = plan_coordinator_job_launch(
            &state,
            &launch,
            store,
            format!("launcher-mind-{suffix}"),
            job_id.clone(),
        )?;
        commit_coordinator_job_launch(
            store,
            &mind_request.thread_id,
            &state,
            &launch,
            &plan,
            "2026-07-15T09:00:06Z".into(),
        )?;
        let payload = RepoFrontierPlanMindDecision {
            mind_request_id: mind_request.request_id.clone(),
            planning_request_id: mind_request.planning_request_id.clone(),
            imagination_result_id: mind_request.imagination_result_id.clone(),
            candidate_id: mind_request.candidate_id.clone(),
            candidate_sha256: mind_request.candidate_sha256.clone(),
            decision,
            rationale: format!("Mind judged the exact typed candidate as {decision:?}."),
            decided_at: "2026-07-15T09:00:07Z".into(),
        };
        let result = EpiphanyRuntimeRoleWorkerResult {
            schema_version: RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION.into(),
            repository_body_observation_basis: None,
            result_id: format!("frontier-mind-result-{suffix}"),
            job_id,
            role_id: "mindAdmissionReview".into(),
            verdict: format!("{decision:?}").to_lowercase(),
            summary: "Judged one exact frontier plan candidate.".into(),
            next_safe_move: "Coordinator may admit this immutable judgment.".into(),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: Vec::new(),
            frontier_node_ids: Vec::new(),
            evidence_ids: vec![mind_request.request_id.clone()],
            artifact_refs: Vec::new(),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            state_patch_msgpack: None,
            self_patch_msgpack: None,
            item_error: None,
            metadata: std::collections::BTreeMap::new(),
            repo_model_patch_msgpack: None,
            verification_request_id: None,
            frontier_route_id: None,
            repo_frontier_modeling_request_id: None,
            proposal_modeling_request_id: None,
            claim_repair_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_candidate_msgpack: None,
            frontier_plan_mind_request_id: Some(mind_request.request_id),
            frontier_plan_mind_decision_msgpack: Some(rmp_serde::to_vec_named(&payload)?),
            imagination_consideration_request_id: None,
            imagination_consideration_candidate_msgpack: None,
            admitted_model_direction_consideration_request_id: None,
            admitted_model_direction_consideration_result_msgpack: None,
        };
        put_runtime_role_worker_result(store, &result)
            .map_err(|error| anyhow!("persist frontier Mind result: {error}"))?;
        Ok(result)
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
        let mut transaction =
            coordinator_state_transaction::open_coordinator_state_transaction(&store, &state)?;
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
                &mut transaction,
                "thread-1",
                &state,
                &request,
                &plan,
                "2026-07-11T00:00:00Z".to_string(),
                None,
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
        let cache = coordinator_acceptance_cache(&store)?;
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

    #[test]
    fn claim_repair_launch_is_coordinator_owned_bound_and_single_use() -> Result<()> {
        let root = tempfile::tempdir()?;
        let (store, state, launch, repair) = claim_repair_launch_fixture(root.path(), "exact")?;
        for mutation in 0..4 {
            let mut forged = launch.clone();
            match mutation {
                0 => forged.owner_role = "Eyes".into(),
                1 => forged.binding_id = "wrong-binding".into(),
                2 => {
                    if let EpiphanyWorkerLaunchDocument::Role(document) =
                        &mut forged.launch_document
                    {
                        document.claim_repair_context =
                            Some(RepoModelClaimRepairContextProjection::from_request(&repair));
                    }
                }
                _ => forged.proposal_modeling_request_id = Some("dual-proposal".into()),
            }
            let before = std::fs::read(&store)?;
            assert!(
                plan_coordinator_job_launch(
                    &state,
                    &forged,
                    &store,
                    format!("lf-{mutation}"),
                    format!("bf-{mutation}")
                )
                .is_err()
            );
            assert_eq!(std::fs::read(&store)?, before);
        }
        let plan = plan_coordinator_job_launch(
            &state,
            &launch,
            &store,
            "launcher-repair".into(),
            "backend-repair".into(),
        )?;
        let committed = commit_coordinator_job_launch(
            &store,
            &repair.thread_id,
            &state,
            &launch,
            &plan,
            "2026-07-14T09:00:03Z".into(),
        )?;
        let persisted =
            runtime_worker_launch_request(&store, "backend-repair")?.expect("repair launch");
        let projection = match persisted.launch_document()? {
            EpiphanyWorkerLaunchDocument::Role(document) => {
                document.claim_repair_context.expect("typed repair context")
            }
            EpiphanyWorkerLaunchDocument::Reorient(_) => panic!("repair cannot reorient"),
        };
        assert_eq!(projection.request_id, repair.request_id);
        assert!(projection.affected_frontier == repair.affected_frontier);
        let cache = coordinator_acceptance_cache(&store)?;
        let binding = cache
            .get::<RepoModelClaimRepairLaunchBinding>(&format!(
                "repo-model-claim-repair-launch-{}",
                repair.request_id
            ))?
            .expect("repair binding");
        assert_eq!(binding.challenge_id, repair.challenge_id);
        assert_eq!(binding.runtime_id, repair.runtime_id);
        assert_eq!(binding.thread_id, repair.thread_id);
        assert_eq!(
            binding.worker_launch_document_sha256,
            format!("{:x}", Sha256::digest(&persisted.launch_document_msgpack))
        );
        let mut second = build_epiphany_role_launch_request(
            &repair.thread_id,
            EpiphanyRoleResultRoleId::Modeling,
            Some(committed.epiphany_state.revision),
            Some(60),
            &committed.epiphany_state,
        )
        .map_err(|error| anyhow!(error))?;
        second.claim_repair_request_id = Some(repair.request_id);
        let before = std::fs::read(&store)?;
        assert!(
            plan_coordinator_job_launch(
                &committed.epiphany_state,
                &second,
                &store,
                "launcher-second".into(),
                "backend-second".into()
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&store)?, before);
        Ok(())
    }

    #[test]
    fn frontier_planning_launch_is_coordinator_owned_exact_and_single_use() -> Result<()> {
        let root = tempfile::tempdir()?;
        let (store, state, launch, planning) =
            frontier_planning_launch_fixture(root.path(), "exact")?;
        for mutation in 0..5 {
            let mut forged = launch.clone();
            match mutation {
                0 => forged.owner_role = "Modeling".into(),
                1 => forged.binding_id = "wrong-binding".into(),
                2 => {
                    if let EpiphanyWorkerLaunchDocument::Role(document) =
                        &mut forged.launch_document
                    {
                        document.frontier_planning_context = Some(
                            RepoFrontierPlanningContextProjection::from_request(&planning),
                        );
                    }
                }
                3 => forged.claim_repair_request_id = Some("dual-repair".into()),
                _ => forged.proposal_modeling_request_id = Some("dual-proposal".into()),
            }
            let before = std::fs::read(&store)?;
            assert!(
                plan_coordinator_job_launch(
                    &state,
                    &forged,
                    &store,
                    format!("launcher-forged-{mutation}"),
                    format!("backend-forged-{mutation}"),
                )
                .is_err()
            );
            assert_eq!(std::fs::read(&store)?, before);
        }
        let plan = plan_coordinator_job_launch(
            &state,
            &launch,
            &store,
            "launcher-planning".into(),
            "backend-planning".into(),
        )?;
        let committed = commit_coordinator_job_launch(
            &store,
            &planning.thread_id,
            &state,
            &launch,
            &plan,
            "2026-07-15T09:00:03Z".into(),
        )?;
        let persisted = runtime_worker_launch_request(&store, "backend-planning")?
            .expect("planning worker launch");
        let projection = match persisted.launch_document()? {
            EpiphanyWorkerLaunchDocument::Role(document) => document
                .frontier_planning_context
                .expect("coordinator-owned planning context"),
            EpiphanyWorkerLaunchDocument::Reorient(_) => panic!("planning cannot reorient"),
        };
        assert_eq!(projection.request_id, planning.request_id);
        assert_eq!(projection.runtime_id, planning.runtime_id);
        assert_eq!(projection.thread_id, planning.thread_id);
        let cache = coordinator_acceptance_cache(&store)?;
        let binding_key = format!("repo-frontier-planning-launch-{}", planning.request_id);
        let binding = cache
            .get::<RepoFrontierPlanningLaunchBinding>(&binding_key)?
            .expect("request-keyed planning launch binding");
        assert_eq!(binding.job_id, "backend-planning");
        assert_eq!(binding.runtime_id, planning.runtime_id);
        assert_eq!(binding.thread_id, planning.thread_id);
        assert_eq!(
            binding.worker_launch_document_sha256,
            format!("{:x}", Sha256::digest(&persisted.launch_document_msgpack))
        );
        let mut second = build_epiphany_role_launch_request(
            &planning.thread_id,
            EpiphanyRoleResultRoleId::Imagination,
            Some(committed.epiphany_state.revision),
            Some(60),
            &committed.epiphany_state,
        )
        .map_err(|error| anyhow!(error))?;
        second.frontier_planning_request_id = Some(planning.request_id);
        let before = std::fs::read(&store)?;
        assert!(
            plan_coordinator_job_launch(
                &committed.epiphany_state,
                &second,
                &store,
                "launcher-second".into(),
                "backend-second".into(),
            )
            .is_err()
        );
        assert_eq!(std::fs::read(&store)?, before);
        Ok(())
    }

    #[test]
    fn consideration_launch_and_result_are_exact_single_use_and_patch_sealed() -> Result<()> {
        let root = tempfile::tempdir()?;
        let (store, state, launch, request) = consideration_launch_fixture(root.path(), "exact")?;
        let plan = plan_coordinator_job_launch(
            &state,
            &launch,
            &store,
            "launcher-consideration".into(),
            "backend-consideration".into(),
        )?;
        commit_coordinator_job_launch(
            &store,
            &request.thread_id,
            &state,
            &launch,
            &plan,
            "2026-07-18T00:00:01Z".into(),
        )?;
        assert!(
            commit_coordinator_job_launch(
                &store,
                &request.thread_id,
                &state,
                &launch,
                &plan,
                "2026-07-18T00:00:02Z".into()
            )
            .is_err()
        );
        let result = consideration_result(&request, "backend-consideration")?;
        put_runtime_role_worker_result(&store, &result)?;
        let mut cargo = consideration_result(&request, "backend-consideration")?;
        cargo.result_id = "cargo-result".into();
        cargo.state_patch_msgpack = Some(vec![0]);
        assert!(put_runtime_role_worker_result(&store, &cargo).is_err());
        Ok(())
    }

    #[test]
    fn consideration_refuses_caller_context_stale_model_and_substituted_candidate() -> Result<()> {
        for mutation in 0..3 {
            let root = tempfile::tempdir()?;
            let (store, state, mut launch, mut request) =
                consideration_launch_fixture(root.path(), &format!("reject-{mutation}"))?;
            if mutation == 0 {
                if let EpiphanyWorkerLaunchDocument::Role(document) = &mut launch.launch_document {
                    let model = runtime_current_repo_model(&store)?.unwrap();
                    document.imagination_consideration_context = Some(
                        ImaginationConsiderationContextProjection::new(&request, &model),
                    );
                }
            } else if mutation == 1 {
                request.model_hash = "stale-model".into();
                let mut cache = coordinator_acceptance_cache(&store)?;
                cache.pull_all_backing_stores()?;
                cache.put(&request.request_id, &request)?;
            }
            let planned = plan_coordinator_job_launch(
                &state,
                &launch,
                &store,
                format!("launcher-{mutation}"),
                format!("backend-{mutation}"),
            );
            if mutation < 2 {
                let plan = planned?;
                assert!(
                    commit_coordinator_job_launch(
                        &store,
                        &request.thread_id,
                        &state,
                        &launch,
                        &plan,
                        "2026-07-18T00:00:01Z".into()
                    )
                    .is_err()
                );
                continue;
            }
            let plan = planned?;
            commit_coordinator_job_launch(
                &store,
                &request.thread_id,
                &state,
                &launch,
                &plan,
                "2026-07-18T00:00:01Z".into(),
            )?;
            let mut result = consideration_result(&request, &format!("backend-{mutation}"))?;
            let mut candidate = result.imagination_consideration_candidate()?.unwrap();
            candidate.feedback_packet_sha256 = "substituted".into();
            result.imagination_consideration_candidate_msgpack =
                Some(rmp_serde::to_vec_named(&candidate)?);
            assert!(put_runtime_role_worker_result(&store, &result).is_err());
        }
        Ok(())
    }

    #[test]
    fn frontier_planning_launch_refuses_swapped_request_bytes_without_writes() -> Result<()> {
        let root = tempfile::tempdir()?;
        for mutation in 0..8 {
            let (store, state, launch, planning) =
                frontier_planning_launch_fixture(root.path(), &format!("causal-{mutation}"))?;
            let mut corrupt = planning.clone();
            match mutation {
                0 => corrupt.model_hash = "swapped-model".into(),
                1 => corrupt.admission_receipt_id = "swapped-admission".into(),
                2 => corrupt.frontier_item_id = "swapped-frontier".into(),
                3 => corrupt.frontier_item_hash = "swapped-frontier-hash".into(),
                4 => corrupt.selected_organ = "Hands".into(),
                5 => corrupt.source_scope.push("outside/scope".into()),
                6 => corrupt.runtime_id = "swapped-runtime".into(),
                _ => corrupt.thread_id = "swapped-thread".into(),
            }
            let mut cache = coordinator_acceptance_cache(&store)?;
            cache.put(&corrupt.request_id, &corrupt)?;
            let before = std::fs::read(&store)?;
            assert!(
                plan_coordinator_job_launch(
                    &state,
                    &launch,
                    &store,
                    format!("launcher-causal-{mutation}"),
                    format!("backend-causal-{mutation}"),
                )
                .is_err(),
                "planning causal mutation {mutation} must be refused"
            );
            assert_eq!(std::fs::read(&store)?, before);
        }
        Ok(())
    }

    #[test]
    fn frontier_planning_result_is_exact_immutable_and_patch_sealed() -> Result<()> {
        let root = tempfile::tempdir()?;
        let (store, state, launch, planning) =
            frontier_planning_launch_fixture(root.path(), "result-exact")?;
        let plan = plan_coordinator_job_launch(
            &state,
            &launch,
            &store,
            "launcher-result".into(),
            "backend-result".into(),
        )?;
        commit_coordinator_job_launch(
            &store,
            &planning.thread_id,
            &state,
            &launch,
            &plan,
            "2026-07-15T09:00:03Z".into(),
        )?;
        let result = frontier_planning_result(&planning, "backend-result", "2026-07-15T09:00:04Z")?;
        put_runtime_role_worker_result(&store, &result)?;
        let stored = runtime_role_worker_result(&store, "backend-result")?
            .expect("immutable planning result");
        assert_eq!(stored, result);
        let candidate = stored
            .frontier_plan_candidate()?
            .expect("typed candidate payload");
        assert_eq!(candidate.planning_request_id, planning.request_id);
        assert_eq!(
            candidate.candidate_id,
            canonical_repo_frontier_plan_candidate_id(&candidate)?
        );
        assert!(
            coordinator_acceptance_cache(&store)?
                .get_all::<RepoFrontierPlanCandidate>()?
                .is_empty(),
            "candidate must remain inside immutable result until Mind admission"
        );
        let mut counterfeit = result;
        counterfeit.summary = "different result bytes".into();
        let before = std::fs::read(&store)?;
        assert!(put_runtime_role_worker_result(&store, &counterfeit).is_err());
        assert_eq!(std::fs::read(&store)?, before);
        Ok(())
    }

    #[test]
    fn frontier_planning_result_refuses_hostile_substitution_before_persistence() -> Result<()> {
        let root = tempfile::tempdir()?;
        for mutation in 0..10 {
            let (store, state, launch, planning) =
                frontier_planning_launch_fixture(root.path(), &format!("result-{mutation}"))?;
            let job_id = format!("backend-result-{mutation}");
            let plan = plan_coordinator_job_launch(
                &state,
                &launch,
                &store,
                format!("launcher-result-{mutation}"),
                job_id.clone(),
            )?;
            commit_coordinator_job_launch(
                &store,
                &planning.thread_id,
                &state,
                &launch,
                &plan,
                "2026-07-15T09:00:03Z".into(),
            )?;
            let mut result = frontier_planning_result(&planning, &job_id, "2026-07-15T09:00:04Z")?;
            match mutation {
                0 => result.frontier_planning_request_id = Some("swapped-request".into()),
                1 => result.frontier_planning_request_id = None,
                2 => result.frontier_plan_candidate_msgpack = None,
                3 => result.role_id = "modeling".into(),
                4 => result.state_patch_msgpack = Some(vec![0]),
                5 => result.self_patch_msgpack = Some(vec![0]),
                6 => result.repo_model_patch_msgpack = Some(vec![0]),
                7 | 8 => {
                    let mut candidate = result.frontier_plan_candidate()?.unwrap();
                    if mutation == 7 {
                        candidate.frontier_item_hash = "adjacent-frontier".into();
                    } else {
                        candidate.safe_paths = vec!["outside/authority".into()];
                    }
                    result.frontier_plan_candidate_msgpack =
                        Some(rmp_serde::to_vec_named(&candidate)?);
                }
                _ => result.item_error = Some("planner failed after emitting cargo".into()),
            }
            let before = std::fs::read(&store)?;
            assert!(
                put_runtime_role_worker_result(&store, &result).is_err(),
                "hostile planning result mutation {mutation} must fail"
            );
            assert_eq!(std::fs::read(&store)?, before);
            assert!(runtime_role_worker_result(&store, &job_id)?.is_none());
        }
        Ok(())
    }

    #[test]
    fn mind_adopt_installs_plan_in_model_and_hands_route_copies_it() -> Result<()> {
        let root = tempfile::tempdir()?;
        let (store, state, launch, planning) =
            frontier_planning_launch_fixture(root.path(), "mind-adopt")
                .map_err(|error| anyhow!("build frontier planning launch fixture: {error}"))?;
        let plan = plan_coordinator_job_launch(
            &state,
            &launch,
            &store,
            "launcher-mind-adopt".into(),
            "backend-mind-adopt".into(),
        )?;
        commit_coordinator_job_launch(
            &store,
            &planning.thread_id,
            &state,
            &launch,
            &plan,
            "2026-07-15T09:00:03Z".into(),
        )
        .map_err(|error| anyhow!("commit frontier Imagination launch: {error}"))?;
        let result =
            frontier_planning_result(&planning, "backend-mind-adopt", "2026-07-15T09:00:04Z")?;
        put_runtime_role_worker_result(&store, &result)
            .map_err(|error| anyhow!("persist frontier Imagination result: {error}"))?;
        let mind_result =
            launch_frontier_mind_result(&store, &result, RepoFrontierPlanDecision::Adopt, "adopt")
                .map_err(|error| anyhow!("launch frontier Mind result: {error}"))?;

        let before = runtime_current_repo_model(&store)?.expect("pre-Adopt model");
        let mut illicit_item = before
            .frontier
            .iter()
            .find(|item| item.id == planning.frontier_item_id)
            .unwrap()
            .clone();
        illicit_item.recommended_next_organ = "Hands".into();
        let illicit = RepoModelPatch {
            patch_id: "generic-adopt-bypass".into(),
            base_revision: before.model_revision,
            base_hash: memory_graph_model_hash(&before)?,
            applied_at: "2026-07-15T09:00:05Z".into(),
            purpose: RepoModelPatchPurpose::Evolution,
            operations: vec![RepoModelPatchOperation::ReviseFrontier { item: illicit_item }],
        };
        assert!(derive_repo_model_patch(&before, &illicit).is_err());

        let decision = commit_repo_frontier_plan_decision(&store, &mind_result.result_id)
            .map_err(|error| anyhow!("commit frontier plan decision: {error}"))?;
        let retry = commit_repo_frontier_plan_decision(&store, &mind_result.result_id)
            .map_err(|error| anyhow!("replay frontier plan decision: {error}"))?;
        assert_eq!(retry, decision);
        assert!(!decision.model_admission_receipt_id.is_empty());
        let current = runtime_current_repo_model(&store)?.expect("admitted Adopt model");
        assert_eq!(current.model_revision, before.model_revision + 1);
        let item = current
            .frontier
            .iter()
            .find(|item| item.id == planning.frontier_item_id)
            .expect("adopted frontier");
        assert_eq!(item.recommended_next_organ, "Hands");
        let adopted = item.adopted_plan.as_ref().expect("model-owned plan");
        assert_eq!(adopted.planning_request_id, planning.request_id);
        assert_eq!(adopted.result_id, result.result_id);
        assert_eq!(adopted.job_id, result.job_id);
        assert_eq!(adopted.candidate_id, decision.candidate_id);
        assert_eq!(adopted.candidate_sha256, decision.candidate_sha256);
        let mut illicit_adopted_item = item.clone();
        illicit_adopted_item.source_scope = vec!["different/execution-scope".into()];
        let illicit_adopted_revision = RepoModelPatch {
            patch_id: "generic-adopted-anatomy-mutation".into(),
            base_revision: current.model_revision,
            base_hash: memory_graph_model_hash(&current)?,
            applied_at: "2026-07-15T09:00:06Z".into(),
            purpose: RepoModelPatchPurpose::Evolution,
            operations: vec![RepoModelPatchOperation::ReviseFrontier {
                item: illicit_adopted_item,
            }],
        };
        assert!(derive_repo_model_patch(&current, &illicit_adopted_revision).is_err());
        let route = select_and_commit_repo_frontier_route(&store, "2026-07-15T09:00:06Z")?;
        assert_eq!(route.adopted_plan.as_ref(), Some(adopted));
        assert_eq!(route.source_scope, adopted.safe_paths);
        let grant = crate::substrate_gate_coordinator_implementation_grant(
            "adopt-hands-grant".into(),
            "adopt-hands-job".into(),
            adopted.safe_paths.clone(),
            "2026-07-15T09:00:08Z".into(),
        );
        crate::put_substrate_gate_repo_access_grant_receipt(&store, &grant)?;
        let intent = crate::HandsActionIntent {
            schema_version: crate::HANDS_ACTION_INTENT_SCHEMA_VERSION.into(),
            intent_id: "adopt-hands-intent".into(),
            runtime_job_id: grant.runtime_job_id.clone(),
            binding_id: grant.binding_id.clone(),
            role: grant.role.clone(),
            authority_scope: grant.authority_scope.clone(),
            requested_action: "continueImplementation".into(),
            requested_paths: adopted.safe_paths.clone(),
            substrate_gate_grant_receipt_id: grant.receipt_id.clone(),
            requested_at: "2026-07-15T09:00:09Z".into(),
            contract: "Hands intent executes only the admitted plan.".into(),
            frontier_route_id: route.route_id.clone(),
            plan_candidate_sha256: adopted.candidate_sha256.clone(),
            plan_action: adopted.action.clone(),
        };
        crate::put_hands_action_intent(&store, &intent)?;
        let review = crate::hands_action_review_for_intent(
            "adopt-hands-review".into(),
            &intent,
            "approved".into(),
            vec!["patch".into(), "command".into(), "commit".into()],
            vec!["Exact admitted plan binding is present.".into()],
            "2026-07-15T09:00:10Z".into(),
        );
        crate::put_hands_action_review(&store, &review)?;
        let authority = crate::RepoFrontierHandsAuthority {
            schema_version: crate::REPO_FRONTIER_HANDS_AUTHORITY_SCHEMA_VERSION.into(),
            authority_id: "adopt-hands-authority".into(),
            route_id: route.route_id.clone(),
            model_revision: route.model_revision,
            model_hash: route.model_hash.clone(),
            frontier_item_id: route.frontier_item_id.clone(),
            frontier_item_hash: route.frontier_item_hash.clone(),
            hands_intent_id: intent.intent_id.clone(),
            hands_review_id: review.review_id.clone(),
            substrate_grant_receipt_id: grant.receipt_id.clone(),
            requested_paths: adopted.safe_paths.clone(),
            granted_at: "2026-07-15T09:00:11Z".into(),
            contract: crate::REPO_FRONTIER_HANDS_AUTHORITY_CONTRACT.into(),
        };
        crate::put_repo_frontier_hands_authority(&store, &authority)?;
        let patch_receipt = crate::hands_patch_receipt_for_review(
            "adopt-hands-patch".into(),
            &intent,
            &review,
            adopted.safe_paths.clone(),
            "bounded patch".into(),
            "2026-07-15T09:00:12Z".into(),
        );
        crate::put_hands_patch_receipt(&store, &patch_receipt)?;
        let wrong_command = crate::hands_command_receipt_for_review(
            "adopt-hands-command-wrong".into(),
            &intent,
            &review,
            "cargo test unrelated".into(),
            "0".into(),
            "stdout".into(),
            "stderr".into(),
            "wrong command".into(),
            "2026-07-15T09:00:13Z".into(),
        );
        let commit_receipt = crate::hands_commit_receipt_for_review(
            "adopt-hands-commit".into(),
            &intent,
            &review,
            "abc123".into(),
            "main".into(),
            adopted.safe_paths.clone(),
            adopted.commit_message.clone(),
            "2026-07-15T09:00:14Z".into(),
        );
        crate::put_hands_commit_receipt(&store, &commit_receipt)?;
        let hostile_request = crate::RepoFrontierVerificationRequest {
            schema_version: crate::REPO_FRONTIER_VERIFICATION_REQUEST_SCHEMA_VERSION.into(),
            request_id: "adopt-verification-hostile".into(),
            route_id: route.route_id.clone(),
            model_revision: route.model_revision,
            model_hash: route.model_hash.clone(),
            frontier_item_id: route.frontier_item_id.clone(),
            frontier_item_hash: route.frontier_item_hash.clone(),
            hands_intent_id: intent.intent_id.clone(),
            hands_review_id: review.review_id.clone(),
            hands_patch_receipt_id: patch_receipt.receipt_id.clone(),
            hands_command_receipt_id: wrong_command.receipt_id.clone(),
            hands_commit_receipt_id: commit_receipt.receipt_id.clone(),
            requested_at: "2026-07-15T09:00:15Z".into(),
            contract: crate::REPO_FRONTIER_VERIFICATION_REQUEST_CONTRACT.into(),
        };
        assert!(crate::put_repo_frontier_verification_request(&store, &hostile_request).is_err());

        // Prove the whole admitted-plan nerve, not merely its neighboring joints. The
        // rejected command above remains immutable evidence, while the exact planned
        // command becomes the only chain Soul is allowed to verify.
        let exact_command = crate::hands_command_receipt_for_review(
            "adopt-hands-command-exact".into(),
            &intent,
            &review,
            adopted.command.clone(),
            "0".into(),
            "stdout-exact".into(),
            "stderr-exact".into(),
            "exact admitted command passed".into(),
            "2026-07-15T09:00:13Z".into(),
        );
        crate::put_hands_command_receipt(&store, &exact_command)?;
        let chain =
            crate::runtime_latest_hands_receipt_chain_after(&store, "2026-07-15T09:00:11Z")?
                .expect("exact admitted Hands chain");
        assert_eq!(chain.patch_receipt_id, patch_receipt.receipt_id);
        assert_eq!(chain.command_receipt_id, exact_command.receipt_id);
        assert_eq!(chain.commit_receipt_id, commit_receipt.receipt_id);
        let verification_request = crate::commit_repo_frontier_verification_request_for_chain(
            &store,
            &chain,
            "2026-07-15T09:00:17Z",
        )?;
        assert_eq!(verification_request.route_id, route.route_id);

        let verification_state = EpiphanyThreadState::default();
        let verification_launch = build_epiphany_role_launch_request(
            &planning.thread_id,
            EpiphanyRoleResultRoleId::Verification,
            Some(verification_state.revision),
            Some(60),
            &verification_state,
        )
        .map_err(|error| anyhow!(error))?;
        crate::open_runtime_spine_heartbeat_job(
            &store,
            crate::RuntimeSpineHeartbeatJobOptions {
                runtime_id: "proposal-runtime-mind-adopt".into(),
                display_name: "Adopt verification fixture".into(),
                session_id: planning.thread_id.clone(),
                objective: "Verify the exact adopted plan consequence.".into(),
                coordinator_note: "Fixture immutable Verification launch.".into(),
                job_id: "adopt-verification-job".into(),
                role: verification_launch.owner_role,
                binding_id: verification_launch.binding_id,
                authority_scope: verification_launch.authority_scope,
                instruction: verification_launch.instruction,
                launch_document: verification_launch.launch_document,
                output_contract_id: verification_launch.output_contract_id,
                organ_launch_contract: verification_launch.organ_launch_contract,
                proposal_modeling_request_id: None,
                claim_repair_request_id: None,
                frontier_planning_request_id: None,
                frontier_plan_mind_request_id: None,
                imagination_consideration_request_id: None,
                admitted_model_direction_consideration_request_id: None,
                created_at: "2026-07-15T09:00:17Z".into(),
            },
        )?;

        let verification_result = crate::EpiphanyRuntimeRoleWorkerResult {
            schema_version: crate::RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION.into(),
            repository_body_observation_basis: None,
            result_id: "adopt-verification-result".into(),
            job_id: "adopt-verification-job".into(),
            role_id: "verification".into(),
            verdict: "pass".into(),
            summary: "Exact adopted plan passed Soul verification.".into(),
            next_safe_move: "Modeling incorporates the verdict.".into(),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: adopted.safe_paths.clone(),
            frontier_node_ids: vec![route.frontier_item_id.clone()],
            evidence_ids: vec![verification_request.request_id.clone()],
            artifact_refs: Vec::new(),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            state_patch_msgpack: None,
            self_patch_msgpack: None,
            item_error: None,
            metadata: Default::default(),
            repo_model_patch_msgpack: None,
            verification_request_id: Some(verification_request.request_id.clone()),
            frontier_route_id: Some(route.route_id.clone()),
            repo_frontier_modeling_request_id: None,
            proposal_modeling_request_id: None,
            claim_repair_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_candidate_msgpack: None,
            frontier_plan_mind_request_id: None,
            frontier_plan_mind_decision_msgpack: None,
            imagination_consideration_request_id: None,
            imagination_consideration_candidate_msgpack: None,
            admitted_model_direction_consideration_request_id: None,
            admitted_model_direction_consideration_result_msgpack: None,
        };
        crate::put_runtime_role_worker_result(&store, &verification_result)?;
        let soul_verdict = crate::SoulVerdictReceipt {
            schema_version: crate::SOUL_VERDICT_RECEIPT_SCHEMA_VERSION.into(),
            receipt_id: "adopt-soul-verdict".into(),
            source_result_id: verification_result.result_id.clone(),
            source_job_id: verification_result.job_id.clone(),
            verdict: "pass".into(),
            summary: verification_result.summary.clone(),
            evidence_ids: verification_result.evidence_ids.clone(),
            risks: Vec::new(),
            emitted_at: "2026-07-15T09:00:18Z".into(),
            contract: "Soul judges the exact routed consequence chain.".into(),
            verification_request_id: verification_request.request_id.clone(),
            frontier_route_id: route.route_id.clone(),
        };
        crate::put_soul_verdict_receipt(&store, &soul_verdict)?;
        let acceptance = epiphany_state_model::EpiphanyAcceptanceReceipt {
            id: "adopt-verification-acceptance".into(),
            result_id: verification_result.result_id.clone(),
            job_id: verification_result.job_id.clone(),
            binding_id: "verification-worker".into(),
            surface: "roleAccept".into(),
            role_id: "verification".into(),
            status: "accepted".into(),
            accepted_at: "2026-07-15T09:00:18Z".into(),
            ..Default::default()
        };
        let accepted_state = epiphany_state_model::EpiphanyThreadState {
            revision: 1,
            acceptance_receipts: vec![acceptance.clone()],
            ..Default::default()
        };
        let mut state_cache = crate::runtime_spine_cache(&store)?;
        state_cache.put(
            crate::THREAD_STATE_KEY,
            &crate::EpiphanyThreadStateEntry::from_state("mind-adopt-thread", &accepted_state)?,
        )?;
        let modeling_request = crate::commit_repo_frontier_modeling_request(&store, &acceptance)?;
        let incorporation_launch = build_epiphany_role_launch_request(
            "mind-adopt-thread",
            EpiphanyRoleResultRoleId::Modeling,
            Some(accepted_state.revision),
            Some(60),
            &accepted_state,
        )
        .map_err(|error| anyhow!(error))?;
        let incorporation_launch_plan = plan_coordinator_job_launch(
            &accepted_state,
            &incorporation_launch,
            &store,
            "adopt-incorporation-launcher".into(),
            "adopt-incorporation-job".into(),
        )?;
        commit_coordinator_job_launch(
            &store,
            "mind-adopt-thread",
            &accepted_state,
            &incorporation_launch,
            &incorporation_launch_plan,
            "2026-07-15T09:00:18Z".into(),
        )?;

        let pre_incorporation =
            crate::runtime_current_repo_model(&store)?.expect("model before verdict incorporation");
        let mut resolved_item = pre_incorporation
            .frontier
            .iter()
            .find(|candidate| candidate.id == route.frontier_item_id)
            .expect("routed adopted item before incorporation")
            .clone();
        let adopted_before = rmp_serde::to_vec_named(
            resolved_item
                .adopted_plan
                .as_ref()
                .expect("adopted plan before closure"),
        )?;
        resolved_item.status = crate::RepoFrontierStatus::Resolved;
        resolved_item.updated_at = Some("2026-07-15T09:00:19Z".into());
        resolved_item
            .evidence_refs
            .push(verification_request.request_id.clone());
        resolved_item
            .evidence_refs
            .push(soul_verdict.receipt_id.clone());
        resolved_item.evidence_refs.sort();
        resolved_item.evidence_refs.dedup();
        let incorporation_patch = crate::RepoModelPatch {
            patch_id: "adopt-verdict-incorporation-patch".into(),
            base_revision: pre_incorporation.model_revision,
            base_hash: crate::memory_graph_model_hash(&pre_incorporation)?,
            applied_at: "2026-07-15T09:00:19Z".into(),
            purpose: crate::RepoModelPatchPurpose::IncorporateFrontierVerdict {
                route_id: route.route_id.clone(),
                soul_verdict_receipt_id: soul_verdict.receipt_id.clone(),
            },
            operations: vec![crate::RepoModelPatchOperation::ReviseFrontier {
                item: resolved_item,
            }],
        };
        let incorporation_bytes = rmp_serde::to_vec_named(&incorporation_patch)?;
        let modeling_result = crate::EpiphanyRuntimeRoleWorkerResult {
            schema_version: crate::RUNTIME_ROLE_WORKER_RESULT_SCHEMA_VERSION.into(),
            repository_body_observation_basis: crate::runtime_worker_launch_body_basis(
                &store,
                "adopt-incorporation-job",
            )?,
            result_id: "adopt-incorporation-result".into(),
            job_id: "adopt-incorporation-job".into(),
            role_id: "modeling".into(),
            verdict: "checkpoint-ready".into(),
            summary: "Close the verified adopted frontier.".into(),
            next_safe_move: "Mind admits the exact Modeling transition.".into(),
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: Vec::new(),
            frontier_node_ids: vec![route.frontier_item_id.clone()],
            evidence_ids: vec![soul_verdict.receipt_id.clone()],
            artifact_refs: Vec::new(),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            state_patch_msgpack: None,
            self_patch_msgpack: None,
            item_error: None,
            metadata: Default::default(),
            repo_model_patch_msgpack: Some(incorporation_bytes.clone()),
            verification_request_id: None,
            frontier_route_id: None,
            repo_frontier_modeling_request_id: Some(modeling_request.request_id.clone()),
            proposal_modeling_request_id: None,
            claim_repair_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_candidate_msgpack: None,
            frontier_plan_mind_request_id: None,
            frontier_plan_mind_decision_msgpack: None,
            imagination_consideration_request_id: None,
            imagination_consideration_candidate_msgpack: None,
            admitted_model_direction_consideration_request_id: None,
            admitted_model_direction_consideration_result_msgpack: None,
        };
        crate::put_runtime_role_worker_result(&store, &modeling_result)?;
        let incorporation_review = crate::RepoModelAdmissionReview {
            schema_version: crate::REPO_MODEL_ADMISSION_REVIEW_SCHEMA_VERSION.into(),
            review_id: "adopt-incorporation-review".into(),
            result_id: modeling_result.result_id.clone(),
            job_id: modeling_result.job_id.clone(),
            patch_id: incorporation_patch.patch_id.clone(),
            patch_sha256: format!("{:x}", Sha256::digest(&incorporation_bytes)),
            base_revision: incorporation_patch.base_revision,
            base_hash: incorporation_patch.base_hash.clone(),
            decision: crate::MindGatewayDecision::Accept,
            evidence_ids: modeling_result.evidence_ids.clone(),
            reviewed_at: "2026-07-15T09:00:20Z".into(),
            contract: crate::REPO_MODEL_ADMISSION_CONTRACT.into(),
            repository_body_observation_basis: modeling_result
                .repository_body_observation_basis
                .clone(),
        };
        let incorporation = crate::commit_repo_model_admission(
            &store,
            &modeling_result.result_id,
            &incorporation_review,
        )?;
        assert_eq!(incorporation.frontier_route_id, route.route_id);
        assert_eq!(
            incorporation.frontier_modeling_request_id,
            modeling_request.request_id
        );
        let closed = crate::runtime_current_repo_model(&store)?.expect("closed model");
        let closed_item = closed
            .frontier
            .iter()
            .find(|candidate| candidate.id == route.frontier_item_id)
            .expect("closed adopted frontier");
        assert_eq!(closed_item.status, crate::RepoFrontierStatus::Resolved);
        assert_eq!(
            rmp_serde::to_vec_named(
                closed_item
                    .adopted_plan
                    .as_ref()
                    .expect("plan survives closure")
            )?,
            adopted_before,
            "verdict incorporation must preserve the adopted plan byte-for-byte"
        );
        let cache = coordinator_acceptance_cache(&store)?;
        let admission = cache
            .get::<RepoModelAdmissionReceipt>(&decision.model_admission_receipt_id)?
            .expect("Adopt admission receipt");
        assert_eq!(admission.frontier_plan_decision_id, decision.decision_id);
        assert_eq!(admission.admitted_hash, memory_graph_model_hash(&current)?);
        assert_eq!(
            commit_repo_frontier_plan_decision(&store, &mind_result.result_id)?,
            decision
        );
        Ok(())
    }

    #[test]
    fn mind_refuse_and_hold_are_terminal_inert_receipts() -> Result<()> {
        let root = tempfile::tempdir()?;
        for (suffix, terminal) in [
            ("refuse", RepoFrontierPlanDecision::Refuse),
            ("hold", RepoFrontierPlanDecision::Hold),
        ] {
            let (store, state, launch, planning) =
                frontier_planning_launch_fixture(root.path(), suffix)?;
            let job_id = format!("backend-{suffix}");
            let plan = plan_coordinator_job_launch(
                &state,
                &launch,
                &store,
                format!("launcher-{suffix}"),
                job_id.clone(),
            )?;
            commit_coordinator_job_launch(
                &store,
                &planning.thread_id,
                &state,
                &launch,
                &plan,
                "2026-07-15T09:00:03Z".into(),
            )?;
            let result = frontier_planning_result(&planning, &job_id, "2026-07-15T09:00:04Z")?;
            put_runtime_role_worker_result(&store, &result)?;
            let mind_result = launch_frontier_mind_result(&store, &result, terminal, suffix)?;
            let before = std::fs::read(&store)?;
            let before_model = runtime_current_repo_model(&store)?.unwrap();
            let receipt = commit_repo_frontier_plan_decision(&store, &mind_result.result_id)?;
            assert!(receipt.model_admission_receipt_id.is_empty());
            assert_eq!(runtime_current_repo_model(&store)?.unwrap(), before_model);
            assert!(select_and_commit_repo_frontier_route(&store, "2026-07-15T09:00:06Z").is_err());
            assert!(std::fs::read(&store)?.len() > before.len());
            assert!(
                select_and_commit_repo_frontier_planning_request(&store, "2026-07-15T09:00:07Z")
                    .is_err()
            );
        }
        Ok(())
    }

    #[test]
    fn concurrent_mind_admission_retries_converge_on_one_terminal_receipt() -> Result<()> {
        let root = tempfile::tempdir()?;
        let (store, state, launch, planning) =
            frontier_planning_launch_fixture(root.path(), "mind-race")?;
        let plan = plan_coordinator_job_launch(
            &state,
            &launch,
            &store,
            "launcher-mind-race".into(),
            "backend-mind-race".into(),
        )?;
        commit_coordinator_job_launch(
            &store,
            &planning.thread_id,
            &state,
            &launch,
            &plan,
            "2026-07-15T09:00:03Z".into(),
        )?;
        let result =
            frontier_planning_result(&planning, "backend-mind-race", "2026-07-15T09:00:04Z")?;
        put_runtime_role_worker_result(&store, &result)?;
        let mind_result =
            launch_frontier_mind_result(&store, &result, RepoFrontierPlanDecision::Adopt, "race")?;
        let left_store = store.clone();
        let right_store = store.clone();
        let left_id = mind_result.result_id.clone();
        let right_id = mind_result.result_id.clone();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let left_barrier = barrier.clone();
        let right_barrier = barrier.clone();
        let left = std::thread::spawn(move || {
            crate::runtime_spine::commit_repo_frontier_plan_decision_with_pre_cas(
                left_store,
                &left_id,
                &move || {
                    left_barrier.wait();
                },
            )
        });
        let right = std::thread::spawn(move || {
            crate::runtime_spine::commit_repo_frontier_plan_decision_with_pre_cas(
                right_store,
                &right_id,
                &move || {
                    right_barrier.wait();
                },
            )
        });
        let outcomes = [left.join().unwrap(), right.join().unwrap()];
        assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 2);
        let mut cache = coordinator_acceptance_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let decisions = cache
            .get_all::<RepoFrontierPlanDecisionReceipt>()?
            .into_iter()
            .filter(|receipt| receipt.planning_request_id == planning.request_id)
            .collect::<Vec<_>>();
        assert_eq!(decisions.len(), 1);
        let current = runtime_current_repo_model(&store)?.unwrap();
        if decisions[0].decision == RepoFrontierPlanDecision::Adopt {
            assert_eq!(current.model_revision, planning.model_revision + 1);
            assert!(!decisions[0].model_admission_receipt_id.is_empty());
        } else {
            assert_eq!(current.model_revision, planning.model_revision);
            assert!(decisions[0].model_admission_receipt_id.is_empty());
        }
        Ok(())
    }

    #[test]
    fn immutable_mind_result_rejects_causal_substitution_and_foreign_cargo() -> Result<()> {
        let root = tempfile::tempdir()?;
        for mutation in 0..7 {
            let suffix = format!("mind-hostile-{mutation}");
            let (store, state, launch, planning) =
                frontier_planning_launch_fixture(root.path(), &suffix)?;
            let imagination_job = format!("backend-imagination-{suffix}");
            let plan = plan_coordinator_job_launch(
                &state,
                &launch,
                &store,
                format!("launcher-imagination-{suffix}"),
                imagination_job.clone(),
            )?;
            commit_coordinator_job_launch(
                &store,
                &planning.thread_id,
                &state,
                &launch,
                &plan,
                "2026-07-15T09:00:03Z".into(),
            )?;
            let imagination =
                frontier_planning_result(&planning, &imagination_job, "2026-07-15T09:00:04Z")?;
            put_runtime_role_worker_result(&store, &imagination)?;
            let mut hostile = launch_frontier_mind_result(
                &store,
                &imagination,
                RepoFrontierPlanDecision::Adopt,
                &suffix,
            )?;
            match mutation {
                0 => hostile.frontier_plan_mind_request_id = Some("swapped-request".into()),
                1 => hostile.job_id = "swapped-job".into(),
                2 => hostile.role_id = "imagination".into(),
                3 => hostile.state_patch_msgpack = Some(vec![0]),
                4 => hostile.repo_model_patch_msgpack = Some(vec![0]),
                5 => {
                    let mut payload = hostile.frontier_plan_mind_decision()?.unwrap();
                    payload.candidate_sha256 = "swapped-candidate-hash".into();
                    hostile.frontier_plan_mind_decision_msgpack =
                        Some(rmp_serde::to_vec_named(&payload)?);
                }
                _ => hostile.item_error = Some("Mind failed after emitting judgment".into()),
            }
            let before = std::fs::read(&store)?;
            assert!(put_runtime_role_worker_result(&store, &hostile).is_err());
            assert_eq!(std::fs::read(&store)?, before);
        }
        Ok(())
    }

    #[test]
    fn concurrent_frontier_planning_launches_leave_only_winner_artifacts() -> Result<()> {
        let root = tempfile::tempdir()?;
        let (store, state, launch, planning) =
            frontier_planning_launch_fixture(root.path(), "race")?;
        let left_plan = plan_coordinator_job_launch(
            &state,
            &launch,
            &store,
            "launcher-left".into(),
            "backend-left".into(),
        )?;
        let right_plan = plan_coordinator_job_launch(
            &state,
            &launch,
            &store,
            "launcher-right".into(),
            "backend-right".into(),
        )?;
        let spawn = |store: std::path::PathBuf,
                     state: EpiphanyThreadState,
                     launch: EpiphanyJobLaunchRequest,
                     plan: EpiphanyCoordinatorJobLaunchPlan,
                     thread: String,
                     at: &'static str| {
            std::thread::spawn(move || {
                commit_coordinator_job_launch(&store, &thread, &state, &launch, &plan, at.into())
            })
        };
        let left = spawn(
            store.clone(),
            state.clone(),
            launch.clone(),
            left_plan,
            planning.thread_id.clone(),
            "2026-07-15T09:00:03Z",
        );
        let right = spawn(
            store.clone(),
            state,
            launch,
            right_plan,
            planning.thread_id,
            "2026-07-15T09:00:04Z",
        );
        let outcomes = [left.join().unwrap(), right.join().unwrap()];
        assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
        let mut cache = coordinator_acceptance_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let bindings = cache
            .get_all::<RepoFrontierPlanningLaunchBinding>()?
            .into_iter()
            .filter(|binding| binding.planning_request_id == planning.request_id)
            .collect::<Vec<_>>();
        assert_eq!(bindings.len(), 1);
        let winner = bindings[0].job_id.as_str();
        let loser = if winner == "backend-left" {
            "backend-right"
        } else {
            assert_eq!(winner, "backend-right");
            "backend-left"
        };
        for job_id in [winner] {
            assert!(cache.get::<EpiphanyRuntimeJob>(job_id)?.is_some());
            assert!(
                cache
                    .get::<EpiphanyRuntimeWorkerLaunchRequest>(job_id)?
                    .is_some()
            );
            assert!(
                cache
                    .get::<EpiphanyRuntimeEvent>(&format!("event-job-opened-{job_id}"))?
                    .is_some()
            );
        }
        assert!(cache.get::<EpiphanyRuntimeJob>(loser)?.is_none());
        assert!(
            cache
                .get::<EpiphanyRuntimeWorkerLaunchRequest>(loser)?
                .is_none()
        );
        assert!(
            cache
                .get::<EpiphanyRuntimeEvent>(&format!("event-job-opened-{loser}"))?
                .is_none()
        );
        Ok(())
    }

    #[test]
    fn claim_repair_launch_refuses_every_swapped_causal_field_without_new_writes() -> Result<()> {
        let root = tempfile::tempdir()?;
        for mutation in 0..8 {
            let (store, state, launch, repair) =
                claim_repair_launch_fixture(root.path(), &format!("causal-{mutation}"))?;
            let mut corrupt = repair.clone();
            match mutation {
                0 => corrupt.model_hash = "swapped-model-hash".into(),
                1 => corrupt.eyes_evidence_packet_id = "swapped-packet".into(),
                2 => corrupt.eyes_evidence_packet_sha256 = "swapped-packet-hash".into(),
                3 => corrupt.current_admission_receipt_id = "swapped-admission".into(),
                4 => corrupt.target_claim_sha256 = "swapped-claim-hash".into(),
                5 => {
                    corrupt.affected_frontier[0].frontier_item_sha256 =
                        "swapped-frontier-hash".into()
                }
                6 => corrupt.runtime_id = "swapped-runtime".into(),
                _ => corrupt.thread_id = "swapped-thread".into(),
            }
            let mut cache = coordinator_acceptance_cache(&store)?;
            cache.put(&corrupt.request_id, &corrupt)?;
            let before = std::fs::read(&store)?;
            assert!(
                plan_coordinator_job_launch(
                    &state,
                    &launch,
                    &store,
                    format!("launcher-causal-{mutation}"),
                    format!("backend-causal-{mutation}"),
                )
                .is_err(),
                "causal mutation {mutation} must be refused"
            );
            assert_eq!(std::fs::read(&store)?, before);
        }
        Ok(())
    }

    #[test]
    fn concurrent_claim_repair_launches_contend_on_one_request_key() -> Result<()> {
        let root = tempfile::tempdir()?;
        let (store, state, launch, repair) = claim_repair_launch_fixture(root.path(), "race")?;
        let left_plan = plan_coordinator_job_launch(
            &state,
            &launch,
            &store,
            "launcher-left".into(),
            "backend-left".into(),
        )?;
        let right_plan = plan_coordinator_job_launch(
            &state,
            &launch,
            &store,
            "launcher-right".into(),
            "backend-right".into(),
        )?;
        let left_store = store.clone();
        let right_store = store.clone();
        let left_state = state.clone();
        let right_state = state.clone();
        let left_launch = launch.clone();
        let right_launch = launch.clone();
        let left_thread = repair.thread_id.clone();
        let right_thread = repair.thread_id.clone();
        let left = std::thread::spawn(move || {
            commit_coordinator_job_launch(
                &left_store,
                &left_thread,
                &left_state,
                &left_launch,
                &left_plan,
                "2026-07-14T09:00:03Z".into(),
            )
        });
        let right = std::thread::spawn(move || {
            commit_coordinator_job_launch(
                &right_store,
                &right_thread,
                &right_state,
                &right_launch,
                &right_plan,
                "2026-07-14T09:00:04Z".into(),
            )
        });
        let outcomes = [left.join().unwrap(), right.join().unwrap()];
        assert_eq!(outcomes.iter().filter(|outcome| outcome.is_ok()).count(), 1);
        let mut cache = coordinator_acceptance_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let bindings = cache
            .get_all::<RepoModelClaimRepairLaunchBinding>()?
            .into_iter()
            .filter(|binding| binding.repair_request_id == repair.request_id)
            .collect::<Vec<_>>();
        assert_eq!(bindings.len(), 1);
        assert_eq!(
            bindings[0].binding_record_id,
            format!("repo-model-claim-repair-launch-{}", repair.request_id)
        );
        let winner = bindings[0].job_id.as_str();
        let loser = if winner == "backend-left" {
            "backend-right"
        } else {
            assert_eq!(winner, "backend-right");
            "backend-left"
        };
        assert!(cache.get::<EpiphanyRuntimeJob>(winner)?.is_some());
        assert!(
            cache
                .get::<EpiphanyRuntimeWorkerLaunchRequest>(winner)?
                .is_some()
        );
        assert!(
            cache
                .get::<EpiphanyRuntimeEvent>(&format!("event-job-opened-{winner}"))?
                .is_some()
        );
        assert!(cache.get::<EpiphanyRuntimeJob>(loser)?.is_none());
        assert!(
            cache
                .get::<EpiphanyRuntimeWorkerLaunchRequest>(loser)?
                .is_none()
        );
        assert!(
            cache
                .get::<EpiphanyRuntimeEvent>(&format!("event-job-opened-{loser}"))?
                .is_none()
        );
        let persisted_state = cache
            .get_required::<EpiphanyThreadStateEntry>(THREAD_STATE_KEY)?
            .state()?;
        assert!(
            persisted_state
                .runtime_links
                .iter()
                .any(|link| link.runtime_job_id == winner)
        );
        assert!(
            persisted_state
                .runtime_links
                .iter()
                .all(|link| link.runtime_job_id != loser)
        );
        Ok(())
    }
}
