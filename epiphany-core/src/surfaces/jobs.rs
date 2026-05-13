use epiphany_state_model::EpiphanyJobBinding;
use epiphany_state_model::EpiphanyJobKind;
use epiphany_state_model::EpiphanyRetrievalState;
use epiphany_state_model::EpiphanyRetrievalStatus;
use epiphany_state_model::EpiphanyRuntimeLink;
use epiphany_state_model::EpiphanyThreadState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyJobStatus {
    Idle,
    Needed,
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Blocked,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyJobView {
    pub id: String,
    pub kind: EpiphanyJobKind,
    pub scope: String,
    pub owner_role: String,
    pub authority_scope: Option<String>,
    pub runtime_job_id: Option<String>,
    pub status: EpiphanyJobStatus,
    pub items_processed: Option<u32>,
    pub items_total: Option<u32>,
    pub progress_note: Option<String>,
    pub last_checkpoint_at_unix_seconds: Option<i64>,
    pub blocking_reason: Option<String>,
    pub active_thread_ids: Vec<String>,
    pub linked_subgoal_ids: Vec<String>,
    pub linked_graph_node_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct EpiphanyJobsInput<'a> {
    pub state: Option<&'a EpiphanyThreadState>,
    pub retrieval_override: Option<&'a EpiphanyRetrievalState>,
}

pub fn derive_jobs(input: EpiphanyJobsInput<'_>) -> Vec<EpiphanyJobView> {
    let mut jobs = vec![
        index_job(input.state, input.retrieval_override),
        remap_job(input.state),
        verification_job(input.state),
    ];

    let Some(state) = input.state else {
        return jobs;
    };

    for binding in &state.job_bindings {
        let runtime_link = latest_runtime_link_for_binding(state, binding.id.as_str());
        let replacement = if let Some(existing) = jobs.iter().find(|job| job.id == binding.id) {
            overlay_job_binding(existing.clone(), binding, runtime_link)
        } else {
            bound_job(binding, runtime_link)
        };

        if let Some(existing) = jobs.iter_mut().find(|job| job.id == binding.id) {
            *existing = replacement;
        } else {
            jobs.push(replacement);
        }
    }

    jobs
}

fn bound_job(
    binding: &EpiphanyJobBinding,
    runtime_link: Option<&EpiphanyRuntimeLink>,
) -> EpiphanyJobView {
    overlay_job_binding(
        EpiphanyJobView {
            id: binding.id.clone(),
            kind: binding.kind,
            scope: binding.scope.clone(),
            owner_role: binding.owner_role.clone(),
            authority_scope: binding.authority_scope.clone(),
            runtime_job_id: runtime_link.map(|link| link.runtime_job_id.clone()),
            status: if binding.blocking_reason.is_some() {
                EpiphanyJobStatus::Blocked
            } else {
                EpiphanyJobStatus::Idle
            },
            items_processed: None,
            items_total: None,
            progress_note: None,
            last_checkpoint_at_unix_seconds: None,
            blocking_reason: None,
            active_thread_ids: Vec::new(),
            linked_subgoal_ids: binding.linked_subgoal_ids.clone(),
            linked_graph_node_ids: binding.linked_graph_node_ids.clone(),
        },
        binding,
        runtime_link,
    )
}

fn overlay_job_binding(
    mut job: EpiphanyJobView,
    binding: &EpiphanyJobBinding,
    runtime_link: Option<&EpiphanyRuntimeLink>,
) -> EpiphanyJobView {
    job.kind = binding.kind;
    job.scope = binding.scope.clone();
    job.owner_role = binding.owner_role.clone();
    job.authority_scope = binding.authority_scope.clone();
    job.runtime_job_id = runtime_link.map(|link| link.runtime_job_id.clone());
    if !binding.linked_subgoal_ids.is_empty() {
        job.linked_subgoal_ids = binding.linked_subgoal_ids.clone();
    }
    if !binding.linked_graph_node_ids.is_empty() {
        job.linked_graph_node_ids = binding.linked_graph_node_ids.clone();
    }
    if let Some(blocking_reason) = binding.blocking_reason.clone() {
        job.blocking_reason = Some(blocking_reason);
    }
    if binding.blocking_reason.is_some() {
        job.status = EpiphanyJobStatus::Blocked;
        return job;
    }

    if runtime_link.is_some() {
        job.status = EpiphanyJobStatus::Pending;
        job.blocking_reason = None;
        job.progress_note =
            Some("Queued for Epiphany heartbeat activation through runtime_links.".to_string());
        return job;
    }

    job
}

fn index_job(
    state: Option<&EpiphanyThreadState>,
    retrieval_override: Option<&EpiphanyRetrievalState>,
) -> EpiphanyJobView {
    let retrieval = state
        .and_then(|state| state.retrieval.as_ref())
        .or(retrieval_override);
    let linked_subgoal_ids = active_subgoal_ids(state);
    let linked_graph_node_ids = active_graph_node_ids(state);

    let Some(retrieval) = retrieval else {
        return EpiphanyJobView {
            id: "retrieval-index".to_string(),
            kind: EpiphanyJobKind::Indexing,
            scope: "workspace".to_string(),
            owner_role: "epiphany-core".to_string(),
            authority_scope: None,
            runtime_job_id: None,
            status: EpiphanyJobStatus::Unavailable,
            items_processed: None,
            items_total: None,
            progress_note: None,
            last_checkpoint_at_unix_seconds: None,
            blocking_reason: Some("Retrieval state is unavailable for this thread.".to_string()),
            active_thread_ids: Vec::new(),
            linked_subgoal_ids,
            linked_graph_node_ids,
        };
    };

    let dirty_path_count = retrieval.dirty_paths.len();
    let progress_note = match retrieval.status {
        EpiphanyRetrievalStatus::Ready if dirty_path_count == 0 => {
            "Retrieval catalog is ready.".to_string()
        }
        EpiphanyRetrievalStatus::Ready => {
            format!("Retrieval catalog is ready with {dirty_path_count} dirty path(s) noted.")
        }
        EpiphanyRetrievalStatus::Stale => {
            format!("Retrieval catalog is stale; {dirty_path_count} dirty path(s) need refresh.")
        }
        EpiphanyRetrievalStatus::Indexing => "Retrieval catalog is indexing.".to_string(),
        EpiphanyRetrievalStatus::Unavailable => "Retrieval catalog is unavailable.".to_string(),
    };

    EpiphanyJobView {
        id: "retrieval-index".to_string(),
        kind: EpiphanyJobKind::Indexing,
        scope: retrieval.workspace_root.display().to_string(),
        owner_role: "epiphany-core".to_string(),
        authority_scope: None,
        runtime_job_id: None,
        status: job_status_from_retrieval_status(retrieval.status),
        items_processed: retrieval.indexed_file_count,
        items_total: None,
        progress_note: Some(progress_note),
        last_checkpoint_at_unix_seconds: retrieval.last_indexed_at_unix_seconds,
        blocking_reason: (retrieval.status == EpiphanyRetrievalStatus::Unavailable).then(|| {
            "Indexing requires a readable workspace and configured retrieval backend.".to_string()
        }),
        active_thread_ids: Vec::new(),
        linked_subgoal_ids,
        linked_graph_node_ids,
    }
}

fn job_status_from_retrieval_status(status: EpiphanyRetrievalStatus) -> EpiphanyJobStatus {
    match status {
        EpiphanyRetrievalStatus::Ready => EpiphanyJobStatus::Idle,
        EpiphanyRetrievalStatus::Stale => EpiphanyJobStatus::Needed,
        EpiphanyRetrievalStatus::Indexing => EpiphanyJobStatus::Running,
        EpiphanyRetrievalStatus::Unavailable => EpiphanyJobStatus::Unavailable,
    }
}

fn remap_job(state: Option<&EpiphanyThreadState>) -> EpiphanyJobView {
    let Some(state) = state else {
        return blocked_state_job(
            "graph-remap",
            EpiphanyJobKind::Remap,
            "architecture/dataflow graphs",
            "Epiphany state is missing, so there is no graph to remap.",
        );
    };

    let frontier = state.graph_frontier.as_ref();
    let dirty_path_count = frontier
        .map(|frontier| frontier.dirty_paths.len())
        .unwrap_or_default();
    let open_count = frontier
        .map(|frontier| frontier.open_question_ids.len() + frontier.open_gap_ids.len())
        .unwrap_or_default();
    let graph_freshness = state
        .churn
        .as_ref()
        .and_then(|churn| churn.graph_freshness.as_deref());
    let freshness_needs_work = graph_freshness
        .is_some_and(|freshness| !matches!(freshness, "fresh" | "ready" | "current" | "ok"));
    let needs_work = dirty_path_count > 0 || open_count > 0 || freshness_needs_work;
    let progress_note = if needs_work {
        format!(
            "Graph frontier has {dirty_path_count} dirty path(s) and {open_count} open question/gap id(s)."
        )
    } else {
        "Graph frontier has no reflected remap pressure.".to_string()
    };

    EpiphanyJobView {
        id: "graph-remap".to_string(),
        kind: EpiphanyJobKind::Remap,
        scope: "architecture/dataflow graphs".to_string(),
        owner_role: "epiphany-core".to_string(),
        authority_scope: None,
        runtime_job_id: None,
        status: if needs_work {
            EpiphanyJobStatus::Needed
        } else {
            EpiphanyJobStatus::Idle
        },
        items_processed: None,
        items_total: None,
        progress_note: Some(progress_note),
        last_checkpoint_at_unix_seconds: None,
        blocking_reason: None,
        active_thread_ids: Vec::new(),
        linked_subgoal_ids: active_subgoal_ids(Some(state)),
        linked_graph_node_ids: active_graph_node_ids(Some(state)),
    }
}

fn verification_job(state: Option<&EpiphanyThreadState>) -> EpiphanyJobView {
    let Some(state) = state else {
        return blocked_state_job(
            "verification",
            EpiphanyJobKind::Verification,
            "invariants/evidence",
            "Epiphany state is missing, so there are no invariants to verify.",
        );
    };

    let total = state.invariants.len() as u32;
    let verified = state
        .invariants
        .iter()
        .filter(|invariant| invariant_status_is_accepting(&invariant.status))
        .count() as u32;
    let status = if total > 0 && verified < total {
        EpiphanyJobStatus::Needed
    } else {
        EpiphanyJobStatus::Idle
    };
    let progress_note = if total == 0 {
        "No invariants are recorded yet.".to_string()
    } else if verified == total {
        format!("All {total} invariant(s) are currently accepting.")
    } else {
        format!("{verified} of {total} invariant(s) are currently accepting.")
    };

    EpiphanyJobView {
        id: "verification".to_string(),
        kind: EpiphanyJobKind::Verification,
        scope: "invariants/evidence".to_string(),
        owner_role: "epiphany-harness".to_string(),
        authority_scope: None,
        runtime_job_id: None,
        status,
        items_processed: Some(verified),
        items_total: Some(total),
        progress_note: Some(progress_note),
        last_checkpoint_at_unix_seconds: None,
        blocking_reason: None,
        active_thread_ids: Vec::new(),
        linked_subgoal_ids: active_subgoal_ids(Some(state)),
        linked_graph_node_ids: active_graph_node_ids(Some(state)),
    }
}

fn blocked_state_job(
    id: &str,
    kind: EpiphanyJobKind,
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
        status: EpiphanyJobStatus::Blocked,
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

fn active_subgoal_ids(state: Option<&EpiphanyThreadState>) -> Vec<String> {
    state
        .and_then(|state| state.active_subgoal_id.clone())
        .map(|id| vec![id])
        .unwrap_or_default()
}

fn active_graph_node_ids(state: Option<&EpiphanyThreadState>) -> Vec<String> {
    state
        .and_then(|state| state.graph_frontier.as_ref())
        .map(|frontier| frontier.active_node_ids.clone())
        .unwrap_or_default()
}

fn invariant_status_is_accepting(status: &str) -> bool {
    matches!(
        status,
        "ok" | "ready" | "accepted" | "verified" | "pass" | "passed"
    )
}

fn latest_runtime_link_for_binding<'a>(
    state: &'a EpiphanyThreadState,
    binding_id: &str,
) -> Option<&'a EpiphanyRuntimeLink> {
    state
        .runtime_links
        .iter()
        .find(|link| link.binding_id == binding_id && !link.runtime_job_id.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_state_model::EpiphanyGraphFrontier;
    use epiphany_state_model::EpiphanyInvariant;
    use std::path::PathBuf;

    #[test]
    fn derives_builtin_jobs_without_state() {
        let jobs = derive_jobs(EpiphanyJobsInput {
            state: None,
            retrieval_override: None,
        });

        assert_eq!(jobs.len(), 3);
        assert_eq!(jobs[0].id, "retrieval-index");
        assert_eq!(jobs[0].status, EpiphanyJobStatus::Unavailable);
        assert_eq!(jobs[1].status, EpiphanyJobStatus::Blocked);
        assert_eq!(jobs[2].status, EpiphanyJobStatus::Blocked);
    }

    #[test]
    fn reflects_retrieval_and_graph_pressure() {
        let state = EpiphanyThreadState {
            active_subgoal_id: Some("subgoal-1".to_string()),
            retrieval: Some(EpiphanyRetrievalState {
                workspace_root: PathBuf::from("E:/repo"),
                status: EpiphanyRetrievalStatus::Stale,
                dirty_paths: vec![PathBuf::from("src/lib.rs")],
                indexed_file_count: Some(7),
                ..Default::default()
            }),
            graph_frontier: Some(EpiphanyGraphFrontier {
                active_node_ids: vec!["node-1".to_string()],
                dirty_paths: vec![PathBuf::from("src/lib.rs")],
                open_question_ids: vec!["q-1".to_string()],
                ..Default::default()
            }),
            invariants: vec![
                EpiphanyInvariant {
                    id: "inv-1".to_string(),
                    description: "verified".to_string(),
                    status: "verified".to_string(),
                    rationale: None,
                },
                EpiphanyInvariant {
                    id: "inv-2".to_string(),
                    description: "pending".to_string(),
                    status: "pending".to_string(),
                    rationale: None,
                },
            ],
            ..Default::default()
        };

        let jobs = derive_jobs(EpiphanyJobsInput {
            state: Some(&state),
            retrieval_override: None,
        });

        let retrieval = jobs.iter().find(|job| job.id == "retrieval-index").unwrap();
        assert_eq!(retrieval.status, EpiphanyJobStatus::Needed);
        assert_eq!(retrieval.items_processed, Some(7));
        assert_eq!(retrieval.linked_subgoal_ids, vec!["subgoal-1".to_string()]);

        let remap = jobs.iter().find(|job| job.id == "graph-remap").unwrap();
        assert_eq!(remap.status, EpiphanyJobStatus::Needed);
        assert_eq!(remap.linked_graph_node_ids, vec!["node-1".to_string()]);

        let verification = jobs.iter().find(|job| job.id == "verification").unwrap();
        assert_eq!(verification.status, EpiphanyJobStatus::Needed);
        assert_eq!(verification.items_processed, Some(1));
        assert_eq!(verification.items_total, Some(2));
    }

    #[test]
    fn overlays_binding_with_runtime_link() {
        let state = EpiphanyThreadState {
            job_bindings: vec![EpiphanyJobBinding {
                id: "binding-1".to_string(),
                kind: EpiphanyJobKind::Specialist,
                scope: "modeling".to_string(),
                owner_role: "body".to_string(),
                authority_scope: Some("epiphany.role.modeling".to_string()),
                linked_subgoal_ids: vec!["subgoal-1".to_string()],
                linked_graph_node_ids: vec!["node-1".to_string()],
                blocking_reason: None,
            }],
            runtime_links: vec![EpiphanyRuntimeLink {
                id: "link-1".to_string(),
                binding_id: "binding-1".to_string(),
                surface: "role".to_string(),
                role_id: "modeling".to_string(),
                authority_scope: "epiphany.role.modeling".to_string(),
                runtime_job_id: "runtime-job-1".to_string(),
                runtime_result_id: None,
                linked_subgoal_ids: Vec::new(),
                linked_graph_node_ids: Vec::new(),
            }],
            ..Default::default()
        };

        let jobs = derive_jobs(EpiphanyJobsInput {
            state: Some(&state),
            retrieval_override: None,
        });

        let binding = jobs.iter().find(|job| job.id == "binding-1").unwrap();
        assert_eq!(binding.status, EpiphanyJobStatus::Pending);
        assert_eq!(binding.runtime_job_id.as_deref(), Some("runtime-job-1"));
        assert_eq!(
            binding.progress_note.as_deref(),
            Some("Queued for Epiphany heartbeat activation through runtime_links.")
        );
    }
}
