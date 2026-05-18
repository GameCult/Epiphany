use super::EpiphanyMemoryContextPacket;
use super::EpiphanyMemoryContextQuery;
use super::EpiphanyMemoryGraphSemanticHit;
use super::EpiphanyMemoryGraphSnapshot;
use super::plan_memory_graph_context_cut;
use super::push_unique;
use super::query_memory_graph_semantic_cache;

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
    let mut warnings = Vec::new();
    if hits.is_empty() {
        return plan_memory_graph_context_cut(snapshot, query);
    }

    let mut seeded_query = query.clone();
    for hit in &hits {
        match hit.document_kind.as_str() {
            "node" => push_unique(&mut seeded_query.node_ids, hit.source_id.clone()),
            "edge" => push_unique(&mut seeded_query.edge_ids, hit.source_id.clone()),
            "summary" => {
                if let Some(summary) = snapshot
                    .summaries
                    .iter()
                    .find(|summary| summary.id == hit.source_id)
                {
                    for node_id in &summary.covers_node_ids {
                        push_unique(&mut seeded_query.node_ids, node_id.clone());
                    }
                    for edge_id in &summary.covers_edge_ids {
                        push_unique(&mut seeded_query.edge_ids, edge_id.clone());
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

    let mut packet = plan_memory_graph_context_cut(snapshot, &seeded_query);
    packet.warnings.push(format!(
        "semantic cache seeded {} typed graph document(s)",
        hits.len()
    ));
    packet.warnings.extend(warnings);
    packet
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
            nodes: vec![EpiphanyMemoryNode {
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
            }],
            ..Default::default()
        }
    }
}
