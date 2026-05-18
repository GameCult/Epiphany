use super::EpiphanyMemoryEmbeddingManifest;
use super::EpiphanyMemoryGraphSnapshot;
use super::memory_graph_embedding_documents;
use super::memory_graph_embedding_manifest;
use crate::semantic_cache::OllamaConfig;
use crate::semantic_cache::OllamaEmbedder;
use crate::semantic_cache::QdrantClient;
use crate::semantic_cache::QdrantConfig;
use crate::semantic_cache::QdrantPointInput;
use crate::semantic_cache::ollama_config_from_env;
use crate::semantic_cache::qdrant_config_from_env;
use crate::semantic_cache::validate_embedding_batch;
use anyhow::Context;
use anyhow::Result;
use serde::Serialize;
use serde_json::json;
use sha1::Digest;
use sha1::Sha1;
use uuid::Uuid;

const MEMORY_GRAPH_INDEX_REVISION: &str = "memory-graph-qdrant-v1";
const DEFAULT_MEMORY_GRAPH_QUERY_INSTRUCTION: &str =
    "Given an agent work question, retrieve the most relevant project memory graph records.";

#[derive(Clone, Debug)]
pub struct EpiphanyMemoryGraphEmbeddingCacheRequest {
    pub collection_name: Option<String>,
    pub embedding_model: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyMemoryGraphEmbeddingCacheReport {
    pub collection_name: String,
    pub embedding_model: String,
    pub vector_dimensions: Option<u32>,
    pub indexed_document_count: usize,
    pub manifest: EpiphanyMemoryEmbeddingManifest,
}

pub fn rebuild_memory_graph_embedding_cache(
    snapshot: &mut EpiphanyMemoryGraphSnapshot,
    request: EpiphanyMemoryGraphEmbeddingCacheRequest,
) -> Result<EpiphanyMemoryGraphEmbeddingCacheReport> {
    let mut ollama_config = ollama_config_from_env(DEFAULT_MEMORY_GRAPH_QUERY_INSTRUCTION);
    if let Some(model) = request.embedding_model.clone() {
        ollama_config.model = model;
    }
    rebuild_memory_graph_embedding_cache_with_config(
        snapshot,
        request,
        qdrant_config_from_env(),
        ollama_config,
    )
}

fn rebuild_memory_graph_embedding_cache_with_config(
    snapshot: &mut EpiphanyMemoryGraphSnapshot,
    request: EpiphanyMemoryGraphEmbeddingCacheRequest,
    qdrant_config: QdrantConfig,
    ollama_config: OllamaConfig,
) -> Result<EpiphanyMemoryGraphEmbeddingCacheReport> {
    let collection_name = request
        .collection_name
        .or_else(|| {
            snapshot
                .embedding_manifest
                .as_ref()
                .and_then(|manifest| manifest.collection_name.clone())
        })
        .unwrap_or_else(|| default_memory_graph_collection_name(&snapshot.graph_id));

    let embedding_model = ollama_config.model.clone();

    let documents = memory_graph_embedding_documents(snapshot);
    let qdrant = QdrantClient::new(qdrant_config)?;
    qdrant.delete_collection(&collection_name)?;

    let vector_dimensions = if documents.is_empty() {
        None
    } else {
        let texts = documents
            .iter()
            .map(|document| document.text.clone())
            .collect::<Vec<_>>();
        let embeddings = OllamaEmbedder::new(ollama_config)?.embed_documents(&texts)?;
        validate_embedding_batch(&embeddings, texts.len())?;
        let vector_size = embeddings
            .first()
            .map(Vec::len)
            .context("embedding backend returned no vectors for memory graph rebuild")?;
        qdrant.create_collection(
            &collection_name,
            vector_size,
            json!({
                "managedBy": "epiphany",
                "schemaVersion": 1,
                "indexRevision": MEMORY_GRAPH_INDEX_REVISION,
                "graphId": snapshot.graph_id,
                "embeddingModel": embedding_model,
                "vectorSize": vector_size,
            }),
        )?;
        let points = documents
            .iter()
            .zip(embeddings)
            .map(|(document, vector)| QdrantPointInput {
                id: memory_graph_point_id(&collection_name, &document.id),
                vector,
                payload: json!({
                    "graphId": snapshot.graph_id,
                    "documentId": document.id,
                    "sourceId": document.source_id,
                    "documentKind": document.document_kind,
                    "profile": document.profile,
                    "lifecycle": document.lifecycle,
                    "sourceHashes": document.source_hashes,
                    "text": document.text,
                }),
            })
            .collect::<Vec<_>>();
        qdrant.upsert_points(&collection_name, &points)?;
        Some(u32::try_from(vector_size).context("embedding vector dimension overflow")?)
    };

    let manifest = memory_graph_embedding_manifest(
        snapshot,
        collection_name.clone(),
        embedding_model.clone(),
        vector_dimensions,
    );
    snapshot.embedding_manifest = Some(manifest.clone());

    Ok(EpiphanyMemoryGraphEmbeddingCacheReport {
        collection_name,
        embedding_model,
        vector_dimensions,
        indexed_document_count: documents.len(),
        manifest,
    })
}

fn default_memory_graph_collection_name(graph_id: &str) -> String {
    format!("epiphany_memory_graph_{}", short_hash(graph_id))
}

fn memory_graph_point_id(collection_name: &str, document_id: &str) -> String {
    let key = format!("{collection_name}\n{document_id}");
    Uuid::new_v5(&Uuid::NAMESPACE_URL, key.as_bytes()).to_string()
}

fn short_hash(value: &str) -> String {
    let mut hasher = Sha1::new();
    hasher.update(value.as_bytes());
    let digest = hasher.finalize();
    hex_lower(&digest[..8])
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_graph::EpiphanyMemoryDomain;
    use crate::memory_graph::EpiphanyMemoryLifecycle;
    use crate::memory_graph::EpiphanyMemoryNode;
    use crate::memory_graph::EpiphanyMemoryNodeKind;
    use crate::memory_graph::EpiphanyMemoryProfile;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;
    use wiremock::matchers::method;
    use wiremock::matchers::path;

    #[test]
    fn memory_graph_qdrant_rebuild_writes_manifest_from_typed_documents() -> Result<()> {
        let runtime = tokio::runtime::Runtime::new()?;
        let qdrant = runtime.block_on(MockServer::start());
        let ollama = runtime.block_on(MockServer::start());

        runtime.block_on(async {
            Mock::given(method("DELETE"))
                .and(path("/collections/test_memory_graph"))
                .respond_with(ResponseTemplate::new(404))
                .mount(&qdrant)
                .await;
            Mock::given(method("POST"))
                .and(path("/api/embed"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "embeddings": [[0.1_f32, 0.2_f32, 0.3_f32]]
                })))
                .mount(&ollama)
                .await;
            Mock::given(method("PUT"))
                .and(path("/collections/test_memory_graph"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"result": true})))
                .mount(&qdrant)
                .await;
            Mock::given(method("PUT"))
                .and(path("/collections/test_memory_graph/points"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({"result": {}})))
                .mount(&qdrant)
                .await;
        });

        let mut snapshot = test_snapshot();
        let report = rebuild_memory_graph_embedding_cache_with_config(
            &mut snapshot,
            EpiphanyMemoryGraphEmbeddingCacheRequest {
                collection_name: Some("test_memory_graph".to_string()),
                embedding_model: None,
            },
            QdrantConfig {
                url: qdrant.uri(),
                api_key: None,
                timeout_ms: 1_000,
            },
            OllamaConfig {
                base_url: ollama.uri(),
                model: "test-embedding".to_string(),
                timeout_ms: 1_000,
                query_instruction: DEFAULT_MEMORY_GRAPH_QUERY_INSTRUCTION.to_string(),
            },
        )?;

        assert_eq!(report.collection_name, "test_memory_graph");
        assert_eq!(report.embedding_model, "test-embedding");
        assert_eq!(report.vector_dimensions, Some(3));
        assert_eq!(report.indexed_document_count, 1);
        assert_eq!(
            snapshot
                .embedding_manifest
                .as_ref()
                .and_then(|manifest| manifest.collection_name.as_deref()),
            Some("test_memory_graph")
        );

        Ok(())
    }

    fn test_snapshot() -> EpiphanyMemoryGraphSnapshot {
        EpiphanyMemoryGraphSnapshot {
            schema_version: Some("epiphany.memory_graph.v0".to_string()),
            graph_id: "graph-for-qdrant-test".to_string(),
            domains: vec![EpiphanyMemoryDomain {
                id: "repo".to_string(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                title: "Repo".to_string(),
                description: None,
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
            }],
            nodes: vec![EpiphanyMemoryNode {
                id: "node-auth-spine".to_string(),
                domain_id: "repo".to_string(),
                profile: EpiphanyMemoryProfile::RepoArchitecture,
                kind: EpiphanyMemoryNodeKind::Module,
                title: "Auth spine".to_string(),
                claim: "Codex owns authentication while Epiphany owns memory.".to_string(),
                question: String::new(),
                tension: String::new(),
                action_implication: String::new(),
                anchors: Vec::new(),
                source_hashes: vec!["source-a".to_string()],
                lifecycle: EpiphanyMemoryLifecycle::Accepted,
                salience: 80,
                confidence: 90,
                created_at: None,
                updated_at: None,
            }],
            ..Default::default()
        }
    }
}
