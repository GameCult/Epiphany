use super::*;

impl CodexMessageProcessor {
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
