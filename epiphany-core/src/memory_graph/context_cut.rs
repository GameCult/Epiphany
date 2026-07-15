use super::EpiphanyMemoryAnchor;
use super::EpiphanyMemoryContextPacket;
use super::EpiphanyMemoryContextQuery;
use super::EpiphanyMemoryEdge;
use super::EpiphanyMemoryFreshnessStatus;
use super::EpiphanyMemoryGraphSnapshot;
use super::EpiphanyMemoryNode;
use super::EpiphanyMemoryProfile;
use super::EpiphanyMemorySummary;
use super::RepoFrontierItem;
use super::RepoFrontierStatus;
use super::SemanticPartition;
use super::derive_memory_graph_freshness;
use super::ids::normalized_key;
use super::ids::stable_memory_graph_id;
use super::memory_graph_model_hash;
use bm25::Document;
use bm25::Language;
use bm25::SearchEngineBuilder;
use std::collections::HashMap;
use std::collections::HashSet;

pub fn plan_memory_graph_context_cut(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    query: &EpiphanyMemoryContextQuery,
) -> EpiphanyMemoryContextPacket {
    plan_memory_graph_context_cut_constrained(snapshot, query, &[], None)
}

pub fn plan_memory_graph_context_cut_with_ranked_ids(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    query: &EpiphanyMemoryContextQuery,
    ranked_document_ids: &[String],
) -> EpiphanyMemoryContextPacket {
    plan_memory_graph_context_cut_constrained(snapshot, query, ranked_document_ids, None)
}

pub(crate) fn plan_memory_graph_context_cut_for_partition(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    query: &EpiphanyMemoryContextQuery,
    ranked_document_ids: &[String],
    partition: SemanticPartition,
) -> EpiphanyMemoryContextPacket {
    plan_memory_graph_context_cut_constrained(snapshot, query, ranked_document_ids, Some(partition))
}

fn plan_memory_graph_context_cut_constrained(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    query: &EpiphanyMemoryContextQuery,
    ranked_document_ids: &[String],
    partition: Option<SemanticPartition>,
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
    let mut summaries: Vec<EpiphanyMemorySummary> = Vec::new();
    let mut warnings = Vec::new();

    // Modeling's unresolved frontier is explicit routing authority. Preserve its
    // stored order and exact claim targets before asking semantic retrieval to
    // infer what matters from the launch prose.
    let (frontier, rejected_frontier_ids) = dependency_ordered_frontier(
        snapshot,
        query,
        partition,
        budget,
        &node_by_id,
        &stale_nodes,
        &domains,
    );
    for id in rejected_frontier_ids {
        warnings.push(format!(
            "frontier {id} was omitted because its target claims are missing, stale, or retired"
        ));
    }
    for node_id in frontier
        .iter()
        .flat_map(|item| item.target_claim_ids.iter())
    {
        if let Some(node) = node_by_id.get(node_id.as_str()) {
            push_unique_node(&mut nodes, (*node).clone(), budget);
        }
    }

    // Semantic projection may rank canonical document identities, never
    // payload-authored claim text. Explicit frontier remains first; ranked
    // canonical objects then seed the cut before BM25 fills remaining space.
    for document_id in ranked_document_ids {
        if let Some(node) = node_by_id.get(document_id.as_str()) {
            if !stale_nodes.contains(node.id.as_str())
                && node_matches(node, query.profile, partition, &domains, &[])
            {
                push_unique_node(&mut nodes, (*node).clone(), budget);
            }
            continue;
        }
        if let Some(edge) = edge_by_id.get(document_id.as_str()) {
            let endpoints_live = [edge.source_id.as_str(), edge.target_id.as_str()]
                .into_iter()
                .all(|id| !stale_nodes.contains(id));
            if !stale_edges.contains(edge.id.as_str())
                && endpoints_live
                && edge_matches(edge, query.profile, partition, &domains, &node_by_id)
            {
                push_unique_edge(&mut edges, (*edge).clone(), budget);
                if let Some(node) = node_by_id.get(edge.source_id.as_str()) {
                    if !stale_nodes.contains(node.id.as_str())
                        && node_matches(node, query.profile, partition, &domains, &[])
                    {
                        push_unique_node(&mut nodes, (*node).clone(), budget);
                    }
                }
                if let Some(node) = node_by_id.get(edge.target_id.as_str()) {
                    if !stale_nodes.contains(node.id.as_str())
                        && node_matches(node, query.profile, partition, &domains, &[])
                    {
                        push_unique_node(&mut nodes, (*node).clone(), budget);
                    }
                }
            }
            continue;
        }
        if let Some(summary) = snapshot
            .summaries
            .iter()
            .find(|summary| summary.id == *document_id)
            && summary_matches(
                summary,
                query.profile,
                partition,
                &domains,
                &[],
                &node_by_id,
            )
            && summary_is_usable(summary, &stale_summaries)
            && summaries.len() < budget
            && !summaries.iter().any(|existing| existing.id == summary.id)
        {
            summaries.push(summary.clone());
        }
    }

    for summary in ranked_summaries(snapshot, query, partition, &domains, &terms, &node_by_id) {
        if summaries.len() >= budget {
            break;
        }
        if !summary_matches(
            summary,
            query.profile,
            partition,
            &domains,
            &terms,
            &node_by_id,
        ) {
            continue;
        }
        if summary_is_usable(summary, &stale_summaries) {
            if !summaries.iter().any(|existing| existing.id == summary.id) {
                summaries.push(summary.clone());
            }
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
            query.profile,
            partition,
            &domains,
            budget,
            &mut nodes,
            &mut edges,
        );
    }

    for node_id in &query.node_ids {
        if let Some(node) = node_by_id.get(node_id.as_str()) {
            if !stale_nodes.contains(node.id.as_str())
                && node_matches(node, query.profile, partition, &domains, &[])
            {
                push_unique_node(&mut nodes, (*node).clone(), budget);
            }
        }
    }
    for edge_id in &query.edge_ids {
        if let Some(edge) = edge_by_id.get(edge_id.as_str()) {
            if !stale_edges.contains(edge.id.as_str())
                && edge_matches(edge, query.profile, partition, &domains, &node_by_id)
            {
                push_unique_edge(&mut edges, (*edge).clone(), budget);
                for node_id in [&edge.source_id, &edge.target_id] {
                    if let Some(node) = node_by_id.get(node_id.as_str())
                        && !stale_nodes.contains(node.id.as_str())
                        && node_matches(node, query.profile, partition, &domains, &[])
                    {
                        push_unique_node(&mut nodes, (*node).clone(), budget);
                    }
                }
            }
        }
    }

    if summaries.is_empty() && nodes.is_empty() {
        for node in ranked_nodes(snapshot, query, partition, &domains, &terms, &stale_nodes) {
            if nodes.len() >= budget {
                break;
            }
            if node_matches(node, query.profile, partition, &domains, &terms) {
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
        repo_model_revision: snapshot.model_revision,
        repo_model_hash: memory_graph_model_hash(snapshot).unwrap_or_default(),
        nodes,
        edges,
        summaries,
        frontier,
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

fn dependency_ordered_frontier(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    query: &EpiphanyMemoryContextQuery,
    partition: Option<SemanticPartition>,
    budget: usize,
    node_by_id: &HashMap<&str, &EpiphanyMemoryNode>,
    stale_nodes: &HashSet<&str>,
    domains: &HashSet<&str>,
) -> (Vec<RepoFrontierItem>, Vec<String>) {
    if partition == Some(SemanticPartition::Mind) {
        return (Vec::new(), Vec::new());
    }
    let unresolved = snapshot
        .frontier
        .iter()
        .filter(|item| {
            matches!(
                item.status,
                RepoFrontierStatus::Active
                    | RepoFrontierStatus::Proposed
                    | RepoFrontierStatus::Blocked
            )
        })
        .collect::<Vec<_>>();
    let unresolved_ids = unresolved
        .iter()
        .map(|item| item.id.as_str())
        .collect::<HashSet<_>>();
    let mut eligible_ids = unresolved
        .iter()
        .filter(|item| {
            !item.target_claim_ids.is_empty()
                && item.target_claim_ids.iter().all(|id| {
                    node_by_id.get(id.as_str()).is_some_and(|node| {
                        !matches!(
                            node.lifecycle,
                            super::EpiphanyMemoryLifecycle::Retired
                                | super::EpiphanyMemoryLifecycle::Stale
                        ) && !stale_nodes.contains(id.as_str())
                            && node_matches(node, query.profile, partition, domains, &[])
                    })
                })
        })
        .map(|item| item.id.as_str())
        .collect::<HashSet<_>>();
    loop {
        let before = eligible_ids.len();
        let current_eligible = eligible_ids.clone();
        eligible_ids.retain(|id| {
            unresolved
                .iter()
                .find(|item| item.id == *id)
                .is_some_and(|item| {
                    item.dependency_item_ids.iter().all(|dependency| {
                        !unresolved_ids.contains(dependency.as_str())
                            || current_eligible.contains(dependency.as_str())
                    })
                })
        });
        if eligible_ids.len() == before {
            break;
        }
    }
    let rejected = unresolved
        .iter()
        .filter(|item| !eligible_ids.contains(item.id.as_str()))
        .map(|item| item.id.clone())
        .collect();
    let eligible = unresolved
        .into_iter()
        .filter(|item| eligible_ids.contains(item.id.as_str()))
        .map(|item| (item.id.as_str(), item))
        .collect::<HashMap<_, _>>();
    let mut ordered = Vec::new();
    let mut complete = HashSet::new();
    let mut visiting = HashSet::new();
    for item in &snapshot.frontier {
        append_frontier_prerequisites(
            &item.id,
            &eligible,
            &mut visiting,
            &mut complete,
            &mut ordered,
        );
    }
    (
        ordered.into_iter().take(budget).cloned().collect(),
        rejected,
    )
}

fn append_frontier_prerequisites<'a>(
    id: &'a str,
    eligible: &HashMap<&'a str, &'a RepoFrontierItem>,
    visiting: &mut HashSet<&'a str>,
    complete: &mut HashSet<&'a str>,
    ordered: &mut Vec<&'a RepoFrontierItem>,
) {
    if complete.contains(id) || !visiting.insert(id) {
        return;
    }
    let Some(item) = eligible.get(id).copied() else {
        visiting.remove(id);
        return;
    };
    for dependency in &item.dependency_item_ids {
        append_frontier_prerequisites(dependency, eligible, visiting, complete, ordered);
    }
    visiting.remove(id);
    if complete.insert(id) {
        ordered.push(item);
    }
}

fn ranked_nodes<'a>(
    snapshot: &'a EpiphanyMemoryGraphSnapshot,
    query: &EpiphanyMemoryContextQuery,
    partition: Option<SemanticPartition>,
    domain_ids: &HashSet<&str>,
    terms: &[String],
    stale_nodes: &HashSet<&str>,
) -> Vec<&'a EpiphanyMemoryNode> {
    let eligible = snapshot
        .nodes
        .iter()
        .filter(|node| {
            node_matches(node, query.profile, partition, domain_ids, &[])
                && !stale_nodes.contains(node.id.as_str())
        })
        .collect::<Vec<_>>();
    rank_typed_documents(eligible, query.text.as_deref(), terms, |node| {
        [
            node.title.as_str(),
            node.claim.as_str(),
            node.question.as_str(),
            node.tension.as_str(),
            node.action_implication.as_str(),
        ]
        .join(" ")
    })
}

fn ranked_summaries<'a>(
    snapshot: &'a EpiphanyMemoryGraphSnapshot,
    query: &EpiphanyMemoryContextQuery,
    partition: Option<SemanticPartition>,
    domain_ids: &HashSet<&str>,
    terms: &[String],
    node_by_id: &HashMap<&str, &EpiphanyMemoryNode>,
) -> Vec<&'a EpiphanyMemorySummary> {
    let eligible = snapshot
        .summaries
        .iter()
        .filter(|summary| {
            summary_matches(
                summary,
                query.profile,
                partition,
                domain_ids,
                &[],
                node_by_id,
            )
        })
        .collect::<Vec<_>>();
    rank_typed_documents(eligible, query.text.as_deref(), terms, |summary| {
        [
            summary.target.as_str(),
            summary.claim.as_str(),
            summary.question.as_str(),
            summary.tension.as_str(),
            summary.action_implication.as_str(),
        ]
        .join(" ")
    })
}

fn rank_typed_documents<'a, T>(
    eligible: Vec<&'a T>,
    query: Option<&str>,
    terms: &[String],
    search_text: impl Fn(&T) -> String,
) -> Vec<&'a T> {
    if eligible.is_empty() || terms.is_empty() {
        return eligible;
    }
    let documents = eligible
        .iter()
        .enumerate()
        .map(|(index, value)| Document::new(index, search_text(value)))
        .collect::<Vec<_>>();
    let engine = SearchEngineBuilder::<usize>::with_documents(Language::English, documents).build();
    engine
        .search(query.unwrap_or_default(), eligible.len())
        .into_iter()
        .filter_map(|result| eligible.get(result.document.id).copied())
        .collect()
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
    partition: Option<SemanticPartition>,
    domain_ids: &HashSet<&str>,
    terms: &[String],
    node_by_id: &HashMap<&str, &EpiphanyMemoryNode>,
) -> bool {
    if !domain_ids.is_empty() && !domain_ids.contains(summary.domain_id.as_str()) {
        return false;
    }
    let covers_partition = summary
        .covers_node_ids
        .iter()
        .filter_map(|id| node_by_id.get(id.as_str()))
        .any(|node| partition_matches(node.profile, partition));
    if partition.is_some() && !covers_partition {
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
    partition: Option<SemanticPartition>,
    domain_ids: &HashSet<&str>,
    terms: &[String],
) -> bool {
    if profile.is_some_and(|profile| node.profile != profile) {
        return false;
    }
    if !partition_matches(node.profile, partition) {
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

fn edge_matches(
    edge: &EpiphanyMemoryEdge,
    profile: Option<EpiphanyMemoryProfile>,
    partition: Option<SemanticPartition>,
    domain_ids: &HashSet<&str>,
    node_by_id: &HashMap<&str, &EpiphanyMemoryNode>,
) -> bool {
    if profile.is_some_and(|profile| edge.profile != profile)
        || !partition_matches(edge.profile, partition)
    {
        return false;
    }
    [&edge.source_id, &edge.target_id].into_iter().all(|id| {
        node_by_id.get(id.as_str()).is_some_and(|node| {
            partition_matches(node.profile, partition)
                && (domain_ids.is_empty() || domain_ids.contains(node.domain_id.as_str()))
        })
    })
}

fn partition_matches(profile: EpiphanyMemoryProfile, partition: Option<SemanticPartition>) -> bool {
    partition.is_none_or(|partition| {
        matches!(
            profile,
            EpiphanyMemoryProfile::RepoArchitecture | EpiphanyMemoryProfile::RepoDataflow
        ) == (partition == SemanticPartition::Modeling)
    })
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
    profile: Option<EpiphanyMemoryProfile>,
    partition: Option<SemanticPartition>,
    domain_ids: &HashSet<&str>,
    budget: usize,
    nodes: &mut Vec<EpiphanyMemoryNode>,
    edges: &mut Vec<EpiphanyMemoryEdge>,
) {
    for node_id in &summary.covers_node_ids {
        if let Some(node) = node_by_id.get(node_id.as_str()) {
            if stale_nodes.contains(node.id.as_str())
                || !node_matches(node, profile, partition, domain_ids, &[])
            {
                continue;
            }
            push_unique_node(nodes, (*node).clone(), budget);
        }
    }
    for edge_id in &summary.covers_edge_ids {
        if let Some(edge) = edge_by_id.get(edge_id.as_str()) {
            if stale_edges.contains(edge.id.as_str())
                || !edge_matches(edge, profile, partition, domain_ids, node_by_id)
            {
                continue;
            }
            push_unique_edge(edges, (*edge).clone(), budget);
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
