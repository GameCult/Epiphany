use anyhow::Context;
use anyhow::Result;
use bm25::Document;
use bm25::Language;
use bm25::SearchEngineBuilder;
use codex_file_search as file_search;
use codex_protocol::protocol::EpiphanyRetrievalShardSummary;
use codex_protocol::protocol::EpiphanyRetrievalState;
use codex_protocol::protocol::EpiphanyRetrievalStatus;
use ignore::WalkBuilder;
use std::cmp::Ordering;
use std::fs;
use std::num::NonZero;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

pub const EPIPHANY_RETRIEVAL_DEFAULT_LIMIT: usize = 10;
pub const EPIPHANY_RETRIEVAL_MAX_LIMIT: usize = 50;

const EXACT_SEARCH_CANDIDATE_MULTIPLIER: usize = 5;
const EXACT_SEARCH_MAX_CANDIDATES: usize = 250;
const FILE_SEARCH_MAX_THREADS: usize = 12;
const QUERY_TIME_INDEX_REVISION: &str = "query-time-bm25-v1";
const MAX_TEXT_FILE_BYTES: u64 = 256 * 1024;
const CHUNK_LINE_COUNT: usize = 24;
const CHUNK_LINE_OVERLAP: usize = 8;
const WORKSPACE_SHARD_ID: &str = "workspace";

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

pub fn retrieval_state_for_workspace(workspace_root: &Path) -> EpiphanyRetrievalState {
    let available = workspace_root.is_dir();
    let status = if available {
        EpiphanyRetrievalStatus::Ready
    } else {
        EpiphanyRetrievalStatus::Unavailable
    };

    EpiphanyRetrievalState {
        workspace_root: workspace_root.to_path_buf(),
        index_revision: Some(QUERY_TIME_INDEX_REVISION.to_string()),
        status,
        semantic_available: available,
        last_indexed_at_unix_seconds: None,
        indexed_file_count: None,
        indexed_chunk_count: None,
        shards: vec![EpiphanyRetrievalShardSummary {
            shard_id: WORKSPACE_SHARD_ID.to_string(),
            path_prefix: PathBuf::from("."),
            indexed_file_count: None,
            indexed_chunk_count: None,
            status,
            exact_available: available,
            semantic_available: available,
        }],
        dirty_paths: Vec::new(),
    }
}

pub fn retrieve_workspace(
    workspace_root: &Path,
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
    let semantic_corpus = collect_semantic_corpus(workspace_root, &path_prefixes)
        .context("semantic corpus collection failed")?;
    let semantic_results = search_semantic_chunks(query_text, limit, &semantic_corpus);

    let mut results = Vec::with_capacity(exact_results.len() + semantic_results.len());
    results.extend(exact_results);
    results.extend(semantic_results);
    sort_results(&mut results);
    results.truncate(limit);

    Ok(EpiphanyRetrieveResponse {
        query: query_text.to_string(),
        index_summary: query_time_retrieval_state(
            workspace_root,
            semantic_corpus.searchable_file_count,
            semantic_corpus.chunks.len(),
        ),
        results,
    })
}

fn run_exact_search(
    workspace_root: &Path,
    query: &str,
    limit: usize,
    path_prefixes: &[PathBuf],
) -> Result<Vec<EpiphanyRetrieveResult>> {
    let candidate_limit = if path_prefixes.is_empty() {
        limit
    } else {
        limit
            .saturating_mul(EXACT_SEARCH_CANDIDATE_MULTIPLIER)
            .min(EXACT_SEARCH_MAX_CANDIDATES)
    };
    #[expect(clippy::expect_used)]
    let candidate_limit =
        NonZero::new(candidate_limit.max(1)).expect("candidate limit should be non-zero");

    let search_results = file_search::run(
        query,
        vec![workspace_root.to_path_buf()],
        file_search::FileSearchOptions {
            limit: candidate_limit,
            threads: search_threads(),
            compute_indices: false,
            ..Default::default()
        },
        /*cancel_flag*/ None,
    )?;

    let mut results = search_results
        .matches
        .into_iter()
        .filter_map(|file_match| {
            let relative_path = file_match
                .full_path()
                .strip_prefix(workspace_root)
                .ok()?
                .to_path_buf();
            if !matches_path_prefixes(&relative_path, path_prefixes) {
                return None;
            }

            Some(EpiphanyRetrieveResult {
                kind: match file_match.match_type {
                    file_search::MatchType::File => EpiphanyRetrieveResultKind::ExactFile,
                    file_search::MatchType::Directory => EpiphanyRetrieveResultKind::ExactDirectory,
                },
                path: relative_path,
                score: file_match.score as f32,
                line_start: None,
                line_end: None,
                excerpt: None,
            })
        })
        .collect::<Vec<_>>();

    results.truncate(limit);
    Ok(results)
}

fn search_threads() -> NonZero<usize> {
    let cores = std::thread::available_parallelism()
        .map(NonZero::get)
        .unwrap_or(1);
    #[expect(clippy::expect_used)]
    NonZero::new(cores.min(FILE_SEARCH_MAX_THREADS).max(1))
        .expect("file-search threads should be non-zero")
}

struct SemanticCorpus {
    searchable_file_count: usize,
    chunks: Vec<SemanticChunk>,
}

#[derive(Clone, Debug)]
struct SemanticChunk {
    path: PathBuf,
    line_start: u32,
    line_end: u32,
    excerpt: String,
    search_text: String,
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

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Result;
    use pretty_assertions::assert_eq;
    use tempfile::TempDir;

    #[test]
    fn retrieve_workspace_prioritizes_exact_hits_without_losing_semantic_chunks() -> Result<()> {
        let temp_dir = TempDir::new()?;
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
        fs::create_dir_all(temp_dir.path().join("src"))?;
        fs::write(
            temp_dir.path().join("src").join("engine.rs"),
            "The frontier checkpoint preserves the machine map before compaction.\n\
             Keep the durable spine and discard disposable scratch.\n",
        )?;

        let response = retrieve_workspace(
            temp_dir.path(),
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
}
