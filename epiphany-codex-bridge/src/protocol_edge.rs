use codex_app_server_protocol::ThreadEpiphanyContextParams;
use codex_app_server_protocol::ThreadEpiphanyDistillParams;
use codex_app_server_protocol::ThreadEpiphanyGraphQuery;
use codex_app_server_protocol::ThreadEpiphanyGraphQueryDirection;
use codex_app_server_protocol::ThreadEpiphanyGraphQueryKind;
use codex_app_server_protocol::ThreadEpiphanyReorientWorkerLaunchDocument;
use codex_app_server_protocol::ThreadEpiphanyRoleWorkerLaunchDocument;
use codex_app_server_protocol::ThreadEpiphanyUpdatePatch;
use codex_app_server_protocol::ThreadEpiphanyViewLens;
use codex_app_server_protocol::ThreadEpiphanyWorkerLaunchDocument;
use epiphany_core::EpiphanyContextParams;
use epiphany_core::EpiphanyDistillInput;
use epiphany_core::EpiphanyGraphQuery;
use epiphany_core::EpiphanyGraphQueryDirection;
use epiphany_core::EpiphanyGraphQueryKind;
use epiphany_core::EpiphanyReorientWorkerLaunchDocument;
use epiphany_core::EpiphanyRoleStatePatchDocument;
use epiphany_core::EpiphanyRoleWorkerLaunchDocument;
use epiphany_core::EpiphanyViewLens;
use epiphany_core::EpiphanyWorkerLaunchDocument;
use epiphany_core::default_epiphany_view_lenses;
use epiphany_core::epiphany_view_needs_jobs;
use epiphany_core::epiphany_view_needs_pressure;
use epiphany_core::epiphany_view_needs_reorientation_inputs;
use epiphany_core::epiphany_view_needs_runtime_store;

pub fn default_core_epiphany_view_lenses() -> Vec<EpiphanyViewLens> {
    default_epiphany_view_lenses()
}

pub fn protocol_view_lenses_to_core(lenses: Vec<ThreadEpiphanyViewLens>) -> Vec<EpiphanyViewLens> {
    lenses.into_iter().map(protocol_view_lens_to_core).collect()
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
