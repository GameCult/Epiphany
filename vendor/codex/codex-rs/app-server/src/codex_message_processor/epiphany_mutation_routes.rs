use chrono::SecondsFormat;
use chrono::Utc;
use codex_app_server_protocol::*;
use codex_core::EpiphanyJobInterruptRequest;
use codex_core::EpiphanyJobLaunchRequest;
use codex_core::EpiphanyPromotionInput;
use codex_core::EpiphanyStateUpdate;
use codex_core::evaluate_promotion;
use codex_protocol::ThreadId;
use codex_protocol::error::CodexErr;
use codex_protocol::protocol::EpiphanyJobKind as CoreEpiphanyJobKind;
use epiphany_codex_bridge::jobs::epiphany_blocked_state_job;
use epiphany_codex_bridge::jobs::map_epiphany_jobs;
use epiphany_codex_bridge::jobs::map_interrupted_epiphany_job;
use epiphany_codex_bridge::jobs::map_launched_epiphany_job;
use epiphany_codex_bridge::launch::EPIPHANY_REORIENT_LAUNCH_BINDING_ID;
use epiphany_codex_bridge::launch::build_epiphany_reorient_launch_request;
use epiphany_codex_bridge::launch::build_epiphany_role_launch_request;
use epiphany_codex_bridge::launch::epiphany_role_binding_id;
use epiphany_codex_bridge::launch::epiphany_role_label;
use epiphany_codex_bridge::launch::map_core_worker_launch_document;
use epiphany_codex_bridge::mutation::build_reorient_acceptance_update;
use epiphany_codex_bridge::mutation::build_role_acceptance_update;
use epiphany_codex_bridge::mutation::epiphany_job_launch_changed_fields;
use epiphany_codex_bridge::mutation::epiphany_promote_changed_fields;
use epiphany_codex_bridge::mutation::epiphany_update_patch_changed_fields;
use epiphany_codex_bridge::mutation::state_update_from_thread_patch;
use epiphany_codex_bridge::mutation::thread_epiphany_patch_has_state_replacements;
use epiphany_codex_bridge::pressure::map_epiphany_pressure;
use epiphany_codex_bridge::reorient::map_epiphany_freshness;
use epiphany_codex_bridge::reorient::map_epiphany_reorient;
use epiphany_codex_bridge::retrieve::map_epiphany_retrieve_index_summary;
use epiphany_codex_bridge::runtime_results::load_completed_epiphany_reorient_finding;
use epiphany_codex_bridge::runtime_results::load_completed_epiphany_role_finding;
use epiphany_codex_bridge::state::client_visible_live_thread_epiphany_state;
use uuid::Uuid;

use super::CodexMessageProcessor;
use super::ConnectionRequestId;
use super::ThreadReadViewError;
use super::epiphany_freshness_watcher_snapshot;

impl CodexMessageProcessor {
    pub(super) async fn thread_epiphany_role_launch(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyRoleLaunchParams,
    ) {
        let ThreadEpiphanyRoleLaunchParams {
            thread_id,
            role_id,
            expected_revision,
            max_runtime_seconds,
        } = params;

        let thread_uuid = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        let loaded_thread = match self.thread_manager.get_thread(thread_uuid).await {
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
        let state = match loaded_thread.epiphany_state().await {
            Some(state) => state,
            None => {
                self.send_invalid_request_error(
                    request_id,
                    "cannot launch an Epiphany role specialist without authoritative Epiphany state"
                        .to_string(),
                )
                .await;
                return;
            }
        };

        let launch_request = match build_epiphany_role_launch_request(
            &thread_id,
            role_id,
            expected_revision,
            max_runtime_seconds,
            &state,
        ) {
            Ok(request) => request,
            Err(message) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
        };
        let changed_fields = epiphany_job_launch_changed_fields();
        let launched = match loaded_thread.epiphany_launch_job(launch_request).await {
            Ok(launched) => launched,
            Err(CodexErr::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to launch Epiphany role specialist for {thread_uuid}: {err}"),
                )
                .await;
                return;
            }
        };

        let epiphany_state = client_visible_live_thread_epiphany_state(
            loaded_thread.as_ref(),
            launched.epiphany_state,
        )
        .await;
        let job = map_launched_epiphany_job(
            &epiphany_state,
            launched.binding_id.as_str(),
            launched.launcher_job_id.as_str(),
            launched.backend_job_id.as_str(),
            CoreEpiphanyJobKind::Specialist,
            "missing launched role projection",
        );

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyRoleLaunchResponse {
                    thread_id: thread_uuid.to_string(),
                    role_id,
                    revision: epiphany_state.revision,
                    changed_fields: changed_fields.clone(),
                    epiphany_state: epiphany_state.clone(),
                    job,
                },
            )
            .await;
        self.outgoing
            .send_server_notification(ServerNotification::ThreadEpiphanyStateUpdated(
                ThreadEpiphanyStateUpdatedNotification {
                    thread_id: thread_uuid.to_string(),
                    source: ThreadEpiphanyStateUpdatedSource::JobLaunch,
                    revision: epiphany_state.revision,
                    changed_fields,
                    epiphany_state,
                },
            ))
            .await;
    }

    pub(super) async fn thread_epiphany_role_accept(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyRoleAcceptParams,
    ) {
        let ThreadEpiphanyRoleAcceptParams {
            thread_id,
            role_id,
            expected_revision,
            binding_id,
        } = params;

        let default_binding_id = match epiphany_role_binding_id(role_id) {
            Ok(binding_id) => binding_id,
            Err(message) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
        };
        let binding_id = binding_id.unwrap_or_else(|| default_binding_id.into());

        let thread_uuid = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        let loaded_thread = match self.thread_manager.get_thread(thread_uuid).await {
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
        let Some(state) = loaded_thread.epiphany_state().await else {
            self.send_invalid_request_error(
                request_id,
                "cannot accept an Epiphany role finding without authoritative Epiphany state"
                    .to_string(),
            )
            .await;
            return;
        };
        if let Some(expected_revision) = expected_revision
            && state.revision != expected_revision
        {
            self.send_invalid_request_error(
                request_id,
                format!(
                    "epiphany state revision mismatch: expected {expected_revision}, found {}",
                    state.revision
                ),
            )
            .await;
            return;
        }

        let finding = match load_completed_epiphany_role_finding(
            loaded_thread.as_ref(),
            &state,
            role_id,
            &binding_id,
        )
        .await
        {
            Ok(finding) => finding,
            Err(CodexErr::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to accept Epiphany role finding: {err}"),
                )
                .await;
                return;
            }
        };

        let accepted_prefix = epiphany_role_label(role_id);
        let accepted_evidence_id = format!("ev-{accepted_prefix}-{}", Uuid::new_v4());
        let accepted_observation_id = format!("obs-{accepted_prefix}-{}", Uuid::new_v4());
        let acceptance_update = match build_role_acceptance_update(
            role_id,
            &binding_id,
            &finding,
            accepted_evidence_id,
            accepted_observation_id,
            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        ) {
            Ok(update) => update,
            Err(message) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
        };
        let accepted_receipt_id = acceptance_update.accepted_receipt_id.clone();
        let accepted_observation_id = acceptance_update.accepted_observation_id.clone();
        let accepted_evidence_id = acceptance_update.accepted_evidence_id.clone();
        let changed_fields = acceptance_update.changed_fields.clone();
        let patch = acceptance_update.patch;
        let applied_patch = patch.clone();

        let epiphany_state = match loaded_thread
            .epiphany_update_state(state_update_from_thread_patch(expected_revision, patch))
            .await
        {
            Ok(state) => {
                client_visible_live_thread_epiphany_state(loaded_thread.as_ref(), state).await
            }
            Err(CodexErr::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to apply Epiphany role finding: {err}"),
                )
                .await;
                return;
            }
        };

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyRoleAcceptResponse {
                    revision: epiphany_state.revision,
                    changed_fields: changed_fields.clone(),
                    epiphany_state: epiphany_state.clone(),
                    role_id,
                    binding_id: binding_id.clone(),
                    accepted_receipt_id,
                    accepted_observation_id,
                    accepted_evidence_id,
                    applied_patch,
                    finding,
                },
            )
            .await;
        self.outgoing
            .send_server_notification(ServerNotification::ThreadEpiphanyStateUpdated(
                ThreadEpiphanyStateUpdatedNotification {
                    thread_id: thread_uuid.to_string(),
                    source: ThreadEpiphanyStateUpdatedSource::RoleAccept,
                    revision: epiphany_state.revision,
                    changed_fields,
                    epiphany_state,
                },
            ))
            .await;
    }

    pub(super) async fn thread_epiphany_reorient_launch(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyReorientLaunchParams,
    ) {
        let ThreadEpiphanyReorientLaunchParams {
            thread_id,
            expected_revision,
            max_runtime_seconds,
        } = params;

        let thread_uuid = match ThreadId::from_string(&thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(request_id, format!("invalid thread id: {err}"))
                    .await;
                return;
            }
        };

        let loaded_thread = match self.thread_manager.get_thread(thread_uuid).await {
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

        let retrieval_override = loaded_thread.epiphany_retrieval_state().await;
        let config_snapshot = loaded_thread.config_snapshot().await;
        self.epiphany_invalidation_manager
            .ensure_thread_watch(&thread_id, &config_snapshot.cwd)
            .await;
        let watcher_snapshot = self
            .epiphany_invalidation_manager
            .snapshot(&thread_id)
            .await;
        let token_usage_info = loaded_thread.token_usage_info().await;
        let (state_revision, retrieval, graph, watcher) = map_epiphany_freshness(
            thread.epiphany_state.as_ref(),
            Some(&retrieval_override),
            Some(epiphany_freshness_watcher_snapshot(&watcher_snapshot)),
        );
        let pressure = map_epiphany_pressure(token_usage_info.as_ref());
        let (state_status, decision) = map_epiphany_reorient(
            thread.epiphany_state.as_ref(),
            &pressure,
            &retrieval,
            &graph,
            &watcher,
        );

        let Some(state) = thread.epiphany_state.as_ref() else {
            self.send_invalid_request_error(
                request_id,
                format!(
                    "cannot launch a reorientation worker without authoritative Epiphany state: {}",
                    decision.note
                ),
            )
            .await;
            return;
        };
        let Some(checkpoint) = state.investigation_checkpoint.as_ref() else {
            self.send_invalid_request_error(
                request_id,
                format!(
                    "cannot launch a reorientation worker without a durable investigation checkpoint: {}",
                    decision.note
                ),
            )
            .await;
            return;
        };

        let launch_request = build_epiphany_reorient_launch_request(
            &thread_id,
            expected_revision,
            max_runtime_seconds,
            state,
            checkpoint,
            &decision,
        );
        let changed_fields = epiphany_job_launch_changed_fields();
        let launched = match loaded_thread.epiphany_launch_job(launch_request).await {
            Ok(launched) => launched,
            Err(CodexErr::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!(
                        "failed to launch Epiphany reorientation worker for {thread_uuid}: {err}"
                    ),
                )
                .await;
                return;
            }
        };

        let epiphany_state = client_visible_live_thread_epiphany_state(
            loaded_thread.as_ref(),
            launched.epiphany_state,
        )
        .await;
        let job = map_epiphany_jobs(Some(&epiphany_state), None)
            .into_iter()
            .find(|job| job.id == EPIPHANY_REORIENT_LAUNCH_BINDING_ID)
            .unwrap_or_else(|| {
                epiphany_blocked_state_job(
                    EPIPHANY_REORIENT_LAUNCH_BINDING_ID,
                    ThreadEpiphanyJobKind::Specialist,
                    "reorient-guided checkpoint regather",
                    "Launched reorientation worker was not reflected in Epiphany state.",
                )
            });

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyReorientLaunchResponse {
                    thread_id: thread_uuid.to_string(),
                    source: ThreadEpiphanyReorientSource::Live,
                    state_status,
                    state_revision,
                    decision,
                    revision: epiphany_state.revision,
                    changed_fields: changed_fields.clone(),
                    epiphany_state: epiphany_state.clone(),
                    job,
                },
            )
            .await;
        self.outgoing
            .send_server_notification(ServerNotification::ThreadEpiphanyStateUpdated(
                ThreadEpiphanyStateUpdatedNotification {
                    thread_id: thread_uuid.to_string(),
                    source: ThreadEpiphanyStateUpdatedSource::JobLaunch,
                    revision: epiphany_state.revision,
                    changed_fields,
                    epiphany_state,
                },
            ))
            .await;
    }

    pub(super) async fn thread_epiphany_reorient_accept(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyReorientAcceptParams,
    ) {
        let ThreadEpiphanyReorientAcceptParams {
            thread_id,
            expected_revision,
            binding_id,
            update_scratch,
            update_investigation_checkpoint,
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

        let loaded_thread = match self.thread_manager.get_thread(thread_uuid).await {
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
        let Some(state) = loaded_thread.epiphany_state().await else {
            self.send_invalid_request_error(
                request_id,
                "cannot accept a reorientation finding without authoritative Epiphany state"
                    .to_string(),
            )
            .await;
            return;
        };
        if let Some(expected_revision) = expected_revision
            && state.revision != expected_revision
        {
            self.send_invalid_request_error(
                request_id,
                format!(
                    "epiphany state revision mismatch: expected {expected_revision}, found {}",
                    state.revision
                ),
            )
            .await;
            return;
        }

        let finding = match load_completed_epiphany_reorient_finding(
            loaded_thread.as_ref(),
            &state,
            binding_id.as_str(),
        )
        .await
        {
            Ok(finding) => finding,
            Err(CodexErr::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to accept Epiphany reorientation finding: {err}"),
                )
                .await;
                return;
            }
        };

        if update_investigation_checkpoint && state.investigation_checkpoint.is_none() {
            self.send_invalid_request_error(
                request_id,
                "cannot update investigation checkpoint because this thread has no durable checkpoint"
                    .to_string(),
            )
            .await;
            return;
        }

        let accepted_evidence_id = format!("ev-reorient-{}", Uuid::new_v4());
        let accepted_observation_id = format!("obs-reorient-{}", Uuid::new_v4());
        let acceptance_update = match build_reorient_acceptance_update(
            &binding_id,
            &finding,
            accepted_evidence_id,
            accepted_observation_id,
            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
            update_scratch,
            update_investigation_checkpoint,
            state.investigation_checkpoint.clone(),
        ) {
            Ok(update) => update,
            Err(message) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
        };
        let accepted_receipt_id = acceptance_update.accepted_receipt_id.clone();
        let accepted_observation_id = acceptance_update.accepted_observation_id.clone();
        let accepted_evidence_id = acceptance_update.accepted_evidence_id.clone();
        let changed_fields = acceptance_update.changed_fields.clone();

        let epiphany_state = match loaded_thread
            .epiphany_update_state(EpiphanyStateUpdate {
                expected_revision,
                scratch: acceptance_update.scratch,
                investigation_checkpoint: acceptance_update.investigation_checkpoint,
                acceptance_receipts: vec![acceptance_update.receipt],
                observations: vec![acceptance_update.observation],
                evidence: vec![acceptance_update.evidence],
                ..Default::default()
            })
            .await
        {
            Ok(state) => {
                client_visible_live_thread_epiphany_state(loaded_thread.as_ref(), state).await
            }
            Err(CodexErr::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to apply Epiphany reorientation finding: {err}"),
                )
                .await;
                return;
            }
        };

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyReorientAcceptResponse {
                    revision: epiphany_state.revision,
                    changed_fields: changed_fields.clone(),
                    epiphany_state: epiphany_state.clone(),
                    binding_id: binding_id.clone(),
                    accepted_receipt_id,
                    accepted_observation_id,
                    accepted_evidence_id,
                    finding,
                },
            )
            .await;
        self.outgoing
            .send_server_notification(ServerNotification::ThreadEpiphanyStateUpdated(
                ThreadEpiphanyStateUpdatedNotification {
                    thread_id: thread_uuid.to_string(),
                    source: ThreadEpiphanyStateUpdatedSource::ReorientAccept,
                    revision: epiphany_state.revision,
                    changed_fields,
                    epiphany_state,
                },
            ))
            .await;
    }

    pub(super) async fn thread_epiphany_index(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyIndexParams,
    ) {
        let ThreadEpiphanyIndexParams {
            thread_id,
            force_full_rebuild,
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

        let response = match thread
            .epiphany_index(force_full_rebuild)
            .await
            .and_then(map_epiphany_retrieve_index_summary)
            .map(|index_summary| ThreadEpiphanyIndexResponse { index_summary })
        {
            Ok(response) => response,
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to index Epiphany retrieval state for {thread_uuid}: {err}"),
                )
                .await;
                return;
            }
        };

        self.outgoing.send_response(request_id, response).await;
    }

    pub(super) async fn thread_epiphany_promote(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyPromoteParams,
    ) {
        let ThreadEpiphanyPromoteParams {
            thread_id,
            expected_revision,
            patch,
            verifier_evidence,
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

        let decision = evaluate_promotion(EpiphanyPromotionInput {
            has_state_replacements: thread_epiphany_patch_has_state_replacements(&patch),
            active_subgoal_id: patch.active_subgoal_id.clone(),
            subgoals: patch.subgoals.clone(),
            invariants: patch.invariants.clone(),
            graphs: patch.graphs.clone(),
            graph_frontier: patch.graph_frontier.clone(),
            graph_checkpoint: patch.graph_checkpoint.clone(),
            investigation_checkpoint: patch.investigation_checkpoint.clone(),
            churn: patch.churn.clone(),
            observations: patch.observations.clone(),
            evidence: patch.evidence.clone(),
            verifier_evidence: verifier_evidence.clone(),
        });
        if !decision.accepted {
            self.outgoing
                .send_response(
                    request_id,
                    ThreadEpiphanyPromoteResponse {
                        accepted: false,
                        reasons: decision.reasons,
                        revision: None,
                        changed_fields: Vec::new(),
                        epiphany_state: None,
                    },
                )
                .await;
            return;
        }

        let changed_fields = epiphany_promote_changed_fields(&patch);
        let mut patch = patch;
        patch.evidence.push(verifier_evidence);
        let update = state_update_from_thread_patch(expected_revision, patch);
        let epiphany_state = match thread.epiphany_update_state(update).await {
            Ok(epiphany_state) => epiphany_state,
            Err(CodexErr::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to promote Epiphany state update: {err}"),
                )
                .await;
                return;
            }
        };
        let epiphany_state =
            client_visible_live_thread_epiphany_state(thread.as_ref(), epiphany_state).await;
        let response = ThreadEpiphanyPromoteResponse {
            accepted: true,
            reasons: Vec::new(),
            revision: Some(epiphany_state.revision),
            changed_fields: changed_fields.clone(),
            epiphany_state: Some(epiphany_state.clone()),
        };

        self.outgoing.send_response(request_id, response).await;
        self.outgoing
            .send_server_notification(ServerNotification::ThreadEpiphanyStateUpdated(
                ThreadEpiphanyStateUpdatedNotification {
                    thread_id: thread_uuid.to_string(),
                    source: ThreadEpiphanyStateUpdatedSource::Promote,
                    revision: epiphany_state.revision,
                    changed_fields,
                    epiphany_state,
                },
            ))
            .await;
    }

    pub(super) async fn thread_epiphany_update(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyUpdateParams,
    ) {
        let ThreadEpiphanyUpdateParams {
            thread_id,
            expected_revision,
            patch,
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

        let changed_fields = epiphany_update_patch_changed_fields(&patch);
        let update = state_update_from_thread_patch(expected_revision, patch);
        let epiphany_state = match thread.epiphany_update_state(update).await {
            Ok(epiphany_state) => epiphany_state,
            Err(CodexErr::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to update Epiphany state for {thread_uuid}: {err}"),
                )
                .await;
                return;
            }
        };
        let epiphany_state =
            client_visible_live_thread_epiphany_state(thread.as_ref(), epiphany_state).await;
        let response = ThreadEpiphanyUpdateResponse {
            revision: epiphany_state.revision,
            changed_fields: changed_fields.clone(),
            epiphany_state: epiphany_state.clone(),
        };

        self.outgoing.send_response(request_id, response).await;
        self.outgoing
            .send_server_notification(ServerNotification::ThreadEpiphanyStateUpdated(
                ThreadEpiphanyStateUpdatedNotification {
                    thread_id: thread_uuid.to_string(),
                    source: ThreadEpiphanyStateUpdatedSource::Update,
                    revision: epiphany_state.revision,
                    changed_fields,
                    epiphany_state,
                },
            ))
            .await;
    }

    pub(super) async fn thread_epiphany_job_launch(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyJobLaunchParams,
    ) {
        let ThreadEpiphanyJobLaunchParams {
            thread_id,
            expected_revision,
            binding_id,
            kind,
            scope,
            owner_role,
            authority_scope,
            linked_subgoal_ids,
            linked_graph_node_ids,
            instruction,
            launch_document,
            output_contract_id,
            max_runtime_seconds,
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

        let launch_document = map_core_worker_launch_document(launch_document);
        let changed_fields = epiphany_job_launch_changed_fields();
        let launched = match thread
            .epiphany_launch_job(EpiphanyJobLaunchRequest {
                expected_revision,
                binding_id: binding_id.clone(),
                kind,
                scope,
                owner_role,
                authority_scope,
                linked_subgoal_ids,
                linked_graph_node_ids,
                instruction,
                launch_document,
                output_contract_id,
                max_runtime_seconds,
            })
            .await
        {
            Ok(launched) => launched,
            Err(CodexErr::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to launch Epiphany job for {thread_uuid}: {err}"),
                )
                .await;
                return;
            }
        };
        let epiphany_state =
            client_visible_live_thread_epiphany_state(thread.as_ref(), launched.epiphany_state)
                .await;
        let job = map_launched_epiphany_job(
            &epiphany_state,
            launched.binding_id.as_str(),
            launched.launcher_job_id.as_str(),
            launched.backend_job_id.as_str(),
            kind,
            "missing launched job projection",
        );

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyJobLaunchResponse {
                    revision: epiphany_state.revision,
                    changed_fields: changed_fields.clone(),
                    epiphany_state: epiphany_state.clone(),
                    job,
                },
            )
            .await;
        self.outgoing
            .send_server_notification(ServerNotification::ThreadEpiphanyStateUpdated(
                ThreadEpiphanyStateUpdatedNotification {
                    thread_id: thread_uuid.to_string(),
                    source: ThreadEpiphanyStateUpdatedSource::JobLaunch,
                    revision: epiphany_state.revision,
                    changed_fields,
                    epiphany_state,
                },
            ))
            .await;
    }

    pub(super) async fn thread_epiphany_job_interrupt(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyJobInterruptParams,
    ) {
        let ThreadEpiphanyJobInterruptParams {
            thread_id,
            expected_revision,
            binding_id,
            reason,
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

        let changed_fields = vec![ThreadEpiphanyStateUpdatedField::JobBindings];
        let interrupted = match thread
            .epiphany_interrupt_job(EpiphanyJobInterruptRequest {
                expected_revision,
                binding_id: binding_id.clone(),
                reason,
            })
            .await
        {
            Ok(interrupted) => interrupted,
            Err(CodexErr::InvalidRequest(message)) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
            Err(err) => {
                self.send_internal_error(
                    request_id,
                    format!("failed to interrupt Epiphany job for {thread_uuid}: {err}"),
                )
                .await;
                return;
            }
        };
        let epiphany_state =
            client_visible_live_thread_epiphany_state(thread.as_ref(), interrupted.epiphany_state)
                .await;
        let job = map_interrupted_epiphany_job(&epiphany_state, &binding_id);

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyJobInterruptResponse {
                    cancel_requested: interrupted.cancel_requested,
                    interrupted_thread_ids: interrupted.interrupted_thread_ids.clone(),
                    revision: epiphany_state.revision,
                    changed_fields: changed_fields.clone(),
                    epiphany_state: epiphany_state.clone(),
                    job,
                },
            )
            .await;
        self.outgoing
            .send_server_notification(ServerNotification::ThreadEpiphanyStateUpdated(
                ThreadEpiphanyStateUpdatedNotification {
                    thread_id: thread_uuid.to_string(),
                    source: ThreadEpiphanyStateUpdatedSource::JobInterrupt,
                    revision: epiphany_state.revision,
                    changed_fields,
                    epiphany_state,
                },
            ))
            .await;
    }
}
