use codex_core::CodexThread;
use codex_protocol::protocol::EpiphanyThreadState;

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
