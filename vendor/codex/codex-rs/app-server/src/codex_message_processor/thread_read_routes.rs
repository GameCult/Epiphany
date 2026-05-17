use super::*;

impl CodexMessageProcessor {
    pub(super) async fn thread_list(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadListParams,
    ) {
        let ThreadListParams {
            cursor,
            limit,
            sort_key,
            sort_direction,
            model_providers,
            source_kinds,
            archived,
            cwd,
            use_state_db_only,
            search_term,
        } = params;
        let cwd_filters = match normalize_thread_list_cwd_filters(cwd) {
            Ok(cwd_filters) => cwd_filters,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        let requested_page_size = limit
            .map(|value| value as usize)
            .unwrap_or(THREAD_LIST_DEFAULT_LIMIT)
            .clamp(1, THREAD_LIST_MAX_LIMIT);
        let store_sort_key = match sort_key.unwrap_or(ThreadSortKey::CreatedAt) {
            ThreadSortKey::CreatedAt => StoreThreadSortKey::CreatedAt,
            ThreadSortKey::UpdatedAt => StoreThreadSortKey::UpdatedAt,
        };
        let sort_direction = sort_direction.unwrap_or(SortDirection::Desc);
        let list_result = self
            .list_threads_common(
                requested_page_size,
                cursor,
                store_sort_key,
                sort_direction,
                ThreadListFilters {
                    model_providers,
                    source_kinds,
                    archived: archived.unwrap_or(false),
                    cwd_filters,
                    search_term,
                    use_state_db_only,
                },
            )
            .await;
        let (summaries, next_cursor) = match list_result {
            Ok(r) => r,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };
        let backwards_cursor = summaries.first().and_then(|summary| {
            thread_backwards_cursor_for_sort_key(summary, store_sort_key, sort_direction)
        });
        let mut threads = Vec::with_capacity(summaries.len());
        let mut thread_ids = HashSet::with_capacity(summaries.len());
        let mut status_ids = Vec::with_capacity(summaries.len());

        for summary in summaries {
            let conversation_id = summary.conversation_id;
            thread_ids.insert(conversation_id);

            let thread = summary_to_thread(summary, &self.config.cwd);
            status_ids.push(thread.id.clone());
            threads.push((conversation_id, thread));
        }

        let names = thread_titles_by_ids(&self.config, &thread_ids).await;

        let statuses = self
            .thread_watch_manager
            .loaded_statuses_for_threads(status_ids)
            .await;

        let data: Vec<_> = threads
            .into_iter()
            .map(|(conversation_id, mut thread)| {
                if let Some(title) = names.get(&conversation_id).cloned() {
                    set_thread_name_from_title(&mut thread, title);
                }
                if let Some(status) = statuses.get(&thread.id) {
                    thread.status = status.clone();
                }
                thread
            })
            .collect();
        let response = ThreadListResponse {
            data,
            next_cursor,
            backwards_cursor,
        };
        self.outgoing.send_response(request_id, response).await;
    }

    pub(super) async fn thread_loaded_list(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadLoadedListParams,
    ) {
        let ThreadLoadedListParams { cursor, limit } = params;
        let mut data = self
            .thread_manager
            .list_thread_ids()
            .await
            .into_iter()
            .map(|thread_id| thread_id.to_string())
            .collect::<Vec<_>>();

        if data.is_empty() {
            let response = ThreadLoadedListResponse {
                data,
                next_cursor: None,
            };
            self.outgoing.send_response(request_id, response).await;
            return;
        }

        data.sort();
        let total = data.len();
        let start = match cursor {
            Some(cursor) => {
                let cursor = match ThreadId::from_string(&cursor) {
                    Ok(id) => id.to_string(),
                    Err(_) => {
                        let error = JSONRPCErrorError {
                            code: INVALID_REQUEST_ERROR_CODE,
                            message: format!("invalid cursor: {cursor}"),
                            data: None,
                        };
                        self.outgoing.send_error(request_id, error).await;
                        return;
                    }
                };
                match data.binary_search(&cursor) {
                    Ok(idx) => idx + 1,
                    Err(idx) => idx,
                }
            }
            None => 0,
        };

        let effective_limit = limit.unwrap_or(total as u32).max(1) as usize;
        let end = start.saturating_add(effective_limit).min(total);
        let page = data[start..end].to_vec();
        let next_cursor = page.last().filter(|_| end < total).cloned();

        let response = ThreadLoadedListResponse {
            data: page,
            next_cursor,
        };
        self.outgoing.send_response(request_id, response).await;
    }

    pub(super) async fn thread_read(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadReadParams,
    ) {
        let ThreadReadParams {
            thread_id,
            include_turns,
        } = params;

        let thread_uuid = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        let thread = match self.read_thread_view(thread_uuid, include_turns).await {
            Ok(thread) => thread,
            Err(ThreadReadViewError::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
            Err(ThreadReadViewError::Internal(message)) => {
                self.send_internal_error(request_id, message).await;
                return;
            }
        };
        let response = ThreadReadResponse { thread };
        self.outgoing.send_response(request_id, response).await;
    }

    /// Builds the API view for `thread/read` from persisted metadata plus optional live state.
    pub(super) async fn read_thread_view(
        &self,
        thread_id: ThreadId,
        include_turns: bool,
    ) -> Result<Thread, ThreadReadViewError> {
        let loaded_thread = self.load_live_thread_for_read(thread_id).await;
        let mut thread = if let Some(thread) = self
            .load_persisted_thread_for_read(thread_id, include_turns)
            .await?
        {
            thread
        } else if let Some(thread) = self
            .load_live_thread_view(thread_id, include_turns, loaded_thread.as_ref())
            .await?
        {
            thread
        } else {
            return Err(ThreadReadViewError::InvalidRequest(format!(
                "thread not loaded: {thread_id}"
            )));
        };

        self.apply_thread_read_epiphany_state(&mut thread, loaded_thread.as_ref())
            .await?;

        let has_live_in_progress_turn = if let Some(loaded_thread) = loaded_thread.as_ref() {
            matches!(loaded_thread.agent_status().await, AgentStatus::Running)
        } else {
            false
        };

        let thread_status = self
            .thread_watch_manager
            .loaded_status_for_thread(&thread.id)
            .await;

        set_thread_status_and_interrupt_stale_turns(
            &mut thread,
            thread_status,
            has_live_in_progress_turn,
        );
        Ok(thread)
    }

    async fn load_live_thread_for_read(&self, thread_id: ThreadId) -> Option<Arc<CodexThread>> {
        self.thread_manager.get_thread(thread_id).await.ok()
    }

    async fn load_persisted_thread_for_read(
        &self,
        thread_id: ThreadId,
        include_turns: bool,
    ) -> Result<Option<Thread>, ThreadReadViewError> {
        let fallback_provider = self.config.model_provider_id.as_str();
        match self
            .thread_store
            .read_thread(StoreReadThreadParams {
                thread_id,
                include_archived: true,
                include_history: include_turns,
            })
            .await
        {
            Ok(stored_thread) => {
                let (mut thread, history) =
                    thread_from_stored_thread(stored_thread, fallback_provider, &self.config.cwd);
                if include_turns && let Some(history) = history {
                    thread.turns = build_turns_from_rollout_items(&history.items);
                }
                Ok(Some(thread))
            }
            Err(ThreadStoreError::InvalidRequest { message })
                if message == format!("no rollout found for thread id {thread_id}") =>
            {
                Ok(None)
            }
            Err(ThreadStoreError::ThreadNotFound {
                thread_id: missing_thread_id,
            }) if missing_thread_id == thread_id => Ok(None),
            Err(ThreadStoreError::InvalidRequest { message }) => {
                Err(ThreadReadViewError::InvalidRequest(message))
            }
            Err(err) => Err(ThreadReadViewError::Internal(format!(
                "failed to read thread: {err}"
            ))),
        }
    }

    async fn load_live_thread_view(
        &self,
        thread_id: ThreadId,
        include_turns: bool,
        loaded_thread: Option<&Arc<CodexThread>>,
    ) -> Result<Option<Thread>, ThreadReadViewError> {
        let Some(thread) = loaded_thread else {
            return Ok(None);
        };
        let config_snapshot = thread.config_snapshot().await;
        let loaded_rollout_path = thread.rollout_path();
        if include_turns && loaded_rollout_path.is_none() {
            return Err(ThreadReadViewError::InvalidRequest(
                "ephemeral threads do not support includeTurns".to_string(),
            ));
        }
        let mut thread =
            build_thread_from_snapshot(thread_id, &config_snapshot, loaded_rollout_path.clone());
        self.apply_thread_read_rollout_fields(
            thread_id,
            &mut thread,
            loaded_rollout_path.as_deref(),
            include_turns,
        )
        .await?;
        Ok(Some(thread))
    }

    async fn apply_thread_read_epiphany_state(
        &self,
        thread: &mut Thread,
        loaded_thread: Option<&Arc<CodexThread>>,
    ) -> Result<(), ThreadReadViewError> {
        if let Some(loaded_thread) = loaded_thread {
            thread.epiphany_state = live_thread_epiphany_state(loaded_thread).await;
            return Ok(());
        }

        let Some(rollout_path) = thread.path.as_deref() else {
            thread.epiphany_state = None;
            return Ok(());
        };

        thread.epiphany_state = load_epiphany_state_from_rollout_path(rollout_path)
            .await
            .map_err(ThreadReadViewError::Internal)?;
        Ok(())
    }

    async fn apply_thread_read_rollout_fields(
        &self,
        thread_id: ThreadId,
        thread: &mut Thread,
        rollout_path: Option<&Path>,
        include_turns: bool,
    ) -> Result<(), ThreadReadViewError> {
        if thread.forked_from_id.is_none()
            && let Some(rollout_path) = rollout_path
        {
            thread.forked_from_id = forked_from_id_from_rollout(rollout_path).await;
        }
        self.attach_thread_name(thread_id, thread).await;

        if include_turns && let Some(rollout_path) = rollout_path {
            match read_rollout_items_from_rollout(rollout_path).await {
                Ok(items) => {
                    thread.turns = build_turns_from_rollout_items(&items);
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    return Err(ThreadReadViewError::InvalidRequest(format!(
                        "thread {thread_id} is not materialized yet; includeTurns is unavailable before first user message"
                    )));
                }
                Err(err) => {
                    return Err(ThreadReadViewError::Internal(format!(
                        "failed to load rollout `{}` for thread {thread_id}: {err}",
                        rollout_path.display()
                    )));
                }
            }
        }

        Ok(())
    }

    pub(super) async fn thread_turns_list(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadTurnsListParams,
    ) {
        let ThreadTurnsListParams {
            thread_id,
            cursor,
            limit,
            sort_direction,
        } = params;

        let thread_uuid = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        let state_db_ctx = get_state_db(&self.config).await;
        let mut rollout_path = self
            .resolve_rollout_path(thread_uuid, state_db_ctx.as_ref())
            .await;
        if rollout_path.is_none() {
            rollout_path =
                match find_thread_path_by_id_str(&self.config.codex_home, &thread_uuid.to_string())
                    .await
                {
                    Ok(Some(path)) => Some(path),
                    Ok(None) => match find_archived_thread_path_by_id_str(
                        &self.config.codex_home,
                        &thread_uuid.to_string(),
                    )
                    .await
                    {
                        Ok(path) => path,
                        Err(err) => {
                            self.send_invalid_request_error(
                                request_id,
                                format!("failed to locate archived thread id {thread_uuid}: {err}"),
                            )
                            .await;
                            return;
                        }
                    },
                    Err(err) => {
                        self.send_invalid_request_error(
                            request_id,
                            format!("failed to locate thread id {thread_uuid}: {err}"),
                        )
                        .await;
                        return;
                    }
                };
        }

        if rollout_path.is_none() {
            match self.thread_manager.get_thread(thread_uuid).await {
                Ok(thread) => {
                    rollout_path = thread.rollout_path();
                    if rollout_path.is_none() {
                        self.send_invalid_request_error(
                            request_id,
                            "ephemeral threads do not support thread/turns/list".to_string(),
                        )
                        .await;
                        return;
                    }
                }
                Err(_) => {
                    self.send_invalid_request_error(
                        request_id,
                        format!("thread not loaded: {thread_uuid}"),
                    )
                    .await;
                    return;
                }
            }
        }

        let Some(rollout_path) = rollout_path.as_ref() else {
            self.send_internal_error(
                request_id,
                format!("failed to locate rollout for thread {thread_uuid}"),
            )
            .await;
            return;
        };

        match read_rollout_items_from_rollout(rollout_path).await {
            Ok(items) => {
                // This API optimizes network transfer by letting clients page through a
                // thread's turns incrementally, but it still replays the entire rollout on
                // every request. Rollback and compaction events can change earlier turns, so
                // the server has to rebuild the full turn list until turn metadata is indexed
                // separately.
                let has_live_in_progress_turn =
                    match self.thread_manager.get_thread(thread_uuid).await {
                        Ok(thread) => matches!(thread.agent_status().await, AgentStatus::Running),
                        Err(_) => false,
                    };
                let turns = reconstruct_thread_turns_from_rollout_items(
                    &items,
                    self.thread_watch_manager
                        .loaded_status_for_thread(&thread_uuid.to_string())
                        .await,
                    has_live_in_progress_turn,
                );
                let page = match paginate_thread_turns(
                    turns,
                    cursor.as_deref(),
                    limit,
                    sort_direction.unwrap_or(SortDirection::Desc),
                ) {
                    Ok(page) => page,
                    Err(error) => {
                        self.outgoing.send_error(request_id, error).await;
                        return;
                    }
                };
                let response = ThreadTurnsListResponse {
                    data: page.turns,
                    next_cursor: page.next_cursor,
                    backwards_cursor: page.backwards_cursor,
                };
                self.outgoing.send_response(request_id, response).await;
            }
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                self.send_invalid_request_error(
                    request_id,
                    format!(
                        "thread {thread_uuid} is not materialized yet; thread/turns/list is unavailable before first user message"
                    ),
                )
                .await;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!(
                        "failed to load rollout `{}` for thread {thread_uuid}: {err}",
                        rollout_path.display()
                    ),
                )
                .await;
            }
        }
    }
}
