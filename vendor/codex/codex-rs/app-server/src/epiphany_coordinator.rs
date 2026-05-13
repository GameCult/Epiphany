use codex_app_server_protocol::ThreadEpiphanyCoordinatorAction;
use codex_app_server_protocol::ThreadEpiphanyCoordinatorSignals;
use codex_app_server_protocol::ThreadEpiphanyCrrcAction;
use codex_app_server_protocol::ThreadEpiphanyCrrcRecommendation;
use codex_app_server_protocol::ThreadEpiphanyJob;
use codex_app_server_protocol::ThreadEpiphanyJobKind;
use codex_app_server_protocol::ThreadEpiphanyJobStatus;
use codex_app_server_protocol::ThreadEpiphanyPressure;
use codex_app_server_protocol::ThreadEpiphanyReorientAction;
use codex_app_server_protocol::ThreadEpiphanyReorientDecision;
use codex_app_server_protocol::ThreadEpiphanyReorientFinding;
use codex_app_server_protocol::ThreadEpiphanyReorientResultStatus;
use codex_app_server_protocol::ThreadEpiphanyReorientStateStatus;
use codex_app_server_protocol::ThreadEpiphanyRoleFinding;
use codex_app_server_protocol::ThreadEpiphanyRoleId;
use codex_app_server_protocol::ThreadEpiphanyRoleLane;
use codex_app_server_protocol::ThreadEpiphanyRoleResultStatus;
use codex_app_server_protocol::ThreadEpiphanyRoleStatus;
use codex_app_server_protocol::ThreadEpiphanySceneAction;
use codex_protocol::protocol::EpiphanyAcceptanceReceipt;
use codex_protocol::protocol::EpiphanyThreadState;
use epiphany_core::EpiphanyCoordinatorAction as CoreEpiphanyCoordinatorAction;
use epiphany_core::EpiphanyCoordinatorAutomationAction as CoreEpiphanyCoordinatorAutomationAction;
use epiphany_core::EpiphanyCoordinatorCrrcRecommendation;
use epiphany_core::EpiphanyCoordinatorDecision as CoreEpiphanyCoordinatorDecision;
use epiphany_core::EpiphanyCoordinatorInput;
use epiphany_core::EpiphanyCoordinatorRoleId as CoreEpiphanyCoordinatorRoleId;
use epiphany_core::EpiphanyCoordinatorRoleLane as CoreEpiphanyCoordinatorRoleLane;
use epiphany_core::EpiphanyCoordinatorRoleResultStatus as CoreEpiphanyCoordinatorRoleResultStatus;
use epiphany_core::EpiphanyCoordinatorRoleStatus as CoreEpiphanyCoordinatorRoleStatus;
use epiphany_core::EpiphanyCoordinatorSceneAction as CoreEpiphanyCoordinatorSceneAction;
use epiphany_core::EpiphanyCoordinatorSignals;
use epiphany_core::EpiphanyCrrcAction as CoreEpiphanyCrrcAction;
use epiphany_core::EpiphanyCrrcInput;
use epiphany_core::EpiphanyCrrcRecommendation as CoreEpiphanyCrrcRecommendation;
use epiphany_core::EpiphanyCrrcReorientAction as CoreEpiphanyCrrcReorientAction;
use epiphany_core::EpiphanyCrrcResultStatus as CoreEpiphanyCrrcResultStatus;
use epiphany_core::EpiphanyCrrcSceneAction as CoreEpiphanyCrrcSceneAction;
use epiphany_core::EpiphanyCrrcStateStatus as CoreEpiphanyCrrcStateStatus;
use epiphany_core::EpiphanyRoleBoardCheckpointSummary;
use epiphany_core::EpiphanyRoleBoardInput;
use epiphany_core::EpiphanyRoleBoardJob;
use epiphany_core::EpiphanyRoleBoardJobStatus;
use epiphany_core::EpiphanyRoleBoardLane;
use epiphany_core::EpiphanyRoleBoardPlanningSummary;
use epiphany_core::coordinator_automation_action;
use epiphany_core::derive_role_board;
use epiphany_core::recommend_coordinator_action;
use epiphany_core::recommend_crrc_action;
use epiphany_core::render_role_board_note;
use epiphany_core::select_coordinator_automation_action;

use crate::epiphany_launch::EPIPHANY_IMAGINATION_OWNER_ROLE;
use crate::epiphany_launch::EPIPHANY_IMAGINATION_ROLE_BINDING_ID;
use crate::epiphany_launch::EPIPHANY_MODELING_ROLE_BINDING_ID;
use crate::epiphany_launch::EPIPHANY_REORIENT_OWNER_ROLE;
use crate::epiphany_launch::EPIPHANY_VERIFICATION_ROLE_BINDING_ID;
use crate::epiphany_launch::epiphany_role_label;

use std::collections::HashSet;

pub(super) fn map_epiphany_crrc_recommendation(
    loaded: bool,
    state_status: ThreadEpiphanyReorientStateStatus,
    pressure: &ThreadEpiphanyPressure,
    decision: &ThreadEpiphanyReorientDecision,
    result_status: ThreadEpiphanyReorientResultStatus,
    checkpoint_present: bool,
    finding_present: bool,
    finding_accepted: bool,
) -> ThreadEpiphanyCrrcRecommendation {
    map_core_crrc_recommendation(recommend_crrc_action(EpiphanyCrrcInput {
        loaded,
        state_status: map_core_crrc_state_status(state_status),
        should_prepare_compaction: pressure.should_prepare_compaction,
        reorient_action: map_core_crrc_reorient_action(decision.action),
        result_status: map_core_crrc_result_status(result_status),
        checkpoint_present,
        finding_present,
        finding_accepted,
    }))
}

fn map_core_crrc_state_status(
    status: ThreadEpiphanyReorientStateStatus,
) -> CoreEpiphanyCrrcStateStatus {
    match status {
        ThreadEpiphanyReorientStateStatus::Missing => CoreEpiphanyCrrcStateStatus::Missing,
        ThreadEpiphanyReorientStateStatus::Ready => CoreEpiphanyCrrcStateStatus::Ready,
    }
}

fn map_core_crrc_reorient_action(
    action: ThreadEpiphanyReorientAction,
) -> CoreEpiphanyCrrcReorientAction {
    match action {
        ThreadEpiphanyReorientAction::Resume => CoreEpiphanyCrrcReorientAction::Resume,
        ThreadEpiphanyReorientAction::Regather => CoreEpiphanyCrrcReorientAction::Regather,
    }
}

fn map_core_crrc_result_status(
    status: ThreadEpiphanyReorientResultStatus,
) -> CoreEpiphanyCrrcResultStatus {
    match status {
        ThreadEpiphanyReorientResultStatus::MissingState => {
            CoreEpiphanyCrrcResultStatus::MissingState
        }
        ThreadEpiphanyReorientResultStatus::MissingBinding => {
            CoreEpiphanyCrrcResultStatus::MissingBinding
        }
        ThreadEpiphanyReorientResultStatus::BackendUnavailable => {
            CoreEpiphanyCrrcResultStatus::BackendUnavailable
        }
        ThreadEpiphanyReorientResultStatus::BackendMissing => {
            CoreEpiphanyCrrcResultStatus::BackendMissing
        }
        ThreadEpiphanyReorientResultStatus::Pending => CoreEpiphanyCrrcResultStatus::Pending,
        ThreadEpiphanyReorientResultStatus::Running => CoreEpiphanyCrrcResultStatus::Running,
        ThreadEpiphanyReorientResultStatus::Completed => CoreEpiphanyCrrcResultStatus::Completed,
        ThreadEpiphanyReorientResultStatus::Failed => CoreEpiphanyCrrcResultStatus::Failed,
        ThreadEpiphanyReorientResultStatus::Cancelled => CoreEpiphanyCrrcResultStatus::Cancelled,
    }
}

fn map_core_crrc_action(action: CoreEpiphanyCrrcAction) -> ThreadEpiphanyCrrcAction {
    match action {
        CoreEpiphanyCrrcAction::Continue => ThreadEpiphanyCrrcAction::Continue,
        CoreEpiphanyCrrcAction::PrepareCheckpoint => ThreadEpiphanyCrrcAction::PrepareCheckpoint,
        CoreEpiphanyCrrcAction::LaunchReorientWorker => {
            ThreadEpiphanyCrrcAction::LaunchReorientWorker
        }
        CoreEpiphanyCrrcAction::WaitForReorientWorker => {
            ThreadEpiphanyCrrcAction::WaitForReorientWorker
        }
        CoreEpiphanyCrrcAction::ReviewReorientResult => {
            ThreadEpiphanyCrrcAction::ReviewReorientResult
        }
        CoreEpiphanyCrrcAction::AcceptReorientResult => {
            ThreadEpiphanyCrrcAction::AcceptReorientResult
        }
        CoreEpiphanyCrrcAction::RegatherManually => ThreadEpiphanyCrrcAction::RegatherManually,
    }
}

fn map_core_crrc_scene_action(action: CoreEpiphanyCrrcSceneAction) -> ThreadEpiphanySceneAction {
    match action {
        CoreEpiphanyCrrcSceneAction::Update => ThreadEpiphanySceneAction::Update,
        CoreEpiphanyCrrcSceneAction::Reorient => ThreadEpiphanySceneAction::Reorient,
        CoreEpiphanyCrrcSceneAction::ReorientLaunch => ThreadEpiphanySceneAction::ReorientLaunch,
        CoreEpiphanyCrrcSceneAction::ReorientResult => ThreadEpiphanySceneAction::ReorientResult,
        CoreEpiphanyCrrcSceneAction::ReorientAccept => ThreadEpiphanySceneAction::ReorientAccept,
    }
}

fn map_core_crrc_recommendation(
    recommendation: CoreEpiphanyCrrcRecommendation,
) -> ThreadEpiphanyCrrcRecommendation {
    ThreadEpiphanyCrrcRecommendation {
        action: map_core_crrc_action(recommendation.action),
        recommended_scene_action: recommendation
            .recommended_scene_action
            .map(map_core_crrc_scene_action),
        reason: recommendation.reason,
    }
}

pub(super) struct EpiphanyCoordinatorDecision {
    pub(super) action: ThreadEpiphanyCoordinatorAction,
    pub(super) target_role: Option<ThreadEpiphanyRoleId>,
    pub(super) recommended_scene_action: Option<ThreadEpiphanySceneAction>,
    pub(super) requires_review: bool,
    pub(super) can_auto_run: bool,
    pub(super) reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum EpiphanyCoordinatorAutomationAction {
    None,
    CompactRehydrateReorient,
    LaunchReorientWorker,
}

pub(super) fn map_epiphany_coordinator_automation_action(
    decision: &EpiphanyCoordinatorDecision,
) -> EpiphanyCoordinatorAutomationAction {
    map_protocol_coordinator_automation_action(coordinator_automation_action(
        &map_core_coordinator_decision_from_protocol(decision),
    ))
}

pub(super) fn select_epiphany_coordinator_automation_action(
    decision: &EpiphanyCoordinatorDecision,
    force_checkpoint_compaction: bool,
) -> EpiphanyCoordinatorAutomationAction {
    map_protocol_coordinator_automation_action(select_coordinator_automation_action(
        &map_core_coordinator_decision_from_protocol(decision),
        force_checkpoint_compaction,
    ))
}

pub(super) fn map_epiphany_coordinator(
    state_status: ThreadEpiphanyReorientStateStatus,
    checkpoint_present: bool,
    pressure: &ThreadEpiphanyPressure,
    recommendation: &ThreadEpiphanyCrrcRecommendation,
    roles: &[ThreadEpiphanyRoleLane],
    signals: &ThreadEpiphanyCoordinatorSignals,
    modeling_result_accepted: bool,
    modeling_result_reviewable: bool,
    modeling_result_accepted_after_verification: bool,
    implementation_evidence_after_verification: bool,
    verification_result_cites_implementation_evidence: bool,
    verification_result_covers_current_modeling: bool,
    verification_result_accepted: bool,
    verification_result_allows_implementation: bool,
    verification_result_needs_evidence: bool,
    reorient_finding_accepted: bool,
) -> EpiphanyCoordinatorDecision {
    map_core_coordinator_decision(recommend_coordinator_action(EpiphanyCoordinatorInput {
        state_status: map_core_crrc_state_status(state_status),
        checkpoint_present,
        should_prepare_compaction: pressure.should_prepare_compaction,
        recommendation: map_core_coordinator_crrc_recommendation(recommendation),
        roles: roles.iter().map(map_core_coordinator_role_lane).collect(),
        signals: EpiphanyCoordinatorSignals {
            modeling_result_status: map_core_coordinator_role_result_status(
                signals.modeling_result_status,
            ),
            verification_result_status: map_core_coordinator_role_result_status(
                signals.verification_result_status,
            ),
        },
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
    }))
}

fn map_core_coordinator_crrc_recommendation(
    recommendation: &ThreadEpiphanyCrrcRecommendation,
) -> EpiphanyCoordinatorCrrcRecommendation {
    EpiphanyCoordinatorCrrcRecommendation {
        action: map_core_crrc_action_from_protocol(recommendation.action),
        recommended_scene_action: recommendation
            .recommended_scene_action
            .map(map_core_coordinator_scene_action_from_protocol),
    }
}

fn map_core_crrc_action_from_protocol(action: ThreadEpiphanyCrrcAction) -> CoreEpiphanyCrrcAction {
    match action {
        ThreadEpiphanyCrrcAction::Continue => CoreEpiphanyCrrcAction::Continue,
        ThreadEpiphanyCrrcAction::PrepareCheckpoint => CoreEpiphanyCrrcAction::PrepareCheckpoint,
        ThreadEpiphanyCrrcAction::LaunchReorientWorker => {
            CoreEpiphanyCrrcAction::LaunchReorientWorker
        }
        ThreadEpiphanyCrrcAction::WaitForReorientWorker => {
            CoreEpiphanyCrrcAction::WaitForReorientWorker
        }
        ThreadEpiphanyCrrcAction::ReviewReorientResult => {
            CoreEpiphanyCrrcAction::ReviewReorientResult
        }
        ThreadEpiphanyCrrcAction::AcceptReorientResult => {
            CoreEpiphanyCrrcAction::AcceptReorientResult
        }
        ThreadEpiphanyCrrcAction::RegatherManually => CoreEpiphanyCrrcAction::RegatherManually,
    }
}

fn map_core_coordinator_role_lane(
    role: &ThreadEpiphanyRoleLane,
) -> CoreEpiphanyCoordinatorRoleLane {
    CoreEpiphanyCoordinatorRoleLane {
        id: map_core_coordinator_role_id(role.id),
        status: map_core_coordinator_role_status(role.status),
    }
}

fn map_core_coordinator_role_id(role_id: ThreadEpiphanyRoleId) -> CoreEpiphanyCoordinatorRoleId {
    match role_id {
        ThreadEpiphanyRoleId::Implementation => CoreEpiphanyCoordinatorRoleId::Implementation,
        ThreadEpiphanyRoleId::Imagination => CoreEpiphanyCoordinatorRoleId::Imagination,
        ThreadEpiphanyRoleId::Modeling => CoreEpiphanyCoordinatorRoleId::Modeling,
        ThreadEpiphanyRoleId::Verification => CoreEpiphanyCoordinatorRoleId::Verification,
        ThreadEpiphanyRoleId::Reorientation => CoreEpiphanyCoordinatorRoleId::Reorientation,
    }
}

fn map_protocol_coordinator_role_id(
    role_id: CoreEpiphanyCoordinatorRoleId,
) -> ThreadEpiphanyRoleId {
    match role_id {
        CoreEpiphanyCoordinatorRoleId::Implementation => ThreadEpiphanyRoleId::Implementation,
        CoreEpiphanyCoordinatorRoleId::Imagination => ThreadEpiphanyRoleId::Imagination,
        CoreEpiphanyCoordinatorRoleId::Modeling => ThreadEpiphanyRoleId::Modeling,
        CoreEpiphanyCoordinatorRoleId::Verification => ThreadEpiphanyRoleId::Verification,
        CoreEpiphanyCoordinatorRoleId::Reorientation => ThreadEpiphanyRoleId::Reorientation,
    }
}

fn map_core_coordinator_role_id_from_protocol(
    role_id: ThreadEpiphanyRoleId,
) -> CoreEpiphanyCoordinatorRoleId {
    map_core_coordinator_role_id(role_id)
}

fn map_core_coordinator_role_status(
    status: ThreadEpiphanyRoleStatus,
) -> CoreEpiphanyCoordinatorRoleStatus {
    match status {
        ThreadEpiphanyRoleStatus::Ready => CoreEpiphanyCoordinatorRoleStatus::Ready,
        ThreadEpiphanyRoleStatus::Needed => CoreEpiphanyCoordinatorRoleStatus::Needed,
        ThreadEpiphanyRoleStatus::Running => CoreEpiphanyCoordinatorRoleStatus::Running,
        ThreadEpiphanyRoleStatus::Waiting => CoreEpiphanyCoordinatorRoleStatus::Waiting,
        ThreadEpiphanyRoleStatus::Review => CoreEpiphanyCoordinatorRoleStatus::Review,
        ThreadEpiphanyRoleStatus::Blocked => CoreEpiphanyCoordinatorRoleStatus::Blocked,
        ThreadEpiphanyRoleStatus::Unavailable => CoreEpiphanyCoordinatorRoleStatus::Unavailable,
        ThreadEpiphanyRoleStatus::Completed => CoreEpiphanyCoordinatorRoleStatus::Completed,
    }
}

fn map_core_coordinator_role_result_status(
    status: ThreadEpiphanyRoleResultStatus,
) -> CoreEpiphanyCoordinatorRoleResultStatus {
    match status {
        ThreadEpiphanyRoleResultStatus::MissingState => {
            CoreEpiphanyCoordinatorRoleResultStatus::MissingState
        }
        ThreadEpiphanyRoleResultStatus::MissingBinding => {
            CoreEpiphanyCoordinatorRoleResultStatus::MissingBinding
        }
        ThreadEpiphanyRoleResultStatus::BackendUnavailable => {
            CoreEpiphanyCoordinatorRoleResultStatus::BackendUnavailable
        }
        ThreadEpiphanyRoleResultStatus::BackendMissing => {
            CoreEpiphanyCoordinatorRoleResultStatus::BackendMissing
        }
        ThreadEpiphanyRoleResultStatus::Pending => CoreEpiphanyCoordinatorRoleResultStatus::Pending,
        ThreadEpiphanyRoleResultStatus::Running => CoreEpiphanyCoordinatorRoleResultStatus::Running,
        ThreadEpiphanyRoleResultStatus::Completed => {
            CoreEpiphanyCoordinatorRoleResultStatus::Completed
        }
        ThreadEpiphanyRoleResultStatus::Failed => CoreEpiphanyCoordinatorRoleResultStatus::Failed,
        ThreadEpiphanyRoleResultStatus::Cancelled => {
            CoreEpiphanyCoordinatorRoleResultStatus::Cancelled
        }
    }
}

fn map_core_coordinator_scene_action_from_protocol(
    action: ThreadEpiphanySceneAction,
) -> CoreEpiphanyCoordinatorSceneAction {
    match action {
        ThreadEpiphanySceneAction::Update => CoreEpiphanyCoordinatorSceneAction::Update,
        ThreadEpiphanySceneAction::Reorient => CoreEpiphanyCoordinatorSceneAction::Reorient,
        ThreadEpiphanySceneAction::ReorientLaunch => {
            CoreEpiphanyCoordinatorSceneAction::ReorientLaunch
        }
        ThreadEpiphanySceneAction::ReorientResult => {
            CoreEpiphanyCoordinatorSceneAction::ReorientResult
        }
        ThreadEpiphanySceneAction::ReorientAccept => {
            CoreEpiphanyCoordinatorSceneAction::ReorientAccept
        }
        ThreadEpiphanySceneAction::RoleLaunch => CoreEpiphanyCoordinatorSceneAction::RoleLaunch,
        ThreadEpiphanySceneAction::RoleResult => CoreEpiphanyCoordinatorSceneAction::RoleResult,
        ThreadEpiphanySceneAction::Index
        | ThreadEpiphanySceneAction::Retrieve
        | ThreadEpiphanySceneAction::Distill
        | ThreadEpiphanySceneAction::Context
        | ThreadEpiphanySceneAction::Planning
        | ThreadEpiphanySceneAction::GraphQuery
        | ThreadEpiphanySceneAction::Jobs
        | ThreadEpiphanySceneAction::Roles
        | ThreadEpiphanySceneAction::Coordinator
        | ThreadEpiphanySceneAction::RoleAccept
        | ThreadEpiphanySceneAction::JobLaunch
        | ThreadEpiphanySceneAction::JobInterrupt
        | ThreadEpiphanySceneAction::Freshness
        | ThreadEpiphanySceneAction::Pressure
        | ThreadEpiphanySceneAction::Crrc
        | ThreadEpiphanySceneAction::Propose
        | ThreadEpiphanySceneAction::Promote => {
            unreachable!("unsupported CRRC coordinator scene action: {action:?}")
        }
    }
}

fn map_protocol_coordinator_scene_action(
    action: CoreEpiphanyCoordinatorSceneAction,
) -> ThreadEpiphanySceneAction {
    match action {
        CoreEpiphanyCoordinatorSceneAction::Update => ThreadEpiphanySceneAction::Update,
        CoreEpiphanyCoordinatorSceneAction::Reorient => ThreadEpiphanySceneAction::Reorient,
        CoreEpiphanyCoordinatorSceneAction::ReorientLaunch => {
            ThreadEpiphanySceneAction::ReorientLaunch
        }
        CoreEpiphanyCoordinatorSceneAction::ReorientResult => {
            ThreadEpiphanySceneAction::ReorientResult
        }
        CoreEpiphanyCoordinatorSceneAction::ReorientAccept => {
            ThreadEpiphanySceneAction::ReorientAccept
        }
        CoreEpiphanyCoordinatorSceneAction::RoleLaunch => ThreadEpiphanySceneAction::RoleLaunch,
        CoreEpiphanyCoordinatorSceneAction::RoleResult => ThreadEpiphanySceneAction::RoleResult,
    }
}

fn map_core_coordinator_action_from_protocol(
    action: ThreadEpiphanyCoordinatorAction,
) -> CoreEpiphanyCoordinatorAction {
    match action {
        ThreadEpiphanyCoordinatorAction::PrepareCheckpoint => {
            CoreEpiphanyCoordinatorAction::PrepareCheckpoint
        }
        ThreadEpiphanyCoordinatorAction::CompactRehydrateReorient => {
            CoreEpiphanyCoordinatorAction::CompactRehydrateReorient
        }
        ThreadEpiphanyCoordinatorAction::LaunchReorientWorker => {
            CoreEpiphanyCoordinatorAction::LaunchReorientWorker
        }
        ThreadEpiphanyCoordinatorAction::WaitForReorientWorker => {
            CoreEpiphanyCoordinatorAction::WaitForReorientWorker
        }
        ThreadEpiphanyCoordinatorAction::ReviewReorientResult => {
            CoreEpiphanyCoordinatorAction::ReviewReorientResult
        }
        ThreadEpiphanyCoordinatorAction::RegatherManually => {
            CoreEpiphanyCoordinatorAction::RegatherManually
        }
        ThreadEpiphanyCoordinatorAction::LaunchModeling => {
            CoreEpiphanyCoordinatorAction::LaunchModeling
        }
        ThreadEpiphanyCoordinatorAction::ReviewModelingResult => {
            CoreEpiphanyCoordinatorAction::ReviewModelingResult
        }
        ThreadEpiphanyCoordinatorAction::LaunchVerification => {
            CoreEpiphanyCoordinatorAction::LaunchVerification
        }
        ThreadEpiphanyCoordinatorAction::ReviewVerificationResult => {
            CoreEpiphanyCoordinatorAction::ReviewVerificationResult
        }
        ThreadEpiphanyCoordinatorAction::ContinueImplementation => {
            CoreEpiphanyCoordinatorAction::ContinueImplementation
        }
    }
}

fn map_protocol_coordinator_action(
    action: CoreEpiphanyCoordinatorAction,
) -> ThreadEpiphanyCoordinatorAction {
    match action {
        CoreEpiphanyCoordinatorAction::PrepareCheckpoint => {
            ThreadEpiphanyCoordinatorAction::PrepareCheckpoint
        }
        CoreEpiphanyCoordinatorAction::CompactRehydrateReorient => {
            ThreadEpiphanyCoordinatorAction::CompactRehydrateReorient
        }
        CoreEpiphanyCoordinatorAction::LaunchReorientWorker => {
            ThreadEpiphanyCoordinatorAction::LaunchReorientWorker
        }
        CoreEpiphanyCoordinatorAction::WaitForReorientWorker => {
            ThreadEpiphanyCoordinatorAction::WaitForReorientWorker
        }
        CoreEpiphanyCoordinatorAction::ReviewReorientResult => {
            ThreadEpiphanyCoordinatorAction::ReviewReorientResult
        }
        CoreEpiphanyCoordinatorAction::RegatherManually => {
            ThreadEpiphanyCoordinatorAction::RegatherManually
        }
        CoreEpiphanyCoordinatorAction::LaunchModeling => {
            ThreadEpiphanyCoordinatorAction::LaunchModeling
        }
        CoreEpiphanyCoordinatorAction::ReviewModelingResult => {
            ThreadEpiphanyCoordinatorAction::ReviewModelingResult
        }
        CoreEpiphanyCoordinatorAction::LaunchVerification => {
            ThreadEpiphanyCoordinatorAction::LaunchVerification
        }
        CoreEpiphanyCoordinatorAction::ReviewVerificationResult => {
            ThreadEpiphanyCoordinatorAction::ReviewVerificationResult
        }
        CoreEpiphanyCoordinatorAction::ContinueImplementation => {
            ThreadEpiphanyCoordinatorAction::ContinueImplementation
        }
    }
}

fn map_core_coordinator_decision_from_protocol(
    decision: &EpiphanyCoordinatorDecision,
) -> CoreEpiphanyCoordinatorDecision {
    CoreEpiphanyCoordinatorDecision {
        action: map_core_coordinator_action_from_protocol(decision.action),
        target_role: decision
            .target_role
            .map(map_core_coordinator_role_id_from_protocol),
        recommended_scene_action: decision
            .recommended_scene_action
            .map(map_core_coordinator_scene_action_from_protocol),
        requires_review: decision.requires_review,
        can_auto_run: decision.can_auto_run,
        reason: decision.reason.clone(),
    }
}

fn map_core_coordinator_decision(
    decision: CoreEpiphanyCoordinatorDecision,
) -> EpiphanyCoordinatorDecision {
    EpiphanyCoordinatorDecision {
        action: map_protocol_coordinator_action(decision.action),
        target_role: decision.target_role.map(map_protocol_coordinator_role_id),
        recommended_scene_action: decision
            .recommended_scene_action
            .map(map_protocol_coordinator_scene_action),
        requires_review: decision.requires_review,
        can_auto_run: decision.can_auto_run,
        reason: decision.reason,
    }
}

fn map_protocol_coordinator_automation_action(
    action: CoreEpiphanyCoordinatorAutomationAction,
) -> EpiphanyCoordinatorAutomationAction {
    match action {
        CoreEpiphanyCoordinatorAutomationAction::None => EpiphanyCoordinatorAutomationAction::None,
        CoreEpiphanyCoordinatorAutomationAction::CompactRehydrateReorient => {
            EpiphanyCoordinatorAutomationAction::CompactRehydrateReorient
        }
        CoreEpiphanyCoordinatorAutomationAction::LaunchReorientWorker => {
            EpiphanyCoordinatorAutomationAction::LaunchReorientWorker
        }
    }
}

pub(super) fn epiphany_reorient_finding_already_accepted(
    state: &EpiphanyThreadState,
    finding: &ThreadEpiphanyReorientFinding,
) -> bool {
    if let Some(result_id) = reorient_finding_runtime_result_id(finding)
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

pub(super) fn epiphany_role_finding_already_accepted(
    state: &EpiphanyThreadState,
    finding: &ThreadEpiphanyRoleFinding,
) -> bool {
    epiphany_role_finding_accepted_index(state, finding).is_some()
}

pub(super) fn epiphany_role_finding_accepted_evidence_id(
    state: &EpiphanyThreadState,
    finding: &ThreadEpiphanyRoleFinding,
) -> Option<String> {
    epiphany_role_finding_acceptance_receipt(state, finding)
        .and_then(|receipt| receipt.accepted_evidence_id.clone())
}

pub(super) fn epiphany_verification_finding_covers_current_modeling(
    state: &EpiphanyThreadState,
    modeling_result_accepted: bool,
    modeling_finding: Option<&ThreadEpiphanyRoleFinding>,
    verification_finding: Option<&ThreadEpiphanyRoleFinding>,
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
    if let Some(accepted_id) = epiphany_role_finding_accepted_evidence_id(state, modeling_finding) {
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

pub(super) fn role_finding_accepted_after(
    state: &EpiphanyThreadState,
    later: Option<&ThreadEpiphanyRoleFinding>,
    earlier: Option<&ThreadEpiphanyRoleFinding>,
) -> bool {
    let Some(later) = later else {
        return false;
    };
    let Some(later_index) = epiphany_role_finding_accepted_order_index(state, later) else {
        return false;
    };
    let Some(earlier) = earlier else {
        return true;
    };
    let Some(earlier_index) = epiphany_role_finding_accepted_order_index(state, earlier) else {
        return true;
    };
    later_index < earlier_index
}

pub(super) fn implementation_evidence_after_role_finding(
    state: &EpiphanyThreadState,
    earlier: Option<&ThreadEpiphanyRoleFinding>,
) -> bool {
    let earlier_index =
        earlier.and_then(|finding| epiphany_role_finding_accepted_index(state, finding));
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

pub(super) fn epiphany_role_finding_cites_implementation_evidence(
    state: &EpiphanyThreadState,
    finding: Option<&ThreadEpiphanyRoleFinding>,
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

fn epiphany_role_finding_accepted_index(
    state: &EpiphanyThreadState,
    finding: &ThreadEpiphanyRoleFinding,
) -> Option<usize> {
    if let Some(receipt) = epiphany_role_finding_acceptance_receipt(state, finding) {
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

fn epiphany_role_finding_accepted_order_index(
    state: &EpiphanyThreadState,
    finding: &ThreadEpiphanyRoleFinding,
) -> Option<usize> {
    epiphany_role_finding_acceptance_receipt_index(state, finding)
}

fn epiphany_role_finding_acceptance_receipt<'a>(
    state: &'a EpiphanyThreadState,
    finding: &ThreadEpiphanyRoleFinding,
) -> Option<&'a EpiphanyAcceptanceReceipt> {
    let result_id = role_finding_runtime_result_id(finding)?;
    state.acceptance_receipts.iter().find(|receipt| {
        receipt.result_id == result_id
            && receipt.status == "accepted"
            && receipt.surface == "roleAccept"
            && receipt.role_id == epiphany_role_label(finding.role_id)
    })
}

fn epiphany_role_finding_acceptance_receipt_index(
    state: &EpiphanyThreadState,
    finding: &ThreadEpiphanyRoleFinding,
) -> Option<usize> {
    let result_id = role_finding_runtime_result_id(finding)?;
    state.acceptance_receipts.iter().position(|receipt| {
        receipt.result_id == result_id
            && receipt.status == "accepted"
            && receipt.surface == "roleAccept"
            && receipt.role_id == epiphany_role_label(finding.role_id)
    })
}

pub(super) fn epiphany_verification_finding_allows_implementation(
    finding: &ThreadEpiphanyRoleFinding,
) -> bool {
    finding.role_id == ThreadEpiphanyRoleId::Verification
        && finding
            .verdict
            .as_deref()
            .is_some_and(|verdict| verdict.eq_ignore_ascii_case("pass"))
}

pub(super) fn epiphany_verification_finding_needs_evidence(
    finding: &ThreadEpiphanyRoleFinding,
) -> bool {
    finding.role_id == ThreadEpiphanyRoleId::Verification
        && finding
            .verdict
            .as_deref()
            .is_some_and(|verdict| verdict.eq_ignore_ascii_case("needs-evidence"))
}

fn role_finding_runtime_result_id(finding: &ThreadEpiphanyRoleFinding) -> Option<String> {
    finding.runtime_result_id.clone()
}

fn reorient_finding_runtime_result_id(finding: &ThreadEpiphanyReorientFinding) -> Option<String> {
    finding.runtime_result_id.clone()
}

pub(super) fn map_epiphany_roles(
    state: Option<&EpiphanyThreadState>,
    jobs: &[ThreadEpiphanyJob],
    decision: &ThreadEpiphanyReorientDecision,
    pressure: &ThreadEpiphanyPressure,
    recommendation: &ThreadEpiphanyCrrcRecommendation,
    result_status: ThreadEpiphanyReorientResultStatus,
    reorient_job: Option<&ThreadEpiphanyJob>,
) -> Vec<ThreadEpiphanyRoleLane> {
    let planning = state.map(|state| &state.planning);
    let checkpoint = state.and_then(|state| state.investigation_checkpoint.as_ref());
    let mut source_jobs = jobs.to_vec();
    if let Some(job) = reorient_job
        && !source_jobs
            .iter()
            .any(|source_job| source_job.id == job.id && source_job.owner_role == job.owner_role)
    {
        source_jobs.push(job.clone());
    }
    derive_role_board(EpiphanyRoleBoardInput {
        state_present: state.is_some(),
        planning: EpiphanyRoleBoardPlanningSummary {
            capture_count: planning
                .map(|planning| planning.captures.len())
                .unwrap_or(0),
            backlog_item_count: planning
                .map(|planning| planning.backlog_items.len())
                .unwrap_or(0),
            roadmap_stream_count: planning
                .map(|planning| planning.roadmap_streams.len())
                .unwrap_or(0),
            objective_draft_count: planning
                .map(|planning| planning.objective_drafts.len())
                .unwrap_or(0),
        },
        checkpoint: checkpoint.map(|checkpoint| EpiphanyRoleBoardCheckpointSummary {
            disposition: Some(format!("{:?}", checkpoint.disposition)),
            next_action: checkpoint.next_action.clone(),
        }),
        reorient_next_action: decision.next_action.clone(),
        jobs: source_jobs.iter().map(map_core_role_board_job).collect(),
        crrc_action: map_core_crrc_action_from_protocol(recommendation.action),
        crrc_recommended_scene_action: recommendation
            .recommended_scene_action
            .map(map_core_coordinator_scene_action_from_protocol),
        crrc_reason: recommendation.reason.clone(),
        reorient_decision_action: format!("{:?}", decision.action),
        pressure_level: format!("{:?}", pressure.level),
        reorient_result_status: map_core_crrc_result_status(result_status),
        reorient_job: reorient_job.map(map_core_role_board_job),
        imagination_binding_id: EPIPHANY_IMAGINATION_ROLE_BINDING_ID.to_string(),
        modeling_binding_id: EPIPHANY_MODELING_ROLE_BINDING_ID.to_string(),
        verification_binding_id: EPIPHANY_VERIFICATION_ROLE_BINDING_ID.to_string(),
        reorient_owner_role: EPIPHANY_REORIENT_OWNER_ROLE.to_string(),
        imagination_owner_role: EPIPHANY_IMAGINATION_OWNER_ROLE.to_string(),
    })
    .into_iter()
    .map(|lane| map_protocol_role_board_lane(lane, &source_jobs))
    .collect()
}

fn map_core_role_board_job(job: &ThreadEpiphanyJob) -> EpiphanyRoleBoardJob {
    EpiphanyRoleBoardJob {
        id: job.id.clone(),
        owner_role: job.owner_role.clone(),
        status: map_core_role_board_job_status(job.status),
        progress_note: job.progress_note.clone(),
        blocking_reason: job.blocking_reason.clone(),
    }
}

fn map_core_role_board_job_status(status: ThreadEpiphanyJobStatus) -> EpiphanyRoleBoardJobStatus {
    match status {
        ThreadEpiphanyJobStatus::Idle => EpiphanyRoleBoardJobStatus::Idle,
        ThreadEpiphanyJobStatus::Needed => EpiphanyRoleBoardJobStatus::Needed,
        ThreadEpiphanyJobStatus::Pending => EpiphanyRoleBoardJobStatus::Pending,
        ThreadEpiphanyJobStatus::Running => EpiphanyRoleBoardJobStatus::Running,
        ThreadEpiphanyJobStatus::Completed => EpiphanyRoleBoardJobStatus::Completed,
        ThreadEpiphanyJobStatus::Failed => EpiphanyRoleBoardJobStatus::Failed,
        ThreadEpiphanyJobStatus::Cancelled => EpiphanyRoleBoardJobStatus::Cancelled,
        ThreadEpiphanyJobStatus::Blocked => EpiphanyRoleBoardJobStatus::Blocked,
        ThreadEpiphanyJobStatus::Unavailable => EpiphanyRoleBoardJobStatus::Unavailable,
    }
}

fn map_protocol_role_board_lane(
    lane: EpiphanyRoleBoardLane,
    source_jobs: &[ThreadEpiphanyJob],
) -> ThreadEpiphanyRoleLane {
    ThreadEpiphanyRoleLane {
        id: map_protocol_coordinator_role_id(lane.id),
        title: lane.title,
        owner_role: lane.owner_role,
        status: map_protocol_coordinator_role_status(lane.status),
        note: lane.note,
        jobs: lane
            .jobs
            .iter()
            .map(|job| map_protocol_role_board_job(job, source_jobs))
            .collect(),
        authority_scopes: lane.authority_scopes,
        recommended_action: lane
            .recommended_action
            .map(map_protocol_coordinator_scene_action),
    }
}

fn map_protocol_role_board_job(
    job: &EpiphanyRoleBoardJob,
    source_jobs: &[ThreadEpiphanyJob],
) -> ThreadEpiphanyJob {
    source_jobs
        .iter()
        .find(|source_job| source_job.id == job.id && source_job.owner_role == job.owner_role)
        .cloned()
        .unwrap_or_else(|| ThreadEpiphanyJob {
            id: job.id.clone(),
            kind: ThreadEpiphanyJobKind::Specialist,
            scope: job.id.clone(),
            owner_role: job.owner_role.clone(),
            launcher_job_id: None,
            authority_scope: None,
            backend_job_id: None,
            status: map_protocol_role_board_job_status(job.status),
            items_processed: None,
            items_total: None,
            progress_note: job.progress_note.clone(),
            last_checkpoint_at_unix_seconds: None,
            blocking_reason: job.blocking_reason.clone(),
            active_thread_ids: Vec::new(),
            linked_subgoal_ids: Vec::new(),
            linked_graph_node_ids: Vec::new(),
        })
}

fn map_protocol_role_board_job_status(
    status: EpiphanyRoleBoardJobStatus,
) -> ThreadEpiphanyJobStatus {
    match status {
        EpiphanyRoleBoardJobStatus::Idle => ThreadEpiphanyJobStatus::Idle,
        EpiphanyRoleBoardJobStatus::Needed => ThreadEpiphanyJobStatus::Needed,
        EpiphanyRoleBoardJobStatus::Pending => ThreadEpiphanyJobStatus::Pending,
        EpiphanyRoleBoardJobStatus::Running => ThreadEpiphanyJobStatus::Running,
        EpiphanyRoleBoardJobStatus::Completed => ThreadEpiphanyJobStatus::Completed,
        EpiphanyRoleBoardJobStatus::Failed => ThreadEpiphanyJobStatus::Failed,
        EpiphanyRoleBoardJobStatus::Cancelled => ThreadEpiphanyJobStatus::Cancelled,
        EpiphanyRoleBoardJobStatus::Blocked => ThreadEpiphanyJobStatus::Blocked,
        EpiphanyRoleBoardJobStatus::Unavailable => ThreadEpiphanyJobStatus::Unavailable,
    }
}

fn map_protocol_coordinator_role_status(
    status: CoreEpiphanyCoordinatorRoleStatus,
) -> ThreadEpiphanyRoleStatus {
    match status {
        CoreEpiphanyCoordinatorRoleStatus::Ready => ThreadEpiphanyRoleStatus::Ready,
        CoreEpiphanyCoordinatorRoleStatus::Needed => ThreadEpiphanyRoleStatus::Needed,
        CoreEpiphanyCoordinatorRoleStatus::Running => ThreadEpiphanyRoleStatus::Running,
        CoreEpiphanyCoordinatorRoleStatus::Waiting => ThreadEpiphanyRoleStatus::Waiting,
        CoreEpiphanyCoordinatorRoleStatus::Review => ThreadEpiphanyRoleStatus::Review,
        CoreEpiphanyCoordinatorRoleStatus::Blocked => ThreadEpiphanyRoleStatus::Blocked,
        CoreEpiphanyCoordinatorRoleStatus::Unavailable => ThreadEpiphanyRoleStatus::Unavailable,
        CoreEpiphanyCoordinatorRoleStatus::Completed => ThreadEpiphanyRoleStatus::Completed,
    }
}

pub(super) fn render_epiphany_roles_note(
    roles: &[ThreadEpiphanyRoleLane],
    state_status: ThreadEpiphanyReorientStateStatus,
    recommendation: ThreadEpiphanyCrrcAction,
) -> String {
    let core_roles = roles
        .iter()
        .map(|role| EpiphanyRoleBoardLane {
            id: map_core_coordinator_role_id(role.id),
            title: role.title.clone(),
            owner_role: role.owner_role.clone(),
            status: map_core_coordinator_role_status(role.status),
            note: role.note.clone(),
            jobs: role.jobs.iter().map(map_core_role_board_job).collect(),
            authority_scopes: role.authority_scopes.clone(),
            recommended_action: role
                .recommended_action
                .map(map_core_coordinator_scene_action_from_protocol),
        })
        .collect::<Vec<_>>();
    render_role_board_note(
        &core_roles,
        format!("{:?}", state_status).as_str(),
        map_core_crrc_action_from_protocol(recommendation),
    )
}
