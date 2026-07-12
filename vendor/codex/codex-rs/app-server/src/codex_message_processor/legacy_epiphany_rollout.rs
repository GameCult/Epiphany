use codex_protocol::models::ResponseItem;
use epiphany_state_model::EpiphanyThreadState;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::RolloutItem;

#[derive(Default)]
struct LegacySegment {
    turn_id: Option<String>,
    user_turn: bool,
    state: Option<EpiphanyThreadState>,
}

fn compatible(active: Option<&str>, item: Option<&str>) -> bool {
    active.is_none_or(|active| item.is_none_or(|item| item == active))
}

fn finish(segment: LegacySegment, state: &mut Option<EpiphanyThreadState>, rollbacks: &mut usize) {
    if *rollbacks > 0 && segment.user_turn {
        *rollbacks -= 1;
    } else if state.is_none()
        && (segment.user_turn || (segment.turn_id.is_none() && segment.state.is_some()))
    {
        *state = segment.state;
    }
}

fn is_out_of_band(segment: &LegacySegment) -> bool {
    segment.turn_id.is_none() && !segment.user_turn && segment.state.is_some()
}

pub(super) fn latest_legacy_epiphany_state(
    items: &[RolloutItem],
) -> Result<Option<EpiphanyThreadState>, String> {
    let mut state = None;
    let mut rollbacks = 0usize;
    let mut segment: Option<LegacySegment> = None;

    for item in items.iter().rev() {
        if segment.as_ref().is_some_and(is_out_of_band)
            && let Some(segment) = segment.take()
        {
            finish(segment, &mut state, &mut rollbacks);
        }
        if state.is_some() {
            break;
        }

        match item {
            RolloutItem::EventMsg(EventMsg::ThreadRolledBack(event)) => {
                rollbacks = rollbacks
                    .saturating_add(usize::try_from(event.num_turns).unwrap_or(usize::MAX));
            }
            RolloutItem::EventMsg(EventMsg::TurnComplete(event)) => {
                let segment = segment.get_or_insert_with(LegacySegment::default);
                segment.turn_id.get_or_insert_with(|| event.turn_id.clone());
            }
            RolloutItem::EventMsg(EventMsg::TurnAborted(event)) => {
                if let Some(turn_id) = &event.turn_id {
                    segment
                        .get_or_insert_with(LegacySegment::default)
                        .turn_id
                        .get_or_insert_with(|| turn_id.clone());
                }
            }
            RolloutItem::EventMsg(EventMsg::UserMessage(_)) => {
                segment.get_or_insert_with(LegacySegment::default).user_turn = true;
            }
            RolloutItem::ResponseItem(ResponseItem::Message { role, .. }) if role == "user" => {
                segment.get_or_insert_with(LegacySegment::default).user_turn = true;
            }
            RolloutItem::LegacyEpiphanyState(payload) => {
                let item: epiphany_state_model::EpiphanyStateItem =
                    serde_json::from_value(payload.clone()).map_err(|error| {
                        format!("invalid legacy Epiphany rollout payload: {error}")
                    })?;
                let segment = segment.get_or_insert_with(LegacySegment::default);
                if segment.turn_id.is_none() {
                    segment.turn_id = item.turn_id.clone();
                }
                if compatible(segment.turn_id.as_deref(), item.turn_id.as_deref())
                    && segment.state.is_none()
                {
                    segment.state = Some(item.state.clone());
                }
            }
            RolloutItem::EventMsg(EventMsg::TurnStarted(event)) => {
                if segment.as_ref().is_some_and(|segment| {
                    compatible(segment.turn_id.as_deref(), Some(event.turn_id.as_str()))
                }) && let Some(segment) = segment.take()
                {
                    finish(segment, &mut state, &mut rollbacks);
                }
            }
            RolloutItem::ResponseItem(_)
            | RolloutItem::EventMsg(_)
            | RolloutItem::Compacted(_)
            | RolloutItem::TurnContext(_)
            | RolloutItem::SessionMeta(_) => {}
        }
    }

    if let Some(segment) = segment {
        finish(segment, &mut state, &mut rollbacks);
    }
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::config_types::ModeKind;
    use epiphany_state_model::EpiphanyStateItem;
    use codex_protocol::protocol::ThreadRolledBackEvent;
    use codex_protocol::protocol::TurnCompleteEvent;
    use codex_protocol::protocol::TurnStartedEvent;
    use codex_protocol::protocol::UserMessageEvent;

    fn state(id: &str) -> EpiphanyThreadState {
        EpiphanyThreadState {
            revision: 1,
            last_updated_turn_id: Some(id.to_string()),
            ..Default::default()
        }
    }

    fn turn(id: &str, state: EpiphanyThreadState) -> Vec<RolloutItem> {
        vec![
            RolloutItem::EventMsg(EventMsg::TurnStarted(TurnStartedEvent {
                turn_id: id.to_string(),
                started_at: None,
                model_context_window: None,
                collaboration_mode_kind: ModeKind::Default,
            })),
            RolloutItem::EventMsg(EventMsg::UserMessage(UserMessageEvent {
                message: id.to_string(),
                images: None,
                local_images: Vec::new(),
                text_elements: Vec::new(),
            })),
            RolloutItem::LegacyEpiphanyState(
                serde_json::to_value(EpiphanyStateItem {
                    turn_id: Some(id.to_string()),
                    state,
                })
                .expect("serialize legacy Epiphany payload"),
            ),
            RolloutItem::EventMsg(EventMsg::TurnComplete(TurnCompleteEvent {
                turn_id: id.to_string(),
                last_agent_message: None,
                completed_at: None,
                duration_ms: None,
                time_to_first_token_ms: None,
            })),
        ]
    }

    #[test]
    fn returns_latest_surviving_legacy_state() {
        let first = state("one");
        let second = state("two");
        let mut items = turn("one", first);
        items.extend(turn("two", second.clone()));
        assert_eq!(latest_legacy_epiphany_state(&items), Ok(Some(second)));
    }

    #[test]
    fn skips_rolled_back_legacy_state() {
        let first = state("one");
        let mut items = turn("one", first.clone());
        items.extend(turn("two", state("two")));
        items.push(RolloutItem::EventMsg(EventMsg::ThreadRolledBack(
            ThreadRolledBackEvent { num_turns: 1 },
        )));
        assert_eq!(latest_legacy_epiphany_state(&items), Ok(Some(first)));
    }

    #[test]
    fn rejects_malformed_legacy_state_payload() {
        let items = vec![RolloutItem::LegacyEpiphanyState(serde_json::json!({
            "turn_id": "not-the-legacy-shape"
        }))];

        assert!(latest_legacy_epiphany_state(&items).is_err());
    }
}
