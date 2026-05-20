use epiphany_core::EpiphanyRetrieveQuery;
use epiphany_core::EpiphanyRetrieveResponse as CoreEpiphanyRetrieveResponse;
use epiphany_core::index_workspace;
use epiphany_core::normalize_epiphany_retrieve_query;
use epiphany_core::retrieval_state_for_workspace;
use epiphany_core::retrieve_workspace;
use epiphany_state_model::EpiphanyRetrievalState;
use std::path::PathBuf;

pub async fn index_epiphany_retrieval_for_paths(
    workspace_root: PathBuf,
    codex_home: PathBuf,
    force_full_rebuild: bool,
) -> anyhow::Result<EpiphanyRetrievalState> {
    index_epiphany_retrieval_state_for_paths(workspace_root, codex_home, force_full_rebuild).await
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
