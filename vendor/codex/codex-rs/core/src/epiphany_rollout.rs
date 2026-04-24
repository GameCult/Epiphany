use crate::context_manager::is_user_turn_boundary;
use codex_protocol::protocol::EpiphanyThreadState;
use codex_protocol::protocol::RolloutItem;

pub fn latest_epiphany_state_from_rollout_items(
    rollout_items: &[RolloutItem],
) -> Option<EpiphanyThreadState> {
    epiphany_core::latest_epiphany_state_from_rollout_items(rollout_items, is_user_turn_boundary)
}
