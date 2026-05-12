use super::EpiphanyCrrcAction;
use super::EpiphanyCrrcSceneAction;
use super::EpiphanyCrrcStateStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyCoordinatorRoleId {
    Implementation,
    Imagination,
    Modeling,
    Verification,
    Reorientation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyCoordinatorRoleStatus {
    Ready,
    Needed,
    Running,
    Waiting,
    Review,
    Blocked,
    Unavailable,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyCoordinatorRoleResultStatus {
    MissingState,
    MissingBinding,
    BackendUnavailable,
    BackendMissing,
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyCoordinatorAction {
    PrepareCheckpoint,
    CompactRehydrateReorient,
    LaunchReorientWorker,
    WaitForReorientWorker,
    ReviewReorientResult,
    RegatherManually,
    LaunchModeling,
    ReviewModelingResult,
    LaunchVerification,
    ReviewVerificationResult,
    ContinueImplementation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyCoordinatorSceneAction {
    Update,
    Reorient,
    ReorientLaunch,
    ReorientResult,
    ReorientAccept,
    RoleLaunch,
    RoleResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyCoordinatorAutomationAction {
    None,
    CompactRehydrateReorient,
    LaunchReorientWorker,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyCoordinatorCrrcRecommendation {
    pub action: EpiphanyCrrcAction,
    pub recommended_scene_action: Option<EpiphanyCoordinatorSceneAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EpiphanyCoordinatorSignals {
    pub modeling_result_status: EpiphanyCoordinatorRoleResultStatus,
    pub verification_result_status: EpiphanyCoordinatorRoleResultStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyCoordinatorRoleLane {
    pub id: EpiphanyCoordinatorRoleId,
    pub status: EpiphanyCoordinatorRoleStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyCoordinatorInput {
    pub state_status: EpiphanyCrrcStateStatus,
    pub checkpoint_present: bool,
    pub should_prepare_compaction: bool,
    pub recommendation: EpiphanyCoordinatorCrrcRecommendation,
    pub roles: Vec<EpiphanyCoordinatorRoleLane>,
    pub signals: EpiphanyCoordinatorSignals,
    pub modeling_result_accepted: bool,
    pub modeling_result_reviewable: bool,
    pub modeling_result_accepted_after_verification: bool,
    pub implementation_evidence_after_verification: bool,
    pub verification_result_cites_implementation_evidence: bool,
    pub verification_result_covers_current_modeling: bool,
    pub verification_result_accepted: bool,
    pub verification_result_allows_implementation: bool,
    pub verification_result_needs_evidence: bool,
    pub reorient_finding_accepted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyCoordinatorDecision {
    pub action: EpiphanyCoordinatorAction,
    pub target_role: Option<EpiphanyCoordinatorRoleId>,
    pub recommended_scene_action: Option<EpiphanyCoordinatorSceneAction>,
    pub requires_review: bool,
    pub can_auto_run: bool,
    pub reason: String,
}

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

pub fn recommend_coordinator_action(
    input: EpiphanyCoordinatorInput,
) -> EpiphanyCoordinatorDecision {
    let build = |action,
                 target_role,
                 recommended_scene_action,
                 requires_review,
                 can_auto_run,
                 reason: &str| EpiphanyCoordinatorDecision {
        action,
        target_role,
        recommended_scene_action,
        requires_review,
        can_auto_run,
        reason: reason.to_string(),
    };

    if input.state_status == EpiphanyCrrcStateStatus::Missing || !input.checkpoint_present {
        return build(
            EpiphanyCoordinatorAction::PrepareCheckpoint,
            Some(EpiphanyCoordinatorRoleId::Modeling),
            Some(EpiphanyCoordinatorSceneAction::Update),
            false,
            false,
            "Authoritative state or investigation checkpoint is missing; prepare a checkpoint before coordination can continue.",
        );
    }

    match input.recommendation.action {
        EpiphanyCrrcAction::WaitForReorientWorker => {
            return build(
                EpiphanyCoordinatorAction::WaitForReorientWorker,
                Some(EpiphanyCoordinatorRoleId::Reorientation),
                Some(EpiphanyCoordinatorSceneAction::ReorientResult),
                false,
                false,
                "A CRRC reorient worker is already in flight; wait for the bounded result.",
            );
        }
        EpiphanyCrrcAction::AcceptReorientResult | EpiphanyCrrcAction::ReviewReorientResult => {
            return build(
                EpiphanyCoordinatorAction::ReviewReorientResult,
                Some(EpiphanyCoordinatorRoleId::Reorientation),
                input.recommendation.recommended_scene_action,
                true,
                false,
                "A CRRC reorientation finding needs human review before continuation.",
            );
        }
        EpiphanyCrrcAction::PrepareCheckpoint
        | EpiphanyCrrcAction::LaunchReorientWorker
        | EpiphanyCrrcAction::RegatherManually
        | EpiphanyCrrcAction::Continue => {}
    }

    if input.should_prepare_compaction
        && !(input.reorient_finding_accepted
            && input.recommendation.action == EpiphanyCrrcAction::Continue)
    {
        return build(
            EpiphanyCoordinatorAction::CompactRehydrateReorient,
            Some(EpiphanyCoordinatorRoleId::Reorientation),
            Some(EpiphanyCoordinatorSceneAction::Reorient),
            false,
            true,
            "Context pressure crossed the preparation threshold; compact, rehydrate, and reorient before more implementation work.",
        );
    }

    if input.recommendation.action == EpiphanyCrrcAction::LaunchReorientWorker {
        return build(
            EpiphanyCoordinatorAction::LaunchReorientWorker,
            Some(EpiphanyCoordinatorRoleId::Reorientation),
            Some(EpiphanyCoordinatorSceneAction::ReorientLaunch),
            false,
            true,
            "CRRC says continuity needs a bounded reorient worker before safe continuation.",
        );
    }

    if input.signals.modeling_result_status == EpiphanyCoordinatorRoleResultStatus::Completed
        && !input.modeling_result_accepted
    {
        if !input.modeling_result_reviewable {
            return build(
                EpiphanyCoordinatorAction::LaunchModeling,
                Some(EpiphanyCoordinatorRoleId::Modeling),
                Some(EpiphanyCoordinatorSceneAction::RoleLaunch),
                false,
                true,
                "The completed modeling/checkpoint finding is not reviewable because it has no acceptable statePatch; relaunch modeling with the typed patch contract before verification or implementation continues.",
            );
        }
        return build(
            EpiphanyCoordinatorAction::ReviewModelingResult,
            Some(EpiphanyCoordinatorRoleId::Modeling),
            Some(EpiphanyCoordinatorSceneAction::RoleResult),
            true,
            false,
            "A modeling/checkpoint finding is complete and must be reviewed before verification or implementation continues.",
        );
    }

    if matches!(
        input.signals.modeling_result_status,
        EpiphanyCoordinatorRoleResultStatus::Pending | EpiphanyCoordinatorRoleResultStatus::Running
    ) {
        return build(
            EpiphanyCoordinatorAction::ReviewModelingResult,
            Some(EpiphanyCoordinatorRoleId::Modeling),
            Some(EpiphanyCoordinatorSceneAction::RoleResult),
            false,
            false,
            "A modeling/checkpoint specialist is already running; wait for its result before reviewing stale verification output.",
        );
    }

    if input.signals.verification_result_status == EpiphanyCoordinatorRoleResultStatus::Completed
        && !input.verification_result_covers_current_modeling
    {
        return build(
            EpiphanyCoordinatorAction::LaunchVerification,
            Some(EpiphanyCoordinatorRoleId::Verification),
            Some(EpiphanyCoordinatorSceneAction::RoleLaunch),
            false,
            true,
            "The completed verification/review finding does not cover the currently accepted modeling evidence; relaunch verification before implementation continues.",
        );
    }

    if input.signals.verification_result_status == EpiphanyCoordinatorRoleResultStatus::Completed
        && !input.verification_result_accepted
    {
        return build(
            EpiphanyCoordinatorAction::ReviewVerificationResult,
            Some(EpiphanyCoordinatorRoleId::Verification),
            Some(EpiphanyCoordinatorSceneAction::RoleResult),
            true,
            false,
            "A verification/review finding is complete and must be reviewed before continuation.",
        );
    }

    if input.verification_result_accepted && input.implementation_evidence_after_verification {
        return build(
            EpiphanyCoordinatorAction::LaunchVerification,
            Some(EpiphanyCoordinatorRoleId::Verification),
            Some(EpiphanyCoordinatorSceneAction::RoleLaunch),
            false,
            true,
            "Implementation evidence was produced after the accepted verification/review finding; rerun verification before implementation continues.",
        );
    }

    if input.recommendation.action == EpiphanyCrrcAction::RegatherManually
        && role_status(&input.roles, EpiphanyCoordinatorRoleId::Implementation)
            == Some(EpiphanyCoordinatorRoleStatus::Blocked)
    {
        return build(
            EpiphanyCoordinatorAction::RegatherManually,
            Some(EpiphanyCoordinatorRoleId::Reorientation),
            input.recommendation.recommended_scene_action,
            true,
            false,
            "CRRC says regather is required and the implementation lane is blocked; repair continuity before another coding turn.",
        );
    }

    if input.signals.verification_result_status == EpiphanyCoordinatorRoleResultStatus::Completed
        && input.verification_result_accepted
        && !input.verification_result_allows_implementation
        && input.verification_result_needs_evidence
        && input.modeling_result_accepted
        && input.verification_result_covers_current_modeling
    {
        return build(
            EpiphanyCoordinatorAction::ContinueImplementation,
            Some(EpiphanyCoordinatorRoleId::Implementation),
            None,
            false,
            false,
            "The accepted verification/review finding asks for implementation evidence from the current modeling checkpoint; continue only the bounded evidence-gathering implementation step before re-verification.",
        );
    }

    if input.signals.verification_result_status == EpiphanyCoordinatorRoleResultStatus::Completed
        && input.verification_result_accepted
        && !input.verification_result_allows_implementation
        && input.verification_result_cites_implementation_evidence
        && input.modeling_result_accepted
        && input.verification_result_covers_current_modeling
    {
        return build(
            EpiphanyCoordinatorAction::ContinueImplementation,
            Some(EpiphanyCoordinatorRoleId::Implementation),
            None,
            false,
            false,
            "The accepted verification/review finding failed against concrete implementation evidence from the current model; continue only the bounded repair step before re-verification.",
        );
    }

    if input.signals.verification_result_status == EpiphanyCoordinatorRoleResultStatus::Completed
        && input.verification_result_accepted
        && !input.verification_result_allows_implementation
        && !input.modeling_result_accepted_after_verification
    {
        return build(
            EpiphanyCoordinatorAction::LaunchModeling,
            Some(EpiphanyCoordinatorRoleId::Modeling),
            Some(EpiphanyCoordinatorSceneAction::RoleLaunch),
            false,
            true,
            "The accepted verification/review finding did not pass; strengthen modeling/checkpoint evidence before implementation continues.",
        );
    }

    if input.signals.modeling_result_status == EpiphanyCoordinatorRoleResultStatus::Completed
        && input.modeling_result_accepted
        && !input.verification_result_allows_implementation
    {
        return build(
            EpiphanyCoordinatorAction::LaunchVerification,
            Some(EpiphanyCoordinatorRoleId::Verification),
            Some(EpiphanyCoordinatorSceneAction::RoleLaunch),
            false,
            true,
            "Modeling/checkpoint guidance is available; run verification before implementation continues.",
        );
    }

    if role_status(&input.roles, EpiphanyCoordinatorRoleId::Modeling).is_some_and(|status| {
        matches!(
            status,
            EpiphanyCoordinatorRoleStatus::Ready | EpiphanyCoordinatorRoleStatus::Needed
        )
    }) && matches!(
        input.signals.modeling_result_status,
        EpiphanyCoordinatorRoleResultStatus::MissingBinding
            | EpiphanyCoordinatorRoleResultStatus::BackendUnavailable
            | EpiphanyCoordinatorRoleResultStatus::BackendMissing
            | EpiphanyCoordinatorRoleResultStatus::Cancelled
            | EpiphanyCoordinatorRoleResultStatus::Failed
    ) {
        return build(
            EpiphanyCoordinatorAction::LaunchModeling,
            Some(EpiphanyCoordinatorRoleId::Modeling),
            Some(EpiphanyCoordinatorSceneAction::RoleLaunch),
            false,
            true,
            "The modeling/checkpoint lane is ready and no current modeling finding is available.",
        );
    }

    if role_status(&input.roles, EpiphanyCoordinatorRoleId::Verification).is_some_and(|status| {
        matches!(
            status,
            EpiphanyCoordinatorRoleStatus::Ready | EpiphanyCoordinatorRoleStatus::Needed
        )
    }) && matches!(
        input.signals.verification_result_status,
        EpiphanyCoordinatorRoleResultStatus::MissingBinding
            | EpiphanyCoordinatorRoleResultStatus::BackendUnavailable
            | EpiphanyCoordinatorRoleResultStatus::BackendMissing
            | EpiphanyCoordinatorRoleResultStatus::Cancelled
            | EpiphanyCoordinatorRoleResultStatus::Failed
    ) {
        return build(
            EpiphanyCoordinatorAction::LaunchVerification,
            Some(EpiphanyCoordinatorRoleId::Verification),
            Some(EpiphanyCoordinatorSceneAction::RoleLaunch),
            false,
            true,
            "The verification/review lane is ready and no current verification finding is available.",
        );
    }

    if matches!(
        input.signals.verification_result_status,
        EpiphanyCoordinatorRoleResultStatus::Pending | EpiphanyCoordinatorRoleResultStatus::Running
    ) {
        return build(
            EpiphanyCoordinatorAction::ReviewVerificationResult,
            Some(EpiphanyCoordinatorRoleId::Verification),
            Some(EpiphanyCoordinatorSceneAction::RoleResult),
            false,
            false,
            "A verification/review specialist is already running; wait for its result.",
        );
    }

    if input.recommendation.action == EpiphanyCrrcAction::RegatherManually {
        return build(
            EpiphanyCoordinatorAction::RegatherManually,
            Some(EpiphanyCoordinatorRoleId::Reorientation),
            input.recommendation.recommended_scene_action,
            true,
            false,
            "CRRC cannot safely continue automatically and no fixed specialist lane is currently able to advance the regather.",
        );
    }

    build(
        EpiphanyCoordinatorAction::ContinueImplementation,
        Some(EpiphanyCoordinatorRoleId::Implementation),
        None,
        false,
        false,
        "CRRC is clear and no specialist lane is currently blocking implementation.",
    )
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
        EpiphanyCoordinatorAction::PrepareCheckpoint
        | EpiphanyCoordinatorAction::WaitForReorientWorker
        | EpiphanyCoordinatorAction::ReviewReorientResult
        | EpiphanyCoordinatorAction::RegatherManually
        | EpiphanyCoordinatorAction::LaunchModeling
        | EpiphanyCoordinatorAction::ReviewModelingResult
        | EpiphanyCoordinatorAction::LaunchVerification
        | EpiphanyCoordinatorAction::ReviewVerificationResult
        | EpiphanyCoordinatorAction::ContinueImplementation => {
            EpiphanyCoordinatorAutomationAction::None
        }
    }
}

pub fn select_coordinator_automation_action(
    decision: &EpiphanyCoordinatorDecision,
    force_checkpoint_compaction: bool,
) -> EpiphanyCoordinatorAutomationAction {
    if force_checkpoint_compaction {
        return EpiphanyCoordinatorAutomationAction::CompactRehydrateReorient;
    }
    coordinator_automation_action(decision)
}

fn role_status(
    roles: &[EpiphanyCoordinatorRoleLane],
    role_id: EpiphanyCoordinatorRoleId,
) -> Option<EpiphanyCoordinatorRoleStatus> {
    roles
        .iter()
        .find(|role| role.id == role_id)
        .map(|role| role.status)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_roles() -> Vec<EpiphanyCoordinatorRoleLane> {
        vec![
            role(
                EpiphanyCoordinatorRoleId::Implementation,
                EpiphanyCoordinatorRoleStatus::Ready,
            ),
            role(
                EpiphanyCoordinatorRoleId::Imagination,
                EpiphanyCoordinatorRoleStatus::Ready,
            ),
            role(
                EpiphanyCoordinatorRoleId::Modeling,
                EpiphanyCoordinatorRoleStatus::Ready,
            ),
            role(
                EpiphanyCoordinatorRoleId::Verification,
                EpiphanyCoordinatorRoleStatus::Ready,
            ),
            role(
                EpiphanyCoordinatorRoleId::Reorientation,
                EpiphanyCoordinatorRoleStatus::Ready,
            ),
        ]
    }

    fn role(
        id: EpiphanyCoordinatorRoleId,
        status: EpiphanyCoordinatorRoleStatus,
    ) -> EpiphanyCoordinatorRoleLane {
        EpiphanyCoordinatorRoleLane { id, status }
    }

    fn recommendation(action: EpiphanyCrrcAction) -> EpiphanyCoordinatorCrrcRecommendation {
        EpiphanyCoordinatorCrrcRecommendation {
            action,
            recommended_scene_action: Some(EpiphanyCoordinatorSceneAction::Reorient),
        }
    }

    fn input() -> EpiphanyCoordinatorInput {
        EpiphanyCoordinatorInput {
            state_status: EpiphanyCrrcStateStatus::Ready,
            checkpoint_present: true,
            should_prepare_compaction: false,
            recommendation: recommendation(EpiphanyCrrcAction::Continue),
            roles: base_roles(),
            signals: EpiphanyCoordinatorSignals {
                modeling_result_status: EpiphanyCoordinatorRoleResultStatus::MissingBinding,
                verification_result_status: EpiphanyCoordinatorRoleResultStatus::MissingBinding,
            },
            modeling_result_accepted: false,
            modeling_result_reviewable: false,
            modeling_result_accepted_after_verification: false,
            implementation_evidence_after_verification: false,
            verification_result_cites_implementation_evidence: false,
            verification_result_covers_current_modeling: true,
            verification_result_accepted: false,
            verification_result_allows_implementation: false,
            verification_result_needs_evidence: false,
            reorient_finding_accepted: false,
        }
    }

    #[test]
    fn prepares_missing_checkpoint() {
        let decision = recommend_coordinator_action(EpiphanyCoordinatorInput {
            checkpoint_present: false,
            recommendation: recommendation(EpiphanyCrrcAction::PrepareCheckpoint),
            ..input()
        });

        assert_eq!(
            decision.action,
            EpiphanyCoordinatorAction::PrepareCheckpoint
        );
        assert_eq!(
            decision.target_role,
            Some(EpiphanyCoordinatorRoleId::Modeling)
        );
        assert!(!decision.can_auto_run);
    }

    #[test]
    fn compacts_at_pressure_threshold() {
        let decision = recommend_coordinator_action(EpiphanyCoordinatorInput {
            should_prepare_compaction: true,
            ..input()
        });

        assert_eq!(
            decision.action,
            EpiphanyCoordinatorAction::CompactRehydrateReorient
        );
        assert_eq!(
            coordinator_automation_action(&decision),
            EpiphanyCoordinatorAutomationAction::CompactRehydrateReorient
        );
    }

    #[test]
    fn does_not_recompact_after_accepted_resume_reorient() {
        let decision = recommend_coordinator_action(EpiphanyCoordinatorInput {
            should_prepare_compaction: true,
            roles: Vec::new(),
            signals: EpiphanyCoordinatorSignals {
                modeling_result_status: EpiphanyCoordinatorRoleResultStatus::BackendMissing,
                verification_result_status: EpiphanyCoordinatorRoleResultStatus::BackendMissing,
            },
            reorient_finding_accepted: true,
            ..input()
        });

        assert_eq!(
            decision.action,
            EpiphanyCoordinatorAction::ContinueImplementation
        );
    }

    #[test]
    fn launches_reorient_worker_from_crrc() {
        let decision = recommend_coordinator_action(EpiphanyCoordinatorInput {
            recommendation: recommendation(EpiphanyCrrcAction::LaunchReorientWorker),
            ..input()
        });

        assert_eq!(
            decision.action,
            EpiphanyCoordinatorAction::LaunchReorientWorker
        );
        assert_eq!(
            coordinator_automation_action(&decision),
            EpiphanyCoordinatorAutomationAction::LaunchReorientWorker
        );
    }

    #[test]
    fn reviews_reorient_result() {
        let decision = recommend_coordinator_action(EpiphanyCoordinatorInput {
            recommendation: EpiphanyCoordinatorCrrcRecommendation {
                action: EpiphanyCrrcAction::AcceptReorientResult,
                recommended_scene_action: Some(EpiphanyCoordinatorSceneAction::ReorientAccept),
            },
            ..input()
        });

        assert_eq!(
            decision.action,
            EpiphanyCoordinatorAction::ReviewReorientResult
        );
        assert!(decision.requires_review);
        assert_eq!(
            coordinator_automation_action(&decision),
            EpiphanyCoordinatorAutomationAction::None
        );
    }

    #[test]
    fn uses_fixed_lanes_before_manual_regather() {
        let launch_modeling = recommend_coordinator_action(EpiphanyCoordinatorInput {
            recommendation: recommendation(EpiphanyCrrcAction::RegatherManually),
            signals: EpiphanyCoordinatorSignals {
                modeling_result_status: EpiphanyCoordinatorRoleResultStatus::MissingBinding,
                verification_result_status: EpiphanyCoordinatorRoleResultStatus::MissingBinding,
            },
            reorient_finding_accepted: true,
            ..input()
        });
        assert_eq!(
            launch_modeling.action,
            EpiphanyCoordinatorAction::LaunchModeling
        );

        let review_modeling = recommend_coordinator_action(EpiphanyCoordinatorInput {
            recommendation: recommendation(EpiphanyCrrcAction::RegatherManually),
            signals: EpiphanyCoordinatorSignals {
                modeling_result_status: EpiphanyCoordinatorRoleResultStatus::Completed,
                verification_result_status: EpiphanyCoordinatorRoleResultStatus::MissingBinding,
            },
            modeling_result_reviewable: true,
            reorient_finding_accepted: true,
            ..input()
        });
        assert_eq!(
            review_modeling.action,
            EpiphanyCoordinatorAction::ReviewModelingResult
        );

        let blocked_regather = recommend_coordinator_action(EpiphanyCoordinatorInput {
            recommendation: recommendation(EpiphanyCrrcAction::RegatherManually),
            roles: vec![role(
                EpiphanyCoordinatorRoleId::Implementation,
                EpiphanyCoordinatorRoleStatus::Blocked,
            )],
            signals: EpiphanyCoordinatorSignals {
                modeling_result_status: EpiphanyCoordinatorRoleResultStatus::Completed,
                verification_result_status: EpiphanyCoordinatorRoleResultStatus::Completed,
            },
            modeling_result_accepted: true,
            modeling_result_accepted_after_verification: true,
            verification_result_accepted: true,
            verification_result_needs_evidence: true,
            reorient_finding_accepted: true,
            ..input()
        });
        assert_eq!(
            blocked_regather.action,
            EpiphanyCoordinatorAction::RegatherManually
        );
    }

    #[test]
    fn runs_modeling_then_verification_then_continue() {
        let launch_modeling = recommend_coordinator_action(input());
        assert_eq!(
            launch_modeling.action,
            EpiphanyCoordinatorAction::LaunchModeling
        );
        assert_eq!(
            coordinator_automation_action(&launch_modeling),
            EpiphanyCoordinatorAutomationAction::None
        );

        let review_modeling = recommend_coordinator_action(EpiphanyCoordinatorInput {
            signals: EpiphanyCoordinatorSignals {
                modeling_result_status: EpiphanyCoordinatorRoleResultStatus::Completed,
                verification_result_status: EpiphanyCoordinatorRoleResultStatus::MissingBinding,
            },
            modeling_result_reviewable: true,
            ..input()
        });
        assert_eq!(
            review_modeling.action,
            EpiphanyCoordinatorAction::ReviewModelingResult
        );
        assert!(review_modeling.requires_review);

        let wait_for_modeling = recommend_coordinator_action(EpiphanyCoordinatorInput {
            signals: EpiphanyCoordinatorSignals {
                modeling_result_status: EpiphanyCoordinatorRoleResultStatus::Running,
                verification_result_status: EpiphanyCoordinatorRoleResultStatus::Completed,
            },
            ..input()
        });
        assert_eq!(
            wait_for_modeling.action,
            EpiphanyCoordinatorAction::ReviewModelingResult
        );
        assert!(wait_for_modeling.reason.contains("stale verification"));

        let relaunch_unreviewable_modeling =
            recommend_coordinator_action(EpiphanyCoordinatorInput {
                signals: EpiphanyCoordinatorSignals {
                    modeling_result_status: EpiphanyCoordinatorRoleResultStatus::Completed,
                    verification_result_status: EpiphanyCoordinatorRoleResultStatus::MissingBinding,
                },
                ..input()
            });
        assert_eq!(
            relaunch_unreviewable_modeling.action,
            EpiphanyCoordinatorAction::LaunchModeling
        );

        let launch_verification = recommend_coordinator_action(EpiphanyCoordinatorInput {
            signals: EpiphanyCoordinatorSignals {
                modeling_result_status: EpiphanyCoordinatorRoleResultStatus::Completed,
                verification_result_status: EpiphanyCoordinatorRoleResultStatus::MissingBinding,
            },
            modeling_result_accepted: true,
            modeling_result_reviewable: true,
            ..input()
        });
        assert_eq!(
            launch_verification.action,
            EpiphanyCoordinatorAction::LaunchVerification
        );

        let verification_done = EpiphanyCoordinatorSignals {
            modeling_result_status: EpiphanyCoordinatorRoleResultStatus::Completed,
            verification_result_status: EpiphanyCoordinatorRoleResultStatus::Completed,
        };

        let stale_verification = recommend_coordinator_action(EpiphanyCoordinatorInput {
            signals: verification_done,
            modeling_result_accepted: true,
            modeling_result_reviewable: true,
            verification_result_covers_current_modeling: false,
            ..input()
        });
        assert_eq!(
            stale_verification.action,
            EpiphanyCoordinatorAction::LaunchVerification
        );

        let review_verification = recommend_coordinator_action(EpiphanyCoordinatorInput {
            signals: verification_done,
            modeling_result_accepted: true,
            modeling_result_reviewable: true,
            ..input()
        });
        assert_eq!(
            review_verification.action,
            EpiphanyCoordinatorAction::ReviewVerificationResult
        );

        let accepted_non_pass = recommend_coordinator_action(EpiphanyCoordinatorInput {
            signals: verification_done,
            modeling_result_accepted: true,
            modeling_result_reviewable: true,
            verification_result_accepted: true,
            ..input()
        });
        assert_eq!(
            accepted_non_pass.action,
            EpiphanyCoordinatorAction::LaunchModeling
        );

        let needs_evidence = recommend_coordinator_action(EpiphanyCoordinatorInput {
            signals: verification_done,
            modeling_result_accepted: true,
            modeling_result_reviewable: true,
            verification_result_accepted: true,
            verification_result_needs_evidence: true,
            ..input()
        });
        assert_eq!(
            needs_evidence.action,
            EpiphanyCoordinatorAction::ContinueImplementation
        );

        let implementation_after_verification =
            recommend_coordinator_action(EpiphanyCoordinatorInput {
                signals: verification_done,
                modeling_result_accepted: true,
                modeling_result_reviewable: true,
                implementation_evidence_after_verification: true,
                verification_result_accepted: true,
                verification_result_needs_evidence: true,
                ..input()
            });
        assert_eq!(
            implementation_after_verification.action,
            EpiphanyCoordinatorAction::LaunchVerification
        );

        let accepted_pass = recommend_coordinator_action(EpiphanyCoordinatorInput {
            signals: verification_done,
            modeling_result_accepted: true,
            modeling_result_reviewable: true,
            verification_result_accepted: true,
            verification_result_allows_implementation: true,
            ..input()
        });
        assert_eq!(
            accepted_pass.action,
            EpiphanyCoordinatorAction::ContinueImplementation
        );
    }

    #[test]
    fn forced_checkpoint_compaction_overrides_decision() {
        let decision = EpiphanyCoordinatorDecision {
            action: EpiphanyCoordinatorAction::ContinueImplementation,
            target_role: Some(EpiphanyCoordinatorRoleId::Implementation),
            recommended_scene_action: None,
            requires_review: false,
            can_auto_run: false,
            reason: "ordinary coordinator would continue".to_string(),
        };

        assert_eq!(
            coordinator_automation_action(&decision),
            EpiphanyCoordinatorAutomationAction::None
        );
        assert_eq!(
            select_coordinator_automation_action(&decision, true),
            EpiphanyCoordinatorAutomationAction::CompactRehydrateReorient
        );
    }
}
