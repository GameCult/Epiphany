use super::HEARTBEAT_COGNITION_KEY;
use super::HEARTBEAT_COGNITION_TYPE;
use super::HEARTBEAT_STATE_KEY;
use super::HEARTBEAT_STATE_TYPE;
use super::HEARTBEAT_STATUS_SCHEMA_VERSION;
use super::HeartbeatHistoryEvent;
use super::HeartbeatParticipant;
use super::HeartbeatPendingTurn;
use super::HeartbeatSelectionPolicy;
use super::VOID_ROUTINE_SCHEMA_VERSION;
use super::effective_cooldown_multiplier;
use super::load_heartbeat_cognition_entry;
use super::load_heartbeat_state_entry;
use super::mood_cooldown_multiplier;
use super::participant_arena;
use super::participant_kind;
use super::personality_cooldown_multiplier;
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
            "availableActions": ["init", "tick", "pump", "complete", "status"],
        }));
    };
    let cognition = load_heartbeat_cognition_entry(store_path)?;
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
    let adaptive_pacing = state
        .adaptive_pacing
        .as_ref()
        .and_then(|pacing| serde_json::to_value(pacing).ok())
        .or_else(|| state.extra.get("adaptivePacing").cloned());
    Ok(serde_json::json!({
        "schema_version": HEARTBEAT_STATUS_SCHEMA_VERSION,
        "ok": true,
        "status": "ready",
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
        "cognitionQuarantine": cognition.as_ref().map(heartbeat_cognition_status_json),
        "sleepCycle": cognition.as_ref().and_then(|entry| entry.sleep_cycle.clone()),
        "memoryResonance": cognition.as_ref().and_then(|entry| entry.memory_resonance.clone()),
        "incubation": cognition.as_ref().and_then(|entry| entry.incubation.clone()),
        "thoughtLanes": cognition.as_ref().and_then(|entry| entry.thought_lanes.clone()),
        "bridge": cognition.as_ref().and_then(|entry| entry.bridge.clone()),
        "candidateInterventions": cognition.as_ref().and_then(|entry| entry.candidate_interventions.clone()),
        "appraisals": cognition.as_ref().and_then(|entry| entry.appraisals.clone()),
        "reactions": cognition.as_ref().and_then(|entry| entry.reactions.clone()),
        "adaptivePacing": adaptive_pacing,
        "availableActions": ["init", "tick", "pump", "complete", "status", "routine"],
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
    if payload.get("schema_version")
        == Some(&Value::String(VOID_ROUTINE_SCHEMA_VERSION.to_string()))
    {
        return serde_json::json!({
            "runId": payload.get("runId"),
            "isNapping": payload.pointer("/sleepCycle/isNapping"),
            "resonanceCount": payload.pointer("/memoryResonance/pairs").and_then(Value::as_array).map(Vec::len),
            "incubationCount": payload.pointer("/incubation/themes").and_then(Value::as_array).map(Vec::len),
            "appraisalCount": payload.pointer("/appraisals/participantAppraisals").and_then(Value::as_array).map(Vec::len),
            "reactionCount": payload.pointer("/reactions/reactions").and_then(Value::as_array).map(Vec::len),
        });
    }
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
        "cognitionEntryType": HEARTBEAT_COGNITION_TYPE,
        "cognitionEntryKey": HEARTBEAT_COGNITION_KEY,
    })
}

fn heartbeat_cognition_status_json(cognition: &super::EpiphanyHeartbeatCognitionEntry) -> Value {
    serde_json::json!({
        "schema_version": cognition.schema_version,
        "updatedAt": cognition.updated_at,
        "latestRunId": cognition.latest_run_id,
        "latestArtifactRef": cognition.latest_artifact_ref,
        "source": cognition.source,
        "contract": "Experimental Void/Ghostlight thought-weather is quarantined outside stable heartbeat scheduling state. Status may project it for inspection, but scheduler policy owns only typed timing physiology.",
    })
}

fn participant_status_json(participant: &HeartbeatParticipant) -> Value {
    let personality_timing = participant
        .personality_timing
        .as_ref()
        .and_then(|timing| serde_json::to_value(timing).ok())
        .or_else(|| participant.extra.get("personalityTiming").cloned());
    let mood_timing = participant
        .mood_timing
        .as_ref()
        .and_then(|timing| serde_json::to_value(timing).ok())
        .or_else(|| participant.extra.get("moodTiming").cloned());
    serde_json::json!({
        "agentId": participant.agent_id,
        "roleId": participant.role_id,
        "displayName": participant.display_name,
        "arena": participant_arena(participant),
        "participantKind": participant_kind(participant),
        "sceneId": participant.scene_id.as_deref().or_else(|| participant.extra.get("sceneId").and_then(Value::as_str)),
        "initiativeSpeed": participant.initiative_speed,
        "personalityCooldownMultiplier": personality_cooldown_multiplier(participant),
        "moodCooldownMultiplier": mood_cooldown_multiplier(participant),
        "effectiveCooldownMultiplier": effective_cooldown_multiplier(participant),
        "personalityTiming": personality_timing,
        "moodTiming": mood_timing,
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
        "personality_cooldown_multiplier": personality_cooldown_multiplier(participant),
        "mood_cooldown_multiplier": mood_cooldown_multiplier(participant),
        "effective_cooldown_multiplier": effective_cooldown_multiplier(participant),
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
        "personalityCooldownMultiplier": pending_turn_typed_or_extra(turn, "personalityCooldownMultiplier", turn.personality_cooldown_multiplier),
        "moodCooldownMultiplier": pending_turn_typed_or_extra(turn, "moodCooldownMultiplier", turn.mood_cooldown_multiplier),
        "effectiveCooldownMultiplier": pending_turn_typed_or_extra(turn, "effectiveCooldownMultiplier", turn.effective_cooldown_multiplier),
        "recovery": turn.recovery,
        "cooldownPolicy": turn.cooldown_policy,
        "completedAt": turn.completed_at,
        "completedSceneClock": turn.completed_scene_clock,
        "nextReadyAt": turn.next_ready_at,
    })
}

fn pending_turn_typed_or_extra(
    turn: &HeartbeatPendingTurn,
    legacy_key: &str,
    typed: Option<f64>,
) -> Value {
    typed
        .map(|value| serde_json::json!(value))
        .or_else(|| turn.extra.get(legacy_key).cloned())
        .unwrap_or(Value::Null)
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
