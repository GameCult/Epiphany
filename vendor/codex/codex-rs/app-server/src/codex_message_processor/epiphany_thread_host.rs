use std::path::Path;

use codex_core::CodexThread;
use codex_core::RolloutRecorder;
use codex_core::latest_epiphany_state_from_rollout_items;
use codex_protocol::error::CodexErr;
use codex_protocol::protocol::EpiphanyThreadState;
use codex_protocol::protocol::InitialHistory;
use epiphany_codex_bridge::mutation_service::EpiphanyMutationHost;
use epiphany_codex_bridge::state::client_visible_epiphany_state_for_paths;
use epiphany_codex_bridge::state::thread_state_mirror_id_from_rollout_path;

pub(super) struct EpiphanyCodexThreadHost<'a> {
    thread: &'a CodexThread,
}

impl<'a> EpiphanyCodexThreadHost<'a> {
    pub(super) fn new(thread: &'a CodexThread) -> Self {
        Self { thread }
    }
}

impl EpiphanyMutationHost for EpiphanyCodexThreadHost<'_> {
    async fn epiphany_state(&self) -> Option<EpiphanyThreadState> {
        self.thread.epiphany_state().await
    }

    async fn epiphany_reference_turn_id(&self) -> Option<String> {
        self.thread.epiphany_reference_turn_id().await
    }

    async fn epiphany_persist_state(
        &self,
        next_state: EpiphanyThreadState,
    ) -> Result<EpiphanyThreadState, CodexErr> {
        self.thread.epiphany_persist_state(next_state).await
    }

    async fn epiphany_runtime_spine_store_path(&self) -> std::path::PathBuf {
        self.thread.epiphany_runtime_spine_store_path().await
    }

    async fn client_visible_epiphany_state(
        &self,
        fallback: EpiphanyThreadState,
    ) -> EpiphanyThreadState {
        client_visible_live_thread_epiphany_state(self.thread, fallback).await
    }
}

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
