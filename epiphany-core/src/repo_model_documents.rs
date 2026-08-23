use std::collections::{BTreeMap, BTreeSet};
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

pub const REPO_MODEL_SCHEMA_EPOCH: &str = "epiphany.repo_model.epoch.v2";
pub const REPO_MODEL_IDENTITY_KEY: &str = "repo-model";
const HISTORICAL_AGGREGATE_REPO_MODEL_TYPE: &str = "epiphany.memory_graph";
const HISTORICAL_AGGREGATE_REPO_MODEL_KEY: &str = "default";

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
    type = "epiphany.mind.repo_model.claim_obligations.v2",
    schema = "EpiphanyRepoModelClaimObligationsDocument"
)]
pub struct EpiphanyRepoModelClaimObligationsDocument {
    #[cultcache(key = 0)]
    pub node_id: String,
    #[cultcache(key = 1)]
    pub unresolved_frontier_ids: Vec<String>,
    #[cultcache(key = 2)]
    pub active_challenge_ids: Vec<String>,
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
    pub surface_offers: Vec<crate::AtlasSurfaceOffer>,
    pub dependency_claims: Vec<crate::AtlasDependencyClaim>,
    pub dependency_verifications: Vec<crate::AtlasDependencyVerification>,
    pub dependency_impacts: Vec<crate::AtlasDependencyImpact>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyRepoModelBasis {
    pub projection_digest: String,
    pub source_documents: Vec<EpiphanyMindDocumentVersion>,
}

impl EpiphanyRepoModelView {
    pub fn reasoning_basis(&self) -> EpiphanyRepoModelBasis {
        EpiphanyRepoModelBasis {
            projection_digest: self.projection_digest.clone(),
            source_documents: self.source_documents.clone(),
        }
    }

    pub fn memory_context_projection(&self) -> EpiphanyMemoryGraphSnapshot {
        EpiphanyMemoryGraphSnapshot {
            schema_version: None,
            graph_id: self.identity.graph_id.clone(),
            model_revision: 0,
            model_hash: String::new(),
            source: None,
            domains: self.domains.clone(),
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            summaries: self.summaries.clone(),
            embedding_manifest: None,
            freshness: None,
            lifecycle_receipts: self.lifecycle_receipts.clone(),
            frontier: self.frontier.clone(),
        }
    }
}

pub(crate) fn derive_repo_model_semantic_projection_obligation(
    view: &EpiphanyRepoModelView,
    created_at: &str,
) -> Result<crate::MemorySemanticProjectionObligation> {
    let mut projection = view.memory_context_projection();
    projection.model_revision = 1;
    projection.model_hash = crate::memory_graph_model_hash(&projection)?;
    crate::derive_memory_semantic_projection_obligation(
        &projection,
        &view.identity.swarm_id,
        &format!("epiphany.runtime/{}/repo-model", view.identity.runtime_id),
        &view.projection_digest,
        created_at,
    )
}

impl EpiphanyRepoModelBasis {
    pub fn validate(&self) -> Result<()> {
        let digest = self
            .projection_digest
            .strip_prefix("sha256:")
            .ok_or_else(|| anyhow!("RepoModel basis digest has no SHA-256 scheme"))?;
        if digest.len() != 64
            || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
            || self.source_documents.is_empty()
        {
            return Err(anyhow!("RepoModel basis is empty or has an invalid digest"));
        }
        let mut canonical = self.source_documents.clone();
        canonical.sort_by(|left, right| {
            left.document_type
                .cmp(&right.document_type)
                .then(left.document_key.cmp(&right.document_key))
        });
        if canonical != self.source_documents
            || canonical
                .windows(2)
                .any(|pair| pair[0].identity() == pair[1].identity())
        {
            return Err(anyhow!("RepoModel basis sources are not canonical"));
        }
        for source in &self.source_documents {
            source.validate()?;
            if source.store_id != "epiphany-mind" {
                return Err(anyhow!("RepoModel basis references a foreign store"));
            }
        }
        let expected = format!(
            "sha256:{:x}",
            Sha256::digest(rmp_serde::to_vec_named(&self.source_documents)?)
        );
        if self.projection_digest != expected {
            return Err(anyhow!("RepoModel basis projection digest mismatch"));
        }
        Ok(())
    }

    pub fn validate_current(&self, store_path: impl AsRef<Path>) -> Result<()> {
        self.validate()?;
        let current = assemble_repo_model_view(store_path)?.reasoning_basis();
        if current != *self {
            return Err(anyhow!("RepoModel basis is stale"));
        }
        Ok(())
    }

    pub fn validate_against_cache(&self, cache: &CultCache) -> Result<()> {
        self.validate()?;
        let live = cache.snapshot_envelopes();
        for source in &self.source_documents {
            let envelope = live
                .iter()
                .find(|envelope| {
                    envelope.r#type == source.document_type && envelope.key == source.document_key
                })
                .ok_or_else(|| anyhow!("RepoModel basis source is absent"))?;
            if EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", envelope)? != *source {
                return Err(anyhow!("RepoModel basis source changed"));
            }
        }
        let mut live_repo_sources = live
            .iter()
            .filter(|envelope| repo_model_write_key(envelope).is_ok_and(|key| key.is_some()))
            .map(|envelope| EpiphanyMindDocumentVersion::from_envelope("epiphany-mind", envelope))
            .collect::<Result<Vec<_>>>()?;
        live_repo_sources.sort_by(|left, right| {
            left.document_type
                .cmp(&right.document_type)
                .then(left.document_key.cmp(&right.document_key))
        });
        let expected = self.source_documents.clone();
        if live_repo_sources != expected {
            return Err(anyhow!(
                "RepoModel basis does not seal the complete keyed view"
            ));
        }
        Ok(())
    }
}

pub const REPO_MODEL_MUTATION_PROPOSAL_SCHEMA_VERSION: &str =
    "epiphany.repo_model.mutation_proposal.v2";
pub const REPO_MODEL_SEED_SCHEMA_VERSION: &str = "epiphany.repo_model.seed.v1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpiphanyRepoModelSeedDocuments {
    pub domains: Vec<EpiphanyMemoryDomain>,
    pub nodes: Vec<EpiphanyMemoryNode>,
    pub edges: Vec<EpiphanyMemoryEdge>,
    pub summaries: Vec<EpiphanyMemorySummary>,
    pub frontier: Vec<RepoFrontierItem>,
    pub lifecycle_receipts: Vec<EpiphanyMemoryLifecycleReceipt>,
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.body.repo_model_seed.v1",
    schema = "EpiphanyRepoModelSeed"
)]
pub struct EpiphanyRepoModelSeed {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub seed_id: String,
    #[cultcache(key = 2)]
    pub graph_id: String,
    #[cultcache(key = 3)]
    pub swarm_id: String,
    #[cultcache(key = 4)]
    pub workspace_id: String,
    #[cultcache(key = 5)]
    pub body_binding_sha256: String,
    #[cultcache(key = 6)]
    pub documents_msgpack: Vec<u8>,
}

impl EpiphanyRepoModelSeed {
    pub fn new(
        seed_id: impl Into<String>,
        graph_id: impl Into<String>,
        swarm_id: impl Into<String>,
        workspace_id: impl Into<String>,
        body_binding_sha256: impl Into<String>,
        mut documents: EpiphanyRepoModelSeedDocuments,
    ) -> Result<Self> {
        documents
            .domains
            .sort_by(|left, right| left.id.cmp(&right.id));
        documents
            .nodes
            .sort_by(|left, right| left.id.cmp(&right.id));
        documents
            .edges
            .sort_by(|left, right| left.id.cmp(&right.id));
        documents
            .summaries
            .sort_by(|left, right| left.id.cmp(&right.id));
        documents
            .frontier
            .sort_by(|left, right| left.id.cmp(&right.id));
        documents
            .lifecycle_receipts
            .sort_by(|left, right| left.id.cmp(&right.id));
        let seed = Self {
            schema_version: REPO_MODEL_SEED_SCHEMA_VERSION.into(),
            seed_id: seed_id.into(),
            graph_id: graph_id.into(),
            swarm_id: swarm_id.into(),
            workspace_id: workspace_id.into(),
            body_binding_sha256: body_binding_sha256.into(),
            documents_msgpack: rmp_serde::to_vec_named(&documents)?,
        };
        seed.validate()?;
        Ok(seed)
    }

    pub fn documents(&self) -> Result<EpiphanyRepoModelSeedDocuments> {
        rmp_serde::from_slice(&self.documents_msgpack)
            .map_err(|error| anyhow!("invalid RepoModel seed documents: {error}"))
    }

    pub fn validate(&self) -> Result<()> {
        if self.schema_version != REPO_MODEL_SEED_SCHEMA_VERSION {
            return Err(anyhow!("unsupported RepoModel seed schema"));
        }
        for (value, label) in [
            (&self.seed_id, "seed"),
            (&self.graph_id, "graph"),
            (&self.swarm_id, "swarm"),
            (&self.workspace_id, "workspace"),
            (&self.body_binding_sha256, "Body binding"),
        ] {
            require_semantic_id(value, label)?;
        }
        let documents = self.documents()?;
        let obligations = claim_obligations_for_frontier(&documents.nodes, &documents.frontier);
        validate_repo_model_parts(
            &EpiphanyRepoModelIdentityDocument {
                schema_epoch: REPO_MODEL_SCHEMA_EPOCH.into(),
                graph_id: self.graph_id.clone(),
                runtime_id: "seed-validation".into(),
                swarm_id: self.swarm_id.clone(),
                workspace_id: self.workspace_id.clone(),
                body_binding_sha256: self.body_binding_sha256.clone(),
            },
            documents.domains,
            documents.nodes,
            documents.edges,
            documents.summaries,
            documents.frontier,
            documents.lifecycle_receipts,
            obligations,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case")]
pub enum EpiphanyRepoModelMutationOperation {
    PutDomain {
        domain: EpiphanyMemoryDomain,
    },
    PutNode {
        node: EpiphanyMemoryNode,
    },
    RetireNode {
        node_id: String,
    },
    PutEdge {
        edge: EpiphanyMemoryEdge,
    },
    RetireEdge {
        edge_id: String,
    },
    PutSummary {
        summary: EpiphanyMemorySummary,
    },
    PutFrontier {
        item: RepoFrontierItem,
    },
    CreateSurfaceOffer {
        label: String,
        contract: crate::AtlasContractDescriptor,
        source_refs: Vec<String>,
    },
    DeprecateSurfaceOffer {
        surface_id: uuid::Uuid,
        replacement_surface_id: Option<uuid::Uuid>,
    },
    WithdrawSurfaceOffer {
        surface_id: uuid::Uuid,
    },
    CreateDependencyClaim {
        label: String,
        target: crate::AtlasDependencyTarget,
        entanglement_kind: crate::AtlasEntanglementKind,
        failure_semantics: crate::AtlasFailureSemantics,
        impact_scope: crate::AtlasImpactScope,
        source_refs: Vec<String>,
    },
    RetireDependencyClaim {
        claim_id: uuid::Uuid,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, DatabaseEntry)]
#[cultcache(
    type = "epiphany.repo_model.mutation_proposal.v2",
    schema = "EpiphanyRepoModelMutationProposal"
)]
pub struct EpiphanyRepoModelMutationProposal {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub proposal_id: String,
    #[cultcache(key = 2)]
    pub causal_request_id: String,
    #[cultcache(key = 3)]
    pub causal_result_id: String,
    #[cultcache(key = 4)]
    pub evidence_ids: Vec<String>,
    #[cultcache(key = 5)]
    pub repository_body_observation_basis: crate::RepositoryBodyObservationBasis,
    #[cultcache(key = 6)]
    pub operations_msgpack: Vec<u8>,
}

impl EpiphanyRepoModelMutationProposal {
    pub fn new(
        proposal_id: impl Into<String>,
        causal_request_id: impl Into<String>,
        causal_result_id: impl Into<String>,
        mut evidence_ids: Vec<String>,
        repository_body_observation_basis: crate::RepositoryBodyObservationBasis,
        operations: Vec<EpiphanyRepoModelMutationOperation>,
    ) -> Result<Self> {
        evidence_ids.sort();
        evidence_ids.dedup();
        let proposal = Self {
            schema_version: REPO_MODEL_MUTATION_PROPOSAL_SCHEMA_VERSION.into(),
            proposal_id: proposal_id.into(),
            causal_request_id: causal_request_id.into(),
            causal_result_id: causal_result_id.into(),
            evidence_ids,
            repository_body_observation_basis,
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
            || self.causal_request_id.trim().is_empty()
            || self.causal_result_id.trim().is_empty()
            || self.evidence_ids.is_empty()
            || self
                .evidence_ids
                .iter()
                .any(|value| value.trim().is_empty())
            || !self.evidence_ids.windows(2).all(|pair| pair[0] < pair[1])
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
    cache.register_entry_type::<EpiphanyRepoModelSeed>()?;
    Ok(())
}

pub fn initialize_keyed_repo_model(
    store_path: impl AsRef<Path>,
    seed: &EpiphanyRepoModelSeed,
    seeded_at: &str,
) -> Result<EpiphanyRepoModelView> {
    seed.validate()?;
    chrono::DateTime::parse_from_rfc3339(seeded_at)
        .map_err(|error| anyhow!("RepoModel seed time is invalid: {error}"))?;
    let store_path = store_path.as_ref();
    let mut cache = runtime_spine_cache(store_path)?;
    cache.pull_all_backing_stores()?;
    if cache.snapshot_envelopes().into_iter().any(|envelope| {
        envelope.r#type == HISTORICAL_AGGREGATE_REPO_MODEL_TYPE
            && envelope.key == HISTORICAL_AGGREGATE_REPO_MODEL_KEY
    }) {
        return Err(anyhow!(
            "keyed RepoModel initialization refuses an aggregate RepoModel store"
        ));
    }
    let runtime = cache
        .get::<crate::EpiphanyRuntimeIdentity>(crate::RUNTIME_IDENTITY_KEY)?
        .ok_or_else(|| anyhow!("RepoModel seed has no runtime identity"))?;
    let mind_identity = cache
        .get::<crate::EpiphanyMindIdentity>(crate::MIND_SCHEMA_EPOCH)?
        .ok_or_else(|| anyhow!("RepoModel seed has no Mind schema identity"))?;
    if mind_identity.schema_epoch != crate::MIND_SCHEMA_EPOCH
        || mind_identity.runtime_id != runtime.runtime_id
    {
        return Err(anyhow!("RepoModel seed crosses the Mind schema epoch"));
    }
    let body_basis = crate::load_current_runtime_repository_body_basis(store_path)?;
    if body_basis.runtime_id != runtime.runtime_id
        || body_basis.swarm_id != seed.swarm_id
        || body_basis.workspace_id != seed.workspace_id
        || body_basis.body_binding_sha256 != seed.body_binding_sha256
    {
        return Err(anyhow!(
            "RepoModel seed disagrees with the authenticated runtime Body binding"
        ));
    }
    let identity = EpiphanyRepoModelIdentityDocument {
        schema_epoch: REPO_MODEL_SCHEMA_EPOCH.into(),
        graph_id: seed.graph_id.clone(),
        runtime_id: runtime.runtime_id,
        swarm_id: seed.swarm_id.clone(),
        workspace_id: seed.workspace_id.clone(),
        body_binding_sha256: seed.body_binding_sha256.clone(),
    };
    if let Some(existing) =
        cache.get::<EpiphanyRepoModelIdentityDocument>(REPO_MODEL_IDENTITY_KEY)?
    {
        if existing != identity
            || cache.get::<EpiphanyRepoModelSeed>(&seed.seed_id)?.as_ref() != Some(seed)
        {
            return Err(anyhow!("RepoModel seed identity collision"));
        }
        let view = assemble_repo_model_view(store_path)?;
        if !repo_model_view_matches_seed(&view, seed)? {
            return Err(anyhow!("RepoModel seed replay found divergent keyed state"));
        }
        crate::runtime_modeling_semantic_projection_input(store_path)?;
        return Ok(view);
    }

    let documents = seed.documents()?;
    let obligations = claim_obligations_for_frontier(&documents.nodes, &documents.frontier);
    let mut writes = Vec::new();
    writes.push(cache.prepare_entry(REPO_MODEL_IDENTITY_KEY, &identity)?.0);
    for value in &documents.domains {
        writes.push(
            cache
                .prepare_entry(&value.id, &EpiphanyRepoModelDomainDocument::new(value)?)?
                .0,
        );
    }
    for value in &documents.nodes {
        writes.push(
            cache
                .prepare_entry(&value.id, &EpiphanyRepoModelNodeDocument::new(value)?)?
                .0,
        );
    }
    for value in &documents.edges {
        writes.push(
            cache
                .prepare_entry(&value.id, &EpiphanyRepoModelEdgeDocument::new(value)?)?
                .0,
        );
    }
    for value in &documents.summaries {
        writes.push(
            cache
                .prepare_entry(&value.id, &EpiphanyRepoModelSummaryDocument::new(value)?)?
                .0,
        );
    }
    for value in &documents.frontier {
        writes.push(
            cache
                .prepare_entry(&value.id, &EpiphanyRepoModelFrontierDocument::new(value)?)?
                .0,
        );
    }
    for value in &documents.lifecycle_receipts {
        writes.push(
            cache
                .prepare_entry(
                    &value.id,
                    &EpiphanyRepoModelLifecycleReceiptDocument::new(value)?,
                )?
                .0,
        );
    }
    for obligation in obligations {
        writes.push(cache.prepare_entry(&obligation.node_id, &obligation)?.0);
    }
    let mind_envelope = cache
        .get_envelope::<crate::EpiphanyMindIdentity>(crate::MIND_SCHEMA_EPOCH)?
        .ok_or_else(|| anyhow!("RepoModel seed lost its Mind identity envelope"))?;
    let provenance = cache.prepare_entry(&seed.seed_id, seed)?.0;
    match crate::commit_operator_mind_mutation(
        store_path,
        provenance,
        "Modeling.repo_model_seed",
        vec![mind_envelope],
        writes,
        seeded_at,
    )? {
        crate::EpiphanyMindCommitOutcome::Committed(_) => {
            crate::runtime_modeling_semantic_projection_input(store_path)?;
            assemble_repo_model_view(store_path)
        }
        crate::EpiphanyMindCommitOutcome::Conflict { .. } => {
            Err(anyhow!("RepoModel seed lost its exact-envelope commit"))
        }
    }
}

fn repo_model_view_matches_seed(
    view: &EpiphanyRepoModelView,
    seed: &EpiphanyRepoModelSeed,
) -> Result<bool> {
    let documents = seed.documents()?;
    Ok(view.identity.graph_id == seed.graph_id
        && view.identity.swarm_id == seed.swarm_id
        && view.identity.workspace_id == seed.workspace_id
        && view.identity.body_binding_sha256 == seed.body_binding_sha256
        && view.domains == documents.domains
        && view.nodes == documents.nodes
        && view.edges == documents.edges
        && view.summaries == documents.summaries
        && view.frontier == documents.frontier
        && view.lifecycle_receipts == documents.lifecycle_receipts
        && view.claim_obligations
            == claim_obligations_for_frontier(&documents.nodes, &documents.frontier))
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
    } else if envelope.r#type == crate::AtlasSurfaceOffer::TYPE {
        let value: crate::AtlasSurfaceOffer = rmp_serde::from_slice(&envelope.payload)?;
        value.validate()?;
        value.surface_id.to_string()
    } else if envelope.r#type == crate::AtlasDependencyClaim::TYPE {
        let value: crate::AtlasDependencyClaim = rmp_serde::from_slice(&envelope.payload)?;
        value.validate()?;
        value.claim_id.to_string()
    } else if envelope.r#type == crate::AtlasDependencyVerification::TYPE {
        let value: crate::AtlasDependencyVerification = rmp_serde::from_slice(&envelope.payload)?;
        value.validate()?;
        value.claim_id.to_string()
    } else if envelope.r#type == crate::AtlasDependencyImpact::TYPE {
        let value: crate::AtlasDependencyImpact = rmp_serde::from_slice(&envelope.payload)?;
        value.validate()?;
        value.impact_id.to_string()
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
    crate::validate_repository_body_observation_basis(
        store_path.as_ref(),
        &proposal.repository_body_observation_basis,
    )?;
    if crate::load_current_runtime_repository_body_basis(store_path.as_ref())?
        != proposal.repository_body_observation_basis
    {
        return Err(anyhow!(
            "RepoModel mutation proposal lost its exact current Repository Body basis"
        ));
    }
    let operations = proposal.operations()?;
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    let view = assemble_repo_model_view(store_path.as_ref())?;
    let local_repository = crate::AtlasRepositoryIdentity::new(
        view.identity.swarm_id.clone(),
        view.identity.workspace_id.clone(),
    )?;
    if proposal.repository_body_observation_basis.swarm_id != view.identity.swarm_id
        || proposal.repository_body_observation_basis.workspace_id != view.identity.workspace_id
        || proposal.repository_body_observation_basis.runtime_id != view.identity.runtime_id
        || proposal
            .repository_body_observation_basis
            .body_binding_sha256
            != view.identity.body_binding_sha256
    {
        return Err(anyhow!(
            "RepoModel mutation proposal Body basis disagrees with the keyed model identity"
        ));
    }
    let body_manifest = crate::authenticated_repository_body_manifest(
        store_path.as_ref(),
        &proposal.repository_body_observation_basis,
    )?;
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
        .enumerate()
        .map(|(index, operation)| {
            repo_model_operation_identity(
                &proposal.proposal_id,
                index,
                &local_repository,
                operation,
            )
        })
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
    for (operation_index, operation) in operations.iter().enumerate() {
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
                let obligation = obligations.entry(node.id.clone()).or_insert_with(|| {
                    EpiphanyRepoModelClaimObligationsDocument {
                        node_id: node.id.clone(),
                        unresolved_frontier_ids: Vec::new(),
                        active_challenge_ids: Vec::new(),
                    }
                });
                if let Some(existing) =
                    cache.get_envelope::<EpiphanyRepoModelClaimObligationsDocument>(&node.id)?
                {
                    insert_strong_envelope(&mut strong, existing)?;
                }
                obligation.active_challenge_ids.clear();
                let envelope = cache
                    .prepare_entry(&node.id, &EpiphanyRepoModelNodeDocument::new(node)?)?
                    .0;
                insert_envelope(&mut writes, envelope)?;
                insert_envelope(&mut writes, cache.prepare_entry(&node.id, obligation)?.0)?;
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
                        active_challenge_ids: Vec::new(),
                    }
                });
                if !obligation.unresolved_frontier_ids.is_empty()
                    || !obligation.active_challenge_ids.is_empty()
                {
                    return Err(anyhow!(
                        "RepoModel node retirement is blocked by unresolved claim obligations"
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
                            active_challenge_ids: Vec::new(),
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
            EpiphanyRepoModelMutationOperation::CreateSurfaceOffer {
                label,
                contract,
                source_refs,
            } => {
                contract.validate()?;
                let body_evidence = resolve_atlas_body_evidence(&body_manifest, source_refs)?;
                let surface_id = derived_atlas_operation_id(
                    &proposal.proposal_id,
                    operation_index,
                    &local_repository,
                    "surface-offer",
                );
                let offer = crate::AtlasSurfaceOffer {
                    schema_version: crate::ATLAS_SURFACE_OFFER_SCHEMA.into(),
                    provider: local_repository.clone(),
                    surface_id,
                    contract: contract.clone(),
                    lifecycle: crate::AtlasOfferLifecycle::Active,
                    label: label.clone(),
                    body_evidence,
                };
                offer.validate()?;
                if let Some(existing) =
                    cache.get_envelope::<crate::AtlasSurfaceOffer>(&surface_id.to_string())?
                {
                    insert_strong_envelope(&mut strong, existing)?;
                }
                insert_envelope(
                    &mut writes,
                    cache.prepare_entry(&surface_id.to_string(), &offer)?.0,
                )?;
            }
            EpiphanyRepoModelMutationOperation::DeprecateSurfaceOffer {
                surface_id,
                replacement_surface_id,
            } => {
                let (mut offer, current_envelope) =
                    load_local_surface_offer(&cache, &local_repository, *surface_id)?;
                if offer.lifecycle != crate::AtlasOfferLifecycle::Active {
                    return Err(anyhow!(
                        "Atlas offer deprecation requires an active local surface"
                    ));
                }
                insert_strong_envelope(&mut strong, current_envelope)?;
                if let Some(replacement_id) = replacement_surface_id {
                    let (replacement, replacement_envelope) =
                        load_local_surface_offer(&cache, &local_repository, *replacement_id)?;
                    if replacement.surface_id == offer.surface_id
                        || replacement.lifecycle != crate::AtlasOfferLifecycle::Active
                        || replacement.contract.contract_id() != offer.contract.contract_id()
                    {
                        return Err(anyhow!(
                            "Atlas replacement surface is recursive, inactive, or contract-ambiguous"
                        ));
                    }
                    insert_strong_envelope(&mut strong, replacement_envelope)?;
                }
                offer.lifecycle = crate::AtlasOfferLifecycle::Deprecated {
                    replacement_surface_id: *replacement_surface_id,
                };
                offer.validate()?;
                insert_envelope(
                    &mut writes,
                    cache.prepare_entry(&surface_id.to_string(), &offer)?.0,
                )?;
            }
            EpiphanyRepoModelMutationOperation::WithdrawSurfaceOffer { surface_id } => {
                let (mut offer, current_envelope) =
                    load_local_surface_offer(&cache, &local_repository, *surface_id)?;
                if !matches!(
                    offer.lifecycle,
                    crate::AtlasOfferLifecycle::Active
                        | crate::AtlasOfferLifecycle::Deprecated { .. }
                ) {
                    return Err(anyhow!(
                        "Atlas offer withdrawal requires an active or deprecated local surface"
                    ));
                }
                insert_strong_envelope(&mut strong, current_envelope)?;
                offer.lifecycle = crate::AtlasOfferLifecycle::Withdrawn;
                offer.validate()?;
                insert_envelope(
                    &mut writes,
                    cache.prepare_entry(&surface_id.to_string(), &offer)?.0,
                )?;
            }
            EpiphanyRepoModelMutationOperation::CreateDependencyClaim {
                label,
                target,
                entanglement_kind,
                failure_semantics,
                impact_scope,
                source_refs,
            } => {
                target.validate()?;
                impact_scope.validate()?;
                let body_evidence = resolve_atlas_body_evidence(&body_manifest, source_refs)?;
                require_local_impact_scope(&cache, &local_repository, impact_scope, &mut strong)?;
                let claim_id = derived_atlas_operation_id(
                    &proposal.proposal_id,
                    operation_index,
                    &local_repository,
                    "dependency-claim",
                );
                let claim = crate::AtlasDependencyClaim {
                    schema_version: crate::ATLAS_DEPENDENCY_CLAIM_SCHEMA.into(),
                    consumer: local_repository.clone(),
                    claim_id,
                    target: target.clone(),
                    entanglement_kind: *entanglement_kind,
                    failure_semantics: *failure_semantics,
                    impact_scope: impact_scope.clone(),
                    lifecycle: crate::AtlasClaimLifecycle::Active,
                    label: label.clone(),
                    body_evidence,
                };
                claim.validate()?;
                if let Some(existing) =
                    cache.get_envelope::<crate::AtlasDependencyClaim>(&claim_id.to_string())?
                {
                    insert_strong_envelope(&mut strong, existing)?;
                }
                insert_envelope(
                    &mut writes,
                    cache.prepare_entry(&claim_id.to_string(), &claim)?.0,
                )?;
            }
            EpiphanyRepoModelMutationOperation::RetireDependencyClaim { claim_id } => {
                let (mut claim, current_envelope) =
                    load_local_dependency_claim(&cache, &local_repository, *claim_id)?;
                if claim.lifecycle != crate::AtlasClaimLifecycle::Active {
                    return Err(anyhow!(
                        "Atlas claim retirement requires an active local claim"
                    ));
                }
                insert_strong_envelope(&mut strong, current_envelope)?;
                claim.lifecycle = crate::AtlasClaimLifecycle::Retired;
                claim.validate()?;
                insert_envelope(
                    &mut writes,
                    cache.prepare_entry(&claim_id.to_string(), &claim)?.0,
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
    proposal_id: &str,
    operation_index: usize,
    local_repository: &crate::AtlasRepositoryIdentity,
    operation: &EpiphanyRepoModelMutationOperation,
) -> Result<(String, String)> {
    let (kind, id) = match operation {
        EpiphanyRepoModelMutationOperation::PutDomain { domain } => ("domain", domain.id.clone()),
        EpiphanyRepoModelMutationOperation::PutNode { node } => ("node", node.id.clone()),
        EpiphanyRepoModelMutationOperation::RetireNode { node_id } => ("node", node_id.clone()),
        EpiphanyRepoModelMutationOperation::PutEdge { edge } => ("edge", edge.id.clone()),
        EpiphanyRepoModelMutationOperation::RetireEdge { edge_id } => ("edge", edge_id.clone()),
        EpiphanyRepoModelMutationOperation::PutSummary { summary } => {
            ("summary", summary.id.clone())
        }
        EpiphanyRepoModelMutationOperation::PutFrontier { item } => ("frontier", item.id.clone()),
        EpiphanyRepoModelMutationOperation::CreateSurfaceOffer { .. } => (
            "surface_offer",
            derived_atlas_operation_id(
                proposal_id,
                operation_index,
                local_repository,
                "surface-offer",
            )
            .to_string(),
        ),
        EpiphanyRepoModelMutationOperation::DeprecateSurfaceOffer { surface_id, .. }
        | EpiphanyRepoModelMutationOperation::WithdrawSurfaceOffer { surface_id } => {
            ("surface_offer", surface_id.to_string())
        }
        EpiphanyRepoModelMutationOperation::CreateDependencyClaim { .. } => (
            "dependency_claim",
            derived_atlas_operation_id(
                proposal_id,
                operation_index,
                local_repository,
                "dependency-claim",
            )
            .to_string(),
        ),
        EpiphanyRepoModelMutationOperation::RetireDependencyClaim { claim_id } => {
            ("dependency_claim", claim_id.to_string())
        }
    };
    require_semantic_id(&id, "RepoModel operation")?;
    Ok((kind.into(), id))
}

fn derived_atlas_operation_id(
    proposal_id: &str,
    operation_index: usize,
    local_repository: &crate::AtlasRepositoryIdentity,
    kind: &str,
) -> uuid::Uuid {
    let name = format!(
        "{}/epiphany/modeling/{proposal_id}/{kind}/{operation_index}",
        local_repository.repository_uri
    );
    uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_URL, name.as_bytes())
}

fn load_local_surface_offer(
    cache: &CultCache,
    local_repository: &crate::AtlasRepositoryIdentity,
    surface_id: uuid::Uuid,
) -> Result<(crate::AtlasSurfaceOffer, CultCacheEnvelope)> {
    if surface_id.is_nil() {
        return Err(anyhow!("Atlas surface transition requires a non-nil UUID"));
    }
    let key = surface_id.to_string();
    let envelope = cache
        .get_envelope::<crate::AtlasSurfaceOffer>(&key)?
        .ok_or_else(|| anyhow!("Atlas surface transition targets a missing local offer"))?;
    let offer: crate::AtlasSurfaceOffer = rmp_serde::from_slice(&envelope.payload)?;
    offer.validate()?;
    if offer.provider != *local_repository || offer.surface_id != surface_id {
        return Err(anyhow!(
            "Atlas surface transition cannot write a foreign or substituted offer"
        ));
    }
    Ok((offer, envelope))
}

fn load_local_dependency_claim(
    cache: &CultCache,
    local_repository: &crate::AtlasRepositoryIdentity,
    claim_id: uuid::Uuid,
) -> Result<(crate::AtlasDependencyClaim, CultCacheEnvelope)> {
    if claim_id.is_nil() {
        return Err(anyhow!("Atlas claim transition requires a non-nil UUID"));
    }
    let key = claim_id.to_string();
    let envelope = cache
        .get_envelope::<crate::AtlasDependencyClaim>(&key)?
        .ok_or_else(|| anyhow!("Atlas claim transition targets a missing local claim"))?;
    let claim: crate::AtlasDependencyClaim = rmp_serde::from_slice(&envelope.payload)?;
    claim.validate()?;
    if claim.consumer != *local_repository || claim.claim_id != claim_id {
        return Err(anyhow!(
            "Atlas claim transition cannot write a foreign or substituted claim"
        ));
    }
    Ok((claim, envelope))
}

fn require_local_impact_scope(
    cache: &CultCache,
    local_repository: &crate::AtlasRepositoryIdentity,
    impact_scope: &crate::AtlasImpactScope,
    strong: &mut BTreeMap<(String, String), CultCacheEnvelope>,
) -> Result<()> {
    let crate::AtlasImpactScope::LocalSurfaces { surface_ids } = impact_scope else {
        return Ok(());
    };
    for surface_id in surface_ids {
        let (_, envelope) = load_local_surface_offer(cache, local_repository, *surface_id)?;
        insert_strong_envelope(strong, envelope)?;
    }
    Ok(())
}

fn resolve_atlas_body_evidence(
    manifest: &crate::RepositoryBodyManifest,
    source_refs: &[String],
) -> Result<Vec<crate::AtlasBodyEvidenceRef>> {
    if source_refs.is_empty() || !source_refs.windows(2).all(|pair| pair[0] < pair[1]) {
        return Err(anyhow!(
            "Atlas offer and claim source refs must be a non-empty strictly sorted set"
        ));
    }
    let entries = manifest
        .entries
        .iter()
        .map(|entry| (entry.path.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    source_refs
        .iter()
        .map(|path| {
            let entry = entries.get(path.as_str()).ok_or_else(|| {
                anyhow!("Atlas source ref {path:?} is absent from the exact Body manifest")
            })?;
            if entry.kind != "regular" {
                return Err(anyhow!(
                    "Atlas source ref {path:?} is not a regular repository Body file"
                ));
            }
            let evidence = crate::AtlasBodyEvidenceRef {
                path: path.clone(),
                raw_sha256: entry.raw_sha256.clone(),
            };
            evidence.validate()?;
            Ok(evidence)
        })
        .collect()
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

fn claim_obligations_for_frontier(
    nodes: &[EpiphanyMemoryNode],
    frontier: &[RepoFrontierItem],
) -> Vec<EpiphanyRepoModelClaimObligationsDocument> {
    let mut obligations = nodes
        .iter()
        .map(|node| (node.id.clone(), BTreeSet::<String>::new()))
        .collect::<BTreeMap<_, _>>();
    for item in frontier.iter().filter(|item| frontier_is_unresolved(item)) {
        for node_id in &item.target_claim_ids {
            obligations
                .entry(node_id.clone())
                .or_default()
                .insert(item.id.clone());
        }
    }
    obligations
        .into_iter()
        .map(
            |(node_id, unresolved_frontier_ids)| EpiphanyRepoModelClaimObligationsDocument {
                node_id,
                unresolved_frontier_ids: unresolved_frontier_ids.into_iter().collect(),
                active_challenge_ids: Vec::new(),
            },
        )
        .collect()
}

pub fn assemble_repo_model_view(store_path: impl AsRef<Path>) -> Result<EpiphanyRepoModelView> {
    let mut cache = runtime_spine_cache(store_path.as_ref())?;
    cache.pull_all_backing_stores()?;
    assemble_repo_model_view_from_cache(&cache)
}

pub(crate) fn assemble_repo_model_view_from_cache(
    cache: &CultCache,
) -> Result<EpiphanyRepoModelView> {
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
    let mut surface_offers = cache.get_all::<crate::AtlasSurfaceOffer>()?;
    let mut dependency_claims = cache.get_all::<crate::AtlasDependencyClaim>()?;
    let mut dependency_verifications = cache.get_all::<crate::AtlasDependencyVerification>()?;
    let mut dependency_impacts = cache.get_all::<crate::AtlasDependencyImpact>()?;
    domains.sort_by(|left, right| left.id.cmp(&right.id));
    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    edges.sort_by(|left, right| left.id.cmp(&right.id));
    summaries.sort_by(|left, right| left.id.cmp(&right.id));
    frontier.sort_by(|left, right| left.id.cmp(&right.id));
    lifecycle_receipts.sort_by(|left, right| left.id.cmp(&right.id));
    claim_obligations.sort_by(|left, right| left.node_id.cmp(&right.node_id));
    surface_offers.sort_by_key(|offer| offer.surface_id);
    dependency_claims.sort_by_key(|claim| claim.claim_id);
    dependency_verifications.sort_by_key(|verification| verification.claim_id);
    dependency_impacts.sort_by_key(|impact| impact.impact_id);

    let local_repository = crate::AtlasRepositoryIdentity::new(
        identity.swarm_id.clone(),
        identity.workspace_id.clone(),
    )?;
    if surface_offers
        .iter()
        .any(|offer| offer.provider != local_repository || offer.validate().is_err())
        || dependency_claims
            .iter()
            .any(|claim| claim.consumer != local_repository || claim.validate().is_err())
        || dependency_verifications.iter().any(|verification| {
            verification.consumer != local_repository || verification.validate().is_err()
        })
        || dependency_impacts
            .iter()
            .any(|impact| impact.consumer != local_repository || impact.validate().is_err())
    {
        return Err(anyhow!(
            "keyed RepoModel view contains a foreign or invalid local Atlas document"
        ));
    }

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
    let challenges = cache.get_all::<crate::RepoModelClaimChallenge>()?;
    for obligation in &claim_obligations {
        for challenge_id in &obligation.active_challenge_ids {
            let challenge = challenges
                .iter()
                .find(|challenge| challenge.challenge_id == *challenge_id)
                .ok_or_else(|| anyhow!("claim obligation names a missing challenge"))?;
            if challenge.target_claim_id != obligation.node_id {
                return Err(anyhow!("claim obligation challenge targets another node"));
            }
        }
    }
    for challenge in &challenges {
        let Some(node) = nodes
            .iter()
            .find(|node| node.id == challenge.target_claim_id)
        else {
            continue;
        };
        let node_sha256 = format!("{:x}", Sha256::digest(rmp_serde::to_vec_named(node)?));
        if node_sha256 == challenge.target_claim_sha256
            && !claim_obligations.iter().any(|obligation| {
                obligation.node_id == challenge.target_claim_id
                    && obligation
                        .active_challenge_ids
                        .contains(&challenge.challenge_id)
            })
        {
            return Err(anyhow!("current claim challenge lost its claim obligation"));
        }
    }

    let mut source_documents = cache
        .snapshot_envelopes()
        .into_iter()
        .filter(|entry| repo_model_write_key(entry).is_ok_and(|key| key.is_some()))
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
        surface_offers,
        dependency_claims,
        dependency_verifications,
        dependency_impacts,
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
    if obligations.len() != nodes.len() {
        return Err(anyhow!(
            "every RepoModel node requires one claim obligation"
        ));
    }
    for obligation in obligations {
        if !nodes.iter().any(|node| node.id == obligation.node_id) {
            return Err(anyhow!("RepoModel claim obligation names a missing node"));
        }
        if !obligation
            .unresolved_frontier_ids
            .windows(2)
            .all(|pair| pair[0] < pair[1])
            || !obligation
                .active_challenge_ids
                .windows(2)
                .all(|pair| pair[0] < pair[1])
            || obligation
                .active_challenge_ids
                .iter()
                .any(|id| id.trim().is_empty())
        {
            return Err(anyhow!("RepoModel claim obligation is not canonical"));
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
        EpiphanyMemoryEdgeKind, EpiphanyMemoryLifecycle, EpiphanyMemoryNodeKind,
        EpiphanyMemoryProfile, RepoFrontierStatus,
    };

    fn bind_test_body(
        store: &Path,
        swarm_id: &str,
        workspace_id: &str,
    ) -> Result<crate::RepositoryBodyObservationBasis> {
        crate::runtime_spine::tests::bind_test_runtime_swarm(store, swarm_id)?;
        crate::runtime_spine::tests::bind_test_repository_body(store, workspace_id)?;
        crate::observe_runtime_repository_body_basis(store)
    }

    fn make_proposal(
        proposal_id: &str,
        body: &crate::RepositoryBodyObservationBasis,
        operations: Vec<EpiphanyRepoModelMutationOperation>,
    ) -> Result<EpiphanyRepoModelMutationProposal> {
        EpiphanyRepoModelMutationProposal::new(
            proposal_id,
            format!("request-{proposal_id}"),
            format!("result-{proposal_id}"),
            vec![format!("evidence-{proposal_id}")],
            body.clone(),
            operations,
        )
    }

    #[test]
    fn fresh_repo_model_seed_commits_keyed_documents_and_refuses_divergent_replay() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("repo-model-seed.cc");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "repo-model-seed".into(),
                display_name: "RepoModel seed".into(),
                created_at: "2026-08-14T00:00:00Z".into(),
            },
        )?;
        let body = bind_test_body(&store, "swarm-seed", "workspace-seed")?;
        let domain = EpiphanyMemoryDomain {
            id: "domain-seed".into(),
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            title: "Seed domain".into(),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        };
        let node = EpiphanyMemoryNode {
            id: "node-seed".into(),
            domain_id: domain.id.clone(),
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            kind: EpiphanyMemoryNodeKind::Module,
            title: "Seed node".into(),
            claim: "Bootstrap is keyed".into(),
            question: "Can the aggregate enter?".into(),
            action_implication: "Refuse it".into(),
            source_hashes: vec!["anchor:missing".into()],
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        };
        let frontier = RepoFrontierItem {
            id: "frontier-seed".into(),
            migration_body: "repo".into(),
            question: "What remains?".into(),
            target_claim_ids: vec![node.id.clone()],
            repository_scope: vec!["epiphany-core".into()],
            recommended_next_organ: "Modeling".into(),
            status: RepoFrontierStatus::Active,
            ..Default::default()
        };
        let seed = EpiphanyRepoModelSeed::new(
            "seed-1",
            "graph-seed",
            "swarm-seed",
            "workspace-seed",
            body.body_binding_sha256.clone(),
            EpiphanyRepoModelSeedDocuments {
                domains: vec![domain],
                nodes: vec![node],
                edges: Vec::new(),
                summaries: Vec::new(),
                frontier: vec![frontier],
                lifecycle_receipts: Vec::new(),
            },
        )?;
        let before_invalid_seed = runtime_spine_cache(&store)?.snapshot_envelopes();
        let invalid_body_seed = EpiphanyRepoModelSeed::new(
            "seed-wrong-body",
            "graph-wrong-body",
            "swarm-seed",
            "workspace-seed",
            "sha256:not-the-runtime-body-binding",
            EpiphanyRepoModelSeedDocuments {
                domains: Vec::new(),
                nodes: Vec::new(),
                edges: Vec::new(),
                summaries: Vec::new(),
                frontier: Vec::new(),
                lifecycle_receipts: Vec::new(),
            },
        )?;
        assert!(
            initialize_keyed_repo_model(&store, &invalid_body_seed, "2026-08-14T00:00:00.500Z")
                .is_err()
        );
        assert_eq!(
            runtime_spine_cache(&store)?.snapshot_envelopes(),
            before_invalid_seed
        );
        let first = initialize_keyed_repo_model(&store, &seed, "2026-08-14T00:00:01Z")?;
        let replay = initialize_keyed_repo_model(&store, &seed, "2026-08-14T00:00:02Z")?;
        assert_eq!(first, replay);
        assert_eq!(first.claim_obligations[0].node_id, "node-seed");
        let invalid_routed_frontier = make_proposal(
            "invalid-routed-frontier",
            &body,
            vec![EpiphanyRepoModelMutationOperation::PutFrontier {
                item: RepoFrontierItem {
                    id: "frontier-invalid-route-scope".into(),
                    migration_body: "Create OX-CAPSTONE.md".into(),
                    question: "Can Planning authorize the intended output?".into(),
                    gap: "The model confused inspected evidence with future path authority.".into(),
                    target_claim_ids: vec!["node-seed".into()],
                    repository_scope: vec!["notes/source.md".into(), "OX-CAPSTONE.md".into()],
                    recommended_next_organ: "Imagination".into(),
                    status: RepoFrontierStatus::Active,
                    ..Default::default()
                },
            }],
        )?;
        let error = plan_repo_model_mutation(&store, &invalid_routed_frontier)
            .expect_err("noncanonical routed scope must refuse before RepoModel writes");
        assert!(error.to_string().contains("repository_scope"));
        assert!(
            runtime_spine_cache(&store)?
                .get::<EpiphanyRepoModelFrontierDocument>("frontier-invalid-route-scope")?
                .is_none()
        );
        let mut cache = runtime_spine_cache(&store)?;
        cache.pull_all_backing_stores()?;
        assert_eq!(
            cache.get::<EpiphanyRepoModelSeed>("seed-1")?,
            Some(seed.clone())
        );
        assert!(
            cache
                .get_all::<crate::EpiphanyMindCommitReceipt>()?
                .iter()
                .any(|receipt| receipt.invariant_owner == "Modeling.repo_model_seed")
        );
        assert_eq!(
            cache
                .get_all::<crate::MemorySemanticProjectionObligation>()?
                .into_iter()
                .filter(|obligation| obligation.partition == "modeling")
                .count(),
            1
        );
        assert!(
            cache
                .snapshot_envelopes()
                .iter()
                .all(|envelope| envelope.r#type != "epiphany.memory_graph")
        );

        let divergent = EpiphanyRepoModelSeed::new(
            "seed-1",
            "other-graph",
            "swarm-seed",
            "workspace-seed",
            body.body_binding_sha256,
            EpiphanyRepoModelSeedDocuments {
                domains: Vec::new(),
                nodes: Vec::new(),
                edges: Vec::new(),
                summaries: Vec::new(),
                frontier: Vec::new(),
                lifecycle_receipts: Vec::new(),
            },
        )?;
        assert!(initialize_keyed_repo_model(&store, &divergent, "2026-08-14T00:00:03Z").is_err());
        Ok(())
    }

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
        let body = bind_test_body(&store, "swarm-1", "workspace-1")?;
        let mut cache = runtime_spine_cache(&store)?;
        cache.put(
            REPO_MODEL_IDENTITY_KEY,
            &EpiphanyRepoModelIdentityDocument {
                schema_epoch: REPO_MODEL_SCHEMA_EPOCH.into(),
                graph_id: "graph-1".into(),
                runtime_id: "repo-model-view".into(),
                swarm_id: "swarm-1".into(),
                workspace_id: "workspace-1".into(),
                body_binding_sha256: body.body_binding_sha256.clone(),
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
            "node-1",
            &EpiphanyRepoModelClaimObligationsDocument {
                node_id: "node-1".into(),
                unresolved_frontier_ids: Vec::new(),
                active_challenge_ids: Vec::new(),
            },
        )?;
        let first = assemble_repo_model_view(&store)?;
        let second = assemble_repo_model_view(&store)?;
        assert_eq!(first, second);
        assert_eq!(first.nodes[0].id, "node-1");
        assert!(first.projection_digest.starts_with("sha256:"));
        let mind = crate::assemble_mind_view(&store)?;
        assert_eq!(mind.repo_model.as_ref(), Some(&first));

        let retire = make_proposal(
            "proposal-retire",
            &body,
            vec![EpiphanyRepoModelMutationOperation::RetireNode {
                node_id: "node-1".into(),
            }],
        )?;
        let retire_plan = plan_repo_model_mutation(&store, &retire)?;
        let add_frontier = make_proposal(
            "proposal-frontier",
            &body,
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
        let body = bind_test_body(&store, "swarm-1", "workspace-1")?;
        let mut cache = runtime_spine_cache(&store)?;
        cache.put(
            REPO_MODEL_IDENTITY_KEY,
            &EpiphanyRepoModelIdentityDocument {
                schema_epoch: REPO_MODEL_SCHEMA_EPOCH.into(),
                graph_id: "graph-atomic".into(),
                runtime_id: "repo-model-atomic-slice".into(),
                swarm_id: "swarm-1".into(),
                workspace_id: "workspace-1".into(),
                body_binding_sha256: body.body_binding_sha256.clone(),
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
        let proposal = make_proposal(
            "proposal-atomic-slice",
            &body,
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

        let retire = make_proposal(
            "proposal-retire-edge",
            &body,
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

        let duplicate = make_proposal(
            "proposal-duplicate-edge",
            &body,
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

    #[test]
    fn concurrent_distinct_nodes_merge_same_identity_conflicts_and_restart_replays() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("repo-model-concurrency.cc");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "repo-model-concurrency".into(),
                display_name: "RepoModel concurrency".into(),
                created_at: "2026-08-18T00:00:00Z".into(),
            },
        )?;
        let body = bind_test_body(&store, "swarm-concurrency", "workspace-concurrency")?;
        let domain = EpiphanyMemoryDomain {
            id: "domain-concurrency".into(),
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            title: "Concurrent domain".into(),
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        };
        initialize_keyed_repo_model(
            &store,
            &EpiphanyRepoModelSeed::new(
                "seed-concurrency",
                "graph-concurrency",
                "swarm-concurrency",
                "workspace-concurrency",
                body.body_binding_sha256.clone(),
                EpiphanyRepoModelSeedDocuments {
                    domains: vec![domain.clone()],
                    nodes: Vec::new(),
                    edges: Vec::new(),
                    summaries: Vec::new(),
                    frontier: Vec::new(),
                    lifecycle_receipts: Vec::new(),
                },
            )?,
            "2026-08-18T00:00:01Z",
        )?;
        let node = |id: &str, claim: &str| EpiphanyMemoryNode {
            id: id.into(),
            domain_id: domain.id.clone(),
            profile: EpiphanyMemoryProfile::RepoArchitecture,
            kind: EpiphanyMemoryNodeKind::Module,
            title: id.into(),
            claim: claim.into(),
            question: "Can this identity commit independently?".into(),
            action_implication: "Use exact-envelope CAS".into(),
            source_hashes: vec!["anchor:missing".into()],
            lifecycle: EpiphanyMemoryLifecycle::Accepted,
            ..Default::default()
        };
        let proposals = [
            make_proposal(
                "proposal-node-left",
                &body,
                vec![EpiphanyRepoModelMutationOperation::PutNode {
                    node: node("node-left", "Left is independently owned"),
                }],
            )?,
            make_proposal(
                "proposal-node-right",
                &body,
                vec![EpiphanyRepoModelMutationOperation::PutNode {
                    node: node("node-right", "Right is independently owned"),
                }],
            )?,
        ];
        let cache = runtime_spine_cache(&store)?;
        let prepared = proposals
            .iter()
            .map(|proposal| {
                Ok((
                    plan_repo_model_mutation(&store, proposal)?,
                    cache.prepare_entry(&proposal.proposal_id, proposal)?.0,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let outcomes = std::thread::scope(|scope| {
            prepared
                .into_iter()
                .enumerate()
                .map(|(index, (plan, provenance))| {
                    let barrier = barrier.clone();
                    let store = store.clone();
                    scope.spawn(move || {
                        barrier.wait();
                        crate::commit_operator_mind_mutation(
                            &store,
                            provenance,
                            "Modeling.repo_model_mutation",
                            plan.strong_reads,
                            plan.writes,
                            if index == 0 {
                                "2026-08-18T00:00:02Z"
                            } else {
                                "2026-08-18T00:00:03Z"
                            },
                        )
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|handle| handle.join().expect("concurrent node writer"))
                .collect::<Result<Vec<_>>>()
        })?;
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(outcome, crate::EpiphanyMindCommitOutcome::Committed(_)))
                .count(),
            2
        );
        let reopened = assemble_repo_model_view(&store)?;
        assert_eq!(
            reopened
                .nodes
                .iter()
                .map(|node| node.id.as_str())
                .collect::<Vec<_>>(),
            ["node-left", "node-right"]
        );
        let semantic = crate::runtime_modeling_semantic_projection_input(&store)?;
        assert_eq!(
            semantic.source_head().source_commit_id,
            reopened.projection_digest
        );
        assert_eq!(
            semantic.obligation().source_commit_id,
            reopened.projection_digest
        );
        assert!(
            semantic
                .authority
                .envelopes
                .iter()
                .any(|envelope| { envelope.r#type == crate::EpiphanyMindCommitReceipt::TYPE })
        );
        let mut reopened_cache = runtime_spine_cache(&store)?;
        reopened_cache.pull_all_backing_stores()?;
        assert_eq!(
            reopened_cache
                .get_all::<crate::MemorySemanticProjectionObligation>()?
                .into_iter()
                .filter(|obligation| obligation.partition == "modeling")
                .count(),
            2
        );
        let replayed_semantic = crate::runtime_modeling_semantic_projection_input(&store)?;
        assert_eq!(replayed_semantic.obligation(), semantic.obligation());

        let competing = [
            make_proposal(
                "proposal-collision-left",
                &body,
                vec![EpiphanyRepoModelMutationOperation::PutNode {
                    node: node("node-collision", "Left claim"),
                }],
            )?,
            make_proposal(
                "proposal-collision-right",
                &body,
                vec![EpiphanyRepoModelMutationOperation::PutNode {
                    node: node("node-collision", "Right claim"),
                }],
            )?,
        ];
        let prepared = competing
            .iter()
            .map(|proposal| {
                Ok((
                    plan_repo_model_mutation(&store, proposal)?,
                    cache.prepare_entry(&proposal.proposal_id, proposal)?.0,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let replay_inputs = prepared.clone();
        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
        let outcomes = std::thread::scope(|scope| {
            prepared
                .into_iter()
                .enumerate()
                .map(|(index, (plan, provenance))| {
                    let barrier = barrier.clone();
                    let store = store.clone();
                    scope.spawn(move || {
                        barrier.wait();
                        crate::commit_operator_mind_mutation(
                            &store,
                            provenance,
                            "Modeling.repo_model_mutation",
                            plan.strong_reads,
                            plan.writes,
                            if index == 0 {
                                "2026-08-18T00:00:04Z"
                            } else {
                                "2026-08-18T00:00:05Z"
                            },
                        )
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|handle| handle.join().expect("competing node writer"))
                .collect::<Result<Vec<_>>>()
        })?;
        let winner = outcomes
            .iter()
            .position(|outcome| matches!(outcome, crate::EpiphanyMindCommitOutcome::Committed(_)))
            .expect("one same-identity writer wins");
        assert_eq!(
            outcomes
                .iter()
                .filter(|outcome| matches!(
                    outcome,
                    crate::EpiphanyMindCommitOutcome::Conflict { .. }
                ))
                .count(),
            1
        );
        let original_receipt = match &outcomes[winner] {
            crate::EpiphanyMindCommitOutcome::Committed(receipt) => receipt.clone(),
            _ => unreachable!(),
        };
        let (plan, provenance) = replay_inputs[winner].clone();
        assert_eq!(
            crate::commit_operator_mind_mutation(
                &store,
                provenance,
                "Modeling.repo_model_mutation",
                plan.strong_reads,
                plan.writes,
                if winner == 0 {
                    "2026-08-18T00:00:04Z"
                } else {
                    "2026-08-18T00:00:05Z"
                },
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(original_receipt)
        );
        let restarted = assemble_repo_model_view(&store)?;
        assert_eq!(restarted.nodes.len(), 3);
        assert!(
            restarted
                .nodes
                .iter()
                .any(|node| node.id == "node-collision")
        );
        Ok(())
    }

    #[test]
    fn modeling_operations_own_local_atlas_offer_and_claim_lifecycles() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let store = temp.path().join("repo-model-atlas.cc");
        initialize_runtime_spine(
            &store,
            RuntimeSpineInitOptions {
                runtime_id: "repo-model-atlas".into(),
                display_name: "RepoModel Atlas".into(),
                created_at: "2026-08-14T00:00:00Z".into(),
            },
        )?;
        let body = bind_test_body(&store, "gamecult-local", "odin")?;
        let mut cache = runtime_spine_cache(&store)?;
        cache.put(
            REPO_MODEL_IDENTITY_KEY,
            &EpiphanyRepoModelIdentityDocument {
                schema_epoch: REPO_MODEL_SCHEMA_EPOCH.into(),
                graph_id: "graph-atlas".into(),
                runtime_id: "repo-model-atlas".into(),
                swarm_id: "gamecult-local".into(),
                workspace_id: "odin".into(),
                body_binding_sha256: body.body_binding_sha256.clone(),
            },
        )?;

        let offer_proposal = make_proposal(
            "modeling-job-create-offer",
            &body,
            vec![EpiphanyRepoModelMutationOperation::CreateSurfaceOffer {
                label: "Odin provider catalog".into(),
                contract: crate::AtlasContractDescriptor::ExactSchema {
                    contract_id: "odin-provider-catalog".into(),
                    schema_id: "cultmesh://odin/rendezvous/provider-catalog".into(),
                },
                source_refs: vec!["body-seed.txt".into()],
            }],
        )?;
        let offer_plan = plan_repo_model_mutation(&store, &offer_proposal)?;
        let offer_provenance = cache
            .prepare_entry(&offer_proposal.proposal_id, &offer_proposal)?
            .0;
        assert!(matches!(
            crate::commit_operator_mind_mutation(
                &store,
                offer_provenance,
                "Modeling.repo_model_mutation",
                offer_plan.strong_reads,
                offer_plan.writes,
                "2026-08-14T00:00:01Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));
        let offer_view = assemble_repo_model_view(&store)?;
        let offer_id = offer_view.surface_offers[0].surface_id;
        assert_eq!(offer_view.surface_offers[0].provider.workspace_id, "odin");
        let manifest = crate::authenticated_repository_body_manifest(&store, &body)?;
        assert_eq!(
            offer_view.surface_offers[0].body_evidence,
            [crate::AtlasBodyEvidenceRef {
                path: manifest.entries[0].path.clone(),
                raw_sha256: manifest.entries[0].raw_sha256.clone(),
            }]
        );
        assert!(offer_view.source_documents.iter().any(|source| {
            source.document_type == crate::AtlasSurfaceOffer::TYPE
                && source.document_key == offer_id.to_string()
        }));
        offer_view.reasoning_basis().validate_current(&store)?;

        let eve = crate::AtlasRepositoryIdentity::new("gamecult-local", "eve")?;
        let eve_surface = uuid::Uuid::new_v4();
        let claim_proposal = make_proposal(
            "modeling-job-create-claim",
            &body,
            vec![EpiphanyRepoModelMutationOperation::CreateDependencyClaim {
                label: "Odin consumes Eve surfaces".into(),
                target: crate::AtlasDependencyTarget::Exact {
                    provider: eve,
                    surface_id: eve_surface,
                    requirement: crate::AtlasContractRequirement::ExactSchema {
                        contract_id: "eve-surface".into(),
                        schema_id: "gamecult.eve.surface.v1".into(),
                    },
                },
                entanglement_kind: crate::AtlasEntanglementKind::SchemaProtocol,
                failure_semantics: crate::AtlasFailureSemantics::FailClosed,
                impact_scope: crate::AtlasImpactScope::LocalSurfaces {
                    surface_ids: vec![offer_id],
                },
                source_refs: vec!["body-seed.txt".into()],
            }],
        )?;
        let claim_plan = plan_repo_model_mutation(&store, &claim_proposal)?;
        let stale_claim_plan = claim_plan.clone();
        let claim_provenance = cache
            .prepare_entry(&claim_proposal.proposal_id, &claim_proposal)?
            .0;
        assert!(matches!(
            crate::commit_operator_mind_mutation(
                &store,
                claim_provenance,
                "Modeling.repo_model_mutation",
                claim_plan.strong_reads,
                claim_plan.writes,
                "2026-08-14T00:00:02Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));
        let claim_id = assemble_repo_model_view(&store)?.dependency_claims[0].claim_id;

        let withdraw = make_proposal(
            "modeling-job-withdraw-offer",
            &body,
            vec![EpiphanyRepoModelMutationOperation::WithdrawSurfaceOffer {
                surface_id: offer_id,
            }],
        )?;
        let withdraw_plan = plan_repo_model_mutation(&store, &withdraw)?;
        let withdraw_provenance = cache.prepare_entry(&withdraw.proposal_id, &withdraw)?.0;
        assert!(matches!(
            crate::commit_operator_mind_mutation(
                &store,
                withdraw_provenance,
                "Modeling.repo_model_mutation",
                withdraw_plan.strong_reads,
                withdraw_plan.writes,
                "2026-08-14T00:00:03Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));
        let stale_provenance = cache
            .prepare_entry("modeling-job-stale-claim", &claim_proposal)?
            .0;
        assert!(matches!(
            crate::commit_operator_mind_mutation(
                &store,
                stale_provenance,
                "Modeling.repo_model_mutation",
                stale_claim_plan.strong_reads,
                stale_claim_plan.writes,
                "2026-08-14T00:00:04Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Conflict { .. }
        ));

        let retire = make_proposal(
            "modeling-job-retire-claim",
            &body,
            vec![EpiphanyRepoModelMutationOperation::RetireDependencyClaim { claim_id }],
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
                "2026-08-14T00:00:05Z",
            )?,
            crate::EpiphanyMindCommitOutcome::Committed(_)
        ));
        let final_view = assemble_repo_model_view(&store)?;
        assert_eq!(
            final_view.surface_offers[0].lifecycle,
            crate::AtlasOfferLifecycle::Withdrawn
        );
        assert_eq!(
            final_view.dependency_claims[0].lifecycle,
            crate::AtlasClaimLifecycle::Retired
        );
        std::fs::write(
            store.with_extension("odin.body-repo").join("body-seed.txt"),
            b"changed Atlas evidence",
        )?;
        crate::observe_runtime_repository_body_basis(&store)?;
        assert!(plan_repo_model_mutation(&store, &offer_proposal).is_err());
        Ok(())
    }
}
