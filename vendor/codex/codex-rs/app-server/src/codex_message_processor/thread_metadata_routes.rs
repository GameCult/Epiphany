use super::*;

impl CodexMessageProcessor {
    pub(super) async fn thread_set_name(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadSetNameParams,
    ) {
        let ThreadSetNameParams { thread_id, name } = params;
        let thread_id = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };
        let Some(name) = codex_core::util::normalize_thread_name(&name) else {
            self.send_invalid_request_error(
                request_id,
                "thread name must not be empty".to_string(),
            )
            .await;
            return;
        };

        if let Ok(thread) = self.thread_manager.get_thread(thread_id).await {
            if let Err(err) = self
                .submit_core_op(&request_id, thread.as_ref(), Op::SetThreadName { name })
                .await
            {
                self.send_internal_error(request_id, format!("failed to set thread name: {err}"))
                    .await;
                return;
            }

            self.outgoing
                .send_response(request_id, ThreadSetNameResponse {})
                .await;
            return;
        }

        if let Err(err) = self
            .thread_store
            .update_thread_metadata(StoreUpdateThreadMetadataParams {
                thread_id,
                patch: StoreThreadMetadataPatch {
                    name: Some(name.clone()),
                    ..Default::default()
                },
                include_archived: false,
            })
            .await
        {
            self.outgoing
                .send_error(request_id, thread_store_write_error("set thread name", err))
                .await;
            return;
        }

        self.outgoing
            .send_response(request_id, ThreadSetNameResponse {})
            .await;
        let notification = ThreadNameUpdatedNotification {
            thread_id: thread_id.to_string(),
            thread_name: Some(name),
        };
        self.outgoing
            .send_server_notification(ServerNotification::ThreadNameUpdated(notification))
            .await;
    }

    pub(super) async fn thread_memory_mode_set(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadMemoryModeSetParams,
    ) {
        let ThreadMemoryModeSetParams { thread_id, mode } = params;
        let thread_id = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        if let Ok(thread) = self.thread_manager.get_thread(thread_id).await {
            if thread.config_snapshot().await.ephemeral {
                self.send_invalid_request_error(
                    request_id,
                    format!("ephemeral thread does not support memory mode updates: {thread_id}"),
                )
                .await;
                return;
            }

            if let Err(err) = thread.set_thread_memory_mode(mode.to_core()).await {
                self.send_internal_error(
                    request_id,
                    format!("failed to set thread memory mode: {err}"),
                )
                .await;
                return;
            }

            self.outgoing
                .send_response(request_id, ThreadMemoryModeSetResponse {})
                .await;
            return;
        }

        if let Err(err) = self
            .thread_store
            .update_thread_metadata(StoreUpdateThreadMetadataParams {
                thread_id,
                patch: StoreThreadMetadataPatch {
                    memory_mode: Some(mode.to_core()),
                    ..Default::default()
                },
                include_archived: false,
            })
            .await
        {
            self.outgoing
                .send_error(
                    request_id,
                    thread_store_write_error("set thread memory mode", err),
                )
                .await;
            return;
        }

        self.outgoing
            .send_response(request_id, ThreadMemoryModeSetResponse {})
            .await;
    }

    pub(super) async fn memory_reset(&self, request_id: ConnectionRequestId, _params: Option<()>) {
        let state_db = match StateRuntime::init(
            self.config.sqlite_home.clone(),
            self.config.model_provider_id.clone(),
        )
        .await
        {
            Ok(state_db) => state_db,
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to open state db for memory reset: {err}"),
                )
                .await;
                return;
            }
        };

        if let Err(err) = state_db.clear_memory_data().await {
            self.send_internal_error(
                request_id,
                format!("failed to clear memory rows in state db: {err}"),
            )
            .await;
            return;
        }

        if let Err(err) = clear_memory_roots_contents(&self.config.codex_home).await {
            self.send_internal_error(
                request_id,
                format!(
                    "failed to clear memory directories under {}: {err}",
                    self.config.codex_home.display()
                ),
            )
            .await;
            return;
        }

        self.outgoing
            .send_response(request_id, MemoryResetResponse {})
            .await;
    }

    pub(super) async fn thread_metadata_update(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadMetadataUpdateParams,
    ) {
        let ThreadMetadataUpdateParams {
            thread_id,
            git_info,
        } = params;

        let thread_uuid = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        let Some(ThreadMetadataGitInfoUpdateParams {
            sha,
            branch,
            origin_url,
        }) = git_info
        else {
            self.send_invalid_request_error(
                request_id,
                "gitInfo must include at least one field".to_string(),
            )
            .await;
            return;
        };

        if sha.is_none() && branch.is_none() && origin_url.is_none() {
            self.send_invalid_request_error(
                request_id,
                "gitInfo must include at least one field".to_string(),
            )
            .await;
            return;
        }

        let loaded_thread = self.thread_manager.get_thread(thread_uuid).await.ok();
        let mut state_db_ctx = loaded_thread.as_ref().and_then(|thread| thread.state_db());
        if state_db_ctx.is_none() {
            state_db_ctx = open_state_db_for_direct_thread_lookup(&self.config).await;
        }
        let Some(state_db_ctx) = state_db_ctx else {
            self.send_internal_error(
                request_id,
                format!("sqlite state db unavailable for thread {thread_uuid}"),
            )
            .await;
            return;
        };

        if let Err(error) = self
            .ensure_thread_metadata_row_exists(thread_uuid, &state_db_ctx, loaded_thread.as_ref())
            .await
        {
            self.outgoing.send_error(request_id, error).await;
            return;
        }

        let git_sha = match sha {
            Some(Some(sha)) => {
                let sha = sha.trim().to_string();
                if sha.is_empty() {
                    self.send_invalid_request_error(
                        request_id,
                        "gitInfo.sha must not be empty".to_string(),
                    )
                    .await;
                    return;
                }
                Some(Some(sha))
            }
            Some(None) => Some(None),
            None => None,
        };
        let git_branch = match branch {
            Some(Some(branch)) => {
                let branch = branch.trim().to_string();
                if branch.is_empty() {
                    self.send_invalid_request_error(
                        request_id,
                        "gitInfo.branch must not be empty".to_string(),
                    )
                    .await;
                    return;
                }
                Some(Some(branch))
            }
            Some(None) => Some(None),
            None => None,
        };
        let git_origin_url = match origin_url {
            Some(Some(origin_url)) => {
                let origin_url = origin_url.trim().to_string();
                if origin_url.is_empty() {
                    self.send_invalid_request_error(
                        request_id,
                        "gitInfo.originUrl must not be empty".to_string(),
                    )
                    .await;
                    return;
                }
                Some(Some(origin_url))
            }
            Some(None) => Some(None),
            None => None,
        };

        let updated = match state_db_ctx
            .update_thread_git_info(
                thread_uuid,
                git_sha.as_ref().map(|value| value.as_deref()),
                git_branch.as_ref().map(|value| value.as_deref()),
                git_origin_url.as_ref().map(|value| value.as_deref()),
            )
            .await
        {
            Ok(updated) => updated,
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to update thread metadata for {thread_uuid}: {err}"),
                )
                .await;
                return;
            }
        };
        if !updated {
            self.send_internal_error(
                request_id,
                format!("thread metadata disappeared before update completed: {thread_uuid}"),
            )
            .await;
            return;
        }

        let Some(summary) =
            read_summary_from_state_db_context_by_thread_id(Some(&state_db_ctx), thread_uuid).await
        else {
            self.send_internal_error(
                request_id,
                format!("failed to reload updated thread metadata for {thread_uuid}"),
            )
            .await;
            return;
        };

        let mut thread = summary_to_thread(summary, &self.config.cwd);
        self.attach_thread_name(thread_uuid, &mut thread).await;
        thread.status = resolve_thread_status(
            self.thread_watch_manager
                .loaded_status_for_thread(&thread.id)
                .await,
            /*has_in_progress_turn*/ false,
        );

        self.outgoing
            .send_response(request_id, ThreadMetadataUpdateResponse { thread })
            .await;
    }

    pub(super) async fn ensure_thread_metadata_row_exists(
        &self,
        thread_uuid: ThreadId,
        state_db_ctx: &Arc<StateRuntime>,
        loaded_thread: Option<&Arc<CodexThread>>,
    ) -> Result<(), JSONRPCErrorError> {
        fn invalid_request(message: String) -> JSONRPCErrorError {
            JSONRPCErrorError {
                code: INVALID_REQUEST_ERROR_CODE,
                message,
                data: None,
            }
        }

        fn internal_error(message: String) -> JSONRPCErrorError {
            JSONRPCErrorError {
                code: INTERNAL_ERROR_CODE,
                message,
                data: None,
            }
        }

        match state_db_ctx.get_thread(thread_uuid).await {
            Ok(Some(_)) => return Ok(()),
            Ok(None) => {}
            Err(err) => {
                return Err(internal_error(format!(
                    "failed to load thread metadata for {thread_uuid}: {err}"
                )));
            }
        }

        if let Some(thread) = loaded_thread {
            let Some(rollout_path) = thread.rollout_path() else {
                return Err(invalid_request(format!(
                    "ephemeral thread does not support metadata updates: {thread_uuid}"
                )));
            };

            reconcile_rollout(
                Some(state_db_ctx),
                rollout_path.as_path(),
                self.config.model_provider_id.as_str(),
                /*builder*/ None,
                &[],
                /*archived_only*/ None,
                /*new_thread_memory_mode*/ None,
            )
            .await;

            match state_db_ctx.get_thread(thread_uuid).await {
                Ok(Some(_)) => return Ok(()),
                Ok(None) => {}
                Err(err) => {
                    return Err(internal_error(format!(
                        "failed to load reconciled thread metadata for {thread_uuid}: {err}"
                    )));
                }
            }

            let config_snapshot = thread.config_snapshot().await;
            let model_provider = config_snapshot.model_provider_id.clone();
            let mut builder = ThreadMetadataBuilder::new(
                thread_uuid,
                rollout_path,
                Utc::now(),
                config_snapshot.session_source.clone(),
            );
            builder.model_provider = Some(model_provider.clone());
            builder.cwd = config_snapshot.cwd.to_path_buf();
            builder.cli_version = Some(env!("CARGO_PKG_VERSION").to_string());
            builder.sandbox_policy = config_snapshot.sandbox_policy.clone();
            builder.approval_mode = config_snapshot.approval_policy;
            let metadata = builder.build(model_provider.as_str());
            if let Err(err) = state_db_ctx.insert_thread_if_absent(&metadata).await {
                return Err(internal_error(format!(
                    "failed to create thread metadata for {thread_uuid}: {err}"
                )));
            }
            return Ok(());
        }

        let rollout_path =
            match find_thread_path_by_id_str(&self.config.codex_home, &thread_uuid.to_string())
                .await
            {
                Ok(Some(path)) => path,
                Ok(None) => match find_archived_thread_path_by_id_str(
                    &self.config.codex_home,
                    &thread_uuid.to_string(),
                )
                .await
                {
                    Ok(Some(path)) => path,
                    Ok(None) => {
                        return Err(invalid_request(format!("thread not found: {thread_uuid}")));
                    }
                    Err(err) => {
                        return Err(internal_error(format!(
                            "failed to locate archived thread id {thread_uuid}: {err}"
                        )));
                    }
                },
                Err(err) => {
                    return Err(internal_error(format!(
                        "failed to locate thread id {thread_uuid}: {err}"
                    )));
                }
            };

        reconcile_rollout(
            Some(state_db_ctx),
            rollout_path.as_path(),
            self.config.model_provider_id.as_str(),
            /*builder*/ None,
            &[],
            /*archived_only*/ None,
            /*new_thread_memory_mode*/ None,
        )
        .await;

        match state_db_ctx.get_thread(thread_uuid).await {
            Ok(Some(_)) => Ok(()),
            Ok(None) => Err(internal_error(format!(
                "failed to create thread metadata from rollout for {thread_uuid}"
            ))),
            Err(err) => Err(internal_error(format!(
                "failed to load reconciled thread metadata for {thread_uuid}: {err}"
            ))),
        }
    }
}
