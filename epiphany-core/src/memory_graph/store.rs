use super::EpiphanyMemoryFreshnessStatus;
use super::EpiphanyMemoryGraphSnapshot;
use super::derive_memory_graph_freshness;
use super::validate_memory_graph_snapshot;
use anyhow::Result;
use anyhow::anyhow;
use cultcache_rs::CacheBackingStore;
use cultcache_rs::CultCache;
use cultcache_rs::CultCacheEnvelope;
use cultcache_rs::DatabaseEntry;
use cultcache_rs::SingleFileMessagePackBackingStore;
use epiphany_state_model::{
    EpiphanyMemoryLifecycle, RepoFrontierStatus, RepoModelPatch, RepoModelPatchOperation,
};
use sha2::{Digest, Sha256};
use std::path::Path;

pub const MEMORY_GRAPH_TYPE: &str = "epiphany.memory_graph";
pub const MEMORY_GRAPH_KEY: &str = "default";
pub const MEMORY_GRAPH_SCHEMA_VERSION: &str = "epiphany.memory_graph.v1";

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(type = "epiphany.memory_graph", schema = "EpiphanyMemoryGraphEntry")]
pub struct EpiphanyMemoryGraphEntry {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub graph_id: String,
    #[cultcache(key = 2)]
    pub snapshot_msgpack: Vec<u8>,
}

impl EpiphanyMemoryGraphEntry {
    pub fn from_snapshot(snapshot: &EpiphanyMemoryGraphSnapshot) -> Result<Self> {
        let snapshot_msgpack = rmp_serde::to_vec_named(snapshot)?;
        Ok(Self {
            schema_version: MEMORY_GRAPH_SCHEMA_VERSION.to_string(),
            graph_id: snapshot.graph_id.clone(),
            snapshot_msgpack,
        })
    }

    pub fn snapshot(&self) -> Result<EpiphanyMemoryGraphSnapshot> {
        rmp_serde::from_slice(&self.snapshot_msgpack)
            .map_err(|error| anyhow!("failed to decode memory graph snapshot MessagePack: {error}"))
    }
}

pub fn memory_graph_cache(store_path: impl AsRef<Path>) -> Result<CultCache> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<EpiphanyMemoryGraphEntry>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(store_path.as_ref()));
    cache.pull_all_backing_stores()?;
    Ok(cache)
}

pub fn load_memory_graph_entry(
    store_path: impl AsRef<Path>,
) -> Result<Option<EpiphanyMemoryGraphEntry>> {
    let cache = memory_graph_cache(store_path)?;
    cache.get::<EpiphanyMemoryGraphEntry>(MEMORY_GRAPH_KEY)
}

pub fn load_memory_graph_snapshot(
    store_path: impl AsRef<Path>,
) -> Result<Option<EpiphanyMemoryGraphSnapshot>> {
    let Some(entry) = load_memory_graph_entry(store_path)? else {
        return Ok(None);
    };
    validate_memory_graph_entry(&entry)?;
    Ok(Some(entry.snapshot()?))
}

pub fn write_memory_graph_entry(
    store_path: impl AsRef<Path>,
    entry: &EpiphanyMemoryGraphEntry,
) -> Result<EpiphanyMemoryGraphEntry> {
    validate_memory_graph_entry(entry)?;
    let backing = SingleFileMessagePackBackingStore::new(store_path.as_ref());
    let next_envelope = memory_graph_envelope(entry, &chrono::Utc::now().to_rfc3339())?;
    let current = backing.pull_all()?.into_iter().find(|candidate| {
        candidate.r#type == MEMORY_GRAPH_TYPE && candidate.key == MEMORY_GRAPH_KEY
    });
    let installed = if let Some(current_envelope) = current {
        let current_entry: EpiphanyMemoryGraphEntry =
            rmp_serde::from_slice(&current_envelope.payload)?;
        validate_memory_graph_entry(&current_entry)?;
        if current_entry.snapshot()?.model_revision > 0 {
            return Err(anyhow!(
                "canonical memory graph entry can only be changed through a stale-checked RepoModelPatch"
            ));
        }
        backing.compare_and_swap_entry(&current_envelope, next_envelope)?
    } else {
        backing.insert_entry_if_absent(next_envelope)?
    };
    if !installed {
        return Err(anyhow!(
            "memory graph bootstrap lost an atomic store race; reload before retrying"
        ));
    }
    Ok(entry.clone())
}

pub fn write_memory_graph_snapshot(
    store_path: impl AsRef<Path>,
    snapshot: &EpiphanyMemoryGraphSnapshot,
) -> Result<EpiphanyMemoryGraphEntry> {
    let entry = EpiphanyMemoryGraphEntry::from_snapshot(snapshot)?;
    write_memory_graph_entry(store_path, &entry)
}

pub(crate) fn load_memory_graph_envelope(store_path: &Path) -> Result<Option<CultCacheEnvelope>> {
    Ok(SingleFileMessagePackBackingStore::new(store_path)
        .pull_all()?
        .into_iter()
        .find(|candidate| {
            candidate.r#type == MEMORY_GRAPH_TYPE && candidate.key == MEMORY_GRAPH_KEY
        }))
}

pub(crate) fn replace_inspected_legacy_memory_graph(
    store_path: &Path,
    expected: Option<&CultCacheEnvelope>,
    snapshot: &EpiphanyMemoryGraphSnapshot,
    stored_at: &str,
) -> Result<bool> {
    let entry = EpiphanyMemoryGraphEntry::from_snapshot(snapshot)?;
    validate_memory_graph_entry(&entry)?;
    let replacement = memory_graph_envelope(&entry, stored_at)?;
    let backing = SingleFileMessagePackBackingStore::new(store_path);
    match expected {
        Some(expected) => backing.compare_and_swap_entry(expected, replacement),
        None => backing.insert_entry_if_absent(replacement),
    }
}

fn memory_graph_envelope(
    entry: &EpiphanyMemoryGraphEntry,
    stored_at: &str,
) -> Result<CultCacheEnvelope> {
    Ok(CultCacheEnvelope {
        key: MEMORY_GRAPH_KEY.to_string(),
        r#type: MEMORY_GRAPH_TYPE.to_string(),
        payload: rmp_serde::to_vec(entry)?,
        stored_at: stored_at.to_string(),
        schema_id: Some(MEMORY_GRAPH_TYPE.to_string()),
    })
}

/// Hashes the complete aggregate with its self-referential hash field cleared.
pub fn memory_graph_model_hash(snapshot: &EpiphanyMemoryGraphSnapshot) -> Result<String> {
    let mut canonical = snapshot.clone();
    canonical.model_hash.clear();
    let bytes = rmp_serde::to_vec_named(&canonical)?;
    Ok(format!("{:x}", Sha256::digest(bytes)))
}

/// Applies a Modeling patch as one stale-checked replacement of the existing
/// CultCache aggregate. Validation occurs before the store is touched.
#[cfg(test)]
pub(crate) fn apply_repo_model_patch(
    store_path: impl AsRef<Path>,
    patch: &RepoModelPatch,
) -> Result<EpiphanyMemoryGraphSnapshot> {
    let store_path = store_path.as_ref();
    if patch.patch_id.trim().is_empty() || patch.applied_at.trim().is_empty() {
        return Err(anyhow!("repo model patch requires patch_id and applied_at"));
    }
    if patch.operations.is_empty() {
        return Err(anyhow!(
            "repo model patch must contain at least one operation"
        ));
    }
    chrono::DateTime::parse_from_rfc3339(&patch.applied_at)
        .map_err(|_| anyhow!("repo model patch applied_at must be RFC3339"))?;

    let backing = SingleFileMessagePackBackingStore::new(store_path);
    let entries = backing.pull_all()?;
    let envelope = entries
        .iter()
        .find(|entry| entry.r#type == MEMORY_GRAPH_TYPE && entry.key == MEMORY_GRAPH_KEY)
        .cloned()
        .ok_or_else(|| anyhow!("cannot patch a missing memory graph"))?;
    let entry: EpiphanyMemoryGraphEntry = rmp_serde::from_slice(&envelope.payload)?;
    validate_memory_graph_entry(&entry)?;
    let current = entry.snapshot()?;
    let next = derive_repo_model_patch(&current, patch)?;
    let next_entry = EpiphanyMemoryGraphEntry::from_snapshot(&next)?;
    let next_envelope = memory_graph_envelope(&next_entry, &patch.applied_at)?;
    if !backing.compare_and_swap_entry(&envelope, next_envelope)? {
        return Err(anyhow!(
            "stale repo model patch: aggregate changed during compare-and-swap"
        ));
    }
    Ok(next)
}

/// Purely derives and validates the next canonical aggregate. Persistence and
/// admission authority remain the caller's responsibility.
pub fn derive_repo_model_patch(
    current: &EpiphanyMemoryGraphSnapshot,
    patch: &RepoModelPatch,
) -> Result<EpiphanyMemoryGraphSnapshot> {
    if patch.patch_id.trim().is_empty() || patch.applied_at.trim().is_empty() {
        return Err(anyhow!("repo model patch requires patch_id and applied_at"));
    }
    if patch.operations.is_empty() {
        return Err(anyhow!(
            "repo model patch must contain at least one operation"
        ));
    }
    chrono::DateTime::parse_from_rfc3339(&patch.applied_at)
        .map_err(|_| anyhow!("repo model patch applied_at must be RFC3339"))?;
    let current_hash = memory_graph_model_hash(current)?;
    if patch.base_revision != current.model_revision || patch.base_hash != current_hash {
        return Err(anyhow!(
            "stale repo model patch: base revision/hash {}/{} does not match current {}/{}",
            patch.base_revision,
            patch.base_hash,
            current.model_revision,
            current_hash
        ));
    }
    match &patch.purpose {
        epiphany_state_model::RepoModelPatchPurpose::AdoptFrontierPlan {
            planning_request_id,
            result_id,
            candidate_id,
        } => {
            if patch.operations.len() != 1 {
                return Err(anyhow!(
                    "AdoptFrontierPlan purpose requires exactly one dedicated operation"
                ));
            }
            let RepoModelPatchOperation::AdoptFrontierPlan { adopted_plan, .. } =
                &patch.operations[0]
            else {
                return Err(anyhow!(
                    "AdoptFrontierPlan purpose requires its dedicated operation"
                ));
            };
            if adopted_plan.planning_request_id != *planning_request_id
                || adopted_plan.result_id != *result_id
                || adopted_plan.candidate_id != *candidate_id
            {
                return Err(anyhow!(
                    "AdoptFrontierPlan purpose and operation provenance mismatch"
                ));
            }
        }
        _ if patch.operations.iter().any(|operation| {
            matches!(operation, RepoModelPatchOperation::AdoptFrontierPlan { .. })
        }) =>
        {
            return Err(anyhow!(
                "only AdoptFrontierPlan purpose may carry the dedicated adoption operation"
            ));
        }
        _ => {}
    }
    let mut next = current.clone();
    if next
        .lifecycle_receipts
        .iter()
        .any(|receipt| receipt.id == patch.patch_id)
    {
        return Err(anyhow!("repo model patch_id has already been applied"));
    }
    for operation in &patch.operations {
        apply_operation(&mut next, operation, &patch.purpose, &patch.applied_at)?;
    }
    next.model_revision = next
        .model_revision
        .checked_add(1)
        .ok_or_else(|| anyhow!("repo model revision overflow"))?;
    next.model_hash.clear();
    let node_ids = patch
        .operations
        .iter()
        .filter_map(|operation| match operation {
            RepoModelPatchOperation::UpsertNode { node }
            | RepoModelPatchOperation::ReviseNode { node } => Some(node.id.clone()),
            RepoModelPatchOperation::RetireNode { node_id } => Some(node_id.clone()),
            _ => None,
        })
        .collect();
    let edge_ids = patch
        .operations
        .iter()
        .filter_map(|operation| match operation {
            RepoModelPatchOperation::UpsertEdge { edge }
            | RepoModelPatchOperation::ReviseEdge { edge } => Some(edge.id.clone()),
            RepoModelPatchOperation::RetireEdge { edge_id } => Some(edge_id.clone()),
            _ => None,
        })
        .collect();
    next.lifecycle_receipts
        .push(epiphany_state_model::EpiphanyMemoryLifecycleReceipt {
            id: patch.patch_id.clone(),
            operation: "repo_model_patch".to_string(),
            status: "applied".to_string(),
            node_ids,
            edge_ids,
            reason: Some(format!("{} typed operations", patch.operations.len())),
            created_at: Some(patch.applied_at.clone()),
            ..Default::default()
        });
    let validation_errors = validate_memory_graph_snapshot(&next);
    if !validation_errors.is_empty() {
        return Err(anyhow!(
            "repo model patch produces invalid aggregate: {}",
            validation_errors
                .iter()
                .map(|error| format!("{}: {}", error.path, error.message))
                .collect::<Vec<_>>()
                .join("; ")
        ));
    }
    next.model_hash = memory_graph_model_hash(&next)?;
    validate_memory_graph_entry(&EpiphanyMemoryGraphEntry::from_snapshot(&next)?)?;
    Ok(next)
}

fn apply_operation(
    snapshot: &mut EpiphanyMemoryGraphSnapshot,
    operation: &RepoModelPatchOperation,
    purpose: &epiphany_state_model::RepoModelPatchPurpose,
    applied_at: &str,
) -> Result<()> {
    match operation {
        RepoModelPatchOperation::UpsertNode { node } => {
            insert_new(&mut snapshot.nodes, node.clone(), |value| &value.id, "node")?
        }
        RepoModelPatchOperation::ReviseNode { node } => {
            revise(&mut snapshot.nodes, node.clone(), |value| &value.id, "node")?
        }
        RepoModelPatchOperation::RetireNode { node_id } => {
            let node = snapshot
                .nodes
                .iter_mut()
                .find(|node| &node.id == node_id)
                .ok_or_else(|| anyhow!("cannot retire missing node {node_id}"))?;
            node.lifecycle = EpiphanyMemoryLifecycle::Retired;
            node.updated_at = Some(applied_at.to_string());
        }
        RepoModelPatchOperation::UpsertEdge { edge } => {
            insert_new(&mut snapshot.edges, edge.clone(), |value| &value.id, "edge")?
        }
        RepoModelPatchOperation::ReviseEdge { edge } => {
            revise(&mut snapshot.edges, edge.clone(), |value| &value.id, "edge")?
        }
        RepoModelPatchOperation::RetireEdge { edge_id } => {
            let edge = snapshot
                .edges
                .iter_mut()
                .find(|edge| &edge.id == edge_id)
                .ok_or_else(|| anyhow!("cannot retire missing edge {edge_id}"))?;
            edge.lifecycle = EpiphanyMemoryLifecycle::Retired;
        }
        RepoModelPatchOperation::UpsertFrontier { item } => {
            if item.adopted_plan.is_some() {
                return Err(anyhow!(
                    "generic frontier upsert cannot author an adopted plan"
                ));
            }
            insert_new(
                &mut snapshot.frontier,
                item.clone(),
                |value| &value.id,
                "frontier item",
            )?
        }
        RepoModelPatchOperation::ReviseFrontier { item } => {
            let existing = snapshot
                .frontier
                .iter()
                .find(|existing| existing.id == item.id)
                .ok_or_else(|| anyhow!("cannot revise missing frontier item {}", item.id))?;
            let verdict_may_close_adopted = matches!(
                purpose,
                epiphany_state_model::RepoModelPatchPurpose::IncorporateFrontierVerdict { .. }
            ) && existing.adopted_plan.is_some()
                && item.adopted_plan == existing.adopted_plan
                && item.migration_body == existing.migration_body
                && item.question == existing.question
                && item.target_claim_ids == existing.target_claim_ids
                && item.source_scope == existing.source_scope
                && item.recommended_next_organ == existing.recommended_next_organ
                && item.dependency_item_ids == existing.dependency_item_ids
                && item.created_at == existing.created_at
                && item.retired_at == existing.retired_at
                && item.superseded_by == existing.superseded_by;
            if (existing.adopted_plan.is_some() && !verdict_may_close_adopted)
                || item.adopted_plan != existing.adopted_plan
                || (existing.recommended_next_organ == "Imagination"
                    && item.recommended_next_organ == "Hands")
            {
                return Err(anyhow!(
                    "generic frontier revision cannot alter adopted execution anatomy or own plan adoption"
                ));
            }
            revise(
                &mut snapshot.frontier,
                item.clone(),
                |value| &value.id,
                "frontier item",
            )?
        }
        RepoModelPatchOperation::RetireFrontier {
            item_id,
            retired_at,
            superseded_by,
        } => {
            let item = snapshot
                .frontier
                .iter_mut()
                .find(|item| &item.id == item_id)
                .ok_or_else(|| anyhow!("cannot retire missing frontier item {item_id}"))?;
            item.status = if superseded_by.is_some() {
                RepoFrontierStatus::Superseded
            } else {
                RepoFrontierStatus::Retired
            };
            item.retired_at = retired_at.clone().or_else(|| Some(applied_at.to_string()));
            item.updated_at = Some(applied_at.to_string());
            item.superseded_by = superseded_by.clone();
        }
        RepoModelPatchOperation::AdoptFrontierPlan {
            frontier_item_id,
            expected_frontier_item_hash,
            adopted_plan,
        } => {
            let item = snapshot
                .frontier
                .iter_mut()
                .find(|item| &item.id == frontier_item_id)
                .ok_or_else(|| {
                    anyhow!("cannot adopt plan for missing frontier item {frontier_item_id}")
                })?;
            let current_hash = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(&*item)?));
            let safe_paths_are_bounded = !adopted_plan.safe_paths.is_empty()
                && adopted_plan
                    .safe_paths
                    .windows(2)
                    .all(|pair| pair[0] < pair[1])
                && adopted_plan.safe_paths.iter().all(|path| {
                    item.source_scope.iter().any(|scope| {
                        path == scope
                            || path
                                .starts_with(&format!("{}/", scope.trim_end_matches(['/', '\\'])))
                    })
                });
            if current_hash != *expected_frontier_item_hash
                || item.status != RepoFrontierStatus::Active
                || item.recommended_next_organ != "Imagination"
                || item.adopted_plan.is_some()
                || adopted_plan.planning_request_id.trim().is_empty()
                || adopted_plan.result_id.trim().is_empty()
                || adopted_plan.job_id.trim().is_empty()
                || adopted_plan.candidate_id.trim().is_empty()
                || adopted_plan.candidate_sha256.trim().is_empty()
                || adopted_plan.action.trim().is_empty()
                || adopted_plan.command.trim().is_empty()
                || adopted_plan.checks.is_empty()
                || adopted_plan.stop_conditions.is_empty()
                || adopted_plan.rollback_steps.is_empty()
                || adopted_plan.commit_message.trim().is_empty()
                || !safe_paths_are_bounded
            {
                return Err(anyhow!(
                    "frontier plan adoption requires the exact active Imagination frontier and complete bounded plan"
                ));
            }
            item.adopted_plan = Some(adopted_plan.clone());
            item.recommended_next_organ = "Hands".to_string();
            item.updated_at = Some(applied_at.to_string());
        }
    }
    Ok(())
}

fn insert_new<T>(
    values: &mut Vec<T>,
    value: T,
    id: impl Fn(&T) -> &String,
    kind: &str,
) -> Result<()> {
    let value_id = id(&value).clone();
    if values.iter().any(|existing| id(existing) == &value_id) {
        return Err(anyhow!(
            "cannot upsert existing {kind} {value_id}; use revise"
        ));
    }
    values.push(value);
    Ok(())
}

fn revise<T>(values: &mut [T], value: T, id: impl Fn(&T) -> &String, kind: &str) -> Result<()> {
    let value_id = id(&value).clone();
    let Some(index) = values.iter().position(|existing| id(existing) == &value_id) else {
        return Err(anyhow!("cannot revise missing {kind} {value_id}"));
    };
    values[index] = value;
    Ok(())
}

pub fn validate_memory_graph_entry(entry: &EpiphanyMemoryGraphEntry) -> Result<()> {
    if entry.schema_version != MEMORY_GRAPH_SCHEMA_VERSION {
        return Err(anyhow!(
            "memory graph schema_version is {:?}, expected {:?}",
            entry.schema_version,
            MEMORY_GRAPH_SCHEMA_VERSION
        ));
    }
    let snapshot = entry.snapshot()?;
    if snapshot.schema_version.as_deref() != Some(MEMORY_GRAPH_SCHEMA_VERSION) {
        return Err(anyhow!(
            "memory graph snapshot schema_version is {:?}, expected {:?}",
            snapshot.schema_version,
            MEMORY_GRAPH_SCHEMA_VERSION
        ));
    }
    if entry.graph_id != snapshot.graph_id {
        return Err(anyhow!(
            "memory graph entry graph_id {:?} does not match snapshot graph_id {:?}",
            entry.graph_id,
            snapshot.graph_id
        ));
    }
    let legacy = snapshot.model_revision == 0
        && snapshot.model_hash.is_empty()
        && snapshot.frontier.is_empty();
    let canonical = snapshot.model_revision > 0
        && !snapshot.model_hash.is_empty()
        && snapshot.model_hash == memory_graph_model_hash(&snapshot)?;
    if !legacy && !canonical {
        return Err(anyhow!(
            "memory graph identity must be exactly legacy (revision 0, empty hash/frontier) or canonical (positive revision, exact nonempty hash)"
        ));
    }
    let errors = validate_memory_graph_snapshot(&snapshot);
    if !errors.is_empty() {
        let message = errors
            .iter()
            .map(|error| format!("{}: {}", error.path, error.message))
            .collect::<Vec<_>>()
            .join("; ");
        return Err(anyhow!("memory graph validation failed: {message}"));
    }
    let dirty = snapshot
        .freshness
        .as_ref()
        .map(|freshness| freshness.dirty_source_hashes.as_slice())
        .unwrap_or(&[]);
    let derived = derive_memory_graph_freshness(&snapshot, dirty);
    if snapshot.frontier.iter().any(|item| {
        matches!(
            item.status,
            RepoFrontierStatus::Active | RepoFrontierStatus::Proposed | RepoFrontierStatus::Blocked
        ) && item
            .target_claim_ids
            .iter()
            .any(|id| derived.stale_node_ids.contains(id))
    }) {
        return Err(anyhow!(
            "unresolved frontier targets a stale source-derived claim"
        ));
    }
    if snapshot.freshness.as_ref().is_some_and(|stored| {
        stored.status != derived.status
            || stored.stale_node_ids != derived.stale_node_ids
            || stored.stale_edge_ids != derived.stale_edge_ids
            || stored.stale_summary_ids != derived.stale_summary_ids
    }) {
        return Err(anyhow!(
            "memory graph stored freshness disagrees with derived lifecycle/source freshness"
        ));
    }
    if snapshot.summaries.iter().any(|summary| {
        derived.stale_summary_ids.contains(&summary.id)
            && summary.freshness != EpiphanyMemoryFreshnessStatus::Stale
    }) {
        return Err(anyhow!(
            "memory graph has a ready summary covering stale anatomy"
        ));
    }
    Ok(())
}
