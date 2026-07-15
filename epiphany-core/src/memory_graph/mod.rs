pub mod compose;
pub mod context_cut;
pub mod freshness;
pub mod ids;
pub mod profiles;
pub mod semantic_index;
pub mod semantic_projection;
pub mod store;
pub mod validation;

pub use epiphany_state_model::EpiphanyMemoryAnchor;
pub use epiphany_state_model::EpiphanyMemoryContextPacket;
pub use epiphany_state_model::EpiphanyMemoryContextQuery;
pub use epiphany_state_model::EpiphanyMemoryDomain;
pub use epiphany_state_model::EpiphanyMemoryEdge;
pub use epiphany_state_model::EpiphanyMemoryEdgeKind;
pub use epiphany_state_model::EpiphanyMemoryEmbeddingManifest;
pub use epiphany_state_model::EpiphanyMemoryFreshness;
pub use epiphany_state_model::EpiphanyMemoryFreshnessStatus;
pub use epiphany_state_model::EpiphanyMemoryGraphSnapshot;
pub use epiphany_state_model::EpiphanyMemoryGraphSource;
pub use epiphany_state_model::EpiphanyMemoryLifecycle;
pub use epiphany_state_model::EpiphanyMemoryLifecycleReceipt;
pub use epiphany_state_model::EpiphanyMemoryNode;
pub use epiphany_state_model::EpiphanyMemoryNodeKind;
pub use epiphany_state_model::EpiphanyMemoryPatchCandidate;
pub use epiphany_state_model::EpiphanyMemoryProfile;
pub use epiphany_state_model::EpiphanyMemorySummary;
pub use epiphany_state_model::RepoFrontierAdoptedPlan;
pub use epiphany_state_model::RepoFrontierItem;
pub use epiphany_state_model::RepoFrontierStatus;
pub use epiphany_state_model::RepoModelPatch;
pub use epiphany_state_model::RepoModelPatchOperation;
pub use epiphany_state_model::RepoModelPatchPurpose;

pub use compose::compose_memory_graph_snapshots;
pub use context_cut::plan_memory_graph_context_cut;
pub(crate) use context_cut::plan_memory_graph_context_cut_for_partition;
pub use context_cut::plan_memory_graph_context_cut_with_ranked_ids;
pub use freshness::derive_memory_graph_freshness;
pub use ids::memory_graph_domain_id;
pub use ids::memory_graph_edge_id;
pub use ids::memory_graph_node_id;
pub use profiles::RepoMemoryGraphRefresh;
pub use profiles::memory_graph_from_agent_memories;
pub use profiles::memory_graph_from_epiphany_graphs;
pub use profiles::memory_graph_from_heartbeat_cognition;
pub use profiles::refresh_or_validate_repo_memory_graph;
pub use semantic_index::*;
pub use semantic_projection::*;
pub use store::EpiphanyMemoryGraphEntry;
pub use store::MEMORY_GRAPH_KEY;
pub use store::MEMORY_GRAPH_SCHEMA_VERSION;
pub use store::MEMORY_GRAPH_TYPE;
#[cfg(test)]
pub(crate) use store::apply_repo_model_patch;
pub use store::derive_repo_model_patch;
pub use store::load_memory_graph_entry;
pub use store::load_memory_graph_snapshot;
pub use store::memory_graph_cache;
pub use store::memory_graph_model_hash;
pub use store::validate_memory_graph_entry;
pub use store::write_memory_graph_entry;
pub use store::write_memory_graph_snapshot;
pub use validation::EpiphanyMemoryGraphValidationError;
pub use validation::lifecycle_allowed_for_profile;
pub use validation::validate_memory_graph_snapshot;

pub(crate) fn push_unique(target: &mut Vec<String>, value: String) {
    if !target.iter().any(|existing| existing == &value) {
        target.push(value);
    }
}

pub(crate) fn unique_strings(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut out = Vec::new();
    for value in values {
        push_unique(&mut out, value);
    }
    out
}

#[cfg(test)]
mod tests;
