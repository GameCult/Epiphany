use anyhow::Context;
use anyhow::Result;
use reqwest::StatusCode;
use reqwest::blocking::Client;
use reqwest::blocking::ClientBuilder;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use std::collections::BTreeMap;
use std::time::Duration;

const QDRANT_POINT_BATCH_SIZE: usize = 128;
const OLLAMA_EMBED_BATCH_SIZE: usize = 32;
pub(crate) const DEFAULT_QDRANT_URL: &str = "http://127.0.0.1:6333";
pub(crate) const DEFAULT_QDRANT_TIMEOUT_MS: u64 = 30_000;
pub(crate) const DEFAULT_OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";
pub(crate) const DEFAULT_OLLAMA_MODEL: &str = "qwen3-embedding:0.6b";
pub(crate) const DEFAULT_OLLAMA_TIMEOUT_MS: u64 = 30_000;

#[derive(Clone, Debug)]
pub(crate) struct QdrantConfig {
    pub(crate) url: String,
    pub(crate) api_key: Option<String>,
    pub(crate) timeout_ms: u64,
}

#[derive(Clone, Debug)]
pub(crate) struct OllamaConfig {
    pub(crate) base_url: String,
    pub(crate) model: String,
    pub(crate) timeout_ms: u64,
    pub(crate) query_instruction: String,
}

pub(crate) fn qdrant_config_from_env() -> QdrantConfig {
    QdrantConfig {
        url: env_override("EPIPHANY_QDRANT_URL", "QDRANT_URL")
            .unwrap_or_else(|| DEFAULT_QDRANT_URL.to_string()),
        api_key: env_override("EPIPHANY_QDRANT_API_KEY", "QDRANT_API_KEY"),
        timeout_ms: env_override("EPIPHANY_QDRANT_TIMEOUT_MS", "QDRANT_TIMEOUT_MS")
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(DEFAULT_QDRANT_TIMEOUT_MS),
    }
}

pub(crate) fn ollama_config_from_env(query_instruction: impl Into<String>) -> OllamaConfig {
    OllamaConfig {
        base_url: env_override("EPIPHANY_OLLAMA_BASE_URL", "RAG_OLLAMA_BASE_URL")
            .unwrap_or_else(|| DEFAULT_OLLAMA_BASE_URL.to_string()),
        model: env_override("EPIPHANY_OLLAMA_MODEL", "RAG_OLLAMA_MODEL")
            .unwrap_or_else(|| DEFAULT_OLLAMA_MODEL.to_string()),
        timeout_ms: env_override("EPIPHANY_OLLAMA_TIMEOUT_MS", "RAG_OLLAMA_TIMEOUT_MS")
            .and_then(|value| value.parse::<u64>().ok())
            .unwrap_or(DEFAULT_OLLAMA_TIMEOUT_MS),
        query_instruction: env_override(
            "EPIPHANY_OLLAMA_QUERY_INSTRUCTION",
            "RAG_SOURCE_QUERY_INSTRUCTION",
        )
        .unwrap_or_else(|| query_instruction.into()),
    }
}

pub(crate) struct QdrantClient {
    base_url: String,
    timeout_seconds: u64,
    client: Client,
}

impl QdrantClient {
    pub(crate) fn new(config: QdrantConfig) -> Result<Self> {
        let mut headers = HeaderMap::new();
        if let Some(api_key) = config.api_key
            && !api_key.is_empty()
        {
            headers.insert(
                "api-key",
                HeaderValue::from_str(&api_key).context("invalid Qdrant api key")?,
            );
        }

        let client = ClientBuilder::new()
            .default_headers(headers)
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .context("failed to build Qdrant client")?;

        Ok(Self {
            base_url: normalize_base_url(&config.url),
            timeout_seconds: timeout_seconds(config.timeout_ms),
            client,
        })
    }

    pub(crate) fn collection_exists(&self, collection_name: &str) -> Result<bool> {
        let response = self
            .client
            .get(format!(
                "{}/collections/{collection_name}/exists",
                self.base_url
            ))
            .send()
            .with_context(|| format!("failed to query Qdrant collection {collection_name}"))?;

        let payload: QdrantEnvelope<QdrantCollectionExistsResult> = parse_qdrant_response(response)
            .with_context(|| {
                format!("failed to decode Qdrant collection-exists response for {collection_name}")
            })?;
        Ok(payload.result.exists)
    }

    pub(crate) fn delete_collection(&self, collection_name: &str) -> Result<()> {
        let response = self
            .client
            .delete(format!("{}/collections/{collection_name}", self.base_url))
            .query(&[("timeout", self.timeout_seconds)])
            .send()
            .with_context(|| format!("failed to delete Qdrant collection {collection_name}"))?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(());
        }
        parse_qdrant_response::<bool>(response).with_context(|| {
            format!("failed to decode Qdrant delete-collection response for {collection_name}")
        })?;
        Ok(())
    }

    pub(crate) fn create_collection(
        &self,
        collection_name: &str,
        vector_size: usize,
        metadata: Value,
    ) -> Result<()> {
        let body = json!({
            "vectors": {
                "size": vector_size,
                "distance": "Cosine",
                "on_disk": true,
            },
            "on_disk_payload": true,
            "metadata": metadata,
        });

        let response = self
            .client
            .put(format!("{}/collections/{collection_name}", self.base_url))
            .query(&[("timeout", self.timeout_seconds)])
            .json(&body)
            .send()
            .with_context(|| format!("failed to create Qdrant collection {collection_name}"))?;

        parse_qdrant_response::<bool>(response).with_context(|| {
            format!("failed to decode Qdrant create-collection response for {collection_name}")
        })?;
        Ok(())
    }

    pub(crate) fn upsert_points(
        &self,
        collection_name: &str,
        points: &[QdrantPointInput],
    ) -> Result<()> {
        for chunk in points.chunks(QDRANT_POINT_BATCH_SIZE) {
            let body = json!({ "points": chunk });
            let response = self
                .client
                .put(format!(
                    "{}/collections/{collection_name}/points",
                    self.base_url
                ))
                .query(&[
                    ("wait", "true"),
                    ("timeout", &self.timeout_seconds.to_string()),
                ])
                .json(&body)
                .send()
                .with_context(|| {
                    format!("failed to upsert Qdrant points into {collection_name}")
                })?;

            parse_qdrant_response::<Value>(response).with_context(|| {
                format!("failed to decode Qdrant upsert response for {collection_name}")
            })?;
        }
        Ok(())
    }

    pub(crate) fn delete_points(&self, collection_name: &str, point_ids: &[String]) -> Result<()> {
        if point_ids.is_empty() {
            return Ok(());
        }

        for batch in point_ids.chunks(QDRANT_POINT_BATCH_SIZE) {
            let body = json!({ "points": batch });
            let response = self
                .client
                .post(format!(
                    "{}/collections/{collection_name}/points/delete",
                    self.base_url
                ))
                .query(&[
                    ("wait", "true"),
                    ("timeout", &self.timeout_seconds.to_string()),
                ])
                .json(&body)
                .send()
                .with_context(|| {
                    format!("failed to delete Qdrant points from {collection_name}")
                })?;

            parse_qdrant_response::<Value>(response).with_context(|| {
                format!("failed to decode Qdrant delete-points response for {collection_name}")
            })?;
        }
        Ok(())
    }

    pub(crate) fn query_points(
        &self,
        collection_name: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<QdrantQueryPoint>> {
        let body = json!({
            "query": query_vector,
            "limit": limit,
            "with_payload": true,
            "with_vector": false,
        });

        let response = self
            .client
            .post(format!(
                "{}/collections/{collection_name}/points/query",
                self.base_url
            ))
            .query(&[("timeout", self.timeout_seconds)])
            .json(&body)
            .send()
            .with_context(|| format!("failed to query Qdrant collection {collection_name}"))?;

        let payload: QdrantEnvelope<QdrantQueryResultEnvelope> = parse_qdrant_response(response)
            .with_context(|| {
                format!("failed to decode Qdrant query response for {collection_name}")
            })?;
        Ok(payload.result.points)
    }
}

pub(crate) struct OllamaEmbedder {
    base_url: String,
    model: String,
    query_instruction: String,
    client: Client,
}

impl OllamaEmbedder {
    pub(crate) fn new(config: OllamaConfig) -> Result<Self> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .context("failed to build Ollama client")?;

        Ok(Self {
            base_url: normalize_base_url(&config.base_url),
            model: config.model,
            query_instruction: config.query_instruction,
            client,
        })
    }

    pub(crate) fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        for batch in texts.chunks(OLLAMA_EMBED_BATCH_SIZE) {
            let payload = self.embed_batch(batch)?;
            embeddings.extend(payload);
        }
        Ok(embeddings)
    }

    pub(crate) fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
        let formatted = format!("Instruct: {}\nQuery: {}", self.query_instruction, query);
        let mut payload = self.embed_batch(&[formatted])?;
        payload
            .pop()
            .context("Ollama embedding backend returned no vector for query")
    }

    fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let response = self
            .client
            .post(format!("{}/api/embed", self.base_url))
            .json(&json!({
                "model": self.model,
                "input": texts,
            }))
            .send()
            .with_context(|| {
                format!(
                    "failed to contact Ollama at {} using model {}",
                    self.base_url, self.model
                )
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            anyhow::bail!(
                "Ollama embedding request failed with {status}: {body}. Make sure Ollama is running at {} and model {} is available",
                self.base_url,
                self.model
            );
        }

        let payload: OllamaEmbedResponse = response.json().with_context(|| {
            format!(
                "failed to decode Ollama embedding response from {}",
                self.base_url
            )
        })?;
        payload
            .embeddings
            .context("Ollama embedding response did not include embeddings")
    }
}

#[derive(Serialize)]
pub(crate) struct QdrantPointInput {
    pub(crate) id: String,
    pub(crate) vector: Vec<f32>,
    pub(crate) payload: Value,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct QdrantQueryPoint {
    pub(crate) score: f32,
    #[serde(default)]
    pub(crate) payload: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize)]
struct QdrantEnvelope<T> {
    result: T,
}

#[derive(Debug, Deserialize)]
struct QdrantCollectionExistsResult {
    exists: bool,
}

#[derive(Debug, Deserialize)]
struct QdrantQueryResultEnvelope {
    points: Vec<QdrantQueryPoint>,
}

#[derive(Debug, Deserialize)]
struct OllamaEmbedResponse {
    embeddings: Option<Vec<Vec<f32>>>,
}

fn parse_qdrant_response<T: for<'de> Deserialize<'de>>(
    response: reqwest::blocking::Response,
) -> Result<QdrantEnvelope<T>> {
    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        anyhow::bail!("Qdrant request failed with {status}: {body}");
    }
    response
        .json()
        .context("failed to decode Qdrant response JSON")
}

pub(crate) fn normalize_base_url(input: &str) -> String {
    input.trim_end_matches('/').to_string()
}

pub(crate) fn validate_embedding_batch(
    embeddings: &[Vec<f32>],
    expected_count: usize,
) -> Result<()> {
    if embeddings.len() != expected_count {
        anyhow::bail!(
            "embedding backend returned {} vectors for {} documents",
            embeddings.len(),
            expected_count
        );
    }

    let Some(vector_length) = embeddings.first().map(Vec::len) else {
        return Ok(());
    };
    if vector_length == 0 {
        anyhow::bail!("embedding backend returned an empty vector");
    }

    for (index, embedding) in embeddings.iter().enumerate() {
        if embedding.len() != vector_length {
            anyhow::bail!("embedding backend returned inconsistent vector length at item {index}");
        }
    }

    Ok(())
}

fn timeout_seconds(timeout_ms: u64) -> u64 {
    timeout_ms.div_ceil(1000).max(1)
}

fn env_override(primary: &str, fallback: &str) -> Option<String> {
    std::env::var(primary)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var(fallback)
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
}
