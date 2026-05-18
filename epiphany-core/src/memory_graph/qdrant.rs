use super::EpiphanyMemoryContextPacket;
use super::EpiphanyMemoryContextQuery;
use super::EpiphanyMemoryEmbeddingManifest;
use super::EpiphanyMemoryGraphSnapshot;
use super::memory_graph_embedding_documents;
use super::memory_graph_embedding_manifest;
use super::plan_memory_graph_context_cut;
use super::push_unique;
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
use serde_json::Value;
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

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EpiphanyMemoryGraphSemanticHit {
    pub document_id: String,
    pub source_id: String,
    pub document_kind: String,
    pub score: f32,
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

pub fn plan_memory_graph_context_cut_with_semantic_cache(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    query: &EpiphanyMemoryContextQuery,
) -> EpiphanyMemoryContextPacket {
    let hits = match query_memory_graph_semantic_cache(snapshot, query) {
        Ok(hits) => hits,
        Err(err) => {
            let mut packet = plan_memory_graph_context_cut(snapshot, query);
            packet.warnings.push(format!(
                "semantic cache unavailable; used typed graph traversal: {err}"
            ));
            return packet;
        }
    };
    plan_memory_graph_context_cut_from_semantic_hits(snapshot, query, hits)
}

fn plan_memory_graph_context_cut_from_semantic_hits(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    query: &EpiphanyMemoryContextQuery,
    hits: Vec<EpiphanyMemoryGraphSemanticHit>,
) -> EpiphanyMemoryContextPacket {
    let mut warnings = Vec::new();
    if hits.is_empty() {
        return plan_memory_graph_context_cut(snapshot, query);
    }

    let mut seeded_query = query.clone();
    for hit in &hits {
        match hit.document_kind.as_str() {
            "node" => push_unique(&mut seeded_query.node_ids, hit.source_id.clone()),
            "edge" => push_unique(&mut seeded_query.edge_ids, hit.source_id.clone()),
            "summary" => {
                if let Some(summary) = snapshot
                    .summaries
                    .iter()
                    .find(|summary| summary.id == hit.source_id)
                {
                    for node_id in &summary.covers_node_ids {
                        push_unique(&mut seeded_query.node_ids, node_id.clone());
                    }
                    for edge_id in &summary.covers_edge_ids {
                        push_unique(&mut seeded_query.edge_ids, edge_id.clone());
                    }
                } else {
                    warnings.push(format!(
                        "semantic cache hit missing summary {}",
                        hit.source_id
                    ));
                }
            }
            other => warnings.push(format!(
                "semantic cache hit {} had unknown document kind {other}",
                hit.document_id
            )),
        }
    }

    let mut packet = plan_memory_graph_context_cut(snapshot, &seeded_query);
    packet.warnings.push(format!(
        "semantic cache seeded {} typed graph document(s)",
        hits.len()
    ));
    packet.warnings.extend(warnings);
    packet
}

pub fn query_memory_graph_semantic_cache(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    query: &EpiphanyMemoryContextQuery,
) -> Result<Vec<EpiphanyMemoryGraphSemanticHit>> {
    let mut ollama_config = ollama_config_from_env(DEFAULT_MEMORY_GRAPH_QUERY_INSTRUCTION);
    if let Some(manifest) = &snapshot.embedding_manifest {
        ollama_config.model = manifest.embedding_model.clone();
    }
    query_memory_graph_semantic_cache_with_config(
        snapshot,
        query,
        qdrant_config_from_env(),
        ollama_config,
    )
}

fn query_memory_graph_semantic_cache_with_config(
    snapshot: &EpiphanyMemoryGraphSnapshot,
    query: &EpiphanyMemoryContextQuery,
    qdrant_config: QdrantConfig,
    ollama_config: OllamaConfig,
) -> Result<Vec<EpiphanyMemoryGraphSemanticHit>> {
    let query_text = query
        .text
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .context("semantic cache context requires query.text")?;
    let manifest = snapshot
        .embedding_manifest
        .as_ref()
        .context("memory graph has no embedding manifest")?;
    let collection_name = manifest
        .collection_name
        .as_deref()
        .context("memory graph embedding manifest has no collection name")?;

    let query_vector = OllamaEmbedder::new(ollama_config)?.embed_query(query_text)?;
    let limit = query.budget.unwrap_or(12).clamp(1, 64) as usize;
    let qdrant = QdrantClient::new(qdrant_config)?;
    let hits = qdrant.query_points(collection_name, &query_vector, limit)?;
    Ok(hits.into_iter().filter_map(map_semantic_hit).collect())
}

fn map_semantic_hit(
    point: crate::semantic_cache::QdrantQueryPoint,
) -> Option<EpiphanyMemoryGraphSemanticHit> {
    let document_id = string_payload(&point.payload, "documentId")?;
    let source_id = string_payload(&point.payload, "sourceId")?;
    let document_kind = string_payload(&point.payload, "documentKind")?;
    Some(EpiphanyMemoryGraphSemanticHit {
        document_id,
        source_id,
        document_kind,
        score: point.score,
    })
}

fn string_payload(
    payload: &std::collections::BTreeMap<String, Value>,
    key: &str,
) -> Option<String> {
    payload.get(key).and_then(Value::as_str).map(str::to_string)
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

    #[test]
    fn memory_graph_semantic_query_returns_ids_not_cached_payload_text() -> Result<()> {
        let runtime = tokio::runtime::Runtime::new()?;
        let qdrant = runtime.block_on(MockServer::start());
        let ollama = runtime.block_on(MockServer::start());

        runtime.block_on(async {
            Mock::given(method("POST"))
                .and(path("/api/embed"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "embeddings": [[0.1_f32, 0.2_f32, 0.3_f32]]
                })))
                .mount(&ollama)
                .await;
            Mock::given(method("POST"))
                .and(path("/collections/test_memory_graph/points/query"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "result": {
                        "points": [{
                            "score": 0.91_f32,
                            "payload": {
                                "documentId": "memembed-node-test",
                                "sourceId": "node-auth-spine",
                                "documentKind": "node",
                                "text": "cached text must not become canonical context"
                            }
                        }]
                    }
                })))
                .mount(&qdrant)
                .await;
        });

        let mut snapshot = test_snapshot();
        snapshot.embedding_manifest = Some(memory_graph_embedding_manifest(
            &snapshot,
            "test_memory_graph",
            "test-embedding",
            Some(3),
        ));
        let hits = query_memory_graph_semantic_cache_with_config(
            &snapshot,
            &EpiphanyMemoryContextQuery {
                id: "semantic-query-test".to_string(),
                text: Some("who owns auth".to_string()),
                budget: Some(3),
                ..Default::default()
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

        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].source_id, "node-auth-spine");
        assert_eq!(hits[0].document_kind, "node");

        let packet = plan_memory_graph_context_cut_from_semantic_hits(
            &snapshot,
            &EpiphanyMemoryContextQuery {
                id: "semantic-query-test".to_string(),
                text: Some("who owns auth".to_string()),
                budget: Some(3),
                ..Default::default()
            },
            hits,
        );
        assert_eq!(packet.nodes.len(), 1);
        assert_eq!(packet.nodes[0].id, "node-auth-spine");
        assert!(
            packet
                .warnings
                .iter()
                .any(|warning| warning.contains("semantic cache seeded 1"))
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
