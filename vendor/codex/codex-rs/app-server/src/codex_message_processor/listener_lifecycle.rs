use super::*;

impl CodexMessageProcessor {
    pub(crate) fn thread_created_receiver(&self) -> broadcast::Receiver<ThreadId> {
        self.thread_manager.subscribe_thread_created()
    }

    pub(crate) async fn connection_initialized(&self, connection_id: ConnectionId) {
        self.thread_state_manager
            .connection_initialized(connection_id)
            .await;
    }

    pub(crate) async fn connection_closed(&self, connection_id: ConnectionId) {
        self.command_exec_manager
            .connection_closed(connection_id)
            .await;
        let thread_ids = self
            .thread_state_manager
            .remove_connection(connection_id)
            .await;

        for thread_id in thread_ids {
            if self.thread_manager.get_thread(thread_id).await.is_err() {
                // Reconcile stale app-server bookkeeping when the thread has already been
                // removed from the core manager.
                self.finalize_thread_teardown(thread_id).await;
            }
        }
    }

    pub(crate) fn subscribe_running_assistant_turn_count(&self) -> watch::Receiver<usize> {
        self.thread_watch_manager.subscribe_running_turn_count()
    }

    /// Best-effort: ensure initialized connections are subscribed to this thread.
    pub(crate) async fn try_attach_thread_listener(
        &self,
        thread_id: ThreadId,
        connection_ids: Vec<ConnectionId>,
    ) {
        if let Ok(thread) = self.thread_manager.get_thread(thread_id).await {
            let config_snapshot = thread.config_snapshot().await;
            let loaded_thread =
                build_thread_from_snapshot(thread_id, &config_snapshot, thread.rollout_path());
            self.thread_watch_manager.upsert_thread(loaded_thread).await;
        }

        for connection_id in connection_ids {
            Self::log_listener_attach_result(
                self.ensure_conversation_listener(
                    thread_id,
                    connection_id,
                    /*raw_events_enabled*/ false,
                    ApiVersion::V2,
                )
                .await,
                thread_id,
                connection_id,
                "thread",
            );
        }
    }

    pub(super) async fn wait_for_thread_shutdown(
        thread: &Arc<CodexThread>,
    ) -> ThreadShutdownResult {
        match tokio::time::timeout(Duration::from_secs(10), thread.shutdown_and_wait()).await {
            Ok(Ok(())) => ThreadShutdownResult::Complete,
            Ok(Err(_)) => ThreadShutdownResult::SubmitFailed,
            Err(_) => ThreadShutdownResult::TimedOut,
        }
    }

    pub(super) async fn finalize_thread_teardown(&self, thread_id: ThreadId) {
        self.pending_thread_unloads.lock().await.remove(&thread_id);
        self.outgoing
            .cancel_requests_for_thread(thread_id, /*error*/ None)
            .await;
        self.thread_state_manager
            .remove_thread_state(thread_id)
            .await;
        self.epiphany_invalidation_manager
            .remove_thread(&thread_id.to_string())
            .await;
        self.thread_watch_manager
            .remove_thread(&thread_id.to_string())
            .await;
    }

    pub(super) async fn unload_thread_without_subscribers(
        thread_manager: Arc<ThreadManager>,
        outgoing: Arc<OutgoingMessageSender>,
        pending_thread_unloads: Arc<Mutex<HashSet<ThreadId>>>,
        thread_state_manager: ThreadStateManager,
        thread_watch_manager: ThreadWatchManager,
        epiphany_invalidation_manager: EpiphanyInvalidationManager,
        thread_id: ThreadId,
        thread: Arc<CodexThread>,
    ) {
        info!("thread {thread_id} has no subscribers and is idle; shutting down");

        // Any pending app-server -> client requests for this thread can no longer be
        // answered; cancel their callbacks before shutdown/unload.
        outgoing
            .cancel_requests_for_thread(thread_id, /*error*/ None)
            .await;
        thread_state_manager.remove_thread_state(thread_id).await;
        epiphany_invalidation_manager
            .remove_thread(&thread_id.to_string())
            .await;

        tokio::spawn(async move {
            match Self::wait_for_thread_shutdown(&thread).await {
                ThreadShutdownResult::Complete => {
                    if thread_manager.remove_thread(&thread_id).await.is_none() {
                        info!("thread {thread_id} was already removed before teardown finalized");
                        epiphany_invalidation_manager
                            .remove_thread(&thread_id.to_string())
                            .await;
                        thread_watch_manager
                            .remove_thread(&thread_id.to_string())
                            .await;
                        pending_thread_unloads.lock().await.remove(&thread_id);
                        return;
                    }
                    epiphany_invalidation_manager
                        .remove_thread(&thread_id.to_string())
                        .await;
                    thread_watch_manager
                        .remove_thread(&thread_id.to_string())
                        .await;
                    let notification = ThreadClosedNotification {
                        thread_id: thread_id.to_string(),
                    };
                    outgoing
                        .send_server_notification(ServerNotification::ThreadClosed(notification))
                        .await;
                    pending_thread_unloads.lock().await.remove(&thread_id);
                }
                ThreadShutdownResult::SubmitFailed => {
                    pending_thread_unloads.lock().await.remove(&thread_id);
                    warn!("failed to submit Shutdown to thread {thread_id}");
                }
                ThreadShutdownResult::TimedOut => {
                    pending_thread_unloads.lock().await.remove(&thread_id);
                    warn!("thread {thread_id} shutdown timed out; leaving thread loaded");
                }
            }
        });
    }

    pub(super) async fn thread_unsubscribe(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadUnsubscribeParams,
    ) {
        let thread_id = match ThreadId::from_string(&params.thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        if self.thread_manager.get_thread(thread_id).await.is_err() {
            // Reconcile stale app-server bookkeeping when the thread has already been
            // removed from the core manager. This keeps loaded-status/subscription state
            // consistent with the source of truth before reporting NotLoaded.
            self.finalize_thread_teardown(thread_id).await;
            self.outgoing
                .send_response(
                    request_id,
                    ThreadUnsubscribeResponse {
                        status: ThreadUnsubscribeStatus::NotLoaded,
                    },
                )
                .await;
            return;
        };

        let was_subscribed = self
            .thread_state_manager
            .unsubscribe_connection_from_thread(thread_id, request_id.connection_id)
            .await;

        let status = if was_subscribed {
            ThreadUnsubscribeStatus::Unsubscribed
        } else {
            ThreadUnsubscribeStatus::NotSubscribed
        };
        self.outgoing
            .send_response(request_id, ThreadUnsubscribeResponse { status })
            .await;
    }

    pub(super) async fn prepare_thread_for_archive(&self, thread_id: ThreadId) {
        // If the thread is active, request shutdown and wait briefly.
        let removed_conversation = self.thread_manager.remove_thread(&thread_id).await;
        if let Some(conversation) = removed_conversation {
            info!("thread {thread_id} was active; shutting down");
            match Self::wait_for_thread_shutdown(&conversation).await {
                ThreadShutdownResult::Complete => {}
                ThreadShutdownResult::SubmitFailed => {
                    error!(
                        "failed to submit Shutdown to thread {thread_id}; proceeding with archive"
                    );
                }
                ThreadShutdownResult::TimedOut => {
                    warn!("thread {thread_id} shutdown timed out; proceeding with archive");
                }
            }
        }
        self.finalize_thread_teardown(thread_id).await;
    }

    pub(super) async fn ensure_conversation_listener(
        &self,
        conversation_id: ThreadId,
        connection_id: ConnectionId,
        raw_events_enabled: bool,
        api_version: ApiVersion,
    ) -> Result<EnsureConversationListenerResult, JSONRPCErrorError> {
        Self::ensure_conversation_listener_task(
            ListenerTaskContext {
                thread_manager: Arc::clone(&self.thread_manager),
                thread_state_manager: self.thread_state_manager.clone(),
                outgoing: Arc::clone(&self.outgoing),
                pending_thread_unloads: Arc::clone(&self.pending_thread_unloads),
                analytics_events_client: self.analytics_events_client.clone(),
                general_analytics_enabled: self.config.features.enabled(Feature::GeneralAnalytics),
                thread_watch_manager: self.thread_watch_manager.clone(),
                epiphany_invalidation_manager: self.epiphany_invalidation_manager.clone(),
                fallback_model_provider: self.config.model_provider_id.clone(),
                codex_home: self.config.codex_home.to_path_buf(),
            },
            conversation_id,
            connection_id,
            raw_events_enabled,
            api_version,
        )
        .await
    }

    #[expect(
        clippy::await_holding_invalid_type,
        reason = "listener subscription must be serialized against pending thread unloads"
    )]
    pub(super) async fn ensure_conversation_listener_task(
        listener_task_context: ListenerTaskContext,
        conversation_id: ThreadId,
        connection_id: ConnectionId,
        raw_events_enabled: bool,
        api_version: ApiVersion,
    ) -> Result<EnsureConversationListenerResult, JSONRPCErrorError> {
        let conversation = match listener_task_context
            .thread_manager
            .get_thread(conversation_id)
            .await
        {
            Ok(conv) => conv,
            Err(_) => {
                return Err(JSONRPCErrorError {
                    code: INVALID_REQUEST_ERROR_CODE,
                    message: format!("thread not found: {conversation_id}"),
                    data: None,
                });
            }
        };
        let thread_state = {
            let pending_thread_unloads = listener_task_context.pending_thread_unloads.lock().await;
            if pending_thread_unloads.contains(&conversation_id) {
                return Err(JSONRPCErrorError {
                    code: INVALID_REQUEST_ERROR_CODE,
                    message: format!(
                        "thread {conversation_id} is closing; retry after the thread is closed"
                    ),
                    data: None,
                });
            }
            let Some(thread_state) = listener_task_context
                .thread_state_manager
                .try_ensure_connection_subscribed(
                    conversation_id,
                    connection_id,
                    raw_events_enabled,
                )
                .await
            else {
                return Ok(EnsureConversationListenerResult::ConnectionClosed);
            };
            thread_state
        };
        if let Err(error) = Self::ensure_listener_task_running_task(
            listener_task_context.clone(),
            conversation_id,
            conversation,
            thread_state,
            api_version,
        )
        .await
        {
            let _ = listener_task_context
                .thread_state_manager
                .unsubscribe_connection_from_thread(conversation_id, connection_id)
                .await;
            return Err(error);
        }
        Ok(EnsureConversationListenerResult::Attached)
    }

    pub(super) fn log_listener_attach_result(
        result: Result<EnsureConversationListenerResult, JSONRPCErrorError>,
        thread_id: ThreadId,
        connection_id: ConnectionId,
        thread_kind: &'static str,
    ) {
        match result {
            Ok(EnsureConversationListenerResult::Attached) => {}
            Ok(EnsureConversationListenerResult::ConnectionClosed) => {
                tracing::debug!(
                    thread_id = %thread_id,
                    connection_id = ?connection_id,
                    "skipping auto-attach for closed connection"
                );
            }
            Err(err) => {
                tracing::warn!(
                    "failed to attach listener for {thread_kind} {thread_id}: {message}",
                    message = err.message
                );
            }
        }
    }

    pub(super) async fn ensure_listener_task_running(
        &self,
        conversation_id: ThreadId,
        conversation: Arc<CodexThread>,
        thread_state: Arc<Mutex<ThreadState>>,
        api_version: ApiVersion,
    ) -> Result<(), JSONRPCErrorError> {
        Self::ensure_listener_task_running_task(
            ListenerTaskContext {
                thread_manager: Arc::clone(&self.thread_manager),
                thread_state_manager: self.thread_state_manager.clone(),
                outgoing: Arc::clone(&self.outgoing),
                pending_thread_unloads: Arc::clone(&self.pending_thread_unloads),
                analytics_events_client: self.analytics_events_client.clone(),
                general_analytics_enabled: self.config.features.enabled(Feature::GeneralAnalytics),
                thread_watch_manager: self.thread_watch_manager.clone(),
                epiphany_invalidation_manager: self.epiphany_invalidation_manager.clone(),
                fallback_model_provider: self.config.model_provider_id.clone(),
                codex_home: self.config.codex_home.to_path_buf(),
            },
            conversation_id,
            conversation,
            thread_state,
            api_version,
        )
        .await
    }

    pub(super) async fn ensure_listener_task_running_task(
        listener_task_context: ListenerTaskContext,
        conversation_id: ThreadId,
        conversation: Arc<CodexThread>,
        thread_state: Arc<Mutex<ThreadState>>,
        api_version: ApiVersion,
    ) -> Result<(), JSONRPCErrorError> {
        let (cancel_tx, mut cancel_rx) = oneshot::channel();
        let Some(mut unloading_state) = UnloadingState::new(
            &listener_task_context,
            conversation_id,
            THREAD_UNLOADING_DELAY,
        )
        .await
        else {
            return Err(JSONRPCErrorError {
                code: INVALID_REQUEST_ERROR_CODE,
                message: format!(
                    "thread {conversation_id} is closing; retry after the thread is closed"
                ),
                data: None,
            });
        };
        let (mut listener_command_rx, listener_generation) = {
            let mut thread_state = thread_state.lock().await;
            if thread_state.listener_matches(&conversation) {
                return Ok(());
            }
            thread_state.set_listener(cancel_tx, &conversation)
        };
        let ListenerTaskContext {
            outgoing,
            thread_manager,
            thread_state_manager,
            pending_thread_unloads,
            analytics_events_client: _,
            general_analytics_enabled: _,
            thread_watch_manager,
            epiphany_invalidation_manager,
            fallback_model_provider,
            codex_home,
        } = listener_task_context;
        let outgoing_for_task = Arc::clone(&outgoing);
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    biased;
                    _ = &mut cancel_rx => {
                        // Listener was superseded or the thread is being torn down.
                        break;
                    }
                    listener_command = listener_command_rx.recv() => {
                        let Some(listener_command) = listener_command else {
                            break;
                        };
                        handle_thread_listener_command(
                            conversation_id,
                            &conversation,
                            codex_home.as_path(),
                            &thread_state_manager,
                            &thread_state,
                            &thread_watch_manager,
                            &outgoing_for_task,
                            &pending_thread_unloads,
                            listener_command,
                        )
                        .await;
                    }
                    event = conversation.next_event() => {
                        let event = match event {
                            Ok(event) => event,
                            Err(err) => {
                                tracing::warn!("thread.next_event() failed with: {err}");
                                break;
                            }
                        };

                        // Track the event before emitting any typed
                        // translations so thread-local state such as raw event
                        // opt-in stays synchronized with the conversation.
                        let raw_events_enabled = {
                            let mut thread_state = thread_state.lock().await;
                            thread_state.track_current_turn_event(&event.msg);
                            thread_state.experimental_raw_events
                        };
                        let subscribed_connection_ids = thread_state_manager
                            .subscribed_connection_ids(conversation_id)
                            .await;
                        let thread_outgoing = ThreadScopedOutgoingMessageSender::new(
                            outgoing_for_task.clone(),
                            subscribed_connection_ids,
                            conversation_id,
                        );

                        if let EventMsg::RawResponseItem(raw_response_item_event) = &event.msg
                            && !raw_events_enabled
                        {
                            maybe_emit_hook_prompt_item_completed(
                                api_version,
                                conversation_id,
                                &event.id,
                                &raw_response_item_event.item,
                                &thread_outgoing,
                            )
                            .await;
                            continue;
                        }

                        apply_bespoke_event_handling(
                            event.clone(),
                            conversation_id,
                            conversation.clone(),
                            thread_manager.clone(),
                            listener_task_context
                                .general_analytics_enabled
                                .then(|| listener_task_context.analytics_events_client.clone()),
                            thread_outgoing,
                            thread_state.clone(),
                            thread_watch_manager.clone(),
                            epiphany_invalidation_manager.clone(),
                            api_version,
                            fallback_model_provider.clone(),
                            codex_home.as_path(),
                        )
                        .await;
                    }
                    unloading_watchers_open = unloading_state.wait_for_unloading_trigger() => {
                        if !unloading_watchers_open {
                            break;
                        }
                        if !unloading_state.should_unload_now() {
                            continue;
                        }
                        if matches!(conversation.agent_status().await, AgentStatus::Running) {
                            unloading_state.note_thread_activity_observed();
                            continue;
                        }
                        {
                            let mut pending_thread_unloads = pending_thread_unloads.lock().await;
                            if pending_thread_unloads.contains(&conversation_id) {
                                continue;
                            }
                            if !unloading_state.should_unload_now() {
                                continue;
                            }
                            pending_thread_unloads.insert(conversation_id);
                        }
                        Self::unload_thread_without_subscribers(
                            thread_manager.clone(),
                            outgoing_for_task.clone(),
                            pending_thread_unloads.clone(),
                            thread_state_manager.clone(),
                            thread_watch_manager.clone(),
                            epiphany_invalidation_manager.clone(),
                            conversation_id,
                            conversation.clone(),
                        )
                        .await;
                        break;
                    }
                }
            }

            let mut thread_state = thread_state.lock().await;
            if thread_state.listener_generation == listener_generation {
                thread_state.clear_listener();
            }
        });
        Ok(())
    }
}
