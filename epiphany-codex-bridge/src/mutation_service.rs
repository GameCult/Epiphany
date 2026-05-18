use chrono::SecondsFormat;
use chrono::Utc;
use codex_app_server_protocol::ThreadEpiphanyJob;
use codex_app_server_protocol::ThreadEpiphanyJobKind;
use codex_app_server_protocol::ThreadEpiphanyReorientDecision;
use codex_app_server_protocol::ThreadEpiphanyReorientFinding;
use codex_app_server_protocol::ThreadEpiphanyReorientSource;
use codex_app_server_protocol::ThreadEpiphanyReorientStateStatus;
use codex_app_server_protocol::ThreadEpiphanyRoleFinding;
use codex_app_server_protocol::ThreadEpiphanyRoleId;
use codex_app_server_protocol::ThreadEpiphanyStateUpdatedField;
use codex_app_server_protocol::ThreadEpiphanyUpdatePatch;
use codex_core::CodexThread;
use codex_protocol::error::CodexErr;
use codex_protocol::protocol::EpiphanyEvidenceRecord;
use codex_protocol::protocol::EpiphanyJobKind as CoreEpiphanyJobKind;
use codex_protocol::protocol::EpiphanyRetrievalState;
use codex_protocol::protocol::EpiphanyThreadState;
use codex_protocol::protocol::TokenUsageInfo as CoreTokenUsageInfo;
use epiphany_core::EpiphanyJobInterruptRequest;
use epiphany_core::EpiphanyJobLaunchRequest;
use epiphany_core::EpiphanyMemoryPatchReview;
use epiphany_core::EpiphanyMemoryPatchReviewStatus;
use epiphany_core::EpiphanyPromotionInput;
use epiphany_core::apply_memory_patch_candidate;
use epiphany_core::evaluate_promotion;
use epiphany_core::load_memory_graph_snapshot;
use epiphany_core::memory_graph_from_epiphany_graphs;
use epiphany_core::runtime_role_worker_result;
use epiphany_core::write_memory_graph_snapshot;
use std::fs;

use crate::jobs::epiphany_blocked_state_job;
use crate::jobs::map_epiphany_jobs;
use crate::jobs::map_interrupted_epiphany_job;
use crate::jobs::map_launched_epiphany_job;
use crate::launch::EPIPHANY_REORIENT_LAUNCH_BINDING_ID;
use crate::launch::build_epiphany_reorient_launch_request;
use crate::launch::build_epiphany_role_launch_request;
use crate::launch::epiphany_role_label;
use crate::mutation::build_reorient_acceptance_update;
use crate::mutation::build_role_acceptance_update;
use crate::mutation::epiphany_job_launch_changed_fields;
use crate::mutation::epiphany_promote_changed_fields;
use crate::mutation::epiphany_update_patch_changed_fields;
use crate::mutation::state_update_from_thread_patch;
use crate::mutation::thread_epiphany_patch_has_state_replacements;
use crate::pressure::map_epiphany_pressure;
use crate::reorient::EpiphanyFreshnessWatcherSnapshot;
use crate::reorient::map_epiphany_freshness;
use crate::reorient::map_epiphany_reorient;
use crate::runtime_results::load_completed_epiphany_reorient_finding;
use crate::runtime_results::load_completed_epiphany_role_finding;
use crate::state::client_visible_live_thread_epiphany_state;
use crate::state::load_thread_memory_graph_snapshot;
use crate::state::memory_graph_store_path;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EpiphanyThreadUpdateApplied {
    pub revision: u64,
    pub changed_fields: Vec<ThreadEpiphanyStateUpdatedField>,
    pub epiphany_state: EpiphanyThreadState,
}

#[derive(Debug, Clone)]
pub enum EpiphanyThreadPromoteApplied {
    Rejected { reasons: Vec<String> },
    Accepted(EpiphanyThreadUpdateApplied),
}

#[derive(Debug, Clone)]
pub struct EpiphanyRoleAcceptApplied {
    pub revision: u64,
    pub changed_fields: Vec<ThreadEpiphanyStateUpdatedField>,
    pub epiphany_state: EpiphanyThreadState,
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
    pub applied_patch: ThreadEpiphanyUpdatePatch,
    pub finding: ThreadEpiphanyRoleFinding,
    pub memory_patch_reviews: Vec<EpiphanyMemoryPatchReview>,
}

#[derive(Debug, Clone)]
pub struct EpiphanyReorientAcceptApplied {
    pub revision: u64,
    pub changed_fields: Vec<ThreadEpiphanyStateUpdatedField>,
    pub epiphany_state: EpiphanyThreadState,
    pub accepted_receipt_id: String,
    pub accepted_observation_id: String,
    pub accepted_evidence_id: String,
    pub finding: ThreadEpiphanyReorientFinding,
}

#[derive(Debug, Clone)]
pub struct EpiphanyJobLaunchApplied {
    pub revision: u64,
    pub changed_fields: Vec<ThreadEpiphanyStateUpdatedField>,
    pub epiphany_state: EpiphanyThreadState,
    pub job: ThreadEpiphanyJob,
}

#[derive(Debug, Clone)]
pub struct EpiphanyJobInterruptApplied {
    pub cancel_requested: bool,
    pub interrupted_thread_ids: Vec<String>,
    pub revision: u64,
    pub changed_fields: Vec<ThreadEpiphanyStateUpdatedField>,
    pub epiphany_state: EpiphanyThreadState,
    pub job: ThreadEpiphanyJob,
}

#[derive(Debug, Clone)]
pub struct EpiphanyReorientLaunchApplied {
    pub source: ThreadEpiphanyReorientSource,
    pub state_status: ThreadEpiphanyReorientStateStatus,
    pub state_revision: Option<u64>,
    pub decision: ThreadEpiphanyReorientDecision,
    pub revision: u64,
    pub changed_fields: Vec<ThreadEpiphanyStateUpdatedField>,
    pub epiphany_state: EpiphanyThreadState,
    pub job: ThreadEpiphanyJob,
}

pub async fn apply_thread_epiphany_update(
    thread: &CodexThread,
    expected_revision: Option<u64>,
    patch: ThreadEpiphanyUpdatePatch,
) -> Result<EpiphanyThreadUpdateApplied, CodexErr> {
    let changed_fields = epiphany_update_patch_changed_fields(&patch);
    let update = state_update_from_thread_patch(expected_revision, patch);
    let epiphany_state = thread.epiphany_update_state(update).await?;
    let epiphany_state = client_visible_live_thread_epiphany_state(thread, epiphany_state).await;
    Ok(EpiphanyThreadUpdateApplied {
        revision: epiphany_state.revision,
        changed_fields,
        epiphany_state,
    })
}

pub async fn apply_thread_epiphany_promote(
    thread: &CodexThread,
    expected_revision: Option<u64>,
    patch: ThreadEpiphanyUpdatePatch,
    verifier_evidence: EpiphanyEvidenceRecord,
) -> Result<EpiphanyThreadPromoteApplied, CodexErr> {
    let decision = evaluate_promotion(EpiphanyPromotionInput {
        has_state_replacements: thread_epiphany_patch_has_state_replacements(&patch),
        active_subgoal_id: patch.active_subgoal_id.clone(),
        subgoals: patch.subgoals.clone(),
        invariants: patch.invariants.clone(),
        graph_frontier: patch.graph_frontier.clone(),
        graph_checkpoint: patch.graph_checkpoint.clone(),
        investigation_checkpoint: patch.investigation_checkpoint.clone(),
        churn: patch.churn.clone(),
        observations: patch.observations.clone(),
        evidence: patch.evidence.clone(),
        verifier_evidence: verifier_evidence.clone(),
    });
    if !decision.accepted {
        return Ok(EpiphanyThreadPromoteApplied::Rejected {
            reasons: decision.reasons,
        });
    }

    let changed_fields = epiphany_promote_changed_fields(&patch);
    let mut patch = patch;
    patch.evidence.push(verifier_evidence);
    let update = state_update_from_thread_patch(expected_revision, patch);
    let epiphany_state = thread.epiphany_update_state(update).await?;
    let epiphany_state = client_visible_live_thread_epiphany_state(thread, epiphany_state).await;
    Ok(EpiphanyThreadPromoteApplied::Accepted(
        EpiphanyThreadUpdateApplied {
            revision: epiphany_state.revision,
            changed_fields,
            epiphany_state,
        },
    ))
}

pub async fn apply_thread_epiphany_role_accept(
    thread: &CodexThread,
    role_id: ThreadEpiphanyRoleId,
    expected_revision: Option<u64>,
    binding_id: &str,
) -> Result<EpiphanyRoleAcceptApplied, CodexErr> {
    let state = thread.epiphany_state().await.ok_or_else(|| {
        CodexErr::InvalidRequest(
            "cannot accept an Epiphany role finding without authoritative Epiphany state"
                .to_string(),
        )
    })?;
    validate_expected_revision(expected_revision, state.revision)?;

    let finding = load_completed_epiphany_role_finding(thread, &state, role_id, binding_id).await?;
    let accepted_prefix = epiphany_role_label(role_id);
    let accepted_at = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    let memory_patch_reviews = apply_modeling_memory_patch_candidates(
        thread,
        role_id,
        &state,
        &finding,
        accepted_at.clone(),
    )
    .await?;
    let acceptance_update = build_role_acceptance_update(
        role_id,
        binding_id,
        &finding,
        format!("ev-{accepted_prefix}-{}", Uuid::new_v4()),
        format!("obs-{accepted_prefix}-{}", Uuid::new_v4()),
        accepted_at,
    )
    .map_err(CodexErr::InvalidRequest)?;

    let accepted_receipt_id = acceptance_update.accepted_receipt_id.clone();
    let accepted_observation_id = acceptance_update.accepted_observation_id.clone();
    let accepted_evidence_id = acceptance_update.accepted_evidence_id.clone();
    let changed_fields = acceptance_update.changed_fields.clone();
    let applied_patch = acceptance_update.patch.clone();
    let epiphany_state = thread
        .epiphany_update_state(state_update_from_thread_patch(
            expected_revision,
            acceptance_update.patch,
        ))
        .await?;
    let epiphany_state = client_visible_live_thread_epiphany_state(thread, epiphany_state).await;

    Ok(EpiphanyRoleAcceptApplied {
        revision: epiphany_state.revision,
        changed_fields,
        epiphany_state,
        accepted_receipt_id,
        accepted_observation_id,
        accepted_evidence_id,
        applied_patch,
        finding,
        memory_patch_reviews,
    })
}

async fn apply_modeling_memory_patch_candidates(
    thread: &CodexThread,
    role_id: ThreadEpiphanyRoleId,
    state: &EpiphanyThreadState,
    finding: &ThreadEpiphanyRoleFinding,
    accepted_at: String,
) -> Result<Vec<EpiphanyMemoryPatchReview>, CodexErr> {
    if role_id != ThreadEpiphanyRoleId::Modeling {
        return Ok(Vec::new());
    }
    let Some(runtime_job_id) = finding.runtime_job_id.as_deref() else {
        return Ok(Vec::new());
    };
    let runtime_store_path = thread.epiphany_runtime_spine_store_path().await;
    let Some(runtime_result) =
        runtime_role_worker_result(runtime_store_path.as_path(), runtime_job_id).map_err(|err| {
            CodexErr::InvalidRequest(format!(
                "failed to read runtime role worker result for memory patch candidates: {err}"
            ))
        })?
    else {
        return Ok(Vec::new());
    };
    let candidates = runtime_result.memory_patch_candidates().map_err(|err| {
        CodexErr::InvalidRequest(format!(
            "failed to decode memory patch candidates from runtime role result: {err}"
        ))
    })?;
    if candidates.is_empty() {
        return Ok(Vec::new());
    }

    let config = thread.config_snapshot().await;
    let store_path = memory_graph_store_path(config.cwd.as_path());
    let mut snapshot = load_memory_graph_snapshot(&store_path)
        .map_err(|err| {
            CodexErr::InvalidRequest(format!(
                "failed to load memory graph store {}: {err}",
                store_path.display()
            ))
        })?
        .unwrap_or_else(|| memory_graph_from_epiphany_graphs("epiphany-memory-graph", &state.graphs));

    let mut reviews = Vec::new();
    for candidate in &candidates {
        let (next_snapshot, review) =
            apply_memory_patch_candidate(&snapshot, candidate, Some(accepted_at.clone()));
        if review.status != EpiphanyMemoryPatchReviewStatus::Accepted {
            return Err(CodexErr::InvalidRequest(format!(
                "memory patch candidate {:?} was rejected: {}",
                candidate.id,
                review.errors.join("; ")
            )));
        }
        snapshot = next_snapshot;
        reviews.push(review);
    }

    if let Some(parent) = store_path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            CodexErr::InvalidRequest(format!(
                "failed to create memory graph store directory {}: {err}",
                parent.display()
            ))
        })?;
    }
    write_memory_graph_snapshot(&store_path, &snapshot).map_err(|err| {
        CodexErr::InvalidRequest(format!(
            "failed to write memory graph store {}: {err}",
            store_path.display()
        ))
    })?;

    Ok(reviews)
}

pub async fn apply_thread_epiphany_reorient_accept(
    thread: &CodexThread,
    expected_revision: Option<u64>,
    binding_id: &str,
    update_scratch: bool,
    update_investigation_checkpoint: bool,
) -> Result<EpiphanyReorientAcceptApplied, CodexErr> {
    let state = thread.epiphany_state().await.ok_or_else(|| {
        CodexErr::InvalidRequest(
            "cannot accept a reorientation finding without authoritative Epiphany state"
                .to_string(),
        )
    })?;
    validate_expected_revision(expected_revision, state.revision)?;
    if update_investigation_checkpoint && state.investigation_checkpoint.is_none() {
        return Err(CodexErr::InvalidRequest(
            "cannot update investigation checkpoint because this thread has no durable checkpoint"
                .to_string(),
        ));
    }

    let finding = load_completed_epiphany_reorient_finding(thread, &state, binding_id).await?;
    let acceptance_update = build_reorient_acceptance_update(
        expected_revision,
        binding_id,
        &finding,
        format!("ev-reorient-{}", Uuid::new_v4()),
        format!("obs-reorient-{}", Uuid::new_v4()),
        Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        update_scratch,
        update_investigation_checkpoint,
        state.investigation_checkpoint.clone(),
    )
    .map_err(CodexErr::InvalidRequest)?;

    let accepted_receipt_id = acceptance_update.accepted_receipt_id.clone();
    let accepted_observation_id = acceptance_update.accepted_observation_id.clone();
    let accepted_evidence_id = acceptance_update.accepted_evidence_id.clone();
    let changed_fields = acceptance_update.changed_fields.clone();
    let epiphany_state = thread
        .epiphany_update_state(acceptance_update.state_update)
        .await?;
    let epiphany_state = client_visible_live_thread_epiphany_state(thread, epiphany_state).await;

    Ok(EpiphanyReorientAcceptApplied {
        revision: epiphany_state.revision,
        changed_fields,
        epiphany_state,
        accepted_receipt_id,
        accepted_observation_id,
        accepted_evidence_id,
        finding,
    })
}

pub async fn launch_thread_epiphany_role(
    thread: &CodexThread,
    thread_id: &str,
    role_id: ThreadEpiphanyRoleId,
    expected_revision: Option<u64>,
    max_runtime_seconds: Option<u64>,
) -> Result<EpiphanyJobLaunchApplied, CodexErr> {
    let state = thread.epiphany_state().await.ok_or_else(|| {
        CodexErr::InvalidRequest(
            "cannot launch an Epiphany role specialist without authoritative Epiphany state"
                .to_string(),
        )
    })?;
    let memory_graph_snapshot = load_thread_memory_graph_snapshot(thread).await?;
    let launch_request = build_epiphany_role_launch_request(
        thread_id,
        role_id,
        expected_revision,
        max_runtime_seconds,
        &state,
        memory_graph_snapshot.as_ref(),
    )
    .map_err(CodexErr::InvalidRequest)?;
    launch_thread_epiphany_job(
        thread,
        launch_request,
        CoreEpiphanyJobKind::Specialist,
        "missing launched role projection",
    )
    .await
}

pub async fn launch_thread_epiphany_job(
    thread: &CodexThread,
    launch_request: EpiphanyJobLaunchRequest,
    kind: CoreEpiphanyJobKind,
    missing_projection_reason: &str,
) -> Result<EpiphanyJobLaunchApplied, CodexErr> {
    let changed_fields = epiphany_job_launch_changed_fields();
    let launched = thread.epiphany_launch_job(launch_request).await?;
    let epiphany_state =
        client_visible_live_thread_epiphany_state(thread, launched.epiphany_state).await;
    let job = map_launched_epiphany_job(
        &epiphany_state,
        launched.binding_id.as_str(),
        launched.launcher_job_id.as_str(),
        launched.backend_job_id.as_str(),
        kind,
        missing_projection_reason,
    );
    Ok(EpiphanyJobLaunchApplied {
        revision: epiphany_state.revision,
        changed_fields,
        epiphany_state,
        job,
    })
}

pub async fn interrupt_thread_epiphany_job(
    thread: &CodexThread,
    expected_revision: Option<u64>,
    binding_id: &str,
    reason: Option<String>,
) -> Result<EpiphanyJobInterruptApplied, CodexErr> {
    let changed_fields = vec![ThreadEpiphanyStateUpdatedField::JobBindings];
    let interrupted = thread
        .epiphany_interrupt_job(EpiphanyJobInterruptRequest {
            expected_revision,
            binding_id: binding_id.to_string(),
            reason,
        })
        .await?;
    let epiphany_state =
        client_visible_live_thread_epiphany_state(thread, interrupted.epiphany_state).await;
    let job = map_interrupted_epiphany_job(&epiphany_state, binding_id);

    Ok(EpiphanyJobInterruptApplied {
        cancel_requested: interrupted.cancel_requested,
        interrupted_thread_ids: interrupted.interrupted_thread_ids,
        revision: epiphany_state.revision,
        changed_fields,
        epiphany_state,
        job,
    })
}

pub async fn launch_thread_epiphany_reorient(
    thread: &CodexThread,
    thread_id: &str,
    expected_revision: Option<u64>,
    max_runtime_seconds: Option<u64>,
    state: Option<&EpiphanyThreadState>,
    retrieval_override: Option<&EpiphanyRetrievalState>,
    watcher_snapshot: Option<EpiphanyFreshnessWatcherSnapshot<'_>>,
    token_usage_info: Option<&CoreTokenUsageInfo>,
) -> Result<EpiphanyReorientLaunchApplied, CodexErr> {
    let (state_revision, retrieval, graph, watcher) =
        map_epiphany_freshness(state, retrieval_override, watcher_snapshot);
    let pressure = map_epiphany_pressure(token_usage_info);
    let (state_status, decision) =
        map_epiphany_reorient(state, &pressure, &retrieval, &graph, &watcher);

    let state = state.ok_or_else(|| {
        CodexErr::InvalidRequest(format!(
            "cannot launch a reorientation worker without authoritative Epiphany state: {}",
            decision.note
        ))
    })?;
    let checkpoint = state.investigation_checkpoint.as_ref().ok_or_else(|| {
        CodexErr::InvalidRequest(format!(
            "cannot launch a reorientation worker without a durable investigation checkpoint: {}",
            decision.note
        ))
    })?;

    let launch_request = build_epiphany_reorient_launch_request(
        thread_id,
        expected_revision,
        max_runtime_seconds,
        state,
        checkpoint,
        &decision,
        load_thread_memory_graph_snapshot(thread).await?.as_ref(),
    );
    let changed_fields = epiphany_job_launch_changed_fields();
    let launched = thread.epiphany_launch_job(launch_request).await?;
    let epiphany_state =
        client_visible_live_thread_epiphany_state(thread, launched.epiphany_state).await;
    let job = map_epiphany_jobs(Some(&epiphany_state), None)
        .into_iter()
        .find(|job| job.id == EPIPHANY_REORIENT_LAUNCH_BINDING_ID)
        .unwrap_or_else(|| {
            epiphany_blocked_state_job(
                EPIPHANY_REORIENT_LAUNCH_BINDING_ID,
                ThreadEpiphanyJobKind::Specialist,
                "reorient-guided checkpoint regather",
                "Launched reorientation worker was not reflected in Epiphany state.",
            )
        });

    Ok(EpiphanyReorientLaunchApplied {
        source: ThreadEpiphanyReorientSource::Live,
        state_status,
        state_revision,
        decision,
        revision: epiphany_state.revision,
        changed_fields,
        epiphany_state,
        job,
    })
}

fn validate_expected_revision(
    expected_revision: Option<u64>,
    actual_revision: u64,
) -> Result<(), CodexErr> {
    if let Some(expected_revision) = expected_revision
        && actual_revision != expected_revision
    {
        return Err(CodexErr::InvalidRequest(format!(
            "epiphany state revision mismatch: expected {expected_revision}, found {actual_revision}"
        )));
    }
    Ok(())
}
