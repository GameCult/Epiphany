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
use std::collections::{HashMap, HashSet};
use std::time::Duration;

pub(crate) const QDRANT_POINT_BATCH_MAX: usize = 128;
const EMBED_BATCH_SIZE: usize = 4;

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct OllamaModelArtifact {
    pub(crate) tag: String,
    pub(crate) digest: String,
}

impl OllamaModelArtifact {
    pub(crate) fn canonical_identity(&self) -> String {
        format!("{}@{}", self.tag, self.digest)
    }
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
    pub(crate) vector: Option<Vec<f32>>,
}

/// Transport acknowledgement only. This proves that Qdrant completed one
/// waited operation; it does not prove any domain projection invariant.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct QdrantCompletedAcknowledgement {
    pub(crate) operation_id: u64,
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

    /// Ensures a collection exists under one exact managed contract. A
    /// concurrent creator may win between exists/create; that is acceptable
    /// only when the collection it created has the identical contract.
    pub(crate) fn ensure_exact_collection(
        &self,
        name: &str,
        contract: &CollectionCompatibility,
    ) -> Result<()> {
        if !self.collection_exists(name)? {
            if let Err(create_error) = self.create_collection(name, contract) {
                if !self.collection_exists(name)? {
                    return Err(create_error).with_context(|| {
                        format!("failed to create managed Qdrant collection {name}")
                    });
                }
            }
        }
        if self.collection_compatibility(name)? != *contract {
            anyhow::bail!("managed Qdrant collection {name} has incompatible metadata");
        }
        Ok(())
    }

    /// Deletes only the exact managed projection named by `contract`.
    /// Absence is already the retired state; incompatible metadata is never
    /// overwritten or deleted merely because its name collides.
    pub(crate) fn retire_exact_collection(
        &self,
        name: &str,
        contract: &CollectionCompatibility,
    ) -> Result<()> {
        if !self.collection_exists(name)? {
            return Ok(());
        }
        if self.collection_compatibility(name)? != *contract {
            anyhow::bail!("refusing to retire Qdrant collection {name} with incompatible metadata");
        }
        let response = self
            .client
            .delete(format!("{}/collections/{name}", self.base_url))
            .query(&[("timeout", self.timeout_seconds)])
            .send()
            .with_context(|| format!("failed to retire Qdrant collection {name}"))?;
        parse_response::<bool>(response)
            .with_context(|| format!("failed to decode Qdrant retirement response for {name}"))?;
        Ok(())
    }

    pub(crate) fn upsert_points<P: Serialize>(
        &self,
        name: &str,
        points: &[SemanticPoint<P>],
    ) -> Result<()> {
        validate_point_batch(points)?;
        for batch in points.chunks(QDRANT_POINT_BATCH_MAX) {
            self.upsert_point_batch_waited(name, batch)?;
        }
        Ok(())
    }

    pub(crate) fn upsert_point_batch_waited<P: Serialize>(
        &self,
        name: &str,
        points: &[SemanticPoint<P>],
    ) -> Result<QdrantCompletedAcknowledgement> {
        if points.is_empty() {
            anyhow::bail!("Qdrant waited upsert batch must not be empty");
        }
        if points.len() > QDRANT_POINT_BATCH_MAX {
            anyhow::bail!(
                "Qdrant waited upsert batch exceeds canonical maximum {QDRANT_POINT_BATCH_MAX}"
            );
        }
        validate_point_batch(points)?;
        let response = self
            .client
            .put(format!("{}/collections/{name}/points", self.base_url))
            .query(&[
                ("wait", "true"),
                ("timeout", &self.timeout_seconds.to_string()),
            ])
            .json(&json!({ "points": points }))
            .send()
            .with_context(|| format!("failed to upsert Qdrant points into {name}"))?;
        let envelope: Envelope<QdrantUpdateResult> = parse_response(response)
            .with_context(|| format!("failed to decode Qdrant upsert response for {name}"))?;
        validate_completed_acknowledgement(envelope.result)
    }

    pub(crate) fn delete_points(&self, name: &str, point_ids: &[String]) -> Result<()> {
        for batch in point_ids.chunks(QDRANT_POINT_BATCH_MAX) {
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

    pub(crate) fn retrieve_points_by_ids<P: DeserializeOwned>(
        &self,
        name: &str,
        point_ids: &[String],
    ) -> Result<Vec<SemanticStoredPoint<P>>> {
        validate_requested_point_ids(point_ids)?;
        let response = self
            .client
            .post(format!("{}/collections/{name}/points", self.base_url))
            .query(&[("timeout", self.timeout_seconds)])
            .json(&json!({
                "ids": point_ids,
                "with_payload": true,
                "with_vector": true,
            }))
            .send()
            .with_context(|| format!("failed to retrieve exact Qdrant points from {name}"))?;
        let envelope: Envelope<Vec<PayloadScrollPoint<P>>> = parse_response(response)
            .with_context(|| format!("failed to decode exact Qdrant points from {name}"))?;
        order_and_validate_retrieved_points(point_ids, envelope.result)
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
        let mut seen_offsets = Vec::<Value>::new();
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
                    "limit": QDRANT_POINT_BATCH_MAX,
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
            if let Some(next_offset) = offset.as_ref() {
                if seen_offsets.contains(next_offset) {
                    anyhow::bail!("Qdrant scroll repeated a prior page offset for {name}");
                }
                seen_offsets.push(next_offset.clone());
            }
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
        let mut seen_offsets = Vec::<Value>::new();
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
                    "limit": QDRANT_POINT_BATCH_MAX,
                    "offset": offset,
                    "with_payload": true,
                    "with_vector": true,
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
                        vector: point.vector,
                    }),
            );
            offset = envelope.result.next_page_offset;
            if let Some(next_offset) = offset.as_ref() {
                if seen_offsets.contains(next_offset) {
                    anyhow::bail!("Qdrant payload scroll repeated a prior page offset for {name}");
                }
                seen_offsets.push(next_offset.clone());
            }
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

    /// Resolves the configured mutable Ollama tag to the immutable artifact
    /// digest currently installed behind it. Projection identity must use this
    /// value; embedding requests continue to use the configured tag.
    pub(crate) fn model_artifact(&self) -> Result<OllamaModelArtifact> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .with_context(|| format!("failed to inspect Ollama models at {}", self.base_url))?;
        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            anyhow::bail!("Ollama model inspection failed with {status}: {body}");
        }
        let tags: OllamaTagsResponse = response.json().with_context(|| {
            format!("failed to decode Ollama model tags from {}", self.base_url)
        })?;
        let mut matches = tags
            .models
            .into_iter()
            .filter(|candidate| candidate.name == self.model || candidate.model == self.model)
            .collect::<Vec<_>>();
        if matches.len() != 1 {
            anyhow::bail!(
                "configured Ollama model {} resolved to {} installed artifacts",
                self.model,
                matches.len()
            );
        }
        let candidate = matches.pop().expect("validated one artifact");
        let digest = normalize_ollama_digest(&candidate.digest)?;
        Ok(OllamaModelArtifact {
            tag: self.model.clone(),
            digest,
        })
    }

    pub(crate) fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
        let formatted = format!("Instruct: {}\nQuery: {}", self.query_instruction, query);
        let mut embeddings = self.embed_batch(&[formatted])?;
        validate_embedding_batch(&embeddings, 1)?;
        Ok(embeddings.pop().expect("validated one embedding"))
    }

    pub(crate) fn embedding_dimensions(&self) -> Result<u32> {
        const DIMENSION_PROBE: &str = "epiphany embedding dimension probe v0";

        let embeddings = self.embed_batch(&[DIMENSION_PROBE.to_string()])?;
        if embeddings.len() != 1 {
            anyhow::bail!(
                "Ollama dimension probe returned {} vectors instead of one",
                embeddings.len()
            );
        }
        let vector = &embeddings[0];
        if vector.is_empty() {
            anyhow::bail!("Ollama dimension probe returned an empty vector");
        }
        if vector.iter().any(|value| !value.is_finite()) {
            anyhow::bail!("Ollama dimension probe returned a non-finite value");
        }
        u32::try_from(vector.len()).context("Ollama embedding dimensions exceed u32")
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
    let mut ids = HashSet::with_capacity(points.len());
    for (index, point) in points.iter().enumerate() {
        if point.id.trim().is_empty() {
            anyhow::bail!("point {index} has a blank id");
        }
        if !ids.insert(point.id.as_str()) {
            anyhow::bail!("point batch contains duplicate id {}", point.id);
        }
        if point.vector.len() != vector_size {
            anyhow::bail!("point {index} has an inconsistent vector length");
        }
        if point.vector.iter().any(|value| !value.is_finite()) {
            anyhow::bail!("point {index} has a non-finite vector");
        }
    }
    Ok(())
}

fn validate_requested_point_ids(point_ids: &[String]) -> Result<()> {
    if point_ids.is_empty() {
        anyhow::bail!("exact Qdrant point retrieval requires at least one id");
    }
    if point_ids.len() > QDRANT_POINT_BATCH_MAX {
        anyhow::bail!(
            "exact Qdrant point retrieval exceeds canonical maximum {QDRANT_POINT_BATCH_MAX}"
        );
    }
    let mut seen = HashSet::with_capacity(point_ids.len());
    for point_id in point_ids {
        if point_id.trim().is_empty() {
            anyhow::bail!("exact Qdrant point retrieval contains a blank requested id");
        }
        if !seen.insert(point_id.as_str()) {
            anyhow::bail!(
                "exact Qdrant point retrieval contains duplicate requested id {point_id}"
            );
        }
    }
    Ok(())
}

fn order_and_validate_retrieved_points<P>(
    requested_ids: &[String],
    returned: Vec<PayloadScrollPoint<P>>,
) -> Result<Vec<SemanticStoredPoint<P>>> {
    validate_requested_point_ids(requested_ids)?;
    let requested = requested_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let mut by_id = HashMap::with_capacity(returned.len());
    for point in returned {
        if !requested.contains(point.id.as_str()) {
            anyhow::bail!(
                "Qdrant substituted or returned unrequested point id {}",
                point.id
            );
        }
        let payload = point
            .payload
            .context("Qdrant exact point response omitted a requested payload")?;
        let vector = point
            .vector
            .context("Qdrant exact point response omitted a requested vector")?;
        if vector.iter().any(|value| !value.is_finite()) {
            anyhow::bail!("Qdrant exact point response contained a non-finite vector");
        }
        let id = point.id;
        if by_id
            .insert(
                id.clone(),
                SemanticStoredPoint {
                    id: id.clone(),
                    payload: Some(payload),
                    vector: Some(vector),
                },
            )
            .is_some()
        {
            anyhow::bail!("Qdrant exact point response duplicated point id {id}");
        }
    }
    if by_id.len() != requested_ids.len() {
        let missing = requested_ids
            .iter()
            .find(|point_id| !by_id.contains_key(point_id.as_str()))
            .expect("cardinality mismatch has a missing requested id");
        anyhow::bail!("Qdrant exact point response omitted requested point id {missing}");
    }
    requested_ids
        .iter()
        .map(|point_id| {
            by_id
                .remove(point_id.as_str())
                .with_context(|| format!("Qdrant exact point response omitted {point_id}"))
        })
        .collect()
}

fn validate_completed_acknowledgement(
    result: QdrantUpdateResult,
) -> Result<QdrantCompletedAcknowledgement> {
    if result.status != "completed" {
        anyhow::bail!(
            "Qdrant waited operation returned status {} instead of completed",
            result.status
        );
    }
    Ok(QdrantCompletedAcknowledgement {
        operation_id: result.operation_id,
    })
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
struct QdrantUpdateResult {
    operation_id: u64,
    status: String,
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
    vector: Option<Vec<f32>>,
}
#[derive(Deserialize)]
struct EmbedResponse {
    embeddings: Option<Vec<Vec<f32>>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OllamaTagsResponse {
    models: Vec<OllamaTag>,
}

#[derive(Deserialize)]
struct OllamaTag {
    name: String,
    model: String,
    digest: String,
}

fn normalize_ollama_digest(value: &str) -> Result<String> {
    let hex = value.strip_prefix("sha256:").unwrap_or(value);
    if hex.len() != 64 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        anyhow::bail!("Ollama model artifact digest is not a SHA-256 digest");
    }
    Ok(format!("sha256:{}", hex.to_ascii_lowercase()))
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

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_json, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn embedder(base_url: String, model: &str) -> Result<OllamaEmbedder> {
        OllamaEmbedder::new(OllamaConfig {
            base_url,
            model: model.into(),
            timeout_ms: 5_000,
            query_instruction: String::new(),
        })
    }

    fn qdrant(base_url: String) -> Result<QdrantBackend> {
        QdrantBackend::new(QdrantConfig {
            url: base_url,
            api_key: None,
            timeout_ms: 5_000,
        })
    }

    #[test]
    fn waited_upsert_requires_completed_qdrant_acknowledgement() -> Result<()> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        for (status, accepted) in [("completed", true), ("acknowledged", false)] {
            let server = runtime.block_on(MockServer::start());
            runtime.block_on(
                Mock::given(method("PUT"))
                    .and(path("/collections/c/points"))
                    .and(query_param("wait", "true"))
                    .and(query_param("timeout", "5"))
                    .and(body_json(json!({
                        "points": [{"id":"p","vector":[1.0],"payload":{"kind":"test"}}]
                    })))
                    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                        "result": {"operation_id": 42, "status": status}
                    })))
                    .mount(&server),
            );
            let points = vec![SemanticPoint {
                id: "p".into(),
                vector: vec![1.0],
                payload: json!({"kind":"test"}),
            }];
            let result = qdrant(server.uri())?.upsert_point_batch_waited("c", &points);
            assert_eq!(result.is_ok(), accepted);
            if let Ok(acknowledgement) = result {
                assert_eq!(acknowledgement.operation_id, 42);
            }
        }
        Ok(())
    }

    #[test]
    fn waited_upsert_enforces_one_canonical_batch() -> Result<()> {
        let backend = qdrant("http://127.0.0.1:1".into())?;
        let empty: Vec<SemanticPoint<Value>> = Vec::new();
        assert!(backend.upsert_point_batch_waited("c", &empty).is_err());
        let too_many = (0..=QDRANT_POINT_BATCH_MAX)
            .map(|index| SemanticPoint {
                id: index.to_string(),
                vector: vec![1.0],
                payload: Value::Null,
            })
            .collect::<Vec<_>>();
        assert!(backend.upsert_point_batch_waited("c", &too_many).is_err());
        Ok(())
    }

    #[test]
    fn exact_point_retrieval_restores_request_order() -> Result<()> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let server = runtime.block_on(MockServer::start());
        runtime.block_on(
                Mock::given(method("POST"))
                .and(path("/collections/c/points"))
                .and(query_param("timeout", "5"))
                .and(body_json(json!({
                    "ids": ["b", "a"],
                    "with_payload": true,
                    "with_vector": true
                })))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "result": [
                        {"id":"a","payload":{"n":1},"vector":[1.0]},
                        {"id":"b","payload":{"n":2},"vector":[2.0]}
                    ]
                })))
                .mount(&server),
        );
        let points = qdrant(server.uri())?
            .retrieve_points_by_ids::<Value>("c", &["b".into(), "a".into()])?;
        assert_eq!(
            points
                .iter()
                .map(|point| point.id.as_str())
                .collect::<Vec<_>>(),
            vec!["b", "a"]
        );
        Ok(())
    }

    #[test]
    fn exact_point_response_validator_rejects_hostile_shapes() {
        let requested = vec!["a".to_string(), "b".to_string()];
        let point =
            |id: &str, payload: Option<Value>, vector: Option<Vec<f32>>| PayloadScrollPoint {
                id: id.into(),
                payload,
                vector,
            };
        assert!(validate_requested_point_ids(&[]).is_err());
        assert!(validate_requested_point_ids(&[" ".into()]).is_err());
        assert!(validate_requested_point_ids(&["a".into(), "a".into()]).is_err());
        assert!(
            validate_requested_point_ids(
                &(0..=QDRANT_POINT_BATCH_MAX)
                    .map(|index| index.to_string())
                    .collect::<Vec<_>>()
            )
            .is_err()
        );
        for returned in [
            vec![
                point("a", Some(json!({})), Some(vec![1.0])),
                point("x", Some(json!({})), Some(vec![2.0])),
            ],
            vec![point("a", Some(json!({})), Some(vec![1.0]))],
            vec![
                point("a", Some(json!({})), Some(vec![1.0])),
                point("a", Some(json!({})), Some(vec![1.0])),
            ],
            vec![
                point("a", None, Some(vec![1.0])),
                point("b", Some(json!({})), Some(vec![2.0])),
            ],
            vec![
                point("a", Some(json!({})), None),
                point("b", Some(json!({})), Some(vec![2.0])),
            ],
            vec![
                point("a", Some(json!({})), Some(vec![f32::NAN])),
                point("b", Some(json!({})), Some(vec![2.0])),
            ],
        ] {
            assert!(order_and_validate_retrieved_points(&requested, returned).is_err());
        }
    }

    #[test]
    fn ollama_model_identity_is_bound_to_exact_installed_digest() -> Result<()> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        let server_a = runtime.block_on(MockServer::start());
        let server_b = runtime.block_on(MockServer::start());
        for (server, digest) in [(&server_a, "aa".repeat(32)), (&server_b, "bb".repeat(32))] {
            runtime.block_on(
                Mock::given(method("GET"))
                    .and(path("/api/tags"))
                    .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                        "models": [{
                            "name": "nomic-embed-text:latest",
                            "model": "nomic-embed-text:latest",
                            "digest": digest
                        }]
                    })))
                    .mount(server),
            );
        }
        let identity_a = embedder(server_a.uri(), "nomic-embed-text:latest")?
            .model_artifact()?
            .canonical_identity();
        let identity_b = embedder(server_b.uri(), "nomic-embed-text:latest")?
            .model_artifact()?
            .canonical_identity();
        assert_ne!(identity_a, identity_b);
        assert_eq!(
            identity_a,
            format!("nomic-embed-text:latest@sha256:{}", "aa".repeat(32))
        );
        Ok(())
    }

    #[test]
    fn ollama_model_identity_refuses_absent_ambiguous_and_invalid_digest() -> Result<()> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?;
        for models in [
            json!([]),
            json!([
                {"name":"m","model":"m","digest":"aa".repeat(32)},
                {"name":"m","model":"m","digest":"bb".repeat(32)}
            ]),
            json!([{"name":"m","model":"m","digest":"not-a-digest"}]),
        ] {
            let server = runtime.block_on(MockServer::start());
            runtime.block_on(
                Mock::given(method("GET"))
                    .and(path("/api/tags"))
                    .respond_with(
                        ResponseTemplate::new(200).set_body_json(json!({"models": models})),
                    )
                    .mount(&server),
            );
            assert!(embedder(server.uri(), "m")?.model_artifact().is_err());
        }
        Ok(())
    }
}
