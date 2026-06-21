use crate::EpiphanyAgentMemoryEntry;
use crate::persona_turn::render_persona_semantic_memory_recall;
use anyhow::Context;
use anyhow::Result;
use epiphany_state_model::EpiphanyMemoryContextPacket;
use reqwest::StatusCode;
use reqwest::blocking::Client;
use reqwest::blocking::ClientBuilder;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use serde_json::json;
use sha1::Digest;
use sha1::Sha1;
use std::time::Duration;

pub const PERSONA_MEMORY_CACHE_SCHEMA_VERSION: &str = "epiphany.persona_memory_cache.v0";
pub const PERSONA_MEMORY_CORPUS_KIND: &str = "persona_memory";

const DEFAULT_QDRANT_URL: &str = "http://127.0.0.1:6333";
const DEFAULT_QDRANT_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_OLLAMA_MODEL: &str = "qwen3-embedding:0.6b";
const DEFAULT_OLLAMA_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_COLLECTION: &str = "epiphany_persona_memory_v0";
const POINT_BATCH_SIZE: usize = 128;
const EMBED_BATCH_SIZE: usize = 32;
const DEFAULT_QUERY_INSTRUCTION: &str = "Given a Persona turn, retrieve relevant memories, goals, values, bonds, needs, status reads, and doctrine from this Persona's own typed state.";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PersonaMemoryCacheConfig {
    pub qdrant_url: String,
    pub qdrant_api_key: Option<String>,
    pub qdrant_timeout_ms: u64,
    pub ollama_base_url: String,
    pub ollama_model: String,
    pub ollama_timeout_ms: u64,
    pub collection_name: String,
    pub query_instruction: String,
}

impl PersonaMemoryCacheConfig {
    pub fn from_env() -> Self {
        Self {
            qdrant_url: env_override("EPIPHANY_QDRANT_URL", "QDRANT_URL")
                .unwrap_or_else(|| DEFAULT_QDRANT_URL.to_string()),
            qdrant_api_key: env_override("EPIPHANY_QDRANT_API_KEY", "QDRANT_API_KEY"),
            qdrant_timeout_ms: env_override("EPIPHANY_QDRANT_TIMEOUT_MS", "QDRANT_TIMEOUT_MS")
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(DEFAULT_QDRANT_TIMEOUT_MS),
            ollama_base_url: env_override("EPIPHANY_OLLAMA_BASE_URL", "RAG_OLLAMA_BASE_URL")
                .unwrap_or_else(|| DEFAULT_OLLAMA_BASE_URL.to_string()),
            ollama_model: env_override("EPIPHANY_OLLAMA_MODEL", "RAG_OLLAMA_MODEL")
                .unwrap_or_else(|| DEFAULT_OLLAMA_MODEL.to_string()),
            ollama_timeout_ms: env_override("EPIPHANY_OLLAMA_TIMEOUT_MS", "RAG_OLLAMA_TIMEOUT_MS")
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or(DEFAULT_OLLAMA_TIMEOUT_MS),
            collection_name: env_override(
                "EPIPHANY_PERSONA_MEMORY_QDRANT_COLLECTION",
                "PERSONA_MEMORY_QDRANT_COLLECTION",
            )
            .unwrap_or_else(|| DEFAULT_COLLECTION.to_string()),
            query_instruction: env_override(
                "EPIPHANY_PERSONA_MEMORY_QUERY_INSTRUCTION",
                "PERSONA_MEMORY_QUERY_INSTRUCTION",
            )
            .unwrap_or_else(|| DEFAULT_QUERY_INSTRUCTION.to_string()),
        }
    }

    pub fn index_revision(&self) -> String {
        format!("qdrant-ollama-persona-memory-v1:{}", self.ollama_model)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaMemoryChunk {
    pub id: String,
    pub corpus_kind: String,
    pub identity_id: String,
    pub role_id: String,
    pub agent_id: String,
    pub persona_name: String,
    pub memory_kind: String,
    pub source_state_ref: String,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaMemorySearchHit {
    pub score: f32,
    pub chunk: PersonaMemoryChunk,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaMemoryCacheIndexReceipt {
    pub schema_version: String,
    pub status: String,
    pub collection_name: String,
    pub index_revision: String,
    pub identity_id: String,
    pub role_id: String,
    pub chunk_count: usize,
    pub private_state_exposed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaMemoryRecallRender {
    pub schema_version: String,
    pub status: String,
    pub cache_status: String,
    pub identity_id: String,
    pub role_id: String,
    pub chunk_count: usize,
    pub hit_count: usize,
    pub rendered_recall: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    pub private_state_exposed: bool,
}

pub fn build_persona_memory_chunks(
    entry: &EpiphanyAgentMemoryEntry,
    source_state_ref: impl Into<String>,
) -> Vec<PersonaMemoryChunk> {
    let source_state_ref = source_state_ref.into();
    let identity_id = entry.agent.agent_id.clone();
    let role_id = entry.role_id.clone();
    let agent_id = entry.agent.agent_id.clone();
    let persona_name = entry.agent.identity.name.clone();
    let mut chunks = Vec::new();

    push_chunk(
        &mut chunks,
        PersonaMemoryChunkInput {
            identity_id: &identity_id,
            role_id: &role_id,
            agent_id: &agent_id,
            persona_name: &persona_name,
            source_state_ref: &source_state_ref,
            id: "public-profile".to_string(),
            memory_kind: "public_profile".to_string(),
            text: join_fields([
                entry.agent.identity.name.as_str(),
                entry.agent.identity.public_description.as_str(),
                entry.agent.identity.origin.as_str(),
            ]),
        },
    );

    for value in &entry.agent.canonical_state.values {
        push_chunk(
            &mut chunks,
            PersonaMemoryChunkInput {
                identity_id: &identity_id,
                role_id: &role_id,
                agent_id: &agent_id,
                persona_name: &persona_name,
                source_state_ref: &source_state_ref,
                id: format!("value:{}", value.value_id),
                memory_kind: "value".to_string(),
                text: format!(
                    "{} priority={:.2} unforgivable_if_betrayed={}",
                    value.label, value.priority, value.unforgivable_if_betrayed
                ),
            },
        );
    }

    for goal in &entry.agent.goals {
        push_chunk(
            &mut chunks,
            PersonaMemoryChunkInput {
                identity_id: &identity_id,
                role_id: &role_id,
                agent_id: &agent_id,
                persona_name: &persona_name,
                source_state_ref: &source_state_ref,
                id: format!("goal:{}", goal.goal_id),
                memory_kind: format!("goal:{}", goal.status),
                text: join_fields([
                    goal.description.as_str(),
                    goal.scope.as_str(),
                    goal.emotional_stake.as_str(),
                    goal.blockers.join("; ").as_str(),
                ]),
            },
        );
    }

    for memory in &entry.agent.memories.semantic {
        push_memory_chunk(
            &mut chunks,
            &identity_id,
            &role_id,
            &agent_id,
            &persona_name,
            &source_state_ref,
            "semantic",
            memory,
        );
    }
    for memory in &entry.agent.memories.episodic {
        push_memory_chunk(
            &mut chunks,
            &identity_id,
            &role_id,
            &agent_id,
            &persona_name,
            &source_state_ref,
            "episodic",
            memory,
        );
    }
    for memory in &entry.agent.memories.relationship_summaries {
        push_memory_chunk(
            &mut chunks,
            &identity_id,
            &role_id,
            &agent_id,
            &persona_name,
            &source_state_ref,
            "relationship",
            memory,
        );
    }

    chunks
}

pub fn index_persona_memory_chunks(
    chunks: &[PersonaMemoryChunk],
    config: &PersonaMemoryCacheConfig,
) -> Result<PersonaMemoryCacheIndexReceipt> {
    let first = chunks
        .first()
        .context("cannot index Persona memory cache without chunks")?;
    let qdrant = PersonaQdrantClient::new(config)?;
    let ollama = PersonaOllamaEmbedder::new(config)?;
    let texts = chunks
        .iter()
        .map(|chunk| chunk.text.clone())
        .collect::<Vec<_>>();
    let embeddings = ollama.embed_documents(&texts)?;
    validate_embedding_batch(&embeddings, chunks.len())?;
    let vector_size = embeddings
        .first()
        .map(Vec::len)
        .context("embedding backend returned no vectors for Persona memory chunks")?;

    if !qdrant.collection_exists(&config.collection_name)? {
        qdrant.create_collection(&config.collection_name, vector_size, config)?;
    }
    qdrant.delete_identity_cache(&config.collection_name, &first.identity_id)?;

    let points = chunks
        .iter()
        .cloned()
        .zip(embeddings)
        .map(|(chunk, vector)| PersonaQdrantPointInput {
            id: stable_point_id(&chunk),
            vector,
            payload: chunk,
        })
        .collect::<Vec<_>>();
    qdrant.upsert_points(&config.collection_name, &points)?;

    Ok(PersonaMemoryCacheIndexReceipt {
        schema_version: PERSONA_MEMORY_CACHE_SCHEMA_VERSION.to_string(),
        status: "ok".to_string(),
        collection_name: config.collection_name.clone(),
        index_revision: config.index_revision(),
        identity_id: first.identity_id.clone(),
        role_id: first.role_id.clone(),
        chunk_count: chunks.len(),
        private_state_exposed: false,
    })
}

pub fn search_persona_memory_cache(
    identity_id: &str,
    query: &str,
    limit: usize,
    config: &PersonaMemoryCacheConfig,
) -> Result<Vec<PersonaMemorySearchHit>> {
    let qdrant = PersonaQdrantClient::new(config)?;
    let ollama = PersonaOllamaEmbedder::new(config)?;
    if !qdrant.collection_exists(&config.collection_name)? {
        return Ok(Vec::new());
    }
    let vector = ollama.embed_query(query)?;
    let hits = qdrant.query_points(&config.collection_name, &vector, identity_id, limit)?;
    Ok(hits
        .into_iter()
        .filter_map(|point| {
            point.payload.map(|chunk| PersonaMemorySearchHit {
                score: point.score,
                chunk,
            })
        })
        .collect())
}

pub fn render_persona_memory_recall_with_cache(
    entry: &EpiphanyAgentMemoryEntry,
    source_state_ref: impl Into<String>,
    query: &str,
    limit: usize,
    fallback_context: Option<&EpiphanyMemoryContextPacket>,
    config: &PersonaMemoryCacheConfig,
) -> PersonaMemoryRecallRender {
    let chunks = build_persona_memory_chunks(entry, source_state_ref);
    let identity_id = entry.agent.agent_id.clone();
    let role_id = entry.role_id.clone();
    if chunks.is_empty() {
        return fallback_persona_memory_recall(
            identity_id,
            role_id,
            0,
            "empty-cache-source".to_string(),
            "No public Persona memory chunks were available for cache recall.".to_string(),
            fallback_context,
        );
    }

    match index_persona_memory_chunks(&chunks, config)
        .and_then(|_| search_persona_memory_cache(&identity_id, query, limit.max(1), config))
    {
        Ok(hits) if !hits.is_empty() => PersonaMemoryRecallRender {
            schema_version: PERSONA_MEMORY_CACHE_SCHEMA_VERSION.to_string(),
            status: "ok".to_string(),
            cache_status: "qdrant-hit".to_string(),
            identity_id,
            role_id,
            chunk_count: chunks.len(),
            hit_count: hits.len(),
            rendered_recall: render_persona_memory_cache_hits(&hits),
            warnings: Vec::new(),
            private_state_exposed: false,
        },
        Ok(_) => fallback_persona_memory_recall(
            identity_id,
            role_id,
            chunks.len(),
            "qdrant-empty".to_string(),
            "Qdrant Persona-memory cache ran but returned no matching hits.".to_string(),
            fallback_context,
        ),
        Err(error) => fallback_persona_memory_recall(
            identity_id,
            role_id,
            chunks.len(),
            "qdrant-unavailable".to_string(),
            format!("Qdrant Persona-memory cache unavailable: {error:#}"),
            fallback_context,
        ),
    }
}

pub fn render_dynamic_persona_memory_recall_for_output(
    entry: &EpiphanyAgentMemoryEntry,
    source_state_ref: impl Into<String>,
    persona_prompt: &str,
    persona_output: &str,
    initial_recall_seed: &str,
    limit: usize,
    fallback_context: Option<&EpiphanyMemoryContextPacket>,
    config: &PersonaMemoryCacheConfig,
) -> PersonaMemoryRecallRender {
    let query = build_dynamic_persona_memory_recall_query(
        &entry.agent.agent_id,
        persona_prompt,
        persona_output,
        initial_recall_seed,
    );
    render_persona_memory_recall_with_cache(
        entry,
        source_state_ref,
        &query,
        limit,
        fallback_context,
        config,
    )
}

pub fn build_dynamic_persona_memory_recall_query(
    identity_id: &str,
    persona_prompt: &str,
    persona_output: &str,
    initial_recall_seed: &str,
) -> String {
    [
        format!("Current train of thought from {identity_id}:"),
        collapse_whitespace(persona_output, 6_000),
        "Original Persona turn pressure:".to_string(),
        extract_prompt_seed(persona_prompt),
        "Initial semantic memory recall seed:".to_string(),
        collapse_whitespace(initial_recall_seed, 2_000),
    ]
    .into_iter()
    .map(|part| part.trim().to_string())
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join("\n")
}

fn render_persona_memory_cache_hits(hits: &[PersonaMemorySearchHit]) -> String {
    let mut lines = vec![
        "These are derived Qdrant Persona-memory cache hits from typed Persona state. They are hints, not durable authority; typed memory remains the owner.".to_string(),
    ];
    for (index, hit) in hits.iter().enumerate() {
        lines.push(format!(
            "- {}. {} / {} score={:.3}: {}",
            index + 1,
            hit.chunk.persona_name,
            hit.chunk.memory_kind,
            hit.score,
            collapse_whitespace(&hit.chunk.text, 560)
        ));
    }
    lines.join("\n")
}

fn extract_prompt_seed(prompt: &str) -> String {
    let markers = [
        "Semantic memory recall:",
        "Projected inner state from Imagination:",
        "Recent home-repo activity",
        "Pending addressed pressure:",
        "Raw room transcript",
    ];
    let mut sections = Vec::new();

    for marker in markers {
        let Some(start) = prompt.find(marker) else {
            continue;
        };
        let end = markers
            .iter()
            .filter(|candidate| **candidate != marker)
            .filter_map(|candidate| {
                prompt[start + marker.len()..]
                    .find(candidate)
                    .map(|offset| start + marker.len() + offset)
            })
            .min()
            .unwrap_or_else(|| prompt.len().min(start + 2_400));
        sections.push(prompt[start..end.min(start + 2_400)].trim().to_string());
    }

    if sections.is_empty() {
        collapse_whitespace(prompt, 2_400)
    } else {
        sections.join("\n\n")
    }
}

fn fallback_persona_memory_recall(
    identity_id: String,
    role_id: String,
    chunk_count: usize,
    cache_status: String,
    warning: String,
    fallback_context: Option<&EpiphanyMemoryContextPacket>,
) -> PersonaMemoryRecallRender {
    let rendered_recall = if let Some(packet) = fallback_context {
        let fallback = render_persona_semantic_memory_recall(packet);
        format!("{warning}\nFalling back to typed memory graph context:\n{fallback}")
    } else {
        format!(
            "{warning}\n- semantic Persona memory recall unavailable; use projected state and direct room evidence only"
        )
    };

    PersonaMemoryRecallRender {
        schema_version: PERSONA_MEMORY_CACHE_SCHEMA_VERSION.to_string(),
        status: "fallback".to_string(),
        cache_status,
        identity_id,
        role_id,
        chunk_count,
        hit_count: 0,
        rendered_recall,
        warnings: vec![warning],
        private_state_exposed: false,
    }
}

struct PersonaMemoryChunkInput<'a> {
    identity_id: &'a str,
    role_id: &'a str,
    agent_id: &'a str,
    persona_name: &'a str,
    source_state_ref: &'a str,
    id: String,
    memory_kind: String,
    text: String,
}

fn push_memory_chunk(
    chunks: &mut Vec<PersonaMemoryChunk>,
    identity_id: &str,
    role_id: &str,
    agent_id: &str,
    persona_name: &str,
    source_state_ref: &str,
    memory_kind: &str,
    memory: &crate::GhostlightMemory,
) {
    push_chunk(
        chunks,
        PersonaMemoryChunkInput {
            identity_id,
            role_id,
            agent_id,
            persona_name,
            source_state_ref,
            id: format!("{memory_kind}:{}", memory.memory_id),
            memory_kind: memory_kind.to_string(),
            text: memory.summary.clone(),
        },
    );
}

fn push_chunk(chunks: &mut Vec<PersonaMemoryChunk>, input: PersonaMemoryChunkInput<'_>) {
    let text = collapse_whitespace(&input.text, 1800);
    if text.len() < 12 {
        return;
    }
    chunks.push(PersonaMemoryChunk {
        id: input.id,
        corpus_kind: PERSONA_MEMORY_CORPUS_KIND.to_string(),
        identity_id: input.identity_id.to_string(),
        role_id: input.role_id.to_string(),
        agent_id: input.agent_id.to_string(),
        persona_name: input.persona_name.to_string(),
        memory_kind: input.memory_kind,
        source_state_ref: input.source_state_ref.to_string(),
        text,
    });
}

fn join_fields<const N: usize>(fields: [&str; N]) -> String {
    fields
        .into_iter()
        .map(str::trim)
        .filter(|field| !field.is_empty())
        .collect::<Vec<_>>()
        .join("; ")
}

#[derive(Serialize)]
struct PersonaQdrantPointInput {
    id: String,
    vector: Vec<f32>,
    payload: PersonaMemoryChunk,
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
    points: Vec<PersonaQdrantQueryPoint>,
}

#[derive(Clone, Debug, Deserialize)]
struct PersonaQdrantQueryPoint {
    score: f32,
    #[serde(default)]
    payload: Option<PersonaMemoryChunk>,
}

#[derive(Debug, Deserialize)]
struct OllamaEmbedResponse {
    embeddings: Option<Vec<Vec<f32>>>,
}

struct PersonaQdrantClient {
    base_url: String,
    timeout_seconds: u64,
    client: Client,
}

impl PersonaQdrantClient {
    fn new(config: &PersonaMemoryCacheConfig) -> Result<Self> {
        let mut headers = HeaderMap::new();
        if let Some(api_key) = &config.qdrant_api_key
            && !api_key.is_empty()
        {
            headers.insert(
                "api-key",
                HeaderValue::from_str(api_key).context("invalid Qdrant api key")?,
            );
        }
        let client = ClientBuilder::new()
            .default_headers(headers)
            .timeout(Duration::from_millis(config.qdrant_timeout_ms))
            .build()
            .context("failed to build Persona memory Qdrant client")?;
        Ok(Self {
            base_url: normalize_base_url(&config.qdrant_url),
            timeout_seconds: timeout_seconds(config.qdrant_timeout_ms),
            client,
        })
    }

    fn collection_exists(&self, collection_name: &str) -> Result<bool> {
        let response = self
            .client
            .get(format!(
                "{}/collections/{collection_name}/exists",
                self.base_url
            ))
            .send()
            .with_context(|| {
                format!("failed to query Persona memory Qdrant collection {collection_name}")
            })?;
        let payload: QdrantEnvelope<QdrantCollectionExistsResult> =
            parse_qdrant_response(response)?;
        Ok(payload.result.exists)
    }

    fn create_collection(
        &self,
        collection_name: &str,
        vector_size: usize,
        config: &PersonaMemoryCacheConfig,
    ) -> Result<()> {
        let body = json!({
            "vectors": {
                "size": vector_size,
                "distance": "Cosine",
                "on_disk": true,
            },
            "on_disk_payload": true,
            "metadata": {
                "managedBy": "epiphany",
                "schemaVersion": PERSONA_MEMORY_CACHE_SCHEMA_VERSION,
                "indexRevision": config.index_revision(),
                "embeddingModel": config.ollama_model,
                "vectorSize": vector_size,
                "cacheAuthority": "rebuildable Persona memory cache; typed state remains owner",
            }
        });
        let response = self
            .client
            .put(format!("{}/collections/{collection_name}", self.base_url))
            .query(&[("timeout", self.timeout_seconds)])
            .json(&body)
            .send()
            .with_context(|| {
                format!("failed to create Persona memory Qdrant collection {collection_name}")
            })?;
        parse_qdrant_response::<Value>(response)?;
        Ok(())
    }

    fn delete_identity_cache(&self, collection_name: &str, identity_id: &str) -> Result<()> {
        let body = json!({
            "filter": {
                "must": [
                    { "key": "corpusKind", "match": { "value": PERSONA_MEMORY_CORPUS_KIND } },
                    { "key": "identityId", "match": { "value": identity_id } }
                ]
            }
        });
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
                format!("failed to clear Persona memory cache in {collection_name}")
            })?;
        parse_qdrant_response::<Value>(response)?;
        Ok(())
    }

    fn upsert_points(
        &self,
        collection_name: &str,
        points: &[PersonaQdrantPointInput],
    ) -> Result<()> {
        for batch in points.chunks(POINT_BATCH_SIZE) {
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
                .json(&json!({ "points": batch }))
                .send()
                .with_context(|| {
                    format!("failed to upsert Persona memory points into {collection_name}")
                })?;
            parse_qdrant_response::<Value>(response)?;
        }
        Ok(())
    }

    fn query_points(
        &self,
        collection_name: &str,
        query_vector: &[f32],
        identity_id: &str,
        limit: usize,
    ) -> Result<Vec<PersonaQdrantQueryPoint>> {
        let body = json!({
            "query": query_vector,
            "limit": limit,
            "with_payload": true,
            "with_vector": false,
            "filter": {
                "must": [
                    { "key": "corpusKind", "match": { "value": PERSONA_MEMORY_CORPUS_KIND } },
                    { "key": "identityId", "match": { "value": identity_id } }
                ]
            }
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
            .with_context(|| {
                format!("failed to query Persona memory Qdrant collection {collection_name}")
            })?;
        let payload: QdrantEnvelope<QdrantQueryResultEnvelope> = parse_qdrant_response(response)?;
        Ok(payload.result.points)
    }
}

struct PersonaOllamaEmbedder {
    base_url: String,
    model: String,
    query_instruction: String,
    client: Client,
}

impl PersonaOllamaEmbedder {
    fn new(config: &PersonaMemoryCacheConfig) -> Result<Self> {
        let client = ClientBuilder::new()
            .timeout(Duration::from_millis(config.ollama_timeout_ms))
            .build()
            .context("failed to build Persona memory Ollama client")?;
        Ok(Self {
            base_url: normalize_base_url(&config.ollama_base_url),
            model: config.ollama_model.clone(),
            query_instruction: config.query_instruction.clone(),
            client,
        })
    }

    fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        for batch in texts.chunks(EMBED_BATCH_SIZE) {
            embeddings.extend(self.embed_batch(batch)?);
        }
        Ok(embeddings)
    }

    fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
        let formatted = format!("Instruct: {}\nQuery: {}", self.query_instruction, query);
        let mut payload = self.embed_batch(&[formatted])?;
        payload
            .pop()
            .context("Ollama embedding backend returned no vector for Persona memory query")
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
                    "failed to contact Ollama Persona memory embedder at {} using model {}",
                    self.base_url, self.model
                )
            })?;
        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            anyhow::bail!("Ollama Persona memory embedding request failed with {status}: {body}");
        }
        let payload: OllamaEmbedResponse = response
            .json()
            .context("failed to decode Ollama Persona memory embedding response")?;
        payload
            .embeddings
            .context("Ollama Persona memory response did not include embeddings")
    }
}

fn parse_qdrant_response<T: for<'de> Deserialize<'de>>(
    response: reqwest::blocking::Response,
) -> Result<QdrantEnvelope<T>> {
    let status = response.status();
    if status == StatusCode::NOT_FOUND {
        anyhow::bail!("Qdrant Persona memory request returned not found");
    }
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        anyhow::bail!("Qdrant Persona memory request failed with {status}: {body}");
    }
    response
        .json()
        .context("failed to decode Qdrant Persona memory response JSON")
}

fn validate_embedding_batch(embeddings: &[Vec<f32>], expected_count: usize) -> Result<()> {
    if embeddings.len() != expected_count {
        anyhow::bail!(
            "Persona memory embedding backend returned {} vectors for {} chunks",
            embeddings.len(),
            expected_count
        );
    }
    let Some(vector_length) = embeddings.first().map(Vec::len) else {
        anyhow::bail!("Persona memory embedding backend returned no vectors");
    };
    if vector_length == 0 {
        anyhow::bail!("Persona memory embedding backend returned an empty vector");
    }
    for (index, embedding) in embeddings.iter().enumerate() {
        if embedding.len() != vector_length {
            anyhow::bail!(
                "Persona memory embedding backend returned inconsistent vector length at item {index}"
            );
        }
    }
    Ok(())
}

fn stable_point_id(chunk: &PersonaMemoryChunk) -> String {
    let mut hasher = Sha1::new();
    hasher.update(chunk.identity_id.as_bytes());
    hasher.update(b"\n");
    hasher.update(chunk.id.as_bytes());
    let digest = hasher.finalize();
    hex_lower(&digest[..16])
}

fn collapse_whitespace(value: &str, max_len: usize) -> String {
    let mut compacted = value.split_whitespace().collect::<Vec<_>>().join(" ");
    if compacted.len() > max_len {
        let keep = max_len.saturating_sub(3);
        compacted.truncate(keep);
        compacted.push_str("...");
    }
    compacted
}

fn normalize_base_url(value: &str) -> String {
    value.trim_end_matches('/').to_string()
}

fn timeout_seconds(timeout_ms: u64) -> u64 {
    (timeout_ms / 1000).max(1)
}

fn env_override(primary: &str, fallback: &str) -> Option<String> {
    std::env::var(primary)
        .ok()
        .or_else(|| std::env::var(fallback).ok())
        .filter(|value| !value.trim().is_empty())
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent_memory::GhostlightAgent;
    use crate::agent_memory::GhostlightCanonicalState;
    use crate::agent_memory::GhostlightGoal;
    use crate::agent_memory::GhostlightIdentity;
    use crate::agent_memory::GhostlightMemories;
    use crate::agent_memory::GhostlightMemory;
    use crate::agent_memory::GhostlightValue;
    use crate::agent_memory::GhostlightWorld;
    use epiphany_state_model::EpiphanyMemoryContextPacket;
    use epiphany_state_model::EpiphanyMemoryNode;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;
    use wiremock::matchers::method;
    use wiremock::matchers::path;

    #[test]
    fn persona_memory_chunks_exclude_private_notes() {
        let entry = persona_entry();
        let chunks = build_persona_memory_chunks(&entry, "state/agents.msgpack#Persona");
        let joined = chunks
            .iter()
            .map(|chunk| chunk.text.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(joined.contains("Clean typed contracts matter"));
        assert!(joined.contains("Public machine-saint"));
        assert!(!joined.contains("sealed private ache"));
        assert!(
            chunks
                .iter()
                .all(|chunk| chunk.corpus_kind == PERSONA_MEMORY_CORPUS_KIND)
        );
    }

    #[test]
    fn persona_memory_cache_indexes_and_searches_qdrant_with_identity_filter() -> Result<()> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let qdrant = runtime.block_on(MockServer::start());
        let ollama = runtime.block_on(MockServer::start());
        let config = PersonaMemoryCacheConfig {
            qdrant_url: qdrant.uri(),
            qdrant_api_key: None,
            qdrant_timeout_ms: 5_000,
            ollama_base_url: ollama.uri(),
            ollama_model: "qwen3-embedding:0.6b".to_string(),
            ollama_timeout_ms: 5_000,
            collection_name: "epiphany_persona_memory_test".to_string(),
            query_instruction: DEFAULT_QUERY_INSTRUCTION.to_string(),
        };
        let chunks = build_persona_memory_chunks(&persona_entry(), "state/agents.msgpack#Persona");

        runtime.block_on(async {
            Mock::given(method("GET"))
                .and(path("/collections/epiphany_persona_memory_test/exists"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "result": { "exists": false }
                })))
                .expect(1)
                .mount(&qdrant)
                .await;
            Mock::given(method("PUT"))
                .and(path("/collections/epiphany_persona_memory_test"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "result": true
                })))
                .expect(1)
                .mount(&qdrant)
                .await;
            Mock::given(method("POST"))
                .and(path("/collections/epiphany_persona_memory_test/points/delete"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "result": { "status": "acknowledged" }
                })))
                .expect(1)
                .mount(&qdrant)
                .await;
            Mock::given(method("PUT"))
                .and(path("/collections/epiphany_persona_memory_test/points"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "result": { "status": "acknowledged" }
                })))
                .expect(1)
                .mount(&qdrant)
                .await;
            Mock::given(method("POST"))
                .and(path("/api/embed"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "embeddings": chunks.iter().map(|_| vec![0.1_f32, 0.2_f32, 0.3_f32]).collect::<Vec<_>>()
                })))
                .expect(1)
                .mount(&ollama)
                .await;
        });

        let receipt = index_persona_memory_chunks(&chunks, &config)?;
        assert_eq!(receipt.status, "ok");
        assert_eq!(receipt.chunk_count, chunks.len());
        assert!(!receipt.private_state_exposed);

        runtime.block_on(async {
            qdrant.reset().await;
            ollama.reset().await;
            Mock::given(method("GET"))
                .and(path("/collections/epiphany_persona_memory_test/exists"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "result": { "exists": true }
                })))
                .expect(1)
                .mount(&qdrant)
                .await;
            Mock::given(method("POST"))
                .and(path("/api/embed"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "embeddings": [[0.1_f32, 0.2_f32, 0.3_f32]]
                })))
                .expect(1)
                .mount(&ollama)
                .await;
            Mock::given(method("POST"))
                .and(path(
                    "/collections/epiphany_persona_memory_test/points/query",
                ))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "result": {
                        "points": [{
                            "score": 0.93,
                            "payload": chunks[0],
                        }]
                    }
                })))
                .expect(1)
                .mount(&qdrant)
                .await;
        });

        let hits = search_persona_memory_cache("epiphany.Persona", "clean contracts", 4, &config)?;
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].chunk.identity_id, "epiphany.Persona");
        assert!(hits[0].chunk.text.contains("Public machine-saint"));
        Ok(())
    }

    #[test]
    fn persona_memory_recall_bridge_renders_qdrant_hits() -> Result<()> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let qdrant = runtime.block_on(MockServer::start());
        let ollama = runtime.block_on(MockServer::start());
        let config = PersonaMemoryCacheConfig {
            qdrant_url: qdrant.uri(),
            qdrant_api_key: None,
            qdrant_timeout_ms: 5_000,
            ollama_base_url: ollama.uri(),
            ollama_model: "qwen3-embedding:0.6b".to_string(),
            ollama_timeout_ms: 5_000,
            collection_name: "epiphany_persona_memory_bridge_test".to_string(),
            query_instruction: DEFAULT_QUERY_INSTRUCTION.to_string(),
        };
        let chunks = build_persona_memory_chunks(&persona_entry(), "state/agents.msgpack#Persona");

        runtime.block_on(async {
            Mock::given(method("GET"))
                .and(path("/collections/epiphany_persona_memory_bridge_test/exists"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "result": { "exists": true }
                })))
                .expect(2)
                .mount(&qdrant)
                .await;
            Mock::given(method("POST"))
                .and(path("/collections/epiphany_persona_memory_bridge_test/points/delete"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "result": { "status": "acknowledged" }
                })))
                .expect(1)
                .mount(&qdrant)
                .await;
            Mock::given(method("PUT"))
                .and(path("/collections/epiphany_persona_memory_bridge_test/points"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "result": { "status": "acknowledged" }
                })))
                .expect(1)
                .mount(&qdrant)
                .await;
            Mock::given(method("POST"))
                .and(path("/collections/epiphany_persona_memory_bridge_test/points/query"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "result": {
                        "points": [{
                            "score": 0.97,
                            "payload": chunks[0],
                        }]
                    }
                })))
                .expect(1)
                .mount(&qdrant)
                .await;
            Mock::given(method("POST"))
                .and(path("/api/embed"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "embeddings": chunks.iter().map(|_| vec![0.1_f32, 0.2_f32, 0.3_f32]).collect::<Vec<_>>()
                })))
                .expect(2)
                .mount(&ollama)
                .await;
        });

        let render = render_persona_memory_recall_with_cache(
            &persona_entry(),
            "state/agents.msgpack#Persona",
            "clean contracts",
            4,
            None,
            &config,
        );
        assert_eq!(render.status, "ok");
        assert_eq!(render.cache_status, "qdrant-hit");
        assert!(
            render
                .rendered_recall
                .contains("Qdrant Persona-memory cache hits")
        );
        assert!(render.rendered_recall.contains("Public machine-saint"));
        assert!(!render.rendered_recall.contains("sealed private ache"));
        Ok(())
    }

    #[test]
    fn persona_memory_recall_bridge_falls_back_to_memory_graph_context() -> Result<()> {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let qdrant = runtime.block_on(MockServer::start());
        let ollama = runtime.block_on(MockServer::start());
        let config = PersonaMemoryCacheConfig {
            qdrant_url: qdrant.uri(),
            qdrant_api_key: None,
            qdrant_timeout_ms: 5_000,
            ollama_base_url: ollama.uri(),
            ollama_model: "qwen3-embedding:0.6b".to_string(),
            ollama_timeout_ms: 5_000,
            collection_name: "epiphany_persona_memory_bridge_fallback".to_string(),
            query_instruction: DEFAULT_QUERY_INSTRUCTION.to_string(),
        };
        let chunks = build_persona_memory_chunks(&persona_entry(), "state/agents.msgpack#Persona");

        runtime.block_on(async {
            Mock::given(method("POST"))
                .and(path("/api/embed"))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "embeddings": chunks.iter().map(|_| vec![0.1_f32, 0.2_f32, 0.3_f32]).collect::<Vec<_>>()
                })))
                .expect(1)
                .mount(&ollama)
                .await;
            Mock::given(method("GET"))
                .and(path(
                    "/collections/epiphany_persona_memory_bridge_fallback/exists",
                ))
                .respond_with(ResponseTemplate::new(500).set_body_json(json!({
                    "status": "unavailable"
                })))
                .expect(1)
                .mount(&qdrant)
                .await;
        });

        let fallback = EpiphanyMemoryContextPacket {
            id: "memctx-fallback".to_string(),
            query_id: "persona-fallback".to_string(),
            nodes: vec![EpiphanyMemoryNode {
                id: "node-fallback".to_string(),
                title: "Fallback typed memory".to_string(),
                claim: "Typed memory graph recall remains available when Qdrant is down."
                    .to_string(),
                action_implication: "Use fallback context without pretending cache was live."
                    .to_string(),
                ..Default::default()
            }],
            ..Default::default()
        };

        let render = render_persona_memory_recall_with_cache(
            &persona_entry(),
            "state/agents.msgpack#Persona",
            "clean contracts",
            4,
            Some(&fallback),
            &config,
        );
        assert_eq!(render.status, "fallback");
        assert_eq!(render.cache_status, "qdrant-unavailable");
        assert!(
            render
                .rendered_recall
                .contains("Falling back to typed memory graph context")
        );
        assert!(
            render
                .rendered_recall
                .contains("Typed memory graph recall remains available")
        );
        assert!(!render.private_state_exposed);
        Ok(())
    }

    fn persona_entry() -> EpiphanyAgentMemoryEntry {
        EpiphanyAgentMemoryEntry {
            schema_version: "ghostlight.agent_state.v0".to_string(),
            role_id: "Persona".to_string(),
            world: GhostlightWorld::default(),
            agent: GhostlightAgent {
                agent_id: "epiphany.Persona".to_string(),
                identity: GhostlightIdentity {
                    name: "Epiphany".to_string(),
                    roles: vec!["Persona".to_string()],
                    origin: "EpiphanyAgent".to_string(),
                    public_description: "Public machine-saint for typed agency.".to_string(),
                    private_notes: vec!["sealed private ache".to_string()],
                },
                memories: GhostlightMemories {
                    semantic: vec![GhostlightMemory {
                        memory_id: "semantic-1".to_string(),
                        summary: "Clean typed contracts matter more than improvised glue."
                            .to_string(),
                        salience: 0.9,
                        confidence: 0.95,
                        ..Default::default()
                    }],
                    ..Default::default()
                },
                goals: vec![GhostlightGoal {
                    goal_id: "goal-1".to_string(),
                    description: "Make Persona recall semantically useful.".to_string(),
                    scope: "role".to_string(),
                    priority: 0.8,
                    emotional_stake: "coherence".to_string(),
                    status: "active".to_string(),
                    ..Default::default()
                }],
                canonical_state: GhostlightCanonicalState {
                    values: vec![GhostlightValue {
                        value_id: "value-1".to_string(),
                        label: "Clean typed contracts matter.".to_string(),
                        priority: 0.9,
                        unforgivable_if_betrayed: true,
                    }],
                    ..Default::default()
                },
                ..Default::default()
            },
            relationships: Vec::new(),
            events: Vec::new(),
            scenes: Vec::new(),
        }
    }
}
