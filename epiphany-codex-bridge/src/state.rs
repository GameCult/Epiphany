use std::fs;
use std::path::Path;
use std::path::PathBuf;

use codex_core::CodexThread;
use codex_core::RolloutRecorder;
use codex_core::latest_epiphany_state_from_rollout_items;
use codex_protocol::protocol::EpiphanyThreadState;
use codex_protocol::protocol::InitialHistory;
use epiphany_core::write_thread_state;
use tracing::warn;

use crate::retrieve::thread_epiphany_retrieval_state;

pub async fn load_epiphany_state_from_rollout_path(
    rollout_path: &Path,
) -> std::result::Result<Option<EpiphanyThreadState>, String> {
    let items = match RolloutRecorder::get_rollout_history(rollout_path)
        .await
        .map_err(|err| {
            format!(
                "failed to load rollout `{}` for Epiphany state: {err}",
                rollout_path.display()
            )
        })? {
        InitialHistory::New | InitialHistory::Cleared => Vec::new(),
        InitialHistory::Forked(items) => items,
        InitialHistory::Resumed(resumed) => resumed.history,
    };
    Ok(latest_epiphany_state_from_rollout_items(&items))
}

pub async fn live_thread_epiphany_state(thread: &CodexThread) -> Option<EpiphanyThreadState> {
    let mut epiphany_state = thread.epiphany_state().await;
    if let Some(state) = epiphany_state.as_mut()
        && state.retrieval.is_none()
    {
        state.retrieval = Some(thread_epiphany_retrieval_state(thread).await);
    }
    if let Some(state) = epiphany_state.as_ref() {
        let config = thread.config_snapshot().await;
        let thread_id = thread_state_mirror_id(thread);
        if let Err(err) = mirror_thread_state_to_workspace(config.cwd.as_path(), &thread_id, state)
        {
            warn!(
                "failed to mirror Epiphany thread state into native store: {err}"
            );
        }
    }
    epiphany_state
}

pub async fn client_visible_live_thread_epiphany_state(
    thread: &CodexThread,
    fallback: EpiphanyThreadState,
) -> EpiphanyThreadState {
    live_thread_epiphany_state(thread).await.unwrap_or(fallback)
}

pub fn thread_state_store_path(workspace_root: &Path) -> PathBuf {
    workspace_root.join("state").join("thread-state.msgpack")
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

fn thread_state_mirror_id(thread: &CodexThread) -> String {
    thread
        .rollout_path()
        .and_then(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .map(str::to_string)
        })
        .unwrap_or_else(|| "live-thread".to_string())
}
