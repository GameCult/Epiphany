use codex_app_server_protocol::ThreadEpiphanyIndexResponse;
use codex_app_server_protocol::ThreadEpiphanyRetrieveIndexSummary;
use codex_app_server_protocol::ThreadEpiphanyRetrieveResponse;
use codex_app_server_protocol::ThreadEpiphanyRetrieveResult;
use codex_app_server_protocol::ThreadEpiphanyRetrieveResultKind;
use codex_app_server_protocol::ThreadEpiphanyRetrieveShardSummary;
use codex_utils_absolute_path::AbsolutePathBuf;
use epiphany_core::EpiphanyRetrieveResponse as CoreEpiphanyRetrieveResponse;
use epiphany_core::EpiphanyRetrieveResult as CoreEpiphanyRetrieveResult;
use epiphany_core::EpiphanyRetrieveResultKind as CoreEpiphanyRetrieveResultKind;
use epiphany_state_model::EpiphanyRetrievalState;

pub fn protocol_retrieve_response(
    response: CoreEpiphanyRetrieveResponse,
) -> anyhow::Result<ThreadEpiphanyRetrieveResponse> {
    Ok(ThreadEpiphanyRetrieveResponse {
        query: response.query,
        index_summary: protocol_retrieve_index_summary(response.index_summary)?,
        results: response
            .results
            .into_iter()
            .map(protocol_retrieve_result)
            .collect(),
    })
}

pub fn protocol_index_response(
    index_summary: EpiphanyRetrievalState,
) -> anyhow::Result<ThreadEpiphanyIndexResponse> {
    Ok(ThreadEpiphanyIndexResponse {
        index_summary: protocol_retrieve_index_summary(index_summary)?,
    })
}

pub fn protocol_retrieve_index_summary(
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

fn protocol_retrieve_result(result: CoreEpiphanyRetrieveResult) -> ThreadEpiphanyRetrieveResult {
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
