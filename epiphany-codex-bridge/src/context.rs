use codex_app_server_protocol::ThreadEpiphanyContext;
use codex_app_server_protocol::ThreadEpiphanyContextMissing;
use codex_app_server_protocol::ThreadEpiphanyContextStateStatus;
use codex_app_server_protocol::ThreadEpiphanyGraphContext;
use codex_app_server_protocol::ThreadEpiphanyGraphQueryMatched;
use codex_app_server_protocol::ThreadEpiphanyGraphQueryMissing;
use codex_app_server_protocol::ThreadEpiphanyPlanningSummary;
use epiphany_core::EpiphanyContext as CoreEpiphanyContext;
use epiphany_core::EpiphanyContextMissing as CoreEpiphanyContextMissing;
use epiphany_core::EpiphanyContextParams as CoreEpiphanyContextParams;
use epiphany_core::EpiphanyContextStateStatus as CoreEpiphanyContextStateStatus;
use epiphany_core::EpiphanyContextView;
use epiphany_core::EpiphanyGraphContext as CoreEpiphanyGraphContext;
use epiphany_core::EpiphanyGraphQuery as CoreEpiphanyGraphQuery;
use epiphany_core::EpiphanyGraphQueryMatched as CoreEpiphanyGraphQueryMatched;
use epiphany_core::EpiphanyGraphQueryMissing as CoreEpiphanyGraphQueryMissing;
use epiphany_core::EpiphanyGraphQueryView;
use epiphany_core::EpiphanyPlanningStateStatus as CoreEpiphanyPlanningStateStatus;
use epiphany_core::EpiphanyPlanningSummary as CoreEpiphanyPlanningSummary;
use epiphany_core::EpiphanyPlanningView;
use epiphany_core::derive_context;
use epiphany_core::derive_graph_query;
use epiphany_core::derive_planning_view;
use epiphany_state_model::EpiphanyGraphCheckpoint;
use epiphany_state_model::EpiphanyGraphFrontier;
use epiphany_state_model::EpiphanyPlanningState;
use epiphany_state_model::EpiphanyThreadState;

pub fn map_epiphany_context(
    state: Option<&EpiphanyThreadState>,
    params: &CoreEpiphanyContextParams,
) -> (
    ThreadEpiphanyContextStateStatus,
    Option<u64>,
    ThreadEpiphanyContext,
    ThreadEpiphanyContextMissing,
) {
    map_core_epiphany_context_view(derive_context(state, params))
}

fn map_core_epiphany_context_view(
    view: EpiphanyContextView,
) -> (
    ThreadEpiphanyContextStateStatus,
    Option<u64>,
    ThreadEpiphanyContext,
    ThreadEpiphanyContextMissing,
) {
    (
        map_core_epiphany_context_state_status(view.state_status),
        view.state_revision,
        map_core_epiphany_context(view.context),
        map_core_epiphany_context_missing(view.missing),
    )
}

fn map_core_epiphany_context_state_status(
    status: CoreEpiphanyContextStateStatus,
) -> ThreadEpiphanyContextStateStatus {
    match status {
        CoreEpiphanyContextStateStatus::Missing => ThreadEpiphanyContextStateStatus::Missing,
        CoreEpiphanyContextStateStatus::Ready => ThreadEpiphanyContextStateStatus::Ready,
    }
}

fn map_core_epiphany_context(context: CoreEpiphanyContext) -> ThreadEpiphanyContext {
    ThreadEpiphanyContext {
        graph: map_core_epiphany_graph_context(context.graph),
        frontier: context.frontier,
        checkpoint: context.checkpoint,
        investigation_checkpoint: context.investigation_checkpoint,
        observations: context.observations,
        evidence: context.evidence,
    }
}

fn map_core_epiphany_graph_context(graph: CoreEpiphanyGraphContext) -> ThreadEpiphanyGraphContext {
    ThreadEpiphanyGraphContext {
        architecture_nodes: graph.architecture_nodes,
        architecture_edges: graph.architecture_edges,
        dataflow_nodes: graph.dataflow_nodes,
        dataflow_edges: graph.dataflow_edges,
        links: graph.links,
    }
}

fn map_core_epiphany_context_missing(
    missing: CoreEpiphanyContextMissing,
) -> ThreadEpiphanyContextMissing {
    ThreadEpiphanyContextMissing {
        graph_node_ids: missing.graph_node_ids,
        graph_edge_ids: missing.graph_edge_ids,
        observation_ids: missing.observation_ids,
        evidence_ids: missing.evidence_ids,
    }
}

pub fn map_epiphany_planning(
    state: Option<&EpiphanyThreadState>,
) -> (
    ThreadEpiphanyContextStateStatus,
    Option<u64>,
    EpiphanyPlanningState,
    ThreadEpiphanyPlanningSummary,
) {
    let view = derive_planning_view(state);
    map_core_epiphany_planning_view(view)
}

fn map_core_epiphany_planning_view(
    view: EpiphanyPlanningView,
) -> (
    ThreadEpiphanyContextStateStatus,
    Option<u64>,
    EpiphanyPlanningState,
    ThreadEpiphanyPlanningSummary,
) {
    (
        match view.state_status {
            CoreEpiphanyPlanningStateStatus::Missing => ThreadEpiphanyContextStateStatus::Missing,
            CoreEpiphanyPlanningStateStatus::Ready => ThreadEpiphanyContextStateStatus::Ready,
        },
        view.state_revision,
        view.planning,
        map_core_epiphany_planning_summary(view.summary),
    )
}

fn map_core_epiphany_planning_summary(
    summary: CoreEpiphanyPlanningSummary,
) -> ThreadEpiphanyPlanningSummary {
    ThreadEpiphanyPlanningSummary {
        capture_count: summary.capture_count,
        pending_capture_count: summary.pending_capture_count,
        github_issue_capture_count: summary.github_issue_capture_count,
        backlog_item_count: summary.backlog_item_count,
        ready_backlog_item_count: summary.ready_backlog_item_count,
        roadmap_stream_count: summary.roadmap_stream_count,
        objective_draft_count: summary.objective_draft_count,
        draft_objective_count: summary.draft_objective_count,
        active_objective: summary.active_objective,
        note: summary.note,
    }
}

pub fn map_epiphany_graph_query(
    state: Option<&EpiphanyThreadState>,
    query: &CoreEpiphanyGraphQuery,
) -> (
    ThreadEpiphanyContextStateStatus,
    Option<u64>,
    ThreadEpiphanyGraphContext,
    Option<EpiphanyGraphFrontier>,
    Option<EpiphanyGraphCheckpoint>,
    ThreadEpiphanyGraphQueryMatched,
    ThreadEpiphanyGraphQueryMissing,
) {
    map_core_epiphany_graph_query_view(derive_graph_query(state, query))
}

fn map_core_epiphany_graph_query_view(
    view: EpiphanyGraphQueryView,
) -> (
    ThreadEpiphanyContextStateStatus,
    Option<u64>,
    ThreadEpiphanyGraphContext,
    Option<EpiphanyGraphFrontier>,
    Option<EpiphanyGraphCheckpoint>,
    ThreadEpiphanyGraphQueryMatched,
    ThreadEpiphanyGraphQueryMissing,
) {
    (
        map_core_epiphany_context_state_status(view.state_status),
        view.state_revision,
        map_core_epiphany_graph_context(view.graph),
        view.frontier,
        view.checkpoint,
        map_core_epiphany_graph_query_matched(view.matched),
        map_core_epiphany_graph_query_missing(view.missing),
    )
}

fn map_core_epiphany_graph_query_matched(
    matched: CoreEpiphanyGraphQueryMatched,
) -> ThreadEpiphanyGraphQueryMatched {
    ThreadEpiphanyGraphQueryMatched {
        node_ids: matched.node_ids,
        edge_ids: matched.edge_ids,
        paths: matched.paths,
        symbols: matched.symbols,
        edge_kinds: matched.edge_kinds,
    }
}

fn map_core_epiphany_graph_query_missing(
    missing: CoreEpiphanyGraphQueryMissing,
) -> ThreadEpiphanyGraphQueryMissing {
    ThreadEpiphanyGraphQueryMissing {
        node_ids: missing.node_ids,
        edge_ids: missing.edge_ids,
    }
}
