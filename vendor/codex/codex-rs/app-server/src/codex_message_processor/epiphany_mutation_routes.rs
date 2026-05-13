use codex_app_server_protocol::*;
use codex_core::CodexThread;
use codex_core::EpiphanyJobLaunchRequest;
use codex_protocol::ThreadId;
use codex_protocol::error::CodexErr;
use codex_protocol::protocol::EpiphanyThreadState;
use epiphany_codex_bridge::launch::EPIPHANY_REORIENT_LAUNCH_BINDING_ID;
use epiphany_codex_bridge::launch::epiphany_role_binding_id;
use epiphany_codex_bridge::launch::map_core_worker_launch_document;
use epiphany_codex_bridge::mutation::epiphany_state_updated_notification;
use epiphany_codex_bridge::mutation_service::EpiphanyThreadPromoteApplied;
use epiphany_codex_bridge::mutation_service::apply_thread_epiphany_promote;
use epiphany_codex_bridge::mutation_service::apply_thread_epiphany_reorient_accept;
use epiphany_codex_bridge::mutation_service::apply_thread_epiphany_role_accept;
use epiphany_codex_bridge::mutation_service::apply_thread_epiphany_update;
use epiphany_codex_bridge::mutation_service::interrupt_thread_epiphany_job;
use epiphany_codex_bridge::mutation_service::launch_thread_epiphany_job;
use epiphany_codex_bridge::mutation_service::launch_thread_epiphany_reorient;
use epiphany_codex_bridge::mutation_service::launch_thread_epiphany_role;
use epiphany_codex_bridge::retrieve::index_thread_epiphany_retrieval;
use std::sync::Arc;

use super::CodexMessageProcessor;
use super::ConnectionRequestId;
use super::ThreadReadViewError;
use super::epiphany_freshness_watcher_snapshot;

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
        changed_fields: Vec<ThreadEpiphanyStateUpdatedField>,
        epiphany_state: EpiphanyThreadState,
    ) {
        self.outgoing
            .send_server_notification(ServerNotification::ThreadEpiphanyStateUpdated(
                epiphany_state_updated_notification(
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
        let ThreadEpiphanyRoleLaunchParams {
            thread_id,
            role_id,
            expected_revision,
            max_runtime_seconds,
        } = params;

        let (thread_uuid, loaded_thread) =
            match self.load_epiphany_thread(&request_id, &thread_id).await {
                Some(thread) => thread,
                None => return,
            };
        let applied = match launch_thread_epiphany_role(
            loaded_thread.as_ref(),
            &thread_id,
            role_id,
            expected_revision,
            max_runtime_seconds,
        )
        .await
        {
            Ok(applied) => applied,
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
        let changed_fields = applied.changed_fields;
        let epiphany_state = applied.epiphany_state;

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyRoleLaunchResponse {
                    thread_id: thread_uuid.to_string(),
                    role_id,
                    revision: applied.revision,
                    changed_fields: changed_fields.clone(),
                    epiphany_state: epiphany_state.clone(),
                    job: applied.job,
                },
            )
            .await;
        self.send_epiphany_state_updated(
            thread_uuid,
            ThreadEpiphanyStateUpdatedSource::JobLaunch,
            changed_fields,
            epiphany_state,
        )
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

        let (thread_uuid, loaded_thread) =
            match self.load_epiphany_thread(&request_id, &thread_id).await {
                Some(thread) => thread,
                None => return,
            };
        let applied = match apply_thread_epiphany_role_accept(
            loaded_thread.as_ref(),
            role_id,
            expected_revision,
            &binding_id,
        )
        .await
        {
            Ok(applied) => applied,
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
        let changed_fields = applied.changed_fields;
        let epiphany_state = applied.epiphany_state;

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyRoleAcceptResponse {
                    revision: applied.revision,
                    changed_fields: changed_fields.clone(),
                    epiphany_state: epiphany_state.clone(),
                    role_id,
                    binding_id: binding_id.clone(),
                    accepted_receipt_id: applied.accepted_receipt_id,
                    accepted_observation_id: applied.accepted_observation_id,
                    accepted_evidence_id: applied.accepted_evidence_id,
                    applied_patch: applied.applied_patch,
                    finding: applied.finding,
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
        let ThreadEpiphanyReorientLaunchParams {
            thread_id,
            expected_revision,
            max_runtime_seconds,
        } = params;

        let (thread_uuid, loaded_thread) =
            match self.load_epiphany_thread(&request_id, &thread_id).await {
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
        let applied = match launch_thread_epiphany_reorient(
            loaded_thread.as_ref(),
            &thread_id,
            expected_revision,
            max_runtime_seconds,
            thread.epiphany_state.as_ref(),
            Some(&retrieval_override),
            Some(epiphany_freshness_watcher_snapshot(&watcher_snapshot)),
            token_usage_info.as_ref(),
        )
        .await
        {
            Ok(applied) => applied,
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
        let changed_fields = applied.changed_fields;
        let epiphany_state = applied.epiphany_state;

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyReorientLaunchResponse {
                    thread_id: thread_uuid.to_string(),
                    source: applied.source,
                    state_status: applied.state_status,
                    state_revision: applied.state_revision,
                    decision: applied.decision,
                    revision: applied.revision,
                    changed_fields: changed_fields.clone(),
                    epiphany_state: epiphany_state.clone(),
                    job: applied.job,
                },
            )
            .await;
        self.send_epiphany_state_updated(
            thread_uuid,
            ThreadEpiphanyStateUpdatedSource::JobLaunch,
            changed_fields,
            epiphany_state,
        )
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

        let (thread_uuid, loaded_thread) =
            match self.load_epiphany_thread(&request_id, &thread_id).await {
                Some(thread) => thread,
                None => return,
            };
        let applied = match apply_thread_epiphany_reorient_accept(
            loaded_thread.as_ref(),
            expected_revision,
            binding_id.as_str(),
            update_scratch,
            update_investigation_checkpoint,
        )
        .await
        {
            Ok(applied) => applied,
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
        let changed_fields = applied.changed_fields;
        let epiphany_state = applied.epiphany_state;

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyReorientAcceptResponse {
                    revision: applied.revision,
                    changed_fields: changed_fields.clone(),
                    epiphany_state: epiphany_state.clone(),
                    binding_id: binding_id.clone(),
                    accepted_receipt_id: applied.accepted_receipt_id,
                    accepted_observation_id: applied.accepted_observation_id,
                    accepted_evidence_id: applied.accepted_evidence_id,
                    finding: applied.finding,
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

        let response = match index_thread_epiphany_retrieval(thread.as_ref(), force_full_rebuild)
            .await
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

        let (thread_uuid, thread) = match self.load_epiphany_thread(&request_id, &thread_id).await {
            Some(thread) => thread,
            None => return,
        };

        let applied = match apply_thread_epiphany_promote(
            thread.as_ref(),
            expected_revision,
            patch,
            verifier_evidence,
        )
        .await
        {
            Ok(applied) => applied,
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

        let applied = match applied {
            EpiphanyThreadPromoteApplied::Accepted(applied) => applied,
            EpiphanyThreadPromoteApplied::Rejected { reasons } => {
                self.outgoing
                    .send_response(
                        request_id,
                        ThreadEpiphanyPromoteResponse {
                            accepted: false,
                            reasons,
                            revision: None,
                            changed_fields: Vec::new(),
                            epiphany_state: None,
                        },
                    )
                    .await;
                return;
            }
        };
        let changed_fields = applied.changed_fields;
        let epiphany_state = applied.epiphany_state;
        let response = ThreadEpiphanyPromoteResponse {
            accepted: true,
            reasons: Vec::new(),
            revision: Some(applied.revision),
            changed_fields: changed_fields.clone(),
            epiphany_state: Some(epiphany_state.clone()),
        };

        self.outgoing.send_response(request_id, response).await;
        self.send_epiphany_state_updated(
            thread_uuid,
            ThreadEpiphanyStateUpdatedSource::Promote,
            changed_fields,
            epiphany_state,
        )
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

        let applied =
            match apply_thread_epiphany_update(thread.as_ref(), expected_revision, patch).await {
                Ok(applied) => applied,
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
        let changed_fields = applied.changed_fields;
        let epiphany_state = applied.epiphany_state;
        let response = ThreadEpiphanyUpdateResponse {
            revision: applied.revision,
            changed_fields: changed_fields.clone(),
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

        let (thread_uuid, thread) = match self.load_epiphany_thread(&request_id, &thread_id).await {
            Some(thread) => thread,
            None => return,
        };

        let launch_document = map_core_worker_launch_document(launch_document);
        let applied = match launch_thread_epiphany_job(
            thread.as_ref(),
            EpiphanyJobLaunchRequest {
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
            },
            kind,
            "missing launched job projection",
        )
        .await
        {
            Ok(applied) => applied,
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
        let changed_fields = applied.changed_fields;
        let epiphany_state = applied.epiphany_state;

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyJobLaunchResponse {
                    revision: applied.revision,
                    changed_fields: changed_fields.clone(),
                    epiphany_state: epiphany_state.clone(),
                    job: applied.job,
                },
            )
            .await;
        self.send_epiphany_state_updated(
            thread_uuid,
            ThreadEpiphanyStateUpdatedSource::JobLaunch,
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

        let applied = match interrupt_thread_epiphany_job(
            thread.as_ref(),
            expected_revision,
            &binding_id,
            reason,
        )
        .await
        {
            Ok(applied) => applied,
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
        let changed_fields = applied.changed_fields;
        let epiphany_state = applied.epiphany_state;

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyJobInterruptResponse {
                    cancel_requested: applied.cancel_requested,
                    interrupted_thread_ids: applied.interrupted_thread_ids.clone(),
                    revision: applied.revision,
                    changed_fields: changed_fields.clone(),
                    epiphany_state: epiphany_state.clone(),
                    job: applied.job,
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
