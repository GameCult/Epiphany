use codex_app_server_protocol::Thread;
use codex_app_server_protocol::ThreadHistoryBuilder;
use codex_app_server_protocol::TurnStatus;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::RolloutItem;
use codex_protocol::protocol::TokenUsageInfo;

pub fn latest_token_usage_info_from_rollout_items(
    rollout_items: &[RolloutItem],
) -> Option<TokenUsageInfo> {
    rollout_items.iter().rev().find_map(|item| match item {
        RolloutItem::EventMsg(EventMsg::TokenCount(event)) => event.info.clone(),
        _ => None,
    })
}

/// Identifies the turn that was active when a `TokenCount` record appeared.
///
/// The id is preferred when it still appears in the rebuilt thread. The position
/// is a fallback for histories whose implicit turn ids are regenerated during
/// reconstruction.
struct TokenUsageTurnOwner {
    id: String,
    position: Option<usize>,
}

pub fn latest_token_usage_turn_id_from_rollout_items(
    rollout_items: &[RolloutItem],
    thread: &Thread,
) -> Option<String> {
    let owner = latest_token_usage_turn_owner_from_rollout_items(rollout_items)?;
    if thread.turns.iter().any(|turn| turn.id == owner.id) {
        return Some(owner.id);
    }
    owner
        .position
        .and_then(|position| thread.turns.get(position))
        .map(|turn| turn.id.clone())
}

fn latest_token_usage_turn_owner_from_rollout_items(
    rollout_items: &[RolloutItem],
) -> Option<TokenUsageTurnOwner> {
    let mut builder = ThreadHistoryBuilder::new();
    let mut token_usage_turn_owner = None;

    for item in rollout_items {
        if matches!(item, RolloutItem::EventMsg(EventMsg::TokenCount(_))) {
            token_usage_turn_owner =
                builder
                    .active_turn_snapshot()
                    .map(|turn| TokenUsageTurnOwner {
                        id: turn.id,
                        position: builder.active_turn_position(),
                    });
        }
        builder.handle_rollout_item(item);
    }

    token_usage_turn_owner
}

/// Chooses a fallback turn id that should own a replayed token usage update.
///
/// Normal replay derives the owner from the rollout position of the latest
/// `TokenCount` event. This fallback only preserves a stable wire shape for
/// unusual histories where that rollout information cannot be read.
pub fn latest_token_usage_turn_id(thread: &Thread) -> String {
    thread
        .turns
        .iter()
        .rev()
        .find(|turn| matches!(turn.status, TurnStatus::Completed | TurnStatus::Failed))
        .or_else(|| thread.turns.last())
        .map(|turn| turn.id.clone())
        .unwrap_or_default()
}
