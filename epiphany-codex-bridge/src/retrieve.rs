use codex_app_server_protocol::ThreadEpiphanyIndexResponse;
use codex_app_server_protocol::ThreadEpiphanyRetrieveIndexSummary;
use codex_app_server_protocol::ThreadEpiphanyRetrieveResponse;
use codex_app_server_protocol::ThreadEpiphanyRetrieveResult;
use codex_app_server_protocol::ThreadEpiphanyRetrieveResultKind;
use codex_app_server_protocol::ThreadEpiphanyRetrieveShardSummary;
use codex_utils_absolute_path::AbsolutePathBuf;
use epiphany_core::EpiphanyRetrieveQuery;
use epiphany_core::EpiphanyRetrieveResponse as CoreEpiphanyRetrieveResponse;
use epiphany_core::EpiphanyRetrieveResult as CoreEpiphanyRetrieveResult;
use epiphany_core::EpiphanyRetrieveResultKind as CoreEpiphanyRetrieveResultKind;
use epiphany_core::index_workspace;
use epiphany_core::normalize_epiphany_retrieve_query;
use epiphany_core::retrieval_state_for_workspace;
use epiphany_core::retrieve_workspace;
use epiphany_state_model::EpiphanyRetrievalState;
use std::path::PathBuf;

pub fn map_epiphany_retrieve_response(
    response: CoreEpiphanyRetrieveResponse,
) -> anyhow::Result<ThreadEpiphanyRetrieveResponse> {
    Ok(ThreadEpiphanyRetrieveResponse {
        query: response.query,
        index_summary: map_epiphany_retrieve_index_summary(response.index_summary)?,
        results: response
            .results
            .into_iter()
            .map(map_epiphany_retrieve_result)
            .collect(),
    })
}

pub async fn index_epiphany_retrieval_for_paths(
    workspace_root: PathBuf,
    codex_home: PathBuf,
    force_full_rebuild: bool,
) -> anyhow::Result<ThreadEpiphanyIndexResponse> {
    let index_summary =
        index_epiphany_retrieval_state_for_paths(workspace_root, codex_home, force_full_rebuild)
            .await
            .and_then(map_epiphany_retrieve_index_summary)?;
    Ok(ThreadEpiphanyIndexResponse { index_summary })
}

pub fn normalize_thread_epiphany_retrieve_query(
    query: String,
    limit: Option<u32>,
    path_prefixes: Vec<PathBuf>,
) -> Result<EpiphanyRetrieveQuery, String> {
    normalize_epiphany_retrieve_query(query, limit, path_prefixes).map_err(str::to_string)
}

pub async fn epiphany_retrieval_state_for_paths(
    workspace_root: PathBuf,
    codex_home: PathBuf,
) -> EpiphanyRetrievalState {
    let fallback_workspace_root = workspace_root.clone();
    let fallback_codex_home = codex_home.clone();
    tokio::task::spawn_blocking(move || {
        retrieval_state_for_workspace(&workspace_root, codex_home.as_path())
    })
    .await
    .unwrap_or_else(|_| {
        retrieval_state_for_workspace(&fallback_workspace_root, fallback_codex_home.as_path())
    })
}

pub async fn index_epiphany_retrieval_state_for_paths(
    workspace_root: PathBuf,
    codex_home: PathBuf,
    force_full_rebuild: bool,
) -> anyhow::Result<EpiphanyRetrievalState> {
    tokio::task::spawn_blocking(move || {
        index_workspace(&workspace_root, codex_home.as_path(), force_full_rebuild)
    })
    .await
    .map_err(|err| anyhow::anyhow!("epiphany index worker failed: {err}"))?
}

pub async fn retrieve_epiphany_for_paths(
    workspace_root: PathBuf,
    codex_home: PathBuf,
    query: EpiphanyRetrieveQuery,
) -> anyhow::Result<CoreEpiphanyRetrieveResponse> {
    tokio::task::spawn_blocking(move || {
        retrieve_workspace(&workspace_root, codex_home.as_path(), query)
    })
    .await
    .map_err(|err| anyhow::anyhow!("epiphany retrieval worker failed: {err}"))?
}

pub fn map_epiphany_retrieve_index_summary(
    summary: EpiphanyRetrievalState,
) -> anyhow::Result<ThreadEpiphanyRetrieveIndexSummary> {
    Ok(ThreadEpiphanyRetrieveIndexSummary {
        workspace_root: AbsolutePathBuf::from_absolute_path(summary.workspace_root)
            .map_err(anyhow::Error::from)?,
        index_revision: summary.index_revision,
        status: summary.status,
        semantic_available: summary.semantic_available,
        last_indexed_at_unix_seconds: summary.last_indexed_at_unix_seconds,
        indexed_file_count: summary.indexed_file_count,
        indexed_chunk_count: summary.indexed_chunk_count,
        shards: summary
            .shards
            .into_iter()
            .map(|shard| ThreadEpiphanyRetrieveShardSummary {
                shard_id: shard.shard_id,
                path_prefix: shard.path_prefix,
                indexed_file_count: shard.indexed_file_count,
                indexed_chunk_count: shard.indexed_chunk_count,
                status: shard.status,
                exact_available: shard.exact_available,
                semantic_available: shard.semantic_available,
            })
            .collect(),
        dirty_paths: summary.dirty_paths,
    })
}

fn map_epiphany_retrieve_result(
    result: CoreEpiphanyRetrieveResult,
) -> ThreadEpiphanyRetrieveResult {
    ThreadEpiphanyRetrieveResult {
        kind: match result.kind {
            CoreEpiphanyRetrieveResultKind::ExactFile => {
                ThreadEpiphanyRetrieveResultKind::ExactFile
            }
            CoreEpiphanyRetrieveResultKind::ExactDirectory => {
                ThreadEpiphanyRetrieveResultKind::ExactDirectory
            }
            CoreEpiphanyRetrieveResultKind::SemanticChunk => {
                ThreadEpiphanyRetrieveResultKind::SemanticChunk
            }
        },
        path: result.path,
        score: result.score,
        line_start: result.line_start,
        line_end: result.line_end,
        excerpt: result.excerpt,
    }
}
