use epiphany_core::EPIPHANY_IMAGINATION_OWNER_ROLE;
use epiphany_core::EPIPHANY_IMAGINATION_ROLE_BINDING_ID;
use epiphany_core::EPIPHANY_IMPLEMENTATION_OWNER_ROLE;
use epiphany_core::EPIPHANY_IMPLEMENTATION_ROLE_BINDING_ID;
use epiphany_core::EPIPHANY_MODELING_ROLE_BINDING_ID;
use epiphany_core::EPIPHANY_REORIENT_LAUNCH_BINDING_ID;
use epiphany_core::EPIPHANY_REORIENT_OWNER_ROLE;
use epiphany_core::EPIPHANY_RESEARCH_ROLE_BINDING_ID;
use epiphany_core::EPIPHANY_VERIFICATION_ROLE_BINDING_ID;
use epiphany_core::EpiphanyCoordinatorAutomationAction as CoreEpiphanyCoordinatorAutomationAction;
use epiphany_core::EpiphanyCoordinatorDecision as CoreEpiphanyCoordinatorDecision;
use epiphany_core::EpiphanyCoordinatorRoleResultStatus as CoreEpiphanyCoordinatorRoleResultStatus;
use epiphany_core::EpiphanyCoordinatorStatus as CoreEpiphanyCoordinatorStatus;
use epiphany_core::EpiphanyCoordinatorStatusInput;
use epiphany_core::EpiphanyCrrcAction as CoreEpiphanyCrrcAction;
use epiphany_core::EpiphanyCrrcInput;
use epiphany_core::EpiphanyCrrcRecommendation as CoreEpiphanyCrrcRecommendation;
use epiphany_core::EpiphanyCrrcReorientAction as CoreEpiphanyCrrcReorientAction;
use epiphany_core::EpiphanyCrrcResultStatus as CoreEpiphanyCrrcResultStatus;
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
use epiphany_core::build_epiphany_reorient_launch_request_with_dynamic_context;
use epiphany_core::derive_coordinator_finding_signals;
use epiphany_core::derive_coordinator_status;
use epiphany_core::derive_role_board;
use epiphany_core::recommend_crrc_action;
use epiphany_core::render_epiphany_coordinator_note;
use epiphany_core::render_role_board_note;
use epiphany_core::reorient_finding_already_accepted;
use epiphany_core::select_coordinator_automation_action;
use epiphany_state_model::EpiphanyRetrievalState;
use epiphany_state_model::EpiphanyThreadState;

use crate::launch_context::render_launch_dynamic_prompt_context;
use crate::launch_context::reorient_launch_context_focus;
use crate::pressure::derive_epiphany_pressure;
use crate::reorient::EpiphanyFreshnessWatcherSnapshot;
use crate::reorient::derive_epiphany_freshness_view;
use crate::reorient::derive_epiphany_reorient;
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
    let (research_result_status, research_finding) = if let Some(state) = state {
        let snapshot = load_core_epiphany_role_result_snapshot(
            state,
            runtime_store_path,
            EpiphanyRoleResultRoleId::Research,
            EPIPHANY_RESEARCH_ROLE_BINDING_ID,
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
    let (implementation_result_status, implementation_finding, implementation_completed_at) =
        if let Some(state) = state {
            let snapshot = load_core_epiphany_role_result_snapshot(
                state,
                runtime_store_path,
                EpiphanyRoleResultRoleId::Implementation,
                EPIPHANY_IMPLEMENTATION_ROLE_BINDING_ID,
            )
            .await;
            (snapshot.status, snapshot.finding, snapshot.completed_at)
        } else {
            (
                CoreEpiphanyCoordinatorRoleResultStatus::MissingState,
                None,
                None,
            )
        };
    let implementation_commit_requires_modeling_refresh =
        implementation_commit_requires_modeling_refresh(
            state,
            implementation_result_status,
            implementation_finding.as_ref(),
            implementation_completed_at.as_deref(),
            modeling_finding.as_ref(),
        );
    let finding_signals = derive_coordinator_finding_signals(
        state,
        research_finding.as_ref(),
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
        research_result_status,
        modeling_result_status,
        verification_result_status,
        reorient_result_status,
        research_result_accepted: finding_signals.research_result_accepted,
        research_result_reviewable: finding_signals.research_result_reviewable,
        modeling_result_requests_regather: finding_signals.modeling_result_requests_regather,
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
        implementation_commit_requires_modeling_refresh,
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

fn implementation_commit_requires_modeling_refresh(
    state: Option<&EpiphanyThreadState>,
    implementation_status: CoreEpiphanyCoordinatorRoleResultStatus,
    implementation_finding: Option<&epiphany_core::EpiphanyRoleFindingInterpretation>,
    implementation_completed_at: Option<&str>,
    modeling_finding: Option<&epiphany_core::EpiphanyRoleFindingInterpretation>,
) -> bool {
    if implementation_status != CoreEpiphanyCoordinatorRoleResultStatus::Completed {
        return false;
    }
    let Some(implementation_finding) = implementation_finding else {
        return false;
    };
    if !implementation_finding
        .evidence_ids
        .iter()
        .any(|id| id.starts_with("hands-receipt:hands-commit-") || id.starts_with("hands-commit-"))
    {
        return false;
    }
    let Some(completed_at) = implementation_completed_at else {
        return true;
    };
    !modeling_accepted_after(state, modeling_finding, completed_at)
}

fn modeling_accepted_after(
    state: Option<&EpiphanyThreadState>,
    modeling_finding: Option<&epiphany_core::EpiphanyRoleFindingInterpretation>,
    completed_at: &str,
) -> bool {
    let Some(state) = state else {
        return false;
    };
    let Some(result_id) = modeling_finding.and_then(|finding| finding.runtime_result_id.as_deref())
    else {
        return false;
    };
    state.acceptance_receipts.iter().any(|receipt| {
        receipt.result_id == result_id
            && receipt.status == "accepted"
            && receipt.surface == "roleAccept"
            && receipt.role_id == "modeling"
            && receipt.accepted_at.as_str() > completed_at
    })
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
                let dynamic_prompt_context = render_launch_dynamic_prompt_context(
                    input.runtime_store_path,
                    input.state,
                    reorient_launch_context_focus(input.state, &reorient_decision.next_action),
                )
                .ok();
                build_epiphany_reorient_launch_request_with_dynamic_context(
                    input.thread_id,
                    Some(input.state.revision),
                    None,
                    input.state,
                    checkpoint,
                    &reorient_decision,
                    dynamic_prompt_context,
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

#[derive(Debug, Clone)]
pub struct EpiphanyRoleBoardStatus {
    pub roles: Vec<EpiphanyRoleBoardLane>,
    pub(crate) source_jobs: Vec<EpiphanyJobView>,
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
        implementation_binding_id: EPIPHANY_IMPLEMENTATION_ROLE_BINDING_ID.to_string(),
        imagination_binding_id: EPIPHANY_IMAGINATION_ROLE_BINDING_ID.to_string(),
        research_binding_id: EPIPHANY_RESEARCH_ROLE_BINDING_ID.to_string(),
        modeling_binding_id: EPIPHANY_MODELING_ROLE_BINDING_ID.to_string(),
        verification_binding_id: EPIPHANY_VERIFICATION_ROLE_BINDING_ID.to_string(),
        implementation_owner_role: EPIPHANY_IMPLEMENTATION_OWNER_ROLE.to_string(),
        reorient_owner_role: EPIPHANY_REORIENT_OWNER_ROLE.to_string(),
        imagination_owner_role: EPIPHANY_IMAGINATION_OWNER_ROLE.to_string(),
        research_owner_role: epiphany_core::EPIPHANY_RESEARCH_OWNER_ROLE.to_string(),
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

pub fn render_epiphany_roles_note(
    roles: &[EpiphanyRoleBoardLane],
    state_status: CoreEpiphanyReorientStateStatus,
    recommendation: CoreEpiphanyCrrcAction,
) -> String {
    render_role_board_note(
        roles,
        format!("{:?}", state_status).as_str(),
        recommendation,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn finding(
        result_id: &str,
        evidence_ids: Vec<String>,
    ) -> epiphany_core::EpiphanyRoleFindingInterpretation {
        epiphany_core::EpiphanyRoleFindingInterpretation {
            verdict: Some("pass".to_string()),
            summary: Some("Finding.".to_string()),
            next_safe_move: None,
            checkpoint_summary: None,
            scratch_summary: None,
            files_inspected: Vec::new(),
            frontier_node_ids: Vec::new(),
            evidence_ids,
            artifact_refs: Vec::new(),
            runtime_result_id: Some(result_id.to_string()),
            runtime_job_id: Some(format!("job-{result_id}")),
            open_questions: Vec::new(),
            evidence_gaps: Vec::new(),
            risks: Vec::new(),
            state_patch: None,
            self_patch: None,
            self_persistence: None,
            job_error: None,
            item_error: None,
        }
    }

    fn state_with_modeling_acceptance(accepted_at: &str) -> EpiphanyThreadState {
        EpiphanyThreadState {
            acceptance_receipts: vec![epiphany_state_model::EpiphanyAcceptanceReceipt {
                id: "accept-modeling".to_string(),
                result_id: "modeling-result".to_string(),
                job_id: "modeling-job".to_string(),
                binding_id: EPIPHANY_MODELING_ROLE_BINDING_ID.to_string(),
                surface: "roleAccept".to_string(),
                role_id: "modeling".to_string(),
                status: "accepted".to_string(),
                accepted_at: accepted_at.to_string(),
                accepted_observation_id: None,
                accepted_evidence_id: None,
                summary: None,
            }],
            ..EpiphanyThreadState::default()
        }
    }

    #[test]
    fn hands_commit_requires_modeling_refresh_until_newer_modeling_acceptance() {
        let implementation = finding(
            "implementation-result",
            vec!["hands-receipt:hands-commit-worker-job-impl".to_string()],
        );
        let modeling = finding("modeling-result", Vec::new());

        assert!(implementation_commit_requires_modeling_refresh(
            None,
            CoreEpiphanyCoordinatorRoleResultStatus::Completed,
            Some(&implementation),
            Some("2026-06-02T12:00:00Z"),
            Some(&modeling),
        ));

        let stale_state = state_with_modeling_acceptance("2026-06-02T11:59:00Z");
        assert!(implementation_commit_requires_modeling_refresh(
            Some(&stale_state),
            CoreEpiphanyCoordinatorRoleResultStatus::Completed,
            Some(&implementation),
            Some("2026-06-02T12:00:00Z"),
            Some(&modeling),
        ));

        let refreshed_state = state_with_modeling_acceptance("2026-06-02T12:01:00Z");
        assert!(!implementation_commit_requires_modeling_refresh(
            Some(&refreshed_state),
            CoreEpiphanyCoordinatorRoleResultStatus::Completed,
            Some(&implementation),
            Some("2026-06-02T12:00:00Z"),
            Some(&modeling),
        ));
    }
}
