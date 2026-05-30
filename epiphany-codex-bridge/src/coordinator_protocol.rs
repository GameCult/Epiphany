use codex_app_server_protocol::ThreadEpiphanyCoordinatorAction;
use codex_app_server_protocol::ThreadEpiphanyCoordinatorSignals;
use codex_app_server_protocol::ThreadEpiphanyCrrcAction;
use codex_app_server_protocol::ThreadEpiphanyCrrcRecommendation;
use codex_app_server_protocol::ThreadEpiphanyJob;
use codex_app_server_protocol::ThreadEpiphanyReorientAction;
use codex_app_server_protocol::ThreadEpiphanyRoleId;
use codex_app_server_protocol::ThreadEpiphanyRoleLane;
use codex_app_server_protocol::ThreadEpiphanyRoleStatus;
use codex_app_server_protocol::ThreadEpiphanyRolesSource;
use codex_app_server_protocol::ThreadEpiphanySceneAction;
use codex_app_server_protocol::ThreadEpiphanyViewCoordinator;
use epiphany_core::EpiphanyCoordinatorAction as CoreEpiphanyCoordinatorAction;
use epiphany_core::EpiphanyCoordinatorRoleId as CoreEpiphanyCoordinatorRoleId;
use epiphany_core::EpiphanyCoordinatorRoleStatus as CoreEpiphanyCoordinatorRoleStatus;
use epiphany_core::EpiphanyCoordinatorSceneAction as CoreEpiphanyCoordinatorSceneAction;
use epiphany_core::EpiphanyCoordinatorSourceSignals;
use epiphany_core::EpiphanyCrrcAction as CoreEpiphanyCrrcAction;
use epiphany_core::EpiphanyCrrcRecommendation as CoreEpiphanyCrrcRecommendation;
use epiphany_core::EpiphanyCrrcSceneAction as CoreEpiphanyCrrcSceneAction;
use epiphany_core::EpiphanyJobStatus as CoreEpiphanyJobStatus;
use epiphany_core::EpiphanyJobView;
use epiphany_core::EpiphanyReorientAction as CoreEpiphanyReorientAction;
use epiphany_core::EpiphanyReorientStateStatus as CoreEpiphanyReorientStateStatus;
use epiphany_core::EpiphanyRoleBoardJob;
use epiphany_core::EpiphanyRoleBoardJobStatus;
use epiphany_core::EpiphanyRoleBoardLane;
use epiphany_state_model::EpiphanyJobKind as CoreEpiphanyJobKind;

use crate::coordinator::EpiphanyCoordinatorStatus;
use crate::coordinator::EpiphanyRoleBoardStatus;
use crate::protocol_edge::protocol_job_from_surface;
use crate::protocol_edge::protocol_pressure_level;
use crate::protocol_edge::protocol_reorient_result_status;
use crate::protocol_edge::protocol_reorient_state_status;
use crate::protocol_edge::protocol_role_result_status;

fn protocol_coordinator_source_signals(
    signals: EpiphanyCoordinatorSourceSignals,
) -> ThreadEpiphanyCoordinatorSignals {
    ThreadEpiphanyCoordinatorSignals {
        pressure_level: protocol_pressure_level(signals.pressure_level),
        should_prepare_compaction: signals.should_prepare_compaction,
        reorient_action: protocol_reorient_action(signals.reorient_action),
        crrc_action: protocol_crrc_action(signals.crrc_action),
        research_result_status: protocol_role_result_status(signals.research_result_status),
        modeling_result_status: protocol_role_result_status(signals.modeling_result_status),
        verification_result_status: protocol_role_result_status(signals.verification_result_status),
        reorient_result_status: protocol_reorient_result_status(signals.reorient_result_status),
    }
}

fn protocol_reorient_action(action: CoreEpiphanyReorientAction) -> ThreadEpiphanyReorientAction {
    match action {
        CoreEpiphanyReorientAction::Resume => ThreadEpiphanyReorientAction::Resume,
        CoreEpiphanyReorientAction::Regather => ThreadEpiphanyReorientAction::Regather,
    }
}

pub fn protocol_crrc_action(action: CoreEpiphanyCrrcAction) -> ThreadEpiphanyCrrcAction {
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

fn protocol_crrc_scene_action(action: CoreEpiphanyCrrcSceneAction) -> ThreadEpiphanySceneAction {
    match action {
        CoreEpiphanyCrrcSceneAction::Update => ThreadEpiphanySceneAction::Update,
        CoreEpiphanyCrrcSceneAction::Reorient => ThreadEpiphanySceneAction::Reorient,
        CoreEpiphanyCrrcSceneAction::ReorientLaunch => ThreadEpiphanySceneAction::ReorientLaunch,
        CoreEpiphanyCrrcSceneAction::ReorientResult => ThreadEpiphanySceneAction::ReorientResult,
        CoreEpiphanyCrrcSceneAction::ReorientAccept => ThreadEpiphanySceneAction::ReorientAccept,
    }
}

pub fn protocol_crrc_recommendation(
    recommendation: CoreEpiphanyCrrcRecommendation,
) -> ThreadEpiphanyCrrcRecommendation {
    ThreadEpiphanyCrrcRecommendation {
        action: protocol_crrc_action(recommendation.action),
        recommended_scene_action: recommendation
            .recommended_scene_action
            .map(protocol_crrc_scene_action),
        reason: recommendation.reason,
    }
}

pub fn protocol_coordinator_view(
    thread_id: String,
    loaded: bool,
    state_status: CoreEpiphanyReorientStateStatus,
    state_revision: Option<u64>,
    status: EpiphanyCoordinatorStatus,
    roles: EpiphanyRoleBoardStatus,
) -> ThreadEpiphanyViewCoordinator {
    ThreadEpiphanyViewCoordinator {
        thread_id,
        source: if loaded {
            ThreadEpiphanyRolesSource::Live
        } else {
            ThreadEpiphanyRolesSource::Stored
        },
        state_status: protocol_reorient_state_status(state_status),
        state_revision,
        action: protocol_coordinator_action(status.core.decision.action),
        target_role: status
            .core
            .decision
            .target_role
            .map(protocol_coordinator_role_id),
        recommended_scene_action: status
            .core
            .decision
            .recommended_scene_action
            .map(protocol_coordinator_scene_action),
        requires_review: status.core.decision.requires_review,
        can_auto_run: status.core.decision.can_auto_run,
        reason: status.core.decision.reason,
        source_signals: protocol_coordinator_source_signals(status.core.source_signals),
        roles: protocol_role_board_lanes(&roles),
        note: status.note,
    }
}

fn protocol_coordinator_role_id(role_id: CoreEpiphanyCoordinatorRoleId) -> ThreadEpiphanyRoleId {
    match role_id {
        CoreEpiphanyCoordinatorRoleId::Implementation => ThreadEpiphanyRoleId::Implementation,
        CoreEpiphanyCoordinatorRoleId::Imagination => ThreadEpiphanyRoleId::Imagination,
        CoreEpiphanyCoordinatorRoleId::Research => ThreadEpiphanyRoleId::Research,
        CoreEpiphanyCoordinatorRoleId::Modeling => ThreadEpiphanyRoleId::Modeling,
        CoreEpiphanyCoordinatorRoleId::Verification => ThreadEpiphanyRoleId::Verification,
        CoreEpiphanyCoordinatorRoleId::Reorientation => ThreadEpiphanyRoleId::Reorientation,
    }
}

fn protocol_coordinator_scene_action(
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

fn protocol_coordinator_action(
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
        CoreEpiphanyCoordinatorAction::LaunchResearch => {
            ThreadEpiphanyCoordinatorAction::LaunchResearch
        }
        CoreEpiphanyCoordinatorAction::ReviewResearchResult => {
            ThreadEpiphanyCoordinatorAction::ReviewResearchResult
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

pub fn protocol_role_board_lanes(
    role_board: &EpiphanyRoleBoardStatus,
) -> Vec<ThreadEpiphanyRoleLane> {
    role_board
        .roles
        .iter()
        .cloned()
        .map(|lane| protocol_role_board_lane(lane, &role_board.source_jobs))
        .collect()
}

fn protocol_role_board_lane(
    lane: EpiphanyRoleBoardLane,
    source_jobs: &[EpiphanyJobView],
) -> ThreadEpiphanyRoleLane {
    ThreadEpiphanyRoleLane {
        id: protocol_coordinator_role_id(lane.id),
        title: lane.title,
        owner_role: lane.owner_role,
        status: protocol_coordinator_role_status(lane.status),
        note: lane.note,
        jobs: lane
            .jobs
            .iter()
            .map(|job| protocol_role_board_job(job, source_jobs))
            .collect(),
        authority_scopes: lane.authority_scopes,
        recommended_action: lane
            .recommended_action
            .map(protocol_coordinator_scene_action),
    }
}

fn protocol_role_board_job(
    job: &EpiphanyRoleBoardJob,
    source_jobs: &[EpiphanyJobView],
) -> ThreadEpiphanyJob {
    source_jobs
        .iter()
        .find(|source_job| source_job.id == job.id && source_job.owner_role == job.owner_role)
        .cloned()
        .map(|job| protocol_job_from_surface(job, None, None))
        .unwrap_or_else(|| {
            protocol_job_from_surface(
                EpiphanyJobView {
                    id: job.id.clone(),
                    kind: CoreEpiphanyJobKind::Specialist,
                    scope: job.id.clone(),
                    owner_role: job.owner_role.clone(),
                    authority_scope: None,
                    runtime_job_id: None,
                    status: core_epiphany_job_status_from_role_board(job.status),
                    items_processed: None,
                    items_total: None,
                    progress_note: job.progress_note.clone(),
                    last_checkpoint_at_unix_seconds: None,
                    blocking_reason: job.blocking_reason.clone(),
                    active_thread_ids: Vec::new(),
                    linked_subgoal_ids: Vec::new(),
                    linked_graph_node_ids: Vec::new(),
                },
                None,
                None,
            )
        })
}

fn core_epiphany_job_status_from_role_board(
    status: EpiphanyRoleBoardJobStatus,
) -> CoreEpiphanyJobStatus {
    match status {
        EpiphanyRoleBoardJobStatus::Idle => CoreEpiphanyJobStatus::Idle,
        EpiphanyRoleBoardJobStatus::Needed => CoreEpiphanyJobStatus::Needed,
        EpiphanyRoleBoardJobStatus::Pending => CoreEpiphanyJobStatus::Pending,
        EpiphanyRoleBoardJobStatus::Running => CoreEpiphanyJobStatus::Running,
        EpiphanyRoleBoardJobStatus::Completed => CoreEpiphanyJobStatus::Completed,
        EpiphanyRoleBoardJobStatus::Failed => CoreEpiphanyJobStatus::Failed,
        EpiphanyRoleBoardJobStatus::Cancelled => CoreEpiphanyJobStatus::Cancelled,
        EpiphanyRoleBoardJobStatus::Blocked => CoreEpiphanyJobStatus::Blocked,
        EpiphanyRoleBoardJobStatus::Unavailable => CoreEpiphanyJobStatus::Unavailable,
    }
}

fn protocol_coordinator_role_status(
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
