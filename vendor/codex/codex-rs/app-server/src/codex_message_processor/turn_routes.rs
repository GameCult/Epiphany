use super::*;

impl CodexMessageProcessor {
    pub(super) async fn turn_start(
        &self,
        request_id: ConnectionRequestId,
        params: TurnStartParams,
        app_server_client_name: Option<String>,
        app_server_client_version: Option<String>,
    ) {
        if let Err(error) = Self::validate_v2_input_limit(&params.input) {
            self.track_error_response(
                &request_id,
                &error,
                Some(AnalyticsJsonRpcError::Input(InputError::TooLarge)),
            );
            self.outgoing.send_error(request_id, error).await;
            return;
        }
        let (_, thread) = match self.load_thread(&params.thread_id).await {
            Ok(v) => v,
            Err(error) => {
                self.track_error_response(&request_id, &error, /*error_type*/ None);
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };
        if let Err(error) = Self::set_app_server_client_info(
            thread.as_ref(),
            app_server_client_name,
            app_server_client_version,
        )
        .await
        {
            self.track_error_response(&request_id, &error, /*error_type*/ None);
            self.outgoing.send_error(request_id, error).await;
            return;
        }

        let collaboration_modes_config = CollaborationModesConfig {
            default_mode_request_user_input: thread.enabled(Feature::DefaultModeRequestUserInput),
        };
        let collaboration_mode = params.collaboration_mode.map(|mode| {
            self.normalize_turn_start_collaboration_mode(mode, collaboration_modes_config)
        });
        let environments = params.environments.map(|environments| {
            environments
                .into_iter()
                .map(|environment| TurnEnvironmentSelection {
                    environment_id: environment.environment_id,
                    cwd: environment.cwd,
                })
                .collect()
        });

        // Map v2 input items to core input items.
        let mapped_items: Vec<CoreInputItem> = params
            .input
            .into_iter()
            .map(V2UserInput::into_core)
            .collect();

        let has_any_overrides = params.cwd.is_some()
            || params.approval_policy.is_some()
            || params.approvals_reviewer.is_some()
            || params.sandbox_policy.is_some()
            || params.permission_profile.is_some()
            || params.model.is_some()
            || params.service_tier.is_some()
            || params.effort.is_some()
            || params.summary.is_some()
            || collaboration_mode.is_some()
            || params.personality.is_some();

        if params.sandbox_policy.is_some() && params.permission_profile.is_some() {
            self.send_invalid_request_error(
                request_id,
                "`permissionProfile` cannot be combined with `sandboxPolicy`".to_string(),
            )
            .await;
            return;
        }

        let cwd = params.cwd;
        let approval_policy = params.approval_policy.map(AskForApproval::to_core);
        let approvals_reviewer = params
            .approvals_reviewer
            .map(codex_app_server_protocol::ApprovalsReviewer::to_core);
        let sandbox_policy = params.sandbox_policy.map(|p| p.to_core());
        let permission_profile = params.permission_profile.map(Into::into);
        let model = params.model;
        let effort = params.effort.map(Some);
        let summary = params.summary;
        let service_tier = params.service_tier;
        let personality = params.personality;

        // If any overrides are provided, validate them synchronously so the
        // request can fail before accepting user input. The actual update is
        // still queued together with the input below to preserve submission order.
        if has_any_overrides {
            let result = thread
                .validate_turn_context_overrides(CodexThreadTurnContextOverrides {
                    cwd: cwd.clone(),
                    approval_policy,
                    approvals_reviewer,
                    sandbox_policy: sandbox_policy.clone(),
                    permission_profile: permission_profile.clone(),
                    windows_sandbox_level: None,
                    model: model.clone(),
                    effort,
                    summary,
                    service_tier,
                    collaboration_mode: collaboration_mode.clone(),
                    personality,
                })
                .await;
            if let Err(err) = result {
                self.send_invalid_request_error(
                    request_id,
                    format!("invalid turn context override: {err}"),
                )
                .await;
                return;
            }
        }

        // Start the turn by submitting the user input. Return its submission id as turn_id.
        let turn_op = if has_any_overrides {
            Op::UserInputWithTurnContext {
                items: mapped_items,
                environments,
                final_output_json_schema: params.output_schema,
                responsesapi_client_metadata: params.responsesapi_client_metadata,
                cwd,
                approval_policy,
                approvals_reviewer,
                sandbox_policy,
                permission_profile,
                windows_sandbox_level: None,
                model,
                effort,
                summary,
                service_tier,
                collaboration_mode,
                personality,
            }
        } else {
            Op::UserInput {
                items: mapped_items,
                environments,
                final_output_json_schema: params.output_schema,
                responsesapi_client_metadata: params.responsesapi_client_metadata,
            }
        };
        let turn_id = self
            .submit_core_op(&request_id, thread.as_ref(), turn_op)
            .await;

        match turn_id {
            Ok(turn_id) => {
                self.outgoing
                    .record_request_turn_id(&request_id, &turn_id)
                    .await;
                let turn = Turn {
                    id: turn_id.clone(),
                    items: vec![],
                    error: None,
                    status: TurnStatus::InProgress,
                    started_at: None,
                    completed_at: None,
                    duration_ms: None,
                };

                let response = TurnStartResponse { turn };
                if self.config.features.enabled(Feature::GeneralAnalytics) {
                    self.analytics_events_client.track_response(
                        request_id.connection_id.0,
                        ClientResponse::TurnStart {
                            request_id: request_id.request_id.clone(),
                            response: response.clone(),
                        },
                    );
                }
                self.outgoing.send_response(request_id, response).await;
            }
            Err(err) => {
                let error = JSONRPCErrorError {
                    code: INTERNAL_ERROR_CODE,
                    message: format!("failed to start turn: {err}"),
                    data: None,
                };
                self.track_error_response(&request_id, &error, /*error_type*/ None);
                self.outgoing.send_error(request_id, error).await;
            }
        }
    }

    pub(super) async fn thread_inject_items(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadInjectItemsParams,
    ) {
        let (_, thread) = match self.load_thread(&params.thread_id).await {
            Ok(value) => value,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        let items = match params
            .items
            .into_iter()
            .enumerate()
            .map(|(index, value)| {
                serde_json::from_value::<ResponseItem>(value)
                    .map_err(|err| format!("items[{index}] is not a valid response item: {err}"))
            })
            .collect::<std::result::Result<Vec<_>, _>>()
        {
            Ok(items) => items,
            Err(message) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
        };

        match thread.inject_response_items(items).await {
            Ok(()) => {
                self.outgoing
                    .send_response(request_id, ThreadInjectItemsResponse {})
                    .await;
            }
            Err(CodexErr::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to inject response items: {err}"),
                )
                .await;
            }
        }
    }

    pub(super) async fn set_app_server_client_info(
        thread: &CodexThread,
        app_server_client_name: Option<String>,
        app_server_client_version: Option<String>,
    ) -> Result<(), JSONRPCErrorError> {
        thread
            .set_app_server_client_info(app_server_client_name, app_server_client_version)
            .await
            .map_err(|err| JSONRPCErrorError {
                code: INTERNAL_ERROR_CODE,
                message: format!("failed to set app server client info: {err}"),
                data: None,
            })
    }

    pub(super) async fn turn_steer(
        &self,
        request_id: ConnectionRequestId,
        params: TurnSteerParams,
    ) {
        let (_, thread) = match self.load_thread(&params.thread_id).await {
            Ok(v) => v,
            Err(error) => {
                self.track_error_response(&request_id, &error, /*error_type*/ None);
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        if params.expected_turn_id.is_empty() {
            self.send_invalid_request_error(
                request_id,
                "expectedTurnId must not be empty".to_string(),
            )
            .await;
            return;
        }
        self.outgoing
            .record_request_turn_id(&request_id, &params.expected_turn_id)
            .await;
        if let Err(error) = Self::validate_v2_input_limit(&params.input) {
            self.track_error_response(
                &request_id,
                &error,
                Some(AnalyticsJsonRpcError::Input(InputError::TooLarge)),
            );
            self.outgoing.send_error(request_id, error).await;
            return;
        }

        let mapped_items: Vec<CoreInputItem> = params
            .input
            .into_iter()
            .map(V2UserInput::into_core)
            .collect();

        match thread
            .steer_input(
                mapped_items,
                Some(&params.expected_turn_id),
                params.responsesapi_client_metadata,
            )
            .await
        {
            Ok(turn_id) => {
                let response = TurnSteerResponse { turn_id };
                if self.config.features.enabled(Feature::GeneralAnalytics) {
                    self.analytics_events_client.track_response(
                        request_id.connection_id.0,
                        ClientResponse::TurnSteer {
                            request_id: request_id.request_id.clone(),
                            response: response.clone(),
                        },
                    );
                }
                self.outgoing.send_response(request_id, response).await;
            }
            Err(err) => {
                let (code, message, data, error_type) = match err {
                    SteerInputError::NoActiveTurn(_) => (
                        INVALID_REQUEST_ERROR_CODE,
                        "no active turn to steer".to_string(),
                        None,
                        Some(AnalyticsJsonRpcError::TurnSteer(
                            TurnSteerRequestError::NoActiveTurn,
                        )),
                    ),
                    SteerInputError::ExpectedTurnMismatch { expected, actual } => (
                        INVALID_REQUEST_ERROR_CODE,
                        format!("expected active turn id `{expected}` but found `{actual}`"),
                        None,
                        Some(AnalyticsJsonRpcError::TurnSteer(
                            TurnSteerRequestError::ExpectedTurnMismatch,
                        )),
                    ),
                    SteerInputError::ActiveTurnNotSteerable { turn_kind } => {
                        let (message, turn_steer_error) = match turn_kind {
                            codex_protocol::protocol::NonSteerableTurnKind::Review => (
                                "cannot steer a review turn".to_string(),
                                TurnSteerRequestError::NonSteerableReview,
                            ),
                            codex_protocol::protocol::NonSteerableTurnKind::Compact => (
                                "cannot steer a compact turn".to_string(),
                                TurnSteerRequestError::NonSteerableCompact,
                            ),
                        };
                        let error = TurnError {
                            message: message.clone(),
                            codex_error_info: Some(CodexErrorInfo::ActiveTurnNotSteerable {
                                turn_kind: turn_kind.into(),
                            }),
                            additional_details: None,
                        };
                        let data = match serde_json::to_value(error) {
                            Ok(data) => Some(data),
                            Err(error) => {
                                tracing::error!(
                                    ?error,
                                    "failed to serialize active-turn-not-steerable turn error"
                                );
                                None
                            }
                        };
                        (
                            INVALID_REQUEST_ERROR_CODE,
                            message,
                            data,
                            Some(AnalyticsJsonRpcError::TurnSteer(turn_steer_error)),
                        )
                    }
                    SteerInputError::EmptyInput => (
                        INVALID_REQUEST_ERROR_CODE,
                        "input must not be empty".to_string(),
                        None,
                        Some(AnalyticsJsonRpcError::Input(InputError::Empty)),
                    ),
                };
                let error = JSONRPCErrorError {
                    code,
                    message,
                    data,
                };
                self.track_error_response(&request_id, &error, error_type);
                self.outgoing.send_error(request_id, error).await;
            }
        }
    }

    pub(super) async fn turn_interrupt(
        &self,
        request_id: ConnectionRequestId,
        params: TurnInterruptParams,
    ) {
        let TurnInterruptParams { thread_id, turn_id } = params;
        let is_startup_interrupt = turn_id.is_empty();
        if !is_startup_interrupt {
            self.outgoing
                .record_request_turn_id(&request_id, &turn_id)
                .await;
        }

        let (thread_uuid, thread) = match self.load_thread(&thread_id).await {
            Ok(v) => v,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        // Record turn interrupts so we can reply when TurnAborted arrives. Startup
        // interrupts do not have a turn and are acknowledged after submission.
        if !is_startup_interrupt {
            let thread_state = self.thread_state_manager.thread_state(thread_uuid).await;
            let mut thread_state = thread_state.lock().await;
            thread_state
                .pending_interrupts
                .push((request_id.clone(), ApiVersion::V2));
        }

        // Submit the interrupt. Turn interrupts respond upon TurnAborted; startup
        // interrupts respond here because startup cancellation has no turn event.
        let submit_result = self
            .submit_core_op(&request_id, thread.as_ref(), Op::Interrupt)
            .await;
        match submit_result {
            Ok(_) if is_startup_interrupt => {
                self.outgoing
                    .send_response(request_id, TurnInterruptResponse {})
                    .await;
            }
            Ok(_) => {}
            Err(err) => {
                if !is_startup_interrupt {
                    let thread_state = self.thread_state_manager.thread_state(thread_uuid).await;
                    let mut thread_state = thread_state.lock().await;
                    thread_state
                        .pending_interrupts
                        .retain(|(pending_request_id, _)| pending_request_id != &request_id);
                }
                let interrupt_target = if is_startup_interrupt {
                    "startup"
                } else {
                    "turn"
                };
                self.send_internal_error(
                    request_id,
                    format!("failed to interrupt {interrupt_target}: {err}"),
                )
                .await;
            }
        }
    }
}
