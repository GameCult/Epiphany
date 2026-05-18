use super::EpiphanyMemoryContextPacket;
use super::EpiphanyMemoryContextQuery;
use super::EpiphanyMemoryEdge;
use super::EpiphanyMemoryFreshnessStatus;
use super::EpiphanyMemoryGraphSemanticHit;
use super::EpiphanyMemoryGraphSnapshot;
use super::EpiphanyMemoryNode;
use super::EpiphanyMemoryProfile;
use super::EpiphanyMemorySummary;
use super::derive_memory_graph_freshness;
use super::ids::stable_memory_graph_id;
use super::plan_memory_graph_context_cut;
use super::query_memory_graph_semantic_cache;
use std::collections::HashMap;
use std::collections::HashSet;

pub fn plan_memory_graph_context_cut_with_semantic_cache(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    query: &EpiphanyMemoryContextQuery,
) -> EpiphanyMemoryContextPacket {
    let hits = match query_memory_graph_semantic_cache(snapshot, query) {
        Ok(hits) => hits,
        Err(err) => {
            let mut packet = plan_memory_graph_context_cut(snapshot, query);
            packet.warnings.push(format!(
                "semantic cache unavailable; used typed graph traversal: {err}"
            ));
            return packet;
        }
    };
    plan_memory_graph_context_cut_from_semantic_hits(snapshot, query, hits)
}

fn plan_memory_graph_context_cut_from_semantic_hits(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    query: &EpiphanyMemoryContextQuery,
    hits: Vec<EpiphanyMemoryGraphSemanticHit>,
) -> EpiphanyMemoryContextPacket {
    if hits.is_empty() {
        return plan_memory_graph_context_cut(snapshot, query);
    }

    let packet = plan_ranked_semantic_context_packet(snapshot, query, &hits);
    if packet.nodes.is_empty() && packet.edges.is_empty() && packet.summaries.is_empty() {
        return plan_memory_graph_context_cut(snapshot, query);
    }
    packet
}

fn plan_ranked_semantic_context_packet(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    query: &EpiphanyMemoryContextQuery,
    hits: &[EpiphanyMemoryGraphSemanticHit],
) -> EpiphanyMemoryContextPacket {
    let budget = query.budget.unwrap_or(12).clamp(1, 64) as usize;
    let domains = query
        .domain_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let node_by_id = snapshot
        .nodes
        .iter()
        .map(|node| (node.id.as_str(), node))
        .collect::<HashMap<_, _>>();
    let edge_by_id = snapshot
        .edges
        .iter()
        .map(|edge| (edge.id.as_str(), edge))
        .collect::<HashMap<_, _>>();
    let summary_by_id = snapshot
        .summaries
        .iter()
        .map(|summary| (summary.id.as_str(), summary))
        .collect::<HashMap<_, _>>();
    let freshness = snapshot
        .freshness
        .clone()
        .unwrap_or_else(|| derive_memory_graph_freshness(snapshot, &[]));
    let stale_summaries = freshness
        .stale_summary_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut summaries = Vec::new();
    let mut warnings = Vec::new();

    for hit in hits {
        if total_items(&nodes, &edges, &summaries) >= budget {
            break;
        }
        match hit.document_kind.as_str() {
            "node" => {
                if let Some(node) = node_by_id.get(hit.source_id.as_str()) {
                    if node_allowed(node, query.profile, &domains) {
                        push_ranked_node(&mut nodes, (*node).clone(), budget);
                    }
                } else {
                    warnings.push(format!("semantic cache hit missing node {}", hit.source_id));
                }
            }
            "edge" => {
                if let Some(edge) = edge_by_id.get(hit.source_id.as_str()) {
                    if edge_allowed(edge, query.profile) {
                        push_ranked_edge(&mut edges, (*edge).clone(), budget);
                        include_edge_nodes(
                            edge,
                            &node_by_id,
                            query.profile,
                            &domains,
                            budget,
                            &mut nodes,
                        );
                    }
                } else {
                    warnings.push(format!("semantic cache hit missing edge {}", hit.source_id));
                }
            }
            "summary" => {
                if let Some(summary) = summary_by_id.get(hit.source_id.as_str()) {
                    if summary_allowed(summary, query.profile, &domains, &node_by_id) {
                        if summary_is_usable(summary, &stale_summaries) {
                            push_ranked_summary(&mut summaries, (*summary).clone(), budget);
                        } else {
                            include_summary_children(
                                summary,
                                &node_by_id,
                                &edge_by_id,
                                query.profile,
                                &domains,
                                budget,
                                &mut nodes,
                                &mut edges,
                            );
                        }
                    }
                } else {
                    warnings.push(format!(
                        "semantic cache hit missing summary {}",
                        hit.source_id
                    ));
                }
            }
            other => warnings.push(format!(
                "semantic cache hit {} had unknown document kind {other}",
                hit.document_id
            )),
        }
    }

    let anchors = unique_anchors(
        nodes
            .iter()
            .flat_map(|node| node.anchors.iter())
            .chain(edges.iter().flat_map(|edge| edge.anchors.iter())),
    );
    warnings.push(format!(
        "semantic cache seeded {} typed graph document(s)",
        hits.len()
    ));

    EpiphanyMemoryContextPacket {
        id: stable_memory_graph_id("memctx", [query.id.as_str(), snapshot.graph_id.as_str()]),
        query_id: query.id.clone(),
        nodes,
        edges,
        summaries,
        anchors,
        warnings,
        missing_node_ids: query
            .node_ids
            .iter()
            .filter(|id| !node_by_id.contains_key(id.as_str()))
            .cloned()
            .collect(),
        missing_edge_ids: query
            .edge_ids
            .iter()
            .filter(|id| !edge_by_id.contains_key(id.as_str()))
            .cloned()
            .collect(),
    }
}

fn include_edge_nodes(
    edge: &EpiphanyMemoryEdge,
    node_by_id: &HashMap<&str, &EpiphanyMemoryNode>,
    profile: Option<EpiphanyMemoryProfile>,
    domains: &HashSet<&str>,
    budget: usize,
    nodes: &mut Vec<EpiphanyMemoryNode>,
) {
    for node_id in [&edge.source_id, &edge.target_id] {
        if let Some(node) = node_by_id.get(node_id.as_str()) {
            if node_allowed(node, profile, domains) {
                push_ranked_node(nodes, (*node).clone(), budget);
            }
        }
    }
}

fn include_summary_children(
    summary: &EpiphanyMemorySummary,
    node_by_id: &HashMap<&str, &EpiphanyMemoryNode>,
    edge_by_id: &HashMap<&str, &EpiphanyMemoryEdge>,
    profile: Option<EpiphanyMemoryProfile>,
    domains: &HashSet<&str>,
    budget: usize,
    nodes: &mut Vec<EpiphanyMemoryNode>,
    edges: &mut Vec<EpiphanyMemoryEdge>,
) {
    for node_id in &summary.covers_node_ids {
        if let Some(node) = node_by_id.get(node_id.as_str()) {
            if node_allowed(node, profile, domains) {
                push_ranked_node(nodes, (*node).clone(), budget);
            }
        }
    }
    for edge_id in &summary.covers_edge_ids {
        if let Some(edge) = edge_by_id.get(edge_id.as_str()) {
            if edge_allowed(edge, profile) {
                push_ranked_edge(edges, (*edge).clone(), budget);
                include_edge_nodes(edge, node_by_id, profile, domains, budget, nodes);
            }
        }
    }
}

fn node_allowed(
    node: &EpiphanyMemoryNode,
    profile: Option<EpiphanyMemoryProfile>,
    domains: &HashSet<&str>,
) -> bool {
    !profile.is_some_and(|profile| node.profile != profile)
        && (domains.is_empty() || domains.contains(node.domain_id.as_str()))
}

fn edge_allowed(edge: &EpiphanyMemoryEdge, profile: Option<EpiphanyMemoryProfile>) -> bool {
    !profile.is_some_and(|profile| edge.profile != profile)
}

fn summary_allowed(
    summary: &EpiphanyMemorySummary,
    profile: Option<EpiphanyMemoryProfile>,
    domains: &HashSet<&str>,
    node_by_id: &HashMap<&str, &EpiphanyMemoryNode>,
) -> bool {
    if !domains.is_empty() && !domains.contains(summary.domain_id.as_str()) {
        return false;
    }
    if let Some(profile) = profile {
        return summary
            .covers_node_ids
            .iter()
            .filter_map(|id| node_by_id.get(id.as_str()))
            .any(|node| node.profile == profile);
    }
    true
}

fn summary_is_usable(summary: &EpiphanyMemorySummary, stale_summaries: &HashSet<&str>) -> bool {
    summary.freshness == EpiphanyMemoryFreshnessStatus::Ready
        && summary.confidence >= 70
        && !stale_summaries.contains(summary.id.as_str())
        && summary.known_omissions.is_empty()
}

fn total_items(
    nodes: &[EpiphanyMemoryNode],
    edges: &[EpiphanyMemoryEdge],
    summaries: &[EpiphanyMemorySummary],
) -> usize {
    nodes.len() + edges.len() + summaries.len()
}

fn push_ranked_node(target: &mut Vec<EpiphanyMemoryNode>, node: EpiphanyMemoryNode, budget: usize) {
    if target.len() < budget && !target.iter().any(|existing| existing.id == node.id) {
        target.push(node);
    }
}

fn push_ranked_edge(target: &mut Vec<EpiphanyMemoryEdge>, edge: EpiphanyMemoryEdge, budget: usize) {
    if target.len() < budget && !target.iter().any(|existing| existing.id == edge.id) {
        target.push(edge);
    }
}

fn push_ranked_summary(
    target: &mut Vec<EpiphanyMemorySummary>,
    summary: EpiphanyMemorySummary,
    budget: usize,
) {
    if target.len() < budget && !target.iter().any(|existing| existing.id == summary.id) {
        target.push(summary);
    }
}

fn unique_anchors<'a>(
    anchors: impl IntoIterator<Item = &'a super::EpiphanyMemoryAnchor>,
) -> Vec<super::EpiphanyMemoryAnchor> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for anchor in anchors {
        if seen.insert(anchor.id.as_str()) {
            out.push(anchor.clone());
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_graph::EpiphanyMemoryDomain;
    use crate::memory_graph::EpiphanyMemoryLifecycle;
    use crate::memory_graph::EpiphanyMemoryNode;
    use crate::memory_graph::EpiphanyMemoryNodeKind;
    use crate::memory_graph::EpiphanyMemoryProfile;

    #[test]
    fn semantic_context_resolves_cache_hits_through_typed_graph() {
        let snapshot = test_snapshot();
        let packet = plan_memory_graph_context_cut_from_semantic_hits(
            &snapshot,
            &EpiphanyMemoryContextQuery {
                id: "semantic-context-test".to_string(),
                text: Some("memory graph".to_string()),
                budget: Some(3),
                ..Default::default()
            },
            vec![EpiphanyMemoryGraphSemanticHit {
                document_id: "memembed-node-test".to_string(),
                source_id: "memnode-memory-graph".to_string(),
                document_kind: "node".to_string(),
                score: 0.91,
            }],
        );

        assert!(
            packet
                .nodes
                .iter()
                .any(|node| node.id == "memnode-memory-graph")
        );
        assert!(
            packet
                .warnings
                .iter()
                .any(|warning| warning.contains("semantic cache seeded 1"))
        );
    }

    #[test]
    fn semantic_context_preserves_cache_hit_order_before_lexical_fallback() {
        let snapshot = test_snapshot();
        let packet = plan_memory_graph_context_cut_from_semantic_hits(
            &snapshot,
            &EpiphanyMemoryContextQuery {
                id: "semantic-context-order-test".to_string(),
                text: Some("memory graph".to_string()),
                budget: Some(2),
                ..Default::default()
            },
            vec![
                EpiphanyMemoryGraphSemanticHit {
                    document_id: "memembed-node-second".to_string(),
                    source_id: "memnode-validation".to_string(),
                    document_kind: "node".to_string(),
                    score: 0.95,
                },
                EpiphanyMemoryGraphSemanticHit {
                    document_id: "memembed-node-first".to_string(),
                    source_id: "memnode-memory-graph".to_string(),
                    document_kind: "node".to_string(),
                    score: 0.90,
                },
            ],
        );

        assert_eq!(packet.nodes.len(), 2);
        assert_eq!(packet.nodes[0].id, "memnode-validation");
        assert_eq!(packet.nodes[1].id, "memnode-memory-graph");
    }

    fn test_snapshot() -> EpiphanyMemoryGraphSnapshot {
        EpiphanyMemoryGraphSnapshot {
            graph_id: "semantic-context-graph".to_string(),
            domains: vec![EpiphanyMemoryDomain {
                id: "repo".to_string(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                title: "Repo".to_string(),
                description: None,
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
            }],
            nodes: vec![
                EpiphanyMemoryNode {
                    id: "memnode-memory-graph".to_string(),
                    domain_id: "repo".to_string(),
                    profile: EpiphanyMemoryProfile::RepoArchitecture,
                    kind: EpiphanyMemoryNodeKind::Module,
                    title: "Memory graph".to_string(),
                    claim: "Typed graph context owns the real packet.".to_string(),
                    question: String::new(),
                    tension: String::new(),
                    action_implication: String::new(),
                    anchors: Vec::new(),
                    source_hashes: vec!["source-a".to_string()],
                    lifecycle: EpiphanyMemoryLifecycle::Accepted,
                    salience: 80,
                    confidence: 90,
                    created_at: None,
                    updated_at: None,
                },
                EpiphanyMemoryNode {
                    id: "memnode-validation".to_string(),
                    domain_id: "repo".to_string(),
                    profile: EpiphanyMemoryProfile::RepoArchitecture,
                    kind: EpiphanyMemoryNodeKind::TestSeam,
                    title: "Validation".to_string(),
                    claim: "Semantic cache ordering must survive context planning.".to_string(),
                    question: String::new(),
                    tension: String::new(),
                    action_implication: String::new(),
                    anchors: Vec::new(),
                    source_hashes: vec!["source-b".to_string()],
                    lifecycle: EpiphanyMemoryLifecycle::Accepted,
                    salience: 80,
                    confidence: 90,
                    created_at: None,
                    updated_at: None,
                },
            ],
            ..Default::default()
        }
    }
}
