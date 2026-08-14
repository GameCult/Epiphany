use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, Result};
use cultcache_rs::{CultCache, CultCacheEnvelope, DatabaseEntry};
use epiphany_state_model::{
    EpiphanyMemoryDomain, EpiphanyMemoryEdge, EpiphanyMemoryGraphSnapshot,
    EpiphanyMemoryLifecycleReceipt, EpiphanyMemoryNode, EpiphanyMemorySummary, RepoFrontierItem,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{runtime_spine_cache, validate_memory_graph_snapshot, EpiphanyMindDocumentVersion};

pub const REPO_MODEL_SCHEMA_EPOCH: &str = "epiphany.repo_model.epoch.v1";
pub const REPO_MODEL_IDENTITY_KEY: &str = "repo-model";

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.repo_model.identity.v1",
    schema = "EpiphanyRepoModelIdentityDocument"
)]
pub struct EpiphanyRepoModelIdentityDocument {
    #[cultcache(key = 0)]
    pub schema_epoch: String,
    #[cultcache(key = 1)]
    pub graph_id: String,
    #[cultcache(key = 2)]
    pub runtime_id: String,
    #[cultcache(key = 3)]
    pub swarm_id: String,
    #[cultcache(key = 4)]
    pub workspace_id: String,
    #[cultcache(key = 5)]
    pub body_binding_sha256: String,
}

macro_rules! repo_document {
    ($name:ident, $type:literal, $schema:literal, $value:ty) => {
        #[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
        #[cultcache(type = $type, schema = $schema)]
        pub struct $name {
            #[cultcache(key = 0)]
            pub payload_msgpack: Vec<u8>,
        }

        impl $name {
            pub fn new(value: &$value) -> Result<Self> {
                Ok(Self {
                    payload_msgpack: rmp_serde::to_vec_named(value)?,
                })
            }

            pub fn value(&self) -> Result<$value> {
                rmp_serde::from_slice(&self.payload_msgpack)
                    .map_err(|error| anyhow!("invalid {} payload: {error}", $schema))
            }
        }
    };
}

repo_document!(
    EpiphanyRepoModelDomainDocument,
    "epiphany.mind.repo_model.domain.v1",
    "EpiphanyRepoModelDomainDocument",
    EpiphanyMemoryDomain
);
repo_document!(
    EpiphanyRepoModelNodeDocument,
    "epiphany.mind.repo_model.node.v1",
    "EpiphanyRepoModelNodeDocument",
    EpiphanyMemoryNode
);
repo_document!(
    EpiphanyRepoModelEdgeDocument,
    "epiphany.mind.repo_model.edge.v1",
    "EpiphanyRepoModelEdgeDocument",
    EpiphanyMemoryEdge
);
repo_document!(
    EpiphanyRepoModelSummaryDocument,
    "epiphany.mind.repo_model.summary.v1",
    "EpiphanyRepoModelSummaryDocument",
    EpiphanyMemorySummary
);
repo_document!(
    EpiphanyRepoModelFrontierDocument,
    "epiphany.mind.repo_model.frontier.v1",
    "EpiphanyRepoModelFrontierDocument",
    RepoFrontierItem
);
repo_document!(
    EpiphanyRepoModelLifecycleReceiptDocument,
    "epiphany.mind.repo_model.lifecycle_receipt.v1",
    "EpiphanyRepoModelLifecycleReceiptDocument",
    EpiphanyMemoryLifecycleReceipt
);

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.mind.repo_model.claim_obligations.v1",
    schema = "EpiphanyRepoModelClaimObligationsDocument"
)]
pub struct EpiphanyRepoModelClaimObligationsDocument {
    #[cultcache(key = 0)]
    pub node_id: String,
    #[cultcache(key = 1)]
    pub unresolved_frontier_ids: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyRepoModelView {
    pub identity: EpiphanyRepoModelIdentityDocument,
    pub projection_digest: String,
    pub source_documents: Vec<EpiphanyMindDocumentVersion>,
    pub domains: Vec<EpiphanyMemoryDomain>,
    pub nodes: Vec<EpiphanyMemoryNode>,
    pub edges: Vec<EpiphanyMemoryEdge>,
    pub summaries: Vec<EpiphanyMemorySummary>,
    pub frontier: Vec<RepoFrontierItem>,
    pub lifecycle_receipts: Vec<EpiphanyMemoryLifecycleReceipt>,
    pub claim_obligations: Vec<EpiphanyRepoModelClaimObligationsDocument>,
}

pub const REPO_MODEL_MUTATION_PROPOSAL_SCHEMA_VERSION: &str =
    "epiphany.repo_model.mutation_proposal.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum EpiphanyRepoModelMutationOperation {
    PutDomain { domain: EpiphanyMemoryDomain },
    PutNode { node: EpiphanyMemoryNode },
    RetireNode { node_id: String },
    PutEdge { edge: EpiphanyMemoryEdge },
    RetireEdge { edge_id: String },
    PutSummary { summary: EpiphanyMemorySummary },
    PutFrontier { item: RepoFrontierItem },
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.repo_model.mutation_proposal.v1",
    schema = "EpiphanyRepoModelMutationProposal"
)]
pub struct EpiphanyRepoModelMutationProposal {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub proposal_id: String,
    #[cultcache(key = 2)]
    pub operations_msgpack: Vec<u8>,
}

impl EpiphanyRepoModelMutationProposal {
    pub fn new(
        proposal_id: impl Into<String>,
        operations: Vec<EpiphanyRepoModelMutationOperation>,
    ) -> Result<Self> {
        let proposal = Self {
            schema_version: REPO_MODEL_MUTATION_PROPOSAL_SCHEMA_VERSION.into(),
            proposal_id: proposal_id.into(),
            operations_msgpack: rmp_serde::to_vec_named(&operations)?,
        };
        proposal.validate()?;
        Ok(proposal)
    }

    pub fn operations(&self) -> Result<Vec<EpiphanyRepoModelMutationOperation>> {
        rmp_serde::from_slice(&self.operations_msgpack)
            .map_err(|error| anyhow!("invalid RepoModel mutation operations: {error}"))
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != REPO_MODEL_MUTATION_PROPOSAL_SCHEMA_VERSION
            || self.proposal_id.trim().is_empty()
            || self.operations()?.is_empty()
        {
            return Err(anyhow!("RepoModel mutation proposal is empty"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct EpiphanyRepoModelMutationPlan {
    pub proposal_id: String,
    pub strong_reads: Vec<CultCacheEnvelope>,
    pub writes: Vec<CultCacheEnvelope>,
}

pub(crate) fn register_repo_model_document_types(cache: &mut CultCache) -> Result<()> {
    cache.register_entry_type::<EpiphanyRepoModelIdentityDocument>()?;
    cache.register_entry_type::<EpiphanyRepoModelDomainDocument>()?;
    cache.register_entry_type::<EpiphanyRepoModelNodeDocument>()?;
    cache.register_entry_type::<EpiphanyRepoModelEdgeDocument>()?;
    cache.register_entry_type::<EpiphanyRepoModelSummaryDocument>()?;
    cache.register_entry_type::<EpiphanyRepoModelFrontierDocument>()?;
    cache.register_entry_type::<EpiphanyRepoModelLifecycleReceiptDocument>()?;
    cache.register_entry_type::<EpiphanyRepoModelClaimObligationsDocument>()?;
    cache.register_entry_type::<EpiphanyRepoModelMutationProposal>()?;
    Ok(())
}

pub(crate) fn repo_model_write_key(envelope: &CultCacheEnvelope) -> Result<Option<String>> {
    let key = if envelope.r#type == EpiphanyRepoModelIdentityDocument::TYPE {
        let value: EpiphanyRepoModelIdentityDocument = rmp_serde::from_slice(&envelope.payload)?;
        if value.schema_epoch != REPO_MODEL_SCHEMA_EPOCH
            || [
                value.graph_id.as_str(),
                value.runtime_id.as_str(),
                value.swarm_id.as_str(),
                value.workspace_id.as_str(),
                value.body_binding_sha256.as_str(),
            ]
            .into_iter()
            .any(|value| value.trim().is_empty())
        {
            return Err(anyhow!("RepoModel identity write is invalid"));
        }
        REPO_MODEL_IDENTITY_KEY.to_string()
    } else if envelope.r#type == EpiphanyRepoModelDomainDocument::TYPE {
        rmp_serde::from_slice::<EpiphanyRepoModelDomainDocument>(&envelope.payload)?
            .value()?
            .id
    } else if envelope.r#type == EpiphanyRepoModelNodeDocument::TYPE {
        rmp_serde::from_slice::<EpiphanyRepoModelNodeDocument>(&envelope.payload)?
            .value()?
            .id
    } else if envelope.r#type == EpiphanyRepoModelEdgeDocument::TYPE {
        rmp_serde::from_slice::<EpiphanyRepoModelEdgeDocument>(&envelope.payload)?
            .value()?
            .id
    } else if envelope.r#type == EpiphanyRepoModelSummaryDocument::TYPE {
        rmp_serde::from_slice::<EpiphanyRepoModelSummaryDocument>(&envelope.payload)?
            .value()?
            .id
    } else if envelope.r#type == EpiphanyRepoModelFrontierDocument::TYPE {
        rmp_serde::from_slice::<EpiphanyRepoModelFrontierDocument>(&envelope.payload)?
            .value()?
            .id
    } else if envelope.r#type == EpiphanyRepoModelLifecycleReceiptDocument::TYPE {
        rmp_serde::from_slice::<EpiphanyRepoModelLifecycleReceiptDocument>(&envelope.payload)?
            .value()?
            .id
    } else if envelope.r#type == EpiphanyRepoModelClaimObligationsDocument::TYPE {
        let value: EpiphanyRepoModelClaimObligationsDocument =
            rmp_serde::from_slice(&envelope.payload)?;
        if value
            .unresolved_frontier_ids
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
        {
            return Err(anyhow!("RepoModel claim obligations are not canonical"));
        }
        value.node_id
    } else {
        return Ok(None);
    };
    if key.trim().is_empty() {
        return Err(anyhow!("RepoModel semantic identity cannot be empty"));
    }
    Ok(Some(key))
}

pub fn plan_repo_model_mutation(
    store_path: impl AsRef<Path>,
    proposal: &EpiphanyRepoModelMutationProposal,
) -> Result<EpiphanyRepoModelMutationPlan> {
    proposal.validate()?;
    let operations = proposal.operations()?;
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    let view = assemble_repo_model_view(store_path.as_ref())?;
    let identity_envelope = cache
        .get_envelope::<EpiphanyRepoModelIdentityDocument>(REPO_MODEL_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("RepoModel mutation lost its identity envelope"))?;
    let mut domains = view
        .domains
        .iter()
        .cloned()
        .map(|value| (value.id.clone(), value))
        .collect::<BTreeMap<_, _>>();
    let mut nodes = view
        .nodes
        .iter()
        .cloned()
        .map(|value| (value.id.clone(), value))
        .collect::<BTreeMap<_, _>>();
    let mut edges = view
        .edges
        .iter()
        .cloned()
        .map(|value| (value.id.clone(), value))
        .collect::<BTreeMap<_, _>>();
    let mut summaries = view
        .summaries
        .iter()
        .cloned()
        .map(|value| (value.id.clone(), value))
        .collect::<BTreeMap<_, _>>();
    let mut frontier = view
        .frontier
        .iter()
        .cloned()
        .map(|value| (value.id.clone(), value))
        .collect::<BTreeMap<_, _>>();
    let mut obligations = view
        .claim_obligations
        .iter()
        .cloned()
        .map(|value| (value.node_id.clone(), value))
        .collect::<BTreeMap<_, _>>();
    let staged_node_ids = operations
        .iter()
        .filter_map(|operation| match operation {
            EpiphanyRepoModelMutationOperation::PutNode { node } => Some(node.id.clone()),
            EpiphanyRepoModelMutationOperation::RetireNode { node_id } => Some(node_id.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let staged_domain_ids = operations
        .iter()
        .filter_map(|operation| match operation {
            EpiphanyRepoModelMutationOperation::PutDomain { domain } => Some(domain.id.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let staged_frontier_ids = operations
        .iter()
        .filter_map(|operation| match operation {
            EpiphanyRepoModelMutationOperation::PutFrontier { item } => Some(item.id.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let staged_edge_ids = operations
        .iter()
        .filter_map(|operation| match operation {
            EpiphanyRepoModelMutationOperation::PutEdge { edge } => Some(edge.id.clone()),
            EpiphanyRepoModelMutationOperation::RetireEdge { edge_id } => Some(edge_id.clone()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let operation_identities = operations
        .iter()
        .map(repo_model_operation_identity)
        .collect::<Result<BTreeSet<_>>>()?;
    if operation_identities.len() != operations.len() {
        return Err(anyhow!(
            "RepoModel mutation proposal repeats a semantic identity"
        ));
    }

    let mut strong = BTreeMap::from([(
        (
            identity_envelope.r#type.clone(),
            identity_envelope.key.clone(),
        ),
        identity_envelope,
    )]);
    let mut writes = BTreeMap::new();
    for operation in &operations {
        match operation {
            EpiphanyRepoModelMutationOperation::PutDomain { domain } => {
                require_semantic_id(&domain.id, "RepoModel domain")?;
                if let Some(existing) =
                    cache.get_envelope::<EpiphanyRepoModelDomainDocument>(&domain.id)?
                {
                    insert_strong_envelope(&mut strong, existing)?;
                }
                domains.insert(domain.id.clone(), domain.clone());
                insert_envelope(
                    &mut writes,
                    cache
                        .prepare_entry(&domain.id, &EpiphanyRepoModelDomainDocument::new(domain)?)?
                        .0,
                )?;
            }
            EpiphanyRepoModelMutationOperation::PutNode { node } => {
                require_semantic_id(&node.id, "RepoModel node")?;
                if !staged_domain_ids.contains(&node.domain_id) {
                    require_dependency_envelope::<EpiphanyRepoModelDomainDocument>(
                        &cache,
                        &node.domain_id,
                        &mut strong,
                    )?;
                }
                if let Some(existing) =
                    cache.get_envelope::<EpiphanyRepoModelNodeDocument>(&node.id)?
                {
                    insert_strong_envelope(&mut strong, existing)?;
                }
                nodes.insert(node.id.clone(), node.clone());
                let envelope = cache
                    .prepare_entry(&node.id, &EpiphanyRepoModelNodeDocument::new(node)?)?
                    .0;
                insert_envelope(&mut writes, envelope)?;
            }
            EpiphanyRepoModelMutationOperation::RetireNode { node_id } => {
                require_semantic_id(node_id, "RepoModel node")?;
                let mut node = nodes
                    .get(node_id)
                    .cloned()
                    .ok_or_else(|| anyhow!("RepoModel mutation retires a missing node"))?;
                let existing = cache
                    .get_envelope::<EpiphanyRepoModelNodeDocument>(node_id)?
                    .ok_or_else(|| anyhow!("RepoModel node lost its envelope"))?;
                insert_strong_envelope(&mut strong, existing)?;
                let obligation = obligations.entry(node_id.clone()).or_insert_with(|| {
                    EpiphanyRepoModelClaimObligationsDocument {
                        node_id: node_id.clone(),
                        unresolved_frontier_ids: Vec::new(),
                    }
                });
                if !obligation.unresolved_frontier_ids.is_empty() {
                    return Err(anyhow!(
                        "RepoModel node retirement is blocked by unresolved frontier"
                    ));
                }
                if let Some(existing) =
                    cache.get_envelope::<EpiphanyRepoModelClaimObligationsDocument>(node_id)?
                {
                    insert_strong_envelope(&mut strong, existing)?;
                }
                node.lifecycle = epiphany_state_model::EpiphanyMemoryLifecycle::Retired;
                nodes.insert(node_id.clone(), node.clone());
                insert_envelope(
                    &mut writes,
                    cache
                        .prepare_entry(node_id, &EpiphanyRepoModelNodeDocument::new(&node)?)?
                        .0,
                )?;
                insert_envelope(&mut writes, cache.prepare_entry(node_id, obligation)?.0)?;
            }
            EpiphanyRepoModelMutationOperation::PutEdge { edge } => {
                require_semantic_id(&edge.id, "RepoModel edge")?;
                for node_id in [&edge.source_id, &edge.target_id] {
                    if !staged_node_ids.contains(node_id) {
                        require_dependency_envelope::<EpiphanyRepoModelNodeDocument>(
                            &cache,
                            node_id,
                            &mut strong,
                        )?;
                    }
                }
                if let Some(existing) =
                    cache.get_envelope::<EpiphanyRepoModelEdgeDocument>(&edge.id)?
                {
                    insert_strong_envelope(&mut strong, existing)?;
                }
                edges.insert(edge.id.clone(), edge.clone());
                insert_envelope(
                    &mut writes,
                    cache
                        .prepare_entry(&edge.id, &EpiphanyRepoModelEdgeDocument::new(edge)?)?
                        .0,
                )?;
            }
            EpiphanyRepoModelMutationOperation::RetireEdge { edge_id } => {
                require_semantic_id(edge_id, "RepoModel edge")?;
                let mut edge = edges
                    .get(edge_id)
                    .cloned()
                    .ok_or_else(|| anyhow!("RepoModel mutation retires a missing edge"))?;
                let existing = cache
                    .get_envelope::<EpiphanyRepoModelEdgeDocument>(edge_id)?
                    .ok_or_else(|| anyhow!("RepoModel edge lost its envelope"))?;
                insert_strong_envelope(&mut strong, existing)?;
                edge.lifecycle = epiphany_state_model::EpiphanyMemoryLifecycle::Retired;
                edges.insert(edge_id.clone(), edge.clone());
                insert_envelope(
                    &mut writes,
                    cache
                        .prepare_entry(edge_id, &EpiphanyRepoModelEdgeDocument::new(&edge)?)?
                        .0,
                )?;
            }
            EpiphanyRepoModelMutationOperation::PutSummary { summary } => {
                require_semantic_id(&summary.id, "RepoModel summary")?;
                if !staged_domain_ids.contains(&summary.domain_id) {
                    require_dependency_envelope::<EpiphanyRepoModelDomainDocument>(
                        &cache,
                        &summary.domain_id,
                        &mut strong,
                    )?;
                }
                for node_id in &summary.covers_node_ids {
                    if !staged_node_ids.contains(node_id) {
                        require_dependency_envelope::<EpiphanyRepoModelNodeDocument>(
                            &cache,
                            node_id,
                            &mut strong,
                        )?;
                    }
                }
                for edge_id in &summary.covers_edge_ids {
                    if !staged_edge_ids.contains(edge_id) {
                        require_dependency_envelope::<EpiphanyRepoModelEdgeDocument>(
                            &cache,
                            edge_id,
                            &mut strong,
                        )?;
                    }
                }
                if let Some(existing) =
                    cache.get_envelope::<EpiphanyRepoModelSummaryDocument>(&summary.id)?
                {
                    insert_strong_envelope(&mut strong, existing)?;
                }
                summaries.insert(summary.id.clone(), summary.clone());
                insert_envelope(
                    &mut writes,
                    cache
                        .prepare_entry(
                            &summary.id,
                            &EpiphanyRepoModelSummaryDocument::new(summary)?,
                        )?
                        .0,
                )?;
            }
            EpiphanyRepoModelMutationOperation::PutFrontier { item } => {
                require_semantic_id(&item.id, "RepoModel frontier")?;
                let prior = frontier.get(&item.id).cloned();
                if let Some(existing) =
                    cache.get_envelope::<EpiphanyRepoModelFrontierDocument>(&item.id)?
                {
                    insert_strong_envelope(&mut strong, existing)?;
                }
                for node_id in &item.target_claim_ids {
                    if !staged_node_ids.contains(node_id) {
                        require_dependency_envelope::<EpiphanyRepoModelNodeDocument>(
                            &cache,
                            node_id,
                            &mut strong,
                        )?;
                    }
                }
                for frontier_id in item
                    .dependency_item_ids
                    .iter()
                    .chain(item.superseded_by.iter())
                {
                    if !staged_frontier_ids.contains(frontier_id) {
                        require_dependency_envelope::<EpiphanyRepoModelFrontierDocument>(
                            &cache,
                            frontier_id,
                            &mut strong,
                        )?;
                    }
                }
                let affected = prior
                    .iter()
                    .flat_map(|value| value.target_claim_ids.iter())
                    .chain(item.target_claim_ids.iter())
                    .cloned()
                    .collect::<BTreeSet<_>>();
                frontier.insert(item.id.clone(), item.clone());
                for node_id in affected {
                    let obligation = obligations.entry(node_id.clone()).or_insert_with(|| {
                        EpiphanyRepoModelClaimObligationsDocument {
                            node_id: node_id.clone(),
                            unresolved_frontier_ids: Vec::new(),
                        }
                    });
                    if let Some(existing) =
                        cache.get_envelope::<EpiphanyRepoModelClaimObligationsDocument>(&node_id)?
                    {
                        insert_strong_envelope(&mut strong, existing)?;
                    }
                    obligation
                        .unresolved_frontier_ids
                        .retain(|frontier_id| frontier_id != &item.id);
                    if frontier_is_unresolved(item) && item.target_claim_ids.contains(&node_id) {
                        obligation.unresolved_frontier_ids.push(item.id.clone());
                        obligation.unresolved_frontier_ids.sort();
                        obligation.unresolved_frontier_ids.dedup();
                    }
                    insert_envelope(&mut writes, cache.prepare_entry(&node_id, obligation)?.0)?;
                }
                insert_envelope(
                    &mut writes,
                    cache
                        .prepare_entry(&item.id, &EpiphanyRepoModelFrontierDocument::new(item)?)?
                        .0,
                )?;
            }
        }
    }

    validate_repo_model_parts(
        &view.identity,
        domains.values().cloned().collect(),
        nodes.values().cloned().collect(),
        edges.values().cloned().collect(),
        summaries.values().cloned().collect(),
        frontier.values().cloned().collect(),
        view.lifecycle_receipts,
        obligations.values().cloned().collect(),
    )?;
    Ok(EpiphanyRepoModelMutationPlan {
        proposal_id: proposal.proposal_id.clone(),
        strong_reads: strong.into_values().collect(),
        writes: writes.into_values().collect(),
    })
}

fn repo_model_operation_identity(
    operation: &EpiphanyRepoModelMutationOperation,
) -> Result<(String, String)> {
    let (kind, id) = match operation {
        EpiphanyRepoModelMutationOperation::PutDomain { domain } => ("domain", &domain.id),
        EpiphanyRepoModelMutationOperation::PutNode { node } => ("node", &node.id),
        EpiphanyRepoModelMutationOperation::RetireNode { node_id } => ("node", node_id),
        EpiphanyRepoModelMutationOperation::PutEdge { edge } => ("edge", &edge.id),
        EpiphanyRepoModelMutationOperation::RetireEdge { edge_id } => ("edge", edge_id),
        EpiphanyRepoModelMutationOperation::PutSummary { summary } => ("summary", &summary.id),
        EpiphanyRepoModelMutationOperation::PutFrontier { item } => ("frontier", &item.id),
    };
    require_semantic_id(id, "RepoModel operation")?;
    Ok((kind.into(), id.clone()))
}

fn insert_envelope(
    target: &mut BTreeMap<(String, String), CultCacheEnvelope>,
    envelope: CultCacheEnvelope,
) -> Result<()> {
    let identity = (envelope.r#type.clone(), envelope.key.clone());
    target.insert(identity, envelope);
    Ok(())
}

fn insert_strong_envelope(
    target: &mut BTreeMap<(String, String), CultCacheEnvelope>,
    envelope: CultCacheEnvelope,
) -> Result<()> {
    let identity = (envelope.r#type.clone(), envelope.key.clone());
    if let Some(existing) = target.get(&identity) {
        if existing != &envelope {
            return Err(anyhow!(
                "RepoModel strong dependency changed while planning"
            ));
        }
        return Ok(());
    }
    target.insert(identity, envelope);
    Ok(())
}

fn require_dependency_envelope<T: DatabaseEntry>(
    cache: &CultCache,
    key: &str,
    strong: &mut BTreeMap<(String, String), CultCacheEnvelope>,
) -> Result<()> {
    require_semantic_id(key, "RepoModel dependency")?;
    let envelope = cache
        .get_envelope::<T>(key)?
        .ok_or_else(|| anyhow!("RepoModel mutation dependency is absent"))?;
    if !strong.contains_key(&(envelope.r#type.clone(), envelope.key.clone())) {
        insert_strong_envelope(strong, envelope)?;
    }
    Ok(())
}

fn require_semantic_id(value: &str, label: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(anyhow!("{label} identity cannot be empty"));
    }
    Ok(())
}

fn frontier_is_unresolved(item: &RepoFrontierItem) -> bool {
    matches!(
        item.status,
        epiphany_state_model::RepoFrontierStatus::Proposed
            | epiphany_state_model::RepoFrontierStatus::Active
            | epiphany_state_model::RepoFrontierStatus::Blocked
    )
}

pub fn assemble_repo_model_view(store_path: impl AsRef<Path>) -> Result<EpiphanyRepoModelView> {
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    let identity = cache
        .get::<EpiphanyRepoModelIdentityDocument>(REPO_MODEL_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("writable Mind store has no keyed RepoModel identity"))?;
    let mut domains = values::<EpiphanyRepoModelDomainDocument, _>(&cache, |entry| entry.value())?;
    let mut nodes = values::<EpiphanyRepoModelNodeDocument, _>(&cache, |entry| entry.value())?;
    let mut edges = values::<EpiphanyRepoModelEdgeDocument, _>(&cache, |entry| entry.value())?;
    let mut summaries =
        values::<EpiphanyRepoModelSummaryDocument, _>(&cache, |entry| entry.value())?;
    let mut frontier =
        values::<EpiphanyRepoModelFrontierDocument, _>(&cache, |entry| entry.value())?;
    let mut lifecycle_receipts =
        values::<EpiphanyRepoModelLifecycleReceiptDocument, _>(&cache, |entry| entry.value())?;
    let mut claim_obligations = cache.get_all::<EpiphanyRepoModelClaimObligationsDocument>()?;
    domains.sort_by(|left, right| left.id.cmp(&right.id));
    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    edges.sort_by(|left, right| left.id.cmp(&right.id));
    summaries.sort_by(|left, right| left.id.cmp(&right.id));
    frontier.sort_by(|left, right| left.id.cmp(&right.id));
    lifecycle_receipts.sort_by(|left, right| left.id.cmp(&right.id));
    claim_obligations.sort_by(|left, right| left.node_id.cmp(&right.node_id));

    validate_repo_model_parts(
        &identity,
        domains.clone(),
        nodes.clone(),
        edges.clone(),
        summaries.clone(),
        frontier.clone(),
        lifecycle_receipts.clone(),
        claim_obligations.clone(),
    )?;

    let mut source_documents = cache
        .snapshot_envelopes()
        .into_iter()
        .filter(|entry| entry.r#type.starts_with("epiphany.mind.repo_model."))
        .map(|entry| EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", &entry))
        .collect::<Result<Vec<_>>>()?;
    source_documents.sort_by(|left, right| {
        (&left.document_type, &left.document_key).cmp(&(&right.document_type, &right.document_key))
    });
    let projection_digest = format!(
        "sha256:{:x}",
        Sha256::digest(rmp_serde::to_vec_named(&source_documents)?)
    );
    Ok(EpiphanyRepoModelView {
        identity,
        projection_digest,
        source_documents,
        domains,
        nodes,
        edges,
        summaries,
        frontier,
        lifecycle_receipts,
        claim_obligations,
    })
}

fn values<T, V>(cache: &CultCache, take: impl Fn(T) -> Result<V>) -> Result<Vec<V>>
where
    T: DatabaseEntry,
{
    cache.get_all::<T>()?.into_iter().map(take).collect()
}

#[allow(clippy::too_many_arguments)]
fn validate_repo_model_parts(
    identity: &EpiphanyRepoModelIdentityDocument,
    domains: Vec<EpiphanyMemoryDomain>,
    nodes: Vec<EpiphanyMemoryNode>,
    edges: Vec<EpiphanyMemoryEdge>,
    summaries: Vec<EpiphanyMemorySummary>,
    frontier: Vec<RepoFrontierItem>,
    lifecycle_receipts: Vec<EpiphanyMemoryLifecycleReceipt>,
    claim_obligations: Vec<EpiphanyRepoModelClaimObligationsDocument>,
) -> Result<()> {
    let validation = EpiphanyMemoryGraphSnapshot {
        graph_id: identity.graph_id.clone(),
        domains,
        nodes,
        edges,
        summaries,
        lifecycle_receipts,
        frontier,
        ..Default::default()
    };
    let errors = validate_memory_graph_snapshot(&validation);
    if !errors.is_empty() {
        return Err(anyhow!(
            "keyed RepoModel view is invalid: {}",
            errors
                .iter()
                .map(|error| format!("{}: {}", error.path, error.message))
                .collect::<Vec<_>>()
                .join("; ")
        ));
    }
    validate_claim_obligations(&validation.nodes, &validation.frontier, &claim_obligations)
}

fn validate_claim_obligations(
    nodes: &[EpiphanyMemoryNode],
    frontier: &[RepoFrontierItem],
    obligations: &[EpiphanyRepoModelClaimObligationsDocument],
) -> Result<()> {
    for obligation in obligations {
        if !nodes.iter().any(|node| node.id == obligation.node_id) {
            return Err(anyhow!("RepoModel claim obligation names a missing node"));
        }
        for frontier_id in &obligation.unresolved_frontier_ids {
            let item = frontier
                .iter()
                .find(|item| item.id == *frontier_id)
                .ok_or_else(|| anyhow!("RepoModel claim obligation names a missing frontier"))?;
            if !item.target_claim_ids.contains(&obligation.node_id)
                || !matches!(
                    item.status,
                    epiphany_state_model::RepoFrontierStatus::Proposed
                        | epiphany_state_model::RepoFrontierStatus::Active
                        | epiphany_state_model::RepoFrontierStatus::Blocked
                )
            {
                return Err(anyhow!("RepoModel claim obligation is not live"));
            }
        }
    }
    for item in frontier.iter().filter(|item| {
        matches!(
            item.status,
            epiphany_state_model::RepoFrontierStatus::Proposed
                | epiphany_state_model::RepoFrontierStatus::Active
                | epiphany_state_model::RepoFrontierStatus::Blocked
        )
    }) {
        for node_id in &item.target_claim_ids {
            if !obligations.iter().any(|obligation| {
                obligation.node_id == *node_id
                    && obligation.unresolved_frontier_ids.contains(&item.id)
            }) {
                return Err(anyhow!("unresolved frontier lost its claim obligation"));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{initialize_runtime_spine, RuntimeSpineInitOptions};
    use epiphany_state_model::{
        EpiphanyMemoryEdgeKind, EpiphanyMemoryLifecycle, EpiphanyMemoryNodeKind,
        EpiphanyMemoryProfile, RepoFrontierStatus,
    };

    #[test]
    fn keyed_repo_model_view_is_deterministic_and_has_no_global_revision() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("repo-model.cc");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "repo-model-view".into(),
                display_name: "RepoModel view".into(),
                created_at: "2026-08-14T00:00:00Z".into(),
            },
        )?;
        let mut cache = runtime_spine_cache(&store)?;
        cache.put(
            REPO_MODEL_IDENTITY_KEY,
            &EpiphanyRepoModelIdentityDocument {
                schema_epoch: REPO_MODEL_SCHEMA_EPOCH.into(),
                graph_id: "graph-1".into(),
                runtime_id: "repo-model-view".into(),
                swarm_id: "swarm-1".into(),
                workspace_id: "workspace-1".into(),
                body_binding_sha256: "sha256:body".into(),
            },
        )?;
        cache.put(
            "domain-1",
            &EpiphanyRepoModelDomainDocument::new(&EpiphanyMemoryDomain {
                id: "domain-1".into(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                title: "Architecture".into(),
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                ..Default::default()
            })?,
        )?;
        cache.put(
            "node-1",
            &EpiphanyRepoModelNodeDocument::new(&EpiphanyMemoryNode {
                id: "node-1".into(),
                domain_id: "domain-1".into(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                kind: EpiphanyMemoryNodeKind::Module,
                title: "Mind".into(),
                claim: "Mind state is keyed".into(),
                question: "Which owner writes next?".into(),
                action_implication: "Keep mutations disjoint".into(),
                source_hashes: vec!["anchor:missing".into()],
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                ..Default::default()
            })?,
        )?;
        let first = assemble_repo_model_view(&store)?;
        let second = assemble_repo_model_view(&store)?;
        assert_eq!(first, second);
        assert_eq!(first.nodes[0].id, "node-1");
        assert!(first.projection_digest.starts_with("sha256:"));
        let mind = crate::assemble_mind_view(&store)?;
        assert_eq!(mind.repo_model.as_ref(), Some(&first));

        let retire = EpiphanyRepoModelMutationProposal::new(
            "proposal-retire",
            vec![EpiphanyRepoModelMutationOperation::RetireNode {
                node_id: "node-1".into(),
            }],
        )?;
        let retire_plan = plan_repo_model_mutation(&store, &retire)?;
        let add_frontier = EpiphanyRepoModelMutationProposal::new(
            "proposal-frontier",
            vec![EpiphanyRepoModelMutationOperation::PutFrontier {
                item: RepoFrontierItem {
                    id: "frontier-1".into(),
                    migration_body: "repo".into(),
                    question: "Cut the aggregate?".into(),
                    target_claim_ids: vec!["node-1".into()],
                    recommended_next_organ: "Modeling".into(),
                    status: RepoFrontierStatus::Active,
                    ..Default::default()
                },
            }],
        )?;
        let frontier_plan = plan_repo_model_mutation(&store, &add_frontier)?;
        let frontier_provenance = cache
            .prepare_entry(&add_frontier.proposal_id, &add_frontier)?
            .0;
        assert!(matches!(
            crate::commit_operator_mind_mutation(
                &store,
                frontier_provenance,
                "Modeling.repo_model_mutation",
                frontier_plan.strong_reads,
                frontier_plan.writes,
                "2026-08-14T00:00:01Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));
        let retire_provenance = cache.prepare_entry(&retire.proposal_id, &retire)?.0;
        assert!(matches!(
            crate::commit_operator_mind_mutation(
                &store,
                retire_provenance,
                "Modeling.repo_model_mutation",
                retire_plan.strong_reads,
                retire_plan.writes,
                "2026-08-14T00:00:02Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Conflict { .. }
        ));
        let after = assemble_repo_model_view(&store)?;
        assert_eq!(after.frontier[0].id, "frontier-1");
        assert_eq!(
            after.claim_obligations[0].unresolved_frontier_ids,
            ["frontier-1"]
        );
        assert_eq!(after.nodes[0].lifecycle, EpiphanyMemoryLifecycle::Accepted);
        Ok(())
    }

    #[test]
    fn one_mutation_can_create_a_complete_keyed_graph_slice_in_any_operation_order() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("repo-model.cc");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "repo-model-atomic-slice".into(),
                display_name: "RepoModel atomic slice".into(),
                created_at: "2026-08-14T00:00:00Z".into(),
            },
        )?;
        let mut cache = runtime_spine_cache(&store)?;
        cache.put(
            REPO_MODEL_IDENTITY_KEY,
            &EpiphanyRepoModelIdentityDocument {
                schema_epoch: REPO_MODEL_SCHEMA_EPOCH.into(),
                graph_id: "graph-atomic".into(),
                runtime_id: "repo-model-atomic-slice".into(),
                swarm_id: "swarm-1".into(),
                workspace_id: "workspace-1".into(),
                body_binding_sha256: "sha256:body".into(),
            },
        )?;
        let domain = EpiphanyMemoryDomain {
            id: "domain-new".into(),
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            title: "New domain".into(),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        };
        let node = |id: &str, claim: &str| EpiphanyMemoryNode {
            id: id.into(),
            domain_id: domain.id.clone(),
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            kind: EpiphanyMemoryNodeKind::Module,
            title: id.into(),
            claim: claim.into(),
            question: "What does this own?".into(),
            action_implication: "Keep the owner explicit".into(),
            source_hashes: vec!["anchor:missing".into()],
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        };
        let left = node("node-left", "Left owns input");
        let right = node("node-right", "Right owns output");
        let edge = EpiphanyMemoryEdge {
            id: "edge-flow".into(),
            source_id: left.id.clone(),
            target_id: right.id.clone(),
            kind: EpiphanyMemoryEdgeKind::Writes,
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            claim: "Left writes Right".into(),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        };
        let summary = EpiphanyMemorySummary {
            id: "summary-flow".into(),
            domain_id: domain.id.clone(),
            covers_node_ids: vec![left.id.clone(), right.id.clone()],
            covers_edge_ids: vec![edge.id.clone()],
            target: "flow".into(),
            claim: "The flow is document-addressed".into(),
            question: "Can its members commit atomically?".into(),
            action_implication: "Commit the exact keyed slice".into(),
            ..Default::default()
        };
        let proposal = EpiphanyRepoModelMutationProposal::new(
            "proposal-atomic-slice",
            vec![
                EpiphanyRepoModelMutationOperation::PutSummary {
                    summary: summary.clone(),
                },
                EpiphanyRepoModelMutationOperation::PutEdge { edge: edge.clone() },
                EpiphanyRepoModelMutationOperation::PutNode {
                    node: right.clone(),
                },
                EpiphanyRepoModelMutationOperation::PutNode { node: left.clone() },
                EpiphanyRepoModelMutationOperation::PutDomain {
                    domain: domain.clone(),
                },
            ],
        )?;
        let plan = plan_repo_model_mutation(&store, &proposal)?;
        let provenance = cache.prepare_entry(&proposal.proposal_id, &proposal)?.0;
        assert!(matches!(
            crate::commit_operator_mind_mutation(
                &store,
                provenance,
                "Modeling.repo_model_mutation",
                plan.strong_reads,
                plan.writes,
                "2026-08-14T00:00:01Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));
        let view = assemble_repo_model_view(&store)?;
        assert_eq!(view.domains, [domain]);
        assert_eq!(view.nodes, [left, right]);
        assert_eq!(view.edges, [edge.clone()]);
        assert_eq!(view.summaries, [summary]);

        let retire = EpiphanyRepoModelMutationProposal::new(
            "proposal-retire-edge",
            vec![EpiphanyRepoModelMutationOperation::RetireEdge {
                edge_id: "edge-flow".into(),
            }],
        )?;
        let retire_plan = plan_repo_model_mutation(&store, &retire)?;
        let retire_provenance = cache.prepare_entry(&retire.proposal_id, &retire)?.0;
        assert!(matches!(
            crate::commit_operator_mind_mutation(
                &store,
                retire_provenance,
                "Modeling.repo_model_mutation",
                retire_plan.strong_reads,
                retire_plan.writes,
                "2026-08-14T00:00:02Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));
        assert_eq!(
            assemble_repo_model_view(&store)?.edges[0].lifecycle,
            EpiphanyMemoryLifecycle::Retired
        );

        let duplicate = EpiphanyRepoModelMutationProposal::new(
            "proposal-duplicate-edge",
            vec![
                EpiphanyRepoModelMutationOperation::PutEdge { edge },
                EpiphanyRepoModelMutationOperation::RetireEdge {
                    edge_id: "edge-flow".into(),
                },
            ],
        )?;
        assert!(plan_repo_model_mutation(&store, &duplicate).is_err());
        Ok(())
    }
}
