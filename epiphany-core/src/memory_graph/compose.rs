use super::EpiphanyMemoryFreshness;
use super::EpiphanyMemoryFreshnessStatus;
use super::EpiphanyMemoryGraphSnapshot;
use super::EpiphanyMemoryGraphValidationError;
use super::validate_memory_graph_snapshot;
use crate::memory_graph::store::MEMORY_GRAPH_SCHEMA_VERSION;

pub fn compose_memory_graph_snapshots(
    graph_id: impl Into<String>,
    snapshots: impl IntoIterator<Item = EpiphanyMemoryGraphSnapshot>,
) -> Result<EpiphanyMemoryGraphSnapshot, Vec<EpiphanyMemoryGraphValidationError>> {
    let mut composed = EpiphanyMemoryGraphSnapshot {
        schema_version: Some(MEMORY_GRAPH_SCHEMA_VERSION.to_string()),
        graph_id: graph_id.into(),
        ..Default::default()
    };

    let mut freshness_notes = Vec::new();
    for snapshot in snapshots {
        composed.domains.extend(snapshot.domains);
        composed.nodes.extend(snapshot.nodes);
        composed.edges.extend(snapshot.edges);
        composed.summaries.extend(snapshot.summaries);
        composed
            .lifecycle_receipts
            .extend(snapshot.lifecycle_receipts);
        if let Some(freshness) = snapshot.freshness {
            merge_freshness(&mut composed, freshness, &mut freshness_notes);
        }
    }

    if let Some(freshness) = composed.freshness.as_mut() {
        if freshness.note.is_none() && !freshness_notes.is_empty() {
            freshness.note = Some(freshness_notes.join(" "));
        }
    }

    let errors = validate_memory_graph_snapshot(&composed);
    if errors.is_empty() {
        Ok(composed)
    } else {
        Err(errors)
    }
}

fn merge_freshness(
    composed: &mut EpiphanyMemoryGraphSnapshot,
    freshness: EpiphanyMemoryFreshness,
    notes: &mut Vec<String>,
) {
    let aggregate = composed
        .freshness
        .get_or_insert_with(|| EpiphanyMemoryFreshness {
            status: EpiphanyMemoryFreshnessStatus::Ready,
            note: None,
            ..Default::default()
        });

    if freshness_rank(freshness.status) > freshness_rank(aggregate.status) {
        aggregate.status = freshness.status;
    }
    merge_unique(&mut aggregate.stale_node_ids, freshness.stale_node_ids);
    merge_unique(&mut aggregate.stale_edge_ids, freshness.stale_edge_ids);
    merge_unique(
        &mut aggregate.stale_summary_ids,
        freshness.stale_summary_ids,
    );
    merge_unique(
        &mut aggregate.dirty_source_hashes,
        freshness.dirty_source_hashes,
    );
    if let Some(note) = freshness.note {
        if !note.trim().is_empty() {
            notes.push(note);
        }
    }
}

fn freshness_rank(status: EpiphanyMemoryFreshnessStatus) -> u8 {
    match status {
        EpiphanyMemoryFreshnessStatus::Stale => 2,
        EpiphanyMemoryFreshnessStatus::Ready => 1,
        _ => 0,
    }
}

fn merge_unique(target: &mut Vec<String>, values: impl IntoIterator<Item = String>) {
    for value in values {
        if !target.iter().any(|existing| existing == &value) {
            target.push(value);
        }
    }
}
