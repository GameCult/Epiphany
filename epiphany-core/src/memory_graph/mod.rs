pub mod compose;
pub mod context_cut;
pub mod freshness;
pub mod ids;
pub mod profiles;
pub mod semantic_index;
pub mod semantic_projection;
pub mod semantic_projector;
pub mod semantic_projector_pulse;
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

pub use compose::compose_memory_graph_snapshots;
pub use context_cut::plan_memory_graph_context_cut;
pub use context_cut::plan_memory_graph_context_cut_with_ranked_ids;
pub(crate) use context_cut::plan_modeling_context_cut;
pub use freshness::derive_memory_graph_freshness;
pub use ids::memory_graph_domain_id;
pub use ids::memory_graph_edge_id;
pub use ids::memory_graph_node_id;
pub use semantic_index::*;
pub use semantic_projection::*;
pub use semantic_projector::*;
pub use semantic_projector_pulse::*;
pub use validation::EpiphanyMemoryGraphValidationError;
pub(crate) use validation::frontier_item_has_routeable_repository_scope;
pub use validation::lifecycle_allowed_for_profile;
pub(crate) use validation::repo_paths_are_canonical_and_safe;
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

pub const MEMORY_GRAPH_PROJECTION_SCHEMA_VERSION: &str = "epiphany.memory_graph.projection.v1";

/// Digest helper for transient memory-context and semantic projection DTOs.
/// This digest is derived notification state; keyed Mind documents own the
/// RepoModel and no aggregate revision can be committed through this module.
pub fn memory_graph_model_hash(snapshot: &EpiphanyMemoryGraphSnapshot) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};

    let mut canonical = snapshot.clone();
    canonical.model_hash.clear();
    Ok(format!(
        "{:x}",
        Sha256::digest(rmp_serde::to_vec_named(&canonical)?)
    ))
}
