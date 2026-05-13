use crate::context_manager;
use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::EpiphanyThreadState;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::RolloutItem;

#[derive(Debug, Default)]
struct ActiveEpiphanyReplaySegment {
    turn_id: Option<String>,
    counts_as_user_turn: bool,
    epiphany_state: Option<EpiphanyThreadState>,
}

fn turn_ids_are_compatible(active_turn_id: Option<&str>, item_turn_id: Option<&str>) -> bool {
    active_turn_id
        .is_none_or(|turn_id| item_turn_id.is_none_or(|item_turn_id| item_turn_id == turn_id))
}

fn finalize_active_segment(
    active_segment: ActiveEpiphanyReplaySegment,
    epiphany_state: &mut Option<EpiphanyThreadState>,
    pending_rollback_turns: &mut usize,
) {
    if *pending_rollback_turns > 0 && active_segment.counts_as_user_turn {
        *pending_rollback_turns -= 1;
        return;
    }

    if epiphany_state.is_none()
        && (active_segment.counts_as_user_turn
            || (active_segment.turn_id.is_none() && active_segment.epiphany_state.is_some()))
    {
        *epiphany_state = active_segment.epiphany_state;
    }
}

fn is_out_of_band_epiphany_segment(active_segment: &ActiveEpiphanyReplaySegment) -> bool {
    active_segment.turn_id.is_none()
        && !active_segment.counts_as_user_turn
        && active_segment.epiphany_state.is_some()
}

pub fn latest_epiphany_state_from_rollout_items<F>(
    rollout_items: &[RolloutItem],
    is_user_turn_boundary: F,
) -> Option<EpiphanyThreadState>
where
    F: Fn(&ResponseItem) -> bool,
{
    let mut epiphany_state = None;
    let mut pending_rollback_turns = 0usize;
    let mut active_segment: Option<ActiveEpiphanyReplaySegment> = None;

    for item in rollout_items.iter().rev() {
        if active_segment
            .as_ref()
            .is_some_and(is_out_of_band_epiphany_segment)
            && let Some(active_segment) = active_segment.take()
        {
            finalize_active_segment(
                active_segment,
                &mut epiphany_state,
                &mut pending_rollback_turns,
            );
        }
        if epiphany_state.is_some() {
            break;
        }

        match item {
            RolloutItem::EventMsg(EventMsg::ThreadRolledBack(rollback)) => {
                pending_rollback_turns = pending_rollback_turns
                    .saturating_add(usize::try_from(rollback.num_turns).unwrap_or(usize::MAX));
            }
            RolloutItem::EventMsg(EventMsg::TurnComplete(event)) => {
                let active_segment =
                    active_segment.get_or_insert_with(ActiveEpiphanyReplaySegment::default);
                if active_segment.turn_id.is_none() {
                    active_segment.turn_id = Some(event.turn_id.clone());
                }
            }
            RolloutItem::EventMsg(EventMsg::TurnAborted(event)) => {
                if let Some(active_segment) = active_segment.as_mut() {
                    if active_segment.turn_id.is_none()
                        && let Some(turn_id) = &event.turn_id
                    {
                        active_segment.turn_id = Some(turn_id.clone());
                    }
                } else if let Some(turn_id) = &event.turn_id {
                    active_segment = Some(ActiveEpiphanyReplaySegment {
                        turn_id: Some(turn_id.clone()),
                        ..Default::default()
                    });
                }
            }
            RolloutItem::EventMsg(EventMsg::UserMessage(_)) => {
                let active_segment =
                    active_segment.get_or_insert_with(ActiveEpiphanyReplaySegment::default);
                active_segment.counts_as_user_turn = true;
            }
            RolloutItem::ResponseItem(response_item) => {
                let active_segment =
                    active_segment.get_or_insert_with(ActiveEpiphanyReplaySegment::default);
                active_segment.counts_as_user_turn |= is_user_turn_boundary(response_item);
            }
            RolloutItem::EpiphanyState(item) => {
                let active_segment =
                    active_segment.get_or_insert_with(ActiveEpiphanyReplaySegment::default);
                if active_segment.turn_id.is_none() {
                    active_segment.turn_id = item.turn_id.clone();
                }
                if turn_ids_are_compatible(
                    active_segment.turn_id.as_deref(),
                    item.turn_id.as_deref(),
                ) && active_segment.epiphany_state.is_none()
                {
                    active_segment.epiphany_state = Some(item.state.clone());
                }
            }
            RolloutItem::EventMsg(EventMsg::TurnStarted(event)) => {
                if active_segment.as_ref().is_some_and(|active_segment| {
                    turn_ids_are_compatible(
                        active_segment.turn_id.as_deref(),
                        Some(event.turn_id.as_str()),
                    )
                }) && let Some(active_segment) = active_segment.take()
                {
                    finalize_active_segment(
                        active_segment,
                        &mut epiphany_state,
                        &mut pending_rollback_turns,
                    );
                }
            }
            RolloutItem::EventMsg(_)
            | RolloutItem::Compacted(_)
            | RolloutItem::TurnContext(_)
            | RolloutItem::SessionMeta(_) => {}
        }

        if epiphany_state.is_some() {
            break;
        }
    }

    if let Some(active_segment) = active_segment.take() {
        finalize_active_segment(
            active_segment,
            &mut epiphany_state,
            &mut pending_rollback_turns,
        );
    }

    epiphany_state
}

pub fn latest_epiphany_state_from_codex_rollout_items(
    rollout_items: &[RolloutItem],
) -> Option<EpiphanyThreadState> {
    latest_epiphany_state_from_rollout_items(rollout_items, context_manager::is_user_turn_boundary)
}

#[cfg(test)]
mod tests {
    use super::latest_epiphany_state_from_rollout_items;
    use codex_protocol::models::ContentItem;
    use codex_protocol::models::ResponseItem;
    use codex_protocol::protocol::CompactedItem;
    use codex_protocol::protocol::EpiphanyStateItem;
    use codex_protocol::protocol::EpiphanyThreadState;
    use codex_protocol::protocol::EventMsg;
    use codex_protocol::protocol::RolloutItem;
    use codex_protocol::protocol::ThreadRolledBackEvent;
    use codex_protocol::protocol::TurnCompleteEvent;
    use codex_protocol::protocol::TurnStartedEvent;
    use codex_protocol::protocol::UserMessageEvent;

    fn simple_is_user_turn_boundary(item: &ResponseItem) -> bool {
        matches!(
            item,
            ResponseItem::Message { role, .. } if role == "user"
        )
    }

    fn user_response_message(text: &str) -> ResponseItem {
        ResponseItem::Message {
            id: None,
            role: "user".to_string(),
            content: vec![ContentItem::InputText {
                text: text.to_string(),
            }],
            end_turn: None,
            phase: None,
        }
    }

    fn sample_epiphany_state(turn_id: &str) -> EpiphanyThreadState {
        EpiphanyThreadState {
            revision: 1,
            objective: Some(format!("Objective for {turn_id}")),
            active_subgoal_id: Some(format!("subgoal-{turn_id}")),
            last_updated_turn_id: Some(turn_id.to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn latest_epiphany_state_from_rollout_items_returns_latest_surviving_snapshot() {
        let first = sample_epiphany_state("turn-1");
        let second = sample_epiphany_state("turn-2");
        let rollout_items = vec![
            RolloutItem::EventMsg(EventMsg::TurnStarted(TurnStartedEvent {
                turn_id: "turn-1".to_string(),
                started_at: None,
                model_context_window: None,
                collaboration_mode_kind: Default::default(),
            })),
            RolloutItem::EventMsg(EventMsg::UserMessage(UserMessageEvent {
                message: "first".to_string(),
                images: None,
                text_elements: Vec::new(),
                local_images: Vec::new(),
            })),
            RolloutItem::EpiphanyState(EpiphanyStateItem {
                turn_id: Some("turn-1".to_string()),
                state: first,
            }),
            RolloutItem::EventMsg(EventMsg::TurnComplete(TurnCompleteEvent {
                turn_id: "turn-1".to_string(),
                last_agent_message: None,
                completed_at: None,
                duration_ms: None,
                time_to_first_token_ms: None,
            })),
            RolloutItem::EventMsg(EventMsg::TurnStarted(TurnStartedEvent {
                turn_id: "turn-2".to_string(),
                started_at: None,
                model_context_window: None,
                collaboration_mode_kind: Default::default(),
            })),
            RolloutItem::ResponseItem(user_response_message("second")),
            RolloutItem::EpiphanyState(EpiphanyStateItem {
                turn_id: Some("turn-2".to_string()),
                state: second.clone(),
            }),
            RolloutItem::EventMsg(EventMsg::TurnComplete(TurnCompleteEvent {
                turn_id: "turn-2".to_string(),
                last_agent_message: None,
                completed_at: None,
                duration_ms: None,
                time_to_first_token_ms: None,
            })),
        ];

        assert_eq!(
            latest_epiphany_state_from_rollout_items(&rollout_items, simple_is_user_turn_boundary),
            Some(second)
        );
    }

    #[test]
    fn latest_epiphany_state_from_rollout_items_skips_rolled_back_turns() {
        let first = sample_epiphany_state("turn-1");
        let second = sample_epiphany_state("turn-2");
        let rollout_items = vec![
            RolloutItem::EventMsg(EventMsg::TurnStarted(TurnStartedEvent {
                turn_id: "turn-1".to_string(),
                started_at: None,
                model_context_window: None,
                collaboration_mode_kind: Default::default(),
            })),
            RolloutItem::EventMsg(EventMsg::UserMessage(UserMessageEvent {
                message: "first".to_string(),
                images: None,
                text_elements: Vec::new(),
                local_images: Vec::new(),
            })),
            RolloutItem::EpiphanyState(EpiphanyStateItem {
                turn_id: Some("turn-1".to_string()),
                state: first.clone(),
            }),
            RolloutItem::EventMsg(EventMsg::TurnComplete(TurnCompleteEvent {
                turn_id: "turn-1".to_string(),
                last_agent_message: None,
                completed_at: None,
                duration_ms: None,
                time_to_first_token_ms: None,
            })),
            RolloutItem::EventMsg(EventMsg::TurnStarted(TurnStartedEvent {
                turn_id: "turn-2".to_string(),
                started_at: None,
                model_context_window: None,
                collaboration_mode_kind: Default::default(),
            })),
            RolloutItem::EventMsg(EventMsg::UserMessage(UserMessageEvent {
                message: "second".to_string(),
                images: None,
                text_elements: Vec::new(),
                local_images: Vec::new(),
            })),
            RolloutItem::EpiphanyState(EpiphanyStateItem {
                turn_id: Some("turn-2".to_string()),
                state: second,
            }),
            RolloutItem::EventMsg(EventMsg::TurnComplete(TurnCompleteEvent {
                turn_id: "turn-2".to_string(),
                last_agent_message: None,
                completed_at: None,
                duration_ms: None,
                time_to_first_token_ms: None,
            })),
            RolloutItem::EventMsg(EventMsg::ThreadRolledBack(ThreadRolledBackEvent {
                num_turns: 1,
            })),
        ];

        assert_eq!(
            latest_epiphany_state_from_rollout_items(&rollout_items, simple_is_user_turn_boundary),
            Some(first)
        );
    }

    #[test]
    fn latest_epiphany_state_from_rollout_items_survives_compaction() {
        let state = sample_epiphany_state("turn-before-compaction");
        let rollout_items = vec![
            RolloutItem::EventMsg(EventMsg::TurnStarted(TurnStartedEvent {
                turn_id: "turn-before-compaction".to_string(),
                started_at: None,
                model_context_window: None,
                collaboration_mode_kind: Default::default(),
            })),
            RolloutItem::EventMsg(EventMsg::UserMessage(UserMessageEvent {
                message: "before compaction".to_string(),
                images: None,
                text_elements: Vec::new(),
                local_images: Vec::new(),
            })),
            RolloutItem::EpiphanyState(EpiphanyStateItem {
                turn_id: Some("turn-before-compaction".to_string()),
                state: state.clone(),
            }),
            RolloutItem::EventMsg(EventMsg::TurnComplete(TurnCompleteEvent {
                turn_id: "turn-before-compaction".to_string(),
                last_agent_message: None,
                completed_at: None,
                duration_ms: None,
                time_to_first_token_ms: None,
            })),
            RolloutItem::Compacted(CompactedItem {
                message: "compact".to_string(),
                replacement_history: None,
            }),
        ];

        assert_eq!(
            latest_epiphany_state_from_rollout_items(&rollout_items, simple_is_user_turn_boundary),
            Some(state)
        );
    }

    #[test]
    fn latest_epiphany_state_from_rollout_items_accepts_out_of_band_snapshot() {
        let state = sample_epiphany_state("control-plane-update");
        let rollout_items = vec![RolloutItem::EpiphanyState(EpiphanyStateItem {
            turn_id: None,
            state: state.clone(),
        })];

        assert_eq!(
            latest_epiphany_state_from_rollout_items(&rollout_items, simple_is_user_turn_boundary),
            Some(state)
        );
    }

    #[test]
    fn latest_epiphany_state_from_rollout_items_accepts_out_of_band_snapshot_after_turn() {
        let turn_state = sample_epiphany_state("turn-before-control-plane-update");
        let control_plane_state = sample_epiphany_state("control-plane-update");
        let rollout_items = vec![
            RolloutItem::EventMsg(EventMsg::TurnStarted(TurnStartedEvent {
                turn_id: "turn-before-control-plane-update".to_string(),
                started_at: None,
                model_context_window: None,
                collaboration_mode_kind: Default::default(),
            })),
            RolloutItem::EventMsg(EventMsg::UserMessage(UserMessageEvent {
                message: "before control-plane update".to_string(),
                images: None,
                text_elements: Vec::new(),
                local_images: Vec::new(),
            })),
            RolloutItem::EpiphanyState(EpiphanyStateItem {
                turn_id: Some("turn-before-control-plane-update".to_string()),
                state: turn_state,
            }),
            RolloutItem::EventMsg(EventMsg::TurnComplete(TurnCompleteEvent {
                turn_id: "turn-before-control-plane-update".to_string(),
                last_agent_message: None,
                completed_at: None,
                duration_ms: None,
                time_to_first_token_ms: None,
            })),
            RolloutItem::EpiphanyState(EpiphanyStateItem {
                turn_id: None,
                state: control_plane_state.clone(),
            }),
        ];

        assert_eq!(
            latest_epiphany_state_from_rollout_items(&rollout_items, simple_is_user_turn_boundary),
            Some(control_plane_state)
        );
    }
}
