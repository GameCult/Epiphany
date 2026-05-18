use crate::memory_graph::EpiphanyMemoryAnchor;
use crate::memory_graph::EpiphanyMemoryDomain;
use crate::memory_graph::EpiphanyMemoryEdge;
use crate::memory_graph::EpiphanyMemoryEdgeKind;
use crate::memory_graph::EpiphanyMemoryFreshness;
use crate::memory_graph::EpiphanyMemoryFreshnessStatus;
use crate::memory_graph::EpiphanyMemoryGraphSnapshot;
use crate::memory_graph::EpiphanyMemoryLifecycle;
use crate::memory_graph::EpiphanyMemoryNode;
use crate::memory_graph::EpiphanyMemoryNodeKind;
use crate::memory_graph::EpiphanyMemoryProfile;
use crate::memory_graph::EpiphanyMemorySummary;
use crate::memory_graph::memory_graph_domain_id;
use crate::memory_graph::memory_graph_edge_id;
use crate::memory_graph::memory_graph_node_id;
use epiphany_state_model::EpiphanyCodeRef;
use epiphany_state_model::EpiphanyGraph;
use epiphany_state_model::EpiphanyGraphEdge;
use epiphany_state_model::EpiphanyGraphLink;
use epiphany_state_model::EpiphanyGraphNode;
use epiphany_state_model::EpiphanyGraphs;
use std::collections::HashMap;

pub fn memory_graph_from_epiphany_graphs(
    graph_id: impl Into<String>,
    graphs: &EpiphanyGraphs,
) -> EpiphanyMemoryGraphSnapshot {
    let architecture_domain = memory_graph_domain_id(
        EpiphanyMemoryProfile::RepoArchitecture,
        "accepted_graph",
        "architecture",
    );
    let dataflow_domain = memory_graph_domain_id(
        EpiphanyMemoryProfile::RepoDataflow,
        "accepted_graph",
        "dataflow",
    );

    let mut domains = Vec::new();
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut summaries = Vec::new();
    let mut node_map = HashMap::new();

    if !graphs.architecture.is_empty() {
        domains.push(repo_domain(
            architecture_domain.clone(),
            EpiphanyMemoryProfile::RepoArchitecture,
            "Accepted architecture graph",
        ));
        import_graph_nodes(
            &graphs.architecture,
            &architecture_domain,
            EpiphanyMemoryProfile::RepoArchitecture,
            &mut node_map,
            &mut nodes,
        );
        import_graph_edges(
            &graphs.architecture,
            EpiphanyMemoryProfile::RepoArchitecture,
            &node_map,
            &mut edges,
        );
        summaries.push(repo_summary(
            "summary-accepted-architecture-graph",
            &architecture_domain,
            "accepted architecture graph",
            "Accepted architecture graph nodes are available as memory graph repo-profile claims.",
            &nodes,
            &edges,
            EpiphanyMemoryProfile::RepoArchitecture,
        ));
    }

    if !graphs.dataflow.is_empty() {
        domains.push(repo_domain(
            dataflow_domain.clone(),
            EpiphanyMemoryProfile::RepoDataflow,
            "Accepted dataflow graph",
        ));
        import_graph_nodes(
            &graphs.dataflow,
            &dataflow_domain,
            EpiphanyMemoryProfile::RepoDataflow,
            &mut node_map,
            &mut nodes,
        );
        import_graph_edges(
            &graphs.dataflow,
            EpiphanyMemoryProfile::RepoDataflow,
            &node_map,
            &mut edges,
        );
        summaries.push(repo_summary(
            "summary-accepted-dataflow-graph",
            &dataflow_domain,
            "accepted dataflow graph",
            "Accepted dataflow graph nodes are available as memory graph repo-profile claims.",
            &nodes,
            &edges,
            EpiphanyMemoryProfile::RepoDataflow,
        ));
    }

    import_graph_links(graphs.links.as_slice(), &node_map, &mut edges);

    EpiphanyMemoryGraphSnapshot {
        schema_version: Some("epiphany.memory_graph.v0".to_string()),
        graph_id: graph_id.into(),
        domains,
        nodes,
        edges,
        summaries,
        freshness: Some(EpiphanyMemoryFreshness {
            status: EpiphanyMemoryFreshnessStatus::Ready,
            note: Some("Imported from accepted Epiphany graph state.".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub fn epiphany_graphs_from_memory_graph(snapshot: &EpiphanyMemoryGraphSnapshot) -> EpiphanyGraphs {
    let architecture = EpiphanyGraph {
        nodes: snapshot
            .nodes
            .iter()
            .filter(|node| node.profile == EpiphanyMemoryProfile::RepoArchitecture)
            .map(graph_node_from_memory_node)
            .collect(),
        edges: snapshot
            .edges
            .iter()
            .filter(|edge| edge.profile == EpiphanyMemoryProfile::RepoArchitecture)
            .map(graph_edge_from_memory_edge)
            .collect(),
    };
    let dataflow = EpiphanyGraph {
        nodes: snapshot
            .nodes
            .iter()
            .filter(|node| node.profile == EpiphanyMemoryProfile::RepoDataflow)
            .map(graph_node_from_memory_node)
            .collect(),
        edges: snapshot
            .edges
            .iter()
            .filter(|edge| edge.profile == EpiphanyMemoryProfile::RepoDataflow)
            .map(graph_edge_from_memory_edge)
            .collect(),
    };
    EpiphanyGraphs {
        architecture,
        dataflow,
        links: repo_graph_links_from_memory_graph(snapshot),
    }
}

fn graph_node_from_memory_node(node: &EpiphanyMemoryNode) -> EpiphanyGraphNode {
    EpiphanyGraphNode {
        id: node.id.clone(),
        title: node.title.clone(),
        purpose: node.claim.clone(),
        mechanism: (!node.tension.trim().is_empty()).then_some(node.tension.clone()),
        metaphor: None,
        status: Some(memory_lifecycle_label(node.lifecycle).to_string()),
        code_refs: code_refs_from_anchors(&node.anchors),
    }
}

fn graph_edge_from_memory_edge(edge: &EpiphanyMemoryEdge) -> EpiphanyGraphEdge {
    EpiphanyGraphEdge {
        source_id: edge.source_id.clone(),
        target_id: edge.target_id.clone(),
        kind: memory_edge_kind_label(edge.kind).to_string(),
        id: Some(edge.id.clone()),
        label: Some(edge.claim.clone()),
        mechanism: None,
        code_refs: code_refs_from_anchors(&edge.anchors),
    }
}

fn repo_graph_links_from_memory_graph(
    snapshot: &EpiphanyMemoryGraphSnapshot,
) -> Vec<EpiphanyGraphLink> {
    snapshot
        .edges
        .iter()
        .filter(|edge| edge.profile == EpiphanyMemoryProfile::RepoDataflow)
        .filter_map(|edge| {
            let architecture_node_id = edge
                .anchors
                .iter()
                .find(|anchor| anchor.kind == "architecture_node")
                .map(|anchor| anchor.target.clone())?;
            Some(EpiphanyGraphLink {
                dataflow_node_id: edge.source_id.clone(),
                architecture_node_id,
                relationship: Some(edge.claim.clone()),
                code_refs: code_refs_from_anchors(&edge.anchors),
            })
        })
        .collect()
}

fn code_refs_from_anchors(anchors: &[EpiphanyMemoryAnchor]) -> Vec<EpiphanyCodeRef> {
    anchors
        .iter()
        .filter_map(|anchor| anchor.code_ref.clone())
        .collect()
}

fn memory_edge_kind_label(kind: EpiphanyMemoryEdgeKind) -> &'static str {
    match kind {
        EpiphanyMemoryEdgeKind::Owns => "owns",
        EpiphanyMemoryEdgeKind::Reads => "reads",
        EpiphanyMemoryEdgeKind::Writes => "writes",
        EpiphanyMemoryEdgeKind::Derives => "derives",
        EpiphanyMemoryEdgeKind::Adapts => "adapts",
        EpiphanyMemoryEdgeKind::Persists => "persists",
        EpiphanyMemoryEdgeKind::Launches => "launches",
        EpiphanyMemoryEdgeKind::Verifies => "verifies",
        EpiphanyMemoryEdgeKind::Supports => "supports",
        EpiphanyMemoryEdgeKind::Contradicts => "contradicts",
        EpiphanyMemoryEdgeKind::Distills => "distills",
        EpiphanyMemoryEdgeKind::Revises => "revises",
        EpiphanyMemoryEdgeKind::Retires => "retires",
        EpiphanyMemoryEdgeKind::Grounds => "grounds",
        EpiphanyMemoryEdgeKind::Triggers => "triggers",
        EpiphanyMemoryEdgeKind::SpokenAs => "spoken_as",
        EpiphanyMemoryEdgeKind::Cools => "cools",
        EpiphanyMemoryEdgeKind::ClustersWith => "clusters_with",
        EpiphanyMemoryEdgeKind::ResonatesWith => "resonates_with",
        EpiphanyMemoryEdgeKind::DependsOn => "depends_on",
        EpiphanyMemoryEdgeKind::Other => "other",
    }
}

fn memory_lifecycle_label(lifecycle: EpiphanyMemoryLifecycle) -> &'static str {
    match lifecycle {
        EpiphanyMemoryLifecycle::Observed => "observed",
        EpiphanyMemoryLifecycle::Proposed => "proposed",
        EpiphanyMemoryLifecycle::Accepted => "accepted",
        EpiphanyMemoryLifecycle::Active => "active",
        EpiphanyMemoryLifecycle::Clustered => "clustered",
        EpiphanyMemoryLifecycle::Distilled => "distilled",
        EpiphanyMemoryLifecycle::Incubated => "incubated",
        EpiphanyMemoryLifecycle::Pruned => "pruned",
        EpiphanyMemoryLifecycle::Revised => "revised",
        EpiphanyMemoryLifecycle::Retired => "retired",
        EpiphanyMemoryLifecycle::Crystallized => "crystallized",
        EpiphanyMemoryLifecycle::Stale => "stale",
        EpiphanyMemoryLifecycle::Deepening => "deepening",
        EpiphanyMemoryLifecycle::Cooling => "cooling",
        EpiphanyMemoryLifecycle::Promoted => "promoted",
        EpiphanyMemoryLifecycle::Queued => "queued",
        EpiphanyMemoryLifecycle::Deferred => "deferred",
        EpiphanyMemoryLifecycle::Spoken => "spoken",
        EpiphanyMemoryLifecycle::Applied => "applied",
        EpiphanyMemoryLifecycle::Obligated => "obligated",
        EpiphanyMemoryLifecycle::Answered => "answered",
        EpiphanyMemoryLifecycle::Reviewed => "reviewed",
        EpiphanyMemoryLifecycle::Contradicted => "contradicted",
        EpiphanyMemoryLifecycle::Superseded => "superseded",
    }
}

fn repo_domain(id: String, profile: EpiphanyMemoryProfile, title: &str) -> EpiphanyMemoryDomain {
    EpiphanyMemoryDomain {
        id,
        profile,
        title: title.to_string(),
        description: Some("Repo profile imported from accepted Epiphany graph state.".to_string()),
        lifecycle: EpiphanyMemoryLifecycle::Accepted,
    }
}

fn import_graph_nodes(
    graph: &EpiphanyGraph,
    domain_id: &str,
    profile: EpiphanyMemoryProfile,
    node_map: &mut HashMap<String, String>,
    nodes: &mut Vec<EpiphanyMemoryNode>,
) {
    for node in &graph.nodes {
        let memory_id = memory_graph_node_id(domain_id, "accepted_graph_node", &node.id, None);
        node_map.insert(node.id.clone(), memory_id.clone());
        nodes.push(memory_node_from_graph_node(
            memory_id, domain_id, profile, node,
        ));
    }
}

fn memory_node_from_graph_node(
    id: String,
    domain_id: &str,
    profile: EpiphanyMemoryProfile,
    node: &EpiphanyGraphNode,
) -> EpiphanyMemoryNode {
    let anchors = anchors_from_code_refs(&id, &node.code_refs);
    let source_hashes = if anchors.is_empty() {
        vec!["anchor:missing".to_string()]
    } else {
        anchors.iter().map(|anchor| anchor.id.clone()).collect()
    };
    EpiphanyMemoryNode {
        id,
        domain_id: domain_id.to_string(),
        profile,
        kind: memory_node_kind(node),
        title: node.title.clone(),
        claim: graph_node_claim(node),
        question: "What changes if this accepted graph node is stale?".to_string(),
        tension: node.mechanism.clone().unwrap_or_default(),
        action_implication: format!(
            "Use accepted graph node {} when selecting repo context.",
            node.id
        ),
        anchors,
        source_hashes,
        lifecycle: graph_status_lifecycle(node.status.as_deref()),
        salience: 70,
        confidence: 80,
        ..Default::default()
    }
}

fn import_graph_edges(
    graph: &EpiphanyGraph,
    profile: EpiphanyMemoryProfile,
    node_map: &HashMap<String, String>,
    edges: &mut Vec<EpiphanyMemoryEdge>,
) {
    for edge in &graph.edges {
        let source_id = node_map
            .get(&edge.source_id)
            .cloned()
            .unwrap_or_else(|| edge.source_id.clone());
        let target_id = node_map
            .get(&edge.target_id)
            .cloned()
            .unwrap_or_else(|| edge.target_id.clone());
        edges.push(memory_edge_from_graph_edge(
            source_id, target_id, profile, edge,
        ));
    }
}

fn memory_edge_from_graph_edge(
    source_id: String,
    target_id: String,
    profile: EpiphanyMemoryProfile,
    edge: &EpiphanyGraphEdge,
) -> EpiphanyMemoryEdge {
    let id = edge.id.clone().unwrap_or_else(|| {
        memory_graph_edge_id(
            &source_id,
            &target_id,
            edge.kind.as_str(),
            edge.code_refs.iter().map(code_ref_key),
        )
    });
    EpiphanyMemoryEdge {
        id,
        source_id,
        target_id,
        kind: memory_edge_kind(edge.kind.as_str()),
        profile,
        claim: graph_edge_claim(edge),
        anchors: anchors_from_code_refs(edge.id.as_deref().unwrap_or("edge"), &edge.code_refs),
        lifecycle: EpiphanyMemoryLifecycle::Accepted,
        confidence: 80,
    }
}

fn import_graph_links(
    links: &[EpiphanyGraphLink],
    node_map: &HashMap<String, String>,
    edges: &mut Vec<EpiphanyMemoryEdge>,
) {
    for link in links {
        let Some(source_id) = node_map.get(&link.dataflow_node_id).cloned() else {
            continue;
        };
        let Some(target_id) = node_map.get(&link.architecture_node_id).cloned() else {
            continue;
        };
        edges.push(EpiphanyMemoryEdge {
            id: memory_graph_edge_id(
                &source_id,
                &target_id,
                "grounds",
                link.code_refs.iter().map(code_ref_key),
            ),
            source_id,
            target_id,
            kind: EpiphanyMemoryEdgeKind::Grounds,
            profile: EpiphanyMemoryProfile::RepoDataflow,
            claim: link
                .relationship
                .clone()
                .unwrap_or_else(|| "Dataflow node is linked to architecture node.".to_string()),
            anchors: anchors_from_code_refs("link", &link.code_refs),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            confidence: 75,
        });
    }
}

fn repo_summary(
    id: &str,
    domain_id: &str,
    target: &str,
    claim: &str,
    nodes: &[EpiphanyMemoryNode],
    edges: &[EpiphanyMemoryEdge],
    profile: EpiphanyMemoryProfile,
) -> EpiphanyMemorySummary {
    let covers_node_ids = nodes
        .iter()
        .filter(|node| node.profile == profile)
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    let covers_edge_ids = edges
        .iter()
        .filter(|edge| edge.profile == profile)
        .map(|edge| edge.id.clone())
        .collect::<Vec<_>>();
    let anchor_count = nodes
        .iter()
        .filter(|node| node.profile == profile)
        .map(|node| node.anchors.len() as u32)
        .sum();
    EpiphanyMemorySummary {
        id: id.to_string(),
        domain_id: domain_id.to_string(),
        covers_node_ids,
        covers_edge_ids,
        target: target.to_string(),
        claim: claim.to_string(),
        question: "Which profile producer should refresh this accepted graph next?".to_string(),
        tension: String::new(),
        action_implication: "Use this summary for broad repo context; descend when exact nodes are stale or relevant.".to_string(),
        anchor_count,
        freshness: EpiphanyMemoryFreshnessStatus::Ready,
        confidence: 80,
        ..Default::default()
    }
}

fn anchors_from_code_refs(
    prefix: &str,
    code_refs: &[EpiphanyCodeRef],
) -> Vec<EpiphanyMemoryAnchor> {
    code_refs
        .iter()
        .enumerate()
        .map(|(index, code_ref)| EpiphanyMemoryAnchor {
            id: format!("anchor-{prefix}-{index}"),
            kind: "code_ref".to_string(),
            target: code_ref_key(code_ref),
            code_ref: Some(code_ref.clone()),
            source_hash: Some(code_ref_key(code_ref)),
            ..Default::default()
        })
        .collect()
}

fn code_ref_key(code_ref: &EpiphanyCodeRef) -> String {
    let mut key = code_ref.path.to_string_lossy().replace('\\', "/");
    if let Some(symbol) = code_ref.symbol.as_deref() {
        key.push('#');
        key.push_str(symbol);
    }
    key
}

fn graph_node_claim(node: &EpiphanyGraphNode) -> String {
    if !node.purpose.trim().is_empty() {
        return node.purpose.clone();
    }
    format!("Accepted graph node {} exists.", node.id)
}

fn graph_edge_claim(edge: &EpiphanyGraphEdge) -> String {
    edge.mechanism
        .clone()
        .or_else(|| edge.label.clone())
        .unwrap_or_else(|| {
            format!(
                "Accepted graph edge {} connects {} to {}.",
                edge.kind, edge.source_id, edge.target_id
            )
        })
}

fn graph_status_lifecycle(status: Option<&str>) -> EpiphanyMemoryLifecycle {
    match status.unwrap_or_default() {
        "observed" => EpiphanyMemoryLifecycle::Observed,
        "proposed" => EpiphanyMemoryLifecycle::Proposed,
        "stale" => EpiphanyMemoryLifecycle::Stale,
        "retired" => EpiphanyMemoryLifecycle::Retired,
        _ => EpiphanyMemoryLifecycle::Accepted,
    }
}

fn memory_node_kind(node: &EpiphanyGraphNode) -> EpiphanyMemoryNodeKind {
    let text = format!("{} {}", node.title, node.purpose).to_lowercase();
    if text.contains("schema") {
        EpiphanyMemoryNodeKind::Schema
    } else if text.contains("runtime") || text.contains("contract") {
        EpiphanyMemoryNodeKind::RuntimeContract
    } else if text.contains("adapter") || text.contains("bridge") {
        EpiphanyMemoryNodeKind::Adapter
    } else if text.contains("test") || text.contains("smoke") {
        EpiphanyMemoryNodeKind::TestSeam
    } else if text.contains("state") || text.contains("store") {
        EpiphanyMemoryNodeKind::StateStore
    } else {
        EpiphanyMemoryNodeKind::Module
    }
}

fn memory_edge_kind(kind: &str) -> EpiphanyMemoryEdgeKind {
    match kind {
        "owns" => EpiphanyMemoryEdgeKind::Owns,
        "reads" => EpiphanyMemoryEdgeKind::Reads,
        "writes" => EpiphanyMemoryEdgeKind::Writes,
        "derives" => EpiphanyMemoryEdgeKind::Derives,
        "adapts" => EpiphanyMemoryEdgeKind::Adapts,
        "persists" => EpiphanyMemoryEdgeKind::Persists,
        "launches" => EpiphanyMemoryEdgeKind::Launches,
        "verifies" | "tests" => EpiphanyMemoryEdgeKind::Verifies,
        "depends_on" | "depends" => EpiphanyMemoryEdgeKind::DependsOn,
        _ => EpiphanyMemoryEdgeKind::Other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_graph::validate_memory_graph_snapshot;
    use epiphany_state_model::EpiphanyCodeRef;
    use epiphany_state_model::EpiphanyGraph;
    use epiphany_state_model::EpiphanyGraphEdge;
    use epiphany_state_model::EpiphanyGraphNode;
    use epiphany_state_model::EpiphanyGraphs;
    use std::path::PathBuf;

    #[test]
    fn repo_profile_imports_accepted_graphs_into_valid_memory_graph() {
        let graphs = EpiphanyGraphs {
            architecture: EpiphanyGraph {
                nodes: vec![EpiphanyGraphNode {
                    id: "core".to_string(),
                    title: "Core policy".to_string(),
                    purpose: "Owns shared policy.".to_string(),
                    code_refs: vec![EpiphanyCodeRef {
                        path: PathBuf::from("epiphany-core/src/lib.rs"),
                        symbol: Some("policy".to_string()),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                edges: vec![EpiphanyGraphEdge {
                    id: Some("edge-core-self".to_string()),
                    source_id: "core".to_string(),
                    target_id: "core".to_string(),
                    kind: "owns".to_string(),
                    label: Some("owns itself for fixture".to_string()),
                    ..Default::default()
                }],
            },
            ..Default::default()
        };

        let snapshot = memory_graph_from_epiphany_graphs("repo-profile", &graphs);
        let errors = validate_memory_graph_snapshot(&snapshot);

        assert!(errors.is_empty(), "{errors:?}");
        assert_eq!(snapshot.domains.len(), 1);
        assert_eq!(snapshot.nodes.len(), 1);
        assert_eq!(snapshot.edges.len(), 1);
        assert_eq!(snapshot.summaries.len(), 1);
        assert_eq!(
            snapshot.nodes[0].profile,
            EpiphanyMemoryProfile::RepoArchitecture
        );
    }

    #[test]
    fn repo_profile_projects_memory_graph_back_to_legacy_graph_view() {
        let graphs = EpiphanyGraphs {
            architecture: EpiphanyGraph {
                nodes: vec![EpiphanyGraphNode {
                    id: "core".to_string(),
                    title: "Core policy".to_string(),
                    purpose: "Owns shared policy.".to_string(),
                    code_refs: vec![EpiphanyCodeRef {
                        path: PathBuf::from("epiphany-core/src/lib.rs"),
                        symbol: Some("policy".to_string()),
                        ..Default::default()
                    }],
                    ..Default::default()
                }],
                edges: vec![EpiphanyGraphEdge {
                    id: Some("edge-core-self".to_string()),
                    source_id: "core".to_string(),
                    target_id: "core".to_string(),
                    kind: "owns".to_string(),
                    label: Some("owns itself for fixture".to_string()),
                    ..Default::default()
                }],
            },
            ..Default::default()
        };

        let snapshot = memory_graph_from_epiphany_graphs("repo-profile", &graphs);
        let projected = epiphany_graphs_from_memory_graph(&snapshot);

        assert_eq!(projected.architecture.nodes.len(), 1);
        assert_eq!(projected.architecture.nodes[0].title, "Core policy");
        assert_eq!(projected.architecture.nodes[0].code_refs.len(), 1);
        assert_eq!(projected.architecture.edges.len(), 1);
        assert_eq!(projected.architecture.edges[0].kind, "owns");
    }

    #[test]
    fn repo_profile_keeps_bad_topology_visible() {
        let graphs = EpiphanyGraphs {
            architecture: EpiphanyGraph {
                edges: vec![EpiphanyGraphEdge {
                    source_id: "missing-source".to_string(),
                    target_id: "missing-target".to_string(),
                    kind: "owns".to_string(),
                    ..Default::default()
                }],
                ..Default::default()
            },
            ..Default::default()
        };

        let snapshot = memory_graph_from_epiphany_graphs("repo-profile", &graphs);
        let errors = validate_memory_graph_snapshot(&snapshot);

        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("missing source node"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.message.contains("missing target node"))
        );
    }
}
