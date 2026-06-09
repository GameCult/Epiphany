use codex_app_server_protocol::*;
use codex_core::CodexThread;
use codex_protocol::ThreadId;
use codex_protocol::protocol::EpiphanyRetrievalState;
use epiphany_codex_bridge::error::EpiphanyBridgeError;
use epiphany_codex_bridge::invalidation::epiphany_freshness_watcher_snapshot;
use epiphany_codex_bridge::protocol_edge::core_epiphany_view_needs_jobs;
use epiphany_codex_bridge::protocol_edge::core_epiphany_view_needs_pressure;
use epiphany_codex_bridge::protocol_edge::core_epiphany_view_needs_reorientation_inputs;
use epiphany_codex_bridge::protocol_edge::core_epiphany_view_needs_runtime_store;
use epiphany_codex_bridge::protocol_edge::default_core_epiphany_view_lenses;
use epiphany_codex_bridge::protocol_edge::protocol_view_lenses_to_core;
use epiphany_codex_bridge::retrieve::epiphany_retrieval_state_for_paths;
use epiphany_codex_bridge::retrieve_protocol::retrieve_thread_epiphany_for_paths;
use epiphany_codex_bridge::view_protocol::EpiphanyFreshnessResponseInput;
use epiphany_codex_bridge::view_protocol::EpiphanyViewResponseInput;
use epiphany_codex_bridge::view_protocol::map_epiphany_freshness_response;
use epiphany_codex_bridge::view_protocol::map_epiphany_view_response;
use epiphany_codex_bridge::view_protocol::map_thread_epiphany_context_response;
use epiphany_codex_bridge::view_protocol::map_thread_epiphany_distill_response;
use epiphany_codex_bridge::view_protocol::map_thread_epiphany_graph_query_response;
use epiphany_codex_bridge::view_protocol::map_thread_epiphany_propose_response;
use epiphany_codex_bridge::view_protocol::map_thread_epiphany_reorient_result_response;
use epiphany_codex_bridge::view_protocol::map_thread_epiphany_role_result_response;

use super::CodexMessageProcessor;
use super::ConnectionRequestId;
use super::ThreadReadViewError;
use super::epiphany_thread_host::epiphany_token_usage_snapshot;
use super::latest_token_usage_info_from_rollout_path;

impl CodexMessageProcessor {
    pub(super) async fn thread_epiphany_view(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyViewParams,
    ) {
        let ThreadEpiphanyViewParams { thread_id, lenses } = params;
        let lenses = if lenses.is_empty() {
            default_core_epiphany_view_lenses()
        } else {
            protocol_view_lenses_to_core(lenses)
        };

        let thread_uuid = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        let loaded_thread = self.thread_manager.get_thread(thread_uuid).await.ok();
        let loaded = loaded_thread.is_some();
        let thread = match self.read_thread_view(thread_uuid, false).await {
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

        let needs_jobs = core_epiphany_view_needs_jobs(&lenses);
        let needs_reorientation_inputs = core_epiphany_view_needs_reorientation_inputs(&lenses);
        let needs_pressure = core_epiphany_view_needs_pressure(&lenses);
        let retrieval_override = if (needs_jobs || needs_reorientation_inputs)
            && thread
                .epiphany_state
                .as_ref()
                .and_then(|state| state.retrieval.as_ref())
                .is_none()
        {
            if let Some(loaded_thread) = loaded_thread.as_ref() {
                Some(thread_epiphany_retrieval_state(loaded_thread.as_ref()).await)
            } else {
                None
            }
        } else {
            None
        };
        let watcher_snapshot = if needs_reorientation_inputs {
            if let Some(loaded_thread) = loaded_thread.as_ref() {
                let config_snapshot = loaded_thread.config_snapshot().await;
                self.epiphany_invalidation_manager
                    .ensure_thread_watch(&thread_id, &config_snapshot.cwd)
                    .await;
                Some(
                    self.epiphany_invalidation_manager
                        .snapshot(&thread_id)
                        .await,
                )
            } else {
                None
            }
        } else {
            None
        };
        let token_usage_info = if needs_pressure {
            if let Some(loaded_thread) = loaded_thread.as_ref() {
                loaded_thread.token_usage_info().await
            } else {
                match thread.path.as_deref() {
                    Some(path) => latest_token_usage_info_from_rollout_path(path).await,
                    None => None,
                }
            }
        } else {
            None
        };
        let runtime_store_path = if core_epiphany_view_needs_runtime_store(&lenses) {
            if let Some(loaded_thread) = loaded_thread.as_ref() {
                Some(loaded_thread.epiphany_runtime_spine_store_path().await)
            } else {
                None
            }
        } else {
            None
        };
        let token_usage_snapshot = epiphany_token_usage_snapshot(token_usage_info.as_ref());
        let response = map_epiphany_view_response(EpiphanyViewResponseInput {
            thread_id: thread_id.clone(),
            lenses,
            loaded,
            state: thread.epiphany_state.as_ref(),
            retrieval_override: retrieval_override.as_ref(),
            watcher_snapshot: watcher_snapshot
                .as_ref()
                .map(epiphany_freshness_watcher_snapshot),
            token_usage_info: token_usage_snapshot.as_ref(),
            runtime_store_path: runtime_store_path.as_deref(),
        })
        .await;
        self.outgoing.send_response(request_id, response).await;
    }

    pub(super) async fn thread_epiphany_role_result(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyRoleResultParams,
    ) {
        let thread_id = params.thread_id.clone();

        let thread_uuid = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        let loaded_thread = self.thread_manager.get_thread(thread_uuid).await.ok();
        let thread = match self.read_thread_view(thread_uuid, false).await {
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
        let loaded = loaded_thread.is_some();

        let runtime_store_path = if let Some(loaded_thread) = loaded_thread.as_ref() {
            Some(loaded_thread.epiphany_runtime_spine_store_path().await)
        } else {
            None
        };
        let response = match map_thread_epiphany_role_result_response(
            params,
            loaded,
            thread.epiphany_state.as_ref(),
            runtime_store_path.as_deref(),
        )
        .await
        {
            Ok(response) => response,
            Err(EpiphanyBridgeError::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
            Err(err) => {
                self.send_internal_error(request_id, err.to_string()).await;
                return;
            }
        };

        self.outgoing.send_response(request_id, response).await;
    }

    pub(super) async fn thread_epiphany_freshness(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyFreshnessParams,
    ) {
        let ThreadEpiphanyFreshnessParams { thread_id } = params;

        let thread_uuid = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        let loaded_thread = self.thread_manager.get_thread(thread_uuid).await.ok();
        let thread = match self.read_thread_view(thread_uuid, false).await {
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

        let retrieval_override = if let Some(loaded_thread) = loaded_thread.as_ref() {
            Some(thread_epiphany_retrieval_state(loaded_thread.as_ref()).await)
        } else {
            None
        };
        let watcher_snapshot = if let Some(loaded_thread) = loaded_thread.as_ref() {
            let config_snapshot = loaded_thread.config_snapshot().await;
            self.epiphany_invalidation_manager
                .ensure_thread_watch(&thread_id, &config_snapshot.cwd)
                .await;
            Some(
                self.epiphany_invalidation_manager
                    .snapshot(&thread_id)
                    .await,
            )
        } else {
            None
        };
        let response = map_epiphany_freshness_response(EpiphanyFreshnessResponseInput {
            thread_id,
            loaded: loaded_thread.is_some(),
            state: thread.epiphany_state.as_ref(),
            retrieval_override: retrieval_override.as_ref(),
            watcher_snapshot: watcher_snapshot
                .as_ref()
                .map(epiphany_freshness_watcher_snapshot),
        });
        self.outgoing.send_response(request_id, response).await;
    }

    pub(super) async fn thread_epiphany_context(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyContextParams,
    ) {
        let thread_id = params.thread_id.clone();
        let thread_uuid = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        let loaded = self.thread_manager.get_thread(thread_uuid).await.is_ok();
        let thread = match self.read_thread_view(thread_uuid, false).await {
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

        let response =
            map_thread_epiphany_context_response(params, loaded, thread.epiphany_state.as_ref());
        self.outgoing.send_response(request_id, response).await;
    }

    pub(super) async fn thread_epiphany_graph_query(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyGraphQueryParams,
    ) {
        let thread_id = params.thread_id.clone();
        let thread_uuid = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        let loaded = self.thread_manager.get_thread(thread_uuid).await.is_ok();
        let thread = match self.read_thread_view(thread_uuid, false).await {
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

        let response = map_thread_epiphany_graph_query_response(
            params,
            loaded,
            thread.epiphany_state.as_ref(),
        );
        self.outgoing.send_response(request_id, response).await;
    }

    pub(super) async fn thread_epiphany_reorient_result(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyReorientResultParams,
    ) {
        let thread_id = params.thread_id.clone();

        let thread_uuid = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        let loaded_thread = self.thread_manager.get_thread(thread_uuid).await.ok();
        let thread = match self.read_thread_view(thread_uuid, false).await {
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

        let loaded = loaded_thread.is_some();
        let runtime_store_path = if let Some(loaded_thread) = loaded_thread.as_ref() {
            Some(loaded_thread.epiphany_runtime_spine_store_path().await)
        } else {
            None
        };
        let response = map_thread_epiphany_reorient_result_response(
            params,
            loaded,
            thread.epiphany_state.as_ref(),
            runtime_store_path.as_deref(),
        )
        .await;

        self.outgoing.send_response(request_id, response).await;
    }

    pub(super) async fn thread_epiphany_retrieve(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyRetrieveParams,
    ) {
        let ThreadEpiphanyRetrieveParams {
            thread_id,
            query,
            limit,
            path_prefixes,
        } = params;

        let thread_uuid = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        let thread = match self.thread_manager.get_thread(thread_uuid).await {
            Ok(thread) => thread,
            Err(_) => {
                self.send_invalid_request_error(
                    request_id,
                    format!("thread not loaded: {thread_uuid}"),
                )
                .await;
                return;
            }
        };

        let config = thread.config_snapshot().await;
        let codex_home = thread.codex_home().await;
        let response = match retrieve_thread_epiphany_for_paths(
            config.cwd.to_path_buf(),
            codex_home,
            query,
            limit,
            path_prefixes,
        )
        .await
        {
            Ok(response) => response,
            Err(EpiphanyBridgeError::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to retrieve Epiphany results for {thread_uuid}: {err}"),
                )
                .await;
                return;
            }
        };

        self.outgoing.send_response(request_id, response).await;
    }

    pub(super) async fn thread_epiphany_distill(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyDistillParams,
    ) {
        let thread_id = params.thread_id.clone();

        let thread_uuid = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        let thread = match self.thread_manager.get_thread(thread_uuid).await {
            Ok(thread) => thread,
            Err(_) => {
                self.send_invalid_request_error(
                    request_id,
                    format!("thread not loaded: {thread_uuid}"),
                )
                .await;
                return;
            }
        };

        let state = thread.epiphany_state().await;
        let response = match map_thread_epiphany_distill_response(params, state.as_ref()) {
            Ok(response) => response,
            Err(EpiphanyBridgeError::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
            Err(err) => {
                self.send_internal_error(request_id, err.to_string()).await;
                return;
            }
        };

        self.outgoing.send_response(request_id, response).await;
    }

    pub(super) async fn thread_epiphany_propose(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyProposeParams,
    ) {
        let ThreadEpiphanyProposeParams { thread_id, .. } = &params;
        let thread_id = thread_id.clone();

        let thread_uuid = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        let thread = match self.thread_manager.get_thread(thread_uuid).await {
            Ok(thread) => thread,
            Err(_) => {
                self.send_invalid_request_error(
                    request_id,
                    format!("thread not loaded: {thread_uuid}"),
                )
                .await;
                return;
            }
        };

        let state = thread.epiphany_state().await;
        let response = match map_thread_epiphany_propose_response(params, state) {
            Ok(response) => response,
            Err(EpiphanyBridgeError::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
            Err(err) => {
                self.send_internal_error(request_id, err.to_string()).await;
                return;
            }
        };

        self.outgoing.send_response(request_id, response).await;
    }
}

async fn thread_epiphany_retrieval_state(thread: &CodexThread) -> EpiphanyRetrievalState {
    let config = thread.config_snapshot().await;
    let codex_home = thread.codex_home().await;
    epiphany_retrieval_state_for_paths(config.cwd.to_path_buf(), codex_home).await
}
