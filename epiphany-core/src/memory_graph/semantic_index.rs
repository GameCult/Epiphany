use super::{
    EpiphanyMemoryContextPacket, EpiphanyMemoryContextQuery, EpiphanyMemoryGraphSnapshot,
    SEMANTIC_PROJECTION_SCHEMA_VERSION, SemanticPartition, SemanticProjectionCandidate,
    SemanticProjectionDocument, derive_semantic_projection,
    plan_memory_graph_context_cut_for_partition, resolve_semantic_candidate,
};
use crate::semantic_backend::{
    CollectionCompatibility, OllamaConfig, OllamaEmbedder, QdrantBackend, QdrantConfig,
    SemanticPoint,
};
use anyhow::{Result, anyhow};
use cultcache_rs::{CultCache, DatabaseEntry, SingleFileMessagePackBackingStore};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::env;
use std::path::Path;

pub const MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION: &str =
    "gamecult.epiphany.memory_semantic_index_receipt.v0";
pub const MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION: &str =
    "gamecult.epiphany.memory_semantic_projection_obligation.v0";
pub const MEMORY_SEMANTIC_PROJECTION_ATTEMPT_SCHEMA_VERSION: &str =
    "gamecult.epiphany.memory_semantic_projection_attempt.v0";
const MODELING_COLLECTION_DEFAULT: &str = "epiphany_modeling_v1";
const MIND_COLLECTION_DEFAULT: &str = "epiphany_mind_v1";
const QUERY_LIMIT_MAX: usize = 64;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MemorySemanticIndexConfig {
    pub qdrant_url: String,
    pub qdrant_api_key: Option<String>,
    pub qdrant_timeout_ms: u64,
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub ollama_timeout_ms: u64,
    pub embedding_provider_id: String,
    pub modeling_collection: String,
    pub mind_collection: String,
    pub modeling_query_instruction: String,
    pub mind_query_instruction: String,
}

impl MemorySemanticIndexConfig {
    pub fn from_env() -> Self {
        Self {
            qdrant_url: env_value("EPIPHANY_QDRANT_URL", "http://127.0.0.1:6333"),
            qdrant_api_key: env::var("EPIPHANY_QDRANT_API_KEY")
                .ok()
                .filter(|value| !value.trim().is_empty()),
            qdrant_timeout_ms: env_u64("EPIPHANY_QDRANT_TIMEOUT_MS", 30_000),
            ollama_base_url: env_value("EPIPHANY_OLLAMA_BASE_URL", "http://127.0.0.1:11434"),
            ollama_model: env_value("EPIPHANY_OLLAMA_MODEL", "qwen3-embedding:0.6b"),
            ollama_timeout_ms: env_u64("EPIPHANY_OLLAMA_TIMEOUT_MS", 30_000),
            embedding_provider_id: env_value(
                "EPIPHANY_EMBEDDING_PROVIDER_ID",
                "gamecult-ollama-embedding",
            ),
            modeling_collection: env_value(
                "EPIPHANY_MODELING_QDRANT_COLLECTION",
                MODELING_COLLECTION_DEFAULT,
            ),
            mind_collection: env_value(
                "EPIPHANY_MIND_QDRANT_COLLECTION",
                MIND_COLLECTION_DEFAULT,
            ),
            modeling_query_instruction: "Given a repository architecture, dataflow, ownership, invariant, or frontier question, rank the canonical Modeling documents that answer it.".to_string(),
            mind_query_instruction: "Given a swarm memory, doctrine, evidence, decision, relationship, or rehydration question, rank the canonical Mind documents that answer it.".to_string(),
        }
    }

    fn collection(&self, partition: SemanticPartition) -> &str {
        match partition {
            SemanticPartition::Mind => &self.mind_collection,
            SemanticPartition::Modeling => &self.modeling_collection,
        }
    }

    fn query_instruction(&self, partition: SemanticPartition) -> &str {
        match partition {
            SemanticPartition::Mind => &self.mind_query_instruction,
            SemanticPartition::Modeling => &self.modeling_query_instruction,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MemorySemanticPointPayload {
    point_id: String,
    swarm_id: String,
    partition: SemanticPartition,
    canonical_locator: String,
    canonical_type: String,
    canonical_key: String,
    canonical_document_id: String,
    canonical_schema_version: String,
    graph_id: String,
    indexed_model_revision: u64,
    indexed_model_hash: String,
    indexed_canonical_content_hash: String,
    projection_schema_version: String,
    embedding_provider_id: String,
    embedding_model: String,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.memory_semantic_projection_obligation",
    schema = "MemorySemanticProjectionObligation"
)]
pub struct MemorySemanticProjectionObligation {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub obligation_id: String,
    #[cultcache(key = 2)]
    pub swarm_id: String,
    #[cultcache(key = 3)]
    pub partition: String,
    #[cultcache(key = 4)]
    pub canonical_source_id: String,
    #[cultcache(key = 5)]
    pub source_commit_id: String,
    #[cultcache(key = 6)]
    pub graph_id: String,
    #[cultcache(key = 7)]
    pub source_generation: u64,
    #[cultcache(key = 8)]
    pub source_model_hash: String,
    #[cultcache(key = 9)]
    pub canonical_content_set_hash: String,
    #[cultcache(key = 10)]
    pub projection_schema_version: String,
    #[cultcache(key = 11)]
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.memory_semantic_projection_attempt",
    schema = "MemorySemanticProjectionAttempt"
)]
pub struct MemorySemanticProjectionAttempt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub attempt_id: String,
    #[cultcache(key = 2)]
    pub obligation_id: String,
    #[cultcache(key = 3)]
    pub started_at: String,
    #[cultcache(key = 4)]
    pub completed_at: Option<String>,
    #[cultcache(key = 5)]
    pub status: String,
    #[cultcache(key = 6)]
    pub error: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MemorySemanticProjectionHealthStatus {
    Pending,
    Failed,
    Stale,
    Ready,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemorySemanticProjectionSourceHead {
    pub swarm_id: String,
    pub partition: String,
    pub canonical_source_id: String,
    pub source_commit_id: String,
    pub graph_id: String,
    pub source_generation: u64,
    pub source_model_hash: String,
    pub canonical_content_set_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MemorySemanticProjectionHealth {
    pub status: MemorySemanticProjectionHealthStatus,
    pub obligation_id: String,
    pub receipt_id: Option<String>,
    pub latest_attempt_id: Option<String>,
    pub latest_error: Option<String>,
    pub query_eligible: bool,
}

#[derive(Clone, Debug, PartialEq, DatabaseEntry)]
#[cultcache(
    type = "gamecult.epiphany.memory_semantic_index_receipt",
    schema = "MemorySemanticIndexReceipt"
)]
pub struct MemorySemanticIndexReceipt {
    #[cultcache(key = 0)]
    pub schema_version: String,
    #[cultcache(key = 1)]
    pub receipt_id: String,
    #[cultcache(key = 2)]
    pub swarm_id: String,
    #[cultcache(key = 3)]
    pub partition: String,
    #[cultcache(key = 4)]
    pub collection_name: String,
    #[cultcache(key = 5)]
    pub graph_id: String,
    #[cultcache(key = 6)]
    pub model_revision: u64,
    #[cultcache(key = 7)]
    pub model_hash: String,
    #[cultcache(key = 8)]
    pub embedding_provider_id: String,
    #[cultcache(key = 9)]
    pub embedding_model: String,
    #[cultcache(key = 10)]
    pub vector_dimensions: u32,
    #[cultcache(key = 11)]
    pub indexed_document_count: u32,
    #[cultcache(key = 12)]
    pub deleted_document_count: u32,
    #[cultcache(key = 13)]
    pub canonical_content_set_hash: String,
    #[cultcache(key = 14)]
    pub indexed_at: String,
    #[cultcache(key = 15)]
    pub status: String,
    #[cultcache(key = 16)]
    pub obligation_id: String,
    #[cultcache(key = 17)]
    pub canonical_source_id: String,
    #[cultcache(key = 18)]
    pub source_commit_id: String,
    #[cultcache(key = 19)]
    pub source_generation: u64,
    #[cultcache(key = 20)]
    pub projection_schema_version: String,
}

pub fn validate_memory_semantic_projection_obligation(
    obligation: &MemorySemanticProjectionObligation,
) -> Result<()> {
    if obligation.schema_version != MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION {
        return Err(anyhow!("unsupported semantic projection obligation schema"));
    }
    for (label, value) in [
        ("obligation_id", obligation.obligation_id.as_str()),
        ("swarm_id", obligation.swarm_id.as_str()),
        ("partition", obligation.partition.as_str()),
        (
            "canonical_source_id",
            obligation.canonical_source_id.as_str(),
        ),
        ("source_commit_id", obligation.source_commit_id.as_str()),
        ("graph_id", obligation.graph_id.as_str()),
        ("source_model_hash", obligation.source_model_hash.as_str()),
        (
            "canonical_content_set_hash",
            obligation.canonical_content_set_hash.as_str(),
        ),
        ("created_at", obligation.created_at.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(anyhow!("semantic projection obligation missing {label}"));
        }
    }
    if !matches!(obligation.partition.as_str(), "mind" | "modeling") {
        return Err(anyhow!(
            "semantic projection obligation has invalid partition"
        ));
    }
    if obligation.projection_schema_version != SEMANTIC_PROJECTION_SCHEMA_VERSION {
        return Err(anyhow!("semantic projection obligation schema mismatch"));
    }
    Ok(())
}

pub fn validate_memory_semantic_projection_attempt(
    attempt: &MemorySemanticProjectionAttempt,
) -> Result<()> {
    if attempt.schema_version != MEMORY_SEMANTIC_PROJECTION_ATTEMPT_SCHEMA_VERSION {
        return Err(anyhow!("unsupported semantic projection attempt schema"));
    }
    if attempt.attempt_id.trim().is_empty()
        || attempt.obligation_id.trim().is_empty()
        || attempt.started_at.trim().is_empty()
    {
        return Err(anyhow!(
            "semantic projection attempt is missing identity or time"
        ));
    }
    match attempt.status.as_str() {
        "running" if attempt.completed_at.is_none() && attempt.error.is_none() => Ok(()),
        "failed" if attempt.completed_at.is_some() && attempt.error.is_some() => Ok(()),
        _ => Err(anyhow!(
            "semantic projection attempt status does not match its terminal fields"
        )),
    }
}

pub fn derive_memory_semantic_projection_health(
    obligation: &MemorySemanticProjectionObligation,
    current: &MemorySemanticProjectionSourceHead,
    attempts: &[MemorySemanticProjectionAttempt],
    receipts: &[MemorySemanticIndexReceipt],
) -> Result<MemorySemanticProjectionHealth> {
    validate_memory_semantic_projection_obligation(obligation)?;
    for attempt in attempts
        .iter()
        .filter(|attempt| attempt.obligation_id == obligation.obligation_id)
    {
        validate_memory_semantic_projection_attempt(attempt)?;
    }
    let stale = !obligation_matches_source(obligation, current);
    let mut matching_receipts = receipts
        .iter()
        .filter(|receipt| receipt_matches_obligation(receipt, obligation))
        .collect::<Vec<_>>();
    matching_receipts.sort_by(|left, right| left.indexed_at.cmp(&right.indexed_at));
    let receipt = matching_receipts.last().copied();
    let mut matching_attempts = attempts
        .iter()
        .filter(|attempt| attempt.obligation_id == obligation.obligation_id)
        .collect::<Vec<_>>();
    matching_attempts.sort_by(|left, right| {
        left.completed_at
            .as_deref()
            .unwrap_or(left.started_at.as_str())
            .cmp(
                right
                    .completed_at
                    .as_deref()
                    .unwrap_or(right.started_at.as_str()),
            )
    });
    let latest_attempt = matching_attempts.last().copied();
    let status = if stale {
        MemorySemanticProjectionHealthStatus::Stale
    } else if receipt.is_some() {
        MemorySemanticProjectionHealthStatus::Ready
    } else if latest_attempt.is_some_and(|attempt| attempt.status == "failed") {
        MemorySemanticProjectionHealthStatus::Failed
    } else {
        MemorySemanticProjectionHealthStatus::Pending
    };
    Ok(MemorySemanticProjectionHealth {
        status,
        obligation_id: obligation.obligation_id.clone(),
        receipt_id: receipt.map(|receipt| receipt.receipt_id.clone()),
        latest_attempt_id: latest_attempt.map(|attempt| attempt.attempt_id.clone()),
        latest_error: latest_attempt.and_then(|attempt| attempt.error.clone()),
        query_eligible: status == MemorySemanticProjectionHealthStatus::Ready,
    })
}

pub fn memory_semantic_projection_query_eligible(
    obligation: &MemorySemanticProjectionObligation,
    current: &MemorySemanticProjectionSourceHead,
    receipt: &MemorySemanticIndexReceipt,
) -> bool {
    validate_memory_semantic_projection_obligation(obligation).is_ok()
        && obligation_matches_source(obligation, current)
        && receipt_matches_obligation(receipt, obligation)
}

pub fn bind_memory_semantic_index_receipt(
    mut receipt: MemorySemanticIndexReceipt,
    obligation: &MemorySemanticProjectionObligation,
) -> Result<MemorySemanticIndexReceipt> {
    validate_memory_semantic_projection_obligation(obligation)?;
    if receipt.swarm_id != obligation.swarm_id
        || receipt.partition != obligation.partition
        || receipt.graph_id != obligation.graph_id
        || receipt.model_revision != obligation.source_generation
        || receipt.model_hash != obligation.source_model_hash
        || receipt.canonical_content_set_hash != obligation.canonical_content_set_hash
        || receipt.status != "ready"
    {
        return Err(anyhow!(
            "semantic index result does not match the exact projection obligation"
        ));
    }
    receipt.obligation_id = obligation.obligation_id.clone();
    receipt.canonical_source_id = obligation.canonical_source_id.clone();
    receipt.source_commit_id = obligation.source_commit_id.clone();
    receipt.source_generation = obligation.source_generation;
    receipt.projection_schema_version = obligation.projection_schema_version.clone();
    Ok(receipt)
}

fn obligation_matches_source(
    obligation: &MemorySemanticProjectionObligation,
    current: &MemorySemanticProjectionSourceHead,
) -> bool {
    obligation.swarm_id == current.swarm_id
        && obligation.partition == current.partition
        && obligation.canonical_source_id == current.canonical_source_id
        && obligation.source_commit_id == current.source_commit_id
        && obligation.graph_id == current.graph_id
        && obligation.source_generation == current.source_generation
        && obligation.source_model_hash == current.source_model_hash
        && obligation.canonical_content_set_hash == current.canonical_content_set_hash
}

fn receipt_matches_obligation(
    receipt: &MemorySemanticIndexReceipt,
    obligation: &MemorySemanticProjectionObligation,
) -> bool {
    receipt.schema_version == MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION
        && receipt.status == "ready"
        && receipt.obligation_id == obligation.obligation_id
        && receipt.swarm_id == obligation.swarm_id
        && receipt.partition == obligation.partition
        && receipt.canonical_source_id == obligation.canonical_source_id
        && receipt.source_commit_id == obligation.source_commit_id
        && receipt.graph_id == obligation.graph_id
        && receipt.source_generation == obligation.source_generation
        && receipt.model_revision == obligation.source_generation
        && receipt.model_hash == obligation.source_model_hash
        && receipt.canonical_content_set_hash == obligation.canonical_content_set_hash
        && receipt.projection_schema_version == obligation.projection_schema_version
}

pub fn index_memory_semantic_partition(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    swarm_id: &str,
    partition: SemanticPartition,
    indexed_at: &str,
    config: &MemorySemanticIndexConfig,
) -> Result<MemorySemanticIndexReceipt> {
    let documents = derive_semantic_projection(swarm_id, snapshot)?
        .into_iter()
        .filter(|document| document.partition == partition)
        .collect::<Vec<_>>();
    if documents.is_empty() {
        return Err(anyhow!(
            "semantic projection partition has no live documents"
        ));
    }
    let embedder = embedder(config, partition)?;
    let texts = documents
        .iter()
        .map(|document| document.projection_text.clone())
        .collect::<Vec<_>>();
    let embeddings = embedder.embed_documents(&texts)?;
    let vector_size = embeddings
        .first()
        .map(Vec::len)
        .ok_or_else(|| anyhow!("semantic projection produced no embeddings"))?;
    let backend = qdrant(config)?;
    let collection = config.collection(partition);
    let compatibility = compatibility(config, partition, vector_size);
    if backend.collection_exists(collection)? {
        let actual = backend.collection_compatibility(collection)?;
        if actual != compatibility {
            return Err(anyhow!(
                "Qdrant collection {collection} is incompatible: actual {actual:?}, expected {compatibility:?}"
            ));
        }
    } else {
        backend.create_collection(collection, &compatibility)?;
    }
    let existing_ids = backend.point_ids_for_scope(
        collection,
        &[
            ("swarmId", swarm_id),
            ("partition", partition_name(partition)),
        ],
    )?;
    let points = documents
        .iter()
        .zip(embeddings)
        .map(|(document, vector)| SemanticPoint {
            id: document.point_id.clone(),
            vector,
            payload: point_payload(document, config),
        })
        .collect::<Vec<_>>();
    backend.upsert_points(collection, &points)?;
    let live_ids = documents
        .iter()
        .map(|document| document.point_id.as_str())
        .collect::<HashSet<_>>();
    let deleted_ids = existing_ids
        .into_iter()
        .filter(|id| !live_ids.contains(id.as_str()))
        .collect::<Vec<_>>();
    backend.delete_points(collection, &deleted_ids)?;
    let content_set_hash = canonical_content_set_hash(&documents);
    let receipt_id = format!(
        "memory-semantic-index-{}-{}-{}-{}",
        partition_name(partition),
        snapshot.model_revision,
        &content_set_hash[..16],
        &format!("{:x}", Sha256::digest(indexed_at.as_bytes()))[..12]
    );
    Ok(MemorySemanticIndexReceipt {
        schema_version: MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION.to_string(),
        receipt_id,
        swarm_id: swarm_id.to_string(),
        partition: partition_name(partition).to_string(),
        collection_name: collection.to_string(),
        graph_id: snapshot.graph_id.clone(),
        model_revision: snapshot.model_revision,
        model_hash: documents[0].model_hash.clone(),
        embedding_provider_id: config.embedding_provider_id.clone(),
        embedding_model: config.ollama_model.clone(),
        vector_dimensions: vector_size as u32,
        indexed_document_count: documents.len() as u32,
        deleted_document_count: deleted_ids.len() as u32,
        canonical_content_set_hash: content_set_hash,
        indexed_at: indexed_at.to_string(),
        status: "ready".to_string(),
        obligation_id: String::new(),
        canonical_source_id: snapshot.graph_id.clone(),
        source_commit_id: String::new(),
        source_generation: snapshot.model_revision,
        projection_schema_version: SEMANTIC_PROJECTION_SCHEMA_VERSION.to_string(),
    })
}

pub fn persist_memory_semantic_index_receipt(
    store_path: impl AsRef<Path>,
    receipt: &MemorySemanticIndexReceipt,
) -> Result<()> {
    let mut cache = CultCache::new();
    cache.register_entry_type::<MemorySemanticIndexReceipt>()?;
    cache.add_generic_backing_store(SingleFileMessagePackBackingStore::new(store_path.as_ref()));
    cache.pull_all_backing_stores()?;
    cache.put(&receipt.receipt_id, receipt)?;
    Ok(())
}

pub fn semantic_memory_context(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    swarm_id: &str,
    partition: SemanticPartition,
    query: &EpiphanyMemoryContextQuery,
    config: &MemorySemanticIndexConfig,
) -> EpiphanyMemoryContextPacket {
    match try_semantic_memory_context(snapshot, swarm_id, partition, query, config) {
        Ok(packet) => packet,
        Err(error) => {
            let mut packet =
                plan_memory_graph_context_cut_for_partition(snapshot, query, &[], partition);
            packet.warnings.push(format!(
                "semantic projection unavailable; used canonical BM25 fallback: {error}"
            ));
            packet
        }
    }
}

fn try_semantic_memory_context(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    swarm_id: &str,
    partition: SemanticPartition,
    query: &EpiphanyMemoryContextQuery,
    config: &MemorySemanticIndexConfig,
) -> Result<EpiphanyMemoryContextPacket> {
    let text = query
        .text
        .as_deref()
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| anyhow!("semantic context requires query text"))?;
    let documents = derive_semantic_projection(swarm_id, snapshot)?;
    let backend = qdrant(config)?;
    let collection = config.collection(partition);
    if !backend.collection_exists(collection)? {
        return Err(anyhow!("semantic collection {collection} is missing"));
    }
    let embedder = embedder(config, partition)?;
    let vector = embedder.embed_query(text)?;
    let expected = compatibility(config, partition, vector.len());
    let actual = backend.collection_compatibility(collection)?;
    if actual != expected {
        return Err(anyhow!("semantic collection compatibility mismatch"));
    }
    let limit = query.budget.unwrap_or(12).clamp(1, QUERY_LIMIT_MAX as u32) as usize;
    let ranked = backend.query_points_for_scope::<MemorySemanticPointPayload>(
        collection,
        &vector,
        limit,
        &[
            ("swarmId", swarm_id),
            ("partition", partition_name(partition)),
        ],
    )?;
    let mut ranked_ids = Vec::new();
    for hit in ranked {
        let payload = hit
            .payload
            .ok_or_else(|| anyhow!("semantic candidate omitted its typed locator payload"))?;
        if payload.swarm_id != swarm_id
            || payload.partition != partition
            || payload.embedding_provider_id != config.embedding_provider_id
            || payload.embedding_model != config.ollama_model
            || payload.projection_schema_version != SEMANTIC_PROJECTION_SCHEMA_VERSION
        {
            continue;
        }
        let candidate = SemanticProjectionCandidate {
            point_id: payload.point_id,
            canonical: super::SemanticCanonicalLocator {
                locator: payload.canonical_locator,
                canonical_type: payload.canonical_type,
                canonical_key: payload.canonical_key,
                canonical_document_id: payload.canonical_document_id,
            },
            partition: payload.partition,
            score: hit.score,
            indexed_model_revision: payload.indexed_model_revision,
            indexed_model_hash: payload.indexed_model_hash,
            indexed_canonical_content_hash: payload.indexed_canonical_content_hash,
        };
        if let Ok(document) = resolve_semantic_candidate(partition, &candidate, &documents) {
            ranked_ids.push(document.canonical.canonical_document_id.clone());
        }
    }
    let mut packet =
        plan_memory_graph_context_cut_for_partition(snapshot, query, &ranked_ids, partition);
    packet.warnings.push(format!(
        "semantic projection ranked {} canonical {} candidates; all payload text was ignored",
        ranked_ids.len(),
        partition_name(partition)
    ));
    Ok(packet)
}

fn point_payload(
    document: &SemanticProjectionDocument,
    config: &MemorySemanticIndexConfig,
) -> MemorySemanticPointPayload {
    MemorySemanticPointPayload {
        point_id: document.point_id.clone(),
        swarm_id: document.swarm_id.clone(),
        partition: document.partition,
        canonical_locator: document.canonical.locator.clone(),
        canonical_type: document.canonical.canonical_type.clone(),
        canonical_key: document.canonical.canonical_key.clone(),
        canonical_document_id: document.canonical.canonical_document_id.clone(),
        canonical_schema_version: document.canonical_schema_version.clone(),
        graph_id: document.graph_id.clone(),
        indexed_model_revision: document.model_revision,
        indexed_model_hash: document.model_hash.clone(),
        indexed_canonical_content_hash: document.canonical_content_hash.clone(),
        projection_schema_version: document.projection_schema_version.clone(),
        embedding_provider_id: config.embedding_provider_id.clone(),
        embedding_model: config.ollama_model.clone(),
    }
}

fn qdrant(config: &MemorySemanticIndexConfig) -> Result<QdrantBackend> {
    QdrantBackend::new(QdrantConfig {
        url: config.qdrant_url.clone(),
        api_key: config.qdrant_api_key.clone(),
        timeout_ms: config.qdrant_timeout_ms,
    })
}

fn embedder(
    config: &MemorySemanticIndexConfig,
    partition: SemanticPartition,
) -> Result<OllamaEmbedder> {
    OllamaEmbedder::new(OllamaConfig {
        base_url: config.ollama_base_url.clone(),
        model: config.ollama_model.clone(),
        timeout_ms: config.ollama_timeout_ms,
        query_instruction: config.query_instruction(partition).to_string(),
    })
}

fn compatibility(
    config: &MemorySemanticIndexConfig,
    partition: SemanticPartition,
    vector_size: usize,
) -> CollectionCompatibility {
    CollectionCompatibility {
        managed_by: "epiphany".to_string(),
        corpus_kind: partition_name(partition).to_string(),
        schema_version: 1,
        projection_version: SEMANTIC_PROJECTION_SCHEMA_VERSION.to_string(),
        embedding_provider_id: config.embedding_provider_id.clone(),
        embedding_model: config.ollama_model.clone(),
        vector_size,
    }
}

fn partition_name(partition: SemanticPartition) -> &'static str {
    match partition {
        SemanticPartition::Mind => "mind",
        SemanticPartition::Modeling => "modeling",
    }
}

fn canonical_content_set_hash(documents: &[SemanticProjectionDocument]) -> String {
    let mut values = documents
        .iter()
        .map(|document| format!("{}:{}", document.point_id, document.canonical_content_hash))
        .collect::<Vec<_>>();
    values.sort();
    format!("{:x}", Sha256::digest(values.join("\n").as_bytes()))
}

fn env_value(name: &str, default: &str) -> String {
    env::var(name)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| default.to_string())
}

fn env_u64(name: &str, default: u64) -> u64 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_graph::{
        EpiphanyMemoryDomain, EpiphanyMemoryLifecycle, EpiphanyMemoryNode, EpiphanyMemoryNodeKind,
        EpiphanyMemoryProfile, MEMORY_GRAPH_SCHEMA_VERSION, memory_graph_model_hash,
    };

    fn snapshot() -> EpiphanyMemoryGraphSnapshot {
        let mut snapshot = EpiphanyMemoryGraphSnapshot {
            schema_version: Some(MEMORY_GRAPH_SCHEMA_VERSION.to_string()),
            graph_id: "semantic-fallback".to_string(),
            model_revision: 3,
            domains: vec![EpiphanyMemoryDomain {
                id: "repo".to_string(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                title: "Repository".to_string(),
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                ..Default::default()
            }],
            nodes: vec![EpiphanyMemoryNode {
                id: "canonical-node".to_string(),
                domain_id: "repo".to_string(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                kind: EpiphanyMemoryNodeKind::Module,
                title: "Semantic projection".to_string(),
                claim: "Canonical graph remains correct without Qdrant.".to_string(),
                question: "How is semantic recall ranked?".to_string(),
                tension: "The vector service may be absent.".to_string(),
                action_implication: "Fall back to typed BM25 traversal.".to_string(),
                source_hashes: vec!["anchor:missing".to_string()],
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                confidence: 90,
                ..Default::default()
            }],
            ..Default::default()
        };
        snapshot.model_hash = memory_graph_model_hash(&snapshot).unwrap();
        snapshot
    }

    #[test]
    fn unavailable_vector_service_preserves_canonical_bm25_packet() {
        let snapshot = snapshot();
        let query = EpiphanyMemoryContextQuery {
            id: "fallback-query".to_string(),
            profile: Some(EpiphanyMemoryProfile::RepoArchitecture),
            text: Some("canonical qdrant semantic".to_string()),
            ..Default::default()
        };
        let expected = plan_memory_graph_context_cut_for_partition(
            &snapshot,
            &query,
            &[],
            SemanticPartition::Modeling,
        );
        let mut config = MemorySemanticIndexConfig::from_env();
        config.qdrant_url = "http://127.0.0.1:1".to_string();
        config.qdrant_timeout_ms = 25;
        let actual = semantic_memory_context(
            &snapshot,
            "swarm",
            SemanticPartition::Modeling,
            &query,
            &config,
        );
        assert_eq!(actual.nodes, expected.nodes);
        assert_eq!(actual.edges, expected.edges);
        assert_eq!(actual.summaries, expected.summaries);
        assert!(
            actual
                .warnings
                .iter()
                .any(|warning| warning.contains("canonical BM25 fallback"))
        );
    }

    #[test]
    fn mind_and_modeling_have_distinct_physical_collections_and_query_profiles() {
        let config = MemorySemanticIndexConfig::from_env();
        assert_ne!(
            config.collection(SemanticPartition::Mind),
            config.collection(SemanticPartition::Modeling)
        );
        assert_ne!(
            config.query_instruction(SemanticPartition::Mind),
            config.query_instruction(SemanticPartition::Modeling)
        );
    }

    fn obligation() -> MemorySemanticProjectionObligation {
        MemorySemanticProjectionObligation {
            schema_version: MEMORY_SEMANTIC_PROJECTION_OBLIGATION_SCHEMA_VERSION.to_string(),
            obligation_id: "projection:swarm:modeling:commit-7".to_string(),
            swarm_id: "swarm".to_string(),
            partition: "modeling".to_string(),
            canonical_source_id: "runtime-spine".to_string(),
            source_commit_id: "commit-7".to_string(),
            graph_id: "graph".to_string(),
            source_generation: 7,
            source_model_hash: "model-hash".to_string(),
            canonical_content_set_hash: "content-hash".to_string(),
            projection_schema_version: SEMANTIC_PROJECTION_SCHEMA_VERSION.to_string(),
            created_at: "2026-07-15T10:00:00Z".to_string(),
        }
    }

    fn source_head() -> MemorySemanticProjectionSourceHead {
        let obligation = obligation();
        MemorySemanticProjectionSourceHead {
            swarm_id: obligation.swarm_id,
            partition: obligation.partition,
            canonical_source_id: obligation.canonical_source_id,
            source_commit_id: obligation.source_commit_id,
            graph_id: obligation.graph_id,
            source_generation: obligation.source_generation,
            source_model_hash: obligation.source_model_hash,
            canonical_content_set_hash: obligation.canonical_content_set_hash,
        }
    }

    fn receipt() -> MemorySemanticIndexReceipt {
        let obligation = obligation();
        MemorySemanticIndexReceipt {
            schema_version: MEMORY_SEMANTIC_INDEX_RECEIPT_SCHEMA_VERSION.to_string(),
            receipt_id: "receipt-7".to_string(),
            swarm_id: obligation.swarm_id,
            partition: obligation.partition,
            collection_name: "epiphany_modeling_v1".to_string(),
            graph_id: obligation.graph_id,
            model_revision: obligation.source_generation,
            model_hash: obligation.source_model_hash,
            embedding_provider_id: "provider".to_string(),
            embedding_model: "model".to_string(),
            vector_dimensions: 1024,
            indexed_document_count: 3,
            deleted_document_count: 0,
            canonical_content_set_hash: obligation.canonical_content_set_hash,
            indexed_at: "2026-07-15T10:01:00Z".to_string(),
            status: "ready".to_string(),
            obligation_id: obligation.obligation_id,
            canonical_source_id: obligation.canonical_source_id,
            source_commit_id: obligation.source_commit_id,
            source_generation: obligation.source_generation,
            projection_schema_version: obligation.projection_schema_version,
        }
    }

    fn failed_attempt() -> MemorySemanticProjectionAttempt {
        MemorySemanticProjectionAttempt {
            schema_version: MEMORY_SEMANTIC_PROJECTION_ATTEMPT_SCHEMA_VERSION.to_string(),
            attempt_id: "attempt-1".to_string(),
            obligation_id: obligation().obligation_id,
            started_at: "2026-07-15T10:00:30Z".to_string(),
            completed_at: Some("2026-07-15T10:00:31Z".to_string()),
            status: "failed".to_string(),
            error: Some("qdrant unavailable".to_string()),
        }
    }

    #[test]
    fn projection_health_is_derived_from_exact_canonical_causality() {
        let obligation = obligation();
        let current = source_head();
        let pending =
            derive_memory_semantic_projection_health(&obligation, &current, &[], &[]).unwrap();
        assert_eq!(
            pending.status,
            MemorySemanticProjectionHealthStatus::Pending
        );
        assert!(!pending.query_eligible);

        let failed = derive_memory_semantic_projection_health(
            &obligation,
            &current,
            &[failed_attempt()],
            &[],
        )
        .unwrap();
        assert_eq!(failed.status, MemorySemanticProjectionHealthStatus::Failed);
        assert_eq!(failed.latest_error.as_deref(), Some("qdrant unavailable"));

        let ready = derive_memory_semantic_projection_health(
            &obligation,
            &current,
            &[failed_attempt()],
            &[receipt()],
        )
        .unwrap();
        assert_eq!(ready.status, MemorySemanticProjectionHealthStatus::Ready);
        assert!(ready.query_eligible);

        let mut newer = current.clone();
        newer.source_generation += 1;
        newer.source_commit_id = "commit-8".to_string();
        let stale = derive_memory_semantic_projection_health(
            &obligation,
            &newer,
            &[failed_attempt()],
            &[receipt()],
        )
        .unwrap();
        assert_eq!(stale.status, MemorySemanticProjectionHealthStatus::Stale);
        assert!(!stale.query_eligible);
    }

    #[test]
    fn hostile_receipt_mismatches_never_grant_query_eligibility() {
        let obligation = obligation();
        let current = source_head();
        for field in [
            "obligation",
            "swarm",
            "partition",
            "source",
            "commit",
            "graph",
            "generation",
            "revision",
            "model",
            "content",
            "projection-schema",
            "status",
        ] {
            let mut hostile = receipt();
            match field {
                "obligation" => hostile.obligation_id = "other".to_string(),
                "swarm" => hostile.swarm_id = "other".to_string(),
                "partition" => hostile.partition = "mind".to_string(),
                "source" => hostile.canonical_source_id = "other".to_string(),
                "commit" => hostile.source_commit_id = "other".to_string(),
                "graph" => hostile.graph_id = "other".to_string(),
                "generation" => hostile.source_generation += 1,
                "revision" => hostile.model_revision += 1,
                "model" => hostile.model_hash = "other".to_string(),
                "content" => hostile.canonical_content_set_hash = "other".to_string(),
                "projection-schema" => hostile.projection_schema_version = "other".to_string(),
                "status" => hostile.status = "failed".to_string(),
                _ => unreachable!(),
            }
            assert!(
                !memory_semantic_projection_query_eligible(&obligation, &current, &hostile),
                "hostile {field} substitution was accepted"
            );
        }
    }

    #[test]
    fn unbound_index_result_is_not_ready_until_exact_obligation_binding() {
        let obligation = obligation();
        let current = source_head();
        let mut result = receipt();
        result.obligation_id.clear();
        result.canonical_source_id.clear();
        result.source_commit_id.clear();
        assert!(!memory_semantic_projection_query_eligible(
            &obligation,
            &current,
            &result
        ));
        let bound = bind_memory_semantic_index_receipt(result, &obligation).unwrap();
        assert!(memory_semantic_projection_query_eligible(
            &obligation,
            &current,
            &bound
        ));
    }
}
