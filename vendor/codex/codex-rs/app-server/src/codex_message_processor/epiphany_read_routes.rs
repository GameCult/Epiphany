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
