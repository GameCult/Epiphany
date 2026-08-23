use super::HEARTBEAT_STATE_KEY;
use super::HEARTBEAT_STATE_TYPE;
use super::HEARTBEAT_STATUS_SCHEMA_VERSION;
use super::HeartbeatHistoryEvent;
use super::HeartbeatParticipant;
use super::HeartbeatPendingTurn;
use super::HeartbeatSelectionPolicy;
use super::load_heartbeat_state_entry;
use super::participant_arena;
use super::participant_kind;
use anyhow::Result;
use serde_json::Value;
use std::cmp::Reverse;
use std::fs;
use std::path::Path;

pub fn heartbeat_status_projection(
    store_path: impl AsRef<Path>,
    artifact_dir: impl AsRef<Path>,
    target_heartbeat_rate: f64,
    artifact_limit: usize,
) -> Result<Value> {
    let store_path = store_path.as_ref();
    let Some(state) = load_heartbeat_state_entry(store_path)? else {
        return Ok(serde_json::json!({
            "schema_version": HEARTBEAT_STATUS_SCHEMA_VERSION,
            "ok": true,
            "status": "missing",
            "schedulerStatus": "missing",
            "stateFile": null,
            "storeFile": store_path,
            "cultCacheStore": cultcache_status(store_path),
            "artifactDir": artifact_dir.as_ref(),
            "targetHeartbeatRate": if target_heartbeat_rate > 0.0 { Some(target_heartbeat_rate) } else { None },
            "sceneClock": null,
            "participants": [],
            "latestEvent": null,
            "history": [],
            "latestArtifacts": latest_json_artifacts(artifact_dir, artifact_limit),
            "availableActions": ["init", "repair-stale", "retain-artifacts", "status", "serve"],
        }));
    };
    let scheduler_status = if state.participants.is_empty() {
        "unconfigured"
    } else if state.participants.iter().all(|participant| {
        participant.status == "active"
            && participant
                .pending_turn
                .as_ref()
                .is_none_or(|turn| turn.status == "running")
    }) {
        "active"
    } else {
        "attention"
    };
    let history: Vec<_> = state
        .history
        .iter()
        .rev()
        .take(artifact_limit)
        .cloned()
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .map(history_event_json)
        .collect();
    Ok(serde_json::json!({
        "schema_version": HEARTBEAT_STATUS_SCHEMA_VERSION,
        "ok": true,
        "status": "loaded",
        "schedulerStatus": scheduler_status,
        "stateFile": null,
        "storeFile": store_path,
        "cultCacheStore": cultcache_status(store_path),
        "artifactDir": artifact_dir.as_ref(),
        "targetHeartbeatRate": state.target_heartbeat_rate,
        "sceneClock": state.scene_clock,
        "participants": state.participants.iter().map(participant_status_json).collect::<Vec<_>>(),
        "latestEvent": history.last().cloned(),
        "history": history,
        "latestArtifacts": latest_json_artifacts(artifact_dir, artifact_limit),
        "availableActions": ["init", "repair-stale", "retain-artifacts", "status", "serve"],
    }))
}

fn latest_json_artifacts(artifact_dir: impl AsRef<Path>, limit: usize) -> Vec<Value> {
    let artifact_dir = artifact_dir.as_ref();
    let Ok(read_dir) = fs::read_dir(artifact_dir) else {
        return Vec::new();
    };
    let mut paths: Vec<_> = read_dir
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "json")
        })
        .filter_map(|path| {
            let modified = path.metadata().and_then(|meta| meta.modified()).ok()?;
            Some((path, modified))
        })
        .collect();
    paths.sort_by_key(|item| Reverse(item.1));
    paths
        .into_iter()
        .take(limit)
        .filter_map(|(path, modified)| {
            let raw = fs::read_to_string(&path).ok()?;
            let payload: Value = serde_json::from_str(&raw).ok()?;
            let modified_at: chrono::DateTime<chrono::Utc> = modified.into();
            Some(serde_json::json!({
                "path": path,
                "name": path.file_name().and_then(|name| name.to_str()),
                "modifiedAt": modified_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                "schemaVersion": payload.get("schema_version"),
                "kind": path.file_stem().and_then(|stem| stem.to_str()).and_then(|stem| stem.rsplit('.').next()).unwrap_or("json"),
                "summary": artifact_summary(&payload),
            }))
        })
        .collect()
}

fn artifact_summary(payload: &Value) -> Value {
    let event = if payload.get("actionId").is_some() {
        Some(payload)
    } else {
        payload.get("event")
    };
    if let Some(event) = event {
        return serde_json::json!({
            "selectedRole": event.get("selectedRole"),
            "actionType": event.get("actionType"),
            "actionId": event.get("actionId"),
            "coordinatorAction": event.get("coordinatorAction"),
        });
    }
    if let Some(selection) = payload
        .get("next_actor_selection")
        .or_else(|| payload.get("nextActorSelection"))
    {
        return serde_json::json!({
            "selectionKind": selection.get("selection_kind").or_else(|| selection.get("selectionKind")),
            "selectedActorId": selection.get("selected_actor_id").or_else(|| selection.get("selectedActorId")),
        });
    }
    let keys = payload
        .as_object()
        .map(|object| object.keys().take(8).cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    serde_json::json!({ "keys": keys })
}

fn cultcache_status(store_path: &Path) -> Value {
    let metadata = store_path.metadata().ok();
    let modified_at = metadata
        .as_ref()
        .and_then(|meta| meta.modified().ok())
        .map(|time| {
            let time: chrono::DateTime<chrono::Utc> = time.into();
            time.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
        });
    serde_json::json!({
        "storeFile": store_path,
        "present": metadata.is_some(),
        "sizeBytes": metadata.as_ref().map(|meta| meta.len()),
        "modifiedAt": modified_at,
        "entryType": HEARTBEAT_STATE_TYPE,
        "entryKey": HEARTBEAT_STATE_KEY,
    })
}

fn participant_status_json(participant: &HeartbeatParticipant) -> Value {
    serde_json::json!({
        "agentId": participant.agent_id,
        "roleId": participant.role_id,
        "displayName": participant.display_name,
        "arena": participant_arena(participant),
        "participantKind": participant_kind(participant),
        "initiativeSpeed": participant.initiative_speed,
        "initiativeFrozen": participant
            .pending_turn
            .as_ref()
            .is_some_and(|turn| turn.status == "running"),
        "initiativeFreezeReason": participant
            .pending_turn
            .as_ref()
            .and_then(|turn| turn.initiative_freeze_reason.as_ref()),
        "nextReadyAt": participant.next_ready_at,
        "reactionBias": participant.reaction_bias,
        "interruptThreshold": participant.interrupt_threshold,
        "currentLoad": participant.current_load,
        "status": participant.status,
        "lastActionId": participant.last_action_id,
        "lastWokeAt": participant.last_woke_at,
        "lastFinishedAt": participant.last_finished_at,
        "pendingTurn": participant.pending_turn.as_ref().map(pending_turn_json),
    })
}

pub(super) fn schedule_participant_json(participant: &HeartbeatParticipant) -> Value {
    serde_json::json!({
        "agent_id": participant.agent_id,
        "role_id": participant.role_id,
        "display_name": participant.display_name,
        "arena": participant_arena(participant),
        "participant_kind": participant_kind(participant),
        "initiative_speed": participant.initiative_speed,
        "initiative_frozen": participant
            .pending_turn
            .as_ref()
            .is_some_and(|turn| turn.status == "running"),
        "initiative_freeze_reason": participant
            .pending_turn
            .as_ref()
            .and_then(|turn| turn.initiative_freeze_reason.as_ref()),
        "next_ready_at": participant.next_ready_at,
        "reaction_bias": participant.reaction_bias,
        "interrupt_threshold": participant.interrupt_threshold,
        "current_load": participant.current_load,
        "status": participant.status,
        "pending_turn": participant.pending_turn.as_ref().map(pending_turn_json),
        "constraints": participant.constraints,
    })
}

pub(super) fn selection_policy_json(policy: &HeartbeatSelectionPolicy) -> Value {
    serde_json::json!({
        "mode": policy.mode,
        "reaction_precedence": policy.reaction_precedence,
        "minimum_speed": policy.minimum_speed,
        "tie_breakers": policy.tie_breakers,
    })
}

pub(super) fn pending_turn_json(turn: &HeartbeatPendingTurn) -> Value {
    serde_json::json!({
        "status": turn.status,
        "scheduleId": turn.schedule_id,
        "actionId": turn.action_id,
        "actionType": turn.action_type,
        "actionScale": turn.action_scale,
        "localAffordanceBasis": turn.local_affordance_basis,
        "startedAt": turn.started_at,
        "startedSceneClock": turn.started_scene_clock,
        "baseRecovery": turn.base_recovery,
        "initiativeFrozen": turn.initiative_frozen,
        "initiativeFreezeReason": turn.initiative_freeze_reason.as_ref(),
        "recovery": turn.recovery,
        "cooldownPolicy": turn.cooldown_policy,
        "completedAt": turn.completed_at,
        "completedSceneClock": turn.completed_scene_clock,
        "nextReadyAt": turn.next_ready_at,
    })
}

pub(super) fn history_event_json(event: HeartbeatHistoryEvent) -> Value {
    serde_json::json!({
        "ts": event.ts,
        "scheduleId": event.schedule_id,
        "selectedRole": event.selected_role,
        "selectedAgentId": event.selected_agent_id,
        "actionId": event.action_id,
        "actionType": event.action_type,
        "arena": event.arena,
        "participantKind": event.participant_kind,
        "actionScale": event.action_scale,
        "coordinatorAction": event.coordinator_action,
        "targetRole": event.target_role,
        "workRole": event.work_role,
        "sceneClock": event.scene_clock,
        "nextReadyAt": event.next_ready_at,
        "turnStatus": event.turn_status,
        "cooldownStartedAfterCompletion": event.cooldown_started_after_completion,
    })
}
