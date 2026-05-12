use super::EpiphanyCoordinatorRoleId;
use super::EpiphanyCoordinatorRoleStatus;
use super::EpiphanyCoordinatorSceneAction;
use super::EpiphanyCrrcAction;
use super::EpiphanyCrrcResultStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyRoleBoardJobStatus {
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
pub struct EpiphanyRoleBoardJob {
    pub id: String,
    pub owner_role: String,
    pub status: EpiphanyRoleBoardJobStatus,
    pub progress_note: Option<String>,
    pub blocking_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyRoleBoardPlanningSummary {
    pub capture_count: usize,
    pub backlog_item_count: usize,
    pub roadmap_stream_count: usize,
    pub objective_draft_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyRoleBoardCheckpointSummary {
    pub disposition: Option<String>,
    pub next_action: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyRoleBoardInput {
    pub state_present: bool,
    pub planning: EpiphanyRoleBoardPlanningSummary,
    pub checkpoint: Option<EpiphanyRoleBoardCheckpointSummary>,
    pub reorient_next_action: String,
    pub jobs: Vec<EpiphanyRoleBoardJob>,
    pub crrc_action: EpiphanyCrrcAction,
    pub crrc_recommended_scene_action: Option<EpiphanyCoordinatorSceneAction>,
    pub crrc_reason: String,
    pub reorient_decision_action: String,
    pub pressure_level: String,
    pub reorient_result_status: EpiphanyCrrcResultStatus,
    pub reorient_job: Option<EpiphanyRoleBoardJob>,
    pub imagination_binding_id: String,
    pub modeling_binding_id: String,
    pub verification_binding_id: String,
    pub reorient_owner_role: String,
    pub imagination_owner_role: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyRoleBoardLane {
    pub id: EpiphanyCoordinatorRoleId,
    pub title: String,
    pub owner_role: String,
    pub status: EpiphanyCoordinatorRoleStatus,
    pub note: String,
    pub jobs: Vec<EpiphanyRoleBoardJob>,
    pub authority_scopes: Vec<String>,
    pub recommended_action: Option<EpiphanyCoordinatorSceneAction>,
}

pub fn derive_role_board(input: EpiphanyRoleBoardInput) -> Vec<EpiphanyRoleBoardLane> {
    let state_present = input.state_present;
    let checkpoint_present = input.checkpoint.is_some();
    let imagination_jobs = input
        .jobs
        .iter()
        .filter(|job| job.id == input.imagination_binding_id)
        .cloned()
        .collect::<Vec<_>>();
    let imagination_bound_job = imagination_jobs
        .iter()
        .find(|job| job.id == input.imagination_binding_id);
    let imagination_has_bound_job = imagination_bound_job.is_some();
    let imagination_status = imagination_bound_job
        .map(|job| role_board_job_status_to_role_status(job.status))
        .unwrap_or_else(|| imagination_role_status(state_present, &input.planning));
    let imagination_note = imagination_bound_job
        .and_then(job_note)
        .unwrap_or_else(|| render_imagination_role_note(state_present, &input.planning));
    let imagination_recommended_action = if imagination_has_bound_job {
        Some(EpiphanyCoordinatorSceneAction::RoleResult)
    } else if state_present {
        Some(EpiphanyCoordinatorSceneAction::RoleLaunch)
    } else {
        Some(EpiphanyCoordinatorSceneAction::Update)
    };

    let modeling_jobs = input
        .jobs
        .iter()
        .filter(|job| job.id == input.modeling_binding_id || job.id == "graph-remap")
        .cloned()
        .collect::<Vec<_>>();
    let modeling_bound_job = modeling_jobs
        .iter()
        .find(|job| job.id == input.modeling_binding_id);
    let modeling_has_bound_job = modeling_bound_job.is_some();
    let modeling_status = modeling_role_status(
        checkpoint_present,
        modeling_bound_job.or_else(|| modeling_jobs.first()),
    );
    let modeling_recommended_action = if modeling_has_bound_job {
        Some(EpiphanyCoordinatorSceneAction::RoleResult)
    } else if state_present {
        Some(EpiphanyCoordinatorSceneAction::RoleLaunch)
    } else {
        Some(EpiphanyCoordinatorSceneAction::Update)
    };

    let verification_jobs = input
        .jobs
        .iter()
        .filter(|job| job.id == input.verification_binding_id || job.id == "verification")
        .cloned()
        .collect::<Vec<_>>();
    let verification_bound_job = verification_jobs
        .iter()
        .find(|job| job.id == input.verification_binding_id);
    let verification_has_bound_job = verification_bound_job.is_some();
    let verification_primary_job = verification_bound_job.or_else(|| verification_jobs.first());
    let verification_status = verification_primary_job
        .map(|job| role_board_job_status_to_role_status(job.status))
        .unwrap_or(EpiphanyCoordinatorRoleStatus::Unavailable);
    let verification_note = verification_primary_job
        .and_then(job_note)
        .unwrap_or_else(|| {
            "Review evidence, worker findings, and verifier results before promotion.".to_string()
        });
    let verification_recommended_action = if verification_has_bound_job {
        Some(EpiphanyCoordinatorSceneAction::RoleResult)
    } else if state_present {
        Some(EpiphanyCoordinatorSceneAction::RoleLaunch)
    } else {
        None
    };
    let reorientation_jobs = input.reorient_job.clone().into_iter().collect::<Vec<_>>();

    vec![
        EpiphanyRoleBoardLane {
            id: EpiphanyCoordinatorRoleId::Implementation,
            title: "Implementation".to_string(),
            owner_role: "coding-agent".to_string(),
            status: if input.crrc_action == EpiphanyCrrcAction::Continue {
                EpiphanyCoordinatorRoleStatus::Ready
            } else {
                EpiphanyCoordinatorRoleStatus::Blocked
            },
            note: if input.crrc_action == EpiphanyCrrcAction::Continue {
                "Continue the bounded coding task.".to_string()
            } else {
                format!("Wait for CRRC action: {:?}.", input.crrc_action)
            },
            jobs: input
                .jobs
                .iter()
                .filter(|job| {
                    matches!(
                        job.owner_role.as_str(),
                        "coding-agent" | "implementation" | "epiphany-implementation"
                    )
                })
                .cloned()
                .collect(),
            authority_scopes: Vec::new(),
            recommended_action: if input.crrc_action == EpiphanyCrrcAction::Continue {
                None
            } else {
                input.crrc_recommended_scene_action
            },
        },
        EpiphanyRoleBoardLane {
            id: EpiphanyCoordinatorRoleId::Imagination,
            title: "Imagination / Planning".to_string(),
            owner_role: input.imagination_owner_role,
            status: imagination_status,
            note: imagination_note,
            jobs: imagination_jobs,
            authority_scopes: vec![
                "thread/epiphany/roleLaunch".to_string(),
                "thread/epiphany/roleResult".to_string(),
                "thread/epiphany/roleAccept".to_string(),
                "thread/epiphany/update".to_string(),
            ],
            recommended_action: imagination_recommended_action,
        },
        EpiphanyRoleBoardLane {
            id: EpiphanyCoordinatorRoleId::Modeling,
            title: "Modeling / Checkpoint".to_string(),
            owner_role: "epiphany-modeler".to_string(),
            status: modeling_status,
            note: render_modeling_role_note(input.checkpoint.as_ref(), &input.reorient_next_action),
            jobs: modeling_jobs,
            authority_scopes: vec![
                "thread/epiphany/roleLaunch".to_string(),
                "thread/epiphany/roleResult".to_string(),
                "thread/epiphany/roleAccept".to_string(),
                "thread/epiphany/update".to_string(),
            ],
            recommended_action: modeling_recommended_action,
        },
        EpiphanyRoleBoardLane {
            id: EpiphanyCoordinatorRoleId::Verification,
            title: "Verification / Review".to_string(),
            owner_role: "epiphany-verifier".to_string(),
            status: verification_status,
            note: verification_note,
            jobs: verification_jobs,
            authority_scopes: vec![
                "thread/epiphany/roleLaunch".to_string(),
                "thread/epiphany/roleResult".to_string(),
                "thread/epiphany/roleAccept".to_string(),
                "thread/epiphany/distill".to_string(),
                "thread/epiphany/propose".to_string(),
                "thread/epiphany/promote".to_string(),
            ],
            recommended_action: verification_recommended_action,
        },
        EpiphanyRoleBoardLane {
            id: EpiphanyCoordinatorRoleId::Reorientation,
            title: "Reorientation".to_string(),
            owner_role: input.reorient_owner_role,
            status: reorientation_role_status(input.crrc_action, input.reorient_result_status),
            note: format!(
                "{} verdict, result {:?}, pressure {}. {}",
                input.reorient_decision_action,
                input.reorient_result_status,
                input.pressure_level,
                input.crrc_reason
            ),
            jobs: reorientation_jobs,
            authority_scopes: vec![
                "thread/epiphany/reorientLaunch".to_string(),
                "thread/epiphany/reorientResult".to_string(),
                "thread/epiphany/reorientAccept".to_string(),
            ],
            recommended_action: input.crrc_recommended_scene_action,
        },
    ]
}

pub fn role_board_job_status_to_role_status(
    status: EpiphanyRoleBoardJobStatus,
) -> EpiphanyCoordinatorRoleStatus {
    match status {
        EpiphanyRoleBoardJobStatus::Idle => EpiphanyCoordinatorRoleStatus::Ready,
        EpiphanyRoleBoardJobStatus::Needed => EpiphanyCoordinatorRoleStatus::Needed,
        EpiphanyRoleBoardJobStatus::Pending | EpiphanyRoleBoardJobStatus::Running => {
            EpiphanyCoordinatorRoleStatus::Running
        }
        EpiphanyRoleBoardJobStatus::Completed => EpiphanyCoordinatorRoleStatus::Completed,
        EpiphanyRoleBoardJobStatus::Failed
        | EpiphanyRoleBoardJobStatus::Cancelled
        | EpiphanyRoleBoardJobStatus::Blocked => EpiphanyCoordinatorRoleStatus::Blocked,
        EpiphanyRoleBoardJobStatus::Unavailable => EpiphanyCoordinatorRoleStatus::Unavailable,
    }
}

pub fn reorientation_role_status(
    action: EpiphanyCrrcAction,
    result_status: EpiphanyCrrcResultStatus,
) -> EpiphanyCoordinatorRoleStatus {
    match result_status {
        EpiphanyCrrcResultStatus::Pending | EpiphanyCrrcResultStatus::Running => {
            return EpiphanyCoordinatorRoleStatus::Waiting;
        }
        EpiphanyCrrcResultStatus::Failed
        | EpiphanyCrrcResultStatus::Cancelled
        | EpiphanyCrrcResultStatus::BackendUnavailable
        | EpiphanyCrrcResultStatus::BackendMissing => {
            return EpiphanyCoordinatorRoleStatus::Blocked;
        }
        _ => {}
    }

    match action {
        EpiphanyCrrcAction::Continue => EpiphanyCoordinatorRoleStatus::Ready,
        EpiphanyCrrcAction::PrepareCheckpoint
        | EpiphanyCrrcAction::LaunchReorientWorker
        | EpiphanyCrrcAction::RegatherManually => EpiphanyCoordinatorRoleStatus::Needed,
        EpiphanyCrrcAction::WaitForReorientWorker => EpiphanyCoordinatorRoleStatus::Waiting,
        EpiphanyCrrcAction::ReviewReorientResult | EpiphanyCrrcAction::AcceptReorientResult => {
            EpiphanyCoordinatorRoleStatus::Review
        }
    }
}

pub fn render_role_board_note(
    roles: &[EpiphanyRoleBoardLane],
    state_status: &str,
    recommendation: EpiphanyCrrcAction,
) -> String {
    let blocked_count = roles
        .iter()
        .filter(|role| {
            matches!(
                role.status,
                EpiphanyCoordinatorRoleStatus::Blocked | EpiphanyCoordinatorRoleStatus::Needed
            )
        })
        .count();
    format!(
        "Role ownership is derived read-only from Epiphany state, jobs, and CRRC. State: {state_status}; recommendation: {recommendation:?}; blocked-or-needed lanes: {blocked_count}.",
    )
}

fn imagination_role_status(
    state_present: bool,
    planning: &EpiphanyRoleBoardPlanningSummary,
) -> EpiphanyCoordinatorRoleStatus {
    if !state_present {
        return EpiphanyCoordinatorRoleStatus::Blocked;
    }
    if planning.capture_count == 0
        && planning.backlog_item_count == 0
        && planning.roadmap_stream_count == 0
        && planning.objective_draft_count == 0
    {
        EpiphanyCoordinatorRoleStatus::Needed
    } else {
        EpiphanyCoordinatorRoleStatus::Ready
    }
}

fn render_imagination_role_note(
    state_present: bool,
    planning: &EpiphanyRoleBoardPlanningSummary,
) -> String {
    if !state_present {
        return "No authoritative Epiphany state exists for planning synthesis.".to_string();
    }
    if planning.capture_count == 0
        && planning.backlog_item_count == 0
        && planning.roadmap_stream_count == 0
        && planning.objective_draft_count == 0
    {
        return "Planning substrate is empty; capture or import backlog material before synthesis."
            .to_string();
    }
    format!(
        "Planning material ready: {} captures, {} backlog items, {} roadmap streams, {} objective drafts.",
        planning.capture_count,
        planning.backlog_item_count,
        planning.roadmap_stream_count,
        planning.objective_draft_count
    )
}

fn modeling_role_status(
    checkpoint_present: bool,
    graph_remap_job: Option<&EpiphanyRoleBoardJob>,
) -> EpiphanyCoordinatorRoleStatus {
    if let Some(job) = graph_remap_job
        && matches!(
            job.status,
            EpiphanyRoleBoardJobStatus::Pending | EpiphanyRoleBoardJobStatus::Running
        )
    {
        return EpiphanyCoordinatorRoleStatus::Running;
    }
    if checkpoint_present {
        EpiphanyCoordinatorRoleStatus::Ready
    } else {
        EpiphanyCoordinatorRoleStatus::Needed
    }
}

fn render_modeling_role_note(
    checkpoint: Option<&EpiphanyRoleBoardCheckpointSummary>,
    reorient_next_action: &str,
) -> String {
    let Some(checkpoint) = checkpoint else {
        return format!("Checkpoint missing: {reorient_next_action}");
    };
    let next_action = checkpoint
        .next_action
        .as_deref()
        .unwrap_or(reorient_next_action);
    let disposition = checkpoint.disposition.as_deref().unwrap_or("unknown");
    format!("{disposition}: {next_action}")
}

fn job_note(job: &EpiphanyRoleBoardJob) -> Option<String> {
    job.blocking_reason
        .clone()
        .or_else(|| job.progress_note.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> EpiphanyRoleBoardInput {
        EpiphanyRoleBoardInput {
            state_present: true,
            planning: EpiphanyRoleBoardPlanningSummary {
                capture_count: 0,
                backlog_item_count: 0,
                roadmap_stream_count: 0,
                objective_draft_count: 0,
            },
            checkpoint: Some(EpiphanyRoleBoardCheckpointSummary {
                disposition: Some("resume_ready".to_string()),
                next_action: Some("keep cutting".to_string()),
            }),
            reorient_next_action: "regather".to_string(),
            jobs: Vec::new(),
            crrc_action: EpiphanyCrrcAction::Continue,
            crrc_recommended_scene_action: Some(EpiphanyCoordinatorSceneAction::Reorient),
            crrc_reason: "continue".to_string(),
            reorient_decision_action: "Resume".to_string(),
            pressure_level: "Low".to_string(),
            reorient_result_status: EpiphanyCrrcResultStatus::MissingBinding,
            reorient_job: None,
            imagination_binding_id: "imagination-synthesis-worker".to_string(),
            modeling_binding_id: "modeling-checkpoint-worker".to_string(),
            verification_binding_id: "verification-review-worker".to_string(),
            reorient_owner_role: "epiphany-reorienter".to_string(),
            imagination_owner_role: "epiphany-imagination".to_string(),
        }
    }

    #[test]
    fn projects_mvp_lanes() {
        let roles = derive_role_board(EpiphanyRoleBoardInput {
            jobs: vec![EpiphanyRoleBoardJob {
                id: "verification".to_string(),
                owner_role: "epiphany-verifier".to_string(),
                status: EpiphanyRoleBoardJobStatus::Needed,
                progress_note: None,
                blocking_reason: None,
            }],
            ..input()
        });

        assert_eq!(roles.len(), 5);
        assert_eq!(roles[0].id, EpiphanyCoordinatorRoleId::Implementation);
        assert_eq!(roles[0].status, EpiphanyCoordinatorRoleStatus::Ready);
        assert_eq!(roles[1].status, EpiphanyCoordinatorRoleStatus::Needed);
        assert_eq!(roles[2].status, EpiphanyCoordinatorRoleStatus::Ready);
        assert_eq!(roles[3].status, EpiphanyCoordinatorRoleStatus::Needed);
        assert_eq!(roles[4].status, EpiphanyCoordinatorRoleStatus::Ready);
    }

    #[test]
    fn blocks_implementation_when_crrc_blocks() {
        let roles = derive_role_board(EpiphanyRoleBoardInput {
            crrc_action: EpiphanyCrrcAction::LaunchReorientWorker,
            crrc_recommended_scene_action: Some(EpiphanyCoordinatorSceneAction::ReorientLaunch),
            ..input()
        });

        assert_eq!(roles[0].status, EpiphanyCoordinatorRoleStatus::Blocked);
        assert_eq!(
            roles[0].recommended_action,
            Some(EpiphanyCoordinatorSceneAction::ReorientLaunch)
        );
    }

    #[test]
    fn reorientation_waits_for_running_worker() {
        assert_eq!(
            reorientation_role_status(
                EpiphanyCrrcAction::Continue,
                EpiphanyCrrcResultStatus::Running
            ),
            EpiphanyCoordinatorRoleStatus::Waiting
        );
    }
}
