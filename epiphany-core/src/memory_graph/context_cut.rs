use super::EpiphanyMemoryAnchor;
use super::EpiphanyMemoryContextPacket;
use super::EpiphanyMemoryContextQuery;
use super::EpiphanyMemoryEdge;
use super::EpiphanyMemoryFreshnessStatus;
use super::EpiphanyMemoryGraphSnapshot;
use super::EpiphanyMemoryNode;
use super::EpiphanyMemoryProfile;
use super::EpiphanyMemorySummary;
use super::derive_memory_graph_freshness;
use super::ids::normalized_key;
use super::ids::stable_memory_graph_id;
use std::collections::HashMap;
use std::collections::HashSet;

pub fn plan_memory_graph_context_cut(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    query: &EpiphanyMemoryContextQuery,
) -> EpiphanyMemoryContextPacket {
    let budget = query.budget.unwrap_or(12).clamp(1, 64) as usize;
    let dirty_source_hashes = snapshot
        .freshness
        .as_ref()
        .map(|freshness| freshness.dirty_source_hashes.as_slice())
        .unwrap_or(&[]);
    let freshness = derive_memory_graph_freshness(snapshot, dirty_source_hashes);
    let stale_nodes: HashSet<&str> = freshness
        .stale_node_ids
        .iter()
        .map(String::as_str)
        .collect();
    let stale_edges: HashSet<&str> = freshness
        .stale_edge_ids
        .iter()
        .map(String::as_str)
        .collect();
    let stale_summaries: HashSet<&str> = freshness
        .stale_summary_ids
        .iter()
        .map(String::as_str)
        .collect();

    let domains = query
        .domain_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let requested_nodes = query
        .node_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let requested_edges = query
        .edge_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let terms = query_terms(query);

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

    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut summaries = Vec::new();
    let mut warnings = Vec::new();

    for summary in &snapshot.summaries {
        if summaries.len() >= budget {
            break;
        }
        if !summary_matches(summary, query.profile, &domains, &terms, &node_by_id) {
            continue;
        }
        if summary_is_usable(summary, &stale_summaries) {
            summaries.push(summary.clone());
            continue;
        }
        warnings.push(format!(
            "summary {} is stale or low-confidence; descended into children",
            summary.id
        ));
        include_summary_children(
            summary,
            &node_by_id,
            &edge_by_id,
            &stale_nodes,
            &stale_edges,
            budget,
            &mut nodes,
            &mut edges,
        );
    }

    for node_id in &query.node_ids {
        if let Some(node) = node_by_id.get(node_id.as_str()) {
            push_unique_node(&mut nodes, (*node).clone(), budget);
        }
    }
    for edge_id in &query.edge_ids {
        if let Some(edge) = edge_by_id.get(edge_id.as_str()) {
            push_unique_edge(&mut edges, (*edge).clone(), budget);
            if let Some(node) = node_by_id.get(edge.source_id.as_str()) {
                push_unique_node(&mut nodes, (*node).clone(), budget);
            }
            if let Some(node) = node_by_id.get(edge.target_id.as_str()) {
                push_unique_node(&mut nodes, (*node).clone(), budget);
            }
        }
    }

    if summaries.is_empty() && nodes.is_empty() {
        for node in &snapshot.nodes {
            if nodes.len() >= budget {
                break;
            }
            if node_matches(node, query.profile, &domains, &terms) {
                push_unique_node(&mut nodes, node.clone(), budget);
            }
        }
    }

    let anchors = unique_anchors(
        nodes
            .iter()
            .flat_map(|node| node.anchors.iter())
            .chain(edges.iter().flat_map(|edge| edge.anchors.iter())),
    );

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
            .filter(|id| !requested_nodes.is_empty() && !node_by_id.contains_key(id.as_str()))
            .cloned()
            .collect(),
        missing_edge_ids: query
            .edge_ids
            .iter()
            .filter(|id| !requested_edges.is_empty() && !edge_by_id.contains_key(id.as_str()))
            .cloned()
            .collect(),
    }
}

fn query_terms(query: &EpiphanyMemoryContextQuery) -> Vec<String> {
    query
        .text
        .as_deref()
        .unwrap_or_default()
        .split(|ch: char| !ch.is_alphanumeric() && ch != '_')
        .filter(|term| term.len() > 2)
        .map(normalized_key)
        .collect()
}

fn summary_matches(
    summary: &EpiphanyMemorySummary,
    profile: Option<EpiphanyMemoryProfile>,
    domain_ids: &HashSet<&str>,
    terms: &[String],
    node_by_id: &HashMap<&str, &EpiphanyMemoryNode>,
) -> bool {
    if !domain_ids.is_empty() && !domain_ids.contains(summary.domain_id.as_str()) {
        return false;
    }
    if let Some(profile) = profile {
        let covers_profile = summary
            .covers_node_ids
            .iter()
            .filter_map(|id| node_by_id.get(id.as_str()))
            .any(|node| node.profile == profile);
        if !covers_profile {
            return false;
        }
    }
    terms.is_empty()
        || text_matches_terms(
            [
                &summary.target,
                &summary.claim,
                &summary.question,
                &summary.tension,
            ],
            terms,
        )
}

fn node_matches(
    node: &EpiphanyMemoryNode,
    profile: Option<EpiphanyMemoryProfile>,
    domain_ids: &HashSet<&str>,
    terms: &[String],
) -> bool {
    if profile.is_some_and(|profile| node.profile != profile) {
        return false;
    }
    if !domain_ids.is_empty() && !domain_ids.contains(node.domain_id.as_str()) {
        return false;
    }
    terms.is_empty()
        || text_matches_terms(
            [
                &node.title,
                &node.claim,
                &node.question,
                &node.tension,
                &node.action_implication,
            ],
            terms,
        )
}

fn text_matches_terms<'a>(values: impl IntoIterator<Item = &'a String>, terms: &[String]) -> bool {
    let joined = values
        .into_iter()
        .map(|value| value.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");
    terms.iter().any(|term| joined.contains(term))
}

fn summary_is_usable(summary: &EpiphanyMemorySummary, stale_summaries: &HashSet<&str>) -> bool {
    summary.freshness == EpiphanyMemoryFreshnessStatus::Ready
        && summary.confidence >= 70
        && !stale_summaries.contains(summary.id.as_str())
        && summary.known_omissions.is_empty()
}

fn include_summary_children(
    summary: &EpiphanyMemorySummary,
    node_by_id: &HashMap<&str, &EpiphanyMemoryNode>,
    edge_by_id: &HashMap<&str, &EpiphanyMemoryEdge>,
    stale_nodes: &HashSet<&str>,
    stale_edges: &HashSet<&str>,
    budget: usize,
    nodes: &mut Vec<EpiphanyMemoryNode>,
    edges: &mut Vec<EpiphanyMemoryEdge>,
) {
    for node_id in &summary.covers_node_ids {
        if let Some(node) = node_by_id.get(node_id.as_str()) {
            push_unique_node(nodes, (*node).clone(), budget);
            if stale_nodes.contains(node.id.as_str()) {
                continue;
            }
        }
    }
    for edge_id in &summary.covers_edge_ids {
        if let Some(edge) = edge_by_id.get(edge_id.as_str()) {
            push_unique_edge(edges, (*edge).clone(), budget);
            if stale_edges.contains(edge.id.as_str()) {
                continue;
            }
        }
    }
}

fn unique_anchors<'a>(
    anchors: impl IntoIterator<Item = &'a EpiphanyMemoryAnchor>,
) -> Vec<EpiphanyMemoryAnchor> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    for anchor in anchors {
        if seen.insert(anchor.id.as_str()) {
            out.push(anchor.clone());
        }
    }
    out
}

fn push_unique_node(target: &mut Vec<EpiphanyMemoryNode>, node: EpiphanyMemoryNode, budget: usize) {
    if target.len() < budget && !target.iter().any(|existing| existing.id == node.id) {
        target.push(node);
    }
}

fn push_unique_edge(target: &mut Vec<EpiphanyMemoryEdge>, edge: EpiphanyMemoryEdge, budget: usize) {
    if target.len() < budget && !target.iter().any(|existing| existing.id == edge.id) {
        target.push(edge);
    }
}
