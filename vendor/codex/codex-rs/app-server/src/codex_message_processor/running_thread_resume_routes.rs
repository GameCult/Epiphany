use super::*;

impl CodexMessageProcessor {
    pub(super) async fn resume_running_thread(
        &self,
        request_id: ConnectionRequestId,
        params: &ThreadResumeParams,
    ) -> bool {
        if let Ok(existing_thread_id) = ThreadId::from_string(&params.thread_id)
            && let Ok(existing_thread) = self.thread_manager.get_thread(existing_thread_id).await
        {
            if params.history.is_some() {
                self.send_invalid_request_error(
                    request_id,
                    format!(
                        "cannot resume thread {existing_thread_id} with history while it is already running"
                    ),
                )
                .await;
                return true;
            }

            let rollout_path = if let Some(path) = existing_thread.rollout_path() {
                if path.exists() {
                    path
                } else {
                    match find_thread_path_by_id_str(
                        &self.config.codex_home,
                        &existing_thread_id.to_string(),
                    )
                    .await
                    {
                        Ok(Some(path)) => path,
                        Ok(None) => {
                            self.send_invalid_request_error(
                                request_id,
                                format!("no rollout found for thread id {existing_thread_id}"),
                            )
                            .await;
                            return true;
                        }
                        Err(err) => {
                            self.send_invalid_request_error(
                                request_id,
                                format!("failed to locate thread id {existing_thread_id}: {err}"),
                            )
                            .await;
                            return true;
                        }
                    }
                }
            } else {
                match find_thread_path_by_id_str(
                    &self.config.codex_home,
                    &existing_thread_id.to_string(),
                )
                .await
                {
                    Ok(Some(path)) => path,
                    Ok(None) => {
                        self.send_invalid_request_error(
                            request_id,
                            format!("no rollout found for thread id {existing_thread_id}"),
                        )
                        .await;
                        return true;
                    }
                    Err(err) => {
                        self.send_invalid_request_error(
                            request_id,
                            format!("failed to locate thread id {existing_thread_id}: {err}"),
                        )
                        .await;
                        return true;
                    }
                }
            };

            if let Some(requested_path) = params.path.as_ref()
                && requested_path != &rollout_path
            {
                self.send_invalid_request_error(
                    request_id,
                    format!(
                        "cannot resume running thread {existing_thread_id} with mismatched path: requested `{}`, active `{}`",
                        requested_path.display(),
                        rollout_path.display()
                    ),
                )
                .await;
                return true;
            }

            let thread_state = self
                .thread_state_manager
                .thread_state(existing_thread_id)
                .await;
            if let Err(error) = self
                .ensure_listener_task_running(
                    existing_thread_id,
                    existing_thread.clone(),
                    thread_state.clone(),
                    ApiVersion::V2,
                )
                .await
            {
                self.outgoing.send_error(request_id, error).await;
                return true;
            }

            let config_snapshot = existing_thread.config_snapshot().await;
            let mismatch_details = collect_resume_override_mismatches(params, &config_snapshot);
            if !mismatch_details.is_empty() {
                tracing::warn!(
                    "thread/resume overrides ignored for running thread {}: {}",
                    existing_thread_id,
                    mismatch_details.join("; ")
                );
            }
            let mut config_for_instruction_sources = self.config.as_ref().clone();
            config_for_instruction_sources.cwd = config_snapshot.cwd.clone();
            let instruction_sources =
                Self::instruction_sources_from_config(&config_for_instruction_sources).await;
            let thread_summary = match load_thread_summary_for_rollout(
                &self.config,
                existing_thread_id,
                rollout_path.as_path(),
                config_snapshot.model_provider_id.as_str(),
                /*persisted_metadata*/ None,
            )
            .await
            {
                Ok(thread) => thread,
                Err(message) => {
                    self.send_internal_error(request_id, message).await;
                    return true;
                }
            };

            let listener_command_tx = {
                let thread_state = thread_state.lock().await;
                thread_state.listener_command_tx()
            };
            let Some(listener_command_tx) = listener_command_tx else {
                let err = JSONRPCErrorError {
                    code: INTERNAL_ERROR_CODE,
                    message: format!(
                        "failed to enqueue running thread resume for thread {existing_thread_id}: thread listener is not running"
                    ),
                    data: None,
                };
                self.outgoing.send_error(request_id, err).await;
                return true;
            };

            let command = crate::thread_state::ThreadListenerCommand::SendThreadResumeResponse(
                Box::new(crate::thread_state::PendingThreadResumeRequest {
                    request_id: request_id.clone(),
                    rollout_path: rollout_path.clone(),
                    config_snapshot,
                    instruction_sources,
                    thread_summary,
                }),
            );
            if listener_command_tx.send(command).is_err() {
                let err = JSONRPCErrorError {
                    code: INTERNAL_ERROR_CODE,
                    message: format!(
                        "failed to enqueue running thread resume for thread {existing_thread_id}: thread listener command channel is closed"
                    ),
                    data: None,
                };
                self.outgoing.send_error(request_id, err).await;
            }
            return true;
        }
        false
    }
}

#[allow(clippy::too_many_arguments)]
#[expect(
    clippy::await_holding_invalid_type,
    reason = "running-thread resume subscription must be serialized against pending unloads"
)]
pub(crate) async fn handle_pending_thread_resume_request(
    conversation_id: ThreadId,
    conversation: &Arc<CodexThread>,
    _codex_home: &Path,
    thread_state_manager: &ThreadStateManager,
    thread_state: &Arc<Mutex<ThreadState>>,
    thread_watch_manager: &ThreadWatchManager,
    outgoing: &Arc<OutgoingMessageSender>,
    pending_thread_unloads: &Arc<Mutex<HashSet<ThreadId>>>,
    pending: crate::thread_state::PendingThreadResumeRequest,
) {
    let active_turn = {
        let state = thread_state.lock().await;
        state.active_turn_snapshot()
    };
    tracing::debug!(
        thread_id = %conversation_id,
        request_id = ?pending.request_id,
        active_turn_present = active_turn.is_some(),
        active_turn_id = ?active_turn.as_ref().map(|turn| turn.id.as_str()),
        active_turn_status = ?active_turn.as_ref().map(|turn| &turn.status),
        "composing running thread resume response"
    );
    let has_live_in_progress_turn =
        matches!(conversation.agent_status().await, AgentStatus::Running)
            || active_turn
                .as_ref()
                .is_some_and(|turn| matches!(turn.status, TurnStatus::InProgress));

    let request_id = pending.request_id;
    let connection_id = request_id.connection_id;
    let mut thread = pending.thread_summary;
    if let Err(message) = populate_thread_turns(
        &mut thread,
        ThreadTurnSource::RolloutPath(pending.rollout_path.as_path()),
        active_turn.as_ref(),
    )
    .await
    {
        outgoing
            .send_error(
                request_id,
                JSONRPCErrorError {
                    code: INTERNAL_ERROR_CODE,
                    message,
                    data: None,
                },
            )
            .await;
        return;
    }

    let thread_status = thread_watch_manager
        .loaded_status_for_thread(&thread.id)
        .await;

    set_thread_status_and_interrupt_stale_turns(
        &mut thread,
        thread_status,
        has_live_in_progress_turn,
    );

    {
        let pending_thread_unloads = pending_thread_unloads.lock().await;
        if pending_thread_unloads.contains(&conversation_id) {
            drop(pending_thread_unloads);
            outgoing
                .send_error(
                    request_id,
                    JSONRPCErrorError {
                        code: INVALID_REQUEST_ERROR_CODE,
                        message: format!(
                            "thread {conversation_id} is closing; retry thread/resume after the thread is closed"
                        ),
                        data: None,
                    },
                )
                .await;
            return;
        }
        if !thread_state_manager
            .try_add_connection_to_thread(conversation_id, connection_id)
            .await
        {
            tracing::debug!(
                thread_id = %conversation_id,
                connection_id = ?connection_id,
                "skipping running thread resume for closed connection"
            );
            return;
        }
    }

    let ThreadConfigSnapshot {
        model,
        model_provider_id,
        service_tier,
        approval_policy,
        approvals_reviewer,
        sandbox_policy,
        permission_profile,
        cwd,
        reasoning_effort,
        ..
    } = pending.config_snapshot;
    let instruction_sources = pending.instruction_sources;
    let permission_profile =
        thread_response_permission_profile(&sandbox_policy, permission_profile);

    let response = ThreadResumeResponse {
        thread,
        model,
        model_provider: model_provider_id,
        service_tier,
        cwd,
        instruction_sources,
        approval_policy: approval_policy.into(),
        approvals_reviewer: approvals_reviewer.into(),
        sandbox: sandbox_policy.into(),
        permission_profile,
        reasoning_effort,
    };
    let token_usage_thread = response.thread.clone();
    let token_usage_turn_id = latest_token_usage_turn_id_from_rollout_path(
        pending.rollout_path.as_path(),
        &token_usage_thread,
    )
    .await;
    outgoing.send_response(request_id, response).await;
    // Rejoining a loaded thread has the same UI contract as a cold resume, but
    // uses the live conversation state instead of reconstructing a new session.
    send_thread_token_usage_update_to_connection(
        outgoing,
        connection_id,
        conversation_id,
        &token_usage_thread,
        conversation.as_ref(),
        token_usage_turn_id,
    )
    .await;
    outgoing
        .replay_requests_to_connection_for_thread(connection_id, conversation_id)
        .await;
}

fn collect_resume_override_mismatches(
    request: &ThreadResumeParams,
    config_snapshot: &ThreadConfigSnapshot,
) -> Vec<String> {
    let mut mismatch_details = Vec::new();

    if let Some(requested_model) = request.model.as_deref()
        && requested_model != config_snapshot.model
    {
        mismatch_details.push(format!(
            "model requested={requested_model} active={}",
            config_snapshot.model
        ));
    }
    if let Some(requested_provider) = request.model_provider.as_deref()
        && requested_provider != config_snapshot.model_provider_id
    {
        mismatch_details.push(format!(
            "model_provider requested={requested_provider} active={}",
            config_snapshot.model_provider_id
        ));
    }
    if let Some(requested_service_tier) = request.service_tier.as_ref()
        && requested_service_tier != &config_snapshot.service_tier
    {
        mismatch_details.push(format!(
            "service_tier requested={requested_service_tier:?} active={:?}",
            config_snapshot.service_tier
        ));
    }
    if let Some(requested_cwd) = request.cwd.as_deref() {
        let requested_cwd_path = std::path::PathBuf::from(requested_cwd);
        if requested_cwd_path != config_snapshot.cwd.as_path() {
            mismatch_details.push(format!(
                "cwd requested={} active={}",
                requested_cwd_path.display(),
                config_snapshot.cwd.display()
            ));
        }
    }
    if let Some(requested_approval) = request.approval_policy.as_ref() {
        let active_approval: AskForApproval = config_snapshot.approval_policy.into();
        if requested_approval != &active_approval {
            mismatch_details.push(format!(
                "approval_policy requested={requested_approval:?} active={active_approval:?}"
            ));
        }
    }
    if let Some(requested_review_policy) = request.approvals_reviewer.as_ref() {
        let active_review_policy: codex_app_server_protocol::ApprovalsReviewer =
            config_snapshot.approvals_reviewer.into();
        if requested_review_policy != &active_review_policy {
            mismatch_details.push(format!(
                "approvals_reviewer requested={requested_review_policy:?} active={active_review_policy:?}"
            ));
        }
    }
    if let Some(requested_sandbox) = request.sandbox.as_ref() {
        let sandbox_matches = matches!(
            (requested_sandbox, &config_snapshot.sandbox_policy),
            (
                SandboxMode::ReadOnly,
                codex_protocol::protocol::SandboxPolicy::ReadOnly { .. }
            ) | (
                SandboxMode::WorkspaceWrite,
                codex_protocol::protocol::SandboxPolicy::WorkspaceWrite { .. }
            ) | (
                SandboxMode::DangerFullAccess,
                codex_protocol::protocol::SandboxPolicy::DangerFullAccess
            ) | (
                SandboxMode::DangerFullAccess,
                codex_protocol::protocol::SandboxPolicy::ExternalSandbox { .. }
            )
        );
        if !sandbox_matches {
            mismatch_details.push(format!(
                "sandbox requested={requested_sandbox:?} active={:?}",
                config_snapshot.sandbox_policy
            ));
        }
    }
    if let Some(requested_permission_profile) = request.permission_profile.as_ref() {
        let requested_permission_profile =
            codex_protocol::models::PermissionProfile::from(requested_permission_profile.clone());
        if requested_permission_profile != config_snapshot.permission_profile {
            mismatch_details.push(format!(
                "permission_profile requested={requested_permission_profile:?} active={:?}",
                config_snapshot.permission_profile
            ));
        }
    }
    if let Some(requested_personality) = request.personality.as_ref()
        && config_snapshot.personality.as_ref() != Some(requested_personality)
    {
        mismatch_details.push(format!(
            "personality requested={requested_personality:?} active={:?}",
            config_snapshot.personality
        ));
    }

    if request.config.is_some() {
        mismatch_details
            .push("config overrides were provided and ignored while running".to_string());
    }
    if request.base_instructions.is_some() {
        mismatch_details
            .push("baseInstructions override was provided and ignored while running".to_string());
    }
    if request.developer_instructions.is_some() {
        mismatch_details.push(
            "developerInstructions override was provided and ignored while running".to_string(),
        );
    }
    if request.persist_extended_history {
        mismatch_details.push(
            "persistExtendedHistory override was provided and ignored while running".to_string(),
        );
    }

    mismatch_details
}
