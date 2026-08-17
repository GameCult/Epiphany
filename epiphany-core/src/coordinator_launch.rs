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
    if request.proposal_modeling_request_id.is_some() {
        return Err(anyhow!(
            "proposal Modeling launch is owned by keyed current work, not the aggregate coordinator transaction"
        ));
    }
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
    if let Some(admitted_thread) =
        cache.get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
    {
        if admitted_thread.thread_id != thread_id || admitted_thread.state()? != *current_state {
            return Err(anyhow!(
                "coordinator launch carrier must use the current admitted thread"
            ));
        }
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
    let frontier_planning_launch =
        if let Some(request_id) = request.frontier_planning_request_id.as_deref() {
            let (planning, identity, attempt_ordinal, superseded_failure_result_id) =
                validate_frontier_planning_launch(cache, current_state, request, request_id)?;
            let projection = RepoFrontierPlanningContextProjection::from_request(&planning);
            match &mut effective_launch_document {
                EpiphanyWorkerLaunchDocument::Role(document) => {
                    if document.proposal_modeling_context.is_some() {
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
            Some((
                planning,
                identity,
                format!("{:x}", Sha256::digest(bytes)),
                attempt_ordinal,
                superseded_failure_result_id,
            ))
        } else {
            None
        };
    let imagination_consideration_launch = if let Some(request_id) =
        request.imagination_consideration_request_id.as_deref()
    {
        let (consideration, identity, attempt_ordinal) =
            validate_imagination_consideration_launch(cache, current_state, request, request_id)?;
        let model = crate::repo_model_documents::assemble_repo_model_view_from_cache(cache)?;
        crate::EpiphanyRepoModelBasis {
            projection_digest: consideration.model_projection_digest.clone(),
            source_documents: consideration.model_source_documents.clone(),
        }
        .validate_against_cache(cache)?;
        let projection = ImaginationConsiderationContextProjection::new(&consideration, &model);
        match &mut effective_launch_document {
            EpiphanyWorkerLaunchDocument::Role(document) => {
                if document.proposal_modeling_context.is_some()
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
        Some((consideration, identity, hash, attempt_ordinal))
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
        let model = crate::repo_model_documents::assemble_repo_model_view_from_cache(cache)?;
        crate::EpiphanyRepoModelBasis {
            projection_digest: consideration.model_projection_digest.clone(),
            source_documents: consideration.model_source_documents.clone(),
        }
        .validate_against_cache(cache)?;
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
            let (
                mind_request,
                planning,
                candidate,
                identity,
                attempt_ordinal,
                superseded_failure_result_id,
            ) = validate_frontier_plan_mind_launch(cache, current_state, request, request_id)?;
            let projection =
                RepoFrontierPlanMindContextProjection::new(&mind_request, &planning, &candidate);
            match &mut effective_launch_document {
                EpiphanyWorkerLaunchDocument::Role(document) => {
                    if document.proposal_modeling_context.is_some()
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
            Some((
                mind_request,
                identity,
                hash,
                attempt_ordinal,
                superseded_failure_result_id,
            ))
        } else {
            None
        };
    let prepared = prepare_runtime_spine_heartbeat_job(
        &cache,
        RuntimeSpineHeartbeatJobOptions {
            runtime_id: runtime_identity.runtime_id,
            display_name: "Epiphany Local".to_string(),
            session_id: crate::runtime_spine::EPIPHANY_RUNTIME_ROOT_SESSION_ID.to_string(),
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
            frontier_planning_request_id: request.frontier_planning_request_id.clone(),
            frontier_plan_mind_request_id: request.frontier_plan_mind_request_id.clone(),
            imagination_consideration_request_id: request
                .imagination_consideration_request_id
                .clone(),
            admitted_model_direction_consideration_request_id: request
                .admitted_model_direction_consideration_request_id
                .clone(),
            repo_frontier_modeling_request_id: request.repo_frontier_modeling_request_id.clone(),
            repo_frontier_research_request_id: request.repo_frontier_research_request_id.clone(),
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
    if let Some((
        planning,
        identity,
        worker_launch_document_sha256,
        attempt_ordinal,
        superseded_failure_result_id,
    )) = frontier_planning_launch
    {
        let binding_record_id = if attempt_ordinal == 0 {
            format!("repo-frontier-planning-launch-{}", planning.request_id)
        } else {
            format!(
                "repo-frontier-planning-launch-{}-attempt-{attempt_ordinal}",
                planning.request_id
            )
        };
        let launch_binding = RepoFrontierPlanningLaunchBinding {
            schema_version: REPO_FRONTIER_PLANNING_LAUNCH_BINDING_SCHEMA_VERSION.into(),
            binding_record_id,
            planning_request_id: planning.request_id,
            job_id: plan.backend_job_id.clone(),
            binding_id: request.binding_id.clone(),
            runtime_id: identity.runtime_id,
            thread_id: planning.thread_id,
            launched_at: created_at.clone(),
            worker_launch_document_sha256,
            contract: REPO_FRONTIER_PLANNING_LAUNCH_BINDING_CONTRACT.into(),
            attempt_ordinal,
            superseded_failure_result_id,
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
    if let Some((consideration, identity, worker_launch_document_sha256, attempt_ordinal)) =
        imagination_consideration_launch
    {
        let binding_record_id = if attempt_ordinal == 0 {
            format!(
                "imagination-consideration-launch-{}",
                consideration.request_id
            )
        } else {
            format!(
                "imagination-consideration-launch-{}-attempt-{attempt_ordinal}",
                consideration.request_id
            )
        };
        let binding = ImaginationConsiderationLaunchBinding {
            schema_version: IMAGINATION_CONSIDERATION_LAUNCH_BINDING_SCHEMA_VERSION.into(),
            binding_record_id,
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
    if let Some((
        mind_request,
        identity,
        worker_launch_document_sha256,
        attempt_ordinal,
        superseded_failure_result_id,
    )) = frontier_plan_mind_launch
    {
        let binding_record_id = if attempt_ordinal == 0 {
            format!("repo-frontier-plan-mind-launch-{}", mind_request.request_id)
        } else {
            format!(
                "repo-frontier-plan-mind-launch-{}-attempt-{attempt_ordinal}",
                mind_request.request_id
            )
        };
        let binding = RepoFrontierPlanMindLaunchBinding {
            schema_version: REPO_FRONTIER_PLAN_MIND_LAUNCH_BINDING_SCHEMA_VERSION.into(),
            binding_record_id,
            mind_request_id: mind_request.request_id,
            job_id: plan.backend_job_id.clone(),
            binding_id: request.binding_id.clone(),
            runtime_id: identity.runtime_id,
            thread_id: mind_request.thread_id,
            launched_at: created_at.clone(),
            worker_launch_document_sha256,
            contract: REPO_FRONTIER_PLAN_MIND_LAUNCH_BINDING_CONTRACT.into(),
            attempt_ordinal,
            superseded_failure_result_id,
        };
        if cache
            .get::<RepoFrontierPlanMindLaunchBinding>(&binding.binding_record_id)?
            .is_some()
        {
            return Err(anyhow!("Mind request is already bound to a launch"));
        }
        batch.push(cache.prepare_entry(&binding.binding_record_id, &binding)?.0);
    }
    if matches!(
        request.binding_id.as_str(),
        EPIPHANY_RESEARCH_ROLE_BINDING_ID
            | crate::EPIPHANY_MODELING_ROLE_BINDING_ID
            | crate::EPIPHANY_VERIFICATION_ROLE_BINDING_ID
    ) {
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
    if request.proposal_modeling_request_id.is_some() {
        return Err(anyhow!(
            "proposal Modeling launch is owned by keyed current work, not the aggregate coordinator transaction"
        ));
    }
    let (
        caller_proposal_projection,
        caller_planning_projection,
        caller_mind_projection,
        caller_body_basis,
    ) = match &request.launch_document {
        EpiphanyWorkerLaunchDocument::Role(document) => (
            document.proposal_modeling_context.as_ref(),
            document.frontier_planning_context.as_ref(),
            document.frontier_plan_mind_context.as_ref(),
            document.repository_body_observation_basis.as_ref(),
        ),
        EpiphanyWorkerLaunchDocument::Reorient(_) => (None, None, None, None),
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
    if let Some(request_id) = request.frontier_planning_request_id.as_deref() {
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
    u64,
    Option<String>,
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
        || request.thread_id.is_empty()
        || launch.launch_document.thread_id() != persisted.thread_id
    {
        return Err(anyhow!("Mind launch provenance mismatch"));
    }
    let persisted_state = persisted.state()?;
    let mut launches = cache
        .get_all::<RepoFrontierPlanMindLaunchBinding>()?
        .into_iter()
        .filter(|binding| binding.mind_request_id == request_id)
        .collect::<Vec<_>>();
    launches.sort_by_key(|binding| binding.attempt_ordinal);
    for (expected, binding) in launches.iter().enumerate() {
        if binding.attempt_ordinal != expected as u64 {
            return Err(anyhow!("Mind planning attempts must be contiguous"));
        }
    }
    let superseded_failure_result_id = if let Some(latest) = launches.last() {
        let results = cache
            .get_all::<EpiphanyRuntimeRoleWorkerResult>()?
            .into_iter()
            .filter(|result| result.job_id == latest.job_id)
            .collect::<Vec<_>>();
        if results.len() != 1 {
            return Err(anyhow!("Mind retry requires one typed prior result"));
        }
        let failure = &results[0];
        if !failure.role_id.eq_ignore_ascii_case("mindAdmissionReview")
            || failure.frontier_plan_mind_request_id.is_some()
            || failure.frontier_plan_mind_decision_msgpack.is_some()
            || failure.item_error.as_deref().is_none_or(str::is_empty)
        {
            return Err(anyhow!(
                "Mind retry requires a non-executable typed failure"
            ));
        }
        let reviewed = persisted_state
            .acceptance_receipts
            .iter()
            .filter(|receipt| {
                receipt.result_id == failure.result_id
                    && receipt.job_id == failure.job_id
                    && receipt.binding_id == EPIPHANY_MIND_ROLE_BINDING_ID
                    && receipt.surface == "roleFailureReview"
                    && receipt.role_id == "mindAdmissionReview"
                    && receipt.status == "superseded"
            })
            .count();
        if reviewed != 1 {
            return Err(anyhow!("Mind retry requires one explicit failure review"));
        }
        Some(failure.result_id.clone())
    } else {
        None
    };
    let attempt_ordinal = u64::try_from(launches.len())
        .map_err(|_| anyhow!("Mind planning attempt ordinal overflow"))?;
    Ok((
        request,
        planning,
        candidate,
        identity,
        attempt_ordinal,
        superseded_failure_result_id,
    ))
}

fn validate_frontier_planning_launch(
    cache: &CultCache,
    state: &EpiphanyThreadState,
    launch: &EpiphanyJobLaunchRequest,
    request_id: &str,
) -> Result<(
    RepoFrontierPlanningRequest,
    EpiphanyRuntimeIdentity,
    u64,
    Option<String>,
)> {
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
    crate::runtime_spine::validate_actionable_repo_frontier_planning_request(cache, &planning)?;
    let identity = cache
        .get::<EpiphanyRuntimeIdentity>(RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("frontier planning launch requires runtime identity"))?;
    let persisted_state = cache
        .get::<crate::EpiphanyThreadStateEntry>(crate::THREAD_STATE_KEY)?
        .ok_or_else(|| anyhow!("frontier planning launch requires authoritative thread state"))?;
    let persisted_state_value = persisted_state.state()?;
    if planning.request_id != request_id
        || planning.runtime_id != identity.runtime_id
        || persisted_state_value != *state
        || launch.launch_document.thread_id() != persisted_state.thread_id
    {
        return Err(anyhow!(
            "frontier planning launch provenance binding mismatch"
        ));
    }
    let mut launches = cache
        .get_all::<RepoFrontierPlanningLaunchBinding>()?
        .into_iter()
        .filter(|binding| binding.planning_request_id == request_id)
        .collect::<Vec<_>>();
    launches.sort_by_key(|binding| binding.attempt_ordinal);
    for (expected, binding) in launches.iter().enumerate() {
        if binding.attempt_ordinal != expected as u64 {
            return Err(anyhow!(
                "frontier planning attempts must be contiguous and monotonic"
            ));
        }
    }
    let superseded_failure_result_id = if let Some(latest) = launches.last() {
        let results = cache
            .get_all::<EpiphanyRuntimeRoleWorkerResult>()?
            .into_iter()
            .filter(|result| result.job_id == latest.job_id)
            .collect::<Vec<_>>();
        if results.len() != 1 {
            return Err(anyhow!(
                "frontier planning retry requires exactly one typed prior attempt result"
            ));
        }
        let failure = &results[0];
        if !failure.role_id.eq_ignore_ascii_case("imagination")
            || failure.frontier_planning_request_id.is_some()
            || failure.frontier_plan_candidate_msgpack.is_some()
            || failure.item_error.as_deref().is_none_or(str::is_empty)
        {
            return Err(anyhow!(
                "frontier planning retry requires a non-executable typed prior failure"
            ));
        }
        let reviewed = persisted_state_value
            .acceptance_receipts
            .iter()
            .filter(|receipt| {
                receipt.result_id == failure.result_id
                    && receipt.job_id == failure.job_id
                    && receipt.binding_id == EPIPHANY_IMAGINATION_ROLE_BINDING_ID
                    && receipt.surface == "roleFailureReview"
                    && receipt.role_id == "imagination"
                    && receipt.status == "superseded"
            })
            .collect::<Vec<_>>();
        if reviewed.len() != 1 {
            return Err(anyhow!(
                "frontier planning retry requires exactly one explicit failure review"
            ));
        }
        Some(failure.result_id.clone())
    } else {
        None
    };
    let attempt_ordinal = u64::try_from(launches.len())
        .map_err(|_| anyhow!("frontier planning attempt ordinal overflow"))?;
    Ok((
        planning,
        identity,
        attempt_ordinal,
        superseded_failure_result_id,
    ))
}

fn validate_imagination_consideration_launch(
    cache: &CultCache,
    state: &EpiphanyThreadState,
    launch: &EpiphanyJobLaunchRequest,
    request_id: &str,
) -> Result<(
    ImaginationConsiderationRequest,
    EpiphanyRuntimeIdentity,
    u64,
)> {
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
        || request.thread_id.is_empty()
        || persisted.state()? != *state
        || launch.launch_document.thread_id() != persisted.thread_id
    {
        return Err(anyhow!("consideration launch provenance mismatch"));
    }
    let prior_bindings = cache
        .get_all::<ImaginationConsiderationLaunchBinding>()?
        .iter()
        .filter(|binding| binding.request_id == request_id)
        .cloned()
        .collect::<Vec<_>>();
    for binding in prior_bindings {
        if !worker_attempt_is_retryable_terminal_failure(cache, &binding.job_id)? {
            return Err(anyhow!("consideration request already bound"));
        }
    }
    let attempt_ordinal = u64::try_from(
        cache
            .get_all::<ImaginationConsiderationLaunchBinding>()?
            .iter()
            .filter(|binding| binding.request_id == request_id)
            .count(),
    )
    .map_err(|_| anyhow!("consideration attempt ordinal overflow"))?;
    Ok((request, identity, attempt_ordinal))
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
        || request.thread_id.is_empty()
        || persisted.state()? != *state
        || launch.launch_document.thread_id() != persisted.thread_id
    {
        return Err(anyhow!("model direction consideration provenance mismatch"));
    }
    let prior_launches = cache
        .get_all::<EpiphanyRuntimeWorkerLaunchRequest>()?
        .iter()
        .filter(|worker| {
            worker
                .admitted_model_direction_consideration_request_id
                .as_deref()
                == Some(request_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    for worker in prior_launches {
        if !worker_attempt_is_retryable_terminal_failure(cache, &worker.job_id)? {
            return Err(anyhow!("model direction consideration already bound"));
        }
    }
    Ok(request)
}

fn worker_attempt_is_retryable_terminal_failure(cache: &CultCache, job_id: &str) -> Result<bool> {
    cache
        .get::<crate::EpiphanyRuntimeWorkerProcessClaim>(&format!(
            "runtime-worker-process-{job_id}"
        ))?
        .map(|claim| Ok(crate::WorkerProcessStatus::parse(&claim.status)?.allows_retry()))
        .unwrap_or(Ok(false))
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
    if let Some(snapshot) = runtime_job_snapshot(runtime_store, &link.runtime_job_id)?
        && let Some(result) = snapshot.result
    {
        let mut terminal = link.clone();
        terminal.id = format!("{}-{}", link.id, result.result_id);
        terminal.surface = "runtimeResult".to_string();
        terminal.runtime_result_id = Some(result.result_id);
        return Ok(Some(terminal));
    }
    let Some(claim) = crate::runtime_worker_process_claim(runtime_store, &link.runtime_job_id)?
    else {
        return Ok(None);
    };
    if !crate::WorkerProcessStatus::parse(&claim.status)?.is_failed_terminal() {
        return Ok(None);
    }
    let terminal_authority_id = claim
        .terminal_authority_id
        .ok_or_else(|| anyhow!("terminal worker process claim lost its authority id"))?;
    let mut terminal = link.clone();
    terminal.id = format!("{}-{}", link.id, terminal_authority_id);
    terminal.surface = "runtimeWorkerProcessTerminal".to_string();
    terminal.runtime_result_id = Some(terminal_authority_id);
    Ok(Some(terminal))
}
