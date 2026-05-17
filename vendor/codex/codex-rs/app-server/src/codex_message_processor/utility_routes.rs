use super::*;

impl CodexMessageProcessor {
    pub(super) async fn git_diff_to_origin(&self, request_id: ConnectionRequestId, cwd: PathBuf) {
        let diff = git_diff_to_remote(&cwd).await;
        match diff {
            Some(value) => {
                let response = GitDiffToRemoteResponse {
                    sha: value.sha,
                    diff: value.diff,
                };
                self.outgoing.send_response(request_id, response).await;
            }
            None => {
                let error = JSONRPCErrorError {
                    code: INVALID_REQUEST_ERROR_CODE,
                    message: format!("failed to compute git diff to remote for cwd: {cwd:?}"),
                    data: None,
                };
                self.outgoing.send_error(request_id, error).await;
            }
        }
    }

    pub(super) async fn fuzzy_file_search(
        &self,
        request_id: ConnectionRequestId,
        params: FuzzyFileSearchParams,
    ) {
        let FuzzyFileSearchParams {
            query,
            roots,
            cancellation_token,
        } = params;

        let cancel_flag = match cancellation_token.clone() {
            Some(token) => {
                let mut pending_fuzzy_searches = self.pending_fuzzy_searches.lock().await;
                // if a cancellation_token is provided and a pending_request exists for
                // that token, cancel it
                if let Some(existing) = pending_fuzzy_searches.get(&token) {
                    existing.store(true, Ordering::Relaxed);
                }
                let flag = Arc::new(AtomicBool::new(false));
                pending_fuzzy_searches.insert(token.clone(), flag.clone());
                flag
            }
            None => Arc::new(AtomicBool::new(false)),
        };

        let results = match query.as_str() {
            "" => vec![],
            _ => run_fuzzy_file_search(query, roots, cancel_flag.clone()).await,
        };

        if let Some(token) = cancellation_token {
            let mut pending_fuzzy_searches = self.pending_fuzzy_searches.lock().await;
            if let Some(current_flag) = pending_fuzzy_searches.get(&token)
                && Arc::ptr_eq(current_flag, &cancel_flag)
            {
                pending_fuzzy_searches.remove(&token);
            }
        }

        let response = FuzzyFileSearchResponse { files: results };
        self.outgoing.send_response(request_id, response).await;
    }

    pub(super) async fn fuzzy_file_search_session_start(
        &self,
        request_id: ConnectionRequestId,
        params: FuzzyFileSearchSessionStartParams,
    ) {
        let FuzzyFileSearchSessionStartParams { session_id, roots } = params;
        if session_id.is_empty() {
            let error = JSONRPCErrorError {
                code: INVALID_REQUEST_ERROR_CODE,
                message: "sessionId must not be empty".to_string(),
                data: None,
            };
            self.outgoing.send_error(request_id, error).await;
            return;
        }

        let session =
            start_fuzzy_file_search_session(session_id.clone(), roots, self.outgoing.clone());
        match session {
            Ok(session) => {
                self.fuzzy_search_sessions
                    .lock()
                    .await
                    .insert(session_id, session);
                self.outgoing
                    .send_response(request_id, FuzzyFileSearchSessionStartResponse {})
                    .await;
            }
            Err(err) => {
                let error = JSONRPCErrorError {
                    code: INTERNAL_ERROR_CODE,
                    message: format!("failed to start fuzzy file search session: {err}"),
                    data: None,
                };
                self.outgoing.send_error(request_id, error).await;
            }
        }
    }

    pub(super) async fn fuzzy_file_search_session_update(
        &self,
        request_id: ConnectionRequestId,
        params: FuzzyFileSearchSessionUpdateParams,
    ) {
        let FuzzyFileSearchSessionUpdateParams { session_id, query } = params;
        let found = {
            let sessions = self.fuzzy_search_sessions.lock().await;
            if let Some(session) = sessions.get(&session_id) {
                session.update_query(query);
                true
            } else {
                false
            }
        };
        if !found {
            let error = JSONRPCErrorError {
                code: INVALID_REQUEST_ERROR_CODE,
                message: format!("fuzzy file search session not found: {session_id}"),
                data: None,
            };
            self.outgoing.send_error(request_id, error).await;
            return;
        }

        self.outgoing
            .send_response(request_id, FuzzyFileSearchSessionUpdateResponse {})
            .await;
    }

    pub(super) async fn fuzzy_file_search_session_stop(
        &self,
        request_id: ConnectionRequestId,
        params: FuzzyFileSearchSessionStopParams,
    ) {
        let FuzzyFileSearchSessionStopParams { session_id } = params;
        {
            let mut sessions = self.fuzzy_search_sessions.lock().await;
            sessions.remove(&session_id);
        }

        self.outgoing
            .send_response(request_id, FuzzyFileSearchSessionStopResponse {})
            .await;
    }

    pub(super) async fn upload_feedback(
        &self,
        request_id: ConnectionRequestId,
        params: FeedbackUploadParams,
    ) {
        if !self.config.feedback_enabled {
            let error = JSONRPCErrorError {
                code: INVALID_REQUEST_ERROR_CODE,
                message: "sending feedback is disabled by configuration".to_string(),
                data: None,
            };
            self.outgoing.send_error(request_id, error).await;
            return;
        }

        let FeedbackUploadParams {
            classification,
            reason,
            thread_id,
            include_logs,
            extra_log_files,
            tags,
        } = params;

        let conversation_id = match thread_id.as_deref() {
            Some(thread_id) => match ThreadId::from_string(thread_id) {
                Ok(conversation_id) => Some(conversation_id),
                Err(err) => {
                    let error = JSONRPCErrorError {
                        code: INVALID_REQUEST_ERROR_CODE,
                        message: format!("invalid thread id: {err}"),
                        data: None,
                    };
                    self.outgoing.send_error(request_id, error).await;
                    return;
                }
            },
            None => None,
        };

        if let Some(chatgpt_user_id) = self
            .auth_manager
            .auth_cached()
            .and_then(|auth| auth.get_chatgpt_user_id())
        {
            tracing::info!(target: "feedback_tags", chatgpt_user_id);
        }
        let snapshot = self.feedback.snapshot(conversation_id);
        let thread_id = snapshot.thread_id.clone();
        let (feedback_thread_ids, sqlite_feedback_logs, state_db_ctx) = if include_logs {
            if let Some(log_db) = self.log_db.as_ref() {
                log_db.flush().await;
            }
            let state_db_ctx = get_state_db(&self.config).await;
            let feedback_thread_ids = match conversation_id {
                Some(conversation_id) => match self
                    .thread_manager
                    .list_agent_subtree_thread_ids(conversation_id)
                    .await
                {
                    Ok(thread_ids) => thread_ids,
                    Err(err) => {
                        warn!(
                            "failed to list feedback subtree for thread_id={conversation_id}: {err}"
                        );
                        let mut thread_ids = vec![conversation_id];
                        if let Some(state_db_ctx) = state_db_ctx.as_ref() {
                            for status in [
                                codex_state::DirectionalThreadSpawnEdgeStatus::Open,
                                codex_state::DirectionalThreadSpawnEdgeStatus::Closed,
                            ] {
                                match state_db_ctx
                                    .list_thread_spawn_descendants_with_status(
                                        conversation_id,
                                        status,
                                    )
                                    .await
                                {
                                    Ok(descendant_ids) => thread_ids.extend(descendant_ids),
                                    Err(err) => warn!(
                                        "failed to list persisted feedback subtree for thread_id={conversation_id}: {err}"
                                    ),
                                }
                            }
                        }
                        thread_ids
                    }
                },
                None => Vec::new(),
            };
            let sqlite_feedback_logs = if let Some(state_db_ctx) = state_db_ctx.as_ref()
                && !feedback_thread_ids.is_empty()
            {
                let thread_id_texts = feedback_thread_ids
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>();
                let thread_id_refs = thread_id_texts
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>();
                match state_db_ctx
                    .query_feedback_logs_for_threads(&thread_id_refs)
                    .await
                {
                    Ok(logs) if logs.is_empty() => None,
                    Ok(logs) => Some(logs),
                    Err(err) => {
                        let thread_ids = thread_id_texts.join(", ");
                        warn!(
                            "failed to query feedback logs from sqlite for thread_ids=[{thread_ids}]: {err}"
                        );
                        None
                    }
                }
            } else {
                None
            };
            (feedback_thread_ids, sqlite_feedback_logs, state_db_ctx)
        } else {
            (Vec::new(), None, None)
        };

        let mut attachment_paths = Vec::new();
        let mut seen_attachment_paths = HashSet::new();
        if include_logs {
            for feedback_thread_id in &feedback_thread_ids {
                let Some(rollout_path) = self
                    .resolve_rollout_path(*feedback_thread_id, state_db_ctx.as_ref())
                    .await
                else {
                    continue;
                };
                if seen_attachment_paths.insert(rollout_path.clone()) {
                    attachment_paths.push(rollout_path);
                }
            }
        }
        if let Some(extra_log_files) = extra_log_files {
            for extra_log_file in extra_log_files {
                if seen_attachment_paths.insert(extra_log_file.clone()) {
                    attachment_paths.push(extra_log_file);
                }
            }
        }

        let session_source = self.thread_manager.session_source();

        let upload_result = tokio::task::spawn_blocking(move || {
            snapshot.upload_feedback(FeedbackUploadOptions {
                classification: &classification,
                reason: reason.as_deref(),
                tags: tags.as_ref(),
                include_logs,
                extra_attachment_paths: &attachment_paths,
                session_source: Some(session_source),
                logs_override: sqlite_feedback_logs,
            })
        })
        .await;

        let upload_result = match upload_result {
            Ok(result) => result,
            Err(join_err) => {
                let error = JSONRPCErrorError {
                    code: INTERNAL_ERROR_CODE,
                    message: format!("failed to upload feedback: {join_err}"),
                    data: None,
                };
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        match upload_result {
            Ok(()) => {
                let response = FeedbackUploadResponse { thread_id };
                self.outgoing.send_response(request_id, response).await;
            }
            Err(err) => {
                let error = JSONRPCErrorError {
                    code: INTERNAL_ERROR_CODE,
                    message: format!("failed to upload feedback: {err}"),
                    data: None,
                };
                self.outgoing.send_error(request_id, error).await;
            }
        }
    }

    pub(super) async fn windows_sandbox_setup_start(
        &self,
        request_id: ConnectionRequestId,
        params: WindowsSandboxSetupStartParams,
    ) {
        self.outgoing
            .send_response(
                request_id.clone(),
                WindowsSandboxSetupStartResponse { started: true },
            )
            .await;

        let mode = match params.mode {
            WindowsSandboxSetupMode::Elevated => CoreWindowsSandboxSetupMode::Elevated,
            WindowsSandboxSetupMode::Unelevated => CoreWindowsSandboxSetupMode::Unelevated,
        };
        let config = Arc::clone(&self.config);
        let config_manager = self.config_manager.clone();
        let command_cwd = params
            .cwd
            .map(PathBuf::from)
            .unwrap_or_else(|| config.cwd.to_path_buf());
        let outgoing = Arc::clone(&self.outgoing);
        let connection_id = request_id.connection_id;

        tokio::spawn(async move {
            let derived_config = config_manager
                .load_for_cwd(
                    /*request_overrides*/ None,
                    ConfigOverrides {
                        cwd: Some(command_cwd.clone()),
                        ..Default::default()
                    },
                    Some(command_cwd.clone()),
                )
                .await;
            let setup_result = match derived_config {
                Ok(config) => {
                    let setup_request = WindowsSandboxSetupRequest {
                        mode,
                        policy: config.permissions.sandbox_policy.get().clone(),
                        policy_cwd: config.cwd.to_path_buf(),
                        command_cwd,
                        env_map: std::env::vars().collect(),
                        codex_home: config.codex_home.to_path_buf(),
                        active_profile: config.active_profile.clone(),
                    };
                    codex_core::windows_sandbox::run_windows_sandbox_setup(setup_request).await
                }
                Err(err) => Err(err.into()),
            };
            let notification = WindowsSandboxSetupCompletedNotification {
                mode: match mode {
                    CoreWindowsSandboxSetupMode::Elevated => WindowsSandboxSetupMode::Elevated,
                    CoreWindowsSandboxSetupMode::Unelevated => WindowsSandboxSetupMode::Unelevated,
                },
                success: setup_result.is_ok(),
                error: setup_result.err().map(|err| err.to_string()),
            };
            outgoing
                .send_server_notification_to_connections(
                    &[connection_id],
                    ServerNotification::WindowsSandboxSetupCompleted(notification),
                )
                .await;
        });
    }
}
