use std::path::Path;

use codex_protocol::models::ResponseItem;
use codex_protocol::protocol::EventMsg;
use codex_protocol::protocol::RolloutItem;
use codex_protocol::protocol::RolloutLine;
use epiphany_state_model::EpiphanyStateItem;
use epiphany_state_model::EpiphanyThreadState;

enum LegacyReplayItem {
    Codex(RolloutItem),
    Epiphany(EpiphanyStateItem),
}

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

fn latest_legacy_epiphany_state(
    items: &[LegacyReplayItem],
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
            LegacyReplayItem::Codex(RolloutItem::EventMsg(EventMsg::ThreadRolledBack(event))) => {
                rollbacks = rollbacks
                    .saturating_add(usize::try_from(event.num_turns).unwrap_or(usize::MAX));
            }
            LegacyReplayItem::Codex(RolloutItem::EventMsg(EventMsg::TurnComplete(event))) => {
                let segment = segment.get_or_insert_with(LegacySegment::default);
                segment.turn_id.get_or_insert_with(|| event.turn_id.clone());
            }
            LegacyReplayItem::Codex(RolloutItem::EventMsg(EventMsg::TurnAborted(event))) => {
                if let Some(turn_id) = &event.turn_id {
                    segment
                        .get_or_insert_with(LegacySegment::default)
                        .turn_id
                        .get_or_insert_with(|| turn_id.clone());
                }
            }
            LegacyReplayItem::Codex(RolloutItem::EventMsg(EventMsg::UserMessage(_))) => {
                segment.get_or_insert_with(LegacySegment::default).user_turn = true;
            }
            LegacyReplayItem::Codex(RolloutItem::ResponseItem(ResponseItem::Message {
                role,
                ..
            })) if role == "user" => {
                segment.get_or_insert_with(LegacySegment::default).user_turn = true;
            }
            LegacyReplayItem::Epiphany(item) => {
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
            LegacyReplayItem::Codex(RolloutItem::EventMsg(EventMsg::TurnStarted(event))) => {
                if segment.as_ref().is_some_and(|segment| {
                    compatible(segment.turn_id.as_deref(), Some(event.turn_id.as_str()))
                }) && let Some(segment) = segment.take()
                {
                    finish(segment, &mut state, &mut rollbacks);
                }
            }
            LegacyReplayItem::Codex(_) => {}
        }
    }

    if let Some(segment) = segment {
        finish(segment, &mut state, &mut rollbacks);
    }
    Ok(state)
}

pub(super) async fn load_epiphany_state_from_rollout_path(
    path: &Path,
) -> Result<Option<EpiphanyThreadState>, String> {
    let text = tokio::fs::read_to_string(path)
        .await
        .map_err(|error| format!("failed to read rollout `{}`: {error}", path.display()))?;
    let mut items = Vec::new();
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        if let Some(item) = parse_replay_line(line)? {
            items.push(item);
        }
    }
    latest_legacy_epiphany_state(&items)
}

fn parse_replay_line(line: &str) -> Result<Option<LegacyReplayItem>, String> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(line) else {
        return Ok(None);
    };
    if value.get("type").and_then(serde_json::Value::as_str) == Some("epiphany_state") {
        let payload = value
            .get("payload")
            .cloned()
            .ok_or_else(|| "legacy Epiphany rollout item is missing payload".to_string())?;
        let item = serde_json::from_value(payload)
            .map_err(|error| format!("invalid legacy Epiphany rollout payload: {error}"))?;
        Ok(Some(LegacyReplayItem::Epiphany(item)))
    } else {
        Ok(serde_json::from_value::<RolloutLine>(value)
            .ok()
            .map(|line| LegacyReplayItem::Codex(line.item)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codex_protocol::config_types::ModeKind;
    use codex_protocol::protocol::ThreadRolledBackEvent;
    use codex_protocol::protocol::TurnCompleteEvent;
    use codex_protocol::protocol::TurnStartedEvent;
    use codex_protocol::protocol::UserMessageEvent;

    #[test]
    fn codex_event_loop_has_no_epiphany_automation_authority() {
        let event_loop = include_str!("../bespoke_event_handling.rs");
        let processor = include_str!("../codex_message_processor.rs");
        let thread_state = include_str!("../thread_state.rs");

        assert!(!event_loop.contains("maybe_run_epiphany"));
        assert!(!processor.contains("mod epiphany_automation"));
        assert!(!thread_state.contains("epiphany_checkpoint_intervention"));
    }
    use epiphany_state_model::EpiphanyStateItem;

    fn state(id: &str) -> EpiphanyThreadState {
        EpiphanyThreadState {
            revision: 1,
            last_updated_turn_id: Some(id.to_string()),
            ..Default::default()
        }
    }

    fn turn(id: &str, state: EpiphanyThreadState) -> Vec<LegacyReplayItem> {
        vec![
            LegacyReplayItem::Codex(RolloutItem::EventMsg(EventMsg::TurnStarted(
                TurnStartedEvent {
                    turn_id: id.to_string(),
                    started_at: None,
                    model_context_window: None,
                    collaboration_mode_kind: ModeKind::Default,
                },
            ))),
            LegacyReplayItem::Codex(RolloutItem::EventMsg(EventMsg::UserMessage(
                UserMessageEvent {
                    message: id.to_string(),
                    images: None,
                    local_images: Vec::new(),
                    text_elements: Vec::new(),
                },
            ))),
            LegacyReplayItem::Epiphany(EpiphanyStateItem {
                turn_id: Some(id.to_string()),
                state,
            }),
            LegacyReplayItem::Codex(RolloutItem::EventMsg(EventMsg::TurnComplete(
                TurnCompleteEvent {
                    turn_id: id.to_string(),
                    last_agent_message: None,
                    completed_at: None,
                    duration_ms: None,
                    time_to_first_token_ms: None,
                },
            ))),
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
        items.push(LegacyReplayItem::Codex(RolloutItem::EventMsg(
            EventMsg::ThreadRolledBack(ThreadRolledBackEvent { num_turns: 1 }),
        )));
        assert_eq!(latest_legacy_epiphany_state(&items), Ok(Some(first)));
    }

    #[test]
    fn rejects_malformed_legacy_state_payload() {
        let line = serde_json::json!({
            "timestamp": "2026-07-12T00:00:00Z",
            "type": "epiphany_state",
            "payload": { "turn_id": "not-the-legacy-shape" }
        });
        assert!(parse_replay_line(&line.to_string()).is_err());
    }

    #[test]
    fn recognizes_only_the_historical_raw_line_tag() {
        let expected = state("raw-line");
        let line = serde_json::json!({
            "timestamp": "2026-07-12T00:00:00Z",
            "type": "epiphany_state",
            "payload": {
                "turn_id": "raw-line",
                "state": expected
            }
        });
        let Some(LegacyReplayItem::Epiphany(item)) =
            parse_replay_line(&line.to_string()).expect("parse historical line")
        else {
            panic!("expected quarantined Epiphany migration item");
        };
        assert_eq!(item.turn_id.as_deref(), Some("raw-line"));
        assert_eq!(item.state, state("raw-line"));

        assert!(parse_replay_line(r#"{"type":"future_unknown","payload":{}}"#)
            .expect("unknown line should remain nonfatal to migration scan")
            .is_none());
    }
}
