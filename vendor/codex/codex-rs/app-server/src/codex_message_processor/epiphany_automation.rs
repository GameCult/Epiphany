use std::sync::Arc;

use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::ThreadEpiphanyReorientStateStatus;
use codex_app_server_protocol::ThreadEpiphanyStateUpdatedNotification;
use codex_app_server_protocol::ThreadEpiphanyStateUpdatedSource;
use codex_core::CodexThread;
use codex_core::SteerInputError;
use codex_protocol::ThreadId;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::TokenUsageInfo as CoreTokenUsageInfo;
use codex_protocol::user_input::UserInput as CoreInputItem;
use epiphany_codex_bridge::coordinator::EpiphanyCoordinatorAutomationAction;
use epiphany_codex_bridge::coordinator::derive_epiphany_coordinator_status;
use epiphany_codex_bridge::coordinator::epiphany_reorient_finding_already_accepted;
use epiphany_codex_bridge::coordinator::map_epiphany_crrc_recommendation;
use epiphany_codex_bridge::coordinator::map_epiphany_roles;
use epiphany_codex_bridge::coordinator::select_epiphany_coordinator_automation_action;
use epiphany_codex_bridge::invalidation::EpiphanyInvalidationManager;
use epiphany_codex_bridge::invalidation::epiphany_freshness_watcher_snapshot;
use epiphany_codex_bridge::jobs::map_epiphany_jobs;
use epiphany_codex_bridge::launch::EPIPHANY_REORIENT_LAUNCH_BINDING_ID;
use epiphany_codex_bridge::launch::build_epiphany_reorient_launch_request;
use epiphany_codex_bridge::mutation::epiphany_job_launch_changed_fields;
use epiphany_codex_bridge::pressure::map_epiphany_pressure;
use epiphany_codex_bridge::pressure::render_epiphany_pre_compaction_checkpoint_intervention;
use epiphany_codex_bridge::pressure::should_run_epiphany_pre_compaction_checkpoint_intervention;
use epiphany_codex_bridge::reorient::map_epiphany_freshness;
use epiphany_codex_bridge::reorient::map_epiphany_reorient;
use epiphany_codex_bridge::runtime_results::load_epiphany_reorient_result_snapshot;
use epiphany_codex_bridge::state::client_visible_live_thread_epiphany_state;
use tokio::sync::Mutex;
use tracing::warn;

use crate::outgoing_message::ThreadScopedOutgoingMessageSender;
use crate::thread_state::ThreadState;

pub(crate) async fn maybe_run_epiphany_coordinator_automation_for_turn_boundary(
    thread_id: ThreadId,
    thread: Arc<CodexThread>,
    epiphany_invalidation_manager: EpiphanyInvalidationManager,
    outgoing: &ThreadScopedOutgoingMessageSender,
    force_checkpoint_compaction: bool,
) {
    let thread_id_text = thread_id.to_string();
    let Some(state) = thread.epiphany_state().await else {
        return;
    };

    let retrieval_override = thread.epiphany_retrieval_state().await;
    let config_snapshot = thread.config_snapshot().await;
    epiphany_invalidation_manager
        .ensure_thread_watch(&thread_id_text, &config_snapshot.cwd)
        .await;
    let watcher_snapshot = epiphany_invalidation_manager
        .snapshot(&thread_id_text)
        .await;
    let token_usage_info = thread.token_usage_info().await;
    let runtime_store_path = thread.epiphany_runtime_spine_store_path().await;

    let (_state_revision, retrieval, graph, watcher) = map_epiphany_freshness(
        Some(&state),
        Some(&retrieval_override),
        Some(epiphany_freshness_watcher_snapshot(&watcher_snapshot)),
    );
    let pressure = map_epiphany_pressure(token_usage_info.as_ref());
    let (state_status, reorient_decision) =
        map_epiphany_reorient(Some(&state), &pressure, &retrieval, &graph, &watcher);
    if state_status != ThreadEpiphanyReorientStateStatus::Ready {
        return;
    }

    let jobs = map_epiphany_jobs(Some(&state), Some(&retrieval_override));
    let reorient_job = jobs
        .iter()
        .find(|job| job.id == EPIPHANY_REORIENT_LAUNCH_BINDING_ID)
        .cloned();
    let (reorient_result_status, reorient_finding, _) = load_epiphany_reorient_result_snapshot(
        Some(&state),
        Some(runtime_store_path.as_path()),
        EPIPHANY_REORIENT_LAUNCH_BINDING_ID,
    )
    .await;
    let reorient_finding_accepted = reorient_finding
        .as_ref()
        .is_some_and(|finding| epiphany_reorient_finding_already_accepted(&state, finding));
    let crrc_recommendation = map_epiphany_crrc_recommendation(
        true,
        state_status,
        &pressure,
        &reorient_decision,
        reorient_result_status,
        state.investigation_checkpoint.as_ref().is_some(),
        reorient_finding.is_some(),
        reorient_finding_accepted,
    );
    let roles = map_epiphany_roles(
        Some(&state),
        &jobs,
        &reorient_decision,
        &pressure,
        &crrc_recommendation,
        reorient_result_status,
        reorient_job.as_ref(),
    );
    let coordinator = derive_epiphany_coordinator_status(
        Some(&state),
        Some(runtime_store_path.as_path()),
        state_status,
        &pressure,
        &crrc_recommendation,
        roles,
        Some(&reorient_decision),
        reorient_result_status,
        reorient_finding.as_ref(),
        state.investigation_checkpoint.as_ref().is_some(),
    )
    .await;

    match select_epiphany_coordinator_automation_action(
        &coordinator.decision,
        force_checkpoint_compaction,
    ) {
        EpiphanyCoordinatorAutomationAction::None => {}
        EpiphanyCoordinatorAutomationAction::CompactRehydrateReorient => {
            if let Err(err) = thread.submit(Op::Compact).await {
                warn!(
                    "failed to run Epiphany coordinator automatic compaction for {thread_id}: {err}"
                );
            }
        }
        EpiphanyCoordinatorAutomationAction::LaunchReorientWorker => {
            let Some(checkpoint) = state.investigation_checkpoint.as_ref() else {
                return;
            };
            let launch_request = build_epiphany_reorient_launch_request(
                &thread_id_text,
                Some(state.revision),
                None,
                &state,
                checkpoint,
                &reorient_decision,
            );
            let launched = match thread.epiphany_launch_job(launch_request).await {
                Ok(launched) => launched,
                Err(err) => {
                    warn!(
                        "failed to launch Epiphany coordinator reorientation worker for {thread_id}: {err}"
                    );
                    return;
                }
            };
            let epiphany_state =
                client_visible_live_thread_epiphany_state(thread.as_ref(), launched.epiphany_state)
                    .await;
            outgoing
                .send_server_notification(ServerNotification::ThreadEpiphanyStateUpdated(
                    ThreadEpiphanyStateUpdatedNotification {
                        thread_id: thread_id_text,
                        source: ThreadEpiphanyStateUpdatedSource::JobLaunch,
                        revision: epiphany_state.revision,
                        changed_fields: epiphany_job_launch_changed_fields(),
                        epiphany_state,
                    },
                ))
                .await;
        }
    }
}

pub(crate) async fn maybe_run_epiphany_pre_compaction_checkpoint_intervention_for_token_count(
    thread_id: ThreadId,
    turn_id: String,
    thread: Arc<CodexThread>,
    token_usage_info: Option<CoreTokenUsageInfo>,
    thread_state: &Arc<Mutex<ThreadState>>,
) {
    let pressure = map_epiphany_pressure(token_usage_info.as_ref());
    if !should_run_epiphany_pre_compaction_checkpoint_intervention(&pressure) {
        return;
    }
    if thread.epiphany_state().await.is_none() {
        return;
    }
    {
        let mut state = thread_state.lock().await;
        if state.turn_summary.context_compaction_started {
            return;
        }
        if !state.record_epiphany_checkpoint_intervention(&turn_id) {
            return;
        }
    }

    let text = render_epiphany_pre_compaction_checkpoint_intervention(&pressure);
    match thread
        .steer_input(
            vec![CoreInputItem::Text {
                text,
                text_elements: Vec::new(),
            }],
            Some(&turn_id),
            None,
        )
        .await
    {
        Ok(_) => {
            thread_state
                .lock()
                .await
                .mark_epiphany_checkpoint_intervention_pending_compaction(&turn_id);
        }
        Err(SteerInputError::NoActiveTurn(_))
        | Err(SteerInputError::ExpectedTurnMismatch { .. })
        | Err(SteerInputError::ActiveTurnNotSteerable { .. })
        | Err(SteerInputError::EmptyInput) => {
            warn!(
                "failed to steer Epiphany pre-compaction checkpoint intervention for {thread_id}"
            );
        }
    }
}
