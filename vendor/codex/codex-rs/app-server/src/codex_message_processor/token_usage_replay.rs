//! Replays persisted token usage snapshots when a client attaches to an existing thread.
//!
//! The message processor decides when replay is allowed and preserves JSON-RPC response
//! ordering. This module owns notification construction and the attribution rules that
//! map the latest persisted `TokenCount` back to a v2 turn id.
//!
//! Rollout histories can contain explicit turn ids or generated turn ids. When explicit
//! ids do not match the rebuilt thread, replay falls back to the active turn position at
//! the time the `TokenCount` was persisted so the notification still targets the
//! corresponding rebuilt turn.

use std::path::Path;
use std::sync::Arc;

use codex_app_server_protocol::ServerNotification;
use codex_app_server_protocol::Thread;
use codex_app_server_protocol::ThreadTokenUsage;
use codex_app_server_protocol::ThreadTokenUsageUpdatedNotification;
use codex_core::CodexThread;
use codex_protocol::ThreadId;
use codex_protocol::protocol::TokenUsageInfo;
use epiphany_codex_bridge::token_usage::latest_token_usage_info_from_rollout_items;
use epiphany_codex_bridge::token_usage::latest_token_usage_turn_id;
pub(super) use epiphany_codex_bridge::token_usage::latest_token_usage_turn_id_from_rollout_items;

use crate::codex_message_processor::read_rollout_items_from_rollout;
use crate::outgoing_message::ConnectionId;
use crate::outgoing_message::OutgoingMessageSender;

/// Sends a restored token usage update to the connection that attached to a thread.
///
/// This is lifecycle replay rather than a model event: the rollout already contains
/// the original `TokenCount`, and emitting through `send_event` here would duplicate
/// persisted usage records. Keeping this helper connection-scoped also avoids
/// surprising other subscribers with a historical usage update while they may be
/// rendering live turn events.
pub(super) async fn send_thread_token_usage_update_to_connection(
    outgoing: &Arc<OutgoingMessageSender>,
    connection_id: ConnectionId,
    thread_id: ThreadId,
    thread: &Thread,
    conversation: &CodexThread,
    token_usage_turn_id: Option<String>,
) {
    let Some(info) = conversation.token_usage_info().await else {
        return;
    };
    let notification = ThreadTokenUsageUpdatedNotification {
        thread_id: thread_id.to_string(),
        turn_id: token_usage_turn_id.unwrap_or_else(|| latest_token_usage_turn_id(thread)),
        token_usage: ThreadTokenUsage::from(info),
    };
    outgoing
        .send_server_notification_to_connections(
            &[connection_id],
            ServerNotification::ThreadTokenUsageUpdated(notification),
        )
        .await;
}

pub(super) async fn latest_token_usage_turn_id_for_thread_path(thread: &Thread) -> Option<String> {
    let rollout_path = thread.path.as_deref()?;
    latest_token_usage_turn_id_from_rollout_path(rollout_path, thread).await
}

pub(super) async fn latest_token_usage_turn_id_from_rollout_path(
    rollout_path: &Path,
    thread: &Thread,
) -> Option<String> {
    let rollout_items = read_rollout_items_from_rollout(rollout_path).await.ok()?;
    latest_token_usage_turn_id_from_rollout_items(&rollout_items, thread)
}

pub(super) async fn latest_token_usage_info_from_rollout_path(
    rollout_path: &Path,
) -> Option<TokenUsageInfo> {
    let rollout_items = read_rollout_items_from_rollout(rollout_path).await.ok()?;
    latest_token_usage_info_from_rollout_items(&rollout_items)
}
