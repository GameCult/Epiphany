use std::path::Path;

use codex_app_server_protocol::*;
use codex_protocol::protocol::EpiphanyRetrievalState;
use codex_protocol::protocol::EpiphanyThreadState;
use codex_protocol::protocol::TokenUsageInfo as CoreTokenUsageInfo;
use epiphany_core::EpiphanySceneInput;
use epiphany_core::derive_scene;

use crate::context::map_epiphany_context;
use crate::context::map_epiphany_graph_query;
use crate::context::map_epiphany_planning;
use crate::coordinator::derive_epiphany_coordinator_status;
use crate::coordinator::epiphany_reorient_finding_already_accepted;
use crate::coordinator::map_epiphany_coordinator_view;
use crate::coordinator::map_epiphany_crrc_recommendation;
use crate::coordinator::map_epiphany_roles;
use crate::coordinator::render_epiphany_roles_note;
use crate::jobs::map_epiphany_jobs;
use crate::launch::EPIPHANY_REORIENT_LAUNCH_BINDING_ID;
use crate::pressure::map_epiphany_pressure;
use crate::reorient::EpiphanyFreshnessWatcherSnapshot;
use crate::reorient::map_epiphany_freshness;
use crate::reorient::map_epiphany_reorient;
use crate::runtime_results::load_epiphany_reorient_result_snapshot;
use crate::runtime_results::load_epiphany_role_result_snapshot;
use crate::scene::map_core_epiphany_scene_action;
use crate::scene::map_epiphany_scene;

pub fn default_epiphany_view_lenses() -> Vec<ThreadEpiphanyViewLens> {
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
}

pub fn epiphany_view_needs_jobs(lenses: &[ThreadEpiphanyViewLens]) -> bool {
    lenses.contains(&ThreadEpiphanyViewLens::Jobs)
        || lenses.contains(&ThreadEpiphanyViewLens::Roles)
        || lenses.contains(&ThreadEpiphanyViewLens::Crrc)
        || lenses.contains(&ThreadEpiphanyViewLens::Coordinator)
}

pub fn epiphany_view_needs_reorientation_inputs(lenses: &[ThreadEpiphanyViewLens]) -> bool {
    lenses.contains(&ThreadEpiphanyViewLens::Roles)
        || lenses.contains(&ThreadEpiphanyViewLens::Reorient)
        || lenses.contains(&ThreadEpiphanyViewLens::Crrc)
        || lenses.contains(&ThreadEpiphanyViewLens::Coordinator)
}

pub fn epiphany_view_needs_pressure(lenses: &[ThreadEpiphanyViewLens]) -> bool {
    lenses.contains(&ThreadEpiphanyViewLens::Pressure)
        || epiphany_view_needs_reorientation_inputs(lenses)
}

pub fn epiphany_view_needs_runtime_store(lenses: &[ThreadEpiphanyViewLens]) -> bool {
    lenses.contains(&ThreadEpiphanyViewLens::Roles)
        || lenses.contains(&ThreadEpiphanyViewLens::Crrc)
        || lenses.contains(&ThreadEpiphanyViewLens::Coordinator)
}

pub struct EpiphanyFreshnessResponseInput<'a> {
    pub thread_id: String,
    pub loaded: bool,
    pub state: Option<&'a EpiphanyThreadState>,
    pub retrieval_override: Option<&'a EpiphanyRetrievalState>,
    pub watcher_snapshot: Option<EpiphanyFreshnessWatcherSnapshot<'a>>,
}

pub fn map_epiphany_freshness_response(
    input: EpiphanyFreshnessResponseInput<'_>,
) -> ThreadEpiphanyFreshnessResponse {
    let (state_revision, retrieval, graph, watcher) = map_epiphany_freshness(
        input.state,
        input.retrieval_override,
        input.watcher_snapshot,
    );
    ThreadEpiphanyFreshnessResponse {
        thread_id: input.thread_id,
        source: if input.loaded {
            ThreadEpiphanyFreshnessSource::Live
        } else {
            ThreadEpiphanyFreshnessSource::Stored
        },
        state_revision,
        retrieval,
        graph,
        watcher,
    }
}

pub fn map_epiphany_context_response(
    thread_id: String,
    loaded: bool,
    state: Option<&EpiphanyThreadState>,
    params: &ThreadEpiphanyContextParams,
) -> ThreadEpiphanyContextResponse {
    let (state_status, state_revision, context, missing) = map_epiphany_context(state, params);
    ThreadEpiphanyContextResponse {
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
    }
}

pub fn map_epiphany_graph_query_response(
    thread_id: String,
    loaded: bool,
    state: Option<&EpiphanyThreadState>,
    query: &ThreadEpiphanyGraphQuery,
) -> ThreadEpiphanyGraphQueryResponse {
    let (state_status, state_revision, graph, frontier, checkpoint, matched, missing) =
        map_epiphany_graph_query(state, query);
    ThreadEpiphanyGraphQueryResponse {
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
    }
}

pub struct EpiphanyViewResponseInput<'a> {
    pub thread_id: String,
    pub lenses: Vec<ThreadEpiphanyViewLens>,
    pub loaded: bool,
    pub state: Option<&'a EpiphanyThreadState>,
    pub retrieval_override: Option<&'a EpiphanyRetrievalState>,
    pub watcher_snapshot: Option<EpiphanyFreshnessWatcherSnapshot<'a>>,
    pub token_usage_info: Option<&'a CoreTokenUsageInfo>,
    pub runtime_store_path: Option<&'a Path>,
}

pub async fn map_epiphany_view_response(
    input: EpiphanyViewResponseInput<'_>,
) -> ThreadEpiphanyViewResponse {
    let EpiphanyViewResponseInput {
        thread_id,
        lenses,
        loaded,
        state,
        retrieval_override,
        watcher_snapshot,
        token_usage_info,
        runtime_store_path,
    } = input;

    let needs_jobs = epiphany_view_needs_jobs(&lenses);
    let needs_reorientation_inputs = epiphany_view_needs_reorientation_inputs(&lenses);
    let needs_pressure = epiphany_view_needs_pressure(&lenses);
    let pressure = needs_pressure.then(|| map_epiphany_pressure(token_usage_info));
    let freshness = needs_reorientation_inputs
        .then(|| map_epiphany_freshness(state, retrieval_override, watcher_snapshot));
    let (state_revision, reorient_state_status, reorient_decision) =
        if let (Some((state_revision, retrieval, graph, watcher)), Some(pressure)) =
            (freshness.as_ref(), pressure.as_ref())
        {
            let (state_status, decision) =
                map_epiphany_reorient(state, pressure, retrieval, graph, watcher);
            (*state_revision, state_status, Some(decision))
        } else {
            (None, ThreadEpiphanyReorientStateStatus::Missing, None)
        };
    let jobs = if needs_jobs {
        map_epiphany_jobs(state, retrieval_override)
    } else {
        Vec::new()
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
            state,
            runtime_store_path,
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
    let checkpoint_present = state
        .and_then(|state| state.investigation_checkpoint.as_ref())
        .is_some();
    let reorient_finding_accepted = reorient_finding.as_ref().is_some_and(|finding| {
        state.is_some_and(|state| epiphany_reorient_finding_already_accepted(state, finding))
    });
    let recommendation =
        if let (Some(pressure), Some(decision)) = (pressure.as_ref(), reorient_decision.as_ref()) {
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
            state,
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
            let status = derive_epiphany_coordinator_status(
                state,
                runtime_store_path,
                reorient_state_status,
                pressure,
                recommendation,
                roles,
                reorient_decision.as_ref(),
                reorient_result_status,
                reorient_finding.as_ref(),
                checkpoint_present,
            )
            .await;
            Some(map_epiphany_coordinator_view(
                thread_id.clone(),
                loaded,
                reorient_state_status,
                state_revision,
                status,
            ))
        } else {
            None
        }
    } else {
        None
    };

    ThreadEpiphanyViewResponse {
        thread_id: thread_id.clone(),
        scene: lenses
            .contains(&ThreadEpiphanyViewLens::Scene)
            .then(|| map_epiphany_scene(state, loaded, EPIPHANY_REORIENT_LAUNCH_BINDING_ID)),
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
            let (state_status, state_revision, planning, summary) = map_epiphany_planning(state);
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
                    state,
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
    }
}

pub struct EpiphanyRoleResultResponseInput<'a> {
    pub thread_id: String,
    pub role_id: ThreadEpiphanyRoleId,
    pub source: ThreadEpiphanyRolesSource,
    pub binding_id: String,
    pub state: Option<&'a EpiphanyThreadState>,
    pub runtime_store_path: Option<&'a Path>,
}

pub async fn map_epiphany_role_result_response(
    input: EpiphanyRoleResultResponseInput<'_>,
) -> ThreadEpiphanyRoleResultResponse {
    let EpiphanyRoleResultResponseInput {
        thread_id,
        role_id,
        source,
        binding_id,
        state,
        runtime_store_path,
    } = input;
    let Some(state) = state else {
        return ThreadEpiphanyRoleResultResponse {
            thread_id,
            role_id,
            source,
            state_status: ThreadEpiphanyReorientStateStatus::Missing,
            state_revision: None,
            binding_id,
            status: ThreadEpiphanyRoleResultStatus::MissingState,
            job: None,
            finding: None,
            note: "No authoritative Epiphany state exists for this thread.".to_string(),
        };
    };

    let job = map_epiphany_jobs(Some(state), None)
        .into_iter()
        .find(|job| job.id == binding_id);
    let (status, finding, note) =
        load_epiphany_role_result_snapshot(state, runtime_store_path, role_id, &binding_id).await;

    ThreadEpiphanyRoleResultResponse {
        thread_id,
        role_id,
        source,
        state_status: ThreadEpiphanyReorientStateStatus::Ready,
        state_revision: Some(state.revision),
        binding_id,
        status,
        job,
        finding,
        note,
    }
}

pub struct EpiphanyReorientResultResponseInput<'a> {
    pub thread_id: String,
    pub source: ThreadEpiphanyReorientSource,
    pub binding_id: String,
    pub state: Option<&'a EpiphanyThreadState>,
    pub runtime_store_path: Option<&'a Path>,
}

pub async fn map_epiphany_reorient_result_response(
    input: EpiphanyReorientResultResponseInput<'_>,
) -> ThreadEpiphanyReorientResultResponse {
    let EpiphanyReorientResultResponseInput {
        thread_id,
        source,
        binding_id,
        state,
        runtime_store_path,
    } = input;
    let Some(state) = state else {
        return ThreadEpiphanyReorientResultResponse {
            thread_id,
            source,
            state_status: ThreadEpiphanyReorientStateStatus::Missing,
            state_revision: None,
            binding_id,
            status: ThreadEpiphanyReorientResultStatus::MissingState,
            job: None,
            finding: None,
            note: "No authoritative Epiphany state exists for this thread.".to_string(),
        };
    };

    let job = map_epiphany_jobs(Some(state), None)
        .into_iter()
        .find(|job| job.id == binding_id);
    let (status, finding, note) =
        load_epiphany_reorient_result_snapshot(Some(state), runtime_store_path, &binding_id).await;

    ThreadEpiphanyReorientResultResponse {
        thread_id,
        source,
        state_status: ThreadEpiphanyReorientStateStatus::Ready,
        state_revision: Some(state.revision),
        binding_id,
        status,
        job,
        finding,
        note,
    }
}
