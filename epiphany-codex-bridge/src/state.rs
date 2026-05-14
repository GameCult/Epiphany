use std::path::Path;

use codex_core::CodexThread;
use codex_core::RolloutRecorder;
use codex_core::latest_epiphany_state_from_rollout_items;
use codex_protocol::protocol::EpiphanyThreadState;
use codex_protocol::protocol::InitialHistory;

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
        state.retrieval = Some(thread.epiphany_retrieval_state().await);
    }
    epiphany_state
}

pub async fn client_visible_live_thread_epiphany_state(
    thread: &CodexThread,
    fallback: EpiphanyThreadState,
) -> EpiphanyThreadState {
    live_thread_epiphany_state(thread).await.unwrap_or(fallback)
}
