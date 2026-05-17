use super::*;

impl CodexMessageProcessor {
    pub(super) async fn mcp_server_refresh(
        &self,
        request_id: ConnectionRequestId,
        _params: Option<()>,
    ) {
        let config = match self.load_latest_config(/*fallback_cwd*/ None).await {
            Ok(config) => config,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        if let Err(error) = self.queue_mcp_server_refresh_for_config(&config).await {
            self.outgoing.send_error(request_id, error).await;
            return;
        }

        let response = McpServerRefreshResponse {};
        self.outgoing.send_response(request_id, response).await;
    }

    async fn queue_mcp_server_refresh_for_config(
        &self,
        config: &Config,
    ) -> Result<(), JSONRPCErrorError> {
        let configured_servers = self
            .thread_manager
            .mcp_manager()
            .configured_servers(config)
            .await;
        let mcp_servers = match serde_json::to_value(configured_servers) {
            Ok(value) => value,
            Err(err) => {
                return Err(JSONRPCErrorError {
                    code: INTERNAL_ERROR_CODE,
                    message: format!("failed to serialize MCP servers: {err}"),
                    data: None,
                });
            }
        };

        let mcp_oauth_credentials_store_mode =
            match serde_json::to_value(config.mcp_oauth_credentials_store_mode) {
                Ok(value) => value,
                Err(err) => {
                    return Err(JSONRPCErrorError {
                        code: INTERNAL_ERROR_CODE,
                        message: format!(
                            "failed to serialize MCP OAuth credentials store mode: {err}"
                        ),
                        data: None,
                    });
                }
            };

        let refresh_config = McpServerRefreshConfig {
            mcp_servers,
            mcp_oauth_credentials_store_mode,
        };

        // Refresh requests are queued per thread; each thread rebuilds MCP connections on its next
        // active turn to avoid work for threads that never resume.
        let thread_manager = Arc::clone(&self.thread_manager);
        thread_manager.refresh_mcp_servers(refresh_config).await;
        Ok(())
    }

    pub(super) async fn mcp_server_oauth_login(
        &self,
        request_id: ConnectionRequestId,
        params: McpServerOauthLoginParams,
    ) {
        let config = match self.load_latest_config(/*fallback_cwd*/ None).await {
            Ok(config) => config,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        let McpServerOauthLoginParams {
            name,
            scopes,
            timeout_secs,
        } = params;

        let configured_servers = self
            .thread_manager
            .mcp_manager()
            .configured_servers(&config)
            .await;
        let Some(server) = configured_servers.get(&name) else {
            let error = JSONRPCErrorError {
                code: INVALID_REQUEST_ERROR_CODE,
                message: format!("No MCP server named '{name}' found."),
                data: None,
            };
            self.outgoing.send_error(request_id, error).await;
            return;
        };

        let (url, http_headers, env_http_headers) = match &server.transport {
            McpServerTransportConfig::StreamableHttp {
                url,
                http_headers,
                env_http_headers,
                ..
            } => (url.clone(), http_headers.clone(), env_http_headers.clone()),
            _ => {
                let error = JSONRPCErrorError {
                    code: INVALID_REQUEST_ERROR_CODE,
                    message: "OAuth login is only supported for streamable HTTP servers."
                        .to_string(),
                    data: None,
                };
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };

        let discovered_scopes = if scopes.is_none() && server.scopes.is_none() {
            discover_supported_scopes(&server.transport).await
        } else {
            None
        };
        let resolved_scopes =
            resolve_oauth_scopes(scopes, server.scopes.clone(), discovered_scopes);

        match perform_oauth_login_return_url(
            &name,
            &url,
            config.mcp_oauth_credentials_store_mode,
            http_headers,
            env_http_headers,
            &resolved_scopes.scopes,
            server.oauth_resource.as_deref(),
            timeout_secs,
            config.mcp_oauth_callback_port,
            config.mcp_oauth_callback_url.as_deref(),
        )
        .await
        {
            Ok(handle) => {
                let authorization_url = handle.authorization_url().to_string();
                let notification_name = name.clone();
                let outgoing = Arc::clone(&self.outgoing);

                tokio::spawn(async move {
                    let (success, error) = match handle.wait().await {
                        Ok(()) => (true, None),
                        Err(err) => (false, Some(err.to_string())),
                    };

                    let notification = ServerNotification::McpServerOauthLoginCompleted(
                        McpServerOauthLoginCompletedNotification {
                            name: notification_name,
                            success,
                            error,
                        },
                    );
                    outgoing.send_server_notification(notification).await;
                });

                let response = McpServerOauthLoginResponse { authorization_url };
                self.outgoing.send_response(request_id, response).await;
            }
            Err(err) => {
                let error = JSONRPCErrorError {
                    code: INTERNAL_ERROR_CODE,
                    message: format!("failed to login to MCP server '{name}': {err}"),
                    data: None,
                };
                self.outgoing.send_error(request_id, error).await;
            }
        }
    }

    pub(super) async fn list_mcp_server_status(
        &self,
        request_id: ConnectionRequestId,
        params: ListMcpServerStatusParams,
    ) {
        let request = request_id.clone();

        let outgoing = Arc::clone(&self.outgoing);
        let config = match self.load_latest_config(/*fallback_cwd*/ None).await {
            Ok(config) => config,
            Err(error) => {
                self.outgoing.send_error(request, error).await;
                return;
            }
        };
        let mcp_config = config.to_mcp_config();
        let auth = self.auth_manager.auth().await;
        let environment_manager = self.thread_manager.environment_manager();
        let runtime_environment = match environment_manager.default_environment() {
            Some(environment) => {
                // Status listing has no turn cwd. This fallback is used by
                // stdio MCPs whose config omits `cwd`.
                McpRuntimeEnvironment::new(environment, config.cwd.to_path_buf())
            }
            None => McpRuntimeEnvironment::new(
                environment_manager.local_environment(),
                config.cwd.to_path_buf(),
            ),
        };

        tokio::spawn(async move {
            Self::list_mcp_server_status_task(
                outgoing,
                request,
                params,
                config,
                mcp_config,
                auth,
                runtime_environment,
            )
            .await;
        });
    }

    async fn list_mcp_server_status_task(
        outgoing: Arc<OutgoingMessageSender>,
        request_id: ConnectionRequestId,
        params: ListMcpServerStatusParams,
        config: Config,
        mcp_config: codex_mcp::McpConfig,
        auth: Option<CodexAuth>,
        runtime_environment: McpRuntimeEnvironment,
    ) {
        let detail = match params.detail.unwrap_or(McpServerStatusDetail::Full) {
            McpServerStatusDetail::Full => McpSnapshotDetail::Full,
            McpServerStatusDetail::ToolsAndAuthOnly => McpSnapshotDetail::ToolsAndAuthOnly,
        };

        let snapshot = collect_mcp_server_status_snapshot_with_detail(
            &mcp_config,
            auth.as_ref(),
            request_id.request_id.to_string(),
            runtime_environment,
            detail,
        )
        .await;

        let effective_servers = effective_mcp_servers(&mcp_config, auth.as_ref());
        let McpServerStatusSnapshot {
            tools_by_server,
            resources,
            resource_templates,
            auth_statuses,
        } = snapshot;

        let mut server_names: Vec<String> = config
            .mcp_servers
            .keys()
            .cloned()
            // Include MCP servers that are present in the effective runtime
            // config even when they are not user-declared in `config.mcp_servers`.
            .chain(effective_servers.keys().cloned())
            .chain(auth_statuses.keys().cloned())
            .chain(resources.keys().cloned())
            .chain(resource_templates.keys().cloned())
            .collect();
        server_names.sort();
        server_names.dedup();

        let total = server_names.len();
        let limit = params.limit.unwrap_or(total as u32).max(1) as usize;
        let effective_limit = limit.min(total);
        let start = match params.cursor {
            Some(cursor) => match cursor.parse::<usize>() {
                Ok(idx) => idx,
                Err(_) => {
                    let error = JSONRPCErrorError {
                        code: INVALID_REQUEST_ERROR_CODE,
                        message: format!("invalid cursor: {cursor}"),
                        data: None,
                    };
                    outgoing.send_error(request_id, error).await;
                    return;
                }
            },
            None => 0,
        };

        if start > total {
            let error = JSONRPCErrorError {
                code: INVALID_REQUEST_ERROR_CODE,
                message: format!("cursor {start} exceeds total MCP servers {total}"),
                data: None,
            };
            outgoing.send_error(request_id, error).await;
            return;
        }

        let end = start.saturating_add(effective_limit).min(total);

        let data: Vec<McpServerStatus> = server_names[start..end]
            .iter()
            .map(|name| McpServerStatus {
                name: name.clone(),
                tools: tools_by_server.get(name).cloned().unwrap_or_default(),
                resources: resources.get(name).cloned().unwrap_or_default(),
                resource_templates: resource_templates.get(name).cloned().unwrap_or_default(),
                auth_status: auth_statuses
                    .get(name)
                    .cloned()
                    .unwrap_or(CoreMcpAuthStatus::Unsupported)
                    .into(),
            })
            .collect();

        let next_cursor = if end < total {
            Some(end.to_string())
        } else {
            None
        };

        let response = ListMcpServerStatusResponse { data, next_cursor };

        outgoing.send_response(request_id, response).await;
    }

    pub(super) async fn read_mcp_resource(
        &self,
        request_id: ConnectionRequestId,
        params: McpResourceReadParams,
    ) {
        let outgoing = Arc::clone(&self.outgoing);
        let McpResourceReadParams {
            thread_id,
            server,
            uri,
        } = params;

        if let Some(thread_id) = thread_id {
            let (_, thread) = match self.load_thread(&thread_id).await {
                Ok(thread) => thread,
                Err(error) => {
                    self.outgoing.send_error(request_id, error).await;
                    return;
                }
            };

            tokio::spawn(async move {
                let result = thread.read_mcp_resource(&server, &uri).await;
                Self::send_mcp_resource_read_response(outgoing, request_id, result).await;
            });
            return;
        }

        let config = match self.load_latest_config(/*fallback_cwd*/ None).await {
            Ok(config) => config,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };
        let mcp_config = config.to_mcp_config();
        let auth = self.auth_manager.auth().await;
        let runtime_environment = {
            let environment_manager = self.thread_manager.environment_manager();
            let environment = environment_manager
                .default_environment()
                .unwrap_or_else(|| environment_manager.local_environment());
            // Resource reads without a thread have no turn cwd. This fallback
            // is used only by executor-backed stdio MCPs whose config omits `cwd`.
            McpRuntimeEnvironment::new(environment, config.cwd.to_path_buf())
        };

        tokio::spawn(async move {
            let result = match read_mcp_resource_without_thread(
                &mcp_config,
                auth.as_ref(),
                runtime_environment,
                &server,
                &uri,
            )
            .await
            {
                Ok(result) => serde_json::to_value(result).map_err(anyhow::Error::from),
                Err(error) => Err(error),
            };
            Self::send_mcp_resource_read_response(outgoing, request_id, result).await;
        });
    }

    async fn send_mcp_resource_read_response(
        outgoing: Arc<OutgoingMessageSender>,
        request_id: ConnectionRequestId,
        result: anyhow::Result<serde_json::Value>,
    ) {
        match result {
            Ok(result) => match serde_json::from_value::<McpResourceReadResponse>(result) {
                Ok(response) => {
                    outgoing.send_response(request_id, response).await;
                }
                Err(error) => {
                    outgoing
                        .send_error(
                            request_id,
                            JSONRPCErrorError {
                                code: INTERNAL_ERROR_CODE,
                                message: format!(
                                    "failed to deserialize MCP resource read response: {error}"
                                ),
                                data: None,
                            },
                        )
                        .await;
                }
            },
            Err(error) => {
                outgoing
                    .send_error(
                        request_id,
                        JSONRPCErrorError {
                            code: INTERNAL_ERROR_CODE,
                            message: format!("{error:#}"),
                            data: None,
                        },
                    )
                    .await;
            }
        }
    }

    pub(super) async fn call_mcp_server_tool(
        &self,
        request_id: ConnectionRequestId,
        params: McpServerToolCallParams,
    ) {
        let outgoing = Arc::clone(&self.outgoing);
        let thread_id = params.thread_id.clone();
        let (_, thread) = match self.load_thread(&thread_id).await {
            Ok(thread) => thread,
            Err(error) => {
                self.outgoing.send_error(request_id, error).await;
                return;
            }
        };
        let meta = with_mcp_tool_call_thread_id_meta(params.meta, &thread_id);

        tokio::spawn(async move {
            let result = thread
                .call_mcp_tool(&params.server, &params.tool, params.arguments, meta)
                .await;
            match result {
                Ok(result) => {
                    outgoing
                        .send_response(request_id, McpServerToolCallResponse::from(result))
                        .await;
                }
                Err(error) => {
                    outgoing
                        .send_error(
                            request_id,
                            JSONRPCErrorError {
                                code: INTERNAL_ERROR_CODE,
                                message: format!("{error:#}"),
                                data: None,
                            },
                        )
                        .await;
                }
            }
        });
    }
}

const MCP_TOOL_THREAD_ID_META_KEY: &str = "threadId";

fn with_mcp_tool_call_thread_id_meta(
    meta: Option<serde_json::Value>,
    thread_id: &str,
) -> Option<serde_json::Value> {
    match meta {
        Some(serde_json::Value::Object(mut map)) => {
            map.insert(
                MCP_TOOL_THREAD_ID_META_KEY.to_string(),
                serde_json::Value::String(thread_id.to_string()),
            );
            Some(serde_json::Value::Object(map))
        }
        None => {
            let mut map = serde_json::Map::new();
            map.insert(
                MCP_TOOL_THREAD_ID_META_KEY.to_string(),
                serde_json::Value::String(thread_id.to_string()),
            );
            Some(serde_json::Value::Object(map))
        }
        other => other,
    }
}
