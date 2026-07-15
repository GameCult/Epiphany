//! Typed boundary around the Qdrant and Ollama HTTP APIs.
//!
//! JSON is deliberately confined to this xenos-facing module. Callers provide
//! typed collection contracts, points, and payloads; Qdrant remains a
//! rebuildable projection rather than canonical authority.

use anyhow::{Context, Result};
use reqwest::blocking::{Client, ClientBuilder, Response};
use reqwest::header::{HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::Duration;

const POINT_BATCH_SIZE: usize = 128;
const EMBED_BATCH_SIZE: usize = 32;

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

/// Exact identity of a managed semantic projection.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CollectionCompatibility {
    pub(crate) managed_by: String,
    pub(crate) corpus_kind: String,
    pub(crate) schema_version: u32,
    pub(crate) projection_version: String,
    pub(crate) embedding_provider_id: String,
    pub(crate) embedding_model: String,
    pub(crate) vector_size: usize,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct SemanticPoint<P> {
    pub(crate) id: String,
    pub(crate) vector: Vec<f32>,
    pub(crate) payload: P,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SemanticCandidate<P> {
    pub(crate) score: f32,
    pub(crate) payload: Option<P>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct SemanticStoredPoint<P> {
    pub(crate) id: String,
    pub(crate) payload: Option<P>,
}

pub(crate) struct QdrantBackend {
    base_url: String,
    timeout_seconds: u64,
    client: Client,
}

impl QdrantBackend {
    pub(crate) fn new(config: QdrantConfig) -> Result<Self> {
        let mut headers = HeaderMap::new();
        if let Some(api_key) = config.api_key.filter(|key| !key.is_empty()) {
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

    pub(crate) fn collection_exists(&self, name: &str) -> Result<bool> {
        let response = self
            .client
            .get(format!("{}/collections/{name}/exists", self.base_url))
            .send()
            .with_context(|| format!("failed to query Qdrant collection {name}"))?;
        let envelope: Envelope<ExistsResult> = parse_response(response).with_context(|| {
            format!("failed to decode Qdrant collection-exists response for {name}")
        })?;
        Ok(envelope.result.exists)
    }

    /// Reads the compatibility contract Qdrant stores with the collection.
    /// Missing or malformed metadata is an error, never implicit compatibility.
    pub(crate) fn collection_compatibility(&self, name: &str) -> Result<CollectionCompatibility> {
        let response = self
            .client
            .get(format!("{}/collections/{name}", self.base_url))
            .send()
            .with_context(|| format!("failed to inspect Qdrant collection {name}"))?;
        let envelope: Envelope<CollectionInfo> = parse_response(response)
            .with_context(|| format!("failed to decode Qdrant collection metadata for {name}"))?;
        envelope
            .result
            .config
            .metadata
            .context("managed Qdrant collection has no compatibility metadata")
    }

    pub(crate) fn create_collection(
        &self,
        name: &str,
        contract: &CollectionCompatibility,
    ) -> Result<()> {
        if contract.vector_size == 0 {
            anyhow::bail!("collection vector size must be greater than zero");
        }
        let body = json!({
            "vectors": { "size": contract.vector_size, "distance": "Cosine", "on_disk": true },
            "on_disk_payload": true,
            "metadata": contract,
        });
        let response = self
            .client
            .put(format!("{}/collections/{name}", self.base_url))
            .query(&[("timeout", self.timeout_seconds)])
            .json(&body)
            .send()
            .with_context(|| format!("failed to create Qdrant collection {name}"))?;
        parse_response::<bool>(response).with_context(|| {
            format!("failed to decode Qdrant create-collection response for {name}")
        })?;
        Ok(())
    }

    pub(crate) fn upsert_points<P: Serialize>(
        &self,
        name: &str,
        points: &[SemanticPoint<P>],
    ) -> Result<()> {
        validate_point_batch(points)?;
        for batch in points.chunks(POINT_BATCH_SIZE) {
            let response = self
                .client
                .put(format!("{}/collections/{name}/points", self.base_url))
                .query(&[
                    ("wait", "true"),
                    ("timeout", &self.timeout_seconds.to_string()),
                ])
                .json(&json!({ "points": batch }))
                .send()
                .with_context(|| format!("failed to upsert Qdrant points into {name}"))?;
            parse_response::<Value>(response)
                .with_context(|| format!("failed to decode Qdrant upsert response for {name}"))?;
        }
        Ok(())
    }

    pub(crate) fn delete_points(&self, name: &str, point_ids: &[String]) -> Result<()> {
        for batch in point_ids.chunks(POINT_BATCH_SIZE) {
            let response = self
                .client
                .post(format!(
                    "{}/collections/{name}/points/delete",
                    self.base_url
                ))
                .query(&[
                    ("wait", "true"),
                    ("timeout", &self.timeout_seconds.to_string()),
                ])
                .json(&json!({ "points": batch }))
                .send()
                .with_context(|| format!("failed to delete Qdrant points from {name}"))?;
            parse_response::<Value>(response).with_context(|| {
                format!("failed to decode Qdrant delete-points response for {name}")
            })?;
        }
        Ok(())
    }

    pub(crate) fn query_points_for_scope<P: DeserializeOwned>(
        &self,
        name: &str,
        vector: &[f32],
        limit: usize,
        scope: &[(&str, &str)],
    ) -> Result<Vec<SemanticCandidate<P>>> {
        if vector.is_empty() {
            anyhow::bail!("query vector must not be empty");
        }
        if limit == 0 {
            return Ok(Vec::new());
        }
        let must = scope
            .iter()
            .map(|(key, value)| json!({ "key": key, "match": { "value": value } }))
            .collect::<Vec<_>>();
        let response = self.client.post(format!("{}/collections/{name}/points/query", self.base_url))
            .query(&[("timeout", self.timeout_seconds)])
            .json(&json!({ "query": vector, "filter": { "must": must }, "limit": limit, "with_payload": true, "with_vector": false }))
            .send().with_context(|| format!("failed to query Qdrant collection {name}"))?;
        let envelope: Envelope<QueryResult<P>> = parse_response(response)
            .with_context(|| format!("failed to decode Qdrant query response for {name}"))?;
        Ok(envelope
            .result
            .points
            .into_iter()
            .map(|point| SemanticCandidate {
                score: point.score,
                payload: point.payload,
            })
            .collect())
    }

    pub(crate) fn point_ids_for_scope(
        &self,
        name: &str,
        scope: &[(&str, &str)],
    ) -> Result<Vec<String>> {
        let must = scope
            .iter()
            .map(|(key, value)| json!({ "key": key, "match": { "value": value } }))
            .collect::<Vec<_>>();
        let mut offset: Option<Value> = None;
        let mut ids = Vec::new();
        loop {
            let response = self
                .client
                .post(format!(
                    "{}/collections/{name}/points/scroll",
                    self.base_url
                ))
                .query(&[("timeout", self.timeout_seconds)])
                .json(&json!({
                    "filter": { "must": must },
                    "limit": POINT_BATCH_SIZE,
                    "offset": offset,
                    "with_payload": false,
                    "with_vector": false,
                }))
                .send()
                .with_context(|| format!("failed to scroll Qdrant scope in {name}"))?;
            let envelope: Envelope<ScrollResult> = parse_response(response)
                .with_context(|| format!("failed to decode Qdrant scroll response for {name}"))?;
            ids.extend(envelope.result.points.into_iter().map(|point| point.id));
            offset = envelope.result.next_page_offset;
            if offset.is_none() {
                break;
            }
        }
        Ok(ids)
    }

    pub(crate) fn points_for_scope<P: DeserializeOwned>(
        &self,
        name: &str,
        scope: &[(&str, &str)],
    ) -> Result<Vec<SemanticStoredPoint<P>>> {
        let must = scope
            .iter()
            .map(|(key, value)| json!({ "key": key, "match": { "value": value } }))
            .collect::<Vec<_>>();
        let mut offset: Option<Value> = None;
        let mut points = Vec::new();
        loop {
            let response = self
                .client
                .post(format!(
                    "{}/collections/{name}/points/scroll",
                    self.base_url
                ))
                .query(&[("timeout", self.timeout_seconds)])
                .json(&json!({
                    "filter": { "must": must },
                    "limit": POINT_BATCH_SIZE,
                    "offset": offset,
                    "with_payload": true,
                    "with_vector": false,
                }))
                .send()
                .with_context(|| format!("failed to observe Qdrant scope in {name}"))?;
            let envelope: Envelope<PayloadScrollResult<P>> = parse_response(response)
                .with_context(|| format!("failed to decode Qdrant scope payloads in {name}"))?;
            points.extend(
                envelope
                    .result
                    .points
                    .into_iter()
                    .map(|point| SemanticStoredPoint {
                        id: point.id,
                        payload: point.payload,
                    }),
            );
            offset = envelope.result.next_page_offset;
            if offset.is_none() {
                break;
            }
        }
        Ok(points)
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
        for batch in texts.chunks(EMBED_BATCH_SIZE) {
            embeddings.extend(self.embed_batch(batch)?);
        }
        validate_embedding_batch(&embeddings, texts.len())?;
        Ok(embeddings)
    }

    pub(crate) fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
        let formatted = format!("Instruct: {}\nQuery: {}", self.query_instruction, query);
        let mut embeddings = self.embed_batch(&[formatted])?;
        validate_embedding_batch(&embeddings, 1)?;
        Ok(embeddings.pop().expect("validated one embedding"))
    }

    fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let response = self
            .client
            .post(format!("{}/api/embed", self.base_url))
            .json(&json!({ "model": self.model, "input": texts }))
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
        let payload: EmbedResponse = response.json().with_context(|| {
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

pub(crate) fn validate_embedding_batch(embeddings: &[Vec<f32>], expected: usize) -> Result<()> {
    if embeddings.len() != expected {
        anyhow::bail!(
            "embedding backend returned {} vectors for {expected} inputs",
            embeddings.len()
        );
    }
    let Some(size) = embeddings.first().map(Vec::len) else {
        return Ok(());
    };
    if size == 0 {
        anyhow::bail!("embedding backend returned an empty vector");
    }
    if let Some((index, _)) = embeddings
        .iter()
        .enumerate()
        .find(|(_, vector)| vector.len() != size)
    {
        anyhow::bail!("embedding backend returned inconsistent vector length at item {index}");
    }
    Ok(())
}

pub(crate) fn validate_point_batch<P>(points: &[SemanticPoint<P>]) -> Result<()> {
    let Some(vector_size) = points.first().map(|point| point.vector.len()) else {
        return Ok(());
    };
    if vector_size == 0 {
        anyhow::bail!("point 0 has an empty vector");
    }
    if let Some((index, _)) = points
        .iter()
        .enumerate()
        .find(|(_, point)| point.vector.len() != vector_size)
    {
        anyhow::bail!("point {index} has an inconsistent vector length");
    }
    Ok(())
}

#[derive(Deserialize)]
struct Envelope<T> {
    result: T,
}
#[derive(Deserialize)]
struct ExistsResult {
    exists: bool,
}
#[derive(Deserialize)]
struct CollectionInfo {
    config: CollectionConfig,
}
#[derive(Deserialize)]
struct CollectionConfig {
    metadata: Option<CollectionCompatibility>,
}
#[derive(Deserialize)]
struct QueryResult<P> {
    points: Vec<QueryPoint<P>>,
}
#[derive(Deserialize)]
struct QueryPoint<P> {
    score: f32,
    payload: Option<P>,
}
#[derive(Deserialize)]
struct ScrollResult {
    points: Vec<ScrollPoint>,
    next_page_offset: Option<Value>,
}
#[derive(Deserialize)]
struct ScrollPoint {
    id: String,
}
#[derive(Deserialize)]
struct PayloadScrollResult<P> {
    points: Vec<PayloadScrollPoint<P>>,
    next_page_offset: Option<Value>,
}
#[derive(Deserialize)]
struct PayloadScrollPoint<P> {
    id: String,
    payload: Option<P>,
}
#[derive(Deserialize)]
struct EmbedResponse {
    embeddings: Option<Vec<Vec<f32>>>,
}

fn parse_response<T: DeserializeOwned>(response: Response) -> Result<Envelope<T>> {
    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        anyhow::bail!("Qdrant request failed with {status}: {body}");
    }
    response
        .json()
        .context("failed to decode Qdrant response JSON")
}

fn normalize_base_url(value: &str) -> String {
    value.trim_end_matches('/').to_string()
}
fn timeout_seconds(timeout_ms: u64) -> u64 {
    timeout_ms.div_ceil(1000).max(1)
}
