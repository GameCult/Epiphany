use std::sync::Arc;

use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::ThreadEpiphanyStateUpdatedNotification;
use codex_app_server_protocol::ThreadEpiphanyStateUpdatedSource;
use codex_core::CodexThread;
use codex_core::SteerInputError;
use codex_protocol::ThreadId;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::TokenUsageInfo as CoreTokenUsageInfo;
use codex_protocol::user_input::UserInput as CoreInputItem;
use epiphany_codex_bridge::coordinator::EpiphanyCoordinatorAutomationAction;
use epiphany_codex_bridge::coordinator::EpiphanyCoordinatorAutomationInput;
use epiphany_codex_bridge::coordinator::select_epiphany_coordinator_automation;
use epiphany_codex_bridge::invalidation::EpiphanyInvalidationManager;
use epiphany_codex_bridge::invalidation::epiphany_freshness_watcher_snapshot;
use epiphany_codex_bridge::mutation::epiphany_job_launch_changed_fields;
use epiphany_codex_bridge::pressure::map_epiphany_pressure;
use epiphany_codex_bridge::pressure::render_epiphany_pre_compaction_checkpoint_intervention;
use epiphany_codex_bridge::pressure::should_run_epiphany_pre_compaction_checkpoint_intervention;
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

    let verdict = select_epiphany_coordinator_automation(EpiphanyCoordinatorAutomationInput {
        thread_id: &thread_id_text,
        state: &state,
        retrieval_override: &retrieval_override,
        watcher_snapshot: epiphany_freshness_watcher_snapshot(&watcher_snapshot),
        token_usage_info: token_usage_info.as_ref(),
        runtime_store_path: runtime_store_path.as_path(),
        force_checkpoint_compaction,
    })
    .await;

    match verdict.action {
        EpiphanyCoordinatorAutomationAction::None => {}
        EpiphanyCoordinatorAutomationAction::CompactRehydrateReorient => {
            if let Err(err) = thread.submit(Op::Compact).await {
                warn!(
                    "failed to run Epiphany coordinator automatic compaction for {thread_id}: {err}"
                );
            }
        }
        EpiphanyCoordinatorAutomationAction::LaunchReorientWorker => {
            let Some(launch_request) = verdict.launch_request else {
                return;
            };
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
