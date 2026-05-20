use std::path::Path;

use codex_app_server_protocol::*;
use epiphany_core::EpiphanyContextParams;
use epiphany_core::EpiphanyDistillInput;
use epiphany_core::EpiphanyGraphQuery;
use epiphany_core::EpiphanyMapProposalInput;
use epiphany_core::EpiphanyRoleResultRoleId;
use epiphany_core::EpiphanySceneInput;
use epiphany_core::EpiphanyTokenUsageSnapshot;
use epiphany_core::derive_scene;
use epiphany_core::distill_observation;
use epiphany_core::propose_map_update;
use epiphany_state_model::EpiphanyRetrievalState;
use epiphany_state_model::EpiphanyThreadState;

use crate::context::map_epiphany_context;
use crate::context::map_epiphany_graph_query;
use crate::context::map_epiphany_planning;
use crate::coordinator::derive_epiphany_coordinator_status;
use crate::coordinator::epiphany_reorient_finding_already_accepted;
use crate::coordinator::map_epiphany_coordinator_view;
use crate::coordinator::map_epiphany_crrc_recommendation;
use crate::coordinator::map_epiphany_roles;
use crate::coordinator::map_protocol_crrc_recommendation;
use crate::coordinator::map_protocol_role_board_lanes;
use crate::coordinator::render_epiphany_roles_note;
use crate::cultnet::EpiphanyFreshnessSurface;
use crate::cultnet::EpiphanySurfaceSource;
use crate::jobs::map_core_epiphany_job_view;
use crate::jobs::map_epiphany_jobs;
use crate::launch::EPIPHANY_REORIENT_LAUNCH_BINDING_ID;
use crate::pressure::derive_epiphany_pressure;
use crate::pressure::map_epiphany_pressure;
use crate::reorient::EpiphanyFreshnessWatcherSnapshot;
use crate::reorient::derive_epiphany_freshness_view;
use crate::reorient::derive_epiphany_reorient;
use crate::reorient::map_protocol_reorient_decision;
use crate::reorient::map_protocol_reorient_state_status;
use crate::results::map_protocol_reorient_finding;
use crate::results::map_protocol_role_result_role_id;
use crate::runtime_results::load_core_epiphany_reorient_result_snapshot;
use crate::runtime_results::load_core_epiphany_role_result_snapshot;
use crate::runtime_results::map_protocol_reorient_result_status;
use crate::runtime_results::map_protocol_role_result_status;
use crate::scene::map_core_epiphany_scene_action;
use crate::scene::map_epiphany_scene;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EpiphanyViewLens {
    Scene,
    Jobs,
    Roles,
    Planning,
    Pressure,
    Reorient,
    Crrc,
    Coordinator,
}

pub fn default_epiphany_view_lenses() -> Vec<EpiphanyViewLens> {
    vec![
        EpiphanyViewLens::Scene,
        EpiphanyViewLens::Jobs,
        EpiphanyViewLens::Roles,
        EpiphanyViewLens::Planning,
        EpiphanyViewLens::Pressure,
        EpiphanyViewLens::Reorient,
        EpiphanyViewLens::Crrc,
        EpiphanyViewLens::Coordinator,
    ]
}

pub fn map_core_epiphany_view_lenses(lenses: Vec<ThreadEpiphanyViewLens>) -> Vec<EpiphanyViewLens> {
    lenses
        .into_iter()
        .map(map_core_epiphany_view_lens)
        .collect()
}

fn map_core_epiphany_view_lens(lens: ThreadEpiphanyViewLens) -> EpiphanyViewLens {
    match lens {
        ThreadEpiphanyViewLens::Scene => EpiphanyViewLens::Scene,
        ThreadEpiphanyViewLens::Jobs => EpiphanyViewLens::Jobs,
        ThreadEpiphanyViewLens::Roles => EpiphanyViewLens::Roles,
        ThreadEpiphanyViewLens::Planning => EpiphanyViewLens::Planning,
        ThreadEpiphanyViewLens::Pressure => EpiphanyViewLens::Pressure,
        ThreadEpiphanyViewLens::Reorient => EpiphanyViewLens::Reorient,
        ThreadEpiphanyViewLens::Crrc => EpiphanyViewLens::Crrc,
        ThreadEpiphanyViewLens::Coordinator => EpiphanyViewLens::Coordinator,
    }
}

fn map_protocol_epiphany_view_lens(lens: EpiphanyViewLens) -> ThreadEpiphanyViewLens {
    match lens {
        EpiphanyViewLens::Scene => ThreadEpiphanyViewLens::Scene,
        EpiphanyViewLens::Jobs => ThreadEpiphanyViewLens::Jobs,
        EpiphanyViewLens::Roles => ThreadEpiphanyViewLens::Roles,
        EpiphanyViewLens::Planning => ThreadEpiphanyViewLens::Planning,
        EpiphanyViewLens::Pressure => ThreadEpiphanyViewLens::Pressure,
        EpiphanyViewLens::Reorient => ThreadEpiphanyViewLens::Reorient,
        EpiphanyViewLens::Crrc => ThreadEpiphanyViewLens::Crrc,
        EpiphanyViewLens::Coordinator => ThreadEpiphanyViewLens::Coordinator,
    }
}

pub fn epiphany_view_needs_jobs(lenses: &[EpiphanyViewLens]) -> bool {
    lenses.contains(&EpiphanyViewLens::Jobs)
        || lenses.contains(&EpiphanyViewLens::Roles)
        || lenses.contains(&EpiphanyViewLens::Crrc)
        || lenses.contains(&EpiphanyViewLens::Coordinator)
}

pub fn epiphany_view_needs_reorientation_inputs(lenses: &[EpiphanyViewLens]) -> bool {
    lenses.contains(&EpiphanyViewLens::Roles)
        || lenses.contains(&EpiphanyViewLens::Reorient)
        || lenses.contains(&EpiphanyViewLens::Crrc)
        || lenses.contains(&EpiphanyViewLens::Coordinator)
}

pub fn epiphany_view_needs_pressure(lenses: &[EpiphanyViewLens]) -> bool {
    lenses.contains(&EpiphanyViewLens::Pressure) || epiphany_view_needs_reorientation_inputs(lenses)
}

pub fn epiphany_view_needs_runtime_store(lenses: &[EpiphanyViewLens]) -> bool {
    lenses.contains(&EpiphanyViewLens::Roles)
        || lenses.contains(&EpiphanyViewLens::Crrc)
        || lenses.contains(&EpiphanyViewLens::Coordinator)
}

pub struct EpiphanyFreshnessResponseInput<'a> {
    pub thread_id: String,
    pub loaded: bool,
    pub state: Option<&'a EpiphanyThreadState>,
    pub retrieval_override: Option<&'a EpiphanyRetrievalState>,
    pub watcher_snapshot: Option<EpiphanyFreshnessWatcherSnapshot<'a>>,
}

pub fn derive_epiphany_freshness_surface(
    input: EpiphanyFreshnessResponseInput<'_>,
) -> EpiphanyFreshnessSurface {
    let freshness = derive_epiphany_freshness_view(
        input.state,
        input.retrieval_override,
        input.watcher_snapshot,
    );
    EpiphanyFreshnessSurface {
        thread_id: input.thread_id,
        source: if input.loaded {
            EpiphanySurfaceSource::Live
        } else {
            EpiphanySurfaceSource::Stored
        },
        state_revision: freshness.state_revision,
        retrieval: freshness.retrieval,
        graph: freshness.graph,
        watcher: freshness.watcher,
    }
}

pub fn map_epiphany_context_response(
    thread_id: String,
    loaded: bool,
    state: Option<&EpiphanyThreadState>,
    params: &EpiphanyContextParams,
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
    query: &EpiphanyGraphQuery,
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
    pub lenses: Vec<EpiphanyViewLens>,
    pub loaded: bool,
    pub state: Option<&'a EpiphanyThreadState>,
    pub retrieval_override: Option<&'a EpiphanyRetrievalState>,
    pub watcher_snapshot: Option<EpiphanyFreshnessWatcherSnapshot<'a>>,
    pub token_usage_info: Option<&'a EpiphanyTokenUsageSnapshot>,
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
    let core_pressure = needs_pressure.then(|| derive_epiphany_pressure(token_usage_info));
    let pressure = needs_pressure.then(|| map_epiphany_pressure(token_usage_info));
    let core_freshness = needs_reorientation_inputs
        .then(|| derive_epiphany_freshness_view(state, retrieval_override, watcher_snapshot));
    let (state_revision, core_reorient_state_status, core_reorient_decision) =
        if let (Some(freshness), Some(core_pressure)) =
            (core_freshness.as_ref(), core_pressure.as_ref())
        {
            let (state_status, decision) = derive_epiphany_reorient(
                state,
                core_pressure,
                &freshness.retrieval,
                &freshness.graph,
                &freshness.watcher,
            );
            (freshness.state_revision, state_status, Some(decision))
        } else {
            (
                None,
                epiphany_core::EpiphanyReorientStateStatus::Missing,
                None,
            )
        };
    let reorient_state_status = map_protocol_reorient_state_status(core_reorient_state_status);
    let reorient_decision = core_reorient_decision
        .clone()
        .map(map_protocol_reorient_decision);
    let jobs = if needs_jobs {
        map_epiphany_jobs(state, retrieval_override)
    } else {
        Vec::new()
    };
    let reorient_job = jobs
        .iter()
        .find(|job| job.id == EPIPHANY_REORIENT_LAUNCH_BINDING_ID)
        .cloned();
    let reorient_result = if runtime_store_path.is_some()
        || lenses.contains(&EpiphanyViewLens::Crrc)
        || lenses.contains(&EpiphanyViewLens::Coordinator)
    {
        load_core_epiphany_reorient_result_snapshot(
            state,
            runtime_store_path,
            EPIPHANY_REORIENT_LAUNCH_BINDING_ID,
        )
        .await
    } else {
        crate::runtime_results::EpiphanyReorientResultSnapshot {
            status: epiphany_core::EpiphanyCrrcResultStatus::MissingState,
            finding: None,
            note: "Reorient result was not requested.".to_string(),
        }
    };
    let reorient_result_status = map_protocol_reorient_result_status(reorient_result.status);
    let reorient_finding = reorient_result
        .finding
        .clone()
        .map(map_protocol_reorient_finding);
    let reorient_result_note = reorient_result.note.clone();
    let checkpoint_present = state
        .and_then(|state| state.investigation_checkpoint.as_ref())
        .is_some();
    let reorient_finding_accepted = reorient_result.finding.as_ref().is_some_and(|finding| {
        state.is_some_and(|state| epiphany_reorient_finding_already_accepted(state, finding))
    });
    let recommendation = if let (Some(core_pressure), Some(decision)) =
        (core_pressure.as_ref(), core_reorient_decision.as_ref())
    {
        Some(map_epiphany_crrc_recommendation(
            loaded,
            core_reorient_state_status,
            core_pressure,
            decision,
            reorient_result.status,
            checkpoint_present,
            reorient_finding.is_some(),
            reorient_finding_accepted,
        ))
    } else {
        None
    };
    let roles = if let (Some(core_pressure), Some(decision), Some(recommendation)) = (
        core_pressure.as_ref(),
        core_reorient_decision.as_ref(),
        recommendation.as_ref(),
    ) {
        Some(map_epiphany_roles(
            state,
            &jobs,
            decision,
            core_pressure,
            recommendation,
            reorient_result.status,
            reorient_job.as_ref(),
        ))
    } else {
        None
    };
    let coordinator_response = if lenses.contains(&EpiphanyViewLens::Coordinator) {
        if let (Some(core_pressure), Some(recommendation), Some(roles)) = (
            core_pressure.as_ref(),
            recommendation.as_ref(),
            roles.clone(),
        ) {
            let protocol_roles = map_protocol_role_board_lanes(&roles);
            let status = derive_epiphany_coordinator_status(
                state,
                runtime_store_path,
                core_reorient_state_status,
                core_pressure,
                recommendation,
                roles.roles.clone(),
                core_reorient_decision.as_ref(),
                reorient_result.status,
                reorient_result.finding.as_ref(),
                checkpoint_present,
            )
            .await;
            Some(map_epiphany_coordinator_view(
                thread_id.clone(),
                loaded,
                reorient_state_status,
                state_revision,
                status,
                protocol_roles,
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
            .contains(&EpiphanyViewLens::Scene)
            .then(|| map_epiphany_scene(state, loaded, EPIPHANY_REORIENT_LAUNCH_BINDING_ID)),
        jobs: if lenses.contains(&EpiphanyViewLens::Jobs) {
            jobs.iter()
                .cloned()
                .map(map_core_epiphany_job_view)
                .collect()
        } else {
            Vec::new()
        },
        roles: lenses.contains(&EpiphanyViewLens::Roles).then(|| {
            let role_board = roles.clone();
            let protocol_roles = role_board
                .as_ref()
                .map(map_protocol_role_board_lanes)
                .unwrap_or_default();
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
                    role_board
                        .as_ref()
                        .map(|role_board| role_board.roles.as_slice())
                        .unwrap_or(&[]),
                    reorient_state_status,
                    recommendation
                        .as_ref()
                        .map(|recommendation| recommendation.action)
                        .unwrap_or(epiphany_core::EpiphanyCrrcAction::Continue),
                ),
                roles: protocol_roles,
            }
        }),
        planning: lenses.contains(&EpiphanyViewLens::Planning).then(|| {
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
            .contains(&EpiphanyViewLens::Pressure)
            .then(|| pressure.clone())
            .flatten(),
        reorient: lenses
            .contains(&EpiphanyViewLens::Reorient)
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
            .contains(&EpiphanyViewLens::Crrc)
            .then(|| {
                let pressure = pressure.clone()?;
                let decision = reorient_decision.clone()?;
                let recommendation = recommendation.clone()?;
                let protocol_recommendation =
                    map_protocol_crrc_recommendation(recommendation.clone());
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
                    recommendation: protocol_recommendation,
                    reorient_binding_id: EPIPHANY_REORIENT_LAUNCH_BINDING_ID.to_string(),
                    reorient_result_status,
                    reorient_job: reorient_job.clone().map(map_core_epiphany_job_view),
                    reorient_finding: reorient_finding.clone(),
                    available_actions,
                    note,
                })
            })
            .flatten(),
        coordinator: coordinator_response,
        lenses: lenses
            .into_iter()
            .map(map_protocol_epiphany_view_lens)
            .collect(),
    }
}

pub struct EpiphanyRoleResultResponseInput<'a> {
    pub thread_id: String,
    pub role_id: EpiphanyRoleResultRoleId,
    pub source: EpiphanySurfaceSource,
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
    let protocol_role_id = map_protocol_role_result_role_id(role_id);
    let protocol_source = map_protocol_roles_source(source);
    let Some(state) = state else {
        return ThreadEpiphanyRoleResultResponse {
            thread_id,
            role_id: protocol_role_id,
            source: protocol_source,
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
        .find(|job| job.id == binding_id)
        .map(map_core_epiphany_job_view);
    let result =
        load_core_epiphany_role_result_snapshot(state, runtime_store_path, role_id, &binding_id)
            .await;

    ThreadEpiphanyRoleResultResponse {
        thread_id,
        role_id: protocol_role_id,
        source: protocol_source,
        state_status: ThreadEpiphanyReorientStateStatus::Ready,
        state_revision: Some(state.revision),
        binding_id,
        status: map_protocol_role_result_status(result.status),
        job,
        finding: result
            .finding
            .map(|finding| crate::results::map_protocol_role_finding(protocol_role_id, finding)),
        note: result.note,
    }
}

pub struct EpiphanyReorientResultResponseInput<'a> {
    pub thread_id: String,
    pub source: EpiphanySurfaceSource,
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
    let protocol_source = map_protocol_reorient_result_source(source);
    let Some(state) = state else {
        return ThreadEpiphanyReorientResultResponse {
            thread_id,
            source: protocol_source,
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
        .find(|job| job.id == binding_id)
        .map(map_core_epiphany_job_view);
    let result =
        load_core_epiphany_reorient_result_snapshot(Some(state), runtime_store_path, &binding_id)
            .await;

    ThreadEpiphanyReorientResultResponse {
        thread_id,
        source: protocol_source,
        state_status: ThreadEpiphanyReorientStateStatus::Ready,
        state_revision: Some(state.revision),
        binding_id,
        status: map_protocol_reorient_result_status(result.status),
        job,
        finding: result.finding.map(map_protocol_reorient_finding),
        note: result.note,
    }
}

fn map_protocol_roles_source(source: EpiphanySurfaceSource) -> ThreadEpiphanyRolesSource {
    match source {
        EpiphanySurfaceSource::Live => ThreadEpiphanyRolesSource::Live,
        EpiphanySurfaceSource::Stored => ThreadEpiphanyRolesSource::Stored,
    }
}

fn map_protocol_reorient_result_source(
    source: EpiphanySurfaceSource,
) -> ThreadEpiphanyReorientSource {
    match source {
        EpiphanySurfaceSource::Live => ThreadEpiphanyReorientSource::Live,
        EpiphanySurfaceSource::Stored => ThreadEpiphanyReorientSource::Stored,
    }
}

pub fn map_epiphany_distill_response(
    expected_revision: u64,
    input: EpiphanyDistillInput,
) -> std::result::Result<ThreadEpiphanyDistillResponse, String> {
    let proposal = distill_observation(input).map_err(|err| err.to_string())?;

    Ok(ThreadEpiphanyDistillResponse {
        expected_revision,
        patch: ThreadEpiphanyUpdatePatch {
            observations: vec![proposal.observation],
            evidence: vec![proposal.evidence],
            ..Default::default()
        },
    })
}

pub fn map_core_epiphany_distill_input(
    params: ThreadEpiphanyDistillParams,
) -> EpiphanyDistillInput {
    let ThreadEpiphanyDistillParams {
        source_kind,
        status,
        text,
        subject,
        evidence_kind,
        code_refs,
        ..
    } = params;
    EpiphanyDistillInput {
        source_kind,
        status,
        text,
        subject,
        evidence_kind,
        code_refs,
    }
}

pub fn map_epiphany_propose_response(
    state: EpiphanyThreadState,
    observation_ids: Vec<String>,
) -> std::result::Result<ThreadEpiphanyProposeResponse, String> {
    let expected_revision = state.revision;
    let proposal = propose_map_update(EpiphanyMapProposalInput {
        state,
        observation_ids,
    })
    .map_err(|err| err.to_string())?;

    Ok(ThreadEpiphanyProposeResponse {
        expected_revision,
        patch: ThreadEpiphanyUpdatePatch {
            observations: vec![proposal.observation],
            evidence: vec![proposal.evidence],
            graphs: Some(proposal.graphs),
            graph_frontier: Some(proposal.graph_frontier),
            churn: Some(proposal.churn),
            ..Default::default()
        },
    })
}
