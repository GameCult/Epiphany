use codex_app_server_protocol::*;
use codex_core::CodexThread;
use codex_protocol::ThreadId;
use codex_protocol::protocol::EpiphanyRetrievalState;
use codex_protocol::protocol::EpiphanyThreadState;
use epiphany_codex_bridge::cultnet::EpiphanyStateUpdatedField;
use epiphany_codex_bridge::error::EpiphanyBridgeError;
use epiphany_codex_bridge::invalidation::epiphany_freshness_watcher_snapshot;
use epiphany_codex_bridge::mutation_service::apply_thread_epiphany_promote;
use epiphany_codex_bridge::mutation_service::apply_thread_epiphany_reorient_accept;
use epiphany_codex_bridge::mutation_service::apply_thread_epiphany_role_accept;
use epiphany_codex_bridge::mutation_service::apply_thread_epiphany_update;
use epiphany_codex_bridge::mutation_service::interrupt_thread_epiphany_job;
use epiphany_codex_bridge::mutation_service::launch_thread_epiphany_job;
use epiphany_codex_bridge::mutation_service::launch_thread_epiphany_reorient;
use epiphany_codex_bridge::mutation_service::launch_thread_epiphany_role;
use epiphany_codex_bridge::protocol_edge::plan_thread_epiphany_job_interrupt;
use epiphany_codex_bridge::protocol_edge::plan_thread_epiphany_job_launch;
use epiphany_codex_bridge::protocol_edge::plan_thread_epiphany_promote;
use epiphany_codex_bridge::protocol_edge::plan_thread_epiphany_reorient_accept;
use epiphany_codex_bridge::protocol_edge::plan_thread_epiphany_reorient_launch;
use epiphany_codex_bridge::protocol_edge::plan_thread_epiphany_role_accept;
use epiphany_codex_bridge::protocol_edge::plan_thread_epiphany_role_launch;
use epiphany_codex_bridge::protocol_edge::plan_thread_epiphany_update;
use epiphany_codex_bridge::protocol_edge::protocol_patch_from_core;
use epiphany_codex_bridge::protocol_edge::protocol_reorient_finding;
use epiphany_codex_bridge::protocol_edge::protocol_role_finding;
use epiphany_codex_bridge::protocol_edge::protocol_state_updated_fields;
use epiphany_codex_bridge::protocol_edge::protocol_state_updated_notification;
use epiphany_codex_bridge::protocol_edge::thread_epiphany_job_interrupt_output;
use epiphany_codex_bridge::protocol_edge::thread_epiphany_job_launch_output;
use epiphany_codex_bridge::protocol_edge::thread_epiphany_promote_output;
use epiphany_codex_bridge::protocol_edge::thread_epiphany_reorient_launch_output;
use epiphany_codex_bridge::protocol_edge::thread_epiphany_role_launch_output;
use epiphany_codex_bridge::protocol_edge::thread_epiphany_update_output;
use epiphany_codex_bridge::retrieve::epiphany_retrieval_state_for_paths;
use epiphany_codex_bridge::retrieve::index_epiphany_retrieval_for_paths;
use epiphany_codex_bridge::retrieve_protocol::protocol_index_response;
use std::sync::Arc;

use super::CodexMessageProcessor;
use super::ConnectionRequestId;
use super::ThreadReadViewError;
use super::epiphany_thread_host::EpiphanyCodexThreadHost;
use super::epiphany_thread_host::epiphany_token_usage_snapshot;

impl CodexMessageProcessor {
    async fn load_epiphany_thread(
        &self,
        request_id: &ConnectionRequestId,
        thread_id: &str,
    ) -> Option<(ThreadId, Arc<CodexThread>)> {
        let thread_uuid = match ThreadId::from_string(thread_id) {
            Ok(id) => id,
            Err(err) => {
                self.send_invalid_request_error(
                    request_id.clone(),
                    format!("invalid thread id: {err}"),
                )
                .await;
                return None;
            }
        };

        let thread = match self.thread_manager.get_thread(thread_uuid).await {
            Ok(thread) => thread,
            Err(_) => {
                self.send_invalid_request_error(
                    request_id.clone(),
                    format!("thread not loaded: {thread_uuid}"),
                )
                .await;
                return None;
            }
        };

        Some((thread_uuid, thread))
    }

    async fn send_epiphany_state_updated(
        &self,
        thread_uuid: ThreadId,
        source: ThreadEpiphanyStateUpdatedSource,
        changed_fields: Vec<EpiphanyStateUpdatedField>,
        epiphany_state: EpiphanyThreadState,
    ) {
        self.outgoing
            .send_server_notification(ServerNotification::ThreadEpiphanyStateUpdated(
                protocol_state_updated_notification(
                    thread_uuid.to_string(),
                    source,
                    epiphany_state.revision,
                    changed_fields,
                    epiphany_state,
                ),
            ))
            .await;
    }

    pub(super) async fn thread_epiphany_role_launch(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyRoleLaunchParams,
    ) {
        let launch_plan = plan_thread_epiphany_role_launch(params);

        let (thread_uuid, loaded_thread) = match self
            .load_epiphany_thread(&request_id, &launch_plan.thread_id)
            .await
        {
            Some(thread) => thread,
            None => return,
        };
        let host = EpiphanyCodexThreadHost::new(loaded_thread.as_ref());
        let applied = match launch_thread_epiphany_role(
            &host,
            &launch_plan.thread_id,
            launch_plan.core_role_id,
            launch_plan.expected_revision,
            launch_plan.max_runtime_seconds,
        )
        .await
        {
            Ok(applied) => applied,
            Err(EpiphanyBridgeError::InvalidRequest(message)) => {
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
        let output = thread_epiphany_role_launch_output(
            thread_uuid.to_string(),
            launch_plan.protocol_role_id,
            applied,
        );

        self.outgoing
            .send_response(request_id, output.response)
            .await;
        self.send_epiphany_state_updated(
            thread_uuid,
            ThreadEpiphanyStateUpdatedSource::JobLaunch,
            output.changed_fields,
            output.epiphany_state,
        )
        .await;
    }

    pub(super) async fn thread_epiphany_role_accept(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyRoleAcceptParams,
    ) {
        let accept_plan = match plan_thread_epiphany_role_accept(params) {
            Ok(plan) => plan,
            Err(message) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
        };

        let (thread_uuid, loaded_thread) = match self
            .load_epiphany_thread(&request_id, &accept_plan.thread_id)
            .await
        {
            Some(thread) => thread,
            None => return,
        };
        let host = EpiphanyCodexThreadHost::new(loaded_thread.as_ref());
        let applied = match apply_thread_epiphany_role_accept(
            &host,
            accept_plan.core_role_id,
            accept_plan.expected_revision,
            &accept_plan.binding_id,
        )
        .await
        {
            Ok(applied) => applied,
            Err(EpiphanyBridgeError::InvalidRequest(message)) => {
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
        let changed_fields = applied.changed_fields;
        let protocol_changed_fields = protocol_state_updated_fields(changed_fields.clone());
        let epiphany_state = applied.epiphany_state;

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyRoleAcceptResponse {
                    revision: applied.revision,
                    changed_fields: protocol_changed_fields,
                    epiphany_state: epiphany_state.clone(),
                    role_id: accept_plan.protocol_role_id,
                    binding_id: accept_plan.binding_id.clone(),
                    accepted_receipt_id: applied.accepted_receipt_id,
                    accepted_observation_id: applied.accepted_observation_id,
                    accepted_evidence_id: applied.accepted_evidence_id,
                    applied_patch: protocol_patch_from_core(applied.applied_patch),
                    finding: protocol_role_finding(accept_plan.protocol_role_id, applied.finding),
                },
            )
            .await;
        self.send_epiphany_state_updated(
            thread_uuid,
            ThreadEpiphanyStateUpdatedSource::RoleAccept,
            changed_fields,
            epiphany_state,
        )
        .await;
    }

    pub(super) async fn thread_epiphany_reorient_launch(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyReorientLaunchParams,
    ) {
        let launch_plan = plan_thread_epiphany_reorient_launch(params);

        let (thread_uuid, loaded_thread) = match self
            .load_epiphany_thread(&request_id, &launch_plan.thread_id)
            .await
        {
            Some(thread) => thread,
            None => return,
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

        let retrieval_override = thread_epiphany_retrieval_state(loaded_thread.as_ref()).await;
        let config_snapshot = loaded_thread.config_snapshot().await;
        self.epiphany_invalidation_manager
            .ensure_thread_watch(&launch_plan.thread_id, &config_snapshot.cwd)
            .await;
        let watcher_snapshot = self
            .epiphany_invalidation_manager
            .snapshot(&launch_plan.thread_id)
            .await;
        let token_usage_info = loaded_thread.token_usage_info().await;
        let host = EpiphanyCodexThreadHost::new(loaded_thread.as_ref());
        let token_usage_snapshot = epiphany_token_usage_snapshot(token_usage_info.as_ref());
        let applied = match launch_thread_epiphany_reorient(
            &host,
            &launch_plan.thread_id,
            launch_plan.expected_revision,
            launch_plan.max_runtime_seconds,
            thread.epiphany_state.as_ref(),
            Some(&retrieval_override),
            Some(epiphany_freshness_watcher_snapshot(&watcher_snapshot)),
            token_usage_snapshot.as_ref(),
        )
        .await
        {
            Ok(applied) => applied,
            Err(EpiphanyBridgeError::InvalidRequest(message)) => {
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
        let output = thread_epiphany_reorient_launch_output(thread_uuid.to_string(), applied);

        self.outgoing
            .send_response(request_id, output.response)
            .await;
        self.send_epiphany_state_updated(
            thread_uuid,
            ThreadEpiphanyStateUpdatedSource::JobLaunch,
            output.changed_fields,
            output.epiphany_state,
        )
        .await;
    }

    pub(super) async fn thread_epiphany_reorient_accept(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyReorientAcceptParams,
    ) {
        let accept_plan = plan_thread_epiphany_reorient_accept(params);

        let (thread_uuid, loaded_thread) = match self
            .load_epiphany_thread(&request_id, &accept_plan.thread_id)
            .await
        {
            Some(thread) => thread,
            None => return,
        };
        let host = EpiphanyCodexThreadHost::new(loaded_thread.as_ref());
        let applied = match apply_thread_epiphany_reorient_accept(
            &host,
            accept_plan.expected_revision,
            accept_plan.binding_id.as_str(),
            accept_plan.update_scratch,
            accept_plan.update_investigation_checkpoint,
        )
        .await
        {
            Ok(applied) => applied,
            Err(EpiphanyBridgeError::InvalidRequest(message)) => {
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
        let changed_fields = applied.changed_fields;
        let protocol_changed_fields = protocol_state_updated_fields(changed_fields.clone());
        let epiphany_state = applied.epiphany_state;

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyReorientAcceptResponse {
                    revision: applied.revision,
                    changed_fields: protocol_changed_fields,
                    epiphany_state: epiphany_state.clone(),
                    binding_id: accept_plan.binding_id.clone(),
                    accepted_receipt_id: applied.accepted_receipt_id,
                    accepted_observation_id: applied.accepted_observation_id,
                    accepted_evidence_id: applied.accepted_evidence_id,
                    finding: protocol_reorient_finding(applied.finding),
                },
            )
            .await;
        self.send_epiphany_state_updated(
            thread_uuid,
            ThreadEpiphanyStateUpdatedSource::ReorientAccept,
            changed_fields,
            epiphany_state,
        )
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

        let (thread_uuid, thread) = match self.load_epiphany_thread(&request_id, &thread_id).await {
            Some(thread) => thread,
            None => return,
        };

        let config = thread.config_snapshot().await;
        let codex_home = thread.codex_home().await;
        let response = match index_epiphany_retrieval_for_paths(
            config.cwd.to_path_buf(),
            codex_home,
            force_full_rebuild,
        )
        .await
        .and_then(protocol_index_response)
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
        let promote_plan = plan_thread_epiphany_promote(params);

        let (thread_uuid, thread) = match self
            .load_epiphany_thread(&request_id, &promote_plan.thread_id)
            .await
        {
            Some(thread) => thread,
            None => return,
        };

        let host = EpiphanyCodexThreadHost::new(thread.as_ref());
        let applied = match apply_thread_epiphany_promote(
            &host,
            promote_plan.expected_revision,
            promote_plan.patch,
            promote_plan.verifier_evidence,
        )
        .await
        {
            Ok(applied) => applied,
            Err(EpiphanyBridgeError::InvalidRequest(message)) => {
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

        let output = thread_epiphany_promote_output(applied);
        self.outgoing
            .send_response(request_id, output.response)
            .await;
        if let Some(accepted) = output.accepted {
            self.send_epiphany_state_updated(
                thread_uuid,
                ThreadEpiphanyStateUpdatedSource::Promote,
                accepted.changed_fields,
                accepted.epiphany_state,
            )
            .await;
        }
    }

    pub(super) async fn thread_epiphany_update(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyUpdateParams,
    ) {
        let update_plan = plan_thread_epiphany_update(params);

        let (thread_uuid, thread) = match self
            .load_epiphany_thread(&request_id, &update_plan.thread_id)
            .await
        {
            Some(thread) => thread,
            None => return,
        };

        let host = EpiphanyCodexThreadHost::new(thread.as_ref());
        let applied = match apply_thread_epiphany_update(
            &host,
            update_plan.expected_revision,
            update_plan.patch,
        )
        .await
        {
            Ok(applied) => applied,
            Err(EpiphanyBridgeError::InvalidRequest(message)) => {
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
        let output = thread_epiphany_update_output(applied);
        self.outgoing
            .send_response(request_id, output.response)
            .await;
        self.send_epiphany_state_updated(
            thread_uuid,
            ThreadEpiphanyStateUpdatedSource::Update,
            output.changed_fields,
            output.epiphany_state,
        )
        .await;
    }

    pub(super) async fn thread_epiphany_job_launch(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyJobLaunchParams,
    ) {
        let launch_plan = plan_thread_epiphany_job_launch(params);

        let (thread_uuid, thread) = match self
            .load_epiphany_thread(&request_id, &launch_plan.thread_id)
            .await
        {
            Some(thread) => thread,
            None => return,
        };

        let host = EpiphanyCodexThreadHost::new(thread.as_ref());
        let applied = match launch_thread_epiphany_job(
            &host,
            launch_plan.launch_request,
            launch_plan.kind,
            "missing launched job projection",
        )
        .await
        {
            Ok(applied) => applied,
            Err(EpiphanyBridgeError::InvalidRequest(message)) => {
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
        let output = thread_epiphany_job_launch_output(applied);

        self.outgoing
            .send_response(request_id, output.response)
            .await;
        self.send_epiphany_state_updated(
            thread_uuid,
            ThreadEpiphanyStateUpdatedSource::JobLaunch,
            output.changed_fields,
            output.epiphany_state,
        )
        .await;
    }

    pub(super) async fn thread_epiphany_job_interrupt(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyJobInterruptParams,
    ) {
        let interrupt_plan = plan_thread_epiphany_job_interrupt(params);

        let (thread_uuid, thread) = match self
            .load_epiphany_thread(&request_id, &interrupt_plan.thread_id)
            .await
        {
            Some(thread) => thread,
            None => return,
        };

        let host = EpiphanyCodexThreadHost::new(thread.as_ref());
        let applied = match interrupt_thread_epiphany_job(
            &host,
            interrupt_plan.expected_revision,
            &interrupt_plan.binding_id,
            interrupt_plan.reason,
        )
        .await
        {
            Ok(applied) => applied,
            Err(EpiphanyBridgeError::InvalidRequest(message)) => {
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
        let output = thread_epiphany_job_interrupt_output(applied);

        self.outgoing
            .send_response(request_id, output.response)
            .await;
        self.send_epiphany_state_updated(
            thread_uuid,
            ThreadEpiphanyStateUpdatedSource::JobInterrupt,
            output.changed_fields,
            output.epiphany_state,
        )
        .await;
    }
}

async fn thread_epiphany_retrieval_state(thread: &CodexThread) -> EpiphanyRetrievalState {
    let config = thread.config_snapshot().await;
    let codex_home = thread.codex_home().await;
    epiphany_retrieval_state_for_paths(config.cwd.to_path_buf(), codex_home).await
}
