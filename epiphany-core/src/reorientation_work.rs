use anyhow::{Result, anyhow};
use cultcache_rs::{CultCache, DatabaseEntry};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;

use crate::{
    EpiphanyAgentPassContinuationAction, EpiphanyMindDocumentVersion, EpiphanyRuntimeJobStatus,
};

pub const REORIENTATION_REQUEST_SCHEMA_VERSION: &str = "epiphany.self.reorientation_request.v1";
pub const MIND_REORIENTATION_DECISION_SCHEMA_VERSION: &str =
    "epiphany.mind.reorientation_decision.v1";
pub const MIND_REORIENTATION_PASS_FAILURE_SCHEMA_VERSION: &str =
    "epiphany.mind.reorientation_pass_failure.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyReorientationStateProjection {
    pub runtime_id: String,
    pub objective: Option<String>,
    pub active_subgoal_id: Option<String>,
    pub investigation_checkpoint: Option<epiphany_state_model::EpiphanyInvestigationCheckpoint>,
    pub mode: Option<epiphany_state_model::EpiphanyModeState>,
    pub repository_body_observation: Option<crate::RepositoryBodyObservationBasis>,
    pub frontier: Vec<epiphany_state_model::RepoFrontierItem>,
    pub projection_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.self.reorientation_request.v1",
    schema = "EpiphanyReorientationRequest"
)]
pub struct EpiphanyReorientationRequest {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub request_id: String,
    #[cultcache(key = 2)]
    pub runtime_id: String,
    #[cultcache(key = 3)]
    pub projection: EpiphanyReorientationStateProjection,
    #[cultcache(key = 4)]
    pub source_documents: Vec<EpiphanyMindDocumentVersion>,
    #[cultcache(key = 5)]
    pub requested_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.reorientation_decision.v1",
    schema = "EpiphanyMindReorientationDecisionDocument"
)]
pub struct EpiphanyMindReorientationDecisionDocument {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub decision_id: String,
    #[cultcache(key = 2)]
    pub request_id: String,
    #[cultcache(key = 3)]
    pub job_id: String,
    #[cultcache(key = 4)]
    pub result_id: String,
    #[cultcache(key = 5)]
    pub decision_context_id: String,
    #[cultcache(key = 6)]
    pub mode: String,
    #[cultcache(key = 7)]
    pub checkpoint_still_valid: Option<bool>,
    #[cultcache(key = 8)]
    pub summary: String,
    #[cultcache(key = 9)]
    pub next_safe_move: String,
    #[cultcache(key = 10)]
    pub decided_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.reorientation_pass_failure.v1",
    schema = "EpiphanyMindReorientationPassFailureDocument"
)]
pub struct EpiphanyMindReorientationPassFailureDocument {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub failure_id: String,
    #[cultcache(key = 2)]
    pub request_id: String,
    #[cultcache(key = 3)]
    pub job_id: String,
    #[cultcache(key = 4)]
    pub runtime_result_id: String,
    #[cultcache(key = 5)]
    pub decision_context_id: String,
    #[cultcache(key = 6)]
    pub verdict: String,
    #[cultcache(key = 7)]
    pub summary: String,
    #[cultcache(key = 8)]
    pub failed_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyReorientationWorkProjection {
    pub request: EpiphanyReorientationRequest,
    #[serde(flatten)]
    pub attempt: crate::EpiphanyAgentPassAttemptProjection,
}

pub fn request_current_reorientation(
    store_path: impl AsRef<Path>,
    requested_at: &str,
) -> Result<EpiphanyReorientationRequest> {
    chrono::DateTime::parse_from_rfc3339(requested_at)
        .map_err(|_| anyhow!("reorientation request time is invalid"))?;
    let store_path = store_path.as_ref();
    let mind = crate::assemble_mind_view(store_path)?;
    let mut source_documents = mind
        .source_documents
        .iter()
        .filter(|source| {
            matches!(
                source.document_type.as_str(),
                crate::EpiphanyMindIdentity::TYPE
                    | crate::EpiphanyMindObjectiveDocument::TYPE
                    | crate::EpiphanyMindFocusDocument::TYPE
                    | crate::EpiphanyMindModeDocument::TYPE
                    | crate::EpiphanyMindInvestigationCheckpointDocument::TYPE
                    | crate::EpiphanyMindRepositoryBodyObservationDocument::TYPE
                    | crate::EpiphanyMindReorientationDecisionDocument::TYPE
                    | crate::EpiphanyRepoModelIdentityDocument::TYPE
                    | crate::EpiphanyRepoModelFrontierDocument::TYPE
            )
        })
        .cloned()
        .collect::<Vec<_>>();
    source_documents.sort_by(|left, right| {
        (&left.document_type, &left.document_key).cmp(&(&right.document_type, &right.document_key))
    });
    let projection_digest = crate::epiphany_mind_projection_digest(&source_documents)?;
    let projection = EpiphanyReorientationStateProjection {
        runtime_id: mind.runtime_id.clone(),
        objective: mind.objective,
        active_subgoal_id: mind.active_subgoal_id,
        investigation_checkpoint: mind.investigation_checkpoint,
        mode: mind.mode,
        repository_body_observation: mind.repository_body_observation,
        frontier: mind
            .repo_model
            .map(|model| model.frontier)
            .unwrap_or_default(),
        projection_digest: projection_digest.clone(),
    };
    let request_id = format!(
        "reorientation-request-{:x}",
        Sha256::digest(rmp_serde::to_vec_named(&(
            REORIENTATION_REQUEST_SCHEMA_VERSION,
            mind.runtime_id.as_str(),
            projection_digest.as_str(),
        ))?)
    );
    let request = EpiphanyReorientationRequest {
        schema_version: REORIENTATION_REQUEST_SCHEMA_VERSION.into(),
        request_id: request_id.clone(),
        runtime_id: mind.runtime_id,
        projection,
        source_documents,
        requested_at: requested_at.into(),
    };
    validate_reorientation_request_intrinsic(&request)?;
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    if let Some(existing) = cache.get::<EpiphanyReorientationRequest>(&request_id)? {
        validate_reorientation_request_current(&cache, &existing)?;
        return Ok(existing);
    }
    let expected = exact_source_envelopes(&cache, &request.source_documents)?;
    let mut replacements = expected.clone();
    let write = cache.prepare_entry(&request_id, &request)?.0;
    replacements.push(write);
    if !crate::runtime_store_backend::runtime_spine_backing_store(store_path)?
        .compare_and_swap_batch(&expected, replacements)?
    {
        return request_current_reorientation(store_path, requested_at);
    }
    Ok(request)
}

fn reorientation_attempt_ordinal(request_id: &str, job_id: &str) -> Result<usize> {
    let prefix = format!("reorientation-{request_id}-attempt-");
    job_id
        .strip_prefix(&prefix)
        .ok_or_else(|| anyhow!("reorientation job identity is not canonical"))?
        .parse::<usize>()
        .map_err(|_| anyhow!("reorientation attempt ordinal is invalid"))
}

fn reorientation_launch_document(
    launch: &crate::EpiphanyRuntimeWorkerLaunchRequest,
) -> Result<crate::EpiphanyReorientWorkerLaunchDocument> {
    let document = match launch.launch_document()? {
        crate::EpiphanyWorkerLaunchDocument::Reorient(document) => document,
        crate::EpiphanyWorkerLaunchDocument::Role(_) => {
            return Err(anyhow!("reorientation owner carried a role launch"));
        }
    };
    if launch.schema_version != crate::RUNTIME_WORKER_LAUNCH_REQUEST_SCHEMA_VERSION
        || launch.role != crate::EPIPHANY_REORIENT_OWNER_ROLE
        || launch.binding_id != crate::EPIPHANY_REORIENT_LAUNCH_BINDING_ID
        || document.schema_version != "epiphany.reorientation_launch_projection.v1"
        || document.request_id.trim().is_empty()
        || document.creation_thread_id != launch.job_id
        || launch.proposal_modeling_request_id.is_some()
        || launch.frontier_planning_request_id.is_some()
        || launch.frontier_plan_mind_request_id.is_some()
        || launch.imagination_consideration_request_id.is_some()
        || launch
            .admitted_model_direction_consideration_request_id
            .is_some()
        || launch.repo_frontier_modeling_request_id.is_some()
        || launch.repo_frontier_research_request_id.is_some()
        || launch.repo_frontier_verification_request_id.is_some()
    {
        return Err(anyhow!(
            "reorientation immutable launch crossed family authority"
        ));
    }
    reorientation_attempt_ordinal(&document.request_id, &launch.job_id)?;
    Ok(document)
}

fn reorientation_launches(
    cache: &CultCache,
    request_id: &str,
) -> Result<Vec<(usize, crate::EpiphanyRuntimeWorkerLaunchRequest)>> {
    let mut launches = Vec::new();
    for launch in cache
        .get_all::<crate::EpiphanyRuntimeWorkerLaunchRequest>()?
        .into_iter()
        .filter(|launch| {
            launch.role == crate::EPIPHANY_REORIENT_OWNER_ROLE
                || launch.binding_id == crate::EPIPHANY_REORIENT_LAUNCH_BINDING_ID
        })
    {
        let document = reorientation_launch_document(&launch)?;
        if document.request_id == request_id {
            launches.push((
                reorientation_attempt_ordinal(request_id, &launch.job_id)?,
                launch,
            ));
        }
    }
    launches.sort_by_key(|(ordinal, _)| *ordinal);
    for (expected, (ordinal, _)) in launches.iter().enumerate() {
        if *ordinal != expected {
            return Err(anyhow!(
                "reorientation request has noncontiguous attempt identity"
            ));
        }
    }
    Ok(launches)
}

pub(crate) fn current_reorientation_work(
    cache: &CultCache,
) -> Result<Option<EpiphanyReorientationWorkProjection>> {
    let mut requests = cache.get_all::<EpiphanyReorientationRequest>()?;
    requests.sort_by(|left, right| left.request_id.cmp(&right.request_id));
    let decisions = cache.get_all::<EpiphanyMindReorientationDecisionDocument>()?;
    let failures = cache.get_all::<EpiphanyMindReorientationPassFailureDocument>()?;
    for request in requests.into_iter().rev() {
        if validate_reorientation_request_current(cache, &request).is_err()
            || decisions
                .iter()
                .any(|decision| decision.request_id == request.request_id)
        {
            continue;
        }
        let launches = reorientation_launches(cache, &request.request_id)?;
        let Some((_, launch)) = launches.last() else {
            return Ok(Some(EpiphanyReorientationWorkProjection {
                request,
                attempt: crate::EpiphanyAgentPassAttemptProjection::unattempted(),
            }));
        };
        let job = cache
            .get::<crate::EpiphanyRuntimeJob>(&launch.job_id)?
            .ok_or_else(|| anyhow!("reorientation launch lost its runtime job"))?;
        let result = cache.get::<crate::EpiphanyRuntimeReorientWorkerResult>(&launch.job_id)?;
        let action = match job.status {
            EpiphanyRuntimeJobStatus::Completed if result.is_some() => {
                EpiphanyAgentPassContinuationAction::Review
            }
            EpiphanyRuntimeJobStatus::Failed => {
                let terminal = exact_runtime_job_result(cache, &launch.job_id)?;
                match terminal.decision_context_id.as_deref() {
                    Some(context_id)
                        if failures.iter().any(|failure| {
                            failure.request_id == request.request_id
                                && failure.job_id == launch.job_id
                                && failure.runtime_result_id == terminal.result_id
                                && failure.decision_context_id == context_id
                        }) =>
                    {
                        EpiphanyAgentPassContinuationAction::Launch
                    }
                    Some(_) => EpiphanyAgentPassContinuationAction::Review,
                    None => EpiphanyAgentPassContinuationAction::Launch,
                }
            }
            EpiphanyRuntimeJobStatus::Cancelled => EpiphanyAgentPassContinuationAction::Launch,
            _ => EpiphanyAgentPassContinuationAction::Wait,
        };
        return Ok(Some(EpiphanyReorientationWorkProjection {
            request,
            attempt: crate::EpiphanyAgentPassAttemptProjection::with(
                action,
                Some(launch.job_id.clone()),
            ),
        }));
    }
    Ok(None)
}

pub fn record_reorientation_pass_failure(
    store_path: impl AsRef<Path>,
    job_id: &str,
) -> Result<crate::EpiphanyMindCommitReceipt> {
    let store_path = store_path.as_ref();
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let launch = cache
        .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(job_id)?
        .ok_or_else(|| anyhow!("reorientation failure has no immutable launch"))?;
    let launch_document = reorientation_launch_document(&launch)?;
    let request = cache
        .get::<EpiphanyReorientationRequest>(&launch_document.request_id)?
        .ok_or_else(|| anyhow!("reorientation failure lost its request"))?;
    validate_reorientation_request_intrinsic(&request)?;
    let job = cache
        .get::<crate::EpiphanyRuntimeJob>(job_id)?
        .ok_or_else(|| anyhow!("reorientation failure lost its runtime job"))?;
    if job.status != EpiphanyRuntimeJobStatus::Failed {
        return Err(anyhow!("reorientation pass is not a failed model pass"));
    }
    let terminal = exact_runtime_job_result(&cache, job_id)?;
    let decision_context_id = terminal
        .decision_context_id
        .as_deref()
        .ok_or_else(|| anyhow!("model-backed reorientation failure has no decision context"))?;
    let context = cache
        .get::<crate::EpiphanyDecisionContext>(decision_context_id)?
        .ok_or_else(|| anyhow!("reorientation failure lost its decision context"))?;
    let basis = cache
        .get::<crate::EpiphanyReasoningBasis>(&context.basis_id)?
        .ok_or_else(|| anyhow!("reorientation failure lost its reasoning basis"))?;
    context.validate(&basis)?;
    let model_failure =
        crate::model_pass_failure_for_request(store_path, &context.terminal_request_id)?
            .ok_or_else(|| anyhow!("reorientation failure has no exact model-pass failure"))?;
    if model_failure.decision_context_id != context.context_id {
        return Err(anyhow!(
            "reorientation failure crossed its model-pass failure context"
        ));
    }
    match basis.projection()? {
        crate::EpiphanyReasoningProjection::ReorientLaunch(projection)
            if projection.request == request => {}
        _ => return Err(anyhow!("reorientation failure crossed its sealed request")),
    }
    let failure = EpiphanyMindReorientationPassFailureDocument {
        schema_version: MIND_REORIENTATION_PASS_FAILURE_SCHEMA_VERSION.into(),
        failure_id: format!("reorientation-pass-failure-{}", terminal.result_id),
        request_id: request.request_id.clone(),
        job_id: job_id.into(),
        runtime_result_id: terminal.result_id.clone(),
        decision_context_id: decision_context_id.into(),
        verdict: terminal.verdict.clone(),
        summary: terminal.summary.clone(),
        failed_at: terminal.completed_at.clone(),
    };
    if let Some(existing) =
        cache.get::<EpiphanyMindReorientationPassFailureDocument>(&failure.failure_id)?
    {
        if existing != failure {
            return Err(anyhow!("reorientation failure replay was substituted"));
        }
        return mind_receipt_for_failure_context(&cache, decision_context_id);
    }
    let snapshot = cache.snapshot_envelopes();
    let mut strong_reads = Vec::new();
    for (document_type, document_key) in [
        (
            EpiphanyReorientationRequest::TYPE,
            request.request_id.as_str(),
        ),
        (crate::EpiphanyRuntimeWorkerLaunchRequest::TYPE, job_id),
        (crate::EpiphanyRuntimeJob::TYPE, job_id),
        (
            crate::EpiphanyRuntimeJobResult::TYPE,
            terminal.result_id.as_str(),
        ),
        (crate::EpiphanyDecisionContext::TYPE, decision_context_id),
        (
            crate::EpiphanyModelPassFailure::TYPE,
            model_failure.failure_id.as_str(),
        ),
    ] {
        strong_reads.push(
            snapshot
                .iter()
                .find(|envelope| envelope.r#type == document_type && envelope.key == document_key)
                .cloned()
                .ok_or_else(|| anyhow!("reorientation failure lost a strong source"))?,
        );
    }
    let write =
        crate::mind_documents::prepare_mind_document(&cache, &failure.failure_id, &failure)?;
    match crate::reasoning_context::commit_mind_mutation(
        store_path,
        decision_context_id,
        "Continuity.reorientation_failure",
        strong_reads,
        vec![write],
        &terminal.completed_at,
    )? {
        crate::EpiphanyMindCommitOutcome::Committed(receipt) => Ok(receipt),
        crate::EpiphanyMindCommitOutcome::Conflict {
            document_identities,
        } => Err(anyhow!(
            "reorientation failure record lost exact runtime authority: {document_identities:?}"
        )),
    }
}

pub fn launch_current_reorientation_work(
    store_path: impl AsRef<Path>,
    created_at: &str,
) -> Result<String> {
    chrono::DateTime::parse_from_rfc3339(created_at)
        .map_err(|_| anyhow!("reorientation launch time is invalid"))?;
    let store_path = store_path.as_ref();
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let work = current_reorientation_work(&cache)?
        .ok_or_else(|| anyhow!("Mind has no unresolved reorientation request"))?;
    if work.attempt.action != EpiphanyAgentPassContinuationAction::Launch {
        return Err(anyhow!("reorientation request is not launchable"));
    }
    validate_reorientation_request_current(&cache, &work.request)?;
    let prior_launches = reorientation_launches(&cache, &work.request.request_id)?;
    let attempt_ordinal = prior_launches.len();
    let job_id = format!(
        "reorientation-{}-attempt-{attempt_ordinal}",
        work.request.request_id
    );
    let launch_document = crate::EpiphanyWorkerLaunchDocument::Reorient(
        crate::EpiphanyReorientWorkerLaunchDocument {
            schema_version: "epiphany.reorientation_launch_projection.v1".into(),
            request_id: work.request.request_id.clone(),
            creation_thread_id: job_id.clone(),
        },
    );
    let output_contract_id = launch_document.output_contract_id().to_string();
    let prepared = crate::prepare_runtime_spine_heartbeat_job(
        &cache,
        crate::RuntimeSpineHeartbeatJobOptions {
            runtime_id: work.request.runtime_id.clone(),
            session_id: crate::EPIPHANY_RUNTIME_ROOT_SESSION_ID.into(),
            objective: "Reorient from one exact keyed Mind projection".into(),
            coordinator_note: "Reorientation current-work owner opened this pass.".into(),
            job_id: job_id.clone(),
            role: crate::EPIPHANY_REORIENT_OWNER_ROLE.into(),
            binding_id: crate::EPIPHANY_REORIENT_LAUNCH_BINDING_ID.into(),
            authority_scope: "epiphany.reorient.decision".into(),
            instruction: "Decide resume or regather from only the sealed typed continuity projection. Return the reorientation result contract; do not invent live state.".into(),
            launch_document,
            output_contract_id: output_contract_id.clone(),
            proposal_modeling_request_id: None,
            frontier_planning_request_id: None,
            frontier_plan_mind_request_id: None,
            imagination_consideration_request_id: None,
            admitted_model_direction_consideration_request_id: None,
            repo_frontier_modeling_request_id: None,
            repo_frontier_research_request_id: None,
            repo_frontier_verification_request_id: None,
            created_at: created_at.into(),
        },
    )?;
    let mut expected = exact_source_envelopes(&cache, &work.request.source_documents)?
        .into_iter()
        .chain(
            cache
                .get_envelope::<EpiphanyReorientationRequest>(&work.request.request_id)?
                .into_iter(),
        )
        .collect::<Vec<_>>();
    for (_, launch) in &prior_launches {
        expected.push(
            cache
                .get_envelope::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&launch.job_id)?
                .ok_or_else(|| anyhow!("reorientation retry lost its prior launch envelope"))?,
        );
    }
    if let Some((_, latest)) = prior_launches.last() {
        expected.push(
            cache
                .get_envelope::<crate::EpiphanyRuntimeJob>(&latest.job_id)?
                .ok_or_else(|| anyhow!("reorientation retry lost its prior job envelope"))?,
        );
        for failure in cache
            .get_all::<EpiphanyMindReorientationPassFailureDocument>()?
            .into_iter()
            .filter(|failure| {
                failure.request_id == work.request.request_id && failure.job_id == latest.job_id
            })
        {
            expected.push(
                cache
                    .get_envelope::<EpiphanyMindReorientationPassFailureDocument>(
                        &failure.failure_id,
                    )?
                    .ok_or_else(|| anyhow!("reorientation retry lost its failure envelope"))?,
            );
            expected.push(
                cache
                    .get_envelope::<crate::EpiphanyRuntimeJobResult>(&failure.runtime_result_id)?
                    .ok_or_else(|| {
                        anyhow!("reorientation retry lost its terminal result envelope")
                    })?,
            );
        }
    }
    crate::current_work::commit_current_work_launch(
        store_path,
        &cache,
        expected,
        prepared,
        Vec::new(),
        "reorientation",
    )?;
    Ok(job_id)
}

pub fn accept_reorientation_result(
    store_path: impl AsRef<Path>,
    job_id: &str,
    accepted_at: &str,
) -> Result<crate::EpiphanyMindCommitReceipt> {
    chrono::DateTime::parse_from_rfc3339(accepted_at)
        .map_err(|_| anyhow!("reorientation acceptance time is invalid"))?;
    let store_path = store_path.as_ref();
    let mut cache = crate::runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    let launch = cache
        .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(job_id)?
        .ok_or_else(|| anyhow!("reorientation result has no immutable launch"))?;
    let launch_document = reorientation_launch_document(&launch)?;
    let request = cache
        .get::<EpiphanyReorientationRequest>(&launch_document.request_id)?
        .ok_or_else(|| anyhow!("reorientation result lost its request"))?;
    validate_reorientation_request_intrinsic(&request)?;
    let job = cache
        .get::<crate::EpiphanyRuntimeJob>(job_id)?
        .ok_or_else(|| anyhow!("reorientation result lost its runtime job"))?;
    if job.status != EpiphanyRuntimeJobStatus::Completed {
        return Err(anyhow!("reorientation runtime job is not completed"));
    }
    let result = cache
        .get::<crate::EpiphanyRuntimeReorientWorkerResult>(job_id)?
        .ok_or_else(|| anyhow!("reorientation pass has no typed result"))?;
    if result.item_error.is_some()
        || !matches!(
            result.mode.trim().to_ascii_lowercase().as_str(),
            "resume" | "regather"
        )
        || result.summary.trim().is_empty()
        || result.next_safe_move.trim().is_empty()
    {
        return Err(anyhow!(
            "reorientation result is not an admissible decision"
        ));
    }
    let context = cache
        .get::<crate::EpiphanyDecisionContext>(&result.decision_context_id)?
        .ok_or_else(|| anyhow!("reorientation result lost its decision context"))?;
    let basis = cache
        .get::<crate::EpiphanyReasoningBasis>(&context.basis_id)?
        .ok_or_else(|| anyhow!("reorientation result lost its reasoning basis"))?;
    match basis.projection()? {
        crate::EpiphanyReasoningProjection::ReorientLaunch(projection)
            if projection.request == request => {}
        _ => return Err(anyhow!("reorientation result crossed its sealed request")),
    }
    if let Some(existing) = cache
        .get_all::<EpiphanyMindReorientationDecisionDocument>()?
        .into_iter()
        .find(|decision| decision.request_id == request.request_id)
    {
        if existing.job_id != job_id
            || existing.result_id != result.result_id
            || existing.decision_context_id != result.decision_context_id
        {
            return Err(anyhow!("reorientation decision replay was substituted"));
        }
        return mind_receipt_for_context(&cache, &result.decision_context_id);
    }
    let decision = EpiphanyMindReorientationDecisionDocument {
        schema_version: MIND_REORIENTATION_DECISION_SCHEMA_VERSION.into(),
        decision_id: format!("reorientation-decision-{}", request.request_id),
        request_id: request.request_id.clone(),
        job_id: job_id.into(),
        result_id: result.result_id.clone(),
        decision_context_id: result.decision_context_id.clone(),
        mode: result.mode.trim().to_ascii_lowercase(),
        checkpoint_still_valid: result.checkpoint_still_valid,
        summary: result.summary.clone(),
        next_safe_move: result.next_safe_move.clone(),
        decided_at: accepted_at.into(),
    };
    let mut strong_reads = exact_source_envelopes(&cache, &request.source_documents)?;
    for (document_type, document_key) in [
        (
            EpiphanyReorientationRequest::TYPE,
            request.request_id.as_str(),
        ),
        (crate::EpiphanyRuntimeWorkerLaunchRequest::TYPE, job_id),
        (crate::EpiphanyRuntimeJob::TYPE, job_id),
        (crate::EpiphanyRuntimeReorientWorkerResult::TYPE, job_id),
        (
            crate::EpiphanyDecisionContext::TYPE,
            result.decision_context_id.as_str(),
        ),
    ] {
        strong_reads.push(
            cache
                .snapshot_envelopes()
                .into_iter()
                .find(|envelope| envelope.r#type == document_type && envelope.key == document_key)
                .ok_or_else(|| anyhow!("reorientation admission lost a strong source"))?,
        );
    }
    let write =
        crate::mind_documents::prepare_mind_document(&cache, &decision.decision_id, &decision)?;
    match crate::reasoning_context::commit_mind_mutation(
        store_path,
        &result.decision_context_id,
        "Continuity.reorientation",
        strong_reads,
        vec![write],
        accepted_at,
    )? {
        crate::EpiphanyMindCommitOutcome::Committed(receipt) => Ok(receipt),
        crate::EpiphanyMindCommitOutcome::Conflict {
            document_identities,
        } => Err(anyhow!(
            "reorientation output cannot be rebased onto changed continuity authority: {document_identities:?}"
        )),
    }
}

pub(crate) fn validate_reorientation_request_intrinsic(
    request: &EpiphanyReorientationRequest,
) -> Result<()> {
    if request.schema_version != REORIENTATION_REQUEST_SCHEMA_VERSION
        || request.request_id.trim().is_empty()
        || request.runtime_id.trim().is_empty()
        || request.projection.runtime_id != request.runtime_id
        || request.projection.projection_digest
            != crate::epiphany_mind_projection_digest(&request.source_documents)?
        || request.source_documents.is_empty()
        || chrono::DateTime::parse_from_rfc3339(&request.requested_at).is_err()
    {
        return Err(anyhow!("reorientation request is invalid"));
    }
    let expected_id = format!(
        "reorientation-request-{:x}",
        Sha256::digest(rmp_serde::to_vec_named(&(
            REORIENTATION_REQUEST_SCHEMA_VERSION,
            request.runtime_id.as_str(),
            request.projection.projection_digest.as_str(),
        ))?)
    );
    if request.request_id != expected_id {
        return Err(anyhow!("reorientation request identity is not canonical"));
    }
    if projection_from_sources(&request.source_documents)? != request.projection {
        return Err(anyhow!(
            "reorientation projection diverges from its exact Mind documents"
        ));
    }
    Ok(())
}

pub(crate) fn validate_reorientation_request_current(
    cache: &CultCache,
    request: &EpiphanyReorientationRequest,
) -> Result<()> {
    validate_reorientation_request_intrinsic(request)?;
    exact_source_envelopes(cache, &request.source_documents)?;
    Ok(())
}

fn exact_source_envelopes(
    cache: &CultCache,
    sources: &[EpiphanyMindDocumentVersion],
) -> Result<Vec<cultcache_rs::CultCacheEnvelope>> {
    let snapshot = cache.snapshot_envelopes();
    sources
        .iter()
        .map(|source| {
            let envelope = snapshot
                .iter()
                .find(|envelope| {
                    envelope.r#type == source.document_type && envelope.key == source.document_key
                })
                .cloned()
                .ok_or_else(|| anyhow!("reorientation source document is absent"))?;
            if EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", &envelope)? != *source {
                return Err(anyhow!("reorientation source document changed"));
            }
            Ok(envelope)
        })
        .collect()
}

fn projection_from_sources(
    sources: &[EpiphanyMindDocumentVersion],
) -> Result<EpiphanyReorientationStateProjection> {
    let mut runtime_id = None;
    let mut objective = None;
    let mut focus = None;
    let mut checkpoints = Vec::new();
    let mut mode = None;
    let mut bodies = Vec::new();
    let mut frontier = Vec::new();
    for source in sources {
        match source.document_type.as_str() {
            crate::EpiphanyMindIdentity::TYPE => {
                runtime_id = Some(
                    rmp_serde::from_slice::<crate::EpiphanyMindIdentity>(&source.payload_msgpack)?
                        .runtime_id,
                );
            }
            crate::EpiphanyMindObjectiveDocument::TYPE => {
                objective = Some(
                    rmp_serde::from_slice::<crate::EpiphanyMindObjectiveDocument>(
                        &source.payload_msgpack,
                    )?
                    .objective,
                );
            }
            crate::EpiphanyMindFocusDocument::TYPE => {
                focus = Some(rmp_serde::from_slice::<crate::EpiphanyMindFocusDocument>(
                    &source.payload_msgpack,
                )?);
            }
            crate::EpiphanyMindInvestigationCheckpointDocument::TYPE => checkpoints.push(
                rmp_serde::from_slice::<crate::EpiphanyMindInvestigationCheckpointDocument>(
                    &source.payload_msgpack,
                )?
                .value,
            ),
            crate::EpiphanyMindModeDocument::TYPE => {
                mode = Some(
                    rmp_serde::from_slice::<crate::EpiphanyMindModeDocument>(
                        &source.payload_msgpack,
                    )?
                    .mode,
                );
            }
            crate::EpiphanyMindRepositoryBodyObservationDocument::TYPE => bodies.push(
                rmp_serde::from_slice::<crate::EpiphanyMindRepositoryBodyObservationDocument>(
                    &source.payload_msgpack,
                )?
                .basis,
            ),
            crate::EpiphanyRepoModelFrontierDocument::TYPE => frontier.push(
                rmp_serde::from_slice::<crate::EpiphanyRepoModelFrontierDocument>(
                    &source.payload_msgpack,
                )?
                .value()?,
            ),
            crate::EpiphanyRepoModelIdentityDocument::TYPE
            | crate::EpiphanyMindReorientationDecisionDocument::TYPE => {}
            other => {
                return Err(anyhow!(
                    "reorientation request contains non-projection source {other:?}"
                ));
            }
        }
    }
    let runtime_id =
        runtime_id.ok_or_else(|| anyhow!("reorientation projection has no Mind identity"))?;
    let active_subgoal_id = focus
        .as_ref()
        .and_then(|value| value.active_subgoal_id.clone());
    let checkpoint_id = focus
        .as_ref()
        .and_then(|value| value.investigation_checkpoint_id.as_deref());
    let investigation_checkpoint = match checkpoint_id {
        Some(id) => Some(
            checkpoints
                .into_iter()
                .find(|checkpoint| checkpoint.checkpoint_id == id)
                .ok_or_else(|| anyhow!("reorientation focus cites a missing checkpoint"))?,
        ),
        None => None,
    };
    bodies.sort_by_key(|body| body.generation);
    frontier.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(EpiphanyReorientationStateProjection {
        runtime_id,
        objective,
        active_subgoal_id,
        investigation_checkpoint,
        mode,
        repository_body_observation: bodies.pop(),
        frontier,
        projection_digest: crate::epiphany_mind_projection_digest(sources)?,
    })
}

fn mind_receipt_for_context(
    cache: &CultCache,
    decision_context_id: &str,
) -> Result<crate::EpiphanyMindCommitReceipt> {
    let mut receipts = cache
        .get_all::<crate::EpiphanyMindCommitReceipt>()?
        .into_iter()
        .filter(|receipt| {
            receipt.invariant_owner == "Continuity.reorientation"
                && matches!(
                    &receipt.authority,
                    crate::EpiphanyMindCommitAuthority::ModelDecisionContext {
                        decision_context_id: id
                    } if id == decision_context_id
                )
        })
        .collect::<Vec<_>>();
    match receipts.len() {
        1 => Ok(receipts.remove(0)),
        0 => Err(anyhow!(
            "reorientation decision lost its Mind commit receipt"
        )),
        _ => Err(anyhow!(
            "reorientation decision has multiple Mind commit receipts"
        )),
    }
}

fn mind_receipt_for_failure_context(
    cache: &CultCache,
    decision_context_id: &str,
) -> Result<crate::EpiphanyMindCommitReceipt> {
    let mut receipts = cache
        .get_all::<crate::EpiphanyMindCommitReceipt>()?
        .into_iter()
        .filter(|receipt| {
            receipt.invariant_owner == "Continuity.reorientation_failure"
                && matches!(
                    &receipt.authority,
                    crate::EpiphanyMindCommitAuthority::ModelDecisionContext {
                        decision_context_id: id
                    } if id == decision_context_id
                )
        })
        .collect::<Vec<_>>();
    match receipts.len() {
        1 => Ok(receipts.remove(0)),
        0 => Err(anyhow!(
            "reorientation failure lost its Mind commit receipt"
        )),
        _ => Err(anyhow!(
            "reorientation failure has multiple Mind commit receipts"
        )),
    }
}

fn exact_runtime_job_result(
    cache: &CultCache,
    job_id: &str,
) -> Result<crate::EpiphanyRuntimeJobResult> {
    let mut results = cache
        .get_all::<crate::EpiphanyRuntimeJobResult>()?
        .into_iter()
        .filter(|result| result.job_id == job_id)
        .collect::<Vec<_>>();
    match results.len() {
        1 => Ok(results.remove(0)),
        0 => Err(anyhow!(
            "reorientation attempt lost its terminal runtime result"
        )),
        _ => Err(anyhow!(
            "reorientation attempt has multiple terminal runtime results"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_model_adapter::EpiphanyModelRequest;

    #[test]
    fn reorientation_is_decision_auditable_thread_free_and_keyed() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("reorientation.cc");
        crate::initialize_runtime_spine(
            &store,
            crate::RuntimeSpineInitOptions {
                runtime_id: "runtime".into(),
                display_name: "Reorientation test".into(),
                created_at: "2026-08-18T10:00:00Z".into(),
            },
        )?;
        crate::runtime_spine::tests::bind_test_runtime_swarm(&store, "reorientation-swarm")?;
        crate::runtime_spine::tests::bind_test_repository_body(&store, "reorientation-workspace")?;
        let body = crate::observe_runtime_repository_body_basis(&store)?;
        crate::initialize_keyed_repo_model(
            &store,
            &crate::EpiphanyRepoModelSeed::new(
                "reorientation-seed",
                "reorientation-graph",
                "reorientation-swarm",
                "reorientation-workspace",
                body.body_binding_sha256,
                crate::EpiphanyRepoModelSeedDocuments {
                    domains: Vec::new(),
                    nodes: Vec::new(),
                    edges: Vec::new(),
                    frontier: Vec::new(),
                },
            )?,
            "2026-08-18T10:00:00Z",
        )?;
        let mut cache = crate::runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        let objective = crate::EpiphanyMindObjectiveDocument {
            objective: "Continue the keyed Mind migration".into(),
        };
        cache.put(crate::MIND_OBJECTIVE_KEY, &objective)?;

        let request = request_current_reorientation(&store, "2026-08-18T10:00:01Z")?;
        assert!(
            request
                .source_documents
                .iter()
                .all(|source| source.document_type != "epiphany.thread_state")
        );
        assert_eq!(
            crate::project_current_work(&store)?
                .reorientation
                .as_ref()
                .map(|work| work.attempt.action),
            Some(EpiphanyAgentPassContinuationAction::Launch)
        );
        let job_id = launch_current_reorientation_work(&store, "2026-08-18T10:00:02Z")?;
        cache.pull_all_backing_stores()?;
        let launch = cache
            .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&job_id)?
            .expect("reorientation launch");
        let crate::EpiphanyWorkerLaunchDocument::Reorient(authority) = launch.launch_document()?
        else {
            panic!("expected reorientation launch");
        };
        assert_eq!(authority.request_id, request.request_id);
        assert_eq!(authority.creation_thread_id, job_id);

        let basis = crate::worker_reasoning_basis(&store, &launch)?;
        crate::put_reasoning_basis(&store, &basis)?;
        assert_eq!(basis.source_documents, request.source_documents);
        let mut native = EpiphanyModelRequest::new(
            "reorientation-model-request",
            "reorientation-conversation",
            "openai-codex",
            "gpt-test",
            "reorient",
        );
        native.reasoning_basis_id = Some(basis.basis_id.clone());
        native.source_worker_job_id = Some(job_id.clone());
        native.output_contract_id = Some(launch.output_contract_id.clone());
        native.output_schema_json = Some(serde_json::to_string(
            &crate::epiphany_reorient_launch_output_schema(),
        )?);
        crate::open_runtime_model_execution(
            &store,
            crate::RuntimeSpineSessionOptions {
                session_id: "reorientation-model-session".into(),
                objective: "Reorient from exact Mind state".into(),
                created_at: "2026-08-18T10:00:03Z".into(),
                coordinator_note: "test".into(),
            },
            crate::RuntimeSpineJobOptions {
                job_id: "reorientation-model-job".into(),
                session_id: "reorientation-model-session".into(),
                role: "openai-model".into(),
                created_at: "2026-08-18T10:00:03Z".into(),
            },
            &native,
            "2026-08-18T10:00:03Z",
        )?;
        let context = crate::EpiphanyDecisionContext::new(&basis, native, Vec::new())?;
        crate::put_decision_context(&store, &context)?;
        let process = crate::ProcessInstanceIdentity {
            process_id: 91,
            creation_token: 17,
            created_at_rfc3339: Some("2026-08-18T10:00:04Z".into()),
            executable_path: "reorientation-worker".into(),
        };
        let activation = "reorientation-activation";
        crate::claim_runtime_worker_process(
            &store,
            &job_id,
            &process,
            &format!("{:x}", Sha256::digest(activation.as_bytes())),
            "2026-08-18T10:00:04Z",
        )?;
        crate::activate_runtime_worker_process(
            &store,
            &job_id,
            &process,
            activation,
            "2026-08-18T10:00:05Z",
        )?;
        let result = crate::EpiphanyRuntimeReorientWorkerResult {
            schema_version: crate::RUNTIME_REORIENT_WORKER_RESULT_SCHEMA_VERSION.into(),
            result_id: "reorientation-result".into(),
            job_id: job_id.clone(),
            mode: "resume".into(),
            summary: "The sealed checkpoint remains coherent.".into(),
            next_safe_move: "Continue the keyed migration.".into(),
            checkpoint_still_valid: Some(true),
            files_inspected: Vec::new(),
            frontier_node_ids: Vec::new(),
            evidence_ids: Vec::new(),
            artifact_refs: Vec::new(),
            open_questions: Vec::new(),
            continuity_risks: Vec::new(),
            item_error: None,
            metadata: Default::default(),
            decision_context_id: context.context_id.clone(),
        };
        crate::put_runtime_reorient_worker_result(&store, &result)?;
        crate::complete_runtime_job(
            &store,
            crate::RuntimeSpineJobResultOptions {
                result_id: format!("runtime-result-{job_id}"),
                job_id: job_id.clone(),
                completed_at: "2026-08-18T10:00:06Z".into(),
                verdict: "resume".into(),
                summary: result.summary.clone(),
                next_safe_move: result.next_safe_move.clone(),
                evidence_refs: Vec::new(),
                artifact_refs: Vec::new(),
                decision_context_id: Some(context.context_id.clone()),
            },
        )?;

        cache.pull_all_backing_stores()?;
        cache.put(
            "unrelated-observation",
            &crate::EpiphanyMindObservationDocument {
                value: epiphany_state_model::EpiphanyObservation {
                    id: "unrelated-observation".into(),
                    summary: "Persona observed an unrelated social fact.".into(),
                    ..Default::default()
                },
            },
        )?;
        let stale_store = temp.path().join("reorientation-stale.cc");
        std::fs::copy(&store, &stale_store)?;

        let receipt = accept_reorientation_result(&store, &job_id, "2026-08-18T10:00:07Z")?;
        assert_eq!(receipt.invariant_owner, "Continuity.reorientation");
        assert_eq!(
            accept_reorientation_result(&store, &job_id, "2026-08-18T10:00:07Z")?,
            receipt
        );
        assert!(crate::project_current_work(&store)?.reorientation.is_none());
        let mut accepted = crate::runtime_spine_cache(&store)?;
        accepted.pull_all_backing_stores()?;
        assert!(
            accepted
                .get::<crate::EpiphanyMindReorientationDecisionDocument>(&format!(
                    "reorientation-decision-{}",
                    request.request_id
                ))?
                .is_some()
        );
        let next_request = request_current_reorientation(&store, "2026-08-18T10:00:08Z")?;
        assert_ne!(next_request.request_id, request.request_id);
        assert!(next_request.source_documents.iter().any(|source| {
            source.document_type == EpiphanyMindReorientationDecisionDocument::TYPE
                && source.document_key == format!("reorientation-decision-{}", request.request_id)
        }));
        let failed_job = launch_current_reorientation_work(&store, "2026-08-18T10:00:09Z")?;
        let mut failed_cache = crate::runtime_spine_cache(&store)?;
        failed_cache.pull_all_backing_stores()?;
        let failed_launch = failed_cache
            .get::<crate::EpiphanyRuntimeWorkerLaunchRequest>(&failed_job)?
            .expect("failed reorientation launch");
        let failed_basis = crate::worker_reasoning_basis(&store, &failed_launch)?;
        crate::put_reasoning_basis(&store, &failed_basis)?;
        let mut failed_native = EpiphanyModelRequest::new(
            "failed-reorientation-model-request",
            "failed-reorientation-conversation",
            "openai-codex",
            "gpt-test",
            "reorient",
        );
        failed_native.reasoning_basis_id = Some(failed_basis.basis_id.clone());
        failed_native.source_worker_job_id = Some(failed_job.clone());
        failed_native.output_contract_id = Some(failed_launch.output_contract_id.clone());
        failed_native.output_schema_json = Some(serde_json::to_string(
            &crate::epiphany_reorient_launch_output_schema(),
        )?);
        crate::open_runtime_model_execution(
            &store,
            crate::RuntimeSpineSessionOptions {
                session_id: "failed-reorientation-model-session".into(),
                objective: "Fail from exact Mind state".into(),
                created_at: "2026-08-18T10:00:10Z".into(),
                coordinator_note: "test".into(),
            },
            crate::RuntimeSpineJobOptions {
                job_id: "failed-reorientation-model-job".into(),
                session_id: "failed-reorientation-model-session".into(),
                role: "openai-model".into(),
                created_at: "2026-08-18T10:00:10Z".into(),
            },
            &failed_native,
            "2026-08-18T10:00:10Z",
        )?;
        let failed_context =
            crate::EpiphanyDecisionContext::new(&failed_basis, failed_native, Vec::new())?;
        crate::put_decision_context(&store, &failed_context)?;
        let model_failure = crate::terminalize_model_pass_failure_session(
            &store,
            crate::ModelPassFailureTerminalOptions {
                decision_context_id: failed_context.context_id.clone(),
                failure_kind: "provider_or_transport_failure".into(),
                summary: "Provider failed after the terminal request was sealed.".into(),
                failed_at: "2026-08-18T10:00:13Z".into(),
            },
        )?;
        assert_eq!(model_failure.decision_context_id, failed_context.context_id);
        let failed_process = crate::ProcessInstanceIdentity {
            process_id: 92,
            creation_token: 18,
            created_at_rfc3339: Some("2026-08-18T10:00:11Z".into()),
            executable_path: "failed-reorientation-worker".into(),
        };
        let failed_activation = "failed-reorientation-activation";
        crate::claim_runtime_worker_process(
            &store,
            &failed_job,
            &failed_process,
            &format!("{:x}", Sha256::digest(failed_activation.as_bytes())),
            "2026-08-18T10:00:11Z",
        )?;
        crate::activate_runtime_worker_process(
            &store,
            &failed_job,
            &failed_process,
            failed_activation,
            "2026-08-18T10:00:12Z",
        )?;
        crate::complete_runtime_job(
            &store,
            crate::RuntimeSpineJobResultOptions {
                result_id: format!("runtime-result-{failed_job}"),
                job_id: failed_job.clone(),
                completed_at: "2026-08-18T10:00:14Z".into(),
                verdict: "failed".into(),
                summary: "Provider failed after the terminal request was sealed.".into(),
                next_safe_move: "Review the typed failure, then retry.".into(),
                evidence_refs: Vec::new(),
                artifact_refs: Vec::new(),
                decision_context_id: Some(failed_context.context_id.clone()),
            },
        )?;
        assert_eq!(
            crate::project_current_work(&store)?
                .reorientation
                .as_ref()
                .map(|work| work.attempt.action),
            Some(EpiphanyAgentPassContinuationAction::Review)
        );
        let failure_receipt = record_reorientation_pass_failure(&store, &failed_job)?;
        assert_eq!(
            record_reorientation_pass_failure(&store, &failed_job)?,
            failure_receipt
        );
        assert_eq!(
            crate::project_current_work(&store)?
                .reorientation
                .as_ref()
                .map(|work| work.attempt.action),
            Some(EpiphanyAgentPassContinuationAction::Launch)
        );
        assert!(
            accepted
                .snapshot_envelopes()
                .iter()
                .all(|envelope| envelope.r#type != "epiphany.thread_state")
        );

        let mut stale = crate::runtime_spine_cache(&stale_store)?;
        stale.pull_all_backing_stores()?;
        stale.put(
            crate::MIND_OBJECTIVE_KEY,
            &crate::EpiphanyMindObjectiveDocument {
                objective: "Changed strong objective".into(),
            },
        )?;
        let before = std::fs::read(&stale_store)?;
        assert!(
            accept_reorientation_result(&stale_store, &job_id, "2026-08-18T10:00:07Z").is_err()
        );
        assert_eq!(std::fs::read(&stale_store)?, before);
        Ok(())
    }
}
