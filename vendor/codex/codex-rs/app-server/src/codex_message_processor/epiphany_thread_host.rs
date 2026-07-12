use std::path::Path;

use codex_core::CodexThread;
use codex_core::RolloutRecorder;
use codex_core::latest_epiphany_state_from_rollout_items;
use codex_protocol::protocol::EpiphanyThreadState;
use codex_protocol::protocol::InitialHistory;
use codex_protocol::protocol::TokenUsageInfo;
use epiphany_codex_bridge::mutation_service::EpiphanyMutationHost;
use epiphany_codex_bridge::mutation_service::load_authoritative_accepted_state;
use epiphany_codex_bridge::pressure::EpiphanyTokenUsageSnapshot;
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
    async fn epiphany_thread_id(&self) -> String {
        thread_state_mirror_id_from_rollout_path(self.thread.rollout_path().as_deref())
    }

    async fn epiphany_state(&self) -> Option<EpiphanyThreadState> {
        let store = self.thread.epiphany_runtime_spine_store_path().await;
        match load_authoritative_accepted_state(&store) {
            Ok(Some(state)) => Some(state),
            Ok(None) => self.thread.epiphany_state().await,
            Err(error) => {
                tracing::error!(%error, store = %store.display(), "failed to read authoritative unified Epiphany acceptance state");
                None
            }
        }
    }

    async fn epiphany_reference_turn_id(&self) -> Option<String> {
        self.thread.epiphany_reference_turn_id().await
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
    let state = load_authoritative_epiphany_state(thread).await?;
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

pub(super) async fn load_authoritative_epiphany_state(
    thread: &CodexThread,
) -> Option<EpiphanyThreadState> {
    let store = thread.epiphany_runtime_spine_store_path().await;
    match load_authoritative_accepted_state(&store) {
        Ok(Some(state)) => Some(state),
        Ok(None) => thread.epiphany_state().await,
        Err(error) => {
            tracing::error!(%error, store = %store.display(), "failed to read unified Epiphany state");
            None
        }
    }
}

pub(super) async fn client_visible_live_thread_epiphany_state(
    thread: &CodexThread,
    fallback: EpiphanyThreadState,
) -> EpiphanyThreadState {
    live_thread_epiphany_state(thread).await.unwrap_or(fallback)
}

pub(super) fn epiphany_token_usage_snapshot(
    info: Option<&TokenUsageInfo>,
) -> Option<EpiphanyTokenUsageSnapshot> {
    info.map(|info| EpiphanyTokenUsageSnapshot {
        total_tokens: info.total_token_usage.total_tokens,
        last_turn_tokens: info.last_token_usage.total_tokens,
        model_context_window: info.model_context_window,
        model_auto_compact_token_limit: info.model_auto_compact_token_limit,
    })
}
