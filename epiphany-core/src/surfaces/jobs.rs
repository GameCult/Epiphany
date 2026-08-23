use epiphany_state_model::EpiphanyJobKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyJobView {
    pub id: String,
    pub kind: EpiphanyJobKind,
    pub scope: String,
    pub owner_role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority_scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_job_id: Option<String>,
    pub status: EpiphanyJobStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items_processed: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items_total: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub progress_note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_checkpoint_at_unix_seconds: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocking_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_thread_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub linked_subgoal_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub linked_graph_node_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct EpiphanyJobsInput<'a> {
    pub mind: Option<&'a crate::EpiphanyMindView>,
}

pub fn derive_jobs(input: EpiphanyJobsInput<'_>) -> Vec<EpiphanyJobView> {
    vec![
        body_observation_job(input.mind),
        repo_model_job(input.mind),
        verification_job(input.mind),
    ]
}

fn base_job(
    id: &str,
    kind: EpiphanyJobKind,
    scope: &str,
    owner_role: &str,
    status: EpiphanyJobStatus,
) -> EpiphanyJobView {
    EpiphanyJobView {
        id: id.into(),
        kind,
        scope: scope.into(),
        owner_role: owner_role.into(),
        authority_scope: None,
        runtime_job_id: None,
        status,
        items_processed: None,
        items_total: None,
        progress_note: None,
        last_checkpoint_at_unix_seconds: None,
        blocking_reason: None,
        active_thread_ids: Vec::new(),
        linked_subgoal_ids: Vec::new(),
        linked_graph_node_ids: Vec::new(),
    }
}

fn body_observation_job(mind: Option<&crate::EpiphanyMindView>) -> EpiphanyJobView {
    let body = mind.and_then(|mind| mind.repository_body_observation.as_ref());
    let mut job = base_job(
        "repository-body-observation",
        EpiphanyJobKind::Indexing,
        "repository Body",
        "epiphany-body-observer",
        if body.is_some() {
            EpiphanyJobStatus::Completed
        } else {
            EpiphanyJobStatus::Needed
        },
    );
    job.linked_subgoal_ids = active_subgoal_ids(mind);
    job.progress_note = Some(match body {
        Some(body) => format!(
            "Mind contains exact Body observation {} at generation {}.",
            body.observation_id, body.generation
        ),
        None => "Mind has no admitted repository Body observation.".into(),
    });
    job
}

fn repo_model_job(mind: Option<&crate::EpiphanyMindView>) -> EpiphanyJobView {
    let model = mind.and_then(|mind| mind.repo_model.as_ref());
    let mut job = base_job(
        "repo-model",
        EpiphanyJobKind::Remap,
        "keyed RepoModel",
        "epiphany-modeling",
        if model.is_some() {
            EpiphanyJobStatus::Idle
        } else {
            EpiphanyJobStatus::Blocked
        },
    );
    job.linked_subgoal_ids = active_subgoal_ids(mind);
    job.linked_graph_node_ids = active_graph_node_ids(mind);
    job.progress_note = model.map(|model| {
        format!(
            "Keyed RepoModel projects {} node(s), {} edge(s), and {} frontier item(s).",
            model.nodes.len(),
            model.edges.len(),
            model.frontier.len()
        )
    });
    if model.is_none() {
        job.blocking_reason = Some("Mind has no keyed RepoModel identity.".into());
    }
    job
}

fn verification_job(mind: Option<&crate::EpiphanyMindView>) -> EpiphanyJobView {
    let Some(mind) = mind else {
        let mut job = base_job(
            "verification",
            EpiphanyJobKind::Verification,
            "invariants/evidence",
            "epiphany-soul",
            EpiphanyJobStatus::Blocked,
        );
        job.blocking_reason = Some("Mind is missing.".into());
        return job;
    };
    let total = mind.invariants.len() as u32;
    let verified = mind
        .invariants
        .iter()
        .filter(|invariant| invariant_status_is_accepting(&invariant.status))
        .count() as u32;
    let mut job = base_job(
        "verification",
        EpiphanyJobKind::Verification,
        "invariants/evidence",
        "epiphany-soul",
        if total > verified {
            EpiphanyJobStatus::Needed
        } else {
            EpiphanyJobStatus::Idle
        },
    );
    job.items_processed = Some(verified);
    job.items_total = Some(total);
    job.linked_subgoal_ids = active_subgoal_ids(Some(mind));
    job.linked_graph_node_ids = active_graph_node_ids(Some(mind));
    job.progress_note = Some(format!("{verified} of {total} invariant(s) are accepting."));
    job
}

fn active_subgoal_ids(mind: Option<&crate::EpiphanyMindView>) -> Vec<String> {
    mind.and_then(|mind| mind.active_subgoal_id.clone())
        .map(|id| vec![id])
        .unwrap_or_default()
}

fn active_graph_node_ids(mind: Option<&crate::EpiphanyMindView>) -> Vec<String> {
    let mut ids = mind
        .and_then(|mind| mind.repo_model.as_ref())
        .into_iter()
        .flat_map(|model| model.frontier.iter())
        .filter(|item| {
            matches!(
                item.status,
                epiphany_state_model::RepoFrontierStatus::Proposed
                    | epiphany_state_model::RepoFrontierStatus::Active
                    | epiphany_state_model::RepoFrontierStatus::Blocked
            )
        })
        .flat_map(|item| item.target_claim_ids.iter().cloned())
        .collect::<Vec<_>>();
    ids.sort();
    ids.dedup();
    ids
}

fn invariant_status_is_accepting(status: &str) -> bool {
    matches!(
        status,
        "ok" | "ready" | "accepted" | "verified" | "pass" | "passed"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_mind_has_only_derived_blocked_or_needed_jobs() {
        let jobs = derive_jobs(EpiphanyJobsInput { mind: None });
        assert_eq!(jobs.len(), 3);
        assert_eq!(jobs[0].status, EpiphanyJobStatus::Needed);
        assert_eq!(jobs[1].status, EpiphanyJobStatus::Blocked);
        assert_eq!(jobs[2].status, EpiphanyJobStatus::Blocked);
        assert!(jobs.iter().all(|job| job.runtime_job_id.is_none()));
    }
}
