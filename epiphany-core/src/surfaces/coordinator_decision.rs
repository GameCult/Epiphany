use crate::{
    EpiphanyAgentPassContinuationAction, EpiphanyCoordinatorAction, EpiphanyCoordinatorDecision,
    EpiphanyCoordinatorInput, EpiphanyCoordinatorRoleId, EpiphanyCoordinatorSceneAction,
    EpiphanyCrrcAction, RepoFrontierPlanningLifecycleStage, RepoFrontierResearchContinuationAction,
};

pub fn recommend_coordinator_action(
    input: EpiphanyCoordinatorInput,
) -> EpiphanyCoordinatorDecision {
    if !input.mind_present {
        return decision(
            EpiphanyCoordinatorAction::PrepareCheckpoint,
            None,
            Some(EpiphanyCoordinatorSceneAction::Update),
            false,
            false,
            "Canonical keyed Mind is missing; admit an explicit operator objective before coordination continues.",
        );
    }

    if let Some(work) = input.current_work.reorientation.as_ref() {
        return continuation_decision(
            work.action,
            EpiphanyCoordinatorAction::LaunchReorientWorker,
            EpiphanyCoordinatorAction::WaitForReorientWorker,
            EpiphanyCoordinatorAction::ReviewReorientResult,
            EpiphanyCoordinatorRoleId::Reorientation,
            "continuity",
        );
    }

    if input.should_prepare_compaction {
        return decision(
            EpiphanyCoordinatorAction::CompactRehydrateReorient,
            Some(EpiphanyCoordinatorRoleId::Reorientation),
            Some(EpiphanyCoordinatorSceneAction::Reorient),
            false,
            true,
            "Context pressure requires a new continuity obligation over the current keyed Mind.",
        );
    }

    if input.recommendation.action == EpiphanyCrrcAction::RegatherManually {
        return decision(
            EpiphanyCoordinatorAction::RegatherManually,
            Some(EpiphanyCoordinatorRoleId::Research),
            Some(EpiphanyCoordinatorSceneAction::Reorient),
            true,
            false,
            "The accepted continuity decision requires explicit operator regather.",
        );
    }

    if let Some(result) = planning_decision(input.current_work.frontier_planning_stage) {
        return result;
    }

    if let Some(work) = input.current_work.proposal_modeling.as_ref() {
        return modeling_decision(work.action, "proposal Modeling");
    }
    if let Some(action) = input.current_work.body_modeling_action {
        return modeling_decision(action, "Body Modeling");
    }
    if let Some(work) = input.current_work.frontier_verdict_modeling.as_ref() {
        return modeling_decision(work.action, "frontier-verdict Modeling");
    }
    if let Some(work) = input.current_work.verification.as_ref() {
        return continuation_decision(
            work.action,
            EpiphanyCoordinatorAction::LaunchVerification,
            EpiphanyCoordinatorAction::ReviewVerificationResult,
            EpiphanyCoordinatorAction::ReviewVerificationResult,
            EpiphanyCoordinatorRoleId::Verification,
            "Verification",
        );
    }
    if let Some(action) = input.current_work.research_continuation_action {
        return match action {
            RepoFrontierResearchContinuationAction::LaunchResearch => decision(
                EpiphanyCoordinatorAction::LaunchResearch,
                Some(EpiphanyCoordinatorRoleId::Research),
                Some(EpiphanyCoordinatorSceneAction::RoleLaunch),
                false,
                true,
                "The exact external-evidence obligation requires an Eyes attempt.",
            ),
            RepoFrontierResearchContinuationAction::ReviewResearchResult => decision(
                EpiphanyCoordinatorAction::ReviewResearchResult,
                Some(EpiphanyCoordinatorRoleId::Research),
                Some(EpiphanyCoordinatorSceneAction::RoleResult),
                true,
                false,
                "The exact Eyes result awaits Mind admission.",
            ),
        };
    }

    if let Some(work) = input.current_work.imagination_considerations.first() {
        return continuation_decision(
            work.action,
            EpiphanyCoordinatorAction::LaunchImaginationConsideration,
            EpiphanyCoordinatorAction::WaitForImaginationConsideration,
            EpiphanyCoordinatorAction::WaitForImaginationConsideration,
            EpiphanyCoordinatorRoleId::Imagination,
            "Persona-feedback Imagination consideration",
        );
    }
    if let Some(work) = input
        .current_work
        .admitted_model_direction_consideration
        .as_ref()
    {
        return continuation_decision(
            work.action,
            EpiphanyCoordinatorAction::LaunchAdmittedModelDirectionConsideration,
            EpiphanyCoordinatorAction::WaitForAdmittedModelDirectionConsideration,
            EpiphanyCoordinatorAction::WaitForAdmittedModelDirectionConsideration,
            EpiphanyCoordinatorRoleId::Imagination,
            "admitted-model direction consideration",
        );
    }

    if input.current_work.hands_frontier_ready {
        return decision(
            EpiphanyCoordinatorAction::ContinueImplementation,
            Some(EpiphanyCoordinatorRoleId::Implementation),
            None,
            false,
            false,
            "Mind has an exact actionable Hands frontier route.",
        );
    }

    decision(
        EpiphanyCoordinatorAction::AwaitFrontierProposal,
        Some(EpiphanyCoordinatorRoleId::Imagination),
        None,
        false,
        false,
        "No unresolved typed work obligation exists; await a new Body, evidence, proposal, or continuity mutation.",
    )
}

fn decision(
    action: EpiphanyCoordinatorAction,
    target_role: Option<EpiphanyCoordinatorRoleId>,
    recommended_scene_action: Option<EpiphanyCoordinatorSceneAction>,
    requires_review: bool,
    can_auto_run: bool,
    reason: &str,
) -> EpiphanyCoordinatorDecision {
    EpiphanyCoordinatorDecision {
        action,
        target_role,
        recommended_scene_action,
        requires_review,
        can_auto_run,
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
        EpiphanyCoordinatorRoleId::Modeling,
        label,
    )
}

fn continuation_decision(
    action: EpiphanyAgentPassContinuationAction,
    launch: EpiphanyCoordinatorAction,
    wait: EpiphanyCoordinatorAction,
    review: EpiphanyCoordinatorAction,
    role: EpiphanyCoordinatorRoleId,
    label: &str,
) -> EpiphanyCoordinatorDecision {
    match action {
        EpiphanyAgentPassContinuationAction::Launch => decision(
            launch,
            Some(role),
            Some(EpiphanyCoordinatorSceneAction::RoleLaunch),
            false,
            true,
            &format!("The exact {label} obligation has no live attempt."),
        ),
        EpiphanyAgentPassContinuationAction::Wait => decision(
            wait,
            Some(role),
            Some(EpiphanyCoordinatorSceneAction::RoleResult),
            false,
            false,
            &format!("The exact {label} attempt is still live."),
        ),
        EpiphanyAgentPassContinuationAction::Review => decision(
            review,
            Some(role),
            Some(EpiphanyCoordinatorSceneAction::RoleResult),
            true,
            false,
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
            Some(EpiphanyCoordinatorRoleId::Imagination),
            None,
            false,
            true,
            "An exact frontier planning obligation is ready to become a typed request.",
        ),
        RepoFrontierPlanningLifecycleStage::ImaginationLaunchReady => decision(
            EpiphanyCoordinatorAction::LaunchImagination,
            Some(EpiphanyCoordinatorRoleId::Imagination),
            Some(EpiphanyCoordinatorSceneAction::RoleLaunch),
            false,
            true,
            "The exact frontier planning request awaits Imagination.",
        ),
        RepoFrontierPlanningLifecycleStage::ImaginationRunning => decision(
            EpiphanyCoordinatorAction::WaitForImaginationResult,
            Some(EpiphanyCoordinatorRoleId::Imagination),
            Some(EpiphanyCoordinatorSceneAction::RoleResult),
            false,
            false,
            "The exact frontier Imagination attempt is still live.",
        ),
        RepoFrontierPlanningLifecycleStage::ImaginationFailed => decision(
            EpiphanyCoordinatorAction::ReviewFrontierPlanningFailure,
            Some(EpiphanyCoordinatorRoleId::Imagination),
            Some(EpiphanyCoordinatorSceneAction::RoleResult),
            true,
            false,
            "The exact frontier Imagination attempt failed.",
        ),
        RepoFrontierPlanningLifecycleStage::ImaginationResultReady => decision(
            EpiphanyCoordinatorAction::RequestMindPlanReview,
            None,
            None,
            false,
            true,
            "The exact Imagination candidate awaits a typed Mind request.",
        ),
        RepoFrontierPlanningLifecycleStage::MindLaunchReady => decision(
            EpiphanyCoordinatorAction::LaunchMindPlanReview,
            None,
            Some(EpiphanyCoordinatorSceneAction::RoleLaunch),
            false,
            true,
            "The exact Mind plan request has no live attempt.",
        ),
        RepoFrontierPlanningLifecycleStage::MindRunning => decision(
            EpiphanyCoordinatorAction::WaitForMindPlanResult,
            None,
            Some(EpiphanyCoordinatorSceneAction::RoleResult),
            false,
            false,
            "The exact Mind plan attempt is still live.",
        ),
        RepoFrontierPlanningLifecycleStage::MindFailed => decision(
            EpiphanyCoordinatorAction::ReviewFrontierPlanningFailure,
            None,
            Some(EpiphanyCoordinatorSceneAction::RoleResult),
            true,
            false,
            "The exact Mind plan attempt failed.",
        ),
        RepoFrontierPlanningLifecycleStage::MindResultReady => decision(
            EpiphanyCoordinatorAction::CommitFrontierPlanDecision,
            None,
            None,
            false,
            true,
            "The exact Mind judgment awaits atomic plan admission.",
        ),
        RepoFrontierPlanningLifecycleStage::Unavailable
        | RepoFrontierPlanningLifecycleStage::Terminal => return None,
    };
    Some(value)
}

#[cfg(test)]
mod authority_tests {
    #[test]
    fn source_has_no_aggregate_or_latest_lane_routing() {
        let source = include_str!("coordinator_decision.rs");
        let production = source.split("#[cfg(test)]").next().unwrap_or(source);
        for forbidden in [
            "EpiphanyThreadState",
            "accepted_after",
            "latest_result",
            "finding_signals",
            "state_revision",
            "coordinator_run_receipt",
        ] {
            assert!(!production.contains(forbidden));
        }
    }
}
