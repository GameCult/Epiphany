use chrono::SecondsFormat;
use chrono::Utc;
use std::path::Path;

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
use epiphany_core::EpiphanyJobInterruptResult;
use epiphany_core::EpiphanyJobLaunchRequest;
use epiphany_core::EpiphanyJobLaunchResult;
use epiphany_core::EpiphanyPromotionInput;
use epiphany_core::EpiphanyStateUpdate;
use epiphany_core::RuntimeSpineHeartbeatJobOptions;
use epiphany_core::RuntimeSpineHeartbeatLaunchPlanOptions;
use epiphany_core::apply_epiphany_state_update;
use epiphany_core::clear_epiphany_job_binding_backend;
use epiphany_core::epiphany_state_update_validation_errors;
use epiphany_core::evaluate_promotion;
use epiphany_core::open_runtime_spine_heartbeat_job;
use epiphany_core::plan_runtime_spine_heartbeat_launch;
use epiphany_core::replace_or_append_epiphany_job_binding;

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
use crate::state::client_visible_epiphany_state_for_paths;
use crate::state::thread_state_mirror_id_from_rollout_path;
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

pub async fn apply_epiphany_state_update_to_thread(
    thread: &CodexThread,
    update: EpiphanyStateUpdate,
) -> Result<EpiphanyThreadState, CodexErr> {
    if update.is_empty() {
        return Err(CodexErr::InvalidRequest(
            "epiphany update patch must contain at least one mutation".to_string(),
        ));
    }

    let mut next_state = thread.epiphany_state().await.unwrap_or_default();
    if let Some(expected_revision) = update.expected_revision
        && next_state.revision != expected_revision
    {
        return Err(CodexErr::InvalidRequest(format!(
            "epiphany state revision mismatch: expected {expected_revision}, found {}",
            next_state.revision
        )));
    }

    let validation_errors = epiphany_state_update_validation_errors(&next_state, &update);
    if !validation_errors.is_empty() {
        return Err(CodexErr::InvalidRequest(format!(
            "invalid epiphany update patch: {}",
            validation_errors.join("; ")
        )));
    }

    let reference_turn_id = thread.epiphany_reference_turn_id().await;
    apply_epiphany_state_update(&mut next_state, update, reference_turn_id);
    thread.epiphany_persist_state(next_state).await
}

pub async fn launch_epiphany_job_on_thread(
    thread: &CodexThread,
    request: EpiphanyJobLaunchRequest,
) -> Result<EpiphanyJobLaunchResult, CodexErr> {
    let current_state = thread.epiphany_state().await.unwrap_or_default();
    validate_expected_revision(request.expected_revision, current_state.revision)?;

    let launcher_job_id = format!("epiphany-heartbeat-launch-{}", Uuid::new_v4());
    let backend_job_id = Uuid::new_v4().to_string();
    let launch_plan = plan_runtime_spine_heartbeat_launch(
        &current_state,
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
            max_runtime_seconds: request.max_runtime_seconds,
            runtime_job_id: backend_job_id.clone(),
        },
    )
    .map_err(|err| CodexErr::InvalidRequest(err.to_string()))?;
    let runtime_store = thread.epiphany_runtime_spine_store_path().await;
    open_epiphany_runtime_spine_job(
        runtime_store.as_path(),
        &current_state,
        &request,
        backend_job_id.as_str(),
    )?;
    let next_job_bindings = replace_or_append_epiphany_job_binding(
        current_state.job_bindings.clone(),
        launch_plan.binding,
    );

    let validation_errors = epiphany_state_update_validation_errors(
        &current_state,
        &EpiphanyStateUpdate {
            job_bindings: Some(next_job_bindings.clone()),
            runtime_links: vec![launch_plan.runtime_link.clone()],
            ..Default::default()
        },
    );
    if !validation_errors.is_empty() {
        return Err(CodexErr::InvalidRequest(format!(
            "invalid Epiphany job launch patch: {}",
            validation_errors.join("; ")
        )));
    }

    let epiphany_state = apply_epiphany_state_update_to_thread(
        thread,
        EpiphanyStateUpdate {
            expected_revision: request.expected_revision,
            job_bindings: Some(next_job_bindings),
            runtime_links: vec![launch_plan.runtime_link],
            ..Default::default()
        },
    )
    .await?;

    Ok(EpiphanyJobLaunchResult {
        epiphany_state,
        binding_id: request.binding_id,
        launcher_job_id,
        backend_job_id,
    })
}

pub async fn interrupt_epiphany_job_on_thread(
    thread: &CodexThread,
    request: EpiphanyJobInterruptRequest,
) -> Result<EpiphanyJobInterruptResult, CodexErr> {
    if request.binding_id.trim().is_empty() {
        return Err(CodexErr::InvalidRequest(
            "epiphany job interrupt binding_id must be non-empty".to_string(),
        ));
    }

    let current_state = thread.epiphany_state().await.unwrap_or_default();
    validate_expected_revision(request.expected_revision, current_state.revision)?;

    let Some(binding_index) = current_state
        .job_bindings
        .iter()
        .position(|binding| binding.id == request.binding_id)
    else {
        return Err(CodexErr::InvalidRequest(format!(
            "epiphany job binding {:?} was not found",
            request.binding_id
        )));
    };
    let interrupted_thread_ids = Vec::new();
    let cancel_requested = false;

    let next_job_bindings = clear_epiphany_job_binding_backend(
        current_state.job_bindings.clone(),
        binding_index,
        "No active heartbeat turn is currently bound; launch explicitly to resume specialist work.",
    );
    let epiphany_state = apply_epiphany_state_update_to_thread(
        thread,
        EpiphanyStateUpdate {
            expected_revision: request.expected_revision,
            job_bindings: Some(next_job_bindings),
            ..Default::default()
        },
    )
    .await?;

    Ok(EpiphanyJobInterruptResult {
        epiphany_state,
        binding_id: request.binding_id,
        cancel_requested,
        interrupted_thread_ids,
    })
}

pub async fn apply_thread_epiphany_update(
    thread: &CodexThread,
    expected_revision: Option<u64>,
    patch: ThreadEpiphanyUpdatePatch,
) -> Result<EpiphanyThreadUpdateApplied, CodexErr> {
    let changed_fields = epiphany_update_patch_changed_fields(&patch);
    let update = state_update_from_thread_patch(expected_revision, patch);
    let epiphany_state = apply_epiphany_state_update_to_thread(thread, update).await?;
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
        graphs: patch.graphs.clone(),
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
    let epiphany_state = apply_epiphany_state_update_to_thread(thread, update).await?;
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

    let runtime_store_path = thread.epiphany_runtime_spine_store_path().await;
    let finding = load_completed_epiphany_role_finding(
        Some(runtime_store_path.as_path()),
        &state,
        role_id,
        binding_id,
    )?;
    let accepted_prefix = epiphany_role_label(role_id);
    let acceptance_update = build_role_acceptance_update(
        role_id,
        binding_id,
        &finding,
        format!("ev-{accepted_prefix}-{}", Uuid::new_v4()),
        format!("obs-{accepted_prefix}-{}", Uuid::new_v4()),
        Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
    )
    .map_err(CodexErr::InvalidRequest)?;

    let accepted_receipt_id = acceptance_update.accepted_receipt_id.clone();
    let accepted_observation_id = acceptance_update.accepted_observation_id.clone();
    let accepted_evidence_id = acceptance_update.accepted_evidence_id.clone();
    let changed_fields = acceptance_update.changed_fields.clone();
    let applied_patch = acceptance_update.patch.clone();
    let epiphany_state = apply_epiphany_state_update_to_thread(
        thread,
        state_update_from_thread_patch(expected_revision, acceptance_update.patch),
    )
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
    })
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

    let runtime_store_path = thread.epiphany_runtime_spine_store_path().await;
    let finding = load_completed_epiphany_reorient_finding(
        Some(runtime_store_path.as_path()),
        &state,
        binding_id,
    )?;
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
    let epiphany_state =
        apply_epiphany_state_update_to_thread(thread, acceptance_update.state_update).await?;
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
    let launch_request = build_epiphany_role_launch_request(
        thread_id,
        role_id,
        expected_revision,
        max_runtime_seconds,
        &state,
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
    let launched = launch_epiphany_job_on_thread(thread, launch_request).await?;
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
    let interrupted = interrupt_epiphany_job_on_thread(
        thread,
        EpiphanyJobInterruptRequest {
            expected_revision,
            binding_id: binding_id.to_string(),
            reason,
        },
    )
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
    );
    let changed_fields = epiphany_job_launch_changed_fields();
    let launched = launch_epiphany_job_on_thread(thread, launch_request).await?;
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

fn open_epiphany_runtime_spine_job(
    store_path: &Path,
    state: &EpiphanyThreadState,
    request: &EpiphanyJobLaunchRequest,
    backend_job_id: &str,
) -> Result<(), CodexErr> {
    let now = Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true);
    open_runtime_spine_heartbeat_job(
        store_path,
        RuntimeSpineHeartbeatJobOptions {
            runtime_id: "epiphany-local".to_string(),
            display_name: "Epiphany Local".to_string(),
            session_id: "epiphany-main".to_string(),
            objective: state
                .objective
                .clone()
                .filter(|objective| !objective.trim().is_empty())
                .unwrap_or_else(|| "Epiphany heartbeat activation".to_string()),
            coordinator_note: "App-server launch opened this typed runtime session.".to_string(),
            job_id: backend_job_id.to_string(),
            role: request.owner_role.clone(),
            binding_id: request.binding_id.clone(),
            authority_scope: request.authority_scope.clone(),
            instruction: request.instruction.clone(),
            launch_document: request.launch_document.clone(),
            output_contract_id: request.output_contract_id.clone(),
            created_at: now,
        },
    )
    .map_err(|err| {
        CodexErr::Fatal(format!(
            "failed to open Epiphany runtime spine job {:?} in {}: {err}",
            backend_job_id,
            store_path.display()
        ))
    })?;
    Ok(())
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

async fn client_visible_live_thread_epiphany_state(
    thread: &CodexThread,
    fallback: EpiphanyThreadState,
) -> EpiphanyThreadState {
    let config = thread.config_snapshot().await;
    let codex_home = thread.codex_home().await;
    let rollout_path = thread.rollout_path();
    let mirror_thread_id = thread_state_mirror_id_from_rollout_path(rollout_path.as_deref());
    client_visible_epiphany_state_for_paths(
        fallback,
        config.cwd.as_path(),
        codex_home,
        Some(mirror_thread_id.as_str()),
    )
    .await
}
