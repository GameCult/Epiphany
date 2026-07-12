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
        Ok(thread)
    }

    pub(super) async fn attach_thread_name(&self, thread_id: ThreadId, thread: &mut Thread) {
        if let Some(title) = title_from_state_db(&self.config, thread_id).await {
            set_thread_name_from_title(thread, title);
        }
    }
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
