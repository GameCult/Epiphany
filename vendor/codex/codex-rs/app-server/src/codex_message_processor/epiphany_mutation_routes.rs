use super::*;

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
        let binding_id = launch_request.binding_id.clone();
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
        let job = map_epiphany_jobs(Some(&epiphany_state), None)
            .into_iter()
            .find(|job| job.id == binding_id)
            .unwrap_or_else(|| ThreadEpiphanyJob {
                id: launched.binding_id.clone(),
                kind: ThreadEpiphanyJobKind::Specialist,
                scope: "missing launched role projection".to_string(),
                owner_role: "epiphany-harness".to_string(),
                launcher_job_id: Some(launched.launcher_job_id.clone()),
                authority_scope: None,
                backend_job_id: Some(launched.backend_job_id.clone()),
                status: ThreadEpiphanyJobStatus::Pending,
                items_processed: None,
                items_total: None,
                progress_note: None,
                last_checkpoint_at_unix_seconds: None,
                blocking_reason: None,
                active_thread_ids: Vec::new(),
                linked_subgoal_ids: Vec::new(),
                linked_graph_node_ids: Vec::new(),
            });

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

        let mut patch = match role_id {
            ThreadEpiphanyRoleId::Imagination => {
                let patch = match parse_role_finding_state_patch(&finding) {
                    Ok(patch) => patch,
                    Err(message) => {
                        self.send_invalid_request_error(request_id, message).await;
                        return;
                    }
                };
                let patch_errors = imagination_role_accept_patch_errors(&patch);
                if !patch_errors.is_empty() {
                    self.send_invalid_request_error(
                        request_id,
                        format!(
                            "imagination role state patch is not acceptable: {}",
                            patch_errors.join("; ")
                        ),
                    )
                    .await;
                    return;
                }
                patch
            }
            ThreadEpiphanyRoleId::Modeling => {
                let patch = match parse_role_finding_state_patch(&finding) {
                    Ok(patch) => patch,
                    Err(message) => {
                        self.send_invalid_request_error(request_id, message).await;
                        return;
                    }
                };
                let patch_errors = modeling_role_accept_patch_errors(&patch);
                if !patch_errors.is_empty() {
                    self.send_invalid_request_error(
                        request_id,
                        format!(
                            "modeling role state patch is not acceptable: {}",
                            patch_errors.join("; ")
                        ),
                    )
                    .await;
                    return;
                }
                patch
            }
            ThreadEpiphanyRoleId::Verification => ThreadEpiphanyUpdatePatch::default(),
            ThreadEpiphanyRoleId::Implementation | ThreadEpiphanyRoleId::Reorientation => {
                self.send_invalid_request_error(
                    request_id,
                    format!("role {:?} cannot be accepted through roleAccept", role_id),
                )
                .await;
                return;
            }
        };

        let accepted_prefix = epiphany_role_label(role_id);
        let accepted_evidence_id = format!("ev-{accepted_prefix}-{}", Uuid::new_v4());
        let accepted_observation_id = format!("obs-{accepted_prefix}-{}", Uuid::new_v4());
        let projected_fields = epiphany_update_patch_changed_fields(&patch)
            .into_iter()
            .map(|field| format!("{field:?}"))
            .collect();
        let acceptance_bundle = match build_role_acceptance_bundle(
            &binding_id,
            EpiphanyRoleAcceptanceFinding {
                role_id: map_core_role_result_role_id(role_id),
                verdict: finding.verdict.clone(),
                summary: finding.summary.clone(),
                next_safe_move: finding.next_safe_move.clone(),
                files_inspected: finding.files_inspected.clone(),
                runtime_result_id: role_finding_runtime_result_id(&finding),
                runtime_job_id: role_finding_runtime_job_id(&finding),
                projected_fields,
            },
            accepted_evidence_id,
            accepted_observation_id,
            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
        ) {
            Ok(bundle) => bundle,
            Err(message) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
        };
        let accepted_receipt_id = acceptance_bundle.accepted_receipt_id.clone();
        let accepted_observation_id = acceptance_bundle.accepted_observation_id.clone();
        let accepted_evidence_id = acceptance_bundle.accepted_evidence_id.clone();
        patch.evidence.push(acceptance_bundle.evidence);
        patch.observations.push(acceptance_bundle.observation);
        patch.acceptance_receipts.push(acceptance_bundle.receipt);
        let changed_fields = epiphany_update_patch_changed_fields(&patch);
        let applied_patch = patch.clone();

        let epiphany_state = match loaded_thread
            .epiphany_update_state(EpiphanyStateUpdate {
                expected_revision,
                objective: patch.objective,
                active_subgoal_id: patch.active_subgoal_id,
                subgoals: patch.subgoals,
                invariants: patch.invariants,
                graphs: patch.graphs,
                graph_frontier: patch.graph_frontier,
                graph_checkpoint: patch.graph_checkpoint,
                scratch: patch.scratch,
                investigation_checkpoint: patch.investigation_checkpoint,
                job_bindings: patch.job_bindings,
                acceptance_receipts: patch.acceptance_receipts,
                runtime_links: patch.runtime_links,
                observations: patch.observations,
                evidence: patch.evidence,
                churn: patch.churn,
                mode: patch.mode,
                planning: patch.planning,
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
            Some(&watcher_snapshot),
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
        let acceptance_bundle = match build_reorient_acceptance_bundle(
            &binding_id,
            EpiphanyReorientAcceptanceFinding {
                mode: finding.mode.clone(),
                summary: finding.summary.clone(),
                next_safe_move: finding.next_safe_move.clone(),
                checkpoint_still_valid: finding.checkpoint_still_valid,
                files_inspected: finding.files_inspected.clone(),
                runtime_result_id: reorient_finding_runtime_result_id(&finding),
                runtime_job_id: reorient_finding_runtime_job_id(&finding),
            },
            accepted_evidence_id,
            accepted_observation_id,
            Utc::now().to_rfc3339_opts(SecondsFormat::Secs, true),
            update_scratch,
            update_investigation_checkpoint
                .then(|| state.investigation_checkpoint.clone())
                .flatten(),
        ) {
            Ok(bundle) => bundle,
            Err(message) => {
                self.send_invalid_request_error(request_id, message).await;
                return;
            }
        };
        let accepted_receipt_id = acceptance_bundle.accepted_receipt_id.clone();
        let accepted_observation_id = acceptance_bundle.accepted_observation_id.clone();
        let accepted_evidence_id = acceptance_bundle.accepted_evidence_id.clone();
        let scratch = acceptance_bundle.scratch;
        let investigation_checkpoint = acceptance_bundle.investigation_checkpoint;

        let mut changed_fields = vec![
            ThreadEpiphanyStateUpdatedField::AcceptanceReceipts,
            ThreadEpiphanyStateUpdatedField::Observations,
            ThreadEpiphanyStateUpdatedField::Evidence,
        ];
        if scratch.is_some() {
            changed_fields.push(ThreadEpiphanyStateUpdatedField::Scratch);
        }
        if investigation_checkpoint.is_some() {
            changed_fields.push(ThreadEpiphanyStateUpdatedField::InvestigationCheckpoint);
        }

        let epiphany_state = match loaded_thread
            .epiphany_update_state(EpiphanyStateUpdate {
                expected_revision,
                scratch,
                investigation_checkpoint,
                acceptance_receipts: vec![acceptance_bundle.receipt],
                observations: vec![acceptance_bundle.observation],
                evidence: vec![acceptance_bundle.evidence],
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
        let mut evidence = patch.evidence;
        evidence.push(verifier_evidence);
        let update = EpiphanyStateUpdate {
            expected_revision,
            objective: patch.objective,
            active_subgoal_id: patch.active_subgoal_id,
            subgoals: patch.subgoals,
            invariants: patch.invariants,
            graphs: patch.graphs,
            graph_frontier: patch.graph_frontier,
            graph_checkpoint: patch.graph_checkpoint,
            scratch: patch.scratch,
            investigation_checkpoint: patch.investigation_checkpoint,
            job_bindings: patch.job_bindings,
            acceptance_receipts: patch.acceptance_receipts,
            runtime_links: patch.runtime_links,
            observations: patch.observations,
            evidence,
            churn: patch.churn,
            mode: patch.mode,
            planning: patch.planning,
        };
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
        let update = EpiphanyStateUpdate {
            expected_revision,
            objective: patch.objective,
            active_subgoal_id: patch.active_subgoal_id,
            subgoals: patch.subgoals,
            invariants: patch.invariants,
            graphs: patch.graphs,
            graph_frontier: patch.graph_frontier,
            graph_checkpoint: patch.graph_checkpoint,
            scratch: patch.scratch,
            investigation_checkpoint: patch.investigation_checkpoint,
            job_bindings: patch.job_bindings,
            acceptance_receipts: patch.acceptance_receipts,
            runtime_links: patch.runtime_links,
            observations: patch.observations,
            evidence: patch.evidence,
            churn: patch.churn,
            mode: patch.mode,
            planning: patch.planning,
        };
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
        let job = map_epiphany_jobs(Some(&epiphany_state), None)
            .into_iter()
            .find(|job| job.id == binding_id)
            .unwrap_or_else(|| ThreadEpiphanyJob {
                id: launched.binding_id.clone(),
                kind: map_core_epiphany_job_kind(kind),
                scope: "missing launched job projection".to_string(),
                owner_role: "epiphany-harness".to_string(),
                launcher_job_id: Some(launched.launcher_job_id.clone()),
                authority_scope: None,
                backend_job_id: Some(launched.backend_job_id.clone()),
                status: ThreadEpiphanyJobStatus::Pending,
                items_processed: None,
                items_total: None,
                progress_note: None,
                last_checkpoint_at_unix_seconds: None,
                blocking_reason: None,
                active_thread_ids: Vec::new(),
                linked_subgoal_ids: Vec::new(),
                linked_graph_node_ids: Vec::new(),
            });

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
        let job = map_epiphany_jobs(Some(&epiphany_state), None)
            .into_iter()
            .find(|job| job.id == binding_id)
            .unwrap_or_else(|| {
                epiphany_blocked_state_job(
                    &binding_id,
                    ThreadEpiphanyJobKind::Specialist,
                    "role-scoped specialist work",
                    "Interrupted job binding was not reflected in Epiphany state.",
                )
            });

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
async fn load_completed_epiphany_reorient_finding(
    thread: &CodexThread,
    state: &EpiphanyThreadState,
    binding_id: &str,
) -> CodexResult<ThreadEpiphanyReorientFinding> {
    if let Some(link) = latest_epiphany_runtime_link_for_binding(state, binding_id) {
        let runtime_store_path = thread.epiphany_runtime_spine_store_path().await;
        let (status, finding, _note) = load_epiphany_reorient_result_from_runtime_spine_job(
            link.runtime_job_id.as_str(),
            Some(runtime_store_path.as_path()),
        );
        if status != ThreadEpiphanyReorientResultStatus::Completed {
            return Err(CodexErr::InvalidRequest(format!(
                "cannot accept reorientation result while worker status is {:?}",
                status
            )));
        }
        return finding.ok_or_else(|| {
            CodexErr::InvalidRequest(
                "cannot accept completed reorientation worker because no typed runtime-spine result was recorded"
                    .to_string(),
            )
        });
    }

    if !state
        .job_bindings
        .iter()
        .any(|binding| binding.id == binding_id)
    {
        return Err(CodexErr::InvalidRequest(format!(
            "epiphany reorientation binding {:?} was not found",
            binding_id
        )));
    }

    Err(CodexErr::InvalidRequest(
        "reorientation findings without runtime-spine results are unsupported; accept only typed runtime-spine results"
            .to_string(),
    ))
}

async fn load_completed_epiphany_role_finding(
    thread: &CodexThread,
    state: &EpiphanyThreadState,
    role_id: ThreadEpiphanyRoleId,
    binding_id: &str,
) -> CodexResult<ThreadEpiphanyRoleFinding> {
    if let Some(link) = latest_epiphany_runtime_link_for_binding(state, binding_id) {
        let runtime_store_path = thread.epiphany_runtime_spine_store_path().await;
        let (status, finding, _note) = load_epiphany_role_result_from_runtime_spine_job(
            link.runtime_job_id.as_str(),
            Some(runtime_store_path.as_path()),
            role_id,
        );
        if status != ThreadEpiphanyRoleResultStatus::Completed {
            return Err(CodexErr::InvalidRequest(format!(
                "cannot accept role result while worker status is {:?}",
                status
            )));
        }
        return finding.ok_or_else(|| {
            CodexErr::InvalidRequest(
                "cannot accept completed role worker because no typed runtime-spine result was recorded"
                    .to_string(),
            )
        });
    }

    if !state
        .job_bindings
        .iter()
        .any(|binding| binding.id == binding_id)
    {
        return Err(CodexErr::InvalidRequest(format!(
            "epiphany role binding {:?} was not found",
            binding_id
        )));
    }

    Err(CodexErr::InvalidRequest(
        "role findings without runtime-spine results are unsupported; accept only typed runtime-spine results"
            .to_string(),
    ))
}

fn parse_role_finding_state_patch(
    finding: &ThreadEpiphanyRoleFinding,
) -> Result<ThreadEpiphanyUpdatePatch, String> {
    finding
        .state_patch
        .clone()
        .ok_or_else(|| "completed role finding did not include a reviewable statePatch".to_string())
}

pub(super) fn imagination_role_accept_patch_errors(
    patch: &ThreadEpiphanyUpdatePatch,
) -> Vec<String> {
    imagination_role_state_patch_policy_errors(&core_state_patch_from_protocol(patch))
}

pub(super) fn modeling_role_accept_patch_errors(patch: &ThreadEpiphanyUpdatePatch) -> Vec<String> {
    modeling_role_state_patch_policy_errors(&core_state_patch_from_protocol(patch))
}

pub(super) fn role_finding_summary(finding: &ThreadEpiphanyRoleFinding) -> String {
    let summary = finding
        .summary
        .clone()
        .unwrap_or_else(|| "Role worker returned a structured finding.".to_string());
    if let Some(next_safe_move) = finding.next_safe_move.as_deref() {
        format!("{summary} Next safe move: {next_safe_move}")
    } else {
        summary
    }
}

pub(super) fn reorient_finding_code_refs(
    finding: &ThreadEpiphanyReorientFinding,
) -> Vec<EpiphanyCodeRef> {
    finding
        .files_inspected
        .iter()
        .filter(|path| !path.trim().is_empty())
        .map(|path| EpiphanyCodeRef {
            path: PathBuf::from(path),
            start_line: None,
            end_line: None,
            symbol: None,
            note: Some("Inspected by accepted reorientation worker.".to_string()),
        })
        .collect()
}

pub(super) fn reorient_finding_scratch(
    binding_id: &str,
    finding: &ThreadEpiphanyReorientFinding,
) -> EpiphanyScratchPad {
    let mode = finding.mode.as_deref().unwrap_or("unknown");
    let checkpoint_validity = match finding.checkpoint_still_valid {
        Some(true) => "valid",
        Some(false) => "invalid",
        None => "unknown",
    };
    EpiphanyScratchPad {
        summary: finding.summary.clone(),
        hypothesis: Some(format!(
            "Accepted {mode} reorientation finding from {binding_id}; checkpoint validity is {checkpoint_validity}."
        )),
        next_probe: finding.next_safe_move.clone(),
        notes: vec![format!(
            "Files inspected: {}",
            if finding.files_inspected.is_empty() {
                "none reported".to_string()
            } else {
                finding.files_inspected.join(", ")
            }
        )],
    }
}

pub(super) fn reorient_finding_investigation_checkpoint(
    checkpoint: &EpiphanyInvestigationCheckpoint,
    evidence_id: &str,
    code_refs: &[EpiphanyCodeRef],
    finding: &ThreadEpiphanyReorientFinding,
) -> EpiphanyInvestigationCheckpoint {
    let mut checkpoint = checkpoint.clone();
    checkpoint.summary = finding.summary.clone().or(checkpoint.summary);
    checkpoint.next_action = finding.next_safe_move.clone().or(checkpoint.next_action);
    checkpoint.disposition = EpiphanyInvestigationDisposition::ResumeReady;
    if !checkpoint
        .evidence_ids
        .iter()
        .any(|existing| existing == evidence_id)
    {
        checkpoint.evidence_ids.push(evidence_id.to_string());
    }
    for code_ref in code_refs {
        if !checkpoint
            .code_refs
            .iter()
            .any(|existing| existing.path == code_ref.path)
        {
            checkpoint.code_refs.push(code_ref.clone());
        }
    }
    checkpoint
}

pub(super) fn epiphany_job_launch_changed_fields() -> Vec<ThreadEpiphanyStateUpdatedField> {
    vec![
        ThreadEpiphanyStateUpdatedField::JobBindings,
        ThreadEpiphanyStateUpdatedField::RuntimeLinks,
    ]
}

fn thread_epiphany_patch_has_state_replacements(patch: &ThreadEpiphanyUpdatePatch) -> bool {
    patch.objective.is_some()
        || patch.active_subgoal_id.is_some()
        || patch.subgoals.is_some()
        || patch.invariants.is_some()
        || patch.graphs.is_some()
        || patch.graph_frontier.is_some()
        || patch.graph_checkpoint.is_some()
        || patch.scratch.is_some()
        || patch.investigation_checkpoint.is_some()
        || patch.job_bindings.is_some()
        || !patch.acceptance_receipts.is_empty()
        || !patch.runtime_links.is_empty()
        || patch.churn.is_some()
        || patch.mode.is_some()
        || patch.planning.is_some()
}

pub(super) fn epiphany_update_patch_changed_fields(
    patch: &ThreadEpiphanyUpdatePatch,
) -> Vec<ThreadEpiphanyStateUpdatedField> {
    let mut fields = Vec::new();
    if patch.objective.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::Objective);
    }
    if patch.active_subgoal_id.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::ActiveSubgoalId);
    }
    if patch.subgoals.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::Subgoals);
    }
    if patch.invariants.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::Invariants);
    }
    if patch.graphs.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::Graphs);
    }
    if patch.graph_frontier.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::GraphFrontier);
    }
    if patch.graph_checkpoint.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::GraphCheckpoint);
    }
    if patch.scratch.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::Scratch);
    }
    if patch.investigation_checkpoint.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::InvestigationCheckpoint);
    }
    if patch.job_bindings.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::JobBindings);
    }
    if !patch.acceptance_receipts.is_empty() {
        fields.push(ThreadEpiphanyStateUpdatedField::AcceptanceReceipts);
    }
    if !patch.runtime_links.is_empty() {
        fields.push(ThreadEpiphanyStateUpdatedField::RuntimeLinks);
    }
    if !patch.observations.is_empty() {
        fields.push(ThreadEpiphanyStateUpdatedField::Observations);
    }
    if !patch.evidence.is_empty() {
        fields.push(ThreadEpiphanyStateUpdatedField::Evidence);
    }
    if patch.churn.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::Churn);
    }
    if patch.mode.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::Mode);
    }
    if patch.planning.is_some() {
        fields.push(ThreadEpiphanyStateUpdatedField::Planning);
    }
    fields
}

pub(super) fn epiphany_promote_changed_fields(
    patch: &ThreadEpiphanyUpdatePatch,
) -> Vec<ThreadEpiphanyStateUpdatedField> {
    let mut fields = epiphany_update_patch_changed_fields(patch);
    if !fields.contains(&ThreadEpiphanyStateUpdatedField::Evidence) {
        fields.push(ThreadEpiphanyStateUpdatedField::Evidence);
    }
    fields
}
