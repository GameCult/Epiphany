use codex_app_server_protocol::ThreadEpiphanyJob;
use codex_app_server_protocol::ThreadEpiphanyJobKind;
use codex_app_server_protocol::ThreadEpiphanyJobStatus;
use epiphany_core::EpiphanyJobStatus as CoreEpiphanyJobStatus;
use epiphany_core::EpiphanyJobView;
use epiphany_core::EpiphanyJobsInput;
use epiphany_core::derive_jobs;
use epiphany_state_model::EpiphanyJobKind as CoreEpiphanyJobKind;
use epiphany_state_model::EpiphanyRetrievalState;
use epiphany_state_model::EpiphanyThreadState;

pub fn derive_epiphany_jobs(
    state: Option<&EpiphanyThreadState>,
    retrieval_override: Option<&EpiphanyRetrievalState>,
) -> Vec<EpiphanyJobView> {
    derive_jobs(EpiphanyJobsInput {
        state,
        retrieval_override,
    })
}

pub fn map_epiphany_jobs(
    state: Option<&EpiphanyThreadState>,
    retrieval_override: Option<&EpiphanyRetrievalState>,
) -> Vec<EpiphanyJobView> {
    derive_epiphany_jobs(state, retrieval_override)
}

pub fn map_protocol_epiphany_jobs(
    state: Option<&EpiphanyThreadState>,
    retrieval_override: Option<&EpiphanyRetrievalState>,
) -> Vec<ThreadEpiphanyJob> {
    derive_epiphany_jobs(state, retrieval_override)
        .into_iter()
        .map(map_core_epiphany_job_view)
        .collect()
}

pub fn map_core_epiphany_job_view(job: EpiphanyJobView) -> ThreadEpiphanyJob {
    ThreadEpiphanyJob {
        id: job.id,
        kind: map_core_epiphany_job_kind(job.kind),
        scope: job.scope,
        owner_role: job.owner_role,
        launcher_job_id: None,
        authority_scope: job.authority_scope,
        backend_job_id: job.runtime_job_id,
        status: map_core_epiphany_job_status(job.status),
        items_processed: job.items_processed,
        items_total: job.items_total,
        progress_note: job.progress_note,
        last_checkpoint_at_unix_seconds: job.last_checkpoint_at_unix_seconds,
        blocking_reason: job.blocking_reason,
        active_thread_ids: job.active_thread_ids,
        linked_subgoal_ids: job.linked_subgoal_ids,
        linked_graph_node_ids: job.linked_graph_node_ids,
    }
}

pub fn map_core_epiphany_job_kind(kind: CoreEpiphanyJobKind) -> ThreadEpiphanyJobKind {
    match kind {
        CoreEpiphanyJobKind::Indexing => ThreadEpiphanyJobKind::Indexing,
        CoreEpiphanyJobKind::Remap => ThreadEpiphanyJobKind::Remap,
        CoreEpiphanyJobKind::Verification => ThreadEpiphanyJobKind::Verification,
        CoreEpiphanyJobKind::Specialist => ThreadEpiphanyJobKind::Specialist,
    }
}

pub fn map_launched_epiphany_job(
    state: &EpiphanyThreadState,
    binding_id: &str,
    backend_job_id: &str,
    fallback_kind: CoreEpiphanyJobKind,
    fallback_scope: &str,
) -> EpiphanyJobView {
    derive_epiphany_jobs(Some(state), None)
        .into_iter()
        .find(|job| job.id == binding_id)
        .unwrap_or_else(|| EpiphanyJobView {
            id: binding_id.to_string(),
            kind: fallback_kind,
            scope: fallback_scope.to_string(),
            owner_role: "epiphany-harness".to_string(),
            authority_scope: None,
            runtime_job_id: Some(backend_job_id.to_string()),
            status: CoreEpiphanyJobStatus::Pending,
            items_processed: None,
            items_total: None,
            progress_note: None,
            last_checkpoint_at_unix_seconds: None,
            blocking_reason: None,
            active_thread_ids: Vec::new(),
            linked_subgoal_ids: Vec::new(),
            linked_graph_node_ids: Vec::new(),
        })
}

pub fn map_interrupted_epiphany_job(
    state: &EpiphanyThreadState,
    binding_id: &str,
) -> EpiphanyJobView {
    derive_epiphany_jobs(Some(state), None)
        .into_iter()
        .find(|job| job.id == binding_id)
        .unwrap_or_else(|| {
            epiphany_blocked_state_job(
                binding_id,
                CoreEpiphanyJobKind::Specialist,
                "role-scoped specialist work",
                "Interrupted job binding was not reflected in Epiphany state.",
            )
        })
}

fn map_core_epiphany_job_status(status: CoreEpiphanyJobStatus) -> ThreadEpiphanyJobStatus {
    match status {
        CoreEpiphanyJobStatus::Idle => ThreadEpiphanyJobStatus::Idle,
        CoreEpiphanyJobStatus::Needed => ThreadEpiphanyJobStatus::Needed,
        CoreEpiphanyJobStatus::Pending => ThreadEpiphanyJobStatus::Pending,
        CoreEpiphanyJobStatus::Running => ThreadEpiphanyJobStatus::Running,
        CoreEpiphanyJobStatus::Completed => ThreadEpiphanyJobStatus::Completed,
        CoreEpiphanyJobStatus::Failed => ThreadEpiphanyJobStatus::Failed,
        CoreEpiphanyJobStatus::Cancelled => ThreadEpiphanyJobStatus::Cancelled,
        CoreEpiphanyJobStatus::Blocked => ThreadEpiphanyJobStatus::Blocked,
        CoreEpiphanyJobStatus::Unavailable => ThreadEpiphanyJobStatus::Unavailable,
    }
}

pub fn epiphany_blocked_state_job(
    id: &str,
    kind: CoreEpiphanyJobKind,
    scope: &str,
    blocking_reason: &str,
) -> EpiphanyJobView {
    EpiphanyJobView {
        id: id.to_string(),
        kind,
        scope: scope.to_string(),
        owner_role: "epiphany-harness".to_string(),
        authority_scope: None,
        runtime_job_id: None,
        status: CoreEpiphanyJobStatus::Blocked,
        items_processed: None,
        items_total: None,
        progress_note: None,
        last_checkpoint_at_unix_seconds: None,
        blocking_reason: Some(blocking_reason.to_string()),
        active_thread_ids: Vec::new(),
        linked_subgoal_ids: Vec::new(),
        linked_graph_node_ids: Vec::new(),
    }
}
