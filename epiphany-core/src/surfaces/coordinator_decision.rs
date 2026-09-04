use crate::{
    EpiphanyAgentPassContinuationAction, EpiphanyCoordinatorAction, EpiphanyCoordinatorDecision,
    EpiphanyCoordinatorInput, RepoFrontierPlanningLifecycleStage,
    RepoFrontierResearchContinuationAction,
};

pub fn recommend_coordinator_action(
    input: EpiphanyCoordinatorInput,
) -> EpiphanyCoordinatorDecision {
    if !input.mind_present {
        return decision(
            EpiphanyCoordinatorAction::PrepareCheckpoint,
            "Canonical keyed Mind is missing; admit an explicit operator objective before coordination continues.",
        );
    }

    if let Some(work) = input.current_work.reorientation.as_ref() {
        return continuation_decision(
            work.attempt.action,
            EpiphanyCoordinatorAction::LaunchReorientWorker,
            EpiphanyCoordinatorAction::WaitForReorientWorker,
            EpiphanyCoordinatorAction::ReviewReorientResult,
            "continuity",
        );
    }

    if input.current_work.operator_regather_required {
        return decision(
            EpiphanyCoordinatorAction::RegatherManually,
            "The accepted continuity decision requires explicit operator regather.",
        );
    }

    if let Some(result) = planning_decision(input.current_work.frontier_planning.stage) {
        return result;
    }

    if let Some(work) = input.current_work.proposal_modeling.as_ref() {
        return modeling_decision(work.attempt.action, "proposal Modeling");
    }
    if let Some(work) = input.current_work.body_modeling.as_ref() {
        return modeling_decision(work.attempt.action, "Body Modeling");
    }
    if let Some(work) = input.current_work.frontier_verdict_modeling.as_ref() {
        return modeling_decision(work.attempt.action, "frontier-verdict Modeling");
    }
    if let Some(work) = input.current_work.verification.as_ref() {
        return continuation_decision(
            work.attempt.action,
            EpiphanyCoordinatorAction::LaunchVerification,
            EpiphanyCoordinatorAction::ReviewVerificationResult,
            EpiphanyCoordinatorAction::ReviewVerificationResult,
            "Verification",
        );
    }
    if let Some(action) = input.current_work.research.continuation_action() {
        return match action {
            RepoFrontierResearchContinuationAction::LaunchResearch => decision(
                EpiphanyCoordinatorAction::LaunchResearch,
                "The exact external-evidence obligation requires an Eyes attempt.",
            ),
            RepoFrontierResearchContinuationAction::ReviewResearchResult => decision(
                EpiphanyCoordinatorAction::ReviewResearchResult,
                "The exact Eyes result awaits Mind admission.",
            ),
        };
    }

    if let Some(work) = input.current_work.imagination_considerations.first() {
        return continuation_decision(
            work.attempt.action,
            EpiphanyCoordinatorAction::LaunchImaginationConsideration,
            EpiphanyCoordinatorAction::WaitForImaginationConsideration,
            EpiphanyCoordinatorAction::WaitForImaginationConsideration,
            "Persona-feedback Imagination consideration",
        );
    }
    if let Some(work) = input
        .current_work
        .admitted_model_direction_consideration
        .as_ref()
    {
        return continuation_decision(
            work.attempt.action,
            EpiphanyCoordinatorAction::LaunchAdmittedModelDirectionConsideration,
            EpiphanyCoordinatorAction::WaitForAdmittedModelDirectionConsideration,
            EpiphanyCoordinatorAction::WaitForAdmittedModelDirectionConsideration,
            "admitted-model direction consideration",
        );
    }

    if input.current_work.hands_frontier_ready {
        return decision(
            EpiphanyCoordinatorAction::ContinueImplementation,
            "Mind has an exact actionable Hands frontier route.",
        );
    }

    decision(
        EpiphanyCoordinatorAction::AwaitFrontierProposal,
        "No unresolved typed work obligation exists; await a new Body, evidence, proposal, or continuity mutation.",
    )
}

fn decision(action: EpiphanyCoordinatorAction, reason: &str) -> EpiphanyCoordinatorDecision {
    EpiphanyCoordinatorDecision {
        action,
        reason: reason.to_string(),
    }
}

fn modeling_decision(
    action: EpiphanyAgentPassContinuationAction,
    label: &str,
) -> EpiphanyCoordinatorDecision {
    continuation_decision(
        action,
        EpiphanyCoordinatorAction::LaunchModeling,
        EpiphanyCoordinatorAction::WaitForModelingResult,
        EpiphanyCoordinatorAction::ReviewModelingResult,
        label,
    )
}

fn continuation_decision(
    action: EpiphanyAgentPassContinuationAction,
    launch: EpiphanyCoordinatorAction,
    wait: EpiphanyCoordinatorAction,
    review: EpiphanyCoordinatorAction,
    label: &str,
) -> EpiphanyCoordinatorDecision {
    match action {
        EpiphanyAgentPassContinuationAction::Launch => decision(
            launch,
            &format!("The exact {label} obligation has no live attempt."),
        ),
        EpiphanyAgentPassContinuationAction::Wait => {
            decision(wait, &format!("The exact {label} attempt is still live."))
        }
        EpiphanyAgentPassContinuationAction::Review => decision(
            review,
            &format!("The exact {label} result awaits its family admission owner."),
        ),
    }
}

fn planning_decision(
    stage: RepoFrontierPlanningLifecycleStage,
) -> Option<EpiphanyCoordinatorDecision> {
    let value = match stage {
        RepoFrontierPlanningLifecycleStage::Ready => decision(
            EpiphanyCoordinatorAction::StartFrontierPlanning,
            "An exact frontier planning obligation is ready to become a typed request.",
        ),
        RepoFrontierPlanningLifecycleStage::ImaginationLaunchReady => decision(
            EpiphanyCoordinatorAction::LaunchImagination,
            "The exact frontier planning request awaits Imagination.",
        ),
        RepoFrontierPlanningLifecycleStage::ImaginationRunning => decision(
            EpiphanyCoordinatorAction::WaitForImaginationResult,
            "The exact frontier Imagination attempt is still live.",
        ),
        RepoFrontierPlanningLifecycleStage::ImaginationFailed => decision(
            EpiphanyCoordinatorAction::ReviewFrontierPlanningFailure,
            "The exact frontier Imagination attempt failed.",
        ),
        RepoFrontierPlanningLifecycleStage::ImaginationResultReady => decision(
            EpiphanyCoordinatorAction::RequestMindPlanReview,
            "The exact Imagination candidate awaits a typed Mind request.",
        ),
        RepoFrontierPlanningLifecycleStage::MindLaunchReady => decision(
            EpiphanyCoordinatorAction::LaunchMindPlanReview,
            "The exact Mind plan request has no live attempt.",
        ),
        RepoFrontierPlanningLifecycleStage::MindRunning => decision(
            EpiphanyCoordinatorAction::WaitForMindPlanResult,
            "The exact Mind plan attempt is still live.",
        ),
        RepoFrontierPlanningLifecycleStage::MindFailed => decision(
            EpiphanyCoordinatorAction::ReviewFrontierPlanningFailure,
            "The exact Mind plan attempt failed.",
        ),
        RepoFrontierPlanningLifecycleStage::MindResultReady => decision(
            EpiphanyCoordinatorAction::CommitFrontierPlanDecision,
            "The exact Mind judgment awaits atomic plan admission.",
        ),
        RepoFrontierPlanningLifecycleStage::Unavailable
        | RepoFrontierPlanningLifecycleStage::Terminal => return None,
    };
    Some(value)
}
