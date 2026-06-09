use std::fs;
use std::path::Path;
use std::path::PathBuf;

use epiphany_core::write_thread_state;
use epiphany_state_model::EpiphanyThreadState;
use tracing::warn;

use crate::retrieve::epiphany_retrieval_state_for_paths;

pub async fn client_visible_epiphany_state_for_paths(
    mut state: EpiphanyThreadState,
    workspace_root: &Path,
    codex_home: PathBuf,
    mirror_thread_id: Option<&str>,
) -> EpiphanyThreadState {
    if state.retrieval.is_none() {
        state.retrieval = Some(
            epiphany_retrieval_state_for_paths(workspace_root.to_path_buf(), codex_home).await,
        );
    }
    if let Some(thread_id) = mirror_thread_id
        && let Err(err) = mirror_thread_state_to_workspace(workspace_root, thread_id, &state)
    {
        warn!("failed to mirror Epiphany thread state into native store: {err}");
    }
    state
}

pub fn thread_state_store_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join("state").join("thread-state.cc")
}

pub fn mirror_thread_state_to_workspace(
    workspace_root: &Path,
    thread_id: &str,
    state: &EpiphanyThreadState,
) -> anyhow::Result<PathBuf> {
    let store_path = thread_state_store_path(workspace_root);
    if let Some(parent) = store_path.parent() {
        fs::create_dir_all(parent)?;
    }
    write_thread_state(&store_path, thread_id, state)?;
    Ok(store_path)
}

pub fn thread_state_mirror_id_from_rollout_path(rollout_path: Option<&Path>) -> String {
    rollout_path
        .and_then(|path| path.file_stem())
        .and_then(|stem| stem.to_str())
        .map(str::to_string)
        .unwrap_or_else(|| "live-thread".to_string())
}
