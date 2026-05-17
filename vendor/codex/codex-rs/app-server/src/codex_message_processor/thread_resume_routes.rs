use super::*;

impl CodexMessageProcessor {
    pub(super) async fn thread_resume(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadResumeParams,
    ) {
        if let Ok(thread_id) = ThreadId::from_string(&params.thread_id)
            && self
                .pending_thread_unloads
                .lock()
                .await
                .contains(&thread_id)
        {
            self.send_invalid_request_error(
                request_id,
                format!(
                    "thread {thread_id} is closing; retry thread/resume after the thread is closed"
                ),
            )
            .await;
            return;
        }

        if params.sandbox.is_some() && params.permission_profile.is_some() {
            self.send_invalid_request_error(
                request_id,
                "`permissionProfile` cannot be combined with `sandbox`".to_string(),
            )
            .await;
            return;
        }

        if self
            .resume_running_thread(request_id.clone(), &params)
            .await
        {
            return;
        }

        let ThreadResumeParams {
            thread_id,
            history,
            path,
            model,
            model_provider,
            service_tier,
            cwd,
            approval_policy,
            approvals_reviewer,
            sandbox,
            permission_profile,
            config: mut request_overrides,
            base_instructions,
            developer_instructions,
            personality,
            persist_extended_history,
        } = params;

        let thread_history = if let Some(history) = history {
            let Some(thread_history) = self
                .resume_thread_from_history(request_id.clone(), history.as_slice())
                .await
            else {
                return;
            };
            thread_history
        } else {
            let Some(thread_history) = self
                .resume_thread_from_rollout(request_id.clone(), &thread_id, path.as_ref())
                .await
            else {
                return;
            };
            thread_history
        };

        let history_cwd = thread_history.session_cwd();
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
            personality,
        );
        let persisted_resume_metadata = self
            .load_and_apply_persisted_resume_metadata(
                &thread_history,
                &mut request_overrides,
                &mut typesafe_overrides,
            )
            .await;

        // Derive a Config using the same logic as new conversation, honoring overrides if provided.
        let config = match self
            .config_manager
            .load_for_cwd(request_overrides, typesafe_overrides, history_cwd)
            .await
        {
            Ok(config) => config,
            Err(err) => {
                let error = config_load_error(&err);
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        let fallback_model_provider = config.model_provider_id.clone();
        let instruction_sources = Self::instruction_sources_from_config(&config).await;
        let response_history = thread_history.clone();

        match self
            .thread_manager
            .resume_thread_with_history(
                config,
                thread_history,
                self.auth_manager.clone(),
                persist_extended_history,
                self.request_trace_context(&request_id).await,
            )
            .await
        {
            Ok(NewThread {
                thread_id,
                thread: codex_thread,
                session_configured,
                ..
            }) => {
                let SessionConfiguredEvent { rollout_path, .. } = session_configured;
                let Some(rollout_path) = rollout_path else {
                    self.send_internal_error(
                        request_id,
                        format!("rollout path missing for thread {thread_id}"),
                    )
                    .await;
                    return;
                };
                // Auto-attach a thread listener when resuming a thread.
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

                let mut thread = match self
                    .load_thread_from_resume_source_or_send_internal(
                        thread_id,
                        codex_thread.as_ref(),
                        &response_history,
                        rollout_path.as_path(),
                        fallback_model_provider.as_str(),
                        persisted_resume_metadata.as_ref(),
                    )
                    .await
                {
                    Ok(thread) => thread,
                    Err(message) => {
                        self.send_internal_error(request_id, message).await;
                        return;
                    }
                };

                self.thread_watch_manager
                    .upsert_thread(thread.clone())
                    .await;

                let thread_status = self
                    .thread_watch_manager
                    .loaded_status_for_thread(&thread.id)
                    .await;

                set_thread_status_and_interrupt_stale_turns(
                    &mut thread,
                    thread_status,
                    /*has_live_in_progress_turn*/ false,
                );
                let permission_profile = thread_response_permission_profile(
                    &session_configured.sandbox_policy,
                    codex_thread.config_snapshot().await.permission_profile,
                );

                let response = ThreadResumeResponse {
                    thread,
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
                        ClientResponse::ThreadResume {
                            request_id: request_id.request_id.clone(),
                            response: response.clone(),
                        },
                    );
                }

                let connection_id = request_id.connection_id;
                let token_usage_thread = response.thread.clone();
                let token_usage_turn_id = latest_token_usage_turn_id_from_rollout_items(
                    &response_history.get_rollout_items(),
                    &token_usage_thread,
                );
                self.outgoing.send_response(request_id, response).await;
                // The client needs restored usage before it starts another turn.
                // Sending after the response preserves JSON-RPC request ordering while
                // still filling the status line before the next turn lifecycle begins.
                send_thread_token_usage_update_to_connection(
                    &self.outgoing,
                    connection_id,
                    thread_id,
                    &token_usage_thread,
                    codex_thread.as_ref(),
                    token_usage_turn_id,
                )
                .await;
            }
            Err(err) => {
                let error = JSONRPCErrorError {
                    code: INTERNAL_ERROR_CODE,
                    message: format!("error resuming thread: {err}"),
                    data: None,
                };
                self.outgoing.send_error(request_id, error).await;
            }
        }
    }

    async fn load_and_apply_persisted_resume_metadata(
        &self,
        thread_history: &InitialHistory,
        request_overrides: &mut Option<HashMap<String, serde_json::Value>>,
        typesafe_overrides: &mut ConfigOverrides,
    ) -> Option<ThreadMetadata> {
        let InitialHistory::Resumed(resumed_history) = thread_history else {
            return None;
        };
        let state_db_ctx = get_state_db(&self.config).await?;
        let persisted_metadata = state_db_ctx
            .get_thread(resumed_history.conversation_id)
            .await
            .ok()
            .flatten()?;
        merge_persisted_resume_metadata(request_overrides, typesafe_overrides, &persisted_metadata);
        Some(persisted_metadata)
    }

    async fn resume_running_thread(
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

    async fn resume_thread_from_history(
        &self,
        request_id: ConnectionRequestId,
        history: &[ResponseItem],
    ) -> Option<InitialHistory> {
        if history.is_empty() {
            self.send_invalid_request_error(request_id, "history must not be empty".to_string())
                .await;
            return None;
        }
        Some(InitialHistory::Forked(
            history
                .iter()
                .cloned()
                .map(RolloutItem::ResponseItem)
                .collect(),
        ))
    }

    async fn resume_thread_from_rollout(
        &self,
        request_id: ConnectionRequestId,
        thread_id: &str,
        path: Option<&PathBuf>,
    ) -> Option<InitialHistory> {
        let rollout_path = if let Some(path) = path {
            path.clone()
        } else {
            let existing_thread_id = match ThreadId::from_string(thread_id) {
                Ok(id) => id,
                Err(err) => {
                    let error = JSONRPCErrorError {
                        code: INVALID_REQUEST_ERROR_CODE,
                        message: format!("invalid thread id: {err}"),
                        data: None,
                    };
                    self.outgoing.send_error(request_id, error).await;
                    return None;
                }
            };

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
                    return None;
                }
                Err(err) => {
                    self.send_invalid_request_error(
                        request_id,
                        format!("failed to locate thread id {existing_thread_id}: {err}"),
                    )
                    .await;
                    return None;
                }
            }
        };

        match RolloutRecorder::get_rollout_history(&rollout_path).await {
            Ok(initial_history) => Some(initial_history),
            Err(err) => {
                self.send_invalid_request_error(
                    request_id,
                    format!("failed to load rollout `{}`: {err}", rollout_path.display()),
                )
                .await;
                None
            }
        }
    }

    async fn load_thread_from_resume_source_or_send_internal(
        &self,
        thread_id: ThreadId,
        live_thread: &CodexThread,
        thread_history: &InitialHistory,
        rollout_path: &Path,
        fallback_provider: &str,
        persisted_resume_metadata: Option<&ThreadMetadata>,
    ) -> std::result::Result<Thread, String> {
        let thread = match thread_history {
            InitialHistory::Resumed(resumed) => {
                load_thread_summary_for_rollout(
                    &self.config,
                    resumed.conversation_id,
                    resumed.rollout_path.as_path(),
                    fallback_provider,
                    persisted_resume_metadata,
                )
                .await
            }
            InitialHistory::Forked(items) => {
                let config_snapshot = live_thread.config_snapshot().await;
                let mut thread = build_thread_from_snapshot(
                    thread_id,
                    &config_snapshot,
                    Some(rollout_path.into()),
                );
                thread.preview = preview_from_rollout_items(items);
                Ok(thread)
            }
            InitialHistory::New | InitialHistory::Cleared => Err(format!(
                "failed to build resume response for thread {thread_id}: initial history missing"
            )),
        };
        let mut thread = thread?;
        thread.id = thread_id.to_string();
        thread.path = Some(rollout_path.to_path_buf());
        let history_items = thread_history.get_rollout_items();
        populate_thread_turns(
            &mut thread,
            ThreadTurnSource::HistoryItems(&history_items),
            /*active_turn*/ None,
        )
        .await?;
        self.attach_thread_name(thread_id, &mut thread).await;
        thread.epiphany_state = live_thread_epiphany_state(live_thread).await;
        Ok(thread)
    }

    pub(super) async fn attach_thread_name(&self, thread_id: ThreadId, thread: &mut Thread) {
        if let Some(title) = title_from_state_db(&self.config, thread_id).await {
            set_thread_name_from_title(thread, title);
        }
    }

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

fn merge_persisted_resume_metadata(
    request_overrides: &mut Option<HashMap<String, serde_json::Value>>,
    typesafe_overrides: &mut ConfigOverrides,
    persisted_metadata: &ThreadMetadata,
) {
    if has_model_resume_override(request_overrides.as_ref(), typesafe_overrides) {
        return;
    }

    typesafe_overrides.model = persisted_metadata.model.clone();

    if let Some(reasoning_effort) = persisted_metadata.reasoning_effort {
        request_overrides.get_or_insert_with(HashMap::new).insert(
            "model_reasoning_effort".to_string(),
            serde_json::Value::String(reasoning_effort.to_string()),
        );
    }
}

fn has_model_resume_override(
    request_overrides: Option<&HashMap<String, serde_json::Value>>,
    typesafe_overrides: &ConfigOverrides,
) -> bool {
    typesafe_overrides.model.is_some()
        || typesafe_overrides.model_provider.is_some()
        || request_overrides.is_some_and(|overrides| overrides.contains_key("model"))
        || request_overrides
            .is_some_and(|overrides| overrides.contains_key("model_reasoning_effort"))
}
