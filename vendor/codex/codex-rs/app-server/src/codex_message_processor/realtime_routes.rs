use super::*;

impl CodexMessageProcessor {
    async fn prepare_realtime_conversation_thread(
        &self,
        request_id: ConnectionRequestId,
        thread_id: &str,
    ) -> Option<(ThreadId, Arc<CodexThread>)> {
        let (thread_id, thread) = match self.load_thread(thread_id).await {
            Ok(v) => v,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return None;
            }
        };

        match self
            .ensure_conversation_listener(
                thread_id,
                request_id.connection_id,
                /*raw_events_enabled*/ false,
                ApiVersion::V2,
            )
            .await
        {
            Ok(EnsureConversationListenerResult::Attached) => {}
            Ok(EnsureConversationListenerResult::ConnectionClosed) => {
                return None;
            }
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return None;
            }
        }

        if !thread.enabled(Feature::RealtimeConversation) {
            self.send_invalid_request_error(
                request_id,
                format!("thread {thread_id} does not support realtime conversation"),
            )
            .await;
            return None;
        }

        Some((thread_id, thread))
    }

    pub(super) async fn thread_realtime_start(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadRealtimeStartParams,
    ) {
        let Some((_, thread)) = self
            .prepare_realtime_conversation_thread(request_id.clone(), &params.thread_id)
            .await
        else {
            return;
        };

        let submit = self
            .submit_core_op(
                &request_id,
                thread.as_ref(),
                Op::RealtimeConversationStart(ConversationStartParams {
                    output_modality: params.output_modality,
                    prompt: params.prompt,
                    session_id: params.session_id,
                    transport: params.transport.map(|transport| match transport {
                        ThreadRealtimeStartTransport::Websocket => {
                            ConversationStartTransport::Websocket
                        }
                        ThreadRealtimeStartTransport::Webrtc { sdp } => {
                            ConversationStartTransport::Webrtc { sdp }
                        }
                    }),
                    voice: params.voice,
                }),
            )
            .await;

        match submit {
            Ok(_) => {
                self.outgoing
                    .send_response(request_id, ThreadRealtimeStartResponse::default())
                    .await;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to start realtime conversation: {err}"),
                )
                .await;
            }
        }
    }

    pub(super) async fn thread_realtime_append_audio(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadRealtimeAppendAudioParams,
    ) {
        let Some((_, thread)) = self
            .prepare_realtime_conversation_thread(request_id.clone(), &params.thread_id)
            .await
        else {
            return;
        };

        let submit = self
            .submit_core_op(
                &request_id,
                thread.as_ref(),
                Op::RealtimeConversationAudio(ConversationAudioParams {
                    frame: params.audio.into(),
                }),
            )
            .await;

        match submit {
            Ok(_) => {
                self.outgoing
                    .send_response(request_id, ThreadRealtimeAppendAudioResponse::default())
                    .await;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to append realtime conversation audio: {err}"),
                )
                .await;
            }
        }
    }

    pub(super) async fn thread_realtime_append_text(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadRealtimeAppendTextParams,
    ) {
        let Some((_, thread)) = self
            .prepare_realtime_conversation_thread(request_id.clone(), &params.thread_id)
            .await
        else {
            return;
        };

        let submit = self
            .submit_core_op(
                &request_id,
                thread.as_ref(),
                Op::RealtimeConversationText(ConversationTextParams { text: params.text }),
            )
            .await;

        match submit {
            Ok(_) => {
                self.outgoing
                    .send_response(request_id, ThreadRealtimeAppendTextResponse::default())
                    .await;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to append realtime conversation text: {err}"),
                )
                .await;
            }
        }
    }

    pub(super) async fn thread_realtime_stop(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadRealtimeStopParams,
    ) {
        let Some((_, thread)) = self
            .prepare_realtime_conversation_thread(request_id.clone(), &params.thread_id)
            .await
        else {
            return;
        };

        let submit = self
            .submit_core_op(&request_id, thread.as_ref(), Op::RealtimeConversationClose)
            .await;

        match submit {
            Ok(_) => {
                self.outgoing
                    .send_response(request_id, ThreadRealtimeStopResponse::default())
                    .await;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to stop realtime conversation: {err}"),
                )
                .await;
            }
        }
    }

    pub(super) async fn thread_realtime_list_voices(
        &self,
        request_id: ConnectionRequestId,
        _params: ThreadRealtimeListVoicesParams,
    ) {
        self.outgoing
            .send_response(
                request_id,
                ThreadRealtimeListVoicesResponse {
                    voices: RealtimeVoicesList::builtin(),
                },
            )
            .await;
    }
}
