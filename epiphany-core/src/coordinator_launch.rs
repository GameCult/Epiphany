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
    cache: &mut coordinator_state_transaction::CoordinatorStateTransaction,
    thread_id: &str,
    current_state: &EpiphanyThreadState,
    request: &EpiphanyJobLaunchRequest,
    plan: &EpiphanyCoordinatorJobLaunchPlan,
    created_at: String,
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
            claim_repair_request_id: request.claim_repair_request_id.clone(),
            frontier_planning_request_id: request.frontier_planning_request_id.clone(),
            created_at: created_at.clone(),
        },
    )?;
    let mut batch = prepared.envelopes;
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
    let (caller_proposal_projection, caller_repair_projection, caller_planning_projection) =
        match &request.launch_document {
            EpiphanyWorkerLaunchDocument::Role(document) => (
                document.proposal_modeling_context.as_ref(),
                document.claim_repair_context.as_ref(),
                document.frontier_planning_context.as_ref(),
            ),
            EpiphanyWorkerLaunchDocument::Reorient(_) => (None, None, None),
        };
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
    if [
        request.proposal_modeling_request_id.is_some(),
        request.claim_repair_request_id.is_some(),
        request.frontier_planning_request_id.is_some(),
    ]
    .into_iter()
    .filter(|present| *present)
    .count()
        > 1
    {
        return Err(anyhow!(
            "proposal Modeling, claim repair, and frontier planning launches are mutually exclusive"
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
