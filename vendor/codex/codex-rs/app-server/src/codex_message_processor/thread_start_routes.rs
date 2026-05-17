use super::*;

impl CodexMessageProcessor {
    pub(super) async fn thread_start(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadStartParams,
        app_server_client_name: Option<String>,
        app_server_client_version: Option<String>,
        request_context: RequestContext,
    ) {
        let ThreadStartParams {
            model,
            model_provider,
            service_tier,
            cwd,
            approval_policy,
            approvals_reviewer,
            sandbox,
            permission_profile,
            config,
            service_name,
            base_instructions,
            developer_instructions,
            dynamic_tools,
            mock_experimental_field: _mock_experimental_field,
            experimental_raw_events,
            personality,
            ephemeral,
            session_start_source,
            persist_extended_history,
        } = params;
        if sandbox.is_some() && permission_profile.is_some() {
            self.send_invalid_request_error(
                request_id,
                "`permissionProfile` cannot be combined with `sandbox`".to_string(),
            )
            .await;
            return;
        }
        let mut typesafe_overrides = self.build_thread_config_overrides(
            model,
            model_provider,
            service_tier,
            cwd,
            approval_policy,
            approvals_reviewer,
            sandbox,
            permission_profile,
            base_instructions,
            developer_instructions,
            personality,
        );
        typesafe_overrides.ephemeral = ephemeral;
        let listener_task_context = ListenerTaskContext {
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
        };
        let request_trace = request_context.request_trace();
        let config_manager = self.config_manager.clone();
        let thread_start_task = async move {
            Self::thread_start_task(
                listener_task_context,
                config_manager,
                request_id,
                app_server_client_name,
                app_server_client_version,
                config,
                typesafe_overrides,
                dynamic_tools,
                session_start_source,
                persist_extended_history,
                service_name,
                experimental_raw_events,
                request_trace,
            )
            .await;
        };
        self.background_tasks
            .spawn(thread_start_task.instrument(request_context.span()));
    }
}
