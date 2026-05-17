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
}
