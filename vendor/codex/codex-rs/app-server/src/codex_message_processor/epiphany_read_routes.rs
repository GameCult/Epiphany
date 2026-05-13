use codex_app_server_protocol::*;
use codex_core::EPIPHANY_RETRIEVAL_DEFAULT_LIMIT;
use codex_core::EPIPHANY_RETRIEVAL_MAX_LIMIT;
use codex_core::EpiphanyDistillInput;
use codex_core::EpiphanyMapProposalInput;
use codex_core::EpiphanyRetrieveQuery;
use codex_core::distill_observation;
use codex_protocol::ThreadId;
use epiphany_codex_bridge::context::map_epiphany_context;
use epiphany_codex_bridge::context::map_epiphany_graph_query;
use epiphany_codex_bridge::jobs::map_epiphany_jobs;
use epiphany_codex_bridge::launch::EPIPHANY_REORIENT_LAUNCH_BINDING_ID;
use epiphany_codex_bridge::launch::epiphany_role_binding_id;
use epiphany_codex_bridge::reorient::map_epiphany_freshness;
use epiphany_codex_bridge::retrieve::map_epiphany_retrieve_response;
use epiphany_codex_bridge::runtime_results::load_epiphany_reorient_result_snapshot;
use epiphany_codex_bridge::runtime_results::load_epiphany_role_result_snapshot;
use epiphany_codex_bridge::view::EpiphanyViewResponseInput;
use epiphany_codex_bridge::view::default_epiphany_view_lenses;
use epiphany_codex_bridge::view::epiphany_view_needs_jobs;
use epiphany_codex_bridge::view::epiphany_view_needs_pressure;
use epiphany_codex_bridge::view::epiphany_view_needs_reorientation_inputs;
use epiphany_codex_bridge::view::epiphany_view_needs_runtime_store;
use epiphany_codex_bridge::view::map_epiphany_view_response;

use super::CodexMessageProcessor;
use super::ConnectionRequestId;
use super::ThreadReadViewError;
use super::epiphany_freshness_watcher_snapshot;
use super::latest_token_usage_info_from_rollout_path;

impl CodexMessageProcessor {
    pub(super) async fn thread_epiphany_view(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyViewParams,
    ) {
        let ThreadEpiphanyViewParams { thread_id, lenses } = params;
        let lenses = if lenses.is_empty() {
            default_epiphany_view_lenses()
        } else {
            lenses
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

        let needs_jobs = epiphany_view_needs_jobs(&lenses);
        let needs_reorientation_inputs = epiphany_view_needs_reorientation_inputs(&lenses);
        let needs_pressure = epiphany_view_needs_pressure(&lenses);
        let retrieval_override = if (needs_jobs || needs_reorientation_inputs)
            && thread
                .epiphany_state
                .as_ref()
                .and_then(|state| state.retrieval.as_ref())
                .is_none()
        {
            if let Some(loaded_thread) = loaded_thread.as_ref() {
                Some(loaded_thread.epiphany_retrieval_state().await)
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
        let runtime_store_path = if epiphany_view_needs_runtime_store(&lenses) {
            if let Some(loaded_thread) = loaded_thread.as_ref() {
                Some(loaded_thread.epiphany_runtime_spine_store_path().await)
            } else {
                None
            }
        } else {
            None
        };
        let response = map_epiphany_view_response(EpiphanyViewResponseInput {
            thread_id: thread_id.clone(),
            lenses,
            loaded,
            state: thread.epiphany_state.as_ref(),
            retrieval_override: retrieval_override.as_ref(),
            watcher_snapshot: watcher_snapshot
                .as_ref()
                .map(epiphany_freshness_watcher_snapshot),
            token_usage_info: token_usage_info.as_ref(),
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
        let ThreadEpiphanyRoleResultParams {
            thread_id,
            role_id,
            binding_id,
        } = params;

        let binding_id = match binding_id {
            Some(binding_id) => binding_id,
            None => match epiphany_role_binding_id(role_id) {
                Ok(binding_id) => binding_id.to_string(),
                Err(message) => {
                    self.send_invalid_request_error(request_id, message).await;
                    return;
                }
            },
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
        let source = if loaded_thread.is_some() {
            ThreadEpiphanyRolesSource::Live
        } else {
            ThreadEpiphanyRolesSource::Stored
        };
        let Some(state) = thread.epiphany_state.as_ref() else {
            self.outgoing
                .send_response(
                    request_id,
                    ThreadEpiphanyRoleResultResponse {
                        thread_id: thread_uuid.to_string(),
                        role_id,
                        source,
                        state_status: ThreadEpiphanyReorientStateStatus::Missing,
                        state_revision: None,
                        binding_id,
                        status: ThreadEpiphanyRoleResultStatus::MissingState,
                        job: None,
                        finding: None,
                        note: "No authoritative Epiphany state exists for this thread.".to_string(),
                    },
                )
                .await;
            return;
        };

        if let Err(message) = epiphany_role_binding_id(role_id) {
            self.send_invalid_request_error(request_id, message).await;
            return;
        }

        let runtime_store_path = if let Some(loaded_thread) = loaded_thread.as_ref() {
            Some(loaded_thread.epiphany_runtime_spine_store_path().await)
        } else {
            None
        };
        let job = map_epiphany_jobs(Some(state), None)
            .into_iter()
            .find(|job| job.id == binding_id);
        let (status, finding, note) = load_epiphany_role_result_snapshot(
            state,
            runtime_store_path.as_deref(),
            role_id,
            &binding_id,
        )
        .await;

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyRoleResultResponse {
                    thread_id: thread_uuid.to_string(),
                    role_id,
                    source,
                    state_status: ThreadEpiphanyReorientStateStatus::Ready,
                    state_revision: Some(state.revision),
                    binding_id,
                    status,
                    job,
                    finding,
                    note,
                },
            )
            .await;
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
            Some(loaded_thread.epiphany_retrieval_state().await)
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
        let (state_revision, retrieval, graph, watcher) = map_epiphany_freshness(
            thread.epiphany_state.as_ref(),
            retrieval_override.as_ref(),
            watcher_snapshot
                .as_ref()
                .map(epiphany_freshness_watcher_snapshot),
        );
        let response = ThreadEpiphanyFreshnessResponse {
            thread_id,
            source: if loaded_thread.is_some() {
                ThreadEpiphanyFreshnessSource::Live
            } else {
                ThreadEpiphanyFreshnessSource::Stored
            },
            state_revision,
            retrieval,
            graph,
            watcher,
        };
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

        let (state_status, state_revision, context, missing) =
            map_epiphany_context(thread.epiphany_state.as_ref(), &params);
        let response = ThreadEpiphanyContextResponse {
            thread_id,
            source: if loaded {
                ThreadEpiphanyContextSource::Live
            } else {
                ThreadEpiphanyContextSource::Stored
            },
            state_status,
            state_revision,
            context,
            missing,
        };
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

        let (state_status, state_revision, graph, frontier, checkpoint, matched, missing) =
            map_epiphany_graph_query(thread.epiphany_state.as_ref(), &params.query);
        let response = ThreadEpiphanyGraphQueryResponse {
            thread_id,
            source: if loaded {
                ThreadEpiphanyContextSource::Live
            } else {
                ThreadEpiphanyContextSource::Stored
            },
            state_status,
            state_revision,
            graph,
            frontier,
            checkpoint,
            matched,
            missing,
        };
        self.outgoing.send_response(request_id, response).await;
    }

    pub(super) async fn thread_epiphany_reorient_result(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyReorientResultParams,
    ) {
        let ThreadEpiphanyReorientResultParams {
            thread_id,
            binding_id,
        } = params;
        let binding_id = binding_id.unwrap_or_else(|| EPIPHANY_REORIENT_LAUNCH_BINDING_ID.into());

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

        let source = if loaded_thread.is_some() {
            ThreadEpiphanyReorientSource::Live
        } else {
            ThreadEpiphanyReorientSource::Stored
        };
        let Some(state) = thread.epiphany_state.as_ref() else {
            self.outgoing
                .send_response(
                    request_id,
                    ThreadEpiphanyReorientResultResponse {
                        thread_id: thread_uuid.to_string(),
                        source,
                        state_status: ThreadEpiphanyReorientStateStatus::Missing,
                        state_revision: None,
                        binding_id,
                        status: ThreadEpiphanyReorientResultStatus::MissingState,
                        job: None,
                        finding: None,
                        note: "No authoritative Epiphany state exists for this thread.".to_string(),
                    },
                )
                .await;
            return;
        };

        let state_revision = Some(state.revision);
        let runtime_store_path = if let Some(loaded_thread) = loaded_thread.as_ref() {
            Some(loaded_thread.epiphany_runtime_spine_store_path().await)
        } else {
            None
        };
        let job = map_epiphany_jobs(Some(state), None)
            .into_iter()
            .find(|job| job.id == binding_id);

        let (status, finding, note) = load_epiphany_reorient_result_snapshot(
            Some(state),
            runtime_store_path.as_deref(),
            binding_id.as_str(),
        )
        .await;

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyReorientResultResponse {
                    thread_id: thread_uuid.to_string(),
                    source,
                    state_status: ThreadEpiphanyReorientStateStatus::Ready,
                    state_revision,
                    binding_id,
                    status,
                    job,
                    finding,
                    note,
                },
            )
            .await;
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

        let query = query.trim().to_string();
        if query.is_empty() {
            self.send_invalid_request_error(request_id, "query must not be empty".to_string())
                .await;
            return;
        }

        if matches!(limit, Some(0)) {
            self.send_invalid_request_error(
                request_id,
                "limit must be greater than zero".to_string(),
            )
            .await;
            return;
        }

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

        let limit = limit
            .map(|value| value as usize)
            .unwrap_or(EPIPHANY_RETRIEVAL_DEFAULT_LIMIT)
            .clamp(1, EPIPHANY_RETRIEVAL_MAX_LIMIT);
        let response = match thread
            .epiphany_retrieve(EpiphanyRetrieveQuery {
                query,
                limit,
                path_prefixes,
            })
            .await
            .and_then(map_epiphany_retrieve_response)
        {
            Ok(response) => response,
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
        let ThreadEpiphanyDistillParams {
            thread_id,
            source_kind,
            status,
            text,
            subject,
            evidence_kind,
            code_refs,
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

        let expected_revision = thread
            .epiphany_state()
            .await
            .map(|state| state.revision)
            .unwrap_or(0);
        let proposal = match distill_observation(EpiphanyDistillInput {
            source_kind,
            status,
            text,
            subject,
            evidence_kind,
            code_refs,
        }) {
            Ok(proposal) => proposal,
            Err(err) => {
                self.send_invalid_request_error(
                    request_id,
                    format!("failed to distill Epiphany observation: {err}"),
                )
                .await;
                return;
            }
        };
        let response = ThreadEpiphanyDistillResponse {
            expected_revision,
            patch: ThreadEpiphanyUpdatePatch {
                observations: vec![proposal.observation],
                evidence: vec![proposal.evidence],
                ..Default::default()
            },
        };

        self.outgoing.send_response(request_id, response).await;
    }

    pub(super) async fn thread_epiphany_propose(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyProposeParams,
    ) {
        let ThreadEpiphanyProposeParams {
            thread_id,
            observation_ids,
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

        let state = match thread.epiphany_state().await {
            Some(state) => state,
            None => {
                self.send_invalid_request_error(
                    request_id,
                    format!("thread has no Epiphany state: {thread_uuid}"),
                )
                .await;
                return;
            }
        };
        let expected_revision = state.revision;
        let proposal = match codex_core::propose_map_update(EpiphanyMapProposalInput {
            state,
            observation_ids,
        }) {
            Ok(proposal) => proposal,
            Err(err) => {
                self.send_invalid_request_error(
                    request_id,
                    format!("failed to propose Epiphany map update: {err}"),
                )
                .await;
                return;
            }
        };
        let response = ThreadEpiphanyProposeResponse {
            expected_revision,
            patch: ThreadEpiphanyUpdatePatch {
                observations: vec![proposal.observation],
                evidence: vec![proposal.evidence],
                graphs: Some(proposal.graphs),
                graph_frontier: Some(proposal.graph_frontier),
                churn: Some(proposal.churn),
                ..Default::default()
            },
        };

        self.outgoing.send_response(request_id, response).await;
    }
}
