use codex_app_server_protocol::*;
use codex_core::CodexThread;
use codex_protocol::ThreadId;
use codex_protocol::protocol::EpiphanyRetrievalState;
use codex_protocol::protocol::EpiphanyThreadState;
use epiphany_codex_bridge::cultnet::EpiphanyJobKind;
use epiphany_codex_bridge::cultnet::EpiphanyJobStatus;
use epiphany_codex_bridge::cultnet::EpiphanyJobView;
use epiphany_codex_bridge::cultnet::EpiphanyReorientAction;
use epiphany_codex_bridge::cultnet::EpiphanyReorientCheckpointStatus;
use epiphany_codex_bridge::cultnet::EpiphanyReorientDecision;
use epiphany_codex_bridge::cultnet::EpiphanyReorientFreshnessStatus;
use epiphany_codex_bridge::cultnet::EpiphanyReorientPressureLevel;
use epiphany_codex_bridge::cultnet::EpiphanyReorientReason;
use epiphany_codex_bridge::cultnet::EpiphanyReorientStateStatus;
use epiphany_codex_bridge::cultnet::EpiphanyStateUpdatedField;
use epiphany_codex_bridge::cultnet::EpiphanySurfaceSource;
use epiphany_codex_bridge::error::EpiphanyBridgeError;
use epiphany_codex_bridge::invalidation::epiphany_freshness_watcher_snapshot;
use epiphany_codex_bridge::launch::EPIPHANY_REORIENT_LAUNCH_BINDING_ID;
use epiphany_codex_bridge::launch::build_epiphany_job_launch_request;
use epiphany_codex_bridge::launch::epiphany_role_binding_id;
use epiphany_codex_bridge::mutation::epiphany_state_updated_notification;
use epiphany_codex_bridge::mutation::map_protocol_state_updated_fields;
use epiphany_codex_bridge::mutation::protocol_patch_from_core;
use epiphany_codex_bridge::mutation_service::EpiphanyThreadPromoteApplied;
use epiphany_codex_bridge::mutation_service::apply_thread_epiphany_promote;
use epiphany_codex_bridge::mutation_service::apply_thread_epiphany_reorient_accept;
use epiphany_codex_bridge::mutation_service::apply_thread_epiphany_role_accept;
use epiphany_codex_bridge::mutation_service::apply_thread_epiphany_update;
use epiphany_codex_bridge::mutation_service::interrupt_thread_epiphany_job;
use epiphany_codex_bridge::mutation_service::launch_thread_epiphany_job;
use epiphany_codex_bridge::mutation_service::launch_thread_epiphany_reorient;
use epiphany_codex_bridge::mutation_service::launch_thread_epiphany_role;
use epiphany_codex_bridge::results::map_core_role_result_role_id;
use epiphany_codex_bridge::results::map_protocol_reorient_finding;
use epiphany_codex_bridge::results::map_protocol_role_finding;
use epiphany_codex_bridge::retrieve::epiphany_retrieval_state_for_paths;
use epiphany_codex_bridge::retrieve::index_epiphany_retrieval_for_paths;
use epiphany_core::EpiphanyReorientWorkerLaunchDocument;
use epiphany_core::EpiphanyRoleStatePatchDocument;
use epiphany_core::EpiphanyRoleWorkerLaunchDocument;
use epiphany_core::EpiphanyWorkerLaunchDocument;
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
        let core_role_id = map_core_role_result_role_id(role_id);

        let (thread_uuid, loaded_thread) =
            match self.load_epiphany_thread(&request_id, &thread_id).await {
                Some(thread) => thread,
                None => return,
            };
        let host = EpiphanyCodexThreadHost::new(loaded_thread.as_ref());
        let applied = match launch_thread_epiphany_role(
            &host,
            &thread_id,
            core_role_id,
            expected_revision,
            max_runtime_seconds,
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
        let changed_fields = applied.changed_fields;
        let protocol_changed_fields = map_protocol_state_updated_fields(changed_fields.clone());
        let epiphany_state = applied.epiphany_state;

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyRoleLaunchResponse {
                    thread_id: thread_uuid.to_string(),
                    role_id,
                    revision: applied.revision,
                    changed_fields: protocol_changed_fields,
                    epiphany_state: epiphany_state.clone(),
                    job: thread_epiphany_job_from_surface(
                        applied.job,
                        Some(applied.launcher_job_id),
                        Some(applied.backend_job_id),
                    ),
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
        let core_role_id = map_core_role_result_role_id(role_id);

        let default_binding_id = match epiphany_role_binding_id(core_role_id) {
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
        let host = EpiphanyCodexThreadHost::new(loaded_thread.as_ref());
        let applied = match apply_thread_epiphany_role_accept(
            &host,
            core_role_id,
            expected_revision,
            &binding_id,
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
        let protocol_changed_fields = map_protocol_state_updated_fields(changed_fields.clone());
        let epiphany_state = applied.epiphany_state;

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyRoleAcceptResponse {
                    revision: applied.revision,
                    changed_fields: protocol_changed_fields,
                    epiphany_state: epiphany_state.clone(),
                    role_id,
                    binding_id: binding_id.clone(),
                    accepted_receipt_id: applied.accepted_receipt_id,
                    accepted_observation_id: applied.accepted_observation_id,
                    accepted_evidence_id: applied.accepted_evidence_id,
                    applied_patch: protocol_patch_from_core(applied.applied_patch),
                    finding: map_protocol_role_finding(role_id, applied.finding),
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

        let retrieval_override = thread_epiphany_retrieval_state(loaded_thread.as_ref()).await;
        let config_snapshot = loaded_thread.config_snapshot().await;
        self.epiphany_invalidation_manager
            .ensure_thread_watch(&thread_id, &config_snapshot.cwd)
            .await;
        let watcher_snapshot = self
            .epiphany_invalidation_manager
            .snapshot(&thread_id)
            .await;
        let token_usage_info = loaded_thread.token_usage_info().await;
        let host = EpiphanyCodexThreadHost::new(loaded_thread.as_ref());
        let token_usage_snapshot = epiphany_token_usage_snapshot(token_usage_info.as_ref());
        let applied = match launch_thread_epiphany_reorient(
            &host,
            &thread_id,
            expected_revision,
            max_runtime_seconds,
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
        let changed_fields = applied.changed_fields;
        let protocol_changed_fields = map_protocol_state_updated_fields(changed_fields.clone());
        let epiphany_state = applied.epiphany_state;

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyReorientLaunchResponse {
                    thread_id: thread_uuid.to_string(),
                    source: thread_epiphany_reorient_source(applied.source),
                    state_status: thread_epiphany_reorient_state_status(applied.state_status),
                    state_revision: applied.state_revision,
                    decision: thread_epiphany_reorient_decision(applied.decision),
                    revision: applied.revision,
                    changed_fields: protocol_changed_fields,
                    epiphany_state: epiphany_state.clone(),
                    job: thread_epiphany_job_from_surface(
                        applied.job,
                        Some(applied.launcher_job_id),
                        Some(applied.backend_job_id),
                    ),
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
        let host = EpiphanyCodexThreadHost::new(loaded_thread.as_ref());
        let applied = match apply_thread_epiphany_reorient_accept(
            &host,
            expected_revision,
            binding_id.as_str(),
            update_scratch,
            update_investigation_checkpoint,
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
        let protocol_changed_fields = map_protocol_state_updated_fields(changed_fields.clone());
        let epiphany_state = applied.epiphany_state;

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyReorientAcceptResponse {
                    revision: applied.revision,
                    changed_fields: protocol_changed_fields,
                    epiphany_state: epiphany_state.clone(),
                    binding_id: binding_id.clone(),
                    accepted_receipt_id: applied.accepted_receipt_id,
                    accepted_observation_id: applied.accepted_observation_id,
                    accepted_evidence_id: applied.accepted_evidence_id,
                    finding: map_protocol_reorient_finding(applied.finding),
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

        let host = EpiphanyCodexThreadHost::new(thread.as_ref());
        let core_patch = thread_epiphany_update_patch_to_core(&patch);
        let applied = match apply_thread_epiphany_promote(
            &host,
            expected_revision,
            core_patch,
            verifier_evidence,
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
        let protocol_changed_fields = map_protocol_state_updated_fields(changed_fields.clone());
        let epiphany_state = applied.epiphany_state;
        let response = ThreadEpiphanyPromoteResponse {
            accepted: true,
            reasons: Vec::new(),
            revision: Some(applied.revision),
            changed_fields: protocol_changed_fields,
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

        let host = EpiphanyCodexThreadHost::new(thread.as_ref());
        let core_patch = thread_epiphany_update_patch_to_core(&patch);
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
        let protocol_changed_fields = map_protocol_state_updated_fields(changed_fields.clone());
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

        let host = EpiphanyCodexThreadHost::new(thread.as_ref());
        let core_launch_document = thread_epiphany_worker_launch_document_to_core(launch_document);
        let applied = match launch_thread_epiphany_job(
            &host,
            build_epiphany_job_launch_request(
                expected_revision,
                binding_id.clone(),
                kind,
                scope,
                owner_role,
                authority_scope,
                linked_subgoal_ids,
                linked_graph_node_ids,
                instruction,
                core_launch_document,
                output_contract_id,
                max_runtime_seconds,
            ),
            kind,
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
        let changed_fields = applied.changed_fields;
        let protocol_changed_fields = map_protocol_state_updated_fields(changed_fields.clone());
        let epiphany_state = applied.epiphany_state;

        self.outgoing
            .send_response(
                request_id,
                ThreadEpiphanyJobLaunchResponse {
                    revision: applied.revision,
                    changed_fields: protocol_changed_fields,
                    epiphany_state: epiphany_state.clone(),
                    job: thread_epiphany_job_from_surface(
                        applied.job,
                        Some(applied.launcher_job_id),
                        Some(applied.backend_job_id),
                    ),
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
        let protocol_changed_fields = map_protocol_state_updated_fields(changed_fields.clone());
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
                    job: thread_epiphany_job_from_surface(applied.job, None, None),
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

fn thread_epiphany_job_from_surface(
    job: EpiphanyJobView,
    launcher_job_id: Option<String>,
    backend_job_id_override: Option<String>,
) -> ThreadEpiphanyJob {
    ThreadEpiphanyJob {
        id: job.id,
        kind: match job.kind {
            EpiphanyJobKind::Indexing => ThreadEpiphanyJobKind::Indexing,
            EpiphanyJobKind::Remap => ThreadEpiphanyJobKind::Remap,
            EpiphanyJobKind::Verification => ThreadEpiphanyJobKind::Verification,
            EpiphanyJobKind::Specialist => ThreadEpiphanyJobKind::Specialist,
        },
        scope: job.scope,
        owner_role: job.owner_role,
        launcher_job_id,
        authority_scope: job.authority_scope,
        backend_job_id: backend_job_id_override.or(job.runtime_job_id),
        status: match job.status {
            EpiphanyJobStatus::Idle => ThreadEpiphanyJobStatus::Idle,
            EpiphanyJobStatus::Needed => ThreadEpiphanyJobStatus::Needed,
            EpiphanyJobStatus::Pending => ThreadEpiphanyJobStatus::Pending,
            EpiphanyJobStatus::Running => ThreadEpiphanyJobStatus::Running,
            EpiphanyJobStatus::Completed => ThreadEpiphanyJobStatus::Completed,
            EpiphanyJobStatus::Failed => ThreadEpiphanyJobStatus::Failed,
            EpiphanyJobStatus::Cancelled => ThreadEpiphanyJobStatus::Cancelled,
            EpiphanyJobStatus::Blocked => ThreadEpiphanyJobStatus::Blocked,
            EpiphanyJobStatus::Unavailable => ThreadEpiphanyJobStatus::Unavailable,
        },
        items_processed: job.items_processed,
        items_total: job.items_total,
        progress_note: job.progress_note,
        last_checkpoint_at_unix_seconds: job.last_checkpoint_at_unix_seconds,
        blocking_reason: job.blocking_reason,
        active_thread_ids: job.active_thread_ids,
        linked_subgoal_ids: job.linked_subgoal_ids,
        linked_graph_node_ids: job.linked_graph_node_ids,
    }
}

fn thread_epiphany_worker_launch_document_to_core(
    document: ThreadEpiphanyWorkerLaunchDocument,
) -> EpiphanyWorkerLaunchDocument {
    match document {
        ThreadEpiphanyWorkerLaunchDocument::Role(document) => EpiphanyWorkerLaunchDocument::Role(
            thread_epiphany_role_worker_launch_document_to_core(document),
        ),
        ThreadEpiphanyWorkerLaunchDocument::Reorient(document) => {
            EpiphanyWorkerLaunchDocument::Reorient(
                thread_epiphany_reorient_worker_launch_document_to_core(document),
            )
        }
    }
}

fn thread_epiphany_update_patch_to_core(
    patch: &ThreadEpiphanyUpdatePatch,
) -> EpiphanyRoleStatePatchDocument {
    EpiphanyRoleStatePatchDocument {
        objective: patch.objective.clone(),
        active_subgoal_id: patch.active_subgoal_id.clone(),
        subgoals: patch.subgoals.clone(),
        invariants: patch.invariants.clone(),
        graphs: patch.graphs.clone(),
        graph_frontier: patch.graph_frontier.clone(),
        graph_checkpoint: patch.graph_checkpoint.clone(),
        scratch: patch.scratch.clone(),
        investigation_checkpoint: patch.investigation_checkpoint.clone(),
        job_bindings: patch.job_bindings.clone(),
        acceptance_receipts: patch.acceptance_receipts.clone(),
        runtime_links: patch.runtime_links.clone(),
        observations: patch.observations.clone(),
        evidence: patch.evidence.clone(),
        churn: patch.churn.clone(),
        mode: patch.mode.clone(),
        planning: patch.planning.clone(),
    }
}

fn thread_epiphany_role_worker_launch_document_to_core(
    document: ThreadEpiphanyRoleWorkerLaunchDocument,
) -> EpiphanyRoleWorkerLaunchDocument {
    EpiphanyRoleWorkerLaunchDocument {
        thread_id: document.thread_id,
        role_id: document.role_id,
        state_revision: document.state_revision,
        objective: document.objective,
        active_subgoal_id: document.active_subgoal_id,
        active_subgoals: document.active_subgoals,
        active_graph_node_ids: document.active_graph_node_ids,
        investigation_checkpoint: document.investigation_checkpoint,
        scratch: document.scratch,
        invariants: document.invariants,
        graphs: document.graphs,
        recent_evidence: document.recent_evidence,
        recent_observations: document.recent_observations,
        graph_frontier: document.graph_frontier,
        graph_checkpoint: document.graph_checkpoint,
        planning: document.planning,
        churn: document.churn,
    }
}

fn thread_epiphany_reorient_worker_launch_document_to_core(
    document: ThreadEpiphanyReorientWorkerLaunchDocument,
) -> EpiphanyReorientWorkerLaunchDocument {
    EpiphanyReorientWorkerLaunchDocument {
        thread_id: document.thread_id,
        mode: document.mode,
        checkpoint_id: document.checkpoint_id,
        checkpoint_kind: document.checkpoint_kind,
        checkpoint_disposition: document.checkpoint_disposition,
        checkpoint_focus: document.checkpoint_focus,
        checkpoint_summary: document.checkpoint_summary,
        checkpoint_next_action: document.checkpoint_next_action,
        checkpoint_open_questions: document.checkpoint_open_questions,
        checkpoint_evidence_ids: document.checkpoint_evidence_ids,
        checkpoint_code_refs: document.checkpoint_code_refs,
        decision_reasons: document.decision_reasons,
        decision_note: document.decision_note,
        pressure_level: document.pressure_level,
        retrieval_status: document.retrieval_status,
        graph_status: document.graph_status,
        watcher_status: document.watcher_status,
        checkpoint_dirty_paths: document.checkpoint_dirty_paths,
        checkpoint_changed_paths: document.checkpoint_changed_paths,
        scratch: document.scratch,
        graphs: document.graphs,
        recent_evidence: document.recent_evidence,
        recent_observations: document.recent_observations,
        active_frontier_node_ids: document.active_frontier_node_ids,
        linked_subgoal_ids: document.linked_subgoal_ids,
        linked_graph_node_ids: document.linked_graph_node_ids,
    }
}

fn thread_epiphany_reorient_state_status(
    status: EpiphanyReorientStateStatus,
) -> ThreadEpiphanyReorientStateStatus {
    match status {
        EpiphanyReorientStateStatus::Missing => ThreadEpiphanyReorientStateStatus::Missing,
        EpiphanyReorientStateStatus::Ready => ThreadEpiphanyReorientStateStatus::Ready,
    }
}

fn thread_epiphany_reorient_source(source: EpiphanySurfaceSource) -> ThreadEpiphanyReorientSource {
    match source {
        EpiphanySurfaceSource::Stored => ThreadEpiphanyReorientSource::Stored,
        EpiphanySurfaceSource::Live => ThreadEpiphanyReorientSource::Live,
    }
}

fn thread_epiphany_reorient_decision(
    decision: EpiphanyReorientDecision,
) -> ThreadEpiphanyReorientDecision {
    ThreadEpiphanyReorientDecision {
        action: match decision.action {
            EpiphanyReorientAction::Resume => ThreadEpiphanyReorientAction::Resume,
            EpiphanyReorientAction::Regather => ThreadEpiphanyReorientAction::Regather,
        },
        checkpoint_status: match decision.checkpoint_status {
            EpiphanyReorientCheckpointStatus::Missing => {
                ThreadEpiphanyReorientCheckpointStatus::Missing
            }
            EpiphanyReorientCheckpointStatus::ResumeReady => {
                ThreadEpiphanyReorientCheckpointStatus::ResumeReady
            }
            EpiphanyReorientCheckpointStatus::RegatherRequired => {
                ThreadEpiphanyReorientCheckpointStatus::RegatherRequired
            }
        },
        checkpoint_id: decision.checkpoint_id,
        pressure_level: match decision.pressure_level {
            EpiphanyReorientPressureLevel::Unknown => ThreadEpiphanyPressureLevel::Unknown,
            EpiphanyReorientPressureLevel::Low => ThreadEpiphanyPressureLevel::Low,
            EpiphanyReorientPressureLevel::Medium => ThreadEpiphanyPressureLevel::Elevated,
            EpiphanyReorientPressureLevel::High => ThreadEpiphanyPressureLevel::High,
            EpiphanyReorientPressureLevel::Critical => ThreadEpiphanyPressureLevel::Critical,
        },
        retrieval_status: thread_epiphany_reorient_retrieval_status(decision.retrieval_status),
        graph_status: thread_epiphany_reorient_graph_status(decision.graph_status),
        watcher_status: thread_epiphany_reorient_watcher_status(decision.watcher_status),
        reasons: decision
            .reasons
            .into_iter()
            .map(thread_epiphany_reorient_reason)
            .collect(),
        checkpoint_dirty_paths: decision.checkpoint_dirty_paths,
        checkpoint_changed_paths: decision.checkpoint_changed_paths,
        active_frontier_node_ids: decision.active_frontier_node_ids,
        next_action: decision.next_action,
        note: decision.note,
    }
}

fn thread_epiphany_reorient_retrieval_status(
    status: EpiphanyReorientFreshnessStatus,
) -> ThreadEpiphanyRetrievalFreshnessStatus {
    match status {
        EpiphanyReorientFreshnessStatus::Unknown => ThreadEpiphanyRetrievalFreshnessStatus::Missing,
        EpiphanyReorientFreshnessStatus::Clean => ThreadEpiphanyRetrievalFreshnessStatus::Ready,
        EpiphanyReorientFreshnessStatus::Dirty => ThreadEpiphanyRetrievalFreshnessStatus::Indexing,
        EpiphanyReorientFreshnessStatus::Stale | EpiphanyReorientFreshnessStatus::Changed => {
            ThreadEpiphanyRetrievalFreshnessStatus::Stale
        }
    }
}

fn thread_epiphany_reorient_graph_status(
    status: EpiphanyReorientFreshnessStatus,
) -> ThreadEpiphanyGraphFreshnessStatus {
    match status {
        EpiphanyReorientFreshnessStatus::Unknown => ThreadEpiphanyGraphFreshnessStatus::Missing,
        EpiphanyReorientFreshnessStatus::Clean => ThreadEpiphanyGraphFreshnessStatus::Ready,
        EpiphanyReorientFreshnessStatus::Dirty
        | EpiphanyReorientFreshnessStatus::Stale
        | EpiphanyReorientFreshnessStatus::Changed => ThreadEpiphanyGraphFreshnessStatus::Stale,
    }
}

fn thread_epiphany_reorient_watcher_status(
    status: EpiphanyReorientFreshnessStatus,
) -> ThreadEpiphanyInvalidationStatus {
    match status {
        EpiphanyReorientFreshnessStatus::Unknown => ThreadEpiphanyInvalidationStatus::Unavailable,
        EpiphanyReorientFreshnessStatus::Clean => ThreadEpiphanyInvalidationStatus::Clean,
        EpiphanyReorientFreshnessStatus::Dirty
        | EpiphanyReorientFreshnessStatus::Stale
        | EpiphanyReorientFreshnessStatus::Changed => ThreadEpiphanyInvalidationStatus::Changed,
    }
}

fn thread_epiphany_reorient_reason(reason: EpiphanyReorientReason) -> ThreadEpiphanyReorientReason {
    match reason {
        EpiphanyReorientReason::MissingState => ThreadEpiphanyReorientReason::MissingState,
        EpiphanyReorientReason::MissingCheckpoint => {
            ThreadEpiphanyReorientReason::MissingCheckpoint
        }
        EpiphanyReorientReason::CheckpointReady => ThreadEpiphanyReorientReason::CheckpointReady,
        EpiphanyReorientReason::CheckpointRequestedRegather => {
            ThreadEpiphanyReorientReason::CheckpointRequestedRegather
        }
        EpiphanyReorientReason::CheckpointPathsDirty => {
            ThreadEpiphanyReorientReason::CheckpointPathsDirty
        }
        EpiphanyReorientReason::CheckpointPathsChanged => {
            ThreadEpiphanyReorientReason::CheckpointPathsChanged
        }
        EpiphanyReorientReason::FrontierChanged => ThreadEpiphanyReorientReason::FrontierChanged,
        EpiphanyReorientReason::UnanchoredCheckpointWhileStateStale => {
            ThreadEpiphanyReorientReason::UnanchoredCheckpointWhileStateStale
        }
    }
}

async fn thread_epiphany_retrieval_state(thread: &CodexThread) -> EpiphanyRetrievalState {
    let config = thread.config_snapshot().await;
    let codex_home = thread.codex_home().await;
    epiphany_retrieval_state_for_paths(config.cwd.to_path_buf(), codex_home).await
}
