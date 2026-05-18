use std::sync::Arc;

use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::ThreadEpiphanyStateUpdatedNotification;
use codex_app_server_protocol::ThreadEpiphanyStateUpdatedSource;
use codex_core::CodexThread;
use codex_core::SteerInputError;
use codex_protocol::ThreadId;
use codex_protocol::protocol::EpiphanyJobKind as CoreEpiphanyJobKind;
use codex_protocol::protocol::Op;
use codex_protocol::protocol::TokenUsageInfo as CoreTokenUsageInfo;
use codex_protocol::user_input::UserInput as CoreInputItem;
use epiphany_codex_bridge::coordinator::EpiphanyCoordinatorAutomationAction;
use epiphany_codex_bridge::coordinator::select_thread_epiphany_coordinator_automation;
use epiphany_codex_bridge::invalidation::EpiphanyInvalidationManager;
use epiphany_codex_bridge::mutation_service::launch_thread_epiphany_job;
use epiphany_codex_bridge::pressure::render_pre_compaction_checkpoint_intervention_for_token_usage;
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
    let Some(verdict) = select_thread_epiphany_coordinator_automation(
        &thread_id_text,
        thread.as_ref(),
        &epiphany_invalidation_manager,
        force_checkpoint_compaction,
    )
    .await
    else {
        return;
    };

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
            let applied = match launch_thread_epiphany_job(
                thread.as_ref(),
                launch_request,
                CoreEpiphanyJobKind::Specialist,
                "missing coordinator-launched reorient projection",
            )
            .await
            {
                Ok(applied) => applied,
                Err(err) => {
                    warn!(
                        "failed to launch Epiphany coordinator reorientation worker for {thread_id}: {err}"
                    );
                    return;
                }
            };
            outgoing
                .send_server_notification(ServerNotification::ThreadEpiphanyStateUpdated(
                    ThreadEpiphanyStateUpdatedNotification {
                        thread_id: thread_id_text,
                        source: ThreadEpiphanyStateUpdatedSource::JobLaunch,
                        revision: applied.revision,
                        changed_fields: applied.changed_fields,
                        epiphany_state: applied.epiphany_state,
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
    let Some(text) =
        render_pre_compaction_checkpoint_intervention_for_token_usage(token_usage_info.as_ref())
    else {
        return;
    };
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
