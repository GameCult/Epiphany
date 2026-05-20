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
