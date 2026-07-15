use super::{
    EpiphanyMemoryContextPacket, EpiphanyMemoryContextQuery, EpiphanyMemoryGraphSnapshot,
    SEMANTIC_PROJECTION_SCHEMA_VERSION, SemanticPartition, SemanticProjectionCandidate,
    SemanticProjectionDocument, derive_semantic_projection, plan_memory_graph_context_cut,
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
            let mut packet = plan_memory_graph_context_cut(snapshot, query);
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
        let expected = plan_memory_graph_context_cut(&snapshot, &query);
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
}
