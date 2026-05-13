use super::*;

impl CodexMessageProcessor {
    pub(super) async fn thread_epiphany_view(
        &self,
        request_id: ConnectionRequestId,
        params: ThreadEpiphanyViewParams,
    ) {
        let ThreadEpiphanyViewParams { thread_id, lenses } = params;
        let lenses = if lenses.is_empty() {
            vec![
                ThreadEpiphanyViewLens::Scene,
                ThreadEpiphanyViewLens::Jobs,
                ThreadEpiphanyViewLens::Roles,
                ThreadEpiphanyViewLens::Planning,
                ThreadEpiphanyViewLens::Pressure,
                ThreadEpiphanyViewLens::Reorient,
                ThreadEpiphanyViewLens::Crrc,
                ThreadEpiphanyViewLens::Coordinator,
            ]
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

        let needs_jobs = lenses.contains(&ThreadEpiphanyViewLens::Jobs)
            || lenses.contains(&ThreadEpiphanyViewLens::Roles)
            || lenses.contains(&ThreadEpiphanyViewLens::Crrc)
            || lenses.contains(&ThreadEpiphanyViewLens::Coordinator);
        let needs_reorientation_inputs = lenses.contains(&ThreadEpiphanyViewLens::Roles)
            || lenses.contains(&ThreadEpiphanyViewLens::Reorient)
            || lenses.contains(&ThreadEpiphanyViewLens::Crrc)
            || lenses.contains(&ThreadEpiphanyViewLens::Coordinator);
        let needs_pressure =
            lenses.contains(&ThreadEpiphanyViewLens::Pressure) || needs_reorientation_inputs;
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
        let pressure = needs_pressure.then(|| map_epiphany_pressure(token_usage_info.as_ref()));
        let freshness = needs_reorientation_inputs.then(|| {
            map_epiphany_freshness(
                thread.epiphany_state.as_ref(),
                retrieval_override.as_ref(),
                watcher_snapshot.as_ref(),
            )
        });
        let (state_revision, reorient_state_status, reorient_decision) =
            if let (Some((state_revision, retrieval, graph, watcher)), Some(pressure)) =
                (freshness.as_ref(), pressure.as_ref())
            {
                let (state_status, decision) = map_epiphany_reorient(
                    thread.epiphany_state.as_ref(),
                    pressure,
                    retrieval,
                    graph,
                    watcher,
                );
                (*state_revision, state_status, Some(decision))
            } else {
                (None, ThreadEpiphanyReorientStateStatus::Missing, None)
            };
        let jobs = if needs_jobs {
            map_epiphany_jobs(thread.epiphany_state.as_ref(), retrieval_override.as_ref())
        } else {
            Vec::new()
        };
        let runtime_store_path = if lenses.contains(&ThreadEpiphanyViewLens::Roles)
            || lenses.contains(&ThreadEpiphanyViewLens::Crrc)
            || lenses.contains(&ThreadEpiphanyViewLens::Coordinator)
        {
            if let Some(loaded_thread) = loaded_thread.as_ref() {
                Some(loaded_thread.epiphany_runtime_spine_store_path().await)
            } else {
                None
            }
        } else {
            None
        };
        let reorient_job = jobs
            .iter()
            .find(|job| job.id == EPIPHANY_REORIENT_LAUNCH_BINDING_ID)
            .cloned();
        let (reorient_result_status, reorient_finding, reorient_result_note) = if runtime_store_path
            .is_some()
            || lenses.contains(&ThreadEpiphanyViewLens::Crrc)
            || lenses.contains(&ThreadEpiphanyViewLens::Coordinator)
        {
            load_epiphany_reorient_result_snapshot(
                thread.epiphany_state.as_ref(),
                runtime_store_path.as_deref(),
                EPIPHANY_REORIENT_LAUNCH_BINDING_ID,
            )
            .await
        } else {
            (
                ThreadEpiphanyReorientResultStatus::MissingState,
                None,
                "Reorient result was not requested.".to_string(),
            )
        };
        let checkpoint_present = thread
            .epiphany_state
            .as_ref()
            .and_then(|state| state.investigation_checkpoint.as_ref())
            .is_some();
        let reorient_finding_accepted = reorient_finding.as_ref().is_some_and(|finding| {
            thread
                .epiphany_state
                .as_ref()
                .is_some_and(|state| epiphany_reorient_finding_already_accepted(state, finding))
        });
        let recommendation = if let (Some(pressure), Some(decision)) =
            (pressure.as_ref(), reorient_decision.as_ref())
        {
            Some(map_epiphany_crrc_recommendation(
                loaded,
                reorient_state_status,
                pressure,
                decision,
                reorient_result_status,
                checkpoint_present,
                reorient_finding.is_some(),
                reorient_finding_accepted,
            ))
        } else {
            None
        };
        let roles = if let (Some(pressure), Some(decision), Some(recommendation)) = (
            pressure.as_ref(),
            reorient_decision.as_ref(),
            recommendation.as_ref(),
        ) {
            Some(map_epiphany_roles(
                thread.epiphany_state.as_ref(),
                &jobs,
                decision,
                pressure,
                recommendation,
                reorient_result_status,
                reorient_job.as_ref(),
            ))
        } else {
            None
        };
        let coordinator_response = if lenses.contains(&ThreadEpiphanyViewLens::Coordinator) {
            if let (Some(pressure), Some(recommendation), Some(roles)) =
                (pressure.as_ref(), recommendation.as_ref(), roles.clone())
            {
                let (modeling_result_status, modeling_finding, _) =
                    if let Some(state) = thread.epiphany_state.as_ref() {
                        load_epiphany_role_result_snapshot(
                            state,
                            runtime_store_path.as_deref(),
                            ThreadEpiphanyRoleId::Modeling,
                            EPIPHANY_MODELING_ROLE_BINDING_ID,
                        )
                        .await
                    } else {
                        (
                            ThreadEpiphanyRoleResultStatus::MissingState,
                            None,
                            "No authoritative Epiphany state exists for this thread.".to_string(),
                        )
                    };
                let modeling_result_accepted = modeling_finding.as_ref().is_some_and(|finding| {
                    thread
                        .epiphany_state
                        .as_ref()
                        .is_some_and(|state| epiphany_role_finding_already_accepted(state, finding))
                });
                let modeling_result_reviewable = modeling_finding
                    .as_ref()
                    .is_some_and(epiphany_modeling_finding_has_reviewable_state_patch);
                let (verification_result_status, verification_finding, _) =
                    if let Some(state) = thread.epiphany_state.as_ref() {
                        load_epiphany_role_result_snapshot(
                            state,
                            runtime_store_path.as_deref(),
                            ThreadEpiphanyRoleId::Verification,
                            EPIPHANY_VERIFICATION_ROLE_BINDING_ID,
                        )
                        .await
                    } else {
                        (
                            ThreadEpiphanyRoleResultStatus::MissingState,
                            None,
                            "No authoritative Epiphany state exists for this thread.".to_string(),
                        )
                    };
                let verification_result_accepted =
                    verification_finding.as_ref().is_some_and(|finding| {
                        thread.epiphany_state.as_ref().is_some_and(|state| {
                            epiphany_role_finding_already_accepted(state, finding)
                        })
                    });
                let verification_result_covers_current_modeling =
                    thread.epiphany_state.as_ref().is_none_or(|state| {
                        epiphany_verification_finding_covers_current_modeling(
                            state,
                            modeling_result_accepted,
                            modeling_finding.as_ref(),
                            verification_finding.as_ref(),
                        )
                    });
                let modeling_result_accepted_after_verification =
                    thread.epiphany_state.as_ref().is_some_and(|state| {
                        role_finding_accepted_after(
                            state,
                            modeling_finding.as_ref(),
                            verification_finding.as_ref(),
                        )
                    });
                let implementation_evidence_after_verification =
                    thread.epiphany_state.as_ref().is_some_and(|state| {
                        implementation_evidence_after_role_finding(
                            state,
                            verification_finding.as_ref(),
                        )
                    });
                let verification_result_cites_implementation_evidence =
                    thread.epiphany_state.as_ref().is_some_and(|state| {
                        epiphany_role_finding_cites_implementation_evidence(
                            state,
                            verification_finding.as_ref(),
                        )
                    });
                let verification_result_allows_implementation = verification_result_accepted
                    && verification_finding
                        .as_ref()
                        .is_some_and(epiphany_verification_finding_allows_implementation);
                let verification_result_needs_evidence = verification_result_accepted
                    && verification_finding
                        .as_ref()
                        .is_some_and(epiphany_verification_finding_needs_evidence);
                let source_signals = ThreadEpiphanyCoordinatorSignals {
                    pressure_level: pressure.level,
                    should_prepare_compaction: pressure.should_prepare_compaction,
                    reorient_action: reorient_decision
                        .as_ref()
                        .map(|decision| decision.action)
                        .unwrap_or(ThreadEpiphanyReorientAction::Resume),
                    crrc_action: recommendation.action,
                    modeling_result_status,
                    verification_result_status,
                    reorient_result_status,
                };
                let coordinator = map_epiphany_coordinator(
                    reorient_state_status,
                    checkpoint_present,
                    pressure,
                    recommendation,
                    &roles,
                    &source_signals,
                    modeling_result_accepted,
                    modeling_result_reviewable,
                    modeling_result_accepted_after_verification,
                    implementation_evidence_after_verification,
                    verification_result_cites_implementation_evidence,
                    verification_result_covers_current_modeling,
                    verification_result_accepted,
                    verification_result_allows_implementation,
                    verification_result_needs_evidence,
                    reorient_finding_accepted,
                );
                let note = render_epiphany_coordinator_note(
                    recommendation.action,
                    pressure.level,
                    modeling_result_status,
                    verification_result_status,
                    reorient_result_status,
                    coordinator.action,
                );
                Some(ThreadEpiphanyViewCoordinator {
                    thread_id: thread_id.clone(),
                    source: if loaded {
                        ThreadEpiphanyRolesSource::Live
                    } else {
                        ThreadEpiphanyRolesSource::Stored
                    },
                    state_status: reorient_state_status,
                    state_revision,
                    action: coordinator.action,
                    target_role: coordinator.target_role,
                    recommended_scene_action: coordinator.recommended_scene_action,
                    requires_review: coordinator.requires_review,
                    can_auto_run: coordinator.can_auto_run,
                    reason: coordinator.reason,
                    source_signals,
                    roles,
                    note,
                })
            } else {
                None
            }
        } else {
            None
        };

        let response = ThreadEpiphanyViewResponse {
            thread_id: thread_id.clone(),
            scene: lenses
                .contains(&ThreadEpiphanyViewLens::Scene)
                .then(|| map_epiphany_scene(thread.epiphany_state.as_ref(), loaded)),
            jobs: if lenses.contains(&ThreadEpiphanyViewLens::Jobs) {
                jobs.clone()
            } else {
                Vec::new()
            },
            roles: lenses.contains(&ThreadEpiphanyViewLens::Roles).then(|| {
                let roles = roles.clone().unwrap_or_default();
                ThreadEpiphanyViewRoles {
                    thread_id: thread_id.clone(),
                    source: if loaded {
                        ThreadEpiphanyRolesSource::Live
                    } else {
                        ThreadEpiphanyRolesSource::Stored
                    },
                    state_status: reorient_state_status,
                    state_revision,
                    note: render_epiphany_roles_note(
                        &roles,
                        reorient_state_status,
                        recommendation
                            .as_ref()
                            .map(|recommendation| recommendation.action)
                            .unwrap_or(ThreadEpiphanyCrrcAction::Continue),
                    ),
                    roles,
                }
            }),
            planning: lenses.contains(&ThreadEpiphanyViewLens::Planning).then(|| {
                let (state_status, state_revision, planning, summary) =
                    map_epiphany_planning(thread.epiphany_state.as_ref());
                ThreadEpiphanyViewPlanning {
                    thread_id: thread_id.clone(),
                    source: if loaded {
                        ThreadEpiphanyContextSource::Live
                    } else {
                        ThreadEpiphanyContextSource::Stored
                    },
                    state_status,
                    state_revision,
                    planning,
                    summary,
                }
            }),
            pressure: lenses
                .contains(&ThreadEpiphanyViewLens::Pressure)
                .then(|| pressure.clone())
                .flatten(),
            reorient: lenses
                .contains(&ThreadEpiphanyViewLens::Reorient)
                .then(|| {
                    reorient_decision
                        .clone()
                        .map(|decision| ThreadEpiphanyViewReorient {
                            thread_id: thread_id.clone(),
                            source: if loaded {
                                ThreadEpiphanyReorientSource::Live
                            } else {
                                ThreadEpiphanyReorientSource::Stored
                            },
                            state_status: reorient_state_status,
                            state_revision,
                            decision,
                        })
                })
                .flatten(),
            crrc: lenses
                .contains(&ThreadEpiphanyViewLens::Crrc)
                .then(|| {
                    let pressure = pressure.clone()?;
                    let decision = reorient_decision.clone()?;
                    let recommendation = recommendation.clone()?;
                    let available_actions = derive_scene(EpiphanySceneInput {
                        state: thread.epiphany_state.as_ref(),
                        loaded,
                        reorient_binding_id: EPIPHANY_REORIENT_LAUNCH_BINDING_ID,
                    })
                    .available_actions
                    .into_iter()
                    .map(map_core_epiphany_scene_action)
                    .collect();
                    let note = format!(
                        "{} Result status: {:?}. {}",
                        recommendation.reason, reorient_result_status, reorient_result_note
                    );
                    Some(ThreadEpiphanyViewCrrc {
                        thread_id: thread_id.clone(),
                        source: if loaded {
                            ThreadEpiphanyReorientSource::Live
                        } else {
                            ThreadEpiphanyReorientSource::Stored
                        },
                        state_status: reorient_state_status,
                        state_revision,
                        pressure,
                        decision,
                        recommendation,
                        reorient_binding_id: EPIPHANY_REORIENT_LAUNCH_BINDING_ID.to_string(),
                        reorient_result_status,
                        reorient_job: reorient_job.clone(),
                        reorient_finding: reorient_finding.clone(),
                        available_actions,
                        note,
                    })
                })
                .flatten(),
            coordinator: coordinator_response,
            lenses,
        };
        self.outgoing.send_response(request_id, response).await;
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
            watcher_snapshot.as_ref(),
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
