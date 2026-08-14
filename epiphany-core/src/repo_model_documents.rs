use std::path::Path;

use anyhow::{Result, anyhow};
use cultcache_rs::{CultCache, CultCacheEnvelope, DatabaseEntry};
use epiphany_state_model::{
    EpiphanyMemoryDomain, EpiphanyMemoryEdge, EpiphanyMemoryGraphSnapshot,
    EpiphanyMemoryLifecycleReceipt, EpiphanyMemoryNode, EpiphanyMemorySummary, RepoFrontierItem,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{EpiphanyMindDocumentVersion, runtime_spine_cache, validate_memory_graph_snapshot};

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

pub(crate) fn register_repo_model_document_types(cache: &mut CultCache) -> Result<()> {
    cache.register_entry_type::<EpiphanyRepoModelIdentityDocument>()?;
    cache.register_entry_type::<EpiphanyRepoModelDomainDocument>()?;
    cache.register_entry_type::<EpiphanyRepoModelNodeDocument>()?;
    cache.register_entry_type::<EpiphanyRepoModelEdgeDocument>()?;
    cache.register_entry_type::<EpiphanyRepoModelSummaryDocument>()?;
    cache.register_entry_type::<EpiphanyRepoModelFrontierDocument>()?;
    cache.register_entry_type::<EpiphanyRepoModelLifecycleReceiptDocument>()?;
    cache.register_entry_type::<EpiphanyRepoModelClaimObligationsDocument>()?;
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

    let validation = EpiphanyMemoryGraphSnapshot {
        graph_id: identity.graph_id.clone(),
        domains: domains.clone(),
        nodes: nodes.clone(),
        edges: edges.clone(),
        summaries: summaries.clone(),
        lifecycle_receipts: lifecycle_receipts.clone(),
        frontier: frontier.clone(),
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
    validate_claim_obligations(&nodes, &frontier, &claim_obligations)?;

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
    use crate::{RuntimeSpineInitOptions, initialize_runtime_spine};
    use epiphany_state_model::{
        EpiphanyMemoryLifecycle, EpiphanyMemoryNodeKind, EpiphanyMemoryProfile, RepoFrontierStatus,
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
        cache.put(
            "frontier-1",
            &EpiphanyRepoModelFrontierDocument::new(&RepoFrontierItem {
                id: "frontier-1".into(),
                migration_body: "repo".into(),
                question: "Cut the aggregate?".into(),
                target_claim_ids: vec!["node-1".into()],
                recommended_next_organ: "Modeling".into(),
                status: RepoFrontierStatus::Active,
                ..Default::default()
            })?,
        )?;
        cache.put(
            "node-1",
            &EpiphanyRepoModelClaimObligationsDocument {
                node_id: "node-1".into(),
                unresolved_frontier_ids: vec!["frontier-1".into()],
            },
        )?;
        let first = assemble_repo_model_view(&store)?;
        let second = assemble_repo_model_view(&store)?;
        assert_eq!(first, second);
        assert_eq!(first.nodes[0].id, "node-1");
        assert!(first.projection_digest.starts_with("sha256:"));
        let mind = crate::assemble_mind_view(&store)?;
        assert_eq!(mind.repo_model.as_ref(), Some(&first));
        Ok(())
    }
}
