use super::*;

impl CodexMessageProcessor {
    pub(super) async fn thread_archive(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadArchiveParams,
    ) {
        let thread_id = match ThreadId::from_string(&params.thread_id) {
            Ok(id) => id,
            Err(err) => {
                let error = JSONRPCErrorError {
                    code: INVALID_REQUEST_ERROR_CODE,
                    message: format!("invalid thread id: {err}"),
                    data: None,
                };
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        let mut thread_ids = vec![thread_id];
        if let Some(state_db_ctx) = get_state_db(&self.config).await {
            let descendants = match state_db_ctx.list_thread_spawn_descendants(thread_id).await {
                Ok(descendants) => descendants,
                Err(err) => {
                    self.outgoing
                        .send_error(
                            request_id,
                            JSONRPCErrorError {
                                code: INTERNAL_ERROR_CODE,
                                message: format!(
                                    "failed to list spawned descendants for thread id {thread_id}: {err}"
                                ),
                                data: None,
                            },
                        )
                        .await;
                    return;
                }
            };
            let mut seen = HashSet::from([thread_id]);
            for descendant_id in descendants {
                if seen.insert(descendant_id) {
                    thread_ids.push(descendant_id);
                }
            }
        }

        let mut archive_thread_ids = Vec::new();
        match self
            .thread_store
            .read_thread(StoreReadThreadParams {
                thread_id,
                include_archived: false,
                include_history: false,
            })
            .await
        {
            Ok(thread) => {
                if thread.archived_at.is_none() {
                    archive_thread_ids.push(thread_id);
                }
            }
            Err(err) => {
                self.outgoing
                    .send_error(request_id, thread_store_archive_error("archive", err))
                    .await;
                return;
            }
        }
        for descendant_thread_id in thread_ids.into_iter().skip(1) {
            match self
                .thread_store
                .read_thread(StoreReadThreadParams {
                    thread_id: descendant_thread_id,
                    include_archived: true,
                    include_history: false,
                })
                .await
            {
                Ok(thread) => {
                    if thread.archived_at.is_none() {
                        archive_thread_ids.push(descendant_thread_id);
                    }
                }
                Err(err) => {
                    warn!(
                        "failed to read spawned descendant thread {descendant_thread_id} while archiving {thread_id}: {err}"
                    );
                }
            }
        }

        let mut archived_thread_ids = Vec::new();
        let Some((parent_thread_id, descendant_thread_ids)) = archive_thread_ids.split_first()
        else {
            self.outgoing
                .send_response(request_id, ThreadArchiveResponse {})
                .await;
            return;
        };

        self.prepare_thread_for_archive(*parent_thread_id).await;
        match self
            .thread_store
            .archive_thread(StoreArchiveThreadParams {
                thread_id: *parent_thread_id,
            })
            .await
        {
            Ok(()) => {
                archived_thread_ids.push(parent_thread_id.to_string());
            }
            Err(err) => {
                self.outgoing
                    .send_error(request_id, thread_store_archive_error("archive", err))
                    .await;
                return;
            }
        }

        for descendant_thread_id in descendant_thread_ids.iter().rev().copied() {
            self.prepare_thread_for_archive(descendant_thread_id).await;
            match self
                .thread_store
                .archive_thread(StoreArchiveThreadParams {
                    thread_id: descendant_thread_id,
                })
                .await
            {
                Ok(()) => {
                    archived_thread_ids.push(descendant_thread_id.to_string());
                }
                Err(err) => {
                    warn!(
                        "failed to archive spawned descendant thread {descendant_thread_id} while archiving {thread_id}: {err}"
                    );
                }
            }
        }

        self.outgoing
            .send_response(request_id, ThreadArchiveResponse {})
            .await;
        for thread_id in archived_thread_ids {
            let notification = ThreadArchivedNotification { thread_id };
            self.outgoing
                .send_server_notification(ServerNotification::ThreadArchived(notification))
                .await;
        }
    }

    pub(super) async fn thread_increment_elicitation(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadIncrementElicitationParams,
    ) {
        let (_, thread) = match self.load_thread(&params.thread_id).await {
            Ok(value) => value,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        match thread.increment_out_of_band_elicitation_count().await {
            Ok(count) => {
                self.outgoing
                    .send_response(
                        request_id,
                        ThreadIncrementElicitationResponse {
                            count,
                            paused: count > 0,
                        },
                    )
                    .await;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to increment out-of-band elicitation counter: {err}"),
                )
                .await;
            }
        }
    }

    pub(super) async fn thread_decrement_elicitation(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadDecrementElicitationParams,
    ) {
        let (_, thread) = match self.load_thread(&params.thread_id).await {
            Ok(value) => value,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        match thread.decrement_out_of_band_elicitation_count().await {
            Ok(count) => {
                self.outgoing
                    .send_response(
                        request_id,
                        ThreadDecrementElicitationResponse {
                            count,
                            paused: count > 0,
                        },
                    )
                    .await;
            }
            Err(CodexErr::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to decrement out-of-band elicitation counter: {err}"),
                )
                .await;
            }
        }
    }

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

    pub(super) async fn thread_unarchive(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadUnarchiveParams,
    ) {
        let thread_id = match ThreadId::from_string(&params.thread_id) {
            Ok(id) => id,
            Err(err) => {
                let error = JSONRPCErrorError {
                    code: INVALID_REQUEST_ERROR_CODE,
                    message: format!("invalid thread id: {err}"),
                    data: None,
                };
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        let fallback_provider = self.config.model_provider_id.clone();
        let result = self
            .thread_store
            .unarchive_thread(StoreArchiveThreadParams { thread_id })
            .await
            .map_err(|err| thread_store_archive_error("unarchive", err))
            .and_then(|stored_thread| {
                summary_from_stored_thread(stored_thread, fallback_provider.as_str())
                    .map(|summary| summary_to_thread(summary, &self.config.cwd))
                    .ok_or_else(|| JSONRPCErrorError {
                        code: INTERNAL_ERROR_CODE,
                        message: format!("failed to read unarchived thread {thread_id}"),
                        data: None,
                    })
            });

        match result {
            Ok(mut thread) => {
                thread.status = resolve_thread_status(
                    self.thread_watch_manager
                        .loaded_status_for_thread(&thread.id)
                        .await,
                    /*has_in_progress_turn*/ false,
                );
                self.attach_thread_name(thread_id, &mut thread).await;
                if let Some(rollout_path) = thread.path.as_deref() {
                    match load_epiphany_state_from_rollout_path(rollout_path).await {
                        Ok(epiphany_state) => {
                            thread.epiphany_state = epiphany_state;
                        }
                        Err(message) => {
                            self.send_internal_error(request_id, message).await;
                            return;
                        }
                    }
                }
                let thread_id = thread.id.clone();
                let response = ThreadUnarchiveResponse { thread };
                self.outgoing.send_response(request_id, response).await;
                let notification = ThreadUnarchivedNotification { thread_id };
                self.outgoing
                    .send_server_notification(ServerNotification::ThreadUnarchived(notification))
                    .await;
            }
            Err(err) => {
                self.outgoing.send_error(request_id, err).await;
            }
        }
    }

    pub(super) async fn thread_rollback(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadRollbackParams,
    ) {
        let ThreadRollbackParams {
            thread_id,
            num_turns,
        } = params;

        if num_turns == 0 {
            self.send_invalid_request_error(request_id, "numTurns must be >= 1".to_string())
                .await;
            return;
        }

        let (thread_id, thread) = match self.load_thread(&thread_id).await {
            Ok(v) => v,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        let request = request_id.clone();

        let rollback_already_in_progress = {
            let thread_state = self.thread_state_manager.thread_state(thread_id).await;
            let mut thread_state = thread_state.lock().await;
            if thread_state.pending_rollbacks.is_some() {
                true
            } else {
                thread_state.pending_rollbacks = Some(request.clone());
                false
            }
        };
        if rollback_already_in_progress {
            self.send_invalid_request_error(
                request.clone(),
                "rollback already in progress for this thread".to_string(),
            )
            .await;
            return;
        }

        if let Err(err) = self
            .submit_core_op(
                &request_id,
                thread.as_ref(),
                Op::ThreadRollback { num_turns },
            )
            .await
        {
            // No ThreadRollback event will arrive if an error occurs.
            // Clean up and reply immediately.
            let thread_state = self.thread_state_manager.thread_state(thread_id).await;
            thread_state.lock().await.pending_rollbacks = None;

            self.send_internal_error(request, format!("failed to start rollback: {err}"))
                .await;
        }
    }

    pub(super) async fn thread_compact_start(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadCompactStartParams,
    ) {
        let ThreadCompactStartParams { thread_id } = params;

        let (_, thread) = match self.load_thread(&thread_id).await {
            Ok(v) => v,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        match self
            .submit_core_op(&request_id, thread.as_ref(), Op::Compact)
            .await
        {
            Ok(_) => {
                self.outgoing
                    .send_response(request_id, ThreadCompactStartResponse {})
                    .await;
            }
            Err(err) => {
                self.send_internal_error(request_id, format!("failed to start compaction: {err}"))
                    .await;
            }
        }
    }

    pub(super) async fn thread_background_terminals_clean(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadBackgroundTerminalsCleanParams,
    ) {
        let ThreadBackgroundTerminalsCleanParams { thread_id } = params;

        let (_, thread) = match self.load_thread(&thread_id).await {
            Ok(v) => v,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        match self
            .submit_core_op(&request_id, thread.as_ref(), Op::CleanBackgroundTerminals)
            .await
        {
            Ok(_) => {
                self.outgoing
                    .send_response(request_id, ThreadBackgroundTerminalsCleanResponse {})
                    .await;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to clean background terminals: {err}"),
                )
                .await;
            }
        }
    }

    pub(super) async fn thread_shell_command(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadShellCommandParams,
    ) {
        let ThreadShellCommandParams { thread_id, command } = params;
        let command = command.trim().to_string();
        if command.is_empty() {
            self.outgoing
                .send_error(
                    request_id,
                    JSONRPCErrorError {
                        code: INVALID_REQUEST_ERROR_CODE,
                        message: "command must not be empty".to_string(),
                        data: None,
                    },
                )
                .await;
            return;
        }

        let (_, thread) = match self.load_thread(&thread_id).await {
            Ok(v) => v,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        match self
            .submit_core_op(
                &request_id,
                thread.as_ref(),
                Op::RunUserShellCommand { command },
            )
            .await
        {
            Ok(_) => {
                self.outgoing
                    .send_response(request_id, ThreadShellCommandResponse {})
                    .await;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to start shell command: {err}"),
                )
                .await;
            }
        }
    }

    pub(super) async fn thread_approve_guardian_denied_action(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadApproveGuardianDeniedActionParams,
    ) {
        let ThreadApproveGuardianDeniedActionParams { thread_id, event } = params;
        let event = match serde_json::from_value(event) {
            Ok(event) => event,
            Err(err) => {
                self.outgoing
                    .send_error(
                        request_id,
                        JSONRPCErrorError {
                            code: INVALID_REQUEST_ERROR_CODE,
                            message: format!("invalid Guardian denial event: {err}"),
                            data: None,
                        },
                    )
                    .await;
                return;
            }
        };
        let (_, thread) = match self.load_thread(&thread_id).await {
            Ok(v) => v,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        match self
            .submit_core_op(
                &request_id,
                thread.as_ref(),
                Op::ApproveGuardianDeniedAction { event },
            )
            .await
        {
            Ok(_) => {
                self.outgoing
                    .send_response(request_id, ThreadApproveGuardianDeniedActionResponse {})
                    .await;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to approve Guardian denial: {err}"),
                )
                .await;
            }
        }
    }
}
