use codex_app_server_protocol::ThreadEpiphanyCoordinatorAction;
use codex_app_server_protocol::ThreadEpiphanyCoordinatorSignals;
use codex_app_server_protocol::ThreadEpiphanyCrrcAction;
use codex_app_server_protocol::ThreadEpiphanyCrrcRecommendation;
use codex_app_server_protocol::ThreadEpiphanyJob;
use codex_app_server_protocol::ThreadEpiphanyReorientAction;
use codex_app_server_protocol::ThreadEpiphanyReorientResultStatus;
use codex_app_server_protocol::ThreadEpiphanyReorientStateStatus;
use codex_app_server_protocol::ThreadEpiphanyRoleId;
use codex_app_server_protocol::ThreadEpiphanyRoleLane;
use codex_app_server_protocol::ThreadEpiphanyRoleResultStatus;
use codex_app_server_protocol::ThreadEpiphanyRoleStatus;
use codex_app_server_protocol::ThreadEpiphanyRolesSource;
use codex_app_server_protocol::ThreadEpiphanySceneAction;
use codex_app_server_protocol::ThreadEpiphanyViewCoordinator;
use epiphany_core::EpiphanyCoordinatorAction as CoreEpiphanyCoordinatorAction;
use epiphany_core::EpiphanyCoordinatorAutomationAction as CoreEpiphanyCoordinatorAutomationAction;
use epiphany_core::EpiphanyCoordinatorDecision as CoreEpiphanyCoordinatorDecision;
use epiphany_core::EpiphanyCoordinatorRoleId as CoreEpiphanyCoordinatorRoleId;
use epiphany_core::EpiphanyCoordinatorRoleResultStatus as CoreEpiphanyCoordinatorRoleResultStatus;
use epiphany_core::EpiphanyCoordinatorRoleStatus as CoreEpiphanyCoordinatorRoleStatus;
use epiphany_core::EpiphanyCoordinatorSceneAction as CoreEpiphanyCoordinatorSceneAction;
use epiphany_core::EpiphanyCoordinatorSourceSignals;
use epiphany_core::EpiphanyCoordinatorStatus as CoreEpiphanyCoordinatorStatus;
use epiphany_core::EpiphanyCoordinatorStatusInput;
use epiphany_core::EpiphanyCrrcAction as CoreEpiphanyCrrcAction;
use epiphany_core::EpiphanyCrrcInput;
use epiphany_core::EpiphanyCrrcRecommendation as CoreEpiphanyCrrcRecommendation;
use epiphany_core::EpiphanyCrrcReorientAction as CoreEpiphanyCrrcReorientAction;
use epiphany_core::EpiphanyCrrcResultStatus as CoreEpiphanyCrrcResultStatus;
use epiphany_core::EpiphanyCrrcSceneAction as CoreEpiphanyCrrcSceneAction;
use epiphany_core::EpiphanyCrrcStateStatus as CoreEpiphanyCrrcStateStatus;
use epiphany_core::EpiphanyJobLaunchRequest;
use epiphany_core::EpiphanyJobStatus as CoreEpiphanyJobStatus;
use epiphany_core::EpiphanyJobView;
use epiphany_core::EpiphanyReorientAction as CoreEpiphanyReorientAction;
use epiphany_core::EpiphanyReorientDecision as CoreEpiphanyReorientDecision;
use epiphany_core::EpiphanyReorientFindingInterpretation as CoreEpiphanyReorientFinding;
use epiphany_core::EpiphanyReorientStateStatus as CoreEpiphanyReorientStateStatus;
use epiphany_core::EpiphanyRoleBoardCheckpointSummary;
use epiphany_core::EpiphanyRoleBoardInput;
use epiphany_core::EpiphanyRoleBoardJob;
use epiphany_core::EpiphanyRoleBoardJobStatus;
use epiphany_core::EpiphanyRoleBoardLane;
use epiphany_core::EpiphanyRoleBoardPlanningSummary;
use epiphany_core::EpiphanyRoleResultRoleId;
use epiphany_core::EpiphanyTokenUsageSnapshot;
use epiphany_core::derive_coordinator_finding_signals;
use epiphany_core::derive_coordinator_status;
use epiphany_core::derive_role_board;
use epiphany_core::recommend_crrc_action;
use epiphany_core::render_role_board_note;
use epiphany_core::reorient_finding_already_accepted;
use epiphany_core::select_coordinator_automation_action;
use epiphany_state_model::EpiphanyJobKind as CoreEpiphanyJobKind;
use epiphany_state_model::EpiphanyRetrievalState;
use epiphany_state_model::EpiphanyThreadState;

use crate::jobs::map_core_epiphany_job_view;
use crate::launch::EPIPHANY_IMAGINATION_OWNER_ROLE;
use crate::launch::EPIPHANY_IMAGINATION_ROLE_BINDING_ID;
use crate::launch::EPIPHANY_MODELING_ROLE_BINDING_ID;
use crate::launch::EPIPHANY_REORIENT_LAUNCH_BINDING_ID;
use crate::launch::EPIPHANY_REORIENT_OWNER_ROLE;
use crate::launch::EPIPHANY_VERIFICATION_ROLE_BINDING_ID;
use crate::launch::build_epiphany_reorient_launch_request;
use crate::launch::render_epiphany_coordinator_note;
use crate::pressure::derive_epiphany_pressure;
use crate::reorient::EpiphanyFreshnessWatcherSnapshot;
use crate::reorient::derive_epiphany_freshness_view;
use crate::reorient::derive_epiphany_reorient;
use crate::reorient::map_protocol_reorient_state_status;
use crate::runtime_results::load_core_epiphany_reorient_result_snapshot;
use crate::runtime_results::load_core_epiphany_role_result_snapshot;

use std::path::Path;

pub fn map_epiphany_crrc_recommendation(
    loaded: bool,
    state_status: CoreEpiphanyReorientStateStatus,
    pressure: &epiphany_core::EpiphanyPressure,
    decision: &CoreEpiphanyReorientDecision,
    result_status: CoreEpiphanyCrrcResultStatus,
    checkpoint_present: bool,
    finding_present: bool,
    finding_accepted: bool,
) -> CoreEpiphanyCrrcRecommendation {
    recommend_crrc_action(EpiphanyCrrcInput {
        loaded,
        state_status: map_core_crrc_state_status_from_reorient(state_status),
        should_prepare_compaction: pressure.should_prepare_compaction,
        reorient_action: map_core_crrc_reorient_action(decision.action),
        result_status,
        checkpoint_present,
        finding_present,
        finding_accepted,
    })
}

fn map_core_crrc_state_status_from_reorient(
    status: CoreEpiphanyReorientStateStatus,
) -> CoreEpiphanyCrrcStateStatus {
    match status {
        CoreEpiphanyReorientStateStatus::Missing => CoreEpiphanyCrrcStateStatus::Missing,
        CoreEpiphanyReorientStateStatus::Ready => CoreEpiphanyCrrcStateStatus::Ready,
    }
}

fn map_core_crrc_reorient_action(
    action: CoreEpiphanyReorientAction,
) -> CoreEpiphanyCrrcReorientAction {
    match action {
        CoreEpiphanyReorientAction::Resume => CoreEpiphanyCrrcReorientAction::Resume,
        CoreEpiphanyReorientAction::Regather => CoreEpiphanyCrrcReorientAction::Regather,
    }
}

fn map_protocol_coordinator_source_signals(
    signals: EpiphanyCoordinatorSourceSignals,
) -> ThreadEpiphanyCoordinatorSignals {
    ThreadEpiphanyCoordinatorSignals {
        pressure_level: map_protocol_pressure_level(signals.pressure_level),
        should_prepare_compaction: signals.should_prepare_compaction,
        reorient_action: map_protocol_reorient_action(signals.reorient_action),
        crrc_action: map_protocol_crrc_action(signals.crrc_action),
        modeling_result_status: map_protocol_coordinator_role_result_status(
            signals.modeling_result_status,
        ),
        verification_result_status: map_protocol_coordinator_role_result_status(
            signals.verification_result_status,
        ),
        reorient_result_status: map_protocol_crrc_result_status(signals.reorient_result_status),
    }
}

fn map_protocol_pressure_level(
    level: epiphany_core::EpiphanyPressureLevel,
) -> codex_app_server_protocol::ThreadEpiphanyPressureLevel {
    match level {
        epiphany_core::EpiphanyPressureLevel::Unknown => {
            codex_app_server_protocol::ThreadEpiphanyPressureLevel::Unknown
        }
        epiphany_core::EpiphanyPressureLevel::Low => {
            codex_app_server_protocol::ThreadEpiphanyPressureLevel::Low
        }
        epiphany_core::EpiphanyPressureLevel::Elevated => {
            codex_app_server_protocol::ThreadEpiphanyPressureLevel::Elevated
        }
        epiphany_core::EpiphanyPressureLevel::High => {
            codex_app_server_protocol::ThreadEpiphanyPressureLevel::High
        }
        epiphany_core::EpiphanyPressureLevel::Critical => {
            codex_app_server_protocol::ThreadEpiphanyPressureLevel::Critical
        }
    }
}

fn map_protocol_reorient_action(
    action: CoreEpiphanyReorientAction,
) -> ThreadEpiphanyReorientAction {
    match action {
        CoreEpiphanyReorientAction::Resume => ThreadEpiphanyReorientAction::Resume,
        CoreEpiphanyReorientAction::Regather => ThreadEpiphanyReorientAction::Regather,
    }
}

fn map_protocol_crrc_result_status(
    status: CoreEpiphanyCrrcResultStatus,
) -> ThreadEpiphanyReorientResultStatus {
    match status {
        CoreEpiphanyCrrcResultStatus::MissingState => {
            ThreadEpiphanyReorientResultStatus::MissingState
        }
        CoreEpiphanyCrrcResultStatus::MissingBinding => {
            ThreadEpiphanyReorientResultStatus::MissingBinding
        }
        CoreEpiphanyCrrcResultStatus::BackendUnavailable => {
            ThreadEpiphanyReorientResultStatus::BackendUnavailable
        }
        CoreEpiphanyCrrcResultStatus::BackendMissing => {
            ThreadEpiphanyReorientResultStatus::BackendMissing
        }
        CoreEpiphanyCrrcResultStatus::Pending => ThreadEpiphanyReorientResultStatus::Pending,
        CoreEpiphanyCrrcResultStatus::Running => ThreadEpiphanyReorientResultStatus::Running,
        CoreEpiphanyCrrcResultStatus::Completed => ThreadEpiphanyReorientResultStatus::Completed,
        CoreEpiphanyCrrcResultStatus::Failed => ThreadEpiphanyReorientResultStatus::Failed,
        CoreEpiphanyCrrcResultStatus::Cancelled => ThreadEpiphanyReorientResultStatus::Cancelled,
    }
}

pub fn map_protocol_crrc_action(action: CoreEpiphanyCrrcAction) -> ThreadEpiphanyCrrcAction {
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

pub fn map_protocol_crrc_recommendation(
    recommendation: CoreEpiphanyCrrcRecommendation,
) -> ThreadEpiphanyCrrcRecommendation {
    ThreadEpiphanyCrrcRecommendation {
        action: map_protocol_crrc_action(recommendation.action),
        recommended_scene_action: recommendation
            .recommended_scene_action
            .map(map_core_crrc_scene_action),
        reason: recommendation.reason,
    }
}

pub type EpiphanyCoordinatorDecision = CoreEpiphanyCoordinatorDecision;

pub struct EpiphanyCoordinatorStatus {
    pub core: CoreEpiphanyCoordinatorStatus,
    pub note: String,
}

pub async fn derive_epiphany_coordinator_status(
    state: Option<&EpiphanyThreadState>,
    runtime_store_path: Option<&Path>,
    state_status: CoreEpiphanyReorientStateStatus,
    pressure: &epiphany_core::EpiphanyPressure,
    recommendation: &CoreEpiphanyCrrcRecommendation,
    roles: Vec<EpiphanyRoleBoardLane>,
    reorient_decision: Option<&CoreEpiphanyReorientDecision>,
    reorient_result_status: CoreEpiphanyCrrcResultStatus,
    reorient_finding: Option<&CoreEpiphanyReorientFinding>,
    checkpoint_present: bool,
) -> EpiphanyCoordinatorStatus {
    let (modeling_result_status, modeling_finding) = if let Some(state) = state {
        let snapshot = load_core_epiphany_role_result_snapshot(
            state,
            runtime_store_path,
            EpiphanyRoleResultRoleId::Modeling,
            EPIPHANY_MODELING_ROLE_BINDING_ID,
        )
        .await;
        (snapshot.status, snapshot.finding)
    } else {
        (CoreEpiphanyCoordinatorRoleResultStatus::MissingState, None)
    };
    let (verification_result_status, verification_finding) = if let Some(state) = state {
        let snapshot = load_core_epiphany_role_result_snapshot(
            state,
            runtime_store_path,
            EpiphanyRoleResultRoleId::Verification,
            EPIPHANY_VERIFICATION_ROLE_BINDING_ID,
        )
        .await;
        (snapshot.status, snapshot.finding)
    } else {
        (CoreEpiphanyCoordinatorRoleResultStatus::MissingState, None)
    };
    let finding_signals = derive_coordinator_finding_signals(
        state,
        modeling_finding.as_ref(),
        verification_finding.as_ref(),
        reorient_finding,
    );
    let core = derive_coordinator_status(EpiphanyCoordinatorStatusInput {
        state_status: map_core_crrc_state_status_from_reorient(state_status),
        checkpoint_present,
        pressure: pressure.clone(),
        recommendation: recommendation.clone(),
        roles,
        reorient_action: reorient_decision
            .map(|decision| decision.action)
            .unwrap_or(CoreEpiphanyReorientAction::Resume),
        modeling_result_status,
        verification_result_status,
        reorient_result_status,
        modeling_result_accepted: finding_signals.modeling_result_accepted,
        modeling_result_reviewable: finding_signals.modeling_result_reviewable,
        modeling_result_accepted_after_verification: finding_signals
            .modeling_result_accepted_after_verification,
        implementation_evidence_after_verification: finding_signals
            .implementation_evidence_after_verification,
        verification_result_cites_implementation_evidence: finding_signals
            .verification_result_cites_implementation_evidence,
        verification_result_covers_current_modeling: finding_signals
            .verification_result_covers_current_modeling,
        verification_result_accepted: finding_signals.verification_result_accepted,
        verification_result_allows_implementation: finding_signals
            .verification_result_allows_implementation,
        verification_result_needs_evidence: finding_signals.verification_result_needs_evidence,
        reorient_finding_accepted: finding_signals.reorient_finding_accepted,
    });
    let note = render_epiphany_coordinator_note(
        recommendation.action,
        pressure.level,
        modeling_result_status,
        verification_result_status,
        core.source_signals.reorient_result_status,
        core.decision.action,
    );

    EpiphanyCoordinatorStatus { core, note }
}

pub fn map_epiphany_coordinator_view(
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
        state_status: map_protocol_reorient_state_status(state_status),
        state_revision,
        action: map_protocol_coordinator_action(status.core.decision.action),
        target_role: status
            .core
            .decision
            .target_role
            .map(map_protocol_coordinator_role_id),
        recommended_scene_action: status
            .core
            .decision
            .recommended_scene_action
            .map(map_protocol_coordinator_scene_action),
        requires_review: status.core.decision.requires_review,
        can_auto_run: status.core.decision.can_auto_run,
        reason: status.core.decision.reason,
        source_signals: map_protocol_coordinator_source_signals(status.core.source_signals),
        roles: map_protocol_role_board_lanes(&roles),
        note: status.note,
    }
}

pub type EpiphanyCoordinatorAutomationAction = CoreEpiphanyCoordinatorAutomationAction;

pub struct EpiphanyCoordinatorAutomationInput<'a> {
    pub thread_id: &'a str,
    pub state: &'a EpiphanyThreadState,
    pub retrieval_override: &'a EpiphanyRetrievalState,
    pub watcher_snapshot: EpiphanyFreshnessWatcherSnapshot<'a>,
    pub token_usage_info: Option<&'a EpiphanyTokenUsageSnapshot>,
    pub runtime_store_path: &'a Path,
    pub force_checkpoint_compaction: bool,
}

pub struct EpiphanyCoordinatorAutomationVerdict {
    pub action: EpiphanyCoordinatorAutomationAction,
    pub launch_request: Option<EpiphanyJobLaunchRequest>,
}

pub async fn select_epiphany_coordinator_automation(
    input: EpiphanyCoordinatorAutomationInput<'_>,
) -> EpiphanyCoordinatorAutomationVerdict {
    let core_freshness = derive_epiphany_freshness_view(
        Some(input.state),
        Some(input.retrieval_override),
        Some(input.watcher_snapshot),
    );
    let core_pressure = derive_epiphany_pressure(input.token_usage_info);
    let (state_status, reorient_decision) = derive_epiphany_reorient(
        Some(input.state),
        &core_pressure,
        &core_freshness.retrieval,
        &core_freshness.graph,
        &core_freshness.watcher,
    );
    if state_status != CoreEpiphanyReorientStateStatus::Ready {
        return EpiphanyCoordinatorAutomationVerdict {
            action: EpiphanyCoordinatorAutomationAction::None,
            launch_request: None,
        };
    }

    let jobs = crate::jobs::map_epiphany_jobs(Some(input.state), Some(input.retrieval_override));
    let reorient_job = jobs
        .iter()
        .find(|job| job.id == EPIPHANY_REORIENT_LAUNCH_BINDING_ID)
        .cloned();
    let reorient_result = load_core_epiphany_reorient_result_snapshot(
        Some(input.state),
        Some(input.runtime_store_path),
        EPIPHANY_REORIENT_LAUNCH_BINDING_ID,
    )
    .await;
    let reorient_finding_accepted = reorient_result
        .finding
        .as_ref()
        .is_some_and(|finding| reorient_finding_already_accepted(input.state, finding));
    let crrc_recommendation = map_epiphany_crrc_recommendation(
        true,
        state_status,
        &core_pressure,
        &reorient_decision,
        reorient_result.status,
        input.state.investigation_checkpoint.as_ref().is_some(),
        reorient_result.finding.is_some(),
        reorient_finding_accepted,
    );
    let roles = map_epiphany_roles(
        Some(input.state),
        &jobs,
        &reorient_decision,
        &core_pressure,
        &crrc_recommendation,
        reorient_result.status,
        reorient_job.as_ref(),
    );
    let coordinator = derive_epiphany_coordinator_status(
        Some(input.state),
        Some(input.runtime_store_path),
        state_status,
        &core_pressure,
        &crrc_recommendation,
        roles.roles,
        Some(&reorient_decision),
        reorient_result.status,
        reorient_result.finding.as_ref(),
        input.state.investigation_checkpoint.as_ref().is_some(),
    )
    .await;

    let action = select_coordinator_automation_action(
        &coordinator.core.decision,
        input.force_checkpoint_compaction,
    );
    let launch_request = match action {
        EpiphanyCoordinatorAutomationAction::LaunchReorientWorker => input
            .state
            .investigation_checkpoint
            .as_ref()
            .map(|checkpoint| {
                build_epiphany_reorient_launch_request(
                    input.thread_id,
                    Some(input.state.revision),
                    None,
                    input.state,
                    checkpoint,
                    &reorient_decision,
                )
            }),
        EpiphanyCoordinatorAutomationAction::None
        | EpiphanyCoordinatorAutomationAction::CompactRehydrateReorient => None,
    };
    let action = if matches!(
        action,
        EpiphanyCoordinatorAutomationAction::LaunchReorientWorker
    ) && launch_request.is_none()
    {
        EpiphanyCoordinatorAutomationAction::None
    } else {
        action
    };

    EpiphanyCoordinatorAutomationVerdict {
        action,
        launch_request,
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

fn map_protocol_coordinator_role_result_status(
    status: CoreEpiphanyCoordinatorRoleResultStatus,
) -> ThreadEpiphanyRoleResultStatus {
    match status {
        CoreEpiphanyCoordinatorRoleResultStatus::MissingState => {
            ThreadEpiphanyRoleResultStatus::MissingState
        }
        CoreEpiphanyCoordinatorRoleResultStatus::MissingBinding => {
            ThreadEpiphanyRoleResultStatus::MissingBinding
        }
        CoreEpiphanyCoordinatorRoleResultStatus::BackendUnavailable => {
            ThreadEpiphanyRoleResultStatus::BackendUnavailable
        }
        CoreEpiphanyCoordinatorRoleResultStatus::BackendMissing => {
            ThreadEpiphanyRoleResultStatus::BackendMissing
        }
        CoreEpiphanyCoordinatorRoleResultStatus::Pending => ThreadEpiphanyRoleResultStatus::Pending,
        CoreEpiphanyCoordinatorRoleResultStatus::Running => ThreadEpiphanyRoleResultStatus::Running,
        CoreEpiphanyCoordinatorRoleResultStatus::Completed => {
            ThreadEpiphanyRoleResultStatus::Completed
        }
        CoreEpiphanyCoordinatorRoleResultStatus::Failed => ThreadEpiphanyRoleResultStatus::Failed,
        CoreEpiphanyCoordinatorRoleResultStatus::Cancelled => {
            ThreadEpiphanyRoleResultStatus::Cancelled
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

#[derive(Debug, Clone)]
pub struct EpiphanyRoleBoardStatus {
    pub roles: Vec<EpiphanyRoleBoardLane>,
    source_jobs: Vec<EpiphanyJobView>,
}

pub fn map_epiphany_roles(
    state: Option<&EpiphanyThreadState>,
    jobs: &[EpiphanyJobView],
    decision: &CoreEpiphanyReorientDecision,
    pressure: &epiphany_core::EpiphanyPressure,
    recommendation: &CoreEpiphanyCrrcRecommendation,
    result_status: CoreEpiphanyCrrcResultStatus,
    reorient_job: Option<&EpiphanyJobView>,
) -> EpiphanyRoleBoardStatus {
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
    let roles = derive_role_board(EpiphanyRoleBoardInput {
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
        crrc_action: recommendation.action,
        crrc_recommended_scene_action: recommendation
            .recommended_scene_action
            .map(epiphany_core::crrc_scene_action_to_coordinator_scene_action),
        crrc_reason: recommendation.reason.clone(),
        reorient_decision_action: format!("{:?}", decision.action),
        pressure_level: format!("{:?}", pressure.level),
        reorient_result_status: result_status,
        reorient_job: reorient_job.map(map_core_role_board_job),
        imagination_binding_id: EPIPHANY_IMAGINATION_ROLE_BINDING_ID.to_string(),
        modeling_binding_id: EPIPHANY_MODELING_ROLE_BINDING_ID.to_string(),
        verification_binding_id: EPIPHANY_VERIFICATION_ROLE_BINDING_ID.to_string(),
        reorient_owner_role: EPIPHANY_REORIENT_OWNER_ROLE.to_string(),
        imagination_owner_role: EPIPHANY_IMAGINATION_OWNER_ROLE.to_string(),
    });

    EpiphanyRoleBoardStatus { roles, source_jobs }
}

fn map_core_role_board_job(job: &EpiphanyJobView) -> EpiphanyRoleBoardJob {
    EpiphanyRoleBoardJob {
        id: job.id.clone(),
        owner_role: job.owner_role.clone(),
        status: map_core_role_board_job_status(job.status),
        progress_note: job.progress_note.clone(),
        blocking_reason: job.blocking_reason.clone(),
    }
}

fn map_core_role_board_job_status(status: CoreEpiphanyJobStatus) -> EpiphanyRoleBoardJobStatus {
    match status {
        CoreEpiphanyJobStatus::Idle => EpiphanyRoleBoardJobStatus::Idle,
        CoreEpiphanyJobStatus::Needed => EpiphanyRoleBoardJobStatus::Needed,
        CoreEpiphanyJobStatus::Pending => EpiphanyRoleBoardJobStatus::Pending,
        CoreEpiphanyJobStatus::Running => EpiphanyRoleBoardJobStatus::Running,
        CoreEpiphanyJobStatus::Completed => EpiphanyRoleBoardJobStatus::Completed,
        CoreEpiphanyJobStatus::Failed => EpiphanyRoleBoardJobStatus::Failed,
        CoreEpiphanyJobStatus::Cancelled => EpiphanyRoleBoardJobStatus::Cancelled,
        CoreEpiphanyJobStatus::Blocked => EpiphanyRoleBoardJobStatus::Blocked,
        CoreEpiphanyJobStatus::Unavailable => EpiphanyRoleBoardJobStatus::Unavailable,
    }
}

pub fn map_protocol_role_board_lanes(
    role_board: &EpiphanyRoleBoardStatus,
) -> Vec<ThreadEpiphanyRoleLane> {
    role_board
        .roles
        .iter()
        .cloned()
        .map(|lane| map_protocol_role_board_lane(lane, &role_board.source_jobs))
        .collect()
}

fn map_protocol_role_board_lane(
    lane: EpiphanyRoleBoardLane,
    source_jobs: &[EpiphanyJobView],
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
    source_jobs: &[EpiphanyJobView],
) -> ThreadEpiphanyJob {
    source_jobs
        .iter()
        .find(|source_job| source_job.id == job.id && source_job.owner_role == job.owner_role)
        .cloned()
        .map(map_core_epiphany_job_view)
        .unwrap_or_else(|| {
            map_core_epiphany_job_view(EpiphanyJobView {
                id: job.id.clone(),
                kind: CoreEpiphanyJobKind::Specialist,
                scope: job.id.clone(),
                owner_role: job.owner_role.clone(),
                authority_scope: None,
                runtime_job_id: None,
                status: map_core_epiphany_job_status_from_role_board(job.status),
                items_processed: None,
                items_total: None,
                progress_note: job.progress_note.clone(),
                last_checkpoint_at_unix_seconds: None,
                blocking_reason: job.blocking_reason.clone(),
                active_thread_ids: Vec::new(),
                linked_subgoal_ids: Vec::new(),
                linked_graph_node_ids: Vec::new(),
            })
        })
}

fn map_core_epiphany_job_status_from_role_board(
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

pub fn render_epiphany_roles_note(
    roles: &[EpiphanyRoleBoardLane],
    state_status: ThreadEpiphanyReorientStateStatus,
    recommendation: CoreEpiphanyCrrcAction,
) -> String {
    render_role_board_note(
        roles,
        format!("{:?}", state_status).as_str(),
        recommendation,
    )
}
