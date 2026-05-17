use super::*;

impl CodexMessageProcessor {
    pub(super) async fn thread_fork(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadForkParams,
    ) {
        let ThreadForkParams {
            thread_id,
            path,
            model,
            model_provider,
            service_tier,
            cwd,
            approval_policy,
            approvals_reviewer,
            sandbox,
            permission_profile,
            config: cli_overrides,
            base_instructions,
            developer_instructions,
            ephemeral,
            persist_extended_history,
        } = params;
        if sandbox.is_some() && permission_profile.is_some() {
            self.send_invalid_request_error(
                request_id,
                "`permissionProfile` cannot be combined with `sandbox`".to_string(),
            )
            .await;
            return;
        }

        let (rollout_path, source_thread_id) = if let Some(path) = path {
            (path, None)
        } else {
            let existing_thread_id = match ThreadId::from_string(&thread_id) {
                Ok(id) => id,
                Err(err) => {
                    self.send_invalid_request_error(
                        request_id,
                        format!("invalid thread id: {err}"),
                    )
                    .await;
                    return;
                }
            };

            match find_thread_path_by_id_str(
                &self.config.codex_home,
                &existing_thread_id.to_string(),
            )
            .await
            {
                Ok(Some(p)) => (p, Some(existing_thread_id)),
                Ok(None) => {
                    self.send_invalid_request_error(
                        request_id,
                        format!("no rollout found for thread id {existing_thread_id}"),
                    )
                    .await;
                    return;
                }
                Err(err) => {
                    self.send_invalid_request_error(
                        request_id,
                        format!("failed to locate thread id {existing_thread_id}: {err}"),
                    )
                    .await;
                    return;
                }
            }
        };

        let history_cwd =
            read_history_cwd_from_state_db(&self.config, source_thread_id, rollout_path.as_path())
                .await;

        // Persist Windows sandbox mode.
        let mut cli_overrides = cli_overrides.unwrap_or_default();
        if cfg!(windows) {
            match WindowsSandboxLevel::from_config(&self.config) {
                WindowsSandboxLevel::Elevated => {
                    cli_overrides
                        .insert("windows.sandbox".to_string(), serde_json::json!("elevated"));
                }
                WindowsSandboxLevel::RestrictedToken => {
                    cli_overrides.insert(
                        "windows.sandbox".to_string(),
                        serde_json::json!("unelevated"),
                    );
                }
                WindowsSandboxLevel::Disabled => {}
            }
        }
        let request_overrides = if cli_overrides.is_empty() {
            None
        } else {
            Some(cli_overrides)
        };
        let mut typesafe_overrides = self.build_thread_config_overrides(
            model,
            model_provider,
            service_tier,
            cwd,
            approval_policy,
            approvals_reviewer,
            sandbox,
            permission_profile,
            base_instructions,
            developer_instructions,
            /*personality*/ None,
        );
        typesafe_overrides.ephemeral = ephemeral.then_some(true);
        // Derive a Config using the same logic as new conversation, honoring overrides if provided.
        let config = match self
            .config_manager
            .load_for_cwd(request_overrides, typesafe_overrides, history_cwd)
            .await
        {
            Ok(config) => config,
            Err(err) => {
                self.outgoing
                    .send_error(request_id, config_load_error(&err))
                    .await;
                return;
            }
        };

        let fallback_model_provider = config.model_provider_id.clone();
        let instruction_sources = Self::instruction_sources_from_config(&config).await;

        let NewThread {
            thread_id,
            thread: forked_thread,
            session_configured,
            ..
        } = match self
            .thread_manager
            .fork_thread(
                ForkSnapshot::Interrupted,
                config,
                rollout_path.clone(),
                persist_extended_history,
                self.request_trace_context(&request_id).await,
            )
            .await
        {
            Ok(thread) => thread,
            Err(err) => {
                match err {
                    CodexErr::Io(_) | CodexErr::Json(_) => {
                        self.send_invalid_request_error(
                            request_id,
                            format!("failed to load rollout `{}`: {err}", rollout_path.display()),
                        )
                        .await;
                    }
                    CodexErr::InvalidRequest(message) => {
                        self.send_invalid_request_error(request_id, message).await;
                    }
                    _ => {
                        self.send_internal_error(
                            request_id,
                            format!("error forking thread: {err}"),
                        )
                        .await;
                    }
                }
                return;
            }
        };

        // Auto-attach a conversation listener when forking a thread.
        Self::log_listener_attach_result(
            self.ensure_conversation_listener(
                thread_id,
                request_id.connection_id,
                /*raw_events_enabled*/ false,
                ApiVersion::V2,
            )
            .await,
            thread_id,
            request_id.connection_id,
            "thread",
        );

        // Persistent forks materialize their own rollout immediately. Ephemeral forks stay
        // pathless, so they rebuild their visible history from the copied source rollout instead.
        let mut thread = if let Some(fork_rollout_path) = session_configured.rollout_path.as_ref() {
            match read_summary_from_rollout(
                fork_rollout_path.as_path(),
                fallback_model_provider.as_str(),
            )
            .await
            {
                Ok(summary) => {
                    let mut thread = summary_to_thread(summary, &self.config.cwd);
                    thread.forked_from_id =
                        forked_from_id_from_rollout(fork_rollout_path.as_path()).await;
                    thread
                }
                Err(err) => {
                    self.send_internal_error(
                        request_id,
                        format!(
                            "failed to load rollout `{}` for thread {thread_id}: {err}",
                            fork_rollout_path.display()
                        ),
                    )
                    .await;
                    return;
                }
            }
        } else {
            let config_snapshot = forked_thread.config_snapshot().await;
            // forked thread names do not inherit the source thread name
            let mut thread =
                build_thread_from_snapshot(thread_id, &config_snapshot, /*path*/ None);
            let history_items = match read_rollout_items_from_rollout(rollout_path.as_path()).await
            {
                Ok(items) => items,
                Err(err) => {
                    self.send_internal_error(
                        request_id,
                        format!(
                            "failed to load source rollout `{}` for thread {thread_id}: {err}",
                            rollout_path.display()
                        ),
                    )
                    .await;
                    return;
                }
            };
            thread.preview = preview_from_rollout_items(&history_items);
            thread.forked_from_id = source_thread_id
                .or_else(|| {
                    history_items.iter().find_map(|item| match item {
                        RolloutItem::SessionMeta(meta_line) => Some(meta_line.meta.id),
                        _ => None,
                    })
                })
                .map(|id| id.to_string());
            if let Err(message) = populate_thread_turns(
                &mut thread,
                ThreadTurnSource::HistoryItems(&history_items),
                /*active_turn*/ None,
            )
            .await
            {
                self.send_internal_error(request_id, message).await;
                return;
            }
            thread
        };

        if let Some(fork_rollout_path) = session_configured.rollout_path.as_ref()
            && let Err(message) = populate_thread_turns(
                &mut thread,
                ThreadTurnSource::RolloutPath(fork_rollout_path.as_path()),
                /*active_turn*/ None,
            )
            .await
        {
            self.send_internal_error(request_id, message).await;
            return;
        }

        thread.epiphany_state = live_thread_epiphany_state(forked_thread.as_ref()).await;

        self.thread_watch_manager
            .upsert_thread_silently(thread.clone())
            .await;

        thread.status = resolve_thread_status(
            self.thread_watch_manager
                .loaded_status_for_thread(&thread.id)
                .await,
            /*has_in_progress_turn*/ false,
        );
        let permission_profile = thread_response_permission_profile(
            &session_configured.sandbox_policy,
            forked_thread.config_snapshot().await.permission_profile,
        );

        let response = ThreadForkResponse {
            thread: thread.clone(),
            model: session_configured.model,
            model_provider: session_configured.model_provider_id,
            service_tier: session_configured.service_tier,
            cwd: session_configured.cwd,
            instruction_sources,
            approval_policy: session_configured.approval_policy.into(),
            approvals_reviewer: session_configured.approvals_reviewer.into(),
            sandbox: session_configured.sandbox_policy.into(),
            permission_profile,
            reasoning_effort: session_configured.reasoning_effort,
        };
        if self.config.features.enabled(Feature::GeneralAnalytics) {
            self.analytics_events_client.track_response(
                request_id.connection_id.0,
                ClientResponse::ThreadFork {
                    request_id: request_id.request_id.clone(),
                    response: response.clone(),
                },
            );
        }

        let connection_id = request_id.connection_id;
        let token_usage_thread = response.thread.clone();
        let token_usage_turn_id = if let Some(turn_id) =
            latest_token_usage_turn_id_for_thread_path(&token_usage_thread).await
        {
            Some(turn_id)
        } else {
            latest_token_usage_turn_id_from_rollout_path(
                rollout_path.as_path(),
                &token_usage_thread,
            )
            .await
        };
        self.outgoing.send_response(request_id, response).await;
        // Mirror the resume contract for forks: the new thread is usable as soon
        // as the response arrives, so restored usage must follow immediately.
        send_thread_token_usage_update_to_connection(
            &self.outgoing,
            connection_id,
            thread_id,
            &token_usage_thread,
            forked_thread.as_ref(),
            token_usage_turn_id,
        )
        .await;

        let notif = ThreadStartedNotification { thread };
        self.outgoing
            .send_server_notification(ServerNotification::ThreadStarted(notif))
            .await;
    }
}
