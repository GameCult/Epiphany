use super::EpiphanyCoordinatorRoleStatus;
use crate::{
    EpiphanyAgentPassContinuationAction, EpiphanyCurrentWorkProjection,
    RepoFrontierPlanningLifecycleStage, RepoFrontierResearchContinuationAction,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyRoleBoardInput {
    pub mind_present: bool,
    pub current_work: EpiphanyCurrentWorkProjection,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyRoleBoardLane {
    pub title: String,
    pub status: EpiphanyCoordinatorRoleStatus,
    pub note: String,
}

pub fn derive_role_board(input: EpiphanyRoleBoardInput) -> Vec<EpiphanyRoleBoardLane> {
    let missing = !input.mind_present;
    let lane = |title: &str, status, note: &str| {
        EpiphanyRoleBoardLane {
            title: title.into(),
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
            "Hands / Implementation",
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
        ),
        lane(
            "Imagination / Planning",
            planning_or_consideration_status(
                input.current_work.frontier_planning.stage,
                imagination_action,
            ),
            "Status derives from exact planning and consideration obligations.",
        ),
        lane(
            "Eyes / Research",
            research_status(input.current_work.research.continuation_action()),
            "Eyes runs only for an explicit external-evidence obligation.",
        ),
        lane(
            "Modeling / Body Map",
            continuation_status(modeling_action),
            "Status derives from exact Body, proposal, or Soul-verdict Modeling work.",
        ),
        lane(
            "Soul / Verification",
            continuation_status(
                input
                    .current_work
                    .verification
                    .as_ref()
                    .map(|work| work.attempt.action),
            ),
            "Verification consumes exact Hands consequences and invariant obligations.",
        ),
        lane(
            "Continuity / Reorientation",
            continuation_status(
                input
                    .current_work
                    .reorientation
                    .as_ref()
                    .map(|work| work.attempt.action),
            ),
            "Status derives from the exact keyed continuity obligation.",
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
