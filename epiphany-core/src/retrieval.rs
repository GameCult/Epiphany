use anyhow::Context;
use anyhow::Result;
use bm25::Document;
use bm25::Language;
use bm25::SearchEngineBuilder;
use epiphany_state_model::EpiphanyRetrievalShardSummary;
use epiphany_state_model::EpiphanyRetrievalState;
use epiphany_state_model::EpiphanyRetrievalStatus;
use ignore::WalkBuilder;
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
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use uuid::Uuid;

pub const EPIPHANY_RETRIEVAL_DEFAULT_LIMIT: usize = 10;
pub const EPIPHANY_RETRIEVAL_MAX_LIMIT: usize = 50;

const EXACT_SEARCH_CANDIDATE_MULTIPLIER: usize = 5;
const EXACT_SEARCH_MAX_CANDIDATES: usize = 250;
const QUERY_TIME_INDEX_REVISION: &str = "query-time-bm25-v1";
const QDRANT_INDEX_REVISION_PREFIX: &str = "qdrant-ollama-v1";
const SEMANTIC_MANIFEST_SCHEMA_VERSION: u32 = 1;
const MAX_TEXT_FILE_BYTES: u64 = 256 * 1024;
const CHUNK_LINE_COUNT: usize = 24;
const CHUNK_LINE_OVERLAP: usize = 8;
const WORKSPACE_SHARD_ID: &str = "workspace";
const MANIFESTS_DIR: &str = "epiphany/retrieval/manifests";
const QDRANT_COLLECTION_PREFIX: &str = "epiphany_workspace";
const QDRANT_POINT_BATCH_SIZE: usize = 128;
const OLLAMA_EMBED_BATCH_SIZE: usize = 32;
const DEFAULT_QDRANT_URL: &str = "http://127.0.0.1:6333";
const DEFAULT_QDRANT_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_OLLAMA_BASE_URL: &str = "http://127.0.0.1:11434";
const DEFAULT_OLLAMA_MODEL: &str = "qwen3-embedding:0.6b";
const DEFAULT_OLLAMA_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_QUERY_INSTRUCTION: &str = "Given a source-tree or codebase question, retrieve relevant files and code snippets that answer it.";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpiphanyRetrieveQuery {
    pub query: String,
    pub limit: usize,
    pub path_prefixes: Vec<PathBuf>,
}

impl EpiphanyRetrieveQuery {
    pub fn new(query: String) -> Self {
        Self {
            query,
            limit: EPIPHANY_RETRIEVAL_DEFAULT_LIMIT,
            path_prefixes: Vec::new(),
        }
    }
}

pub fn normalize_epiphany_retrieve_query(
    query: String,
    limit: Option<u32>,
    path_prefixes: Vec<PathBuf>,
) -> std::result::Result<EpiphanyRetrieveQuery, &'static str> {
    let query = query.trim().to_string();
    if query.is_empty() {
        return Err("query must not be empty");
    }
    if matches!(limit, Some(0)) {
        return Err("limit must be greater than zero");
    }
    let limit = limit
        .map(|value| value as usize)
        .unwrap_or(EPIPHANY_RETRIEVAL_DEFAULT_LIMIT)
        .clamp(1, EPIPHANY_RETRIEVAL_MAX_LIMIT);
    Ok(EpiphanyRetrieveQuery {
        query,
        limit,
        path_prefixes,
    })
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EpiphanyRetrieveResultKind {
    ExactFile,
    ExactDirectory,
    SemanticChunk,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EpiphanyRetrieveResult {
    pub kind: EpiphanyRetrieveResultKind,
    pub path: PathBuf,
    pub score: f32,
    pub line_start: Option<u32>,
    pub line_end: Option<u32>,
    pub excerpt: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EpiphanyRetrieveResponse {
    pub query: String,
    pub index_summary: EpiphanyRetrievalState,
    pub results: Vec<EpiphanyRetrieveResult>,
}

pub fn retrieval_state_for_workspace(
    workspace_root: &Path,
    codex_home: &Path,
) -> EpiphanyRetrievalState {
    if !workspace_root.is_dir() {
        return unavailable_retrieval_state(workspace_root);
    }

    let config = SemanticBackendConfig::from_env();
    let manifest_path = config.manifest_path(codex_home, workspace_root);
    let Ok(Some(manifest)) = load_semantic_manifest(&manifest_path) else {
        return baseline_retrieval_state(workspace_root);
    };

    if manifest.index_revision != config.index_revision()
        || manifest.collection_name != config.collection_name(workspace_root)
    {
        return manifest_to_retrieval_state(
            workspace_root,
            &manifest,
            EpiphanyRetrievalStatus::Stale,
            Vec::new(),
        );
    }

    match collect_workspace_index_snapshot(workspace_root) {
        Ok(snapshot) => {
            let dirty_paths = dirty_paths_from_manifest(&manifest, &snapshot);
            let status = if dirty_paths.is_empty() {
                EpiphanyRetrievalStatus::Ready
            } else {
                EpiphanyRetrievalStatus::Stale
            };
            manifest_to_retrieval_state(workspace_root, &manifest, status, dirty_paths)
        }
        Err(_) => manifest_to_retrieval_state(
            workspace_root,
            &manifest,
            EpiphanyRetrievalStatus::Stale,
            Vec::new(),
        ),
    }
}

pub fn index_workspace(
    workspace_root: &Path,
    codex_home: &Path,
    force_full_rebuild: bool,
) -> Result<EpiphanyRetrievalState> {
    if !workspace_root.is_dir() {
        anyhow::bail!(
            "workspace root does not exist: {}",
            workspace_root.display()
        );
    }

    let config = SemanticBackendConfig::from_env();
    index_workspace_with_config(workspace_root, codex_home, force_full_rebuild, &config)
}

pub fn retrieve_workspace(
    workspace_root: &Path,
    codex_home: &Path,
    query: EpiphanyRetrieveQuery,
) -> Result<EpiphanyRetrieveResponse> {
    if !workspace_root.is_dir() {
        anyhow::bail!(
            "workspace root does not exist: {}",
            workspace_root.display()
        );
    }

    let query_text = query.query.trim();
    if query_text.is_empty() {
        anyhow::bail!("query must not be empty");
    }

    let limit = query.limit.clamp(1, EPIPHANY_RETRIEVAL_MAX_LIMIT);
    let path_prefixes = normalize_path_prefixes(workspace_root, &query.path_prefixes);

    let exact_results = run_exact_search(workspace_root, query_text, limit, &path_prefixes)
        .context("exact retrieval failed")?;

    let semantic_limit = semantic_candidate_limit(limit, &path_prefixes);
    let persistent_search = match search_persistent_semantic_chunks(
        workspace_root,
        codex_home,
        query_text,
        semantic_limit,
    ) {
        Ok(search) => search,
        Err(err) => {
            tracing::warn!(
                workspace_root = %workspace_root.display(),
                error = %err,
                "Epiphany persistent semantic search failed; falling back to query-time BM25"
            );
            PersistentSemanticSearch::Fallback {
                summary: stale_summary_from_manifest(workspace_root, codex_home),
            }
        }
    };

    let (index_summary, semantic_results) = match persistent_search {
        PersistentSemanticSearch::Ready { summary, results } => (
            summary,
            results
                .into_iter()
                .filter(|result| matches_path_prefixes(&result.path, &path_prefixes))
                .take(limit)
                .collect::<Vec<_>>(),
        ),
        PersistentSemanticSearch::Fallback { summary } => {
            let semantic_corpus = collect_semantic_corpus(workspace_root, &path_prefixes)
                .context("semantic corpus collection failed")?;
            let semantic_results =
                search_semantic_chunks(query_text, semantic_limit, &semantic_corpus);
            (
                summary.unwrap_or_else(|| {
                    query_time_retrieval_state(
                        workspace_root,
                        semantic_corpus.searchable_file_count,
                        semantic_corpus.chunks.len(),
                    )
                }),
                semantic_results,
            )
        }
        PersistentSemanticSearch::None => {
            let semantic_corpus = collect_semantic_corpus(workspace_root, &path_prefixes)
                .context("semantic corpus collection failed")?;
            let semantic_results =
                search_semantic_chunks(query_text, semantic_limit, &semantic_corpus);
            (
                query_time_retrieval_state(
                    workspace_root,
                    semantic_corpus.searchable_file_count,
                    semantic_corpus.chunks.len(),
                ),
                semantic_results,
            )
        }
    };

    let mut results = Vec::with_capacity(exact_results.len() + semantic_results.len());
    results.extend(exact_results);
    results.extend(semantic_results);
    sort_results(&mut results);
    results.truncate(limit);

    Ok(EpiphanyRetrieveResponse {
        query: query_text.to_string(),
        index_summary,
        results,
    })
}

fn index_workspace_with_config(
    workspace_root: &Path,
    codex_home: &Path,
    force_full_rebuild: bool,
    config: &SemanticBackendConfig,
) -> Result<EpiphanyRetrievalState> {
    let manifest_path = config.manifest_path(codex_home, workspace_root);
    let snapshot = collect_workspace_index_snapshot(workspace_root)?;
    let existing_manifest = load_semantic_manifest(&manifest_path)?;
    let expected_collection = config.collection_name(workspace_root);
    let qdrant = QdrantClient::new(config.qdrant.clone())?;

    let manifest_compatible = existing_manifest.as_ref().is_some_and(|manifest| {
        manifest.index_revision == config.index_revision()
            && manifest.collection_name == expected_collection
    });
    let collection_exists = qdrant.collection_exists(&expected_collection)?;

    let full_rebuild = force_full_rebuild || !manifest_compatible || !collection_exists;
    if full_rebuild {
        if let Some(manifest) = existing_manifest.as_ref()
            && manifest.collection_name != expected_collection
        {
            qdrant.delete_collection(&manifest.collection_name)?;
        }
        if collection_exists {
            qdrant.delete_collection(&expected_collection)?;
        }
    }

    if snapshot.files.is_empty() {
        let manifest = SemanticIndexManifest {
            schema_version: SEMANTIC_MANIFEST_SCHEMA_VERSION,
            workspace_root: workspace_root.to_path_buf(),
            collection_name: expected_collection,
            index_revision: config.index_revision(),
            indexed_at_unix_seconds: unix_timestamp_seconds(SystemTime::now())?,
            files: Vec::new(),
        };
        write_semantic_manifest(&manifest_path, &manifest)?;
        return Ok(manifest_to_retrieval_state(
            workspace_root,
            &manifest,
            EpiphanyRetrievalStatus::Ready,
            Vec::new(),
        ));
    }

    let (paths_to_reindex, deleted_point_ids, carried_files) = if full_rebuild {
        (
            snapshot.files.keys().cloned().collect::<BTreeSet<_>>(),
            Vec::new(),
            BTreeMap::new(),
        )
    } else {
        compute_incremental_plan(existing_manifest.as_ref(), &snapshot)
    };

    let indexed_files = collect_indexed_files(workspace_root, &paths_to_reindex)?;

    let mut new_manifest_files = carried_files;
    for indexed_file in &indexed_files {
        let snapshot_file = snapshot.files.get(&indexed_file.path).with_context(|| {
            format!("missing index snapshot for {}", indexed_file.path.display())
        })?;
        new_manifest_files.insert(
            indexed_file.path.clone(),
            SemanticIndexedFile {
                path: indexed_file.path.clone(),
                size_bytes: snapshot_file.size_bytes,
                modified_unix_nanos: snapshot_file.modified_unix_nanos,
                chunk_count: indexed_file.chunks.len() as u32,
                point_ids: indexed_file
                    .chunks
                    .iter()
                    .map(|chunk| chunk_point_id(&expected_collection, chunk))
                    .collect(),
            },
        );
    }

    if !deleted_point_ids.is_empty() {
        qdrant.delete_points(&expected_collection, &deleted_point_ids)?;
    }

    if !indexed_files.is_empty() {
        let ollama = OllamaEmbedder::new(config.ollama.clone())?;
        let documents = indexed_files
            .iter()
            .flat_map(|file| file.chunks.iter().cloned())
            .collect::<Vec<_>>();
        let texts = documents
            .iter()
            .map(|chunk| chunk.search_text.clone())
            .collect::<Vec<_>>();
        let embeddings = ollama.embed_documents(&texts)?;
        validate_embedding_batch(&embeddings, texts.len())?;

        if full_rebuild {
            let vector_size = embeddings
                .first()
                .map(Vec::len)
                .context("embedding backend returned no vectors for rebuild")?;
            qdrant.create_collection(
                &expected_collection,
                vector_size,
                &config.index_revision(),
                workspace_root,
                &config.ollama.model,
            )?;
        }

        let points = documents
            .into_iter()
            .zip(embeddings)
            .map(|(chunk, vector)| QdrantPointInput {
                id: chunk_point_id(&expected_collection, &chunk),
                vector,
                payload: json!({
                    "path": normalize_payload_path(&chunk.path),
                    "line_start": chunk.line_start,
                    "line_end": chunk.line_end,
                    "excerpt": chunk.excerpt,
                }),
            })
            .collect::<Vec<_>>();
        qdrant.upsert_points(&expected_collection, &points)?;
    }

    let manifest = SemanticIndexManifest {
        schema_version: SEMANTIC_MANIFEST_SCHEMA_VERSION,
        workspace_root: workspace_root.to_path_buf(),
        collection_name: expected_collection,
        index_revision: config.index_revision(),
        indexed_at_unix_seconds: unix_timestamp_seconds(SystemTime::now())?,
        files: new_manifest_files.into_values().collect(),
    };
    write_semantic_manifest(&manifest_path, &manifest)?;

    Ok(manifest_to_retrieval_state(
        workspace_root,
        &manifest,
        EpiphanyRetrievalStatus::Ready,
        Vec::new(),
    ))
}

fn search_persistent_semantic_chunks(
    workspace_root: &Path,
    codex_home: &Path,
    query: &str,
    limit: usize,
) -> Result<PersistentSemanticSearch> {
    let config = SemanticBackendConfig::from_env();
    let manifest_path = config.manifest_path(codex_home, workspace_root);
    let Some(manifest) = load_semantic_manifest(&manifest_path)? else {
        return Ok(PersistentSemanticSearch::None);
    };

    if manifest.index_revision != config.index_revision()
        || manifest.collection_name != config.collection_name(workspace_root)
    {
        return Ok(PersistentSemanticSearch::Fallback {
            summary: Some(manifest_to_retrieval_state(
                workspace_root,
                &manifest,
                EpiphanyRetrievalStatus::Stale,
                Vec::new(),
            )),
        });
    }

    let snapshot = collect_workspace_index_snapshot(workspace_root)?;
    let dirty_paths = dirty_paths_from_manifest(&manifest, &snapshot);
    if !dirty_paths.is_empty() {
        return Ok(PersistentSemanticSearch::Fallback {
            summary: Some(manifest_to_retrieval_state(
                workspace_root,
                &manifest,
                EpiphanyRetrievalStatus::Stale,
                dirty_paths,
            )),
        });
    }

    let qdrant = QdrantClient::new(config.qdrant.clone())?;
    if !qdrant.collection_exists(&manifest.collection_name)? {
        return Ok(PersistentSemanticSearch::Fallback {
            summary: Some(manifest_to_retrieval_state(
                workspace_root,
                &manifest,
                EpiphanyRetrievalStatus::Stale,
                Vec::new(),
            )),
        });
    }

    let ollama = OllamaEmbedder::new(config.ollama.clone())?;
    let query_vector = ollama.embed_query(query)?;
    let search_hits = qdrant.query_points(&manifest.collection_name, &query_vector, limit)?;

    let results = search_hits
        .into_iter()
        .filter_map(map_qdrant_hit_to_result)
        .collect::<Vec<_>>();
    let summary = manifest_to_retrieval_state(
        workspace_root,
        &manifest,
        EpiphanyRetrievalStatus::Ready,
        Vec::new(),
    );

    Ok(PersistentSemanticSearch::Ready { summary, results })
}

fn stale_summary_from_manifest(
    workspace_root: &Path,
    codex_home: &Path,
) -> Option<EpiphanyRetrievalState> {
    let config = SemanticBackendConfig::from_env();
    let manifest_path = config.manifest_path(codex_home, workspace_root);
    let manifest = load_semantic_manifest(&manifest_path).ok().flatten()?;
    Some(manifest_to_retrieval_state(
        workspace_root,
        &manifest,
        EpiphanyRetrievalStatus::Stale,
        Vec::new(),
    ))
}

fn run_exact_search(
    workspace_root: &Path,
    query: &str,
    limit: usize,
    path_prefixes: &[PathBuf],
) -> Result<Vec<EpiphanyRetrieveResult>> {
    let query_terms = exact_search_terms(query);
    if query_terms.is_empty() {
        return Ok(Vec::new());
    }

    let mut walker = WalkBuilder::new(workspace_root);
    walker.hidden(false).follow_links(true).require_git(true);

    let mut results = walker
        .build()
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let relative_path = entry
                .path()
                .strip_prefix(workspace_root)
                .ok()?
                .to_path_buf();
            if relative_path.as_os_str().is_empty() || should_skip_relative_path(&relative_path) {
                return None;
            }
            if !matches_path_prefixes(&relative_path, path_prefixes) {
                return None;
            }
            let file_type = entry.file_type()?;
            let kind = if file_type.is_file() {
                EpiphanyRetrieveResultKind::ExactFile
            } else if file_type.is_dir() {
                EpiphanyRetrieveResultKind::ExactDirectory
            } else {
                return None;
            };
            let score = score_exact_path_match(&relative_path, &query_terms)?;

            Some(EpiphanyRetrieveResult {
                kind,
                path: relative_path,
                score,
                line_start: None,
                line_end: None,
                excerpt: None,
            })
        })
        .collect::<Vec<_>>();

    sort_results(&mut results);
    results.truncate(limit);
    Ok(results)
}

fn exact_search_terms(query: &str) -> Vec<String> {
    query
        .split(|character: char| !character.is_alphanumeric())
        .filter(|term| !term.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn score_exact_path_match(relative_path: &Path, query_terms: &[String]) -> Option<f32> {
    let normalized_path = normalize_payload_path(relative_path).to_lowercase();
    if !query_terms
        .iter()
        .all(|term| normalized_path.contains(term))
    {
        return None;
    }

    let file_name = relative_path
        .file_name()
        .map(|name| name.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let term_score = query_terms.iter().fold(0.0, |score, term| {
        score
            + if file_name.contains(term) { 30.0 } else { 0.0 }
            + if normalized_path.contains(term) {
                20.0
            } else {
                0.0
            }
    });
    let joined_query = query_terms.join(" ");
    let joined_with_underscores = query_terms.join("_");
    let joined_with_hyphens = query_terms.join("-");
    let phrase_score = if file_name == joined_query
        || file_name == joined_with_underscores
        || file_name == joined_with_hyphens
    {
        200.0
    } else if file_name.contains(&joined_query)
        || file_name.contains(&joined_with_underscores)
        || file_name.contains(&joined_with_hyphens)
    {
        100.0
    } else if normalized_path.contains(&joined_query)
        || normalized_path.contains(&joined_with_underscores)
        || normalized_path.contains(&joined_with_hyphens)
    {
        50.0
    } else {
        0.0
    };
    let depth_penalty = relative_path.components().count() as f32;
    Some(term_score + phrase_score - depth_penalty)
}

#[derive(Clone, Debug)]
struct SemanticChunk {
    path: PathBuf,
    line_start: u32,
    line_end: u32,
    excerpt: String,
    search_text: String,
}

struct SemanticCorpus {
    searchable_file_count: usize,
    chunks: Vec<SemanticChunk>,
}

struct WorkspaceIndexSnapshot {
    files: BTreeMap<PathBuf, WorkspaceIndexSnapshotFile>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct WorkspaceIndexSnapshotFile {
    size_bytes: u64,
    modified_unix_nanos: i64,
    chunk_count: u32,
}

#[derive(Clone, Debug)]
struct IndexedFile {
    path: PathBuf,
    chunks: Vec<SemanticChunk>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SemanticIndexManifest {
    schema_version: u32,
    workspace_root: PathBuf,
    collection_name: String,
    index_revision: String,
    indexed_at_unix_seconds: i64,
    #[serde(default)]
    files: Vec<SemanticIndexedFile>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct SemanticIndexedFile {
    path: PathBuf,
    size_bytes: u64,
    modified_unix_nanos: i64,
    chunk_count: u32,
    #[serde(default)]
    point_ids: Vec<String>,
}

enum PersistentSemanticSearch {
    Ready {
        summary: EpiphanyRetrievalState,
        results: Vec<EpiphanyRetrieveResult>,
    },
    Fallback {
        summary: Option<EpiphanyRetrievalState>,
    },
    None,
}

#[derive(Clone, Debug)]
struct SemanticBackendConfig {
    qdrant: QdrantConfig,
    ollama: OllamaConfig,
}

impl SemanticBackendConfig {
    fn from_env() -> Self {
        Self {
            qdrant: QdrantConfig {
                url: env_override("EPIPHANY_QDRANT_URL", "QDRANT_URL")
                    .unwrap_or_else(|| DEFAULT_QDRANT_URL.to_string()),
                api_key: env_override("EPIPHANY_QDRANT_API_KEY", "QDRANT_API_KEY"),
                timeout_ms: env_override("EPIPHANY_QDRANT_TIMEOUT_MS", "QDRANT_TIMEOUT_MS")
                    .and_then(|value| value.parse::<u64>().ok())
                    .unwrap_or(DEFAULT_QDRANT_TIMEOUT_MS),
            },
            ollama: OllamaConfig {
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
                .unwrap_or_else(|| DEFAULT_QUERY_INSTRUCTION.to_string()),
            },
        }
    }

    fn index_revision(&self) -> String {
        format!("{QDRANT_INDEX_REVISION_PREFIX}:{}", self.ollama.model)
    }

    fn collection_name(&self, workspace_root: &Path) -> String {
        let mut hasher = Sha1::new();
        hasher.update(workspace_root.to_string_lossy().as_bytes());
        hasher.update(b"\n");
        hasher.update(self.index_revision().as_bytes());
        let digest = hasher.finalize();
        format!("{QDRANT_COLLECTION_PREFIX}_{}", hex_lower(&digest[..8]))
    }

    fn manifest_path(&self, codex_home: &Path, workspace_root: &Path) -> PathBuf {
        let mut hasher = Sha1::new();
        hasher.update(workspace_root.to_string_lossy().as_bytes());
        let digest = hasher.finalize();
        codex_home
            .join(MANIFESTS_DIR)
            .join(format!("{}.json", hex_lower(&digest[..10])))
    }
}

#[derive(Clone, Debug)]
struct QdrantConfig {
    url: String,
    api_key: Option<String>,
    timeout_ms: u64,
}

#[derive(Clone, Debug)]
struct OllamaConfig {
    base_url: String,
    model: String,
    timeout_ms: u64,
    query_instruction: String,
}

struct QdrantClient {
    base_url: String,
    timeout_seconds: u64,
    client: Client,
}

impl QdrantClient {
    fn new(config: QdrantConfig) -> Result<Self> {
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

    fn collection_exists(&self, collection_name: &str) -> Result<bool> {
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

    fn delete_collection(&self, collection_name: &str) -> Result<()> {
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

    fn create_collection(
        &self,
        collection_name: &str,
        vector_size: usize,
        index_revision: &str,
        workspace_root: &Path,
        embedding_model: &str,
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
                "schemaVersion": SEMANTIC_MANIFEST_SCHEMA_VERSION,
                "indexRevision": index_revision,
                "workspaceRoot": workspace_root.to_string_lossy(),
                "embeddingModel": embedding_model,
                "vectorSize": vector_size,
            }
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

    fn upsert_points(&self, collection_name: &str, points: &[QdrantPointInput]) -> Result<()> {
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

    fn delete_points(&self, collection_name: &str, point_ids: &[String]) -> Result<()> {
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

    fn query_points(
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

struct OllamaEmbedder {
    base_url: String,
    model: String,
    query_instruction: String,
    client: Client,
}

impl OllamaEmbedder {
    fn new(config: OllamaConfig) -> Result<Self> {
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

    fn embed_documents(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        for batch in texts.chunks(OLLAMA_EMBED_BATCH_SIZE) {
            let payload = self.embed_batch(batch)?;
            embeddings.extend(payload);
        }
        Ok(embeddings)
    }

    fn embed_query(&self, query: &str) -> Result<Vec<f32>> {
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
struct QdrantPointInput {
    id: String,
    vector: Vec<f32>,
    payload: Value,
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

#[derive(Clone, Debug, Deserialize)]
struct QdrantQueryPoint {
    score: f32,
    #[serde(default)]
    payload: BTreeMap<String, Value>,
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

fn collect_semantic_corpus(
    workspace_root: &Path,
    path_prefixes: &[PathBuf],
) -> Result<SemanticCorpus> {
    let mut builder = WalkBuilder::new(workspace_root);
    builder.hidden(false);

    let mut searchable_file_count = 0usize;
    let mut chunks = Vec::new();
    for entry in builder.build() {
        let Ok(entry) = entry else {
            continue;
        };
        if !entry
            .file_type()
            .is_some_and(|file_type| file_type.is_file())
        {
            continue;
        }

        let path = entry.path();
        let Ok(relative_path) = path.strip_prefix(workspace_root) else {
            continue;
        };
        if should_skip_relative_path(relative_path) {
            continue;
        }
        if !matches_path_prefixes(relative_path, path_prefixes) {
            continue;
        }

        let Ok(metadata) = fs::metadata(path) else {
            continue;
        };
        if metadata.len() > MAX_TEXT_FILE_BYTES {
            continue;
        }

        let Ok(contents) = fs::read_to_string(path) else {
            continue;
        };
        let file_chunks = build_semantic_chunks(relative_path, &contents);
        if file_chunks.is_empty() {
            continue;
        }

        searchable_file_count += 1;
        chunks.extend(file_chunks);
    }

    Ok(SemanticCorpus {
        searchable_file_count,
        chunks,
    })
}

fn collect_workspace_index_snapshot(workspace_root: &Path) -> Result<WorkspaceIndexSnapshot> {
    let mut builder = WalkBuilder::new(workspace_root);
    builder.hidden(false);

    let mut files = BTreeMap::new();
    for entry in builder.build() {
        let Ok(entry) = entry else {
            continue;
        };
        if !entry
            .file_type()
            .is_some_and(|file_type| file_type.is_file())
        {
            continue;
        }

        let path = entry.path();
        let Ok(relative_path) = path.strip_prefix(workspace_root) else {
            continue;
        };
        if should_skip_relative_path(relative_path) {
            continue;
        }

        let metadata = match fs::metadata(path) {
            Ok(metadata) if metadata.len() <= MAX_TEXT_FILE_BYTES => metadata,
            _ => continue,
        };

        let Ok(contents) = fs::read_to_string(path) else {
            continue;
        };
        let chunk_count = build_semantic_chunks(relative_path, &contents).len() as u32;
        if chunk_count == 0 {
            continue;
        }

        files.insert(
            relative_path.to_path_buf(),
            WorkspaceIndexSnapshotFile {
                size_bytes: metadata.len(),
                modified_unix_nanos: unix_timestamp_nanos(metadata.modified()?)?,
                chunk_count,
            },
        );
    }

    Ok(WorkspaceIndexSnapshot { files })
}

fn collect_indexed_files(
    workspace_root: &Path,
    relative_paths: &BTreeSet<PathBuf>,
) -> Result<Vec<IndexedFile>> {
    let mut files = Vec::with_capacity(relative_paths.len());
    for relative_path in relative_paths {
        let absolute_path = workspace_root.join(relative_path);
        let contents = fs::read_to_string(&absolute_path).with_context(|| {
            format!("failed to read indexable file {}", absolute_path.display())
        })?;
        let chunks = build_semantic_chunks(relative_path, &contents);
        if chunks.is_empty() {
            continue;
        }
        files.push(IndexedFile {
            path: relative_path.clone(),
            chunks,
        });
    }
    Ok(files)
}

fn build_semantic_chunks(relative_path: &Path, contents: &str) -> Vec<SemanticChunk> {
    let lines = contents.lines().collect::<Vec<_>>();
    if lines.is_empty() {
        return Vec::new();
    }

    let step = CHUNK_LINE_COUNT.saturating_sub(CHUNK_LINE_OVERLAP).max(1);
    let mut chunks = Vec::new();
    let mut start = 0usize;
    while start < lines.len() {
        let end = (start + CHUNK_LINE_COUNT).min(lines.len());
        let excerpt = lines[start..end].join("\n").trim().to_string();
        if !excerpt.is_empty() {
            chunks.push(SemanticChunk {
                path: relative_path.to_path_buf(),
                line_start: (start + 1) as u32,
                line_end: end as u32,
                search_text: format!("{}\n{}", relative_path.display(), excerpt),
                excerpt,
            });
        }

        if end == lines.len() {
            break;
        }
        start += step;
    }

    chunks
}

fn compute_incremental_plan(
    manifest: Option<&SemanticIndexManifest>,
    snapshot: &WorkspaceIndexSnapshot,
) -> (
    BTreeSet<PathBuf>,
    Vec<String>,
    BTreeMap<PathBuf, SemanticIndexedFile>,
) {
    let Some(manifest) = manifest else {
        return (
            snapshot.files.keys().cloned().collect(),
            Vec::new(),
            BTreeMap::new(),
        );
    };

    let manifest_map = manifest_file_map(manifest);
    let mut paths_to_reindex = BTreeSet::new();
    let mut deleted_point_ids = Vec::new();
    let mut carried_files = BTreeMap::new();

    for (path, manifest_file) in &manifest_map {
        match snapshot.files.get(path) {
            Some(snapshot_file) if same_snapshot_file(manifest_file, snapshot_file) => {
                carried_files.insert(path.clone(), manifest_file.clone());
            }
            Some(_) => {
                paths_to_reindex.insert(path.clone());
                deleted_point_ids.extend(manifest_file.point_ids.clone());
            }
            None => {
                deleted_point_ids.extend(manifest_file.point_ids.clone());
            }
        }
    }

    for path in snapshot.files.keys() {
        if !manifest_map.contains_key(path) {
            paths_to_reindex.insert(path.clone());
        }
    }

    (paths_to_reindex, deleted_point_ids, carried_files)
}

fn manifest_file_map(manifest: &SemanticIndexManifest) -> BTreeMap<PathBuf, SemanticIndexedFile> {
    manifest
        .files
        .iter()
        .cloned()
        .map(|file| (file.path.clone(), file))
        .collect()
}

fn same_snapshot_file(
    manifest_file: &SemanticIndexedFile,
    snapshot_file: &WorkspaceIndexSnapshotFile,
) -> bool {
    manifest_file.size_bytes == snapshot_file.size_bytes
        && manifest_file.modified_unix_nanos == snapshot_file.modified_unix_nanos
        && manifest_file.chunk_count == snapshot_file.chunk_count
}

fn dirty_paths_from_manifest(
    manifest: &SemanticIndexManifest,
    snapshot: &WorkspaceIndexSnapshot,
) -> Vec<PathBuf> {
    let manifest_map = manifest_file_map(manifest);
    let mut dirty_paths = BTreeSet::new();

    for (path, manifest_file) in &manifest_map {
        match snapshot.files.get(path) {
            Some(snapshot_file) if same_snapshot_file(manifest_file, snapshot_file) => {}
            _ => {
                dirty_paths.insert(path.clone());
            }
        }
    }

    for path in snapshot.files.keys() {
        if !manifest_map.contains_key(path) {
            dirty_paths.insert(path.clone());
        }
    }

    dirty_paths.into_iter().collect()
}

fn load_semantic_manifest(path: &Path) -> Result<Option<SemanticIndexManifest>> {
    if !path.is_file() {
        return Ok(None);
    }

    let bytes = fs::read(path)
        .with_context(|| format!("failed to read semantic index manifest {}", path.display()))?;
    let manifest: SemanticIndexManifest = serde_json::from_slice(&bytes).with_context(|| {
        format!(
            "failed to decode semantic index manifest {}",
            path.display()
        )
    })?;
    if manifest.schema_version != SEMANTIC_MANIFEST_SCHEMA_VERSION {
        return Ok(None);
    }

    Ok(Some(manifest))
}

fn write_semantic_manifest(path: &Path, manifest: &SemanticIndexManifest) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create semantic index manifest directory {}",
                parent.display()
            )
        })?;
    }

    let payload =
        serde_json::to_vec_pretty(manifest).context("failed to encode semantic manifest")?;
    fs::write(path, payload)
        .with_context(|| format!("failed to write semantic manifest {}", path.display()))
}

fn search_semantic_chunks(
    query: &str,
    limit: usize,
    semantic_corpus: &SemanticCorpus,
) -> Vec<EpiphanyRetrieveResult> {
    if semantic_corpus.chunks.is_empty() {
        return Vec::new();
    }

    let documents = semantic_corpus
        .chunks
        .iter()
        .enumerate()
        .map(|(index, chunk)| Document::new(index, chunk.search_text.clone()))
        .collect::<Vec<_>>();
    let search_engine =
        SearchEngineBuilder::<usize>::with_documents(Language::English, documents).build();

    search_engine
        .search(query, limit)
        .into_iter()
        .filter_map(|result| {
            semantic_corpus
                .chunks
                .get(result.document.id)
                .map(|chunk| EpiphanyRetrieveResult {
                    kind: EpiphanyRetrieveResultKind::SemanticChunk,
                    path: chunk.path.clone(),
                    score: result.score,
                    line_start: Some(chunk.line_start),
                    line_end: Some(chunk.line_end),
                    excerpt: Some(chunk.excerpt.clone()),
                })
        })
        .collect()
}

fn map_qdrant_hit_to_result(point: QdrantQueryPoint) -> Option<EpiphanyRetrieveResult> {
    let path = point.payload.get("path")?.as_str()?;
    let excerpt = point.payload.get("excerpt")?.as_str()?.to_string();
    let line_start = point.payload.get("line_start")?.as_u64()? as u32;
    let line_end = point.payload.get("line_end")?.as_u64()? as u32;

    Some(EpiphanyRetrieveResult {
        kind: EpiphanyRetrieveResultKind::SemanticChunk,
        path: PathBuf::from(path),
        score: point.score,
        line_start: Some(line_start),
        line_end: Some(line_end),
        excerpt: Some(excerpt),
    })
}

fn unavailable_retrieval_state(workspace_root: &Path) -> EpiphanyRetrievalState {
    EpiphanyRetrievalState {
        workspace_root: workspace_root.to_path_buf(),
        index_revision: Some(QUERY_TIME_INDEX_REVISION.to_string()),
        status: EpiphanyRetrievalStatus::Unavailable,
        semantic_available: false,
        last_indexed_at_unix_seconds: None,
        indexed_file_count: None,
        indexed_chunk_count: None,
        shards: vec![EpiphanyRetrievalShardSummary {
            shard_id: WORKSPACE_SHARD_ID.to_string(),
            path_prefix: PathBuf::from("."),
            indexed_file_count: None,
            indexed_chunk_count: None,
            status: EpiphanyRetrievalStatus::Unavailable,
            exact_available: false,
            semantic_available: false,
        }],
        dirty_paths: Vec::new(),
    }
}

fn baseline_retrieval_state(workspace_root: &Path) -> EpiphanyRetrievalState {
    EpiphanyRetrievalState {
        workspace_root: workspace_root.to_path_buf(),
        index_revision: Some(QUERY_TIME_INDEX_REVISION.to_string()),
        status: EpiphanyRetrievalStatus::Ready,
        semantic_available: true,
        last_indexed_at_unix_seconds: None,
        indexed_file_count: None,
        indexed_chunk_count: None,
        shards: vec![EpiphanyRetrievalShardSummary {
            shard_id: WORKSPACE_SHARD_ID.to_string(),
            path_prefix: PathBuf::from("."),
            indexed_file_count: None,
            indexed_chunk_count: None,
            status: EpiphanyRetrievalStatus::Ready,
            exact_available: true,
            semantic_available: true,
        }],
        dirty_paths: Vec::new(),
    }
}

fn query_time_retrieval_state(
    workspace_root: &Path,
    searchable_file_count: usize,
    chunk_count: usize,
) -> EpiphanyRetrievalState {
    let indexed_file_count = u32::try_from(searchable_file_count).ok();
    let indexed_chunk_count = u32::try_from(chunk_count).ok();

    EpiphanyRetrievalState {
        workspace_root: workspace_root.to_path_buf(),
        index_revision: Some(QUERY_TIME_INDEX_REVISION.to_string()),
        status: EpiphanyRetrievalStatus::Ready,
        semantic_available: chunk_count > 0,
        last_indexed_at_unix_seconds: Some(chrono::Utc::now().timestamp()),
        indexed_file_count,
        indexed_chunk_count,
        shards: vec![EpiphanyRetrievalShardSummary {
            shard_id: WORKSPACE_SHARD_ID.to_string(),
            path_prefix: PathBuf::from("."),
            indexed_file_count,
            indexed_chunk_count,
            status: EpiphanyRetrievalStatus::Ready,
            exact_available: true,
            semantic_available: chunk_count > 0,
        }],
        dirty_paths: Vec::new(),
    }
}

fn manifest_to_retrieval_state(
    workspace_root: &Path,
    manifest: &SemanticIndexManifest,
    status: EpiphanyRetrievalStatus,
    dirty_paths: Vec<PathBuf>,
) -> EpiphanyRetrievalState {
    let indexed_file_count = u32::try_from(manifest.files.len()).ok();
    let indexed_chunk_count = manifest
        .files
        .iter()
        .try_fold(0u32, |count, file| count.checked_add(file.chunk_count));
    let semantic_available = indexed_chunk_count.unwrap_or(0) > 0;

    EpiphanyRetrievalState {
        workspace_root: workspace_root.to_path_buf(),
        index_revision: Some(manifest.index_revision.clone()),
        status,
        semantic_available,
        last_indexed_at_unix_seconds: Some(manifest.indexed_at_unix_seconds),
        indexed_file_count,
        indexed_chunk_count,
        shards: vec![EpiphanyRetrievalShardSummary {
            shard_id: WORKSPACE_SHARD_ID.to_string(),
            path_prefix: PathBuf::from("."),
            indexed_file_count,
            indexed_chunk_count,
            status,
            exact_available: true,
            semantic_available,
        }],
        dirty_paths,
    }
}

fn sort_results(results: &mut [EpiphanyRetrieveResult]) {
    results.sort_by(|left, right| {
        result_priority(&right.kind)
            .cmp(&result_priority(&left.kind))
            .then_with(|| {
                right
                    .score
                    .partial_cmp(&left.score)
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| left.path.cmp(&right.path))
            .then_with(|| {
                left.line_start
                    .unwrap_or(u32::MAX)
                    .cmp(&right.line_start.unwrap_or(u32::MAX))
            })
    });
}

fn result_priority(kind: &EpiphanyRetrieveResultKind) -> u8 {
    match kind {
        EpiphanyRetrieveResultKind::ExactFile => 3,
        EpiphanyRetrieveResultKind::SemanticChunk => 2,
        EpiphanyRetrieveResultKind::ExactDirectory => 1,
    }
}

fn semantic_candidate_limit(limit: usize, path_prefixes: &[PathBuf]) -> usize {
    if path_prefixes.is_empty() {
        limit
    } else {
        limit
            .saturating_mul(EXACT_SEARCH_CANDIDATE_MULTIPLIER)
            .min(EXACT_SEARCH_MAX_CANDIDATES)
    }
}

fn normalize_path_prefixes(workspace_root: &Path, path_prefixes: &[PathBuf]) -> Vec<PathBuf> {
    let mut normalized = path_prefixes
        .iter()
        .filter_map(|prefix| normalize_path_prefix(workspace_root, prefix.as_path()))
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

fn normalize_path_prefix(workspace_root: &Path, prefix: &Path) -> Option<PathBuf> {
    if prefix.as_os_str().is_empty() || prefix == Path::new(".") {
        return None;
    }

    let relative = if prefix.is_absolute() {
        prefix.strip_prefix(workspace_root).ok()?
    } else {
        prefix
    };

    let mut normalized = PathBuf::new();
    for component in relative.components() {
        match component {
            Component::Normal(value) => normalized.push(value),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => return None,
        }
    }

    if normalized.as_os_str().is_empty() || normalized == Path::new(".") {
        None
    } else {
        Some(normalized)
    }
}

fn matches_path_prefixes(path: &Path, path_prefixes: &[PathBuf]) -> bool {
    path_prefixes.is_empty() || path_prefixes.iter().any(|prefix| path.starts_with(prefix))
}

fn should_skip_relative_path(relative_path: &Path) -> bool {
    relative_path
        .components()
        .any(|component| matches!(component, Component::Normal(value) if value == ".git"))
}

fn chunk_point_id(collection_name: &str, chunk: &SemanticChunk) -> String {
    let point_key = format!(
        "{collection_name}\n{}\n{}\n{}",
        normalize_payload_path(&chunk.path),
        chunk.line_start,
        chunk.line_end
    );
    Uuid::new_v5(&Uuid::NAMESPACE_URL, point_key.as_bytes()).to_string()
}

fn normalize_payload_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().to_string()),
            Component::CurDir => None,
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn validate_embedding_batch(embeddings: &[Vec<f32>], expected_count: usize) -> Result<()> {
    if embeddings.len() != expected_count {
        anyhow::bail!(
            "embedding backend returned {} vectors for {} chunks",
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

fn unix_timestamp_seconds(time: SystemTime) -> Result<i64> {
    Ok(i64::try_from(
        time.duration_since(UNIX_EPOCH)
            .context("timestamp before unix epoch")?
            .as_secs(),
    )?)
}

fn unix_timestamp_nanos(time: SystemTime) -> Result<i64> {
    let duration = time
        .duration_since(UNIX_EPOCH)
        .context("timestamp before unix epoch")?;
    let nanos = duration
        .as_secs()
        .checked_mul(1_000_000_000)
        .and_then(|seconds| seconds.checked_add(u64::from(duration.subsec_nanos())))
        .context("timestamp overflow")?;
    Ok(i64::try_from(nanos)?)
}

fn normalize_base_url(input: &str) -> String {
    input.trim_end_matches('/').to_string()
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

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;
    use wiremock::Mock;
    use wiremock::MockServer;
    use wiremock::ResponseTemplate;
    use wiremock::matchers::method;
    use wiremock::matchers::path;

    #[test]
    fn normalize_epiphany_retrieve_query_trims_and_clamps_limit() {
        let query =
            normalize_epiphany_retrieve_query("  auth spine  ".to_string(), Some(999), Vec::new())
                .unwrap();

        assert_eq!(query.query, "auth spine");
        assert_eq!(query.limit, EPIPHANY_RETRIEVAL_MAX_LIMIT);
    }

    #[test]
    fn normalize_epiphany_retrieve_query_rejects_empty_query_and_zero_limit() {
        assert_eq!(
            normalize_epiphany_retrieve_query("  ".to_string(), Some(1), Vec::new()),
            Err("query must not be empty")
        );
        assert_eq!(
            normalize_epiphany_retrieve_query("auth".to_string(), Some(0), Vec::new()),
            Err("limit must be greater than zero")
        );
    }

    #[test]
    fn retrieve_workspace_prioritizes_exact_hits_without_losing_semantic_chunks() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let codex_home = TempDir::new()?;
        fs::create_dir_all(temp_dir.path().join("src"))?;
        fs::create_dir_all(temp_dir.path().join("notes"))?;
        fs::write(
            temp_dir.path().join("src").join("session_checkpoint.rs"),
            "pub fn route_session() {}\n",
        )?;
        fs::write(
            temp_dir.path().join("notes").join("design.md"),
            "The compaction checkpoint keeps the field notebook alive.\n\
             Retrieval should honor the checkpoint before the next pass.\n",
        )?;

        let response = retrieve_workspace(
            temp_dir.path(),
            codex_home.path(),
            EpiphanyRetrieveQuery {
                query: "session checkpoint".to_string(),
                limit: 5,
                path_prefixes: Vec::new(),
            },
        )?;

        assert_eq!(
            response.results.first().map(|result| &result.kind),
            Some(&EpiphanyRetrieveResultKind::ExactFile)
        );
        assert!(
            response
                .results
                .iter()
                .any(|result| result.kind == EpiphanyRetrieveResultKind::SemanticChunk),
            "expected at least one semantic chunk result"
        );
        Ok(())
    }

    #[test]
    fn retrieve_workspace_finds_semantic_match_without_path_match() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let codex_home = TempDir::new()?;
        fs::create_dir_all(temp_dir.path().join("src"))?;
        fs::write(
            temp_dir.path().join("src").join("engine.rs"),
            "The frontier checkpoint preserves the machine map before compaction.\n\
             Keep the durable spine and discard disposable scratch.\n",
        )?;

        let response = retrieve_workspace(
            temp_dir.path(),
            codex_home.path(),
            EpiphanyRetrieveQuery {
                query: "durable spine compaction checkpoint".to_string(),
                limit: 3,
                path_prefixes: Vec::new(),
            },
        )?;

        assert!(
            response.results.iter().any(|result| {
                result.kind == EpiphanyRetrieveResultKind::SemanticChunk
                    && result.path == PathBuf::from("src").join("engine.rs")
            }),
            "expected semantic retrieval to find engine.rs"
        );
        Ok(())
    }

    #[test]
    fn retrieve_workspace_respects_path_prefixes() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let codex_home = TempDir::new()?;
        fs::create_dir_all(temp_dir.path().join("src"))?;
        fs::create_dir_all(temp_dir.path().join("docs"))?;
        fs::write(
            temp_dir.path().join("src").join("router.rs"),
            "checkpoint frontier checkpoint frontier\n",
        )?;
        fs::write(
            temp_dir.path().join("docs").join("guide.md"),
            "checkpoint frontier checkpoint frontier\n",
        )?;

        let response = retrieve_workspace(
            temp_dir.path(),
            codex_home.path(),
            EpiphanyRetrieveQuery {
                query: "checkpoint frontier".to_string(),
                limit: 10,
                path_prefixes: vec![PathBuf::from("docs")],
            },
        )?;

        assert!(
            response
                .results
                .iter()
                .all(|result| result.path.starts_with("docs")),
            "all results should stay inside the requested path prefix"
        );
        Ok(())
    }

    #[test]
    fn index_workspace_builds_manifest_and_retrieve_uses_qdrant_when_fresh() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let codex_home = TempDir::new()?;
        fs::create_dir_all(temp_dir.path().join("src"))?;
        fs::write(
            temp_dir.path().join("src").join("router.rs"),
            "The checkpoint frontier keeps the durable spine intact.\n",
        )?;

        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let qdrant = runtime.block_on(MockServer::start());
        let ollama = runtime.block_on(MockServer::start());
        let config = SemanticBackendConfig {
            qdrant: QdrantConfig {
                url: qdrant.uri(),
                api_key: None,
                timeout_ms: 5_000,
            },
            ollama: OllamaConfig {
                base_url: ollama.uri(),
                model: "qwen3-embedding:0.6b".to_string(),
                timeout_ms: 5_000,
                query_instruction: DEFAULT_QUERY_INSTRUCTION.to_string(),
            },
        };

        runtime.block_on(async {
            Mock::given(method("GET"))
                .and(path(format!(
                    "/collections/{}/exists",
                    config.collection_name(temp_dir.path())
                )))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "result": { "exists": false }
                })))
                .expect(1)
                .mount(&qdrant)
                .await;

            Mock::given(method("PUT"))
                .and(path(format!(
                    "/collections/{}",
                    config.collection_name(temp_dir.path())
                )))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "result": true })))
                .expect(1)
                .mount(&qdrant)
                .await;

            Mock::given(method("PUT"))
                .and(path(format!(
                    "/collections/{}/points",
                    config.collection_name(temp_dir.path())
                )))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "result": true })))
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
        });

        let summary =
            index_workspace_with_config(temp_dir.path(), codex_home.path(), false, &config)?;
        assert_eq!(summary.status, EpiphanyRetrievalStatus::Ready);
        assert_eq!(summary.indexed_file_count, Some(1));

        runtime.block_on(async {
            qdrant.reset().await;
            ollama.reset().await;

            Mock::given(method("GET"))
                .and(path(format!(
                    "/collections/{}/exists",
                    config.collection_name(temp_dir.path())
                )))
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
                .and(path(format!(
                    "/collections/{}/points/query",
                    config.collection_name(temp_dir.path())
                )))
                .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                    "result": {
                        "points": [
                            {
                                "score": 0.93,
                                "payload": {
                                    "path": "src/router.rs",
                                    "line_start": 1,
                                    "line_end": 1,
                                    "excerpt": "The checkpoint frontier keeps the durable spine intact."
                                }
                            }
                        ]
                    }
                })))
                .expect(1)
                .mount(&qdrant)
                .await;
        });

        let response = retrieve_workspace_with_config(
            temp_dir.path(),
            codex_home.path(),
            EpiphanyRetrieveQuery::new("durable spine".to_string()),
            &config,
        )?;
        assert_eq!(
            response.index_summary.index_revision,
            Some(config.index_revision())
        );
        assert!(response.results.iter().any(|result| {
            result.kind == EpiphanyRetrieveResultKind::SemanticChunk
                && result.path == Path::new("src/router.rs")
        }));
        drop(qdrant);
        drop(ollama);
        drop(runtime);
        Ok(())
    }

    #[test]
    fn retrieval_state_marks_manifest_stale_after_workspace_change() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let codex_home = TempDir::new()?;
        fs::create_dir_all(temp_dir.path().join("src"))?;
        fs::write(
            temp_dir.path().join("src").join("router.rs"),
            "The checkpoint frontier keeps the durable spine intact.\n",
        )?;

        let config = SemanticBackendConfig::from_env();
        let snapshot = collect_workspace_index_snapshot(temp_dir.path())?;
        let manifest = SemanticIndexManifest {
            schema_version: SEMANTIC_MANIFEST_SCHEMA_VERSION,
            workspace_root: temp_dir.path().to_path_buf(),
            collection_name: config.collection_name(temp_dir.path()),
            index_revision: config.index_revision(),
            indexed_at_unix_seconds: 1_744_500_000,
            files: snapshot
                .files
                .iter()
                .map(|(path, file)| SemanticIndexedFile {
                    path: path.clone(),
                    size_bytes: file.size_bytes,
                    modified_unix_nanos: file.modified_unix_nanos,
                    chunk_count: file.chunk_count,
                    point_ids: vec!["point-1".to_string()],
                })
                .collect(),
        };
        write_semantic_manifest(
            &config.manifest_path(codex_home.path(), temp_dir.path()),
            &manifest,
        )?;

        fs::write(
            temp_dir.path().join("src").join("router.rs"),
            "The checkpoint frontier moved and now the file changed.\n",
        )?;

        let state = retrieval_state_for_workspace(temp_dir.path(), codex_home.path());
        assert_eq!(state.status, EpiphanyRetrievalStatus::Stale);
        assert_eq!(state.dirty_paths, vec![PathBuf::from("src/router.rs")]);
        Ok(())
    }

    fn retrieve_workspace_with_config(
        workspace_root: &Path,
        codex_home: &Path,
        query: EpiphanyRetrieveQuery,
        config: &SemanticBackendConfig,
    ) -> Result<EpiphanyRetrieveResponse> {
        let query_text = query.query.trim();
        let limit = query.limit.clamp(1, EPIPHANY_RETRIEVAL_MAX_LIMIT);
        let path_prefixes = normalize_path_prefixes(workspace_root, &query.path_prefixes);
        let exact_results = run_exact_search(workspace_root, query_text, limit, &path_prefixes)?;
        let semantic_limit = semantic_candidate_limit(limit, &path_prefixes);
        let persistent_search = search_persistent_semantic_chunks_with_config(
            workspace_root,
            codex_home,
            query_text,
            semantic_limit,
            config,
        )?;

        let (index_summary, semantic_results) = match persistent_search {
            PersistentSemanticSearch::Ready { summary, results } => (summary, results),
            PersistentSemanticSearch::Fallback { summary } => (summary.unwrap(), Vec::new()),
            PersistentSemanticSearch::None => {
                (baseline_retrieval_state(workspace_root), Vec::new())
            }
        };

        let mut results = exact_results;
        results.extend(semantic_results);
        sort_results(&mut results);
        Ok(EpiphanyRetrieveResponse {
            query: query_text.to_string(),
            index_summary,
            results,
        })
    }

    fn search_persistent_semantic_chunks_with_config(
        workspace_root: &Path,
        codex_home: &Path,
        query: &str,
        limit: usize,
        config: &SemanticBackendConfig,
    ) -> Result<PersistentSemanticSearch> {
        let manifest_path = config.manifest_path(codex_home, workspace_root);
        let Some(manifest) = load_semantic_manifest(&manifest_path)? else {
            return Ok(PersistentSemanticSearch::None);
        };
        let snapshot = collect_workspace_index_snapshot(workspace_root)?;
        let dirty_paths = dirty_paths_from_manifest(&manifest, &snapshot);
        if !dirty_paths.is_empty() {
            return Ok(PersistentSemanticSearch::Fallback {
                summary: Some(manifest_to_retrieval_state(
                    workspace_root,
                    &manifest,
                    EpiphanyRetrievalStatus::Stale,
                    dirty_paths,
                )),
            });
        }

        let qdrant = QdrantClient::new(config.qdrant.clone())?;
        if !qdrant.collection_exists(&manifest.collection_name)? {
            return Ok(PersistentSemanticSearch::Fallback {
                summary: Some(manifest_to_retrieval_state(
                    workspace_root,
                    &manifest,
                    EpiphanyRetrievalStatus::Stale,
                    Vec::new(),
                )),
            });
        }

        let ollama = OllamaEmbedder::new(config.ollama.clone())?;
        let query_vector = ollama.embed_query(query)?;
        let hits = qdrant.query_points(&manifest.collection_name, &query_vector, limit)?;
        Ok(PersistentSemanticSearch::Ready {
            summary: manifest_to_retrieval_state(
                workspace_root,
                &manifest,
                EpiphanyRetrievalStatus::Ready,
                Vec::new(),
            ),
            results: hits
                .into_iter()
                .filter_map(map_qdrant_hit_to_result)
                .collect(),
        })
    }
}
