use codex_app_server_protocol::*;
use codex_core::CodexThread;
use codex_protocol::ThreadId;
use epiphany_state_model::EpiphanyThreadState;
use epiphany_codex_bridge::cultnet::EpiphanyStateUpdatedField;
use epiphany_codex_bridge::error::EpiphanyBridgeError;
use epiphany_codex_bridge::mutation_service::apply_thread_epiphany_update;
use epiphany_codex_bridge::mutation_service::interrupt_thread_epiphany_job;
use epiphany_codex_bridge::protocol_edge::protocol_job_from_surface;
use epiphany_codex_bridge::protocol_edge::protocol_state_updated_fields;
use epiphany_codex_bridge::protocol_edge::protocol_state_updated_notification;
use epiphany_codex_bridge::protocol_edge::protocol_update_patch_to_core;
use std::sync::Arc;

use super::CodexMessageProcessor;
use super::ConnectionRequestId;
use super::epiphany_thread_host::EpiphanyCodexThreadHost;

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

        let (thread_uuid, thread) = match self.load_epiphany_thread(&request_id, &thread_id).await {
            Some(thread) => thread,
            None => return,
        };

        let host = EpiphanyCodexThreadHost::new(thread.as_ref());
        let core_patch = protocol_update_patch_to_core(&patch);
        let applied = match apply_thread_epiphany_update(&host, expected_revision, core_patch).await
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
        let changed_fields = applied.changed_fields;
        let protocol_changed_fields = protocol_state_updated_fields(changed_fields.clone());
        let epiphany_state = applied.epiphany_state;
        let response = ThreadEpiphanyUpdateResponse {
            revision: applied.revision,
            changed_fields: protocol_changed_fields,
            epiphany_state: epiphany_state.clone(),
        };

        self.outgoing.send_response(request_id, response).await;
        self.send_epiphany_state_updated(
            thread_uuid,
            ThreadEpiphanyStateUpdatedSource::Update,
            changed_fields,
            epiphany_state,
        )
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

        let (thread_uuid, thread) = match self.load_epiphany_thread(&request_id, &thread_id).await {
            Some(thread) => thread,
            None => return,
        };

        let host = EpiphanyCodexThreadHost::new(thread.as_ref());
        let applied = match interrupt_thread_epiphany_job(
            &host,
            expected_revision,
            &binding_id,
            reason,
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
        let changed_fields = applied.changed_fields;
        let protocol_changed_fields = protocol_state_updated_fields(changed_fields.clone());
        let epiphany_state = applied.epiphany_state;

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyJobInterruptResponse {
                    cancel_requested: applied.cancel_requested,
                    interrupted_thread_ids: applied.interrupted_thread_ids.clone(),
                    revision: applied.revision,
                    changed_fields: protocol_changed_fields,
                    epiphany_state: epiphany_state.clone(),
                    job: protocol_job_from_surface(applied.job, None, None),
                },
            )
            .await;
        self.send_epiphany_state_updated(
            thread_uuid,
            ThreadEpiphanyStateUpdatedSource::JobInterrupt,
            changed_fields,
            epiphany_state,
        )
        .await;
    }
}
