use super::*;

pub(crate) struct ThreadTurnsPage {
    pub(crate) turns: Vec<Turn>,
    pub(crate) next_cursor: Option<String>,
    pub(crate) backwards_cursor: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ThreadTurnsCursor {
    turn_id: String,
    include_anchor: bool,
}

pub(crate) fn paginate_thread_turns(
    turns: Vec<Turn>,
    cursor: Option<&str>,
    limit: Option<u32>,
    sort_direction: SortDirection,
) -> Result<ThreadTurnsPage, JSONRPCErrorError> {
    if turns.is_empty() {
        return Ok(ThreadTurnsPage {
            turns: Vec::new(),
            next_cursor: None,
            backwards_cursor: None,
        });
    }

    let anchor = cursor.map(parse_thread_turns_cursor).transpose()?;
    let page_size = limit
        .map(|value| value as usize)
        .unwrap_or(THREAD_TURNS_DEFAULT_LIMIT)
        .clamp(1, THREAD_TURNS_MAX_LIMIT);

    let anchor_index = anchor
        .as_ref()
        .and_then(|anchor| turns.iter().position(|turn| turn.id == anchor.turn_id));
    if anchor.is_some() && anchor_index.is_none() {
        return Err(JSONRPCErrorError {
            code: INVALID_REQUEST_ERROR_CODE,
            message: "invalid cursor: anchor turn is no longer present".to_string(),
            data: None,
        });
    }

    let mut keyed_turns: Vec<_> = turns.into_iter().enumerate().collect();
    match sort_direction {
        SortDirection::Asc => {
            if let (Some(anchor), Some(anchor_index)) = (anchor.as_ref(), anchor_index) {
                keyed_turns.retain(|(index, _)| {
                    if anchor.include_anchor {
                        *index >= anchor_index
                    } else {
                        *index > anchor_index
                    }
                });
            }
        }
        SortDirection::Desc => {
            keyed_turns.reverse();
            if let (Some(anchor), Some(anchor_index)) = (anchor.as_ref(), anchor_index) {
                keyed_turns.retain(|(index, _)| {
                    if anchor.include_anchor {
                        *index <= anchor_index
                    } else {
                        *index < anchor_index
                    }
                });
            }
        }
    }

    let more_turns_available = keyed_turns.len() > page_size;
    keyed_turns.truncate(page_size);
    let backwards_cursor = keyed_turns
        .first()
        .map(|(_, turn)| serialize_thread_turns_cursor(&turn.id, /*include_anchor*/ true))
        .transpose()?;
    let next_cursor = if more_turns_available {
        keyed_turns
            .last()
            .map(|(_, turn)| serialize_thread_turns_cursor(&turn.id, /*include_anchor*/ false))
            .transpose()?
    } else {
        None
    };
    let turns = keyed_turns.into_iter().map(|(_, turn)| turn).collect();

    Ok(ThreadTurnsPage {
        turns,
        next_cursor,
        backwards_cursor,
    })
}

fn serialize_thread_turns_cursor(
    turn_id: &str,
    include_anchor: bool,
) -> Result<String, JSONRPCErrorError> {
    serde_json::to_string(&ThreadTurnsCursor {
        turn_id: turn_id.to_string(),
        include_anchor,
    })
    .map_err(|err| JSONRPCErrorError {
        code: INTERNAL_ERROR_CODE,
        message: format!("failed to serialize cursor: {err}"),
        data: None,
    })
}

fn parse_thread_turns_cursor(cursor: &str) -> Result<ThreadTurnsCursor, JSONRPCErrorError> {
    serde_json::from_str(cursor).map_err(|_| JSONRPCErrorError {
        code: INVALID_REQUEST_ERROR_CODE,
        message: format!("invalid cursor: {cursor}"),
        data: None,
    })
}

pub(crate) enum ThreadTurnSource<'a> {
    RolloutPath(&'a Path),
    HistoryItems(&'a [RolloutItem]),
}

pub(crate) async fn populate_thread_turns(
    thread: &mut Thread,
    turn_source: ThreadTurnSource<'_>,
    active_turn: Option<&Turn>,
) -> std::result::Result<(), String> {
    let mut turns = match turn_source {
        ThreadTurnSource::RolloutPath(rollout_path) => {
            read_rollout_items_from_rollout(rollout_path)
                .await
                .map(|items| build_turns_from_rollout_items(&items))
                .map_err(|err| {
                    format!(
                        "failed to load rollout `{}` for thread {}: {err}",
                        rollout_path.display(),
                        thread.id
                    )
                })?
        }
        ThreadTurnSource::HistoryItems(items) => build_turns_from_rollout_items(items),
    };
    if let Some(active_turn) = active_turn {
        merge_turn_history_with_active_turn(&mut turns, active_turn.clone());
    }
    thread.turns = turns;
    Ok(())
}

fn merge_turn_history_with_active_turn(turns: &mut Vec<Turn>, active_turn: Turn) {
    turns.retain(|turn| turn.id != active_turn.id);
    turns.push(active_turn);
}

pub(crate) fn set_thread_status_and_interrupt_stale_turns(
    thread: &mut Thread,
    loaded_status: ThreadStatus,
    has_live_in_progress_turn: bool,
) {
    let status = resolve_thread_status(loaded_status, has_live_in_progress_turn);
    if !matches!(status, ThreadStatus::Active { .. }) {
        for turn in &mut thread.turns {
            if matches!(turn.status, TurnStatus::InProgress) {
                turn.status = TurnStatus::Interrupted;
            }
        }
    }
    thread.status = status;
}

pub(crate) fn reconstruct_thread_turns_from_rollout_items(
    items: &[RolloutItem],
    loaded_status: ThreadStatus,
    has_live_in_progress_turn: bool,
) -> Vec<Turn> {
    let mut turns = build_turns_from_rollout_items(items);
    normalize_thread_turns_status(&mut turns, loaded_status, has_live_in_progress_turn);
    turns
}

fn normalize_thread_turns_status(
    turns: &mut [Turn],
    loaded_status: ThreadStatus,
    has_live_in_progress_turn: bool,
) {
    let status = resolve_thread_status(loaded_status, has_live_in_progress_turn);
    if matches!(status, ThreadStatus::Active { .. }) {
        return;
    }
    for turn in turns {
        if matches!(turn.status, TurnStatus::InProgress) {
            turn.status = TurnStatus::Interrupted;
        }
    }
}
