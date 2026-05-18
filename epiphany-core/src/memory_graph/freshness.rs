use super::EpiphanyMemoryFreshness;
use super::EpiphanyMemoryFreshnessStatus;
use super::EpiphanyMemoryGraphSnapshot;
use super::EpiphanyMemoryLifecycle;
use super::push_unique;
use super::unique_strings;
use std::collections::HashSet;

pub fn derive_memory_graph_freshness(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    dirty_source_hashes: &[String],
) -> EpiphanyMemoryFreshness {
    let dirty: HashSet<&str> = dirty_source_hashes.iter().map(String::as_str).collect();
    let mut stale_node_ids = Vec::new();
    let mut stale_edge_ids = Vec::new();
    let mut stale_summary_ids = Vec::new();

    for node in &snapshot.nodes {
        if node.lifecycle == EpiphanyMemoryLifecycle::Stale
            || node
                .source_hashes
                .iter()
                .any(|hash| dirty.contains(hash.as_str()))
        {
            push_unique(&mut stale_node_ids, node.id.clone());
        }
    }

    for edge in &snapshot.edges {
        if edge.lifecycle == EpiphanyMemoryLifecycle::Stale
            || edge
                .anchors
                .iter()
                .filter_map(|anchor| anchor.source_hash.as_deref())
                .any(|hash| dirty.contains(hash))
        {
            push_unique(&mut stale_edge_ids, edge.id.clone());
        }
    }

    for summary in &snapshot.summaries {
        if summary.freshness == EpiphanyMemoryFreshnessStatus::Stale
            || summary
                .source_hashes
                .iter()
                .any(|hash| dirty.contains(hash.as_str()))
            || summary
                .covers_node_ids
                .iter()
                .any(|id| stale_node_ids.iter().any(|stale| stale == id))
            || summary
                .covers_edge_ids
                .iter()
                .any(|id| stale_edge_ids.iter().any(|stale| stale == id))
        {
            push_unique(&mut stale_summary_ids, summary.id.clone());
        }
    }

    let status = if stale_node_ids.is_empty()
        && stale_edge_ids.is_empty()
        && stale_summary_ids.is_empty()
        && dirty_source_hashes.is_empty()
    {
        EpiphanyMemoryFreshnessStatus::Ready
    } else {
        EpiphanyMemoryFreshnessStatus::Stale
    };

    EpiphanyMemoryFreshness {
        status,
        stale_node_ids,
        stale_edge_ids,
        stale_summary_ids,
        dirty_source_hashes: unique_strings(dirty_source_hashes.iter().cloned()),
        note: Some(match status {
            EpiphanyMemoryFreshnessStatus::Ready => "Memory graph is ready.".to_string(),
            EpiphanyMemoryFreshnessStatus::Stale => {
                "Memory graph has stale source-backed records.".to_string()
            }
            _ => "Memory graph freshness is not derived.".to_string(),
        }),
    }
}
