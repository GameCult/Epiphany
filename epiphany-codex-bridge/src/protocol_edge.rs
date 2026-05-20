use crate::cultnet::EpiphanyFreshnessSurface;
use crate::cultnet::EpiphanyGraphFreshnessStatus;
use crate::cultnet::EpiphanyInvalidationStatus;
use crate::cultnet::EpiphanyJobKind;
use crate::cultnet::EpiphanyJobStatus;
use crate::cultnet::EpiphanyJobView;
use crate::cultnet::EpiphanyReorientAction;
use crate::cultnet::EpiphanyReorientCheckpointStatus;
use crate::cultnet::EpiphanyReorientDecision;
use crate::cultnet::EpiphanyReorientFreshnessStatus;
use crate::cultnet::EpiphanyReorientPressureLevel;
use crate::cultnet::EpiphanyReorientReason;
use crate::cultnet::EpiphanyReorientStateStatus;
use crate::cultnet::EpiphanyRetrievalFreshnessStatus;
use crate::cultnet::EpiphanySurfaceSource;
use codex_app_server_protocol::ThreadEpiphanyContextParams;
use codex_app_server_protocol::ThreadEpiphanyDistillParams;
use codex_app_server_protocol::ThreadEpiphanyFreshnessResponse;
use codex_app_server_protocol::ThreadEpiphanyFreshnessSource;
use codex_app_server_protocol::ThreadEpiphanyGraphFreshness;
use codex_app_server_protocol::ThreadEpiphanyGraphFreshnessStatus;
use codex_app_server_protocol::ThreadEpiphanyGraphQuery;
use codex_app_server_protocol::ThreadEpiphanyGraphQueryDirection;
use codex_app_server_protocol::ThreadEpiphanyGraphQueryKind;
use codex_app_server_protocol::ThreadEpiphanyInvalidationInput;
use codex_app_server_protocol::ThreadEpiphanyInvalidationStatus;
use codex_app_server_protocol::ThreadEpiphanyJob;
use codex_app_server_protocol::ThreadEpiphanyJobKind;
use codex_app_server_protocol::ThreadEpiphanyJobStatus;
use codex_app_server_protocol::ThreadEpiphanyPressure;
use codex_app_server_protocol::ThreadEpiphanyPressureBasis;
use codex_app_server_protocol::ThreadEpiphanyPressureLevel;
use codex_app_server_protocol::ThreadEpiphanyPressureStatus;
use codex_app_server_protocol::ThreadEpiphanyReorientAction;
use codex_app_server_protocol::ThreadEpiphanyReorientCheckpointStatus;
use codex_app_server_protocol::ThreadEpiphanyReorientDecision;
use codex_app_server_protocol::ThreadEpiphanyReorientFinding;
use codex_app_server_protocol::ThreadEpiphanyReorientReason;
use codex_app_server_protocol::ThreadEpiphanyReorientResultStatus;
use codex_app_server_protocol::ThreadEpiphanyReorientSource;
use codex_app_server_protocol::ThreadEpiphanyReorientStateStatus;
use codex_app_server_protocol::ThreadEpiphanyReorientWorkerLaunchDocument;
use codex_app_server_protocol::ThreadEpiphanyRetrievalFreshness;
use codex_app_server_protocol::ThreadEpiphanyRetrievalFreshnessStatus;
use codex_app_server_protocol::ThreadEpiphanyRoleFinding;
use codex_app_server_protocol::ThreadEpiphanyRoleId;
use codex_app_server_protocol::ThreadEpiphanyRoleResultStatus;
use codex_app_server_protocol::ThreadEpiphanyRoleSelfPersistenceReview;
use codex_app_server_protocol::ThreadEpiphanyRoleSelfPersistenceStatus;
use codex_app_server_protocol::ThreadEpiphanyRoleWorkerLaunchDocument;
use codex_app_server_protocol::ThreadEpiphanyRolesSource;
use codex_app_server_protocol::ThreadEpiphanyStateUpdatedField;
use codex_app_server_protocol::ThreadEpiphanyStateUpdatedNotification;
use codex_app_server_protocol::ThreadEpiphanyStateUpdatedSource;
use codex_app_server_protocol::ThreadEpiphanyUpdatePatch;
use codex_app_server_protocol::ThreadEpiphanyViewLens;
use codex_app_server_protocol::ThreadEpiphanyWorkerLaunchDocument;
use epiphany_core::EpiphanyContextParams;
use epiphany_core::EpiphanyCoordinatorRoleResultStatus;
use epiphany_core::EpiphanyCrrcResultStatus;
use epiphany_core::EpiphanyDistillInput;
use epiphany_core::EpiphanyGraphQuery;
use epiphany_core::EpiphanyGraphQueryDirection;
use epiphany_core::EpiphanyGraphQueryKind;
use epiphany_core::EpiphanyPressure;
use epiphany_core::EpiphanyPressureBasis;
use epiphany_core::EpiphanyPressureLevel;
use epiphany_core::EpiphanyPressureStatus;
use epiphany_core::EpiphanyReorientFindingInterpretation;
use epiphany_core::EpiphanyReorientWorkerLaunchDocument;
use epiphany_core::EpiphanyRoleFindingInterpretation;
use epiphany_core::EpiphanyRoleResultRoleId;
use epiphany_core::EpiphanyRoleSelfPersistenceReview;
use epiphany_core::EpiphanyRoleSelfPersistenceStatus;
use epiphany_core::EpiphanyRoleStatePatchDocument;
use epiphany_core::EpiphanyRoleWorkerLaunchDocument;
use epiphany_core::EpiphanyStateUpdatedField;
use epiphany_core::EpiphanyViewLens;
use epiphany_core::EpiphanyWorkerLaunchDocument;
use epiphany_core::default_epiphany_view_lenses;
use epiphany_core::epiphany_view_needs_jobs;
use epiphany_core::epiphany_view_needs_pressure;
use epiphany_core::epiphany_view_needs_reorientation_inputs;
use epiphany_core::epiphany_view_needs_runtime_store;
use epiphany_state_model::EpiphanyThreadState;

pub fn default_core_epiphany_view_lenses() -> Vec<EpiphanyViewLens> {
    default_epiphany_view_lenses()
}

pub fn protocol_view_lenses_to_core(lenses: Vec<ThreadEpiphanyViewLens>) -> Vec<EpiphanyViewLens> {
    lenses.into_iter().map(protocol_view_lens_to_core).collect()
}

pub fn protocol_view_lens_from_core(lens: EpiphanyViewLens) -> ThreadEpiphanyViewLens {
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

pub fn core_epiphany_view_needs_jobs(lenses: &[EpiphanyViewLens]) -> bool {
    epiphany_view_needs_jobs(lenses)
}

pub fn core_epiphany_view_needs_reorientation_inputs(lenses: &[EpiphanyViewLens]) -> bool {
    epiphany_view_needs_reorientation_inputs(lenses)
}

pub fn core_epiphany_view_needs_pressure(lenses: &[EpiphanyViewLens]) -> bool {
    epiphany_view_needs_pressure(lenses)
}

pub fn core_epiphany_view_needs_runtime_store(lenses: &[EpiphanyViewLens]) -> bool {
    epiphany_view_needs_runtime_store(lenses)
}

pub fn protocol_context_params_to_core(
    params: &ThreadEpiphanyContextParams,
) -> EpiphanyContextParams {
    EpiphanyContextParams {
        graph_node_ids: params.graph_node_ids.clone(),
        graph_edge_ids: params.graph_edge_ids.clone(),
        observation_ids: params.observation_ids.clone(),
        evidence_ids: params.evidence_ids.clone(),
        include_active_frontier: params.include_active_frontier,
        include_linked_evidence: params.include_linked_evidence,
    }
}

pub fn protocol_distill_params_to_core(
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

pub fn protocol_graph_query_to_core(query: &ThreadEpiphanyGraphQuery) -> EpiphanyGraphQuery {
    EpiphanyGraphQuery {
        kind: protocol_graph_query_kind_to_core(query.kind),
        node_ids: query.node_ids.clone(),
        edge_ids: query.edge_ids.clone(),
        paths: query.paths.clone(),
        symbols: query.symbols.clone(),
        edge_kinds: query.edge_kinds.clone(),
        direction: query.direction.map(protocol_graph_query_direction_to_core),
        depth: query.depth,
        include_links: query.include_links,
    }
}

pub fn protocol_worker_launch_document_to_core(
    document: ThreadEpiphanyWorkerLaunchDocument,
) -> EpiphanyWorkerLaunchDocument {
    match document {
        ThreadEpiphanyWorkerLaunchDocument::Role(document) => EpiphanyWorkerLaunchDocument::Role(
            protocol_role_worker_launch_document_to_core(document),
        ),
        ThreadEpiphanyWorkerLaunchDocument::Reorient(document) => {
            EpiphanyWorkerLaunchDocument::Reorient(
                protocol_reorient_worker_launch_document_to_core(document),
            )
        }
    }
}

pub fn protocol_update_patch_to_core(
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

pub fn protocol_patch_from_core(
    patch: EpiphanyRoleStatePatchDocument,
) -> ThreadEpiphanyUpdatePatch {
    ThreadEpiphanyUpdatePatch {
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
    }
}

pub fn protocol_role_id_to_core(role_id: ThreadEpiphanyRoleId) -> EpiphanyRoleResultRoleId {
    match role_id {
        ThreadEpiphanyRoleId::Implementation => EpiphanyRoleResultRoleId::Implementation,
        ThreadEpiphanyRoleId::Imagination => EpiphanyRoleResultRoleId::Imagination,
        ThreadEpiphanyRoleId::Modeling => EpiphanyRoleResultRoleId::Modeling,
        ThreadEpiphanyRoleId::Verification => EpiphanyRoleResultRoleId::Verification,
        ThreadEpiphanyRoleId::Reorientation => EpiphanyRoleResultRoleId::Reorientation,
    }
}

pub fn protocol_role_id_from_core(role_id: EpiphanyRoleResultRoleId) -> ThreadEpiphanyRoleId {
    match role_id {
        EpiphanyRoleResultRoleId::Implementation => ThreadEpiphanyRoleId::Implementation,
        EpiphanyRoleResultRoleId::Imagination => ThreadEpiphanyRoleId::Imagination,
        EpiphanyRoleResultRoleId::Modeling => ThreadEpiphanyRoleId::Modeling,
        EpiphanyRoleResultRoleId::Verification => ThreadEpiphanyRoleId::Verification,
        EpiphanyRoleResultRoleId::Reorientation => ThreadEpiphanyRoleId::Reorientation,
    }
}

pub fn protocol_reorient_finding(
    finding: EpiphanyReorientFindingInterpretation,
) -> ThreadEpiphanyReorientFinding {
    ThreadEpiphanyReorientFinding {
        mode: finding.mode,
        summary: finding.summary,
        next_safe_move: finding.next_safe_move,
        checkpoint_still_valid: finding.checkpoint_still_valid,
        files_inspected: finding.files_inspected,
        frontier_node_ids: finding.frontier_node_ids,
        evidence_ids: finding.evidence_ids,
        artifact_refs: finding.artifact_refs,
        runtime_result_id: finding.runtime_result_id,
        runtime_job_id: finding.runtime_job_id,
        job_error: finding.job_error,
        item_error: finding.item_error,
    }
}

pub fn protocol_role_finding(
    role_id: ThreadEpiphanyRoleId,
    finding: EpiphanyRoleFindingInterpretation,
) -> ThreadEpiphanyRoleFinding {
    let state_patch = finding.state_patch.map(protocol_patch_from_core);
    ThreadEpiphanyRoleFinding {
        role_id,
        verdict: finding.verdict,
        summary: finding.summary,
        next_safe_move: finding.next_safe_move,
        checkpoint_summary: finding.checkpoint_summary,
        scratch_summary: finding.scratch_summary,
        files_inspected: finding.files_inspected,
        frontier_node_ids: finding.frontier_node_ids,
        evidence_ids: finding.evidence_ids,
        artifact_refs: finding.artifact_refs,
        runtime_result_id: finding.runtime_result_id,
        runtime_job_id: finding.runtime_job_id,
        open_questions: finding.open_questions,
        evidence_gaps: finding.evidence_gaps,
        risks: finding.risks,
        state_patch,
        self_patch: finding.self_patch.map(|patch| {
            serde_json::to_value(patch)
                .expect("AgentSelfPatch is a serializable protocol projection")
        }),
        self_persistence: finding
            .self_persistence
            .map(protocol_self_persistence_review),
        job_error: finding.job_error,
        item_error: finding.item_error,
    }
}

pub fn protocol_state_updated_notification(
    thread_id: String,
    source: ThreadEpiphanyStateUpdatedSource,
    revision: u64,
    changed_fields: Vec<EpiphanyStateUpdatedField>,
    epiphany_state: EpiphanyThreadState,
) -> ThreadEpiphanyStateUpdatedNotification {
    ThreadEpiphanyStateUpdatedNotification {
        thread_id,
        source,
        revision,
        changed_fields: protocol_state_updated_fields(changed_fields),
        epiphany_state,
    }
}

pub fn protocol_state_updated_fields(
    fields: Vec<EpiphanyStateUpdatedField>,
) -> Vec<ThreadEpiphanyStateUpdatedField> {
    fields
        .into_iter()
        .map(protocol_state_updated_field)
        .collect()
}

pub fn protocol_state_updated_field(
    field: EpiphanyStateUpdatedField,
) -> ThreadEpiphanyStateUpdatedField {
    match field {
        EpiphanyStateUpdatedField::Objective => ThreadEpiphanyStateUpdatedField::Objective,
        EpiphanyStateUpdatedField::ActiveSubgoalId => {
            ThreadEpiphanyStateUpdatedField::ActiveSubgoalId
        }
        EpiphanyStateUpdatedField::Subgoals => ThreadEpiphanyStateUpdatedField::Subgoals,
        EpiphanyStateUpdatedField::Invariants => ThreadEpiphanyStateUpdatedField::Invariants,
        EpiphanyStateUpdatedField::Graphs => ThreadEpiphanyStateUpdatedField::Graphs,
        EpiphanyStateUpdatedField::GraphFrontier => ThreadEpiphanyStateUpdatedField::GraphFrontier,
        EpiphanyStateUpdatedField::GraphCheckpoint => {
            ThreadEpiphanyStateUpdatedField::GraphCheckpoint
        }
        EpiphanyStateUpdatedField::Scratch => ThreadEpiphanyStateUpdatedField::Scratch,
        EpiphanyStateUpdatedField::InvestigationCheckpoint => {
            ThreadEpiphanyStateUpdatedField::InvestigationCheckpoint
        }
        EpiphanyStateUpdatedField::JobBindings => ThreadEpiphanyStateUpdatedField::JobBindings,
        EpiphanyStateUpdatedField::AcceptanceReceipts => {
            ThreadEpiphanyStateUpdatedField::AcceptanceReceipts
        }
        EpiphanyStateUpdatedField::RuntimeLinks => ThreadEpiphanyStateUpdatedField::RuntimeLinks,
        EpiphanyStateUpdatedField::Observations => ThreadEpiphanyStateUpdatedField::Observations,
        EpiphanyStateUpdatedField::Evidence => ThreadEpiphanyStateUpdatedField::Evidence,
        EpiphanyStateUpdatedField::Churn => ThreadEpiphanyStateUpdatedField::Churn,
        EpiphanyStateUpdatedField::Mode => ThreadEpiphanyStateUpdatedField::Mode,
        EpiphanyStateUpdatedField::Planning => ThreadEpiphanyStateUpdatedField::Planning,
    }
}

pub fn protocol_freshness_response_from_surface(
    surface: EpiphanyFreshnessSurface,
) -> ThreadEpiphanyFreshnessResponse {
    ThreadEpiphanyFreshnessResponse {
        thread_id: surface.thread_id,
        source: match surface.source {
            EpiphanySurfaceSource::Stored => ThreadEpiphanyFreshnessSource::Stored,
            EpiphanySurfaceSource::Live => ThreadEpiphanyFreshnessSource::Live,
        },
        state_revision: surface.state_revision,
        retrieval: ThreadEpiphanyRetrievalFreshness {
            status: match surface.retrieval.status {
                EpiphanyRetrievalFreshnessStatus::Missing => {
                    ThreadEpiphanyRetrievalFreshnessStatus::Missing
                }
                EpiphanyRetrievalFreshnessStatus::Ready => {
                    ThreadEpiphanyRetrievalFreshnessStatus::Ready
                }
                EpiphanyRetrievalFreshnessStatus::Stale => {
                    ThreadEpiphanyRetrievalFreshnessStatus::Stale
                }
                EpiphanyRetrievalFreshnessStatus::Indexing => {
                    ThreadEpiphanyRetrievalFreshnessStatus::Indexing
                }
                EpiphanyRetrievalFreshnessStatus::Unavailable => {
                    ThreadEpiphanyRetrievalFreshnessStatus::Unavailable
                }
            },
            semantic_available: surface.retrieval.semantic_available,
            last_indexed_at_unix_seconds: surface.retrieval.last_indexed_at_unix_seconds,
            indexed_file_count: surface.retrieval.indexed_file_count,
            indexed_chunk_count: surface.retrieval.indexed_chunk_count,
            dirty_paths: surface.retrieval.dirty_paths,
            note: surface.retrieval.note,
        },
        graph: ThreadEpiphanyGraphFreshness {
            status: match surface.graph.status {
                EpiphanyGraphFreshnessStatus::Missing => {
                    ThreadEpiphanyGraphFreshnessStatus::Missing
                }
                EpiphanyGraphFreshnessStatus::Ready => ThreadEpiphanyGraphFreshnessStatus::Ready,
                EpiphanyGraphFreshnessStatus::Stale => ThreadEpiphanyGraphFreshnessStatus::Stale,
            },
            graph_freshness: surface.graph.graph_freshness,
            checkpoint_id: surface.graph.checkpoint_id,
            dirty_path_count: surface.graph.dirty_path_count,
            dirty_paths: surface.graph.dirty_paths,
            open_question_count: surface.graph.open_question_count,
            open_gap_count: surface.graph.open_gap_count,
            note: surface.graph.note,
        },
        watcher: ThreadEpiphanyInvalidationInput {
            status: match surface.watcher.status {
                EpiphanyInvalidationStatus::Unavailable => {
                    ThreadEpiphanyInvalidationStatus::Unavailable
                }
                EpiphanyInvalidationStatus::Clean => ThreadEpiphanyInvalidationStatus::Clean,
                EpiphanyInvalidationStatus::Changed => ThreadEpiphanyInvalidationStatus::Changed,
            },
            watched_root: surface.watcher.watched_root,
            observed_at_unix_seconds: surface.watcher.observed_at_unix_seconds,
            changed_path_count: surface.watcher.changed_path_count,
            changed_paths: surface.watcher.changed_paths,
            graph_node_ids: surface.watcher.graph_node_ids,
            active_frontier_node_ids: surface.watcher.active_frontier_node_ids,
            note: surface.watcher.note,
        },
    }
}

pub fn protocol_job_from_surface(
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

pub fn protocol_reorient_state_status(
    status: EpiphanyReorientStateStatus,
) -> ThreadEpiphanyReorientStateStatus {
    match status {
        EpiphanyReorientStateStatus::Missing => ThreadEpiphanyReorientStateStatus::Missing,
        EpiphanyReorientStateStatus::Ready => ThreadEpiphanyReorientStateStatus::Ready,
    }
}

pub fn protocol_reorient_source(source: EpiphanySurfaceSource) -> ThreadEpiphanyReorientSource {
    match source {
        EpiphanySurfaceSource::Stored => ThreadEpiphanyReorientSource::Stored,
        EpiphanySurfaceSource::Live => ThreadEpiphanyReorientSource::Live,
    }
}

pub fn protocol_roles_source(source: EpiphanySurfaceSource) -> ThreadEpiphanyRolesSource {
    match source {
        EpiphanySurfaceSource::Stored => ThreadEpiphanyRolesSource::Stored,
        EpiphanySurfaceSource::Live => ThreadEpiphanyRolesSource::Live,
    }
}

pub fn protocol_reorient_decision(
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
        retrieval_status: protocol_reorient_retrieval_status(decision.retrieval_status),
        graph_status: protocol_reorient_graph_status(decision.graph_status),
        watcher_status: protocol_reorient_watcher_status(decision.watcher_status),
        reasons: decision
            .reasons
            .into_iter()
            .map(protocol_reorient_reason)
            .collect(),
        checkpoint_dirty_paths: decision.checkpoint_dirty_paths,
        checkpoint_changed_paths: decision.checkpoint_changed_paths,
        active_frontier_node_ids: decision.active_frontier_node_ids,
        next_action: decision.next_action,
        note: decision.note,
    }
}

pub fn protocol_pressure_from_core(pressure: EpiphanyPressure) -> ThreadEpiphanyPressure {
    ThreadEpiphanyPressure {
        status: match pressure.status {
            EpiphanyPressureStatus::Unknown => ThreadEpiphanyPressureStatus::Unknown,
            EpiphanyPressureStatus::Ready => ThreadEpiphanyPressureStatus::Ready,
        },
        level: protocol_pressure_level(pressure.level),
        basis: match pressure.basis {
            EpiphanyPressureBasis::Unknown => ThreadEpiphanyPressureBasis::Unknown,
            EpiphanyPressureBasis::AutoCompactLimit => {
                ThreadEpiphanyPressureBasis::AutoCompactLimit
            }
            EpiphanyPressureBasis::ModelContextWindow => {
                ThreadEpiphanyPressureBasis::ModelContextWindow
            }
        },
        used_tokens: pressure.used_tokens,
        model_context_window: pressure.model_context_window,
        model_auto_compact_token_limit: pressure.model_auto_compact_token_limit,
        remaining_tokens: pressure.remaining_tokens,
        ratio_per_mille: pressure.ratio_per_mille,
        should_prepare_compaction: pressure.should_prepare_compaction,
        note: pressure.note,
    }
}

pub fn protocol_pressure_level(level: EpiphanyPressureLevel) -> ThreadEpiphanyPressureLevel {
    match level {
        EpiphanyPressureLevel::Unknown => ThreadEpiphanyPressureLevel::Unknown,
        EpiphanyPressureLevel::Low => ThreadEpiphanyPressureLevel::Low,
        EpiphanyPressureLevel::Elevated => ThreadEpiphanyPressureLevel::Elevated,
        EpiphanyPressureLevel::High => ThreadEpiphanyPressureLevel::High,
        EpiphanyPressureLevel::Critical => ThreadEpiphanyPressureLevel::Critical,
    }
}

pub fn protocol_role_result_status(
    status: EpiphanyCoordinatorRoleResultStatus,
) -> ThreadEpiphanyRoleResultStatus {
    match status {
        EpiphanyCoordinatorRoleResultStatus::MissingState => {
            ThreadEpiphanyRoleResultStatus::MissingState
        }
        EpiphanyCoordinatorRoleResultStatus::MissingBinding => {
            ThreadEpiphanyRoleResultStatus::MissingBinding
        }
        EpiphanyCoordinatorRoleResultStatus::BackendUnavailable => {
            ThreadEpiphanyRoleResultStatus::BackendUnavailable
        }
        EpiphanyCoordinatorRoleResultStatus::BackendMissing => {
            ThreadEpiphanyRoleResultStatus::BackendMissing
        }
        EpiphanyCoordinatorRoleResultStatus::Pending => ThreadEpiphanyRoleResultStatus::Pending,
        EpiphanyCoordinatorRoleResultStatus::Running => ThreadEpiphanyRoleResultStatus::Running,
        EpiphanyCoordinatorRoleResultStatus::Completed => ThreadEpiphanyRoleResultStatus::Completed,
        EpiphanyCoordinatorRoleResultStatus::Failed => ThreadEpiphanyRoleResultStatus::Failed,
        EpiphanyCoordinatorRoleResultStatus::Cancelled => ThreadEpiphanyRoleResultStatus::Cancelled,
    }
}

pub fn protocol_reorient_result_status(
    status: EpiphanyCrrcResultStatus,
) -> ThreadEpiphanyReorientResultStatus {
    match status {
        EpiphanyCrrcResultStatus::MissingState => ThreadEpiphanyReorientResultStatus::MissingState,
        EpiphanyCrrcResultStatus::MissingBinding => {
            ThreadEpiphanyReorientResultStatus::MissingBinding
        }
        EpiphanyCrrcResultStatus::BackendUnavailable => {
            ThreadEpiphanyReorientResultStatus::BackendUnavailable
        }
        EpiphanyCrrcResultStatus::BackendMissing => {
            ThreadEpiphanyReorientResultStatus::BackendMissing
        }
        EpiphanyCrrcResultStatus::Pending => ThreadEpiphanyReorientResultStatus::Pending,
        EpiphanyCrrcResultStatus::Running => ThreadEpiphanyReorientResultStatus::Running,
        EpiphanyCrrcResultStatus::Completed => ThreadEpiphanyReorientResultStatus::Completed,
        EpiphanyCrrcResultStatus::Failed => ThreadEpiphanyReorientResultStatus::Failed,
        EpiphanyCrrcResultStatus::Cancelled => ThreadEpiphanyReorientResultStatus::Cancelled,
    }
}

fn protocol_view_lens_to_core(lens: ThreadEpiphanyViewLens) -> EpiphanyViewLens {
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

fn protocol_graph_query_kind_to_core(kind: ThreadEpiphanyGraphQueryKind) -> EpiphanyGraphQueryKind {
    match kind {
        ThreadEpiphanyGraphQueryKind::Node => EpiphanyGraphQueryKind::Node,
        ThreadEpiphanyGraphQueryKind::Path => EpiphanyGraphQueryKind::Path,
        ThreadEpiphanyGraphQueryKind::FrontierNeighborhood => {
            EpiphanyGraphQueryKind::FrontierNeighborhood
        }
        ThreadEpiphanyGraphQueryKind::Neighbors => EpiphanyGraphQueryKind::Neighbors,
    }
}

fn protocol_graph_query_direction_to_core(
    direction: ThreadEpiphanyGraphQueryDirection,
) -> EpiphanyGraphQueryDirection {
    match direction {
        ThreadEpiphanyGraphQueryDirection::Incoming => EpiphanyGraphQueryDirection::Incoming,
        ThreadEpiphanyGraphQueryDirection::Outgoing => EpiphanyGraphQueryDirection::Outgoing,
        ThreadEpiphanyGraphQueryDirection::Both => EpiphanyGraphQueryDirection::Both,
    }
}

fn protocol_role_worker_launch_document_to_core(
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

fn protocol_reorient_worker_launch_document_to_core(
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

fn protocol_reorient_retrieval_status(
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

fn protocol_reorient_graph_status(
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

fn protocol_reorient_watcher_status(
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

fn protocol_reorient_reason(reason: EpiphanyReorientReason) -> ThreadEpiphanyReorientReason {
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

fn protocol_self_persistence_review(
    review: EpiphanyRoleSelfPersistenceReview,
) -> ThreadEpiphanyRoleSelfPersistenceReview {
    ThreadEpiphanyRoleSelfPersistenceReview {
        status: match review.status {
            EpiphanyRoleSelfPersistenceStatus::Missing => {
                ThreadEpiphanyRoleSelfPersistenceStatus::Missing
            }
            EpiphanyRoleSelfPersistenceStatus::Accepted => {
                ThreadEpiphanyRoleSelfPersistenceStatus::Accepted
            }
            EpiphanyRoleSelfPersistenceStatus::Rejected => {
                ThreadEpiphanyRoleSelfPersistenceStatus::Rejected
            }
        },
        target_agent_id: review.target_agent_id,
        target_path: review.target_path,
        reasons: review.reasons,
    }
}
