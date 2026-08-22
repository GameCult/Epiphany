use epiphany_core::{
    EpiphanyCoordinatorAction, EpiphanyCoordinatorAutomationAction, EpiphanyCoordinatorDecision,
    EpiphanyCoordinatorSceneAction, EpiphanyCoordinatorSourceSignals, EpiphanyCoordinatorStatus,
    EpiphanyCoordinatorStatusInput, EpiphanyCrrcSceneAction,
};

pub fn crrc_scene_action_to_coordinator_scene_action(
    action: EpiphanyCrrcSceneAction,
) -> EpiphanyCoordinatorSceneAction {
    match action {
        EpiphanyCrrcSceneAction::Update => EpiphanyCoordinatorSceneAction::Update,
        EpiphanyCrrcSceneAction::Reorient => EpiphanyCoordinatorSceneAction::Reorient,
        EpiphanyCrrcSceneAction::ReorientLaunch => EpiphanyCoordinatorSceneAction::ReorientLaunch,
        EpiphanyCrrcSceneAction::ReorientResult => EpiphanyCoordinatorSceneAction::ReorientResult,
        EpiphanyCrrcSceneAction::ReorientAccept => EpiphanyCoordinatorSceneAction::ReorientAccept,
    }
}

pub fn derive_coordinator_status(
    input: EpiphanyCoordinatorStatusInput,
) -> EpiphanyCoordinatorStatus {
    let source_signals = EpiphanyCoordinatorSourceSignals {
        pressure_level: input.pressure.level,
        should_prepare_compaction: input.pressure.should_prepare_compaction,
        reorient_action: input.reorient_action,
        crrc_action: input.recommendation.action,
        reorient_result_status: input.reorient_result_status,
    };
    let decision =
        epiphany_core::recommend_coordinator_action(epiphany_core::EpiphanyCoordinatorInput {
            mind_present: input.mind_present,
            should_prepare_compaction: input.pressure.should_prepare_compaction,
            recommendation: epiphany_core::EpiphanyCoordinatorCrrcRecommendation {
                action: input.recommendation.action,
                recommended_scene_action: input
                    .recommendation
                    .recommended_scene_action
                    .map(crrc_scene_action_to_coordinator_scene_action),
            },
            current_work: input.current_work,
        });
    EpiphanyCoordinatorStatus {
        decision,
        source_signals,
        roles: input.roles,
    }
}

pub fn coordinator_automation_action(
    decision: &EpiphanyCoordinatorDecision,
) -> EpiphanyCoordinatorAutomationAction {
    if !decision.can_auto_run {
        return EpiphanyCoordinatorAutomationAction::None;
    }
    match decision.action {
        EpiphanyCoordinatorAction::CompactRehydrateReorient => {
            EpiphanyCoordinatorAutomationAction::CompactRehydrateReorient
        }
        EpiphanyCoordinatorAction::LaunchReorientWorker => {
            EpiphanyCoordinatorAutomationAction::LaunchReorientWorker
        }
        _ => EpiphanyCoordinatorAutomationAction::None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use epiphany_core::{
        EpiphanyAgentPassContinuationAction, EpiphanyBodyModelingCurrentWorkProjection,
        EpiphanyBodyModelingWorkProjection, EpiphanyCoordinatorCrrcRecommendation,
        EpiphanyCoordinatorInput, EpiphanyCrrcAction, EpiphanyRepoModelBasis,
        RepoFrontierPlanningLifecycleStage, RepoFrontierResearchContinuationAction,
        RepositoryBodyObservationBasis, recommend_coordinator_action,
    };

    fn current_work() -> epiphany_core::EpiphanyCurrentWorkProjection {
        epiphany_core::EpiphanyCurrentWorkProjection {
            mind_projection_digest: "sha256:mind".into(),
            body_modeling: None,
            research_continuation_action: None,
            frontier_planning_stage: RepoFrontierPlanningLifecycleStage::Terminal,
            proposal_modeling: None,
            frontier_verdict_modeling: None,
            verification: None,
            reorientation: None,
            imagination_considerations: Vec::new(),
            admitted_model_direction_consideration: None,
            hands_frontier_ready: false,
        }
    }

    fn input() -> EpiphanyCoordinatorInput {
        EpiphanyCoordinatorInput {
            mind_present: true,
            should_prepare_compaction: false,
            recommendation: EpiphanyCoordinatorCrrcRecommendation {
                action: EpiphanyCrrcAction::Continue,
                recommended_scene_action: None,
            },
            current_work: current_work(),
        }
    }

    #[test]
    fn absent_work_waits_instead_of_manufacturing_modeling() {
        assert_eq!(
            recommend_coordinator_action(input()).action,
            EpiphanyCoordinatorAction::AwaitFrontierProposal
        );
    }

    #[test]
    fn exact_body_obligation_routes_modeling() {
        let mut input = input();
        input.current_work.body_modeling = Some(EpiphanyBodyModelingCurrentWorkProjection {
            work: EpiphanyBodyModelingWorkProjection {
                work_id: "body-work".into(),
                runtime_id: "runtime".into(),
                body_basis: RepositoryBodyObservationBasis {
                    schema_version: "body-basis".into(),
                    workspace_id: "workspace".into(),
                    swarm_id: "swarm".into(),
                    runtime_id: "runtime".into(),
                    scope: "git_worktree".into(),
                    body_binding_sha256: "sha256:body".into(),
                    observation_id: "observation".into(),
                    generation: 1,
                    manifest_root_sha256: "sha256:manifest".into(),
                    scan_started_at: "2026-08-22T00:00:00Z".into(),
                    scan_finished_at: "2026-08-22T00:00:01Z".into(),
                },
                repo_model_basis: EpiphanyRepoModelBasis {
                    projection_digest: "sha256:model".into(),
                    source_documents: Vec::new(),
                },
            },
            action: EpiphanyAgentPassContinuationAction::Launch,
            job_id: None,
        });
        assert_eq!(
            recommend_coordinator_action(input).action,
            EpiphanyCoordinatorAction::LaunchModeling
        );
    }

    #[test]
    fn exact_eyes_obligation_routes_research() {
        let mut input = input();
        input.current_work.research_continuation_action =
            Some(RepoFrontierResearchContinuationAction::LaunchResearch);
        assert_eq!(
            recommend_coordinator_action(input).action,
            EpiphanyCoordinatorAction::LaunchResearch
        );
    }
}
