use codex_protocol::protocol::EpiphanyCodeRef;
use codex_protocol::protocol::EpiphanyEvidenceRecord;
use codex_protocol::protocol::EpiphanyGraphCheckpoint;
use codex_protocol::protocol::EpiphanyGraphEdge;
use codex_protocol::protocol::EpiphanyGraphFrontier;
use codex_protocol::protocol::EpiphanyGraphLink;
use codex_protocol::protocol::EpiphanyGraphNode;
use codex_protocol::protocol::EpiphanyInvestigationCheckpoint;
use codex_protocol::protocol::EpiphanyObservation;
use codex_protocol::protocol::EpiphanyThreadState;
use std::collections::HashSet;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyContextStateStatus {
    Missing,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EpiphanyGraphContext {
    pub architecture_nodes: Vec<EpiphanyGraphNode>,
    pub architecture_edges: Vec<EpiphanyGraphEdge>,
    pub dataflow_nodes: Vec<EpiphanyGraphNode>,
    pub dataflow_edges: Vec<EpiphanyGraphEdge>,
    pub links: Vec<EpiphanyGraphLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EpiphanyContext {
    pub graph: EpiphanyGraphContext,
    pub frontier: Option<EpiphanyGraphFrontier>,
    pub checkpoint: Option<EpiphanyGraphCheckpoint>,
    pub investigation_checkpoint: Option<EpiphanyInvestigationCheckpoint>,
    pub observations: Vec<EpiphanyObservation>,
    pub evidence: Vec<EpiphanyEvidenceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EpiphanyContextMissing {
    pub graph_node_ids: Vec<String>,
    pub graph_edge_ids: Vec<String>,
    pub observation_ids: Vec<String>,
    pub evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EpiphanyContextParams {
    pub graph_node_ids: Vec<String>,
    pub graph_edge_ids: Vec<String>,
    pub observation_ids: Vec<String>,
    pub evidence_ids: Vec<String>,
    pub include_active_frontier: Option<bool>,
    pub include_linked_evidence: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyContextView {
    pub state_status: EpiphanyContextStateStatus,
    pub state_revision: Option<u64>,
    pub context: EpiphanyContext,
    pub missing: EpiphanyContextMissing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyGraphQueryKind {
    Node,
    Path,
    FrontierNeighborhood,
    Neighbors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpiphanyGraphQueryDirection {
    Incoming,
    Outgoing,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyGraphQuery {
    pub kind: EpiphanyGraphQueryKind,
    pub node_ids: Vec<String>,
    pub edge_ids: Vec<String>,
    pub paths: Vec<PathBuf>,
    pub symbols: Vec<String>,
    pub edge_kinds: Vec<String>,
    pub direction: Option<EpiphanyGraphQueryDirection>,
    pub depth: Option<u32>,
    pub include_links: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EpiphanyGraphQueryMatched {
    pub node_ids: Vec<String>,
    pub edge_ids: Vec<String>,
    pub paths: Vec<PathBuf>,
    pub symbols: Vec<String>,
    pub edge_kinds: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EpiphanyGraphQueryMissing {
    pub node_ids: Vec<String>,
    pub edge_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EpiphanyGraphQueryView {
    pub state_status: EpiphanyContextStateStatus,
    pub state_revision: Option<u64>,
    pub graph: EpiphanyGraphContext,
    pub frontier: Option<EpiphanyGraphFrontier>,
    pub checkpoint: Option<EpiphanyGraphCheckpoint>,
    pub matched: EpiphanyGraphQueryMatched,
    pub missing: EpiphanyGraphQueryMissing,
}

pub fn derive_context(
    state: Option<&EpiphanyThreadState>,
    params: &EpiphanyContextParams,
) -> EpiphanyContextView {
    let include_active_frontier = params.include_active_frontier.unwrap_or(true);
    let include_linked_evidence = params.include_linked_evidence.unwrap_or(true);
    let Some(state) = state else {
        return EpiphanyContextView {
            state_status: EpiphanyContextStateStatus::Missing,
            state_revision: None,
            context: EpiphanyContext::default(),
            missing: EpiphanyContextMissing {
                graph_node_ids: unique_strings(params.graph_node_ids.iter().cloned()),
                graph_edge_ids: unique_strings(params.graph_edge_ids.iter().cloned()),
                observation_ids: unique_strings(params.observation_ids.iter().cloned()),
                evidence_ids: unique_strings(params.evidence_ids.iter().cloned()),
            },
        };
    };

    let mut graph_node_ids = unique_strings(params.graph_node_ids.iter().cloned());
    let mut graph_edge_ids = unique_strings(params.graph_edge_ids.iter().cloned());
    if include_active_frontier && let Some(frontier) = state.graph_frontier.as_ref() {
        extend_unique_strings(
            &mut graph_node_ids,
            frontier.active_node_ids.iter().cloned(),
        );
        extend_unique_strings(
            &mut graph_edge_ids,
            frontier.active_edge_ids.iter().cloned(),
        );
    }

    let graph_node_id_set: HashSet<&str> = graph_node_ids.iter().map(String::as_str).collect();
    let graph_edge_id_set: HashSet<&str> = graph_edge_ids.iter().map(String::as_str).collect();

    let architecture_nodes = state
        .graphs
        .architecture
        .nodes
        .iter()
        .filter(|node| graph_node_id_set.contains(node.id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let dataflow_nodes = state
        .graphs
        .dataflow
        .nodes
        .iter()
        .filter(|node| graph_node_id_set.contains(node.id.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    let found_node_ids: HashSet<String> = architecture_nodes
        .iter()
        .chain(dataflow_nodes.iter())
        .map(|node| node.id.clone())
        .collect();

    let architecture_edges = select_graph_edges(
        &state.graphs.architecture.edges,
        &graph_node_id_set,
        &graph_edge_id_set,
    );
    let dataflow_edges = select_graph_edges(
        &state.graphs.dataflow.edges,
        &graph_node_id_set,
        &graph_edge_id_set,
    );

    let found_edge_ids: HashSet<String> = architecture_edges
        .iter()
        .chain(dataflow_edges.iter())
        .filter_map(|edge| edge.id.clone())
        .filter(|id| graph_edge_id_set.contains(id.as_str()))
        .collect();

    let links = state
        .graphs
        .links
        .iter()
        .filter(|link| {
            graph_node_id_set.contains(link.architecture_node_id.as_str())
                || graph_node_id_set.contains(link.dataflow_node_id.as_str())
        })
        .cloned()
        .collect();

    let observation_ids = unique_strings(params.observation_ids.iter().cloned());
    let observation_id_set: HashSet<&str> = observation_ids.iter().map(String::as_str).collect();
    let observations = state
        .observations
        .iter()
        .filter(|observation| observation_id_set.contains(observation.id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let found_observation_ids: HashSet<String> = observations
        .iter()
        .map(|observation| observation.id.clone())
        .collect();

    let mut evidence_ids = unique_strings(params.evidence_ids.iter().cloned());
    if include_linked_evidence {
        for observation in &observations {
            extend_unique_strings(&mut evidence_ids, observation.evidence_ids.iter().cloned());
        }
    }
    let evidence_id_set: HashSet<&str> = evidence_ids.iter().map(String::as_str).collect();
    let evidence = state
        .recent_evidence
        .iter()
        .filter(|evidence| evidence_id_set.contains(evidence.id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let found_evidence_ids: HashSet<String> = evidence
        .iter()
        .map(|evidence| evidence.id.clone())
        .collect();

    EpiphanyContextView {
        state_status: EpiphanyContextStateStatus::Ready,
        state_revision: Some(state.revision),
        context: EpiphanyContext {
            graph: EpiphanyGraphContext {
                architecture_nodes,
                architecture_edges,
                dataflow_nodes,
                dataflow_edges,
                links,
            },
            frontier: include_active_frontier
                .then(|| state.graph_frontier.clone())
                .flatten(),
            checkpoint: state.graph_checkpoint.clone(),
            investigation_checkpoint: state.investigation_checkpoint.clone(),
            observations,
            evidence,
        },
        missing: EpiphanyContextMissing {
            graph_node_ids: graph_node_ids
                .iter()
                .filter(|id| !found_node_ids.contains(*id))
                .cloned()
                .collect(),
            graph_edge_ids: graph_edge_ids
                .iter()
                .filter(|id| !found_edge_ids.contains(*id))
                .cloned()
                .collect(),
            observation_ids: observation_ids
                .iter()
                .filter(|id| !found_observation_ids.contains(*id))
                .cloned()
                .collect(),
            evidence_ids: evidence_ids
                .iter()
                .filter(|id| !found_evidence_ids.contains(*id))
                .cloned()
                .collect(),
        },
    }
}

pub fn derive_graph_query(
    state: Option<&EpiphanyThreadState>,
    query: &EpiphanyGraphQuery,
) -> EpiphanyGraphQueryView {
    let Some(state) = state else {
        return EpiphanyGraphQueryView {
            state_status: EpiphanyContextStateStatus::Missing,
            state_revision: None,
            graph: EpiphanyGraphContext::default(),
            frontier: None,
            checkpoint: None,
            matched: EpiphanyGraphQueryMatched::default(),
            missing: EpiphanyGraphQueryMissing {
                node_ids: unique_strings(query.node_ids.iter().cloned()),
                edge_ids: unique_strings(query.edge_ids.iter().cloned()),
            },
        };
    };

    let mut node_ids = unique_strings(query.node_ids.iter().cloned());
    let mut edge_ids = unique_strings(query.edge_ids.iter().cloned());
    let mut matched_paths = Vec::new();
    let mut matched_symbols = Vec::new();
    let mut matched_edge_kinds = Vec::new();

    match query.kind {
        EpiphanyGraphQueryKind::Node => {}
        EpiphanyGraphQueryKind::Path => {
            collect_graph_query_path_matches(
                state,
                query,
                &mut node_ids,
                &mut edge_ids,
                &mut matched_paths,
                &mut matched_symbols,
            );
        }
        EpiphanyGraphQueryKind::FrontierNeighborhood => {
            if let Some(frontier) = state.graph_frontier.as_ref() {
                extend_unique_strings(&mut node_ids, frontier.active_node_ids.iter().cloned());
                extend_unique_strings(&mut edge_ids, frontier.active_edge_ids.iter().cloned());
            }
            expand_graph_query_neighbors(state, query, &mut node_ids, &mut edge_ids);
        }
        EpiphanyGraphQueryKind::Neighbors => {
            expand_graph_query_neighbors(state, query, &mut node_ids, &mut edge_ids);
        }
    }

    collect_graph_query_edge_kind_matches(
        state,
        query,
        &mut node_ids,
        &mut edge_ids,
        &mut matched_edge_kinds,
    );

    if query.include_links.unwrap_or(true) {
        expand_graph_query_links(state, &mut node_ids);
    }

    let node_id_set: HashSet<&str> = node_ids.iter().map(String::as_str).collect();
    let edge_id_set: HashSet<&str> = edge_ids.iter().map(String::as_str).collect();

    let architecture_nodes = state
        .graphs
        .architecture
        .nodes
        .iter()
        .filter(|node| node_id_set.contains(node.id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let dataflow_nodes = state
        .graphs
        .dataflow
        .nodes
        .iter()
        .filter(|node| node_id_set.contains(node.id.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    let found_node_ids: HashSet<String> = architecture_nodes
        .iter()
        .chain(dataflow_nodes.iter())
        .map(|node| node.id.clone())
        .collect();
    let matched_node_ids = architecture_nodes
        .iter()
        .chain(dataflow_nodes.iter())
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();

    let architecture_edges =
        select_graph_edges(&state.graphs.architecture.edges, &node_id_set, &edge_id_set);
    let dataflow_edges =
        select_graph_edges(&state.graphs.dataflow.edges, &node_id_set, &edge_id_set);
    let found_edge_ids: HashSet<String> = architecture_edges
        .iter()
        .chain(dataflow_edges.iter())
        .filter_map(|edge| edge.id.clone())
        .collect();
    let matched_edge_ids = architecture_edges
        .iter()
        .chain(dataflow_edges.iter())
        .filter_map(|edge| edge.id.clone())
        .collect::<Vec<_>>();

    let links = state
        .graphs
        .links
        .iter()
        .filter(|link| {
            node_id_set.contains(link.architecture_node_id.as_str())
                || node_id_set.contains(link.dataflow_node_id.as_str())
        })
        .cloned()
        .collect::<Vec<_>>();

    EpiphanyGraphQueryView {
        state_status: EpiphanyContextStateStatus::Ready,
        state_revision: Some(state.revision),
        graph: EpiphanyGraphContext {
            architecture_nodes,
            architecture_edges,
            dataflow_nodes,
            dataflow_edges,
            links,
        },
        frontier: state.graph_frontier.clone(),
        checkpoint: state.graph_checkpoint.clone(),
        matched: EpiphanyGraphQueryMatched {
            node_ids: matched_node_ids,
            edge_ids: matched_edge_ids,
            paths: matched_paths,
            symbols: matched_symbols,
            edge_kinds: matched_edge_kinds,
        },
        missing: EpiphanyGraphQueryMissing {
            node_ids: node_ids
                .iter()
                .filter(|id| !found_node_ids.contains(*id))
                .cloned()
                .collect(),
            edge_ids: edge_ids
                .iter()
                .filter(|id| !found_edge_ids.contains(*id))
                .cloned()
                .collect(),
        },
    }
}

fn collect_graph_query_path_matches(
    state: &EpiphanyThreadState,
    query: &EpiphanyGraphQuery,
    node_ids: &mut Vec<String>,
    edge_ids: &mut Vec<String>,
    matched_paths: &mut Vec<PathBuf>,
    matched_symbols: &mut Vec<String>,
) {
    if query.paths.is_empty() && query.symbols.is_empty() {
        return;
    }
    for node in state
        .graphs
        .architecture
        .nodes
        .iter()
        .chain(state.graphs.dataflow.nodes.iter())
    {
        if code_refs_match_query(&node.code_refs, query, matched_paths, matched_symbols) {
            extend_unique_strings(node_ids, [node.id.clone()]);
        }
    }
    for edge in state
        .graphs
        .architecture
        .edges
        .iter()
        .chain(state.graphs.dataflow.edges.iter())
    {
        if code_refs_match_query(&edge.code_refs, query, matched_paths, matched_symbols) {
            if let Some(id) = edge.id.clone() {
                extend_unique_strings(edge_ids, [id]);
            }
            extend_unique_strings(node_ids, [edge.source_id.clone(), edge.target_id.clone()]);
        }
    }
    for link in &state.graphs.links {
        if code_refs_match_query(&link.code_refs, query, matched_paths, matched_symbols) {
            extend_unique_strings(
                node_ids,
                [
                    link.dataflow_node_id.clone(),
                    link.architecture_node_id.clone(),
                ],
            );
        }
    }
}

fn collect_graph_query_edge_kind_matches(
    state: &EpiphanyThreadState,
    query: &EpiphanyGraphQuery,
    node_ids: &mut Vec<String>,
    edge_ids: &mut Vec<String>,
    matched_edge_kinds: &mut Vec<String>,
) {
    if query.edge_kinds.is_empty() {
        return;
    }
    let wanted: HashSet<&str> = query.edge_kinds.iter().map(String::as_str).collect();
    for edge in state
        .graphs
        .architecture
        .edges
        .iter()
        .chain(state.graphs.dataflow.edges.iter())
    {
        if wanted.contains(edge.kind.as_str()) {
            extend_unique_strings(matched_edge_kinds, [edge.kind.clone()]);
            if let Some(id) = edge.id.clone() {
                extend_unique_strings(edge_ids, [id]);
            }
            extend_unique_strings(node_ids, [edge.source_id.clone(), edge.target_id.clone()]);
        }
    }
}

fn expand_graph_query_neighbors(
    state: &EpiphanyThreadState,
    query: &EpiphanyGraphQuery,
    node_ids: &mut Vec<String>,
    edge_ids: &mut Vec<String>,
) {
    let direction = query.direction.unwrap_or(EpiphanyGraphQueryDirection::Both);
    let depth = query.depth.unwrap_or(1).clamp(1, 3);
    for _ in 0..depth {
        let current: HashSet<String> = node_ids.iter().cloned().collect();
        let before = node_ids.len();
        for edge in state
            .graphs
            .architecture
            .edges
            .iter()
            .chain(state.graphs.dataflow.edges.iter())
        {
            let source_selected = current.contains(&edge.source_id);
            let target_selected = current.contains(&edge.target_id);
            let include_outgoing = matches!(
                direction,
                EpiphanyGraphQueryDirection::Outgoing | EpiphanyGraphQueryDirection::Both
            ) && source_selected;
            let include_incoming = matches!(
                direction,
                EpiphanyGraphQueryDirection::Incoming | EpiphanyGraphQueryDirection::Both
            ) && target_selected;
            if include_outgoing || include_incoming {
                if let Some(id) = edge.id.clone() {
                    extend_unique_strings(edge_ids, [id]);
                }
            }
            if include_outgoing {
                extend_unique_strings(node_ids, [edge.target_id.clone()]);
            }
            if include_incoming {
                extend_unique_strings(node_ids, [edge.source_id.clone()]);
            }
        }
        if node_ids.len() == before {
            break;
        }
    }
}

fn expand_graph_query_links(state: &EpiphanyThreadState, node_ids: &mut Vec<String>) {
    let current: HashSet<&str> = node_ids.iter().map(String::as_str).collect();
    let mut linked = Vec::new();
    for link in &state.graphs.links {
        if current.contains(link.dataflow_node_id.as_str()) {
            linked.push(link.architecture_node_id.clone());
        }
        if current.contains(link.architecture_node_id.as_str()) {
            linked.push(link.dataflow_node_id.clone());
        }
    }
    extend_unique_strings(node_ids, linked);
}

fn code_refs_match_query(
    code_refs: &[EpiphanyCodeRef],
    query: &EpiphanyGraphQuery,
    matched_paths: &mut Vec<PathBuf>,
    matched_symbols: &mut Vec<String>,
) -> bool {
    code_refs.iter().any(|code_ref| {
        let path_matches = query.paths.is_empty()
            || query.paths.iter().any(|path| {
                code_ref.path == *path
                    || code_ref.path.ends_with(path)
                    || path.ends_with(code_ref.path.as_path())
            });
        let symbol_matches = query.symbols.is_empty()
            || code_ref
                .symbol
                .as_deref()
                .is_some_and(|symbol| query.symbols.iter().any(|wanted| wanted == symbol));
        if path_matches && symbol_matches {
            if !query.paths.is_empty() {
                extend_unique_paths(matched_paths, [code_ref.path.clone()]);
            }
            if let Some(symbol) = code_ref.symbol.clone()
                && !query.symbols.is_empty()
            {
                extend_unique_strings(matched_symbols, [symbol]);
            }
            true
        } else {
            false
        }
    })
}

fn extend_unique_paths(target: &mut Vec<PathBuf>, values: impl IntoIterator<Item = PathBuf>) {
    for value in values {
        if !target.iter().any(|existing| existing == &value) {
            target.push(value);
        }
    }
}

fn select_graph_edges(
    edges: &[EpiphanyGraphEdge],
    graph_node_id_set: &HashSet<&str>,
    graph_edge_id_set: &HashSet<&str>,
) -> Vec<EpiphanyGraphEdge> {
    edges
        .iter()
        .filter(|edge| {
            edge.id
                .as_deref()
                .is_some_and(|id| graph_edge_id_set.contains(id))
                || graph_node_id_set.contains(edge.source_id.as_str())
                || graph_node_id_set.contains(edge.target_id.as_str())
        })
        .cloned()
        .collect()
}

fn unique_strings(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut unique = Vec::new();
    extend_unique_strings(&mut unique, values);
    unique
}

fn extend_unique_strings(target: &mut Vec<String>, values: impl IntoIterator<Item = String>) {
    for value in values {
        if !target.iter().any(|existing| existing == &value) {
            target.push(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph_state() -> EpiphanyThreadState {
        EpiphanyThreadState {
            revision: 9,
            graphs: codex_protocol::protocol::EpiphanyGraphs {
                architecture: codex_protocol::protocol::EpiphanyGraph {
                    nodes: vec![
                        EpiphanyGraphNode {
                            id: "a".to_string(),
                            title: "A".to_string(),
                            purpose: "source".to_string(),
                            mechanism: None,
                            metaphor: None,
                            status: None,
                            code_refs: vec![EpiphanyCodeRef {
                                path: PathBuf::from("src/a.rs"),
                                start_line: None,
                                end_line: None,
                                symbol: Some("Alpha".to_string()),
                                note: None,
                            }],
                        },
                        EpiphanyGraphNode {
                            id: "b".to_string(),
                            title: "B".to_string(),
                            purpose: "target".to_string(),
                            mechanism: None,
                            metaphor: None,
                            status: None,
                            code_refs: Vec::new(),
                        },
                    ],
                    edges: vec![EpiphanyGraphEdge {
                        source_id: "a".to_string(),
                        target_id: "b".to_string(),
                        kind: "calls".to_string(),
                        id: Some("edge-1".to_string()),
                        label: None,
                        mechanism: None,
                        code_refs: Vec::new(),
                    }],
                },
                ..Default::default()
            },
            graph_frontier: Some(EpiphanyGraphFrontier {
                active_node_ids: vec!["a".to_string()],
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn context_includes_active_frontier() {
        let state = graph_state();
        let view = derive_context(
            Some(&state),
            &EpiphanyContextParams {
                include_active_frontier: Some(true),
                ..Default::default()
            },
        );

        assert_eq!(view.state_status, EpiphanyContextStateStatus::Ready);
        assert_eq!(view.state_revision, Some(9));
        assert_eq!(view.context.graph.architecture_nodes[0].id, "a");
        assert!(view.missing.graph_node_ids.is_empty());
    }

    #[test]
    fn graph_query_expands_neighbors() {
        let state = graph_state();
        let view = derive_graph_query(
            Some(&state),
            &EpiphanyGraphQuery {
                kind: EpiphanyGraphQueryKind::Neighbors,
                node_ids: vec!["a".to_string()],
                edge_ids: Vec::new(),
                paths: Vec::new(),
                symbols: Vec::new(),
                edge_kinds: Vec::new(),
                direction: Some(EpiphanyGraphQueryDirection::Outgoing),
                depth: Some(1),
                include_links: Some(false),
            },
        );

        assert_eq!(view.state_status, EpiphanyContextStateStatus::Ready);
        assert_eq!(view.graph.architecture_nodes.len(), 2);
        assert_eq!(view.matched.edge_ids, vec!["edge-1".to_string()]);
    }

    #[test]
    fn graph_query_matches_paths_and_symbols() {
        let state = graph_state();
        let view = derive_graph_query(
            Some(&state),
            &EpiphanyGraphQuery {
                kind: EpiphanyGraphQueryKind::Path,
                node_ids: Vec::new(),
                edge_ids: Vec::new(),
                paths: vec![PathBuf::from("src/a.rs")],
                symbols: vec!["Alpha".to_string()],
                edge_kinds: Vec::new(),
                direction: None,
                depth: None,
                include_links: Some(false),
            },
        );

        assert_eq!(view.matched.node_ids, vec!["a".to_string()]);
        assert_eq!(view.matched.symbols, vec!["Alpha".to_string()]);
    }
}
