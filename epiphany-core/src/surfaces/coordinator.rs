use super::EpiphanyCrrcAction;
use super::EpiphanyCrrcSceneAction;
use super::EpiphanyCrrcStateStatus;
use super::EpiphanyPressure;
use super::EpiphanyPressureLevel;
use super::EpiphanyReorientAction;
use super::EpiphanyReorientFindingInterpretation;
use super::EpiphanyRoleBoardLane;
use super::EpiphanyRoleFindingInterpretation;
use epiphany_state_model::EpiphanyThreadState;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyCoordinatorRoleId {
    Implementation,
    Imagination,
    Modeling,
    Verification,
    Reorientation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyCoordinatorSceneAction {
    Update,
    Reorient,
    ReorientLaunch,
    ReorientResult,
    ReorientAccept,
    RoleLaunch,
    RoleResult,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum EpiphanyCoordinatorAutomationAction {
    None,
    CompactRehydrateReorient,
    LaunchReorientWorker,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyCoordinatorCrrcRecommendation {
    pub action: EpiphanyCrrcAction,
    pub recommended_scene_action: Option<EpiphanyCoordinatorSceneAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyCoordinatorSignals {
    pub modeling_result_status: EpiphanyCoordinatorRoleResultStatus,
    pub verification_result_status: EpiphanyCoordinatorRoleResultStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyCoordinatorSourceSignals {
    pub pressure_level: EpiphanyPressureLevel,
    pub should_prepare_compaction: bool,
    pub reorient_action: EpiphanyReorientAction,
    pub crrc_action: EpiphanyCrrcAction,
    pub modeling_result_status: EpiphanyCoordinatorRoleResultStatus,
    pub verification_result_status: EpiphanyCoordinatorRoleResultStatus,
    pub reorient_result_status: super::EpiphanyCrrcResultStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyCoordinatorDecision {
    pub action: EpiphanyCoordinatorAction,
    pub target_role: Option<EpiphanyCoordinatorRoleId>,
    pub recommended_scene_action: Option<EpiphanyCoordinatorSceneAction>,
    pub requires_review: bool,
    pub can_auto_run: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyCoordinatorStatusInput {
    pub state_status: EpiphanyCrrcStateStatus,
    pub checkpoint_present: bool,
    pub pressure: EpiphanyPressure,
    pub recommendation: super::EpiphanyCrrcRecommendation,
    pub roles: Vec<EpiphanyRoleBoardLane>,
    pub reorient_action: EpiphanyReorientAction,
    pub modeling_result_status: EpiphanyCoordinatorRoleResultStatus,
    pub verification_result_status: EpiphanyCoordinatorRoleResultStatus,
    pub reorient_result_status: super::EpiphanyCrrcResultStatus,
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyCoordinatorStatus {
    pub decision: EpiphanyCoordinatorDecision,
    pub source_signals: EpiphanyCoordinatorSourceSignals,
    pub roles: Vec<EpiphanyRoleBoardLane>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EpiphanyCoordinatorFindingSignals {
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
        modeling_result_status: input.modeling_result_status,
        verification_result_status: input.verification_result_status,
        reorient_result_status: input.reorient_result_status,
    };
    let coordinator_roles = input
        .roles
        .iter()
        .map(coordinator_role_lane_from_role_board)
        .collect();
    let decision = recommend_coordinator_action(EpiphanyCoordinatorInput {
        state_status: input.state_status,
        checkpoint_present: input.checkpoint_present,
        should_prepare_compaction: input.pressure.should_prepare_compaction,
        recommendation: EpiphanyCoordinatorCrrcRecommendation {
            action: input.recommendation.action,
            recommended_scene_action: input
                .recommendation
                .recommended_scene_action
                .map(crrc_scene_action_to_coordinator_scene_action),
        },
        roles: coordinator_roles,
        signals: EpiphanyCoordinatorSignals {
            modeling_result_status: input.modeling_result_status,
            verification_result_status: input.verification_result_status,
        },
        modeling_result_accepted: input.modeling_result_accepted,
        modeling_result_reviewable: input.modeling_result_reviewable,
        modeling_result_accepted_after_verification: input
            .modeling_result_accepted_after_verification,
        implementation_evidence_after_verification: input
            .implementation_evidence_after_verification,
        verification_result_cites_implementation_evidence: input
            .verification_result_cites_implementation_evidence,
        verification_result_covers_current_modeling: input
            .verification_result_covers_current_modeling,
        verification_result_accepted: input.verification_result_accepted,
        verification_result_allows_implementation: input.verification_result_allows_implementation,
        verification_result_needs_evidence: input.verification_result_needs_evidence,
        reorient_finding_accepted: input.reorient_finding_accepted,
    });
    EpiphanyCoordinatorStatus {
        decision,
        source_signals,
        roles: input.roles,
    }
}

pub fn derive_coordinator_finding_signals(
    state: Option<&EpiphanyThreadState>,
    modeling_finding: Option<&EpiphanyRoleFindingInterpretation>,
    verification_finding: Option<&EpiphanyRoleFindingInterpretation>,
    reorient_finding: Option<&EpiphanyReorientFindingInterpretation>,
) -> EpiphanyCoordinatorFindingSignals {
    let modeling_result_accepted = modeling_finding.as_ref().is_some_and(|finding| {
        state.is_some_and(|state| {
            role_finding_already_accepted(state, EpiphanyCoordinatorRoleId::Modeling, finding)
        })
    });
    let modeling_result_reviewable =
        modeling_finding.is_some_and(modeling_finding_has_reviewable_state_patch);
    let verification_result_accepted = verification_finding.as_ref().is_some_and(|finding| {
        state.is_some_and(|state| {
            role_finding_already_accepted(state, EpiphanyCoordinatorRoleId::Verification, finding)
        })
    });
    let verification_result_covers_current_modeling = state.is_none_or(|state| {
        verification_finding_covers_current_modeling(
            state,
            modeling_result_accepted,
            modeling_finding,
            verification_finding,
        )
    });
    let modeling_result_accepted_after_verification = state.is_some_and(|state| {
        role_finding_accepted_after(
            state,
            EpiphanyCoordinatorRoleId::Modeling,
            modeling_finding,
            EpiphanyCoordinatorRoleId::Verification,
            verification_finding,
        )
    });
    let implementation_evidence_after_verification = state.is_some_and(|state| {
        implementation_evidence_after_role_finding(
            state,
            EpiphanyCoordinatorRoleId::Verification,
            verification_finding,
        )
    });
    let verification_result_cites_implementation_evidence = state.is_some_and(|state| {
        role_finding_cites_implementation_evidence(state, verification_finding)
    });
    let verification_result_allows_implementation = verification_result_accepted
        && verification_finding.is_some_and(verification_finding_allows_implementation);
    let verification_result_needs_evidence = verification_result_accepted
        && verification_finding.is_some_and(verification_finding_needs_evidence);
    let reorient_finding_accepted = reorient_finding.as_ref().is_some_and(|finding| {
        state.is_some_and(|state| reorient_finding_already_accepted(state, finding))
    });

    EpiphanyCoordinatorFindingSignals {
        modeling_result_accepted,
        modeling_result_reviewable,
        modeling_result_accepted_after_verification,
        implementation_evidence_after_verification,
        verification_result_cites_implementation_evidence,
        verification_result_covers_current_modeling,
        verification_result_accepted,
        verification_result_allows_implementation,
        verification_result_needs_evidence,
        reorient_finding_accepted,
    }
}

pub fn reorient_finding_already_accepted(
    state: &EpiphanyThreadState,
    finding: &EpiphanyReorientFindingInterpretation,
) -> bool {
    if let Some(result_id) = finding.runtime_result_id.clone()
        && state.acceptance_receipts.iter().any(|receipt| {
            receipt.result_id == result_id
                && receipt.status == "accepted"
                && receipt.surface == "reorientAccept"
        })
    {
        return true;
    }

    false
}

fn modeling_finding_has_reviewable_state_patch(
    finding: &EpiphanyRoleFindingInterpretation,
) -> bool {
    finding
        .state_patch
        .as_ref()
        .is_some_and(|patch| super::modeling_role_state_patch_policy_errors(patch).is_empty())
}

fn role_finding_already_accepted(
    state: &EpiphanyThreadState,
    role_id: EpiphanyCoordinatorRoleId,
    finding: &EpiphanyRoleFindingInterpretation,
) -> bool {
    role_finding_accepted_index(state, role_id, finding).is_some()
}

fn role_finding_accepted_evidence_id(
    state: &EpiphanyThreadState,
    role_id: EpiphanyCoordinatorRoleId,
    finding: &EpiphanyRoleFindingInterpretation,
) -> Option<String> {
    role_finding_acceptance_receipt(state, role_id, finding)
        .and_then(|receipt| receipt.accepted_evidence_id.clone())
}

fn verification_finding_covers_current_modeling(
    state: &EpiphanyThreadState,
    modeling_result_accepted: bool,
    modeling_finding: Option<&EpiphanyRoleFindingInterpretation>,
    verification_finding: Option<&EpiphanyRoleFindingInterpretation>,
) -> bool {
    if !modeling_result_accepted {
        return true;
    }
    let Some(modeling_finding) = modeling_finding else {
        return true;
    };
    let Some(verification_finding) = verification_finding else {
        return false;
    };

    let mut modeling_evidence_ids: HashSet<String> =
        modeling_finding.evidence_ids.iter().cloned().collect();
    if let Some(accepted_id) = role_finding_accepted_evidence_id(
        state,
        EpiphanyCoordinatorRoleId::Modeling,
        modeling_finding,
    ) {
        modeling_evidence_ids.insert(accepted_id);
    }
    if modeling_evidence_ids.is_empty() {
        return true;
    }
    verification_finding
        .evidence_ids
        .iter()
        .any(|id| modeling_evidence_ids.contains(id))
}

fn role_finding_accepted_after(
    state: &EpiphanyThreadState,
    later_role_id: EpiphanyCoordinatorRoleId,
    later: Option<&EpiphanyRoleFindingInterpretation>,
    earlier_role_id: EpiphanyCoordinatorRoleId,
    earlier: Option<&EpiphanyRoleFindingInterpretation>,
) -> bool {
    let Some(later) = later else {
        return false;
    };
    let Some(later_index) = role_finding_accepted_order_index(state, later_role_id, later) else {
        return false;
    };
    let Some(earlier) = earlier else {
        return true;
    };
    let Some(earlier_index) = role_finding_accepted_order_index(state, earlier_role_id, earlier)
    else {
        return true;
    };
    later_index < earlier_index
}

fn implementation_evidence_after_role_finding(
    state: &EpiphanyThreadState,
    role_id: EpiphanyCoordinatorRoleId,
    earlier: Option<&EpiphanyRoleFindingInterpretation>,
) -> bool {
    let earlier_index =
        earlier.and_then(|finding| role_finding_accepted_index(state, role_id, finding));
    state
        .recent_evidence
        .iter()
        .enumerate()
        .find(|(index, evidence)| {
            evidence.kind == "implementation-audit"
                && earlier_index.is_none_or(|earlier_index| *index < earlier_index)
        })
        .is_some_and(|(_, evidence)| evidence.status == "ok")
}

fn role_finding_cites_implementation_evidence(
    state: &EpiphanyThreadState,
    finding: Option<&EpiphanyRoleFindingInterpretation>,
) -> bool {
    let Some(finding) = finding else {
        return false;
    };
    finding.evidence_ids.iter().any(|id| {
        state.recent_evidence.iter().any(|evidence| {
            evidence.id == *id && evidence.kind == "implementation-audit" && evidence.status == "ok"
        })
    })
}

fn role_finding_accepted_index(
    state: &EpiphanyThreadState,
    role_id: EpiphanyCoordinatorRoleId,
    finding: &EpiphanyRoleFindingInterpretation,
) -> Option<usize> {
    if let Some(receipt) = role_finding_acceptance_receipt(state, role_id, finding) {
        return receipt
            .accepted_evidence_id
            .as_ref()
            .and_then(|id| {
                state
                    .recent_evidence
                    .iter()
                    .position(|evidence| evidence.id == *id)
            })
            .or(Some(0));
    }
    None
}

fn role_finding_accepted_order_index(
    state: &EpiphanyThreadState,
    role_id: EpiphanyCoordinatorRoleId,
    finding: &EpiphanyRoleFindingInterpretation,
) -> Option<usize> {
    role_finding_acceptance_receipt_index(state, role_id, finding)
}

fn role_finding_acceptance_receipt<'a>(
    state: &'a EpiphanyThreadState,
    role_id: EpiphanyCoordinatorRoleId,
    finding: &EpiphanyRoleFindingInterpretation,
) -> Option<&'a epiphany_state_model::EpiphanyAcceptanceReceipt> {
    let result_id = finding.runtime_result_id.clone()?;
    state.acceptance_receipts.iter().find(|receipt| {
        receipt.result_id == result_id
            && receipt.status == "accepted"
            && receipt.surface == "roleAccept"
            && receipt.role_id == coordinator_role_label(role_id)
    })
}

fn role_finding_acceptance_receipt_index(
    state: &EpiphanyThreadState,
    role_id: EpiphanyCoordinatorRoleId,
    finding: &EpiphanyRoleFindingInterpretation,
) -> Option<usize> {
    let result_id = finding.runtime_result_id.clone()?;
    state.acceptance_receipts.iter().position(|receipt| {
        receipt.result_id == result_id
            && receipt.status == "accepted"
            && receipt.surface == "roleAccept"
            && receipt.role_id == coordinator_role_label(role_id)
    })
}

fn coordinator_role_label(role_id: EpiphanyCoordinatorRoleId) -> &'static str {
    match role_id {
        EpiphanyCoordinatorRoleId::Implementation => "implementation",
        EpiphanyCoordinatorRoleId::Imagination => "imagination",
        EpiphanyCoordinatorRoleId::Modeling => "modeling",
        EpiphanyCoordinatorRoleId::Verification => "verification",
        EpiphanyCoordinatorRoleId::Reorientation => "reorientation",
    }
}

fn verification_finding_allows_implementation(finding: &EpiphanyRoleFindingInterpretation) -> bool {
    finding
        .verdict
        .as_deref()
        .is_some_and(|verdict| verdict.eq_ignore_ascii_case("pass"))
}

fn verification_finding_needs_evidence(finding: &EpiphanyRoleFindingInterpretation) -> bool {
    finding
        .verdict
        .as_deref()
        .is_some_and(|verdict| verdict.eq_ignore_ascii_case("needs-evidence"))
}

fn coordinator_role_lane_from_role_board(
    lane: &EpiphanyRoleBoardLane,
) -> EpiphanyCoordinatorRoleLane {
    EpiphanyCoordinatorRoleLane {
        id: lane.id,
        status: lane.status,
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

    if input.signals.modeling_result_status == EpiphanyCoordinatorRoleResultStatus::Failed
        && !input.modeling_result_accepted
    {
        return build(
            EpiphanyCoordinatorAction::ReviewModelingResult,
            Some(EpiphanyCoordinatorRoleId::Modeling),
            Some(EpiphanyCoordinatorSceneAction::RoleResult),
            false,
            false,
            "The modeling/checkpoint worker failed; inspect the failed result before verification or implementation continues.",
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

        let review_failed_modeling = recommend_coordinator_action(EpiphanyCoordinatorInput {
            signals: EpiphanyCoordinatorSignals {
                modeling_result_status: EpiphanyCoordinatorRoleResultStatus::Failed,
                verification_result_status: EpiphanyCoordinatorRoleResultStatus::MissingBinding,
            },
            ..input()
        });
        assert_eq!(
            review_failed_modeling.action,
            EpiphanyCoordinatorAction::ReviewModelingResult
        );
        assert!(!review_failed_modeling.can_auto_run);

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
