use super::{
    EpiphanyCoordinatorRoleId, EpiphanyCoordinatorRoleStatus, EpiphanyCoordinatorSceneAction,
};
use crate::{
    EpiphanyAgentPassContinuationAction, EpiphanyCurrentWorkProjection,
    RepoFrontierPlanningLifecycleStage, RepoFrontierResearchContinuationAction,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyRoleBoardJob {
    pub id: String,
    pub owner_role: String,
    pub status: EpiphanyRoleBoardJobStatus,
    pub progress_note: Option<String>,
    pub blocking_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyRoleBoardInput {
    pub mind_present: bool,
    pub current_work: EpiphanyCurrentWorkProjection,
    pub jobs: Vec<EpiphanyRoleBoardJob>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
    let missing = !input.mind_present;
    let lane =
        |id, title: &str, owner_role: &str, status, note: &str, scope: &str, recommended_action| {
            EpiphanyRoleBoardLane {
                id,
                title: title.into(),
                owner_role: owner_role.into(),
                status: if missing {
                    EpiphanyCoordinatorRoleStatus::Blocked
                } else {
                    status
                },
                note: if missing {
                    "Canonical keyed Mind is missing.".into()
                } else {
                    note.into()
                },
                jobs: input
                    .jobs
                    .iter()
                    .filter(|job| job.owner_role == owner_role)
                    .cloned()
                    .collect(),
                authority_scopes: vec![scope.into()],
                recommended_action,
            }
        };

    let modeling_action = input
        .current_work
        .proposal_modeling
        .as_ref()
        .map(|work| work.attempt.action)
        .or(input
            .current_work
            .body_modeling
            .as_ref()
            .map(|work| work.attempt.action))
        .or(input
            .current_work
            .frontier_verdict_modeling
            .as_ref()
            .map(|work| work.attempt.action));
    let imagination_action = consideration_action(&input.current_work);

    vec![
        lane(
            EpiphanyCoordinatorRoleId::Implementation,
            "Hands / Implementation",
            "epiphany-hands",
            if input.current_work.hands_frontier_ready {
                EpiphanyCoordinatorRoleStatus::Ready
            } else {
                EpiphanyCoordinatorRoleStatus::Blocked
            },
            if input.current_work.hands_frontier_ready {
                "An adopted plan has an exact actionable Hands route."
            } else {
                "No exact actionable Hands route exists."
            },
            "epiphany.hands.route",
            None,
        ),
        lane(
            EpiphanyCoordinatorRoleId::Imagination,
            "Imagination / Planning",
            "epiphany-imagination",
            planning_or_consideration_status(
                input.current_work.frontier_planning.stage,
                imagination_action,
            ),
            "Status derives from exact planning and consideration obligations.",
            "epiphany.imagination.current_work",
            action_scene(imagination_action),
        ),
        lane(
            EpiphanyCoordinatorRoleId::Research,
            "Eyes / Research",
            "epiphany-eyes",
            research_status(input.current_work.research.continuation_action()),
            "Eyes runs only for an explicit external-evidence obligation.",
            "epiphany.eyes.current_work",
            input
                .current_work
                .research
                .continuation_action()
                .map(|action| match action {
                    RepoFrontierResearchContinuationAction::LaunchResearch => {
                        EpiphanyCoordinatorSceneAction::RoleLaunch
                    }
                    RepoFrontierResearchContinuationAction::ReviewResearchResult => {
                        EpiphanyCoordinatorSceneAction::RoleResult
                    }
                }),
        ),
        lane(
            EpiphanyCoordinatorRoleId::Modeling,
            "Modeling / Body Map",
            "epiphany-modeling",
            continuation_status(modeling_action),
            "Status derives from exact Body, proposal, or Soul-verdict Modeling work.",
            "epiphany.modeling.current_work",
            action_scene(modeling_action),
        ),
        lane(
            EpiphanyCoordinatorRoleId::Verification,
            "Soul / Verification",
            "epiphany-soul",
            continuation_status(
                input
                    .current_work
                    .verification
                    .as_ref()
                    .map(|work| work.attempt.action),
            ),
            "Verification consumes exact Hands consequences and invariant obligations.",
            "epiphany.soul.current_work",
            action_scene(
                input
                    .current_work
                    .verification
                    .as_ref()
                    .map(|work| work.attempt.action),
            ),
        ),
        lane(
            EpiphanyCoordinatorRoleId::Reorientation,
            "Continuity / Reorientation",
            "epiphany-continuity",
            continuation_status(
                input
                    .current_work
                    .reorientation
                    .as_ref()
                    .map(|work| work.attempt.action),
            ),
            "Status derives from the exact keyed continuity obligation.",
            "epiphany.continuity.current_work",
            action_scene(
                input
                    .current_work
                    .reorientation
                    .as_ref()
                    .map(|work| work.attempt.action),
            ),
        ),
    ]
}

fn consideration_action(
    work: &EpiphanyCurrentWorkProjection,
) -> Option<EpiphanyAgentPassContinuationAction> {
    work.imagination_considerations
        .first()
        .map(|item| item.attempt.action)
        .or(work
            .admitted_model_direction_consideration
            .as_ref()
            .map(|item| item.attempt.action))
}

fn continuation_status(
    action: Option<EpiphanyAgentPassContinuationAction>,
) -> EpiphanyCoordinatorRoleStatus {
    match action {
        Some(EpiphanyAgentPassContinuationAction::Launch) => EpiphanyCoordinatorRoleStatus::Needed,
        Some(EpiphanyAgentPassContinuationAction::Wait) => EpiphanyCoordinatorRoleStatus::Running,
        Some(EpiphanyAgentPassContinuationAction::Review) => EpiphanyCoordinatorRoleStatus::Review,
        None => EpiphanyCoordinatorRoleStatus::Ready,
    }
}

fn action_scene(
    action: Option<EpiphanyAgentPassContinuationAction>,
) -> Option<EpiphanyCoordinatorSceneAction> {
    match action {
        Some(EpiphanyAgentPassContinuationAction::Launch) => {
            Some(EpiphanyCoordinatorSceneAction::RoleLaunch)
        }
        Some(EpiphanyAgentPassContinuationAction::Wait)
        | Some(EpiphanyAgentPassContinuationAction::Review) => {
            Some(EpiphanyCoordinatorSceneAction::RoleResult)
        }
        None => None,
    }
}

fn research_status(
    action: Option<RepoFrontierResearchContinuationAction>,
) -> EpiphanyCoordinatorRoleStatus {
    match action {
        Some(RepoFrontierResearchContinuationAction::LaunchResearch) => {
            EpiphanyCoordinatorRoleStatus::Needed
        }
        Some(RepoFrontierResearchContinuationAction::ReviewResearchResult) => {
            EpiphanyCoordinatorRoleStatus::Review
        }
        None => EpiphanyCoordinatorRoleStatus::Ready,
    }
}

fn planning_or_consideration_status(
    stage: RepoFrontierPlanningLifecycleStage,
    consideration: Option<EpiphanyAgentPassContinuationAction>,
) -> EpiphanyCoordinatorRoleStatus {
    if !matches!(
        stage,
        RepoFrontierPlanningLifecycleStage::Unavailable
            | RepoFrontierPlanningLifecycleStage::Terminal
    ) {
        return match stage {
            RepoFrontierPlanningLifecycleStage::ImaginationRunning
            | RepoFrontierPlanningLifecycleStage::MindRunning => {
                EpiphanyCoordinatorRoleStatus::Running
            }
            RepoFrontierPlanningLifecycleStage::ImaginationFailed
            | RepoFrontierPlanningLifecycleStage::MindFailed
            | RepoFrontierPlanningLifecycleStage::ImaginationResultReady
            | RepoFrontierPlanningLifecycleStage::MindResultReady => {
                EpiphanyCoordinatorRoleStatus::Review
            }
            _ => EpiphanyCoordinatorRoleStatus::Needed,
        };
    }
    continuation_status(consideration)
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

pub fn render_role_board_note(
    roles: &[EpiphanyRoleBoardLane],
    state_status: &str,
    recommendation: crate::EpiphanyCrrcAction,
) -> String {
    let active = roles
        .iter()
        .filter(|lane| {
            !matches!(
                lane.status,
                EpiphanyCoordinatorRoleStatus::Ready | EpiphanyCoordinatorRoleStatus::Completed
            )
        })
        .count();
    format!(
        "Mind {state_status}; {active} role lane(s) carry current work; continuity recommendation {recommendation:?}."
    )
}
