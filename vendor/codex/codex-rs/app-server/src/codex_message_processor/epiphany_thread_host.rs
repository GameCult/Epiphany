use std::path::Path;

use codex_core::CodexThread;
use codex_core::RolloutRecorder;
use codex_core::latest_epiphany_state_from_rollout_items;
use codex_protocol::protocol::EpiphanyThreadState;
use codex_protocol::protocol::InitialHistory;
use epiphany_codex_bridge::state::client_visible_epiphany_state_for_paths;
use epiphany_codex_bridge::state::thread_state_mirror_id_from_rollout_path;

pub(super) async fn load_epiphany_state_from_rollout_path(
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

pub(super) async fn live_thread_epiphany_state(
    thread: &CodexThread,
) -> Option<EpiphanyThreadState> {
    let state = thread.epiphany_state().await?;
    let config = thread.config_snapshot().await;
    let codex_home = thread.codex_home().await;
    let rollout_path = thread.rollout_path();
    let mirror_thread_id = thread_state_mirror_id_from_rollout_path(rollout_path.as_deref());
    Some(
        client_visible_epiphany_state_for_paths(
            state,
            config.cwd.as_path(),
            codex_home,
            Some(mirror_thread_id.as_str()),
        )
        .await,
    )
}

pub(super) async fn client_visible_live_thread_epiphany_state(
    thread: &CodexThread,
    fallback: EpiphanyThreadState,
) -> EpiphanyThreadState {
    live_thread_epiphany_state(thread).await.unwrap_or(fallback)
}
